// SQL query implementations for lease_manager
// Using runtime queries instead of compile-time checked macros

use crate::database::models::{DhcpSubnet, DhcpLease, DhcpReservation};
use sqlx::{PgPool, Row};
use std::net::Ipv4Addr;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::Result;

pub async fn fetch_all_subnets(db: &PgPool) -> Result<Vec<DhcpSubnet>> {
    let rows = sqlx::query(
        r#"
        SELECT
            id, name, network, start_ip, end_ip, gateway,
            dns_servers, domain_name, lease_duration, vlan_id,
            ipv6_prefix, enabled, description, created_at, updated_at
        FROM dhcp_subnets
        WHERE enabled = true
        "#
    )
    .fetch_all(db)
    .await?;

    let mut subnets = Vec::new();
    for row in rows {
        let subnet = DhcpSubnet {
            id: row.get("id"),
            name: row.get("name"),
            network: row.get("network"),
            start_ip: row.get::<std::net::IpAddr, _>("start_ip").to_string().parse()?,
            end_ip: row.get::<std::net::IpAddr, _>("end_ip").to_string().parse()?,
            gateway: row.get::<std::net::IpAddr, _>("gateway").to_string().parse()?,
            dns_servers: serde_json::from_value(row.get("dns_servers"))?,
            domain_name: row.get("domain_name"),
            lease_duration: row.get("lease_duration"),
            vlan_id: row.get("vlan_id"),
            ipv6_prefix: row.get("ipv6_prefix"),
            enabled: row.get("enabled"),
            description: row.get("description"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };
        subnets.push(subnet);
    }

    Ok(subnets)
}

pub async fn count_active_leases(db: &PgPool, subnet_id: Uuid, ip: Ipv4Addr) -> Result<i64> {
    let row = sqlx::query(
        r#"
        SELECT COUNT(*) as count
        FROM dhcp_leases
        WHERE subnet_id = $1
            AND ip_address = $2
            AND state = 'active'
            AND lease_end > NOW()
        "#
    )
    .bind(subnet_id)
    .bind(std::net::IpAddr::V4(ip))
    .fetch_one(db)
    .await?;

    Ok(row.get("count"))
}

pub async fn count_reservations(db: &PgPool, subnet_id: Uuid, ip: Ipv4Addr) -> Result<i64> {
    let row = sqlx::query(
        r#"
        SELECT COUNT(*) as count
        FROM dhcp_reservations
        WHERE subnet_id = $1 AND ip_address = $2
        "#
    )
    .bind(subnet_id)
    .bind(std::net::IpAddr::V4(ip))
    .fetch_one(db)
    .await?;

    Ok(row.get("count"))
}

pub async fn insert_or_update_lease(
    db: &PgPool,
    subnet_id: Uuid,
    mac_address: &[u8],
    ip_address: Ipv4Addr,
    hostname: Option<String>,
    lease_start: DateTime<Utc>,
    lease_end: DateTime<Utc>,
) -> Result<DhcpLease> {
    let row = sqlx::query(
        r#"
        INSERT INTO dhcp_leases (
            subnet_id, mac_address, ip_address, hostname,
            lease_start, lease_end, state
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'active')
        ON CONFLICT (mac_address)
        DO UPDATE SET
            subnet_id = $1,
            ip_address = $3,
            lease_start = $5,
            lease_end = $6,
            state = 'active',
            hostname = $4,
            updated_at = NOW()
        RETURNING *
        "#
    )
    .bind(subnet_id)
    .bind(mac_address)
    .bind(std::net::IpAddr::V4(ip_address))
    .bind(hostname)
    .bind(lease_start)
    .bind(lease_end)
    .fetch_one(db)
    .await?;

    Ok(DhcpLease {
        id: row.get("id"),
        subnet_id: row.get("subnet_id"),
        mac_address: row.get("mac_address"),
        ip_address: row.get::<std::net::IpAddr, _>("ip_address").to_string().parse()?,
        hostname: row.get("hostname"),
        lease_start: row.get("lease_start"),
        lease_end: row.get("lease_end"),
        state: row.get("state"),
        client_identifier: row.get("client_identifier"),
        vendor_class: row.get("vendor_class"),
        user_class: row.get("user_class"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

pub async fn find_active_lease_by_mac_and_ip(
    db: &PgPool,
    mac_address: &[u8],
    ip_address: Ipv4Addr,
) -> Result<Option<DhcpLease>> {
    let row = sqlx::query(
        r#"
        SELECT *
        FROM dhcp_leases
        WHERE mac_address = $1
            AND ip_address = $2
            AND state = 'active'
        "#
    )
    .bind(mac_address)
    .bind(std::net::IpAddr::V4(ip_address))
    .fetch_optional(db)
    .await?;

    match row {
        Some(row) => Ok(Some(DhcpLease {
            id: row.get("id"),
            subnet_id: row.get("subnet_id"),
            mac_address: row.get("mac_address"),
            ip_address: row.get::<std::net::IpAddr, _>("ip_address").to_string().parse()?,
            hostname: row.get("hostname"),
            lease_start: row.get("lease_start"),
            lease_end: row.get("lease_end"),
            state: row.get("state"),
            client_identifier: row.get("client_identifier"),
            vendor_class: row.get("vendor_class"),
            user_class: row.get("user_class"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })),
        None => Ok(None),
    }
}

pub async fn update_lease_end(db: &PgPool, lease_id: Uuid, new_lease_end: DateTime<Utc>) -> Result<DhcpLease> {
    let row = sqlx::query(
        r#"
        UPDATE dhcp_leases
        SET lease_end = $1, updated_at = NOW()
        WHERE id = $2
        RETURNING *
        "#
    )
    .bind(new_lease_end)
    .bind(lease_id)
    .fetch_one(db)
    .await?;

    Ok(DhcpLease {
        id: row.get("id"),
        subnet_id: row.get("subnet_id"),
        mac_address: row.get("mac_address"),
        ip_address: row.get::<std::net::IpAddr, _>("ip_address").to_string().parse()?,
        hostname: row.get("hostname"),
        lease_start: row.get("lease_start"),
        lease_end: row.get("lease_end"),
        state: row.get("state"),
        client_identifier: row.get("client_identifier"),
        vendor_class: row.get("vendor_class"),
        user_class: row.get("user_class"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

pub async fn release_lease(db: &PgPool, mac_address: &[u8], ip_address: Ipv4Addr) -> Result<bool> {
    let result = sqlx::query(
        r#"
        UPDATE dhcp_leases
        SET state = 'released', updated_at = NOW()
        WHERE mac_address = $1
            AND ip_address = $2
            AND state = 'active'
        "#
    )
    .bind(mac_address)
    .bind(std::net::IpAddr::V4(ip_address))
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_reservation(db: &PgPool, subnet_id: Uuid, mac_address: &[u8]) -> Result<Option<DhcpReservation>> {
    let row = sqlx::query(
        r#"
        SELECT *
        FROM dhcp_reservations
        WHERE subnet_id = $1 AND mac_address = $2
        "#
    )
    .bind(subnet_id)
    .bind(mac_address)
    .fetch_optional(db)
    .await?;

    match row {
        Some(row) => Ok(Some(DhcpReservation {
            id: row.get("id"),
            subnet_id: row.get("subnet_id"),
            mac_address: row.get("mac_address"),
            ip_address: row.get::<std::net::IpAddr, _>("ip_address").to_string().parse()?,
            hostname: row.get("hostname"),
            description: row.get("description"),
            created_at: row.get("created_at"),
        })),
        None => Ok(None),
    }
}

pub async fn get_active_lease_by_mac(db: &PgPool, mac_address: &[u8]) -> Result<Option<DhcpLease>> {
    let row = sqlx::query(
        r#"
        SELECT *
        FROM dhcp_leases
        WHERE mac_address = $1
            AND state = 'active'
            AND lease_end > NOW()
        ORDER BY lease_end DESC
        LIMIT 1
        "#
    )
    .bind(mac_address)
    .fetch_optional(db)
    .await?;

    match row {
        Some(row) => Ok(Some(DhcpLease {
            id: row.get("id"),
            subnet_id: row.get("subnet_id"),
            mac_address: row.get("mac_address"),
            ip_address: row.get::<std::net::IpAddr, _>("ip_address").to_string().parse()?,
            hostname: row.get("hostname"),
            lease_start: row.get("lease_start"),
            lease_end: row.get("lease_end"),
            state: row.get("state"),
            client_identifier: row.get("client_identifier"),
            vendor_class: row.get("vendor_class"),
            user_class: row.get("user_class"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })),
        None => Ok(None),
    }
}

pub async fn expire_old_leases(db: &PgPool) -> Result<u64> {
    let result = sqlx::query(
        r#"
        UPDATE dhcp_leases
        SET state = 'expired'
        WHERE state = 'active'
            AND lease_end < NOW()
        "#
    )
    .execute(db)
    .await?;

    Ok(result.rows_affected())
}