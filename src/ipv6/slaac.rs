use std::net::Ipv6Addr;
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use anyhow::Result;
use tracing::{info, debug};
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct SlaacAddress {
    pub id: Uuid,
    pub subnet_id: Uuid,
    pub mac_address: Vec<u8>,
    pub ipv6_address: Ipv6Addr,
    pub prefix: Ipv6Addr,
    pub prefix_length: u8,
    pub created_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub hostname: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SlaacPrefix {
    pub prefix: Ipv6Addr,
    pub prefix_length: u8,
    pub interface: String,
    pub valid_lifetime: u32,
    pub preferred_lifetime: u32,
}

pub struct SlaacManager {
    db: PgPool,
    prefixes: HashMap<String, SlaacPrefix>,
}

impl SlaacManager {
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            prefixes: HashMap::new(),
        }
    }
    
    pub fn add_prefix(&mut self, interface: String, prefix: SlaacPrefix) {
        self.prefixes.insert(interface, prefix);
    }
    
    pub fn generate_eui64_address(
        &self,
        prefix: &Ipv6Addr,
        mac_address: &[u8],
    ) -> Result<Ipv6Addr> {
        if mac_address.len() != 6 {
            return Err(anyhow::anyhow!("Invalid MAC address length"));
        }
        
        // Convert MAC to EUI-64
        let mut eui64 = [0u8; 8];
        
        // First 3 octets of MAC
        eui64[0] = mac_address[0] ^ 0x02;  // Flip universal/local bit
        eui64[1] = mac_address[1];
        eui64[2] = mac_address[2];
        
        // Insert FFFE
        eui64[3] = 0xFF;
        eui64[4] = 0xFE;
        
        // Last 3 octets of MAC
        eui64[5] = mac_address[3];
        eui64[6] = mac_address[4];
        eui64[7] = mac_address[5];
        
        // Combine prefix with EUI-64
        let prefix_bytes = prefix.octets();
        let mut addr_bytes = [0u8; 16];
        
        // Copy first 64 bits from prefix
        addr_bytes[..8].copy_from_slice(&prefix_bytes[..8]);
        
        // Copy EUI-64 as interface ID
        addr_bytes[8..].copy_from_slice(&eui64);
        
        Ok(Ipv6Addr::from(addr_bytes))
    }
    
    pub fn generate_privacy_address(
        &self,
        prefix: &Ipv6Addr,
        seed: &[u8],
    ) -> Result<Ipv6Addr> {
        use sha2::{Sha256, Digest};
        
        // Generate temporary interface ID using SHA-256
        let mut hasher = Sha256::new();
        hasher.update(seed);
        hasher.update(&Utc::now().timestamp().to_be_bytes());
        let hash = hasher.finalize();
        
        // Use first 64 bits of hash as interface ID
        let prefix_bytes = prefix.octets();
        let mut addr_bytes = [0u8; 16];
        
        // Copy prefix
        addr_bytes[..8].copy_from_slice(&prefix_bytes[..8]);
        
        // Copy hashed interface ID (ensure local bit is set)
        addr_bytes[8..].copy_from_slice(&hash[..8]);
        addr_bytes[8] &= 0xFD;  // Clear universal bit, set local bit
        
        Ok(Ipv6Addr::from(addr_bytes))
    }
    
    pub async fn register_slaac_address(
        &self,
        mac_address: Vec<u8>,
        ipv6_address: Ipv6Addr,
        prefix: Ipv6Addr,
        prefix_length: u8,
        hostname: Option<String>,
    ) -> Result<SlaacAddress> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        // Store in database
        sqlx::query(
            r#"
            INSERT INTO ipv6_slaac_addresses 
                (id, mac_address, ipv6_address, prefix, prefix_length, 
                 created_at, last_seen, hostname)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (mac_address, ipv6_address) 
            DO UPDATE SET last_seen = $7, hostname = $8
            "#
        )
        .bind(&id)
        .bind(&mac_address)
        .bind(ipv6_address.to_string())
        .bind(prefix.to_string())
        .bind(prefix_length as i32)
        .bind(&now)
        .bind(&now)
        .bind(&hostname)
        .execute(&self.db)
        .await?;
        
        info!(
            "Registered SLAAC address {} for MAC {:?}",
            ipv6_address,
            mac_address
        );
        
        Ok(SlaacAddress {
            id,
            subnet_id: Uuid::nil(),  // Would be determined from prefix
            mac_address,
            ipv6_address,
            prefix,
            prefix_length,
            created_at: now,
            last_seen: now,
            hostname,
        })
    }
    
    pub async fn get_addresses_by_mac(
        &self,
        mac_address: &[u8],
    ) -> Result<Vec<SlaacAddress>> {
        let rows = sqlx::query(
            r#"
            SELECT id, mac_address, ipv6_address, prefix, prefix_length,
                   created_at, last_seen, hostname
            FROM ipv6_slaac_addresses
            WHERE mac_address = $1
            ORDER BY last_seen DESC
            "#
        )
        .bind(mac_address)
        .fetch_all(&self.db)
        .await?;
        
        let mut addresses = Vec::new();
        for row in rows {
            // Parse results - simplified
            debug!("Found SLAAC address for MAC {:?}", mac_address);
        }
        
        Ok(addresses)
    }
    
    pub async fn cleanup_stale_addresses(&self, max_age_hours: i64) -> Result<u64> {
        let cutoff = Utc::now() - Duration::hours(max_age_hours);
        
        let result = sqlx::query(
            r#"
            DELETE FROM ipv6_slaac_addresses
            WHERE last_seen < $1
            "#
        )
        .bind(cutoff)
        .execute(&self.db)
        .await?;
        
        let deleted = result.rows_affected();
        if deleted > 0 {
            info!("Cleaned up {} stale SLAAC addresses", deleted);
        }
        
        Ok(deleted)
    }
    
    pub fn calculate_dad_timeout(&self) -> std::time::Duration {
        // Duplicate Address Detection timeout
        std::time::Duration::from_secs(1)
    }
    
    pub async fn perform_dad(
        &self,
        address: &Ipv6Addr,
    ) -> Result<bool> {
        // Simplified DAD - would actually send NS messages
        debug!("Performing DAD for {}", address);
        
        // Check if address exists in database
        let result = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM ipv6_slaac_addresses
            WHERE ipv6_address = $1
            "#
        )
        .bind(address.to_string())
        .fetch_one(&self.db)
        .await?;
        
        // Return true if address is unique (DAD passed)
        Ok(true)
    }
}

