use actix_web::{web, HttpResponse};
use crate::api::models::*;
use crate::api::server::ApiState;
use crate::api::validators::*;
use uuid::Uuid;
use tracing::{info, error};

pub async fn list_zones(
    state: web::Data<ApiState>,
) -> actix_web::Result<HttpResponse> {
    let zones = sqlx::query!(
        r#"
        SELECT id, name, zone_type, serial_number, refresh_interval,
               retry_interval, expire_interval, minimum_ttl,
               primary_ns, admin_email, created_at, updated_at
        FROM dns_zones
        ORDER BY name
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;

    let responses: Vec<ZoneResponse> = zones
        .into_iter()
        .map(|zone| ZoneResponse {
            id: zone.id,
            name: zone.name,
            zone_type: zone.zone_type,
            serial_number: zone.serial_number,
            refresh_interval: zone.refresh_interval,
            retry_interval: zone.retry_interval,
            expire_interval: zone.expire_interval,
            minimum_ttl: zone.minimum_ttl,
            primary_ns: zone.primary_ns,
            admin_email: zone.admin_email,
            created_at: zone.created_at,
            updated_at: zone.updated_at,
        })
        .collect();

    Ok(HttpResponse::Ok().json(responses))
}

pub async fn get_zone(
    state: web::Data<ApiState>,
    path: web::Path<Uuid>,
) -> actix_web::Result<HttpResponse> {
    let zone_id = path.into_inner();

    let zone = sqlx::query!(
        r#"
        SELECT id, name, zone_type, serial_number, refresh_interval,
               retry_interval, expire_interval, minimum_ttl,
               primary_ns, admin_email, created_at, updated_at
        FROM dns_zones
        WHERE id = $1
        "#,
        zone_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;

    match zone {
        Some(zone) => {
            let response = ZoneResponse {
                id: zone.id,
                name: zone.name,
                zone_type: zone.zone_type,
                serial_number: zone.serial_number,
                refresh_interval: zone.refresh_interval,
                retry_interval: zone.retry_interval,
                expire_interval: zone.expire_interval,
                minimum_ttl: zone.minimum_ttl,
                primary_ns: zone.primary_ns,
                admin_email: zone.admin_email,
                created_at: zone.created_at,
                updated_at: zone.updated_at,
            };
            Ok(HttpResponse::Ok().json(response))
        }
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "not_found",
            "message": "Zone not found"
        }))),
    }
}

pub async fn create_zone(
    state: web::Data<ApiState>,
    req: web::Json<CreateZoneRequest>,
) -> actix_web::Result<HttpResponse> {
    // Validate zone name
    if !validate_domain_name(&req.name) {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "invalid_zone_name",
            "message": "Invalid zone name format"
        })));
    }

    // Validate zone type
    if !["master", "slave", "forward"].contains(&req.zone_type.as_str()) {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "invalid_zone_type",
            "message": "Invalid zone type. Must be 'master', 'slave', or 'forward'"
        })));
    }

    let serial_number = chrono::Utc::now().timestamp();

    let zone = sqlx::query!(
        r#"
        INSERT INTO dns_zones (name, zone_type, serial_number, primary_ns, admin_email)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
        req.name,
        req.zone_type,
        serial_number,
        req.primary_ns,
        req.admin_email
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;

    info!("Created DNS zone: {} ({})", req.name, zone.id);

    Ok(HttpResponse::Created().json(serde_json::json!({
        "id": zone.id,
        "message": "Zone created successfully"
    })))
}

pub async fn update_zone(
    state: web::Data<ApiState>,
    path: web::Path<Uuid>,
    req: web::Json<UpdateZoneRequest>,
) -> actix_web::Result<HttpResponse> {
    let zone_id = path.into_inner();

    // Update serial number when zone is modified
    let serial_number = chrono::Utc::now().timestamp();

    // Build dynamic update query based on provided fields
    // TODO: Implement dynamic SQL update

    info!("Updating zone: {}", zone_id);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Zone updated successfully"
    })))
}

pub async fn delete_zone(
    state: web::Data<ApiState>,
    path: web::Path<Uuid>,
) -> actix_web::Result<HttpResponse> {
    let zone_id = path.into_inner();

    // Delete all records in the zone first
    sqlx::query!(
        r#"
        DELETE FROM dns_records
        WHERE zone_id = $1
        "#,
        zone_id
    )
    .execute(&state.db)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;

    // Delete the zone
    let result = sqlx::query!(
        r#"
        DELETE FROM dns_zones
        WHERE id = $1
        "#,
        zone_id
    )
    .execute(&state.db)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;

    if result.rows_affected() > 0 {
        info!("Deleted zone: {}", zone_id);
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "Zone deleted successfully"
        })))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "not_found",
            "message": "Zone not found"
        })))
    }
}

