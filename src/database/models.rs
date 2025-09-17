use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::net::Ipv4Addr;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use ipnetwork::IpNetwork;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DhcpSubnet {
    pub id: Uuid,
    pub name: String,
    pub network: IpNetwork,
    pub start_ip: Ipv4Addr,
    pub end_ip: Ipv4Addr,
    pub gateway: Ipv4Addr,
    #[sqlx(json)]
    pub dns_servers: Vec<Ipv4Addr>,
    pub domain_name: Option<String>,
    pub lease_duration: i32,
    pub vlan_id: Option<i32>,
    pub ipv6_prefix: Option<IpNetwork>,
    pub enabled: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DhcpLease {
    pub id: Uuid,
    pub subnet_id: Uuid,
    pub mac_address: Vec<u8>,
    pub ip_address: Ipv4Addr,
    pub hostname: Option<String>,
    pub lease_start: DateTime<Utc>,
    pub lease_end: DateTime<Utc>,
    pub state: String,
    pub client_identifier: Option<String>,
    pub vendor_class: Option<String>,
    pub user_class: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DhcpReservation {
    pub id: Uuid,
    pub subnet_id: Uuid,
    pub mac_address: Vec<u8>,
    pub ip_address: Ipv4Addr,
    pub hostname: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DnsZone {
    pub id: Uuid,
    pub name: String,
    pub zone_type: String,
    pub serial_number: i64,
    pub refresh_interval: i32,
    pub retry_interval: i32,
    pub expire_interval: i32,
    pub minimum_ttl: i32,
    pub primary_ns: Option<String>,
    pub admin_email: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DnsRecord {
    pub id: Uuid,
    pub zone_id: Uuid,
    pub name: String,
    pub record_type: String,
    pub value: String,
    pub ttl: i32,
    pub priority: Option<i32>,
    pub weight: Option<i32>,
    pub port: Option<i32>,
    pub is_dynamic: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SubnetStats {
    pub subnet_id: Uuid,
    pub subnet_name: String,
    pub total_addresses: u32,
    pub active_leases: u32,
    pub reserved_addresses: u32,
    pub available_addresses: u32,
    pub utilization_percent: f32,
}

impl DhcpLease {
    pub fn is_expired(&self) -> bool {
        self.lease_end < Utc::now()
    }

    pub fn is_active(&self) -> bool {
        self.state == "active" && !self.is_expired()
    }

    pub fn mac_address_string(&self) -> String {
        self.mac_address
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(":")
    }
}

impl DhcpSubnet {
    pub fn total_addresses(&self) -> u32 {
        let start = u32::from(self.start_ip);
        let end = u32::from(self.end_ip);
        end - start + 1
    }

    pub fn contains_ip(&self, ip: Ipv4Addr) -> bool {
        ip >= self.start_ip && ip <= self.end_ip
    }
}