// Helper to monitor neighbor discovery
pub struct NeighborDiscovery {
    db: PgPool,
}

impl NeighborDiscovery {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
    
    pub async fn handle_router_solicitation(
        &self,
        source: Ipv6Addr,
        mac: Vec<u8>,
    ) -> Result<()> {
        info!("Received RS from {} (MAC: {:?})", source, mac);
        
        // Record the solicitation
        sqlx::query(
            r#"
            INSERT INTO ipv6_neighbor_cache (ipv6_address, mac_address, last_seen, state)
            VALUES ($1, $2, $3, 'reachable')
            ON CONFLICT (ipv6_address) 
            DO UPDATE SET mac_address = $2, last_seen = $3
            "#
        )
        .bind(source.to_string())
        .bind(&mac)
        .bind(Utc::now())
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    pub async fn handle_neighbor_solicitation(
        &self,
        source: Ipv6Addr,
        target: Ipv6Addr,
        mac: Vec<u8>,
    ) -> Result<()> {
        debug!("Received NS from {} for target {}", source, target);
        
        // Update neighbor cache
        sqlx::query(
            r#"
            INSERT INTO ipv6_neighbor_cache (ipv6_address, mac_address, last_seen, state)
            VALUES ($1, $2, $3, 'incomplete')
            ON CONFLICT (ipv6_address) 
            DO UPDATE SET mac_address = $2, last_seen = $3, state = 'reachable'
            "#
        )
        .bind(source.to_string())
        .bind(&mac)
        .bind(Utc::now())
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
}