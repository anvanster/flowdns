use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::net::Ipv4Addr;

// Authentication models
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

// DHCP models
#[derive(Debug, Serialize, Deserialize)]
pub struct LeaseResponse {
    pub id: Uuid,
    pub subnet_id: Uuid,
    pub mac_address: String,
    pub ip_address: Ipv4Addr,
    pub hostname: Option<String>,
    pub lease_start: DateTime<Utc>,
    pub lease_end: DateTime<Utc>,
    pub state: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateLeaseRequest {
    pub subnet_id: Uuid,
    pub mac_address: String,
    pub ip_address: Option<Ipv4Addr>,
    pub hostname: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubnetResponse {
    pub id: Uuid,
    pub name: String,
    pub network: String,
    pub start_ip: Ipv4Addr,
    pub end_ip: Ipv4Addr,
    pub gateway: Ipv4Addr,
    pub dns_servers: Vec<Ipv4Addr>,
    pub domain_name: Option<String>,
    pub lease_duration: i32,
    pub vlan_id: Option<i32>,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateSubnetRequest {
    pub name: String,
    pub network: String,
    pub start_ip: Ipv4Addr,
    pub end_ip: Ipv4Addr,
    pub gateway: Ipv4Addr,
    pub dns_servers: Vec<Ipv4Addr>,
    pub domain_name: Option<String>,
    pub lease_duration: Option<i32>,
    pub vlan_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSubnetRequest {
    pub name: Option<String>,
    pub start_ip: Option<Ipv4Addr>,
    pub end_ip: Option<Ipv4Addr>,
    pub gateway: Option<Ipv4Addr>,
    pub dns_servers: Option<Vec<Ipv4Addr>>,
    pub domain_name: Option<String>,
    pub lease_duration: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReservationResponse {
    pub id: Uuid,
    pub subnet_id: Uuid,
    pub mac_address: String,
    pub ip_address: Ipv4Addr,
    pub hostname: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateReservationRequest {
    pub subnet_id: Uuid,
    pub mac_address: String,
    pub ip_address: Ipv4Addr,
    pub hostname: Option<String>,
    pub description: Option<String>,
}

// DNS models
#[derive(Debug, Serialize, Deserialize)]
pub struct ZoneResponse {
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

#[derive(Debug, Deserialize)]
pub struct CreateZoneRequest {
    pub name: String,
    pub zone_type: String,
    pub primary_ns: Option<String>,
    pub admin_email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateZoneRequest {
    pub primary_ns: Option<String>,
    pub admin_email: Option<String>,
    pub refresh_interval: Option<i32>,
    pub retry_interval: Option<i32>,
    pub expire_interval: Option<i32>,
    pub minimum_ttl: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordResponse {
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

#[derive(Debug, Deserialize)]
pub struct CreateRecordRequest {
    pub name: String,
    pub record_type: String,
    pub value: String,
    pub ttl: Option<i32>,
    pub priority: Option<i32>,
    pub weight: Option<i32>,
    pub port: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRecordRequest {
    pub value: Option<String>,
    pub ttl: Option<i32>,
    pub priority: Option<i32>,
    pub weight: Option<i32>,
    pub port: Option<i32>,
}

// System models
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub database: String,
    pub dhcp_server: String,
    pub dns_server: String,
    pub api_server: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct MetricsResponse {
    pub dhcp: DhcpMetrics,
    pub dns: DnsMetrics,
    pub system: SystemMetrics,
}

#[derive(Debug, Serialize)]
pub struct DhcpMetrics {
    pub total_subnets: i64,
    pub active_leases: i64,
    pub expired_leases: i64,
    pub reserved_addresses: i64,
    pub available_addresses: i64,
}

#[derive(Debug, Serialize)]
pub struct DnsMetrics {
    pub total_zones: i64,
    pub total_records: i64,
    pub dynamic_records: i64,
}

#[derive(Debug, Serialize)]
pub struct SystemMetrics {
    pub uptime_seconds: i64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
}

// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub status_code: u16,
}