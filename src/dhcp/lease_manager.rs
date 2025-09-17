use crate::database::models::{DhcpSubnet, DhcpLease, DhcpReservation};
use crate::config::Settings;
use sqlx::PgPool;
use std::net::Ipv4Addr;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{Utc, Duration};
use anyhow::{Result, anyhow};
use tracing::{info, warn, debug};

pub struct LeaseManager {
    db: PgPool,
    subnets: Arc<RwLock<HashMap<Uuid, DhcpSubnet>>>,
    settings: Arc<Settings>,
}

impl LeaseManager {
    pub async fn new(db: PgPool, settings: Arc<Settings>) -> Result<Self> {
        let mut manager = Self {
            db,
            subnets: Arc::new(RwLock::new(HashMap::new())),
            settings,
        };

        manager.load_subnets().await?;
        Ok(manager)
    }

    async fn load_subnets(&mut self) -> Result<()> {
        use super::lease_manager_queries;

        let subnets = lease_manager_queries::fetch_all_subnets(&self.db).await?;

        let mut subnet_map = self.subnets.write().await;
        for subnet in subnets {
            subnet_map.insert(subnet.id, subnet);
        }
        info!("Loaded {} subnets", subnet_map.len());

        Ok(())
    }

    pub async fn find_subnet_for_client(
        &self,
        client_ip: Ipv4Addr,
        relay_agent_ip: Option<Ipv4Addr>
    ) -> Option<DhcpSubnet> {
        let target_ip = relay_agent_ip.unwrap_or(client_ip);
        let subnets = self.subnets.read().await;

        for subnet in subnets.values() {
            if subnet.network.contains(std::net::IpAddr::V4(target_ip)) {
                return Some(subnet.clone());
            }
        }

        None
    }

    pub async fn find_available_ip(
        &self,
        subnet_id: Uuid,
        mac_address: &[u8]
    ) -> Result<Option<Ipv4Addr>> {
        let subnets = self.subnets.read().await;
        let subnet = subnets.get(&subnet_id)
            .ok_or_else(|| anyhow!("Subnet not found: {}", subnet_id))?;

        // Check for existing reservation
        if let Some(reservation) = self.get_reservation(subnet_id, mac_address).await? {
            debug!("Found reservation for MAC {}: {}",
                   format_mac(mac_address), reservation.ip_address);
            return Ok(Some(reservation.ip_address));
        }

        // Check for existing active lease
        if let Some(lease) = self.get_active_lease_by_mac(mac_address).await? {
            if lease.subnet_id == subnet_id {
                debug!("Found existing lease for MAC {}: {}",
                       format_mac(mac_address), lease.ip_address);
                return Ok(Some(lease.ip_address));
            }
        }

        // Find next available IP in range
        let start = u32::from(subnet.start_ip);
        let end = u32::from(subnet.end_ip);

        for ip_num in start..=end {
            let ip = Ipv4Addr::from(ip_num);

            // Skip network and broadcast addresses
            let network = subnet.network.ip();
            let broadcast = subnet.network.broadcast();
            if std::net::IpAddr::V4(ip) == network || std::net::IpAddr::V4(ip) == broadcast {
                continue;
            }

            // Check if IP is available
            if !self.is_ip_in_use(subnet_id, ip).await? {
                debug!("Found available IP: {}", ip);
                return Ok(Some(ip));
            }
        }

        warn!("No available IPs in subnet {}", subnet.name);
        Ok(None)
    }

    async fn is_ip_in_use(&self, subnet_id: Uuid, ip: Ipv4Addr) -> Result<bool> {
        use super::lease_manager_queries;

        let lease_count = lease_manager_queries::count_active_leases(&self.db, subnet_id, ip).await?;
        if lease_count > 0 {
            return Ok(true);
        }

        let reservation_count = lease_manager_queries::count_reservations(&self.db, subnet_id, ip).await?;
        Ok(reservation_count > 0)
    }

