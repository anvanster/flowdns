use std::net::Ipv6Addr;
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use anyhow::Result;
use tracing::{info, debug, warn};
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct DelegatedPrefix {
    pub id: Uuid,
    pub client_duid: Vec<u8>,
    pub iaid: u32,
    pub prefix: Ipv6Addr,
    pub prefix_length: u8,
    pub delegated_length: u8,  // Length delegated to client
    pub valid_lifetime: u32,
    pub preferred_lifetime: u32,
    pub lease_start: DateTime<Utc>,
    pub lease_end: DateTime<Utc>,
    pub state: PrefixState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrefixState {
    Available,
    Delegated,
    Reserved,
    Expired,
}

#[derive(Debug, Clone)]
pub struct PrefixPool {
    pub id: Uuid,
    pub name: String,
    pub prefix: Ipv6Addr,
    pub prefix_length: u8,
    pub delegation_length: u8,  // Size of prefixes to delegate
    pub total_prefixes: u32,
    pub available_prefixes: u32,
}

pub struct PrefixDelegationManager {
    db: PgPool,
    pools: HashMap<Uuid, PrefixPool>,
}

impl PrefixDelegationManager {
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            pools: HashMap::new(),
        }
    }
    
    pub async fn init_pools(&mut self) -> Result<()> {
        // Load prefix pools from database
        let rows = sqlx::query(
            r#"
            SELECT id, name, prefix, prefix_length, delegation_length
            FROM ipv6_prefix_pools
            WHERE enabled = true
            "#
        )
        .fetch_all(&self.db)
        .await?;
        
        for row in rows {
            // Parse and add pools - simplified
            info!("Loaded prefix pool from database");
        }
        
        // Add default pool if none exist
        if self.pools.is_empty() {
            self.add_default_pool().await?;
        }
        
        Ok(())
    }
    
    async fn add_default_pool(&mut self) -> Result<()> {
        let pool = PrefixPool {
            id: Uuid::new_v4(),
            name: "default".to_string(),
            prefix: Ipv6Addr::new(0x2001, 0xdb8, 0x1000, 0, 0, 0, 0, 0),
            prefix_length: 48,
            delegation_length: 56,  // Delegate /56 prefixes
            total_prefixes: 256,
            available_prefixes: 256,
        };
        
        self.pools.insert(pool.id, pool.clone());
        
        // Store in database
        sqlx::query(
            r#"
            INSERT INTO ipv6_prefix_pools 
                (id, name, prefix, prefix_length, delegation_length, enabled)
            VALUES ($1, $2, $3, $4, $5, true)
            ON CONFLICT (name) DO NOTHING
            "#
        )
        .bind(&pool.id)
        .bind(&pool.name)
        .bind(pool.prefix.to_string())
        .bind(pool.prefix_length as i32)
        .bind(pool.delegation_length as i32)
        .execute(&self.db)
        .await?;
        
        info!("Added default prefix pool: {}/{}", pool.prefix, pool.prefix_length);
        
        Ok(())
    }
    
    pub async fn request_prefix(
        &self,
        client_duid: Vec<u8>,
        iaid: u32,
        requested_length: Option<u8>,
        lifetime_hint: Option<u32>,
    ) -> Result<DelegatedPrefix> {
        // Check for existing delegation
        if let Ok(existing) = self.get_existing_delegation(&client_duid, iaid).await {
            if existing.state == PrefixState::Delegated {
                info!("Renewing existing prefix delegation for client");
                return Ok(existing);
            }
        }
        
        // Find available prefix from pool
        let prefix = self.allocate_prefix(requested_length).await?;
        
        // Calculate lifetimes
        let valid_lifetime = lifetime_hint.unwrap_or(86400);  // 24 hours default
        let preferred_lifetime = valid_lifetime * 3 / 4;
        let lease_start = Utc::now();
        let lease_end = lease_start + Duration::seconds(valid_lifetime as i64);
        
        let delegation = DelegatedPrefix {
            id: Uuid::new_v4(),
            client_duid: client_duid.clone(),
            iaid,
            prefix: prefix.0,
            prefix_length: prefix.1,
            delegated_length: prefix.1,
            valid_lifetime,
            preferred_lifetime,
            lease_start,
            lease_end,
            state: PrefixState::Delegated,
        };
        
        // Store in database
        self.store_delegation(&delegation).await?;
        
        info!(
            "Delegated prefix {}/{} to client DUID {:?}",
            delegation.prefix,
            delegation.prefix_length,
            client_duid
        );
        
        Ok(delegation)
    }
    
    async fn get_existing_delegation(
        &self,
        client_duid: &[u8],
        iaid: u32,
    ) -> Result<DelegatedPrefix> {
        let row = sqlx::query(
            r#"
            SELECT id, prefix, prefix_length, delegated_length,
                   valid_lifetime, preferred_lifetime, lease_start, lease_end, state
            FROM ipv6_delegated_prefixes
            WHERE client_duid = $1 AND iaid = $2 AND state = 'delegated'
            ORDER BY lease_end DESC
            LIMIT 1
            "#
        )
        .bind(client_duid)
        .bind(iaid)
        .fetch_optional(&self.db)
        .await?;
        
        match row {
            Some(_row) => {
                // Parse and return delegation - simplified
                Err(anyhow::anyhow!("No existing delegation found"))
            }
            None => Err(anyhow::anyhow!("No existing delegation found")),
        }
    }
    
    async fn allocate_prefix(
        &self,
        requested_length: Option<u8>,
    ) -> Result<(Ipv6Addr, u8)> {
        // Find first available pool
        let pool = self.pools.values()
            .find(|p| p.available_prefixes > 0)
            .ok_or_else(|| anyhow::anyhow!("No prefixes available"))?;
        
        let delegation_length = requested_length.unwrap_or(pool.delegation_length);
        
        // Calculate next available prefix
        // This is simplified - real implementation would track allocated prefixes
        let prefix_num = (pool.total_prefixes - pool.available_prefixes) as u128;
        let prefix_shift = 128 - delegation_length;
        
        let base_addr = u128::from_be_bytes(pool.prefix.octets());
        let delegated_addr = base_addr | (prefix_num << prefix_shift);
        
        let prefix = Ipv6Addr::from(delegated_addr.to_be_bytes());
        
        Ok((prefix, delegation_length))
    }
    
    async fn store_delegation(&self, delegation: &DelegatedPrefix) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO ipv6_delegated_prefixes
                (id, client_duid, iaid, prefix, prefix_length, delegated_length,
                 valid_lifetime, preferred_lifetime, lease_start, lease_end, state)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (client_duid, iaid) 
            DO UPDATE SET 
                prefix = $4,
                prefix_length = $5,
                delegated_length = $6,
                valid_lifetime = $7,
                preferred_lifetime = $8,
                lease_start = $9,
                lease_end = $10,
                state = $11
            "#
        )
        .bind(&delegation.id)
        .bind(&delegation.client_duid)
        .bind(delegation.iaid)
        .bind(delegation.prefix.to_string())
        .bind(delegation.prefix_length as i32)
        .bind(delegation.delegated_length as i32)
        .bind(delegation.valid_lifetime as i32)
        .bind(delegation.preferred_lifetime as i32)
        .bind(&delegation.lease_start)
        .bind(&delegation.lease_end)
        .bind("delegated")
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    pub async fn release_prefix(
        &self,
        client_duid: &[u8],
        iaid: u32,
        prefix: &Ipv6Addr,
    ) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE ipv6_delegated_prefixes
            SET state = 'available', lease_end = NOW()
            WHERE client_duid = $1 AND iaid = $2 AND prefix = $3
            "#
        )
        .bind(client_duid)
        .bind(iaid)
        .bind(prefix.to_string())
        .execute(&self.db)
        .await?;
        
        if result.rows_affected() > 0 {
            info!("Released prefix {}/{} from client", prefix, iaid);
        } else {
            warn!("Attempted to release unknown prefix {}", prefix);
        }
        
        Ok(())
    }
    
    pub async fn cleanup_expired(&self) -> Result<u64> {
        let result = sqlx::query(
            r#"
            UPDATE ipv6_delegated_prefixes
            SET state = 'expired'
            WHERE state = 'delegated' AND lease_end < NOW()
            "#
        )
        .execute(&self.db)
        .await?;
        
        let expired = result.rows_affected();
        if expired > 0 {
            info!("Marked {} delegated prefixes as expired", expired);
        }
        
        Ok(expired)
    }
    
    pub async fn reclaim_expired(&self, grace_period_hours: i64) -> Result<u64> {
        let cutoff = Utc::now() - Duration::hours(grace_period_hours);
        
        let result = sqlx::query(
            r#"
            UPDATE ipv6_delegated_prefixes
            SET state = 'available'
            WHERE state = 'expired' AND lease_end < $1
            "#
        )
        .bind(cutoff)
        .execute(&self.db)
        .await?;
        
        let reclaimed = result.rows_affected();
        if reclaimed > 0 {
            info!("Reclaimed {} expired prefixes", reclaimed);
        }
        
        Ok(reclaimed)
    }
    
    pub async fn get_statistics(&self) -> Result<PrefixStats> {
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) FILTER (WHERE state = 'delegated') as delegated,
                COUNT(*) FILTER (WHERE state = 'available') as available,
                COUNT(*) FILTER (WHERE state = 'reserved') as reserved,
                COUNT(*) FILTER (WHERE state = 'expired') as expired
            FROM ipv6_delegated_prefixes
            "#
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(PrefixStats {
            total_pools: self.pools.len(),
            delegated_prefixes: 0,  // Would parse from row
            available_prefixes: 0,
            reserved_prefixes: 0,
            expired_prefixes: 0,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PrefixStats {
    pub total_pools: usize,
    pub delegated_prefixes: u32,
    pub available_prefixes: u32,
    pub reserved_prefixes: u32,
    pub expired_prefixes: u32,
}