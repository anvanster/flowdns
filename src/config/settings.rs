use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use std::collections::HashMap;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub dns: DnsConfig,
    pub dhcp: DhcpConfig,
    pub ipv6: IPv6Config,
    pub routing: RoutingConfig,
    pub api: ApiConfig,
    pub subnets: HashMap<String, SubnetConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub log_level: String,
    pub threads: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: u64,
    pub idle_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsConfig {
    pub enabled: bool,
    pub bind_address: String,
    pub port: u16,
    pub forward_servers: Vec<String>,
    pub domain_suffix: String,
    pub dynamic_updates: bool,
    pub hostname_template: String,
    pub ttl_default: u32,
    pub cache_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhcpConfig {
    pub enabled: bool,
    pub bind_address: String,
    pub port: u16,
    pub default_lease_time: u32,
    pub max_lease_time: u32,
    pub renewal_time: u32,
    pub rebind_time: u32,
    pub decline_time: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPv6Config {
    pub enabled: bool,
    pub radvd_config_path: String,
    pub prefix_length: u8,
    pub router_lifetime: u32,
    pub reachable_time: u32,
    pub retransmit_time: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    pub management_subnet: String,
    pub upstream_gateway: Ipv4Addr,
    pub enable_inter_subnet_routing: bool,
    pub nat_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub enabled: bool,
    pub bind_address: String,
    pub port: u16,
    pub cors_enabled: bool,
    pub cors_origins: Vec<String>,
    pub jwt_secret: String,
    pub jwt_expiry: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubnetConfig {
    pub network: String,
    pub start_ip: Ipv4Addr,
    pub end_ip: Ipv4Addr,
    pub gateway: Ipv4Addr,
    pub dns_servers: Vec<Ipv4Addr>,
    pub domain_name: String,
    pub lease_time: u32,
    pub ipv6_prefix: Option<String>,
    pub vlan_id: Option<u16>,
    pub description: String,
    pub enabled: bool,
}

impl Settings {
    pub fn load(config_path: &str) -> Result<Self> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name(config_path).required(false))
            .add_source(config::Environment::with_prefix("FLOWDNS").separator("__"))
            .build()?;

        Ok(settings.try_deserialize()?)
    }

    pub fn validate(&self) -> Result<()> {
        // Validate configuration
        if self.database.url.is_empty() {
            anyhow::bail!("Database URL is required");
        }

        if self.api.enabled && self.api.jwt_secret.len() < 32 {
            anyhow::bail!("JWT secret must be at least 32 characters");
        }

        for (name, subnet) in &self.subnets {
            let network: ipnetwork::IpNetwork = subnet.network.parse()?;

            if !network.contains(std::net::IpAddr::V4(subnet.start_ip)) {
                anyhow::bail!("Subnet {}: start_ip not in network", name);
            }

            if !network.contains(std::net::IpAddr::V4(subnet.end_ip)) {
                anyhow::bail!("Subnet {}: end_ip not in network", name);
            }

            if subnet.start_ip > subnet.end_ip {
                anyhow::bail!("Subnet {}: start_ip must be less than end_ip", name);
            }
        }

        Ok(())
    }
}