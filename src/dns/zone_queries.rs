// Runtime SQL queries for DNS zone management
use crate::database::models::{DnsZone, DnsRecord};
use sqlx::{PgPool, Row};
use uuid::Uuid;
use anyhow::Result;

pub async fn fetch_all_zones(db: &PgPool) -> Result<Vec<DnsZone>> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, zone_type, primary_ns, admin_email, serial_number,
               refresh_interval, retry_interval, expire_interval, minimum_ttl,
               created_at, updated_at
        FROM dns_zones
        WHERE zone_type IN ('master', 'forward')
        "#
    )
    .fetch_all(db)
    .await?;

    let mut zones = Vec::new();
    for row in rows {
        let zone = DnsZone {
            id: row.get("id"),
            name: row.get("name"),
            zone_type: row.get("zone_type"),
            primary_ns: row.get("primary_ns"),
            admin_email: row.get("admin_email"),
            serial_number: row.get("serial_number"),
            refresh_interval: row.get("refresh_interval"),
            retry_interval: row.get("retry_interval"),
            expire_interval: row.get("expire_interval"),
            minimum_ttl: row.get("minimum_ttl"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };
        zones.push(zone);
    }

    Ok(zones)
}

pub async fn fetch_zone_records(db: &PgPool, zone_id: Uuid) -> Result<Vec<DnsRecord>> {
    let rows = sqlx::query(
        r#"
        SELECT id, zone_id, name, record_type, value, ttl, priority, weight, port,
               is_dynamic, created_at, updated_at
        FROM dns_records
        WHERE zone_id = $1
        "#
    )
    .bind(zone_id)
    .fetch_all(db)
    .await?;

    let mut records = Vec::new();
    for row in rows {
        let record = DnsRecord {
            id: row.get("id"),
            zone_id: row.get("zone_id"),
            name: row.get("name"),
            record_type: row.get("record_type"),
            value: row.get("value"),
            ttl: row.get("ttl"),
            priority: row.get("priority"),
            weight: row.get("weight"),
            port: row.get("port"),
            is_dynamic: row.get("is_dynamic"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };
        records.push(record);
    }

    Ok(records)
}

pub async fn insert_dns_record(
    db: &PgPool,
    zone_id: Uuid,
    name: &str,
    record_type: &str,
    value: &str,
    ttl: Option<i32>,
    priority: Option<i32>,
) -> Result<DnsRecord> {
    let row = sqlx::query(
        r#"
        INSERT INTO dns_records (zone_id, name, record_type, value, ttl, priority, is_dynamic)
        VALUES ($1, $2, $3, $4, $5, $6, true)
        RETURNING *
        "#
    )
    .bind(zone_id)
    .bind(name)
    .bind(record_type)
    .bind(value)
    .bind(ttl)
    .bind(priority)
    .fetch_one(db)
    .await?;

    Ok(DnsRecord {
        id: row.get("id"),
        zone_id: row.get("zone_id"),
        name: row.get("name"),
        record_type: row.get("record_type"),
        value: row.get("value"),
        ttl: row.get("ttl"),
        priority: row.get("priority"),
        weight: row.get("weight"),
        port: row.get("port"),
        is_dynamic: row.get("is_dynamic"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

pub async fn delete_dns_record(db: &PgPool, record_id: Uuid) -> Result<bool> {
    let result = sqlx::query(
        r#"
        DELETE FROM dns_records
        WHERE id = $1
        "#
    )
    .bind(record_id)
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn update_zone_serial(db: &PgPool, zone_id: Uuid, serial: u32) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE dns_zones
        SET serial = $1, updated_at = NOW()
        WHERE id = $2
        "#
    )
    .bind(serial as i32)
    .bind(zone_id)
    .execute(db)
    .await?;

    Ok(())
}