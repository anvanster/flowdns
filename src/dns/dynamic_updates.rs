use crate::dns::simple_zone_manager::SimpleZoneManager;
use std::sync::Arc;
use std::net::IpAddr;
use anyhow::{Result, anyhow};
use tracing::{info, warn, debug};

pub struct DynamicUpdater {
    zone_manager: Arc<SimpleZoneManager>,
}

impl DynamicUpdater {
    pub fn new(zone_manager: Arc<SimpleZoneManager>) -> Self {
        Self { zone_manager }
    }

    /// Add or update a DNS record when a DHCP lease is created or renewed
    pub async fn add_dhcp_record(
        &self,
        hostname: &str,
        ip: IpAddr,
        domain: &str,
        ttl: u32,
    ) -> Result<()> {
        if hostname.is_empty() {
            return Err(anyhow!("Hostname cannot be empty"));
        }

        // Create FQDN if not already
        let fqdn = if hostname.contains('.') {
            hostname.to_string()
        } else {
            format!("{}.{}", hostname, domain)
        };

        debug!("Adding dynamic DNS record: {} -> {}", fqdn, ip);

        // Add the A or AAAA record
        self.zone_manager
            .add_dynamic_record(domain, &fqdn, ip, ttl)
            .await?;

        info!("Successfully added DNS record: {} -> {}", fqdn, ip);
        Ok(())
    }

    /// Remove a DNS record when a DHCP lease expires or is released
    pub async fn remove_dhcp_record(&self, hostname: &str, domain: &str) -> Result<()> {
        if hostname.is_empty() {
            return Err(anyhow!("Hostname cannot be empty"));
        }

        let fqdn = if hostname.contains('.') {
            hostname.to_string()
        } else {
            format!("{}.{}", hostname, domain)
        };

        debug!("Removing dynamic DNS record: {}", fqdn);

        self.zone_manager
            .remove_dynamic_record(domain, &fqdn)
            .await?;

        info!("Successfully removed DNS record: {}", fqdn);
        Ok(())
    }

    /// Update DNS record when IP changes
    pub async fn update_dhcp_record(
        &self,
        hostname: &str,
        old_ip: IpAddr,
        new_ip: IpAddr,
        domain: &str,
        ttl: u32,
    ) -> Result<()> {
        if old_ip == new_ip {
            debug!("IP unchanged for {}, skipping update", hostname);
            return Ok(());
        }

        // Remove old record
        self.remove_dhcp_record(hostname, domain).await?;

        // Add new record
        self.add_dhcp_record(hostname, new_ip, domain, ttl).await?;

        info!("Updated DNS record: {} from {} to {}", hostname, old_ip, new_ip);
        Ok(())
    }

    /// Bulk update for multiple records (useful during startup)
    pub async fn sync_dhcp_records(
        &self,
        records: Vec<(String, IpAddr)>,
        domain: &str,
        ttl: u32,
    ) -> Result<()> {
        info!("Syncing {} DHCP records to DNS", records.len());

        let mut success_count = 0;
        let mut error_count = 0;

        for (hostname, ip) in records {
            match self.add_dhcp_record(&hostname, ip, domain, ttl).await {
                Ok(_) => success_count += 1,
                Err(e) => {
                    warn!("Failed to sync record {} -> {}: {}", hostname, ip, e);
                    error_count += 1;
                }
            }
        }

        info!(
            "DNS sync completed: {} successful, {} failed",
            success_count, error_count
        );

        if error_count > 0 {
            Err(anyhow!("Some records failed to sync"))
        } else {
            Ok(())
        }
    }
}

/// Integration point for DHCP server to update DNS
pub struct DhcpDnsIntegration {
    updater: Arc<DynamicUpdater>,
    default_domain: String,
    default_ttl: u32,
}

impl DhcpDnsIntegration {
    pub fn new(zone_manager: Arc<SimpleZoneManager>, default_domain: String, default_ttl: u32) -> Self {
        Self {
            updater: Arc::new(DynamicUpdater::new(zone_manager)),
            default_domain,
            default_ttl,
        }
    }

    pub async fn on_lease_created(
        &self,
        hostname: Option<String>,
        ip: IpAddr,
    ) -> Result<()> {
        if let Some(hostname) = hostname {
            self.updater
                .add_dhcp_record(&hostname, ip, &self.default_domain, self.default_ttl)
                .await?;
        }
        Ok(())
    }

    pub async fn on_lease_renewed(
        &self,
        hostname: Option<String>,
        ip: IpAddr,
    ) -> Result<()> {
        // Same as created for now, but could have different logic
        self.on_lease_created(hostname, ip).await
    }

    pub async fn on_lease_released(
        &self,
        hostname: Option<String>,
    ) -> Result<()> {
        if let Some(hostname) = hostname {
            self.updater
                .remove_dhcp_record(&hostname, &self.default_domain)
                .await?;
        }
        Ok(())
    }

    pub async fn on_lease_expired(
        &self,
        hostname: Option<String>,
    ) -> Result<()> {
        // Same as released
        self.on_lease_released(hostname).await
    }
}