    pub async fn create_lease(
        &self,
        subnet_id: Uuid,
        mac_address: &[u8],
        ip_address: Ipv4Addr,
        hostname: Option<String>
    ) -> Result<DhcpLease> {
        use super::lease_manager_queries;

        let subnets = self.subnets.read().await;
        let subnet = subnets.get(&subnet_id)
            .ok_or_else(|| anyhow!("Subnet not found"))?;

        let lease_start = Utc::now();
        let lease_end = lease_start + Duration::seconds(subnet.lease_duration as i64);

        let final_hostname = hostname.or_else(|| {
            self.generate_hostname(ip_address)
        });

        let lease = lease_manager_queries::insert_or_update_lease(
            &self.db,
            subnet_id,
            mac_address,
            ip_address,
            final_hostname,
            lease_start,
            lease_end,
        )
        .await?;

        info!("Created lease: MAC {} -> IP {} (expires: {})",
             format_mac(mac_address), ip_address, lease_end);

        Ok(lease)
    }

    pub async fn renew_lease(
        &self,
        mac_address: &[u8],
        requested_ip: Ipv4Addr
    ) -> Result<Option<DhcpLease>> {
        use super::lease_manager_queries;

        let existing_lease = lease_manager_queries::find_active_lease_by_mac_and_ip(
            &self.db,
            mac_address,
            requested_ip,
        )
        .await?;

        if let Some(lease) = existing_lease {
            let subnets = self.subnets.read().await;
            let subnet = subnets.get(&lease.subnet_id)
                .ok_or_else(|| anyhow!("Subnet not found"))?;

            let new_lease_end = Utc::now() + Duration::seconds(subnet.lease_duration as i64);

            let renewed_lease = lease_manager_queries::update_lease_end(
                &self.db,
                lease.id,
                new_lease_end,
            )
            .await?;

            info!("Renewed lease: MAC {} -> IP {} (new expiry: {})",
                 format_mac(mac_address), requested_ip, new_lease_end);

            return Ok(Some(renewed_lease));
        }

        Ok(None)
    }

    pub async fn release_lease(
        &self,
        mac_address: &[u8],
        ip_address: Ipv4Addr
    ) -> Result<bool> {
        use super::lease_manager_queries;

        let released = lease_manager_queries::release_lease(
            &self.db,
            mac_address,
            ip_address,
        )
        .await?;

        if released {
            info!("Released lease: MAC {} -> IP {}",
                 format_mac(mac_address), ip_address);
        }

        Ok(released)
    }

    async fn get_reservation(
        &self,
        subnet_id: Uuid,
        mac_address: &[u8]
    ) -> Result<Option<DhcpReservation>> {
        use super::lease_manager_queries;

        lease_manager_queries::get_reservation(
            &self.db,
            subnet_id,
            mac_address,
        )
        .await
    }

    async fn get_active_lease_by_mac(
        &self,
        mac_address: &[u8]
    ) -> Result<Option<DhcpLease>> {
        use super::lease_manager_queries;

        lease_manager_queries::get_active_lease_by_mac(
            &self.db,
            mac_address,
        )
        .await
    }

    fn generate_hostname(&self, ip: Ipv4Addr) -> Option<String> {
        let template = &self.settings.dns.hostname_template;
        if template.is_empty() {
            return None;
        }

        let hostname = template.replace("{ip}", &ip.to_string())
            .replace("{ip_dash}", &ip.to_string().replace('.', "-"))
            .replace("{ip_last}", &ip.octets()[3].to_string());

        Some(hostname)
    }

    pub async fn cleanup_expired_leases(&self) -> Result<u64> {
        use super::lease_manager_queries;

        let count = lease_manager_queries::expire_old_leases(&self.db).await?;
        if count > 0 {
            info!("Cleaned up {} expired leases", count);
        }

        Ok(count)
    }
}

fn format_mac(mac: &[u8]) -> String {
    mac.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(":")
}