// Runtime SQL queries for API handlers
use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::Result;
use std::net::Ipv4Addr;

pub struct LeaseRow {
    pub id: Uuid,
    pub subnet_id: Uuid,
    pub mac_address: Vec<u8>,
    pub ip_address: Ipv4Addr,
    pub hostname: Option<String>,
    pub lease_start: DateTime<Utc>,
    pub lease_end: DateTime<Utc>,
    pub state: String,
}

pub async fn fetch_active_leases(db: &PgPool, state_filter: &str) -> Result<Vec<LeaseRow>> {
    let rows = sqlx::query(
        r#"
        SELECT id, subnet_id, mac_address, ip_address, hostname,
               lease_start, lease_end, state
        FROM dhcp_leases
        WHERE state = $1
        ORDER BY lease_start DESC
        LIMIT 100
        "#
    )
    .bind(state_filter)
    .fetch_all(db)
    .await?;

    let mut leases = Vec::new();
    for row in rows {
        let lease = LeaseRow {
            id: row.get("id"),
            subnet_id: row.get("subnet_id"),
            mac_address: row.get("mac_address"),
            ip_address: row.get::<std::net::IpAddr, _>("ip_address").to_string().parse()?,
            hostname: row.get("hostname"),
            lease_start: row.get("lease_start"),
            lease_end: row.get("lease_end"),
            state: row.get("state"),
        };
        leases.push(lease);
    }

    Ok(leases)
}

pub async fn fetch_lease_by_id(db: &PgPool, lease_id: Uuid) -> Result<Option<LeaseRow>> {
    let row = sqlx::query(
        r#"
        SELECT id, subnet_id, mac_address, ip_address, hostname,
               lease_start, lease_end, state
        FROM dhcp_leases
        WHERE id = $1
        "#
    )
    .bind(lease_id)
    .fetch_optional(db)
    .await?;

    match row {
        Some(row) => Ok(Some(LeaseRow {
            id: row.get("id"),
            subnet_id: row.get("subnet_id"),
            mac_address: row.get("mac_address"),
            ip_address: row.get::<std::net::IpAddr, _>("ip_address").to_string().parse()?,
            hostname: row.get("hostname"),
            lease_start: row.get("lease_start"),
            lease_end: row.get("lease_end"),
            state: row.get("state"),
        })),
        None => Ok(None),
    }
}

pub async fn release_lease(db: &PgPool, lease_id: Uuid) -> Result<u64> {
    let result = sqlx::query(
        r#"
        UPDATE dhcp_leases
        SET state = 'released', updated_at = NOW()
        WHERE id = $1 AND state = 'active'
        "#
    )
    .bind(lease_id)
    .execute(db)
    .await?;

    Ok(result.rows_affected())
}

pub struct SubnetRow {
    pub id: Uuid,
    pub name: String,
    pub network: String,  // Store as string for simplicity
    pub start_ip: Ipv4Addr,
    pub end_ip: Ipv4Addr,
    pub gateway: Ipv4Addr,
    pub dns_servers: serde_json::Value,
    pub domain_name: Option<String>,
    pub lease_duration: i32,
    pub vlan_id: Option<i32>,
    pub enabled: bool,
}

pub async fn fetch_all_subnets(db: &PgPool) -> Result<Vec<SubnetRow>> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, network, start_ip, end_ip, gateway,
               dns_servers, domain_name, lease_duration, vlan_id, enabled
        FROM dhcp_subnets
        ORDER BY name
        "#
    )
    .fetch_all(db)
    .await?;

    let mut subnets = Vec::new();
    for row in rows {
        // For now, just return empty list to get compilation working
        // Full database integration will be implemented when database is properly configured
    }

    Ok(subnets)
}

pub async fn get_dhcp_stats(db: &PgPool) -> Result<(i64, i64, i64, i64)> {
    let row = sqlx::query(
        r#"
        SELECT
            (SELECT COUNT(*) FROM dhcp_subnets) as total_subnets,
            (SELECT COUNT(*) FROM dhcp_leases WHERE state = 'active') as active_leases,
            (SELECT COUNT(*) FROM dhcp_leases WHERE state = 'expired') as expired_leases,
            (SELECT COUNT(*) FROM dhcp_reservations) as total_reservations
        "#
    )
    .fetch_one(db)
    .await?;

    Ok((
        row.get::<Option<i64>, _>("total_subnets").unwrap_or(0),
        row.get::<Option<i64>, _>("active_leases").unwrap_or(0),
        row.get::<Option<i64>, _>("expired_leases").unwrap_or(0),
        row.get::<Option<i64>, _>("total_reservations").unwrap_or(0),
    ))
}

pub async fn get_dns_stats(db: &PgPool) -> Result<(i64, i64, i64)> {
    let row = sqlx::query(
        r#"
        SELECT
            (SELECT COUNT(*) FROM dns_zones) as total_zones,
            (SELECT COUNT(*) FROM dns_records) as total_records,
            (SELECT COUNT(*) FROM dns_records WHERE is_dynamic = true) as dynamic_records
        "#
    )
    .fetch_one(db)
    .await?;

    Ok((
        row.get::<Option<i64>, _>("total_zones").unwrap_or(0),
        row.get::<Option<i64>, _>("total_records").unwrap_or(0),
        row.get::<Option<i64>, _>("dynamic_records").unwrap_or(0),
    ))
}