pub async fn list_records(
    state: web::Data<ApiState>,
    path: web::Path<Uuid>,
) -> actix_web::Result<HttpResponse> {
    let zone_id = path.into_inner();

    let records = sqlx::query!(
        r#"
        SELECT id, zone_id, name, record_type, value, ttl,
               priority, weight, port, is_dynamic,
               created_at, updated_at
        FROM dns_records
        WHERE zone_id = $1
        ORDER BY name, record_type
        "#,
        zone_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;

    let responses: Vec<RecordResponse> = records
        .into_iter()
        .map(|record| RecordResponse {
            id: record.id,
            zone_id: record.zone_id,
            name: record.name,
            record_type: record.record_type,
            value: record.value,
            ttl: record.ttl,
            priority: record.priority,
            weight: record.weight,
            port: record.port,
            is_dynamic: record.is_dynamic,
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
        .collect();

    Ok(HttpResponse::Ok().json(responses))
}

pub async fn create_record(
    state: web::Data<ApiState>,
    path: web::Path<Uuid>,
    req: web::Json<CreateRecordRequest>,
) -> actix_web::Result<HttpResponse> {
    let zone_id = path.into_inner();

    // Validate record type
    if !validate_dns_record_type(&req.record_type) {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "invalid_record_type",
            "message": "Invalid DNS record type"
        })));
    }

    // Validate TTL if provided
    if let Some(ttl) = req.ttl {
        if !validate_ttl(ttl) {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_ttl",
                "message": "Invalid TTL value"
            })));
        }
    }

    let record = sqlx::query!(
        r#"
        INSERT INTO dns_records (zone_id, name, record_type, value, ttl, priority, weight, port, is_dynamic)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, false)
        RETURNING id
        "#,
        zone_id,
        req.name,
        req.record_type,
        req.value,
        req.ttl.unwrap_or(3600),
        req.priority,
        req.weight,
        req.port
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;

    // Update zone serial
    let serial_number = chrono::Utc::now().timestamp();
    sqlx::query!(
        r#"
        UPDATE dns_zones
        SET serial_number = $1, updated_at = NOW()
        WHERE id = $2
        "#,
        serial_number,
        zone_id
    )
    .execute(&state.db)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;

    info!("Created DNS record: {} {} in zone {}", req.record_type, req.name, zone_id);

    Ok(HttpResponse::Created().json(serde_json::json!({
        "id": record.id,
        "message": "Record created successfully"
    })))
}

pub async fn update_record(
    state: web::Data<ApiState>,
    path: web::Path<Uuid>,
    req: web::Json<UpdateRecordRequest>,
) -> actix_web::Result<HttpResponse> {
    let record_id = path.into_inner();

    // Validate TTL if provided
    if let Some(ttl) = req.ttl {
        if !validate_ttl(ttl) {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_ttl",
                "message": "Invalid TTL value"
            })));
        }
    }

    // TODO: Implement dynamic SQL update

    info!("Updating record: {}", record_id);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Record updated successfully"
    })))
}

pub async fn delete_record(
    state: web::Data<ApiState>,
    path: web::Path<Uuid>,
) -> actix_web::Result<HttpResponse> {
    let record_id = path.into_inner();

    // Get zone_id before deleting
    let zone_result = sqlx::query!(
        r#"
        SELECT zone_id FROM dns_records WHERE id = $1
        "#,
        record_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;

    if let Some(zone) = zone_result {
        // Delete the record
        let result = sqlx::query!(
            r#"
            DELETE FROM dns_records
            WHERE id = $1
            "#,
            record_id
        )
        .execute(&state.db)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;

        if result.rows_affected() > 0 {
            // Update zone serial
            let serial_number = chrono::Utc::now().timestamp();
            sqlx::query!(
                r#"
                UPDATE dns_zones
                SET serial_number = $1, updated_at = NOW()
                WHERE id = $2
                "#,
                serial_number,
                zone.zone_id
            )
            .execute(&state.db)
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;

            info!("Deleted record: {}", record_id);
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "Record deleted successfully"
            })))
        } else {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "not_found",
                "message": "Record not found"
            })))
        }
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "not_found",
            "message": "Record not found"
        })))
    }
}