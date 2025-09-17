use actix_web::{web, HttpResponse};
use crate::api::models::*;
use crate::api::server::ApiState;
use crate::api::validators::*;
use crate::database::models::{DhcpLease, DhcpSubnet, DhcpReservation};
use uuid::Uuid;
use sqlx::FromRow;
use tracing::{info, error};
use std::net::Ipv4Addr;

pub async fn list_leases(
    state: web::Data<ApiState>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> actix_web::Result<HttpResponse> {
    let state_filter = query.get("state").map(|s| s.as_str()).unwrap_or("active");

    let leases = sqlx::query!(
        r#"
        SELECT id, subnet_id, mac_address,
               ip_address as "ip_address: std::net::Ipv4Addr",
               hostname, lease_start, lease_end, state
        FROM dhcp_leases
        WHERE state = $1
        ORDER BY lease_start DESC
        LIMIT 100
        "#,
        state_filter
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;

    let responses: Vec<LeaseResponse> = leases
        .into_iter()
        .map(|lease| LeaseResponse {
            id: lease.id,
            subnet_id: lease.subnet_id,
            mac_address: bytes_to_mac_string(&lease.mac_address),
            ip_address: lease.ip_address,
            hostname: lease.hostname,
            lease_start: lease.lease_start,
            lease_end: lease.lease_end,
            state: lease.state,
        })
        .collect();

    Ok(HttpResponse::Ok().json(responses))
}

pub async fn get_lease(
    state: web::Data<ApiState>,
    path: web::Path<Uuid>,
) -> actix_web::Result<HttpResponse> {
    let lease_id = path.into_inner();

    let lease = sqlx::query!(
        r#"
        SELECT id, subnet_id, mac_address,
               ip_address as "ip_address: std::net::Ipv4Addr",
               hostname, lease_start, lease_end, state
        FROM dhcp_leases
        WHERE id = $1
        "#,
        lease_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;

    match lease {
        Some(lease) => {
            let response = LeaseResponse {
                id: lease.id,
                subnet_id: lease.subnet_id,
                mac_address: bytes_to_mac_string(&lease.mac_address),
                ip_address: lease.ip_address,
                hostname: lease.hostname,
                lease_start: lease.lease_start,
                lease_end: lease.lease_end,
                state: lease.state,
            };
            Ok(HttpResponse::Ok().json(response))
        }
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "not_found",
            "message": "Lease not found"
        }))),
    }
}

pub async fn create_lease(
    state: web::Data<ApiState>,
    req: web::Json<CreateLeaseRequest>,
) -> actix_web::Result<HttpResponse> {
    // Validate MAC address
    if !validate_mac_address(&req.mac_address) {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "invalid_mac",
            "message": "Invalid MAC address format"
        })));
    }

    let mac_bytes = mac_string_to_bytes(&req.mac_address)
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Invalid MAC address"))?;

    // TODO: Implement actual lease creation logic with lease manager
    info!("Creating lease for MAC: {}", req.mac_address);

    Ok(HttpResponse::Created().json(serde_json::json!({
        "message": "Lease creation initiated",
        "mac_address": req.mac_address
    })))
}

pub async fn release_lease(
    state: web::Data<ApiState>,
    path: web::Path<Uuid>,
) -> actix_web::Result<HttpResponse> {
    let lease_id = path.into_inner();

    let result = sqlx::query!(
        r#"
        UPDATE dhcp_leases
        SET state = 'released', updated_at = NOW()
        WHERE id = $1 AND state = 'active'
        "#,
        lease_id
    )
    .execute(&state.db)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;

    if result.rows_affected() > 0 {
        info!("Released lease: {}", lease_id);
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "Lease released successfully"
        })))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "not_found",
            "message": "Lease not found or already released"
        })))
    }
}

pub async fn list_subnets(
    state: web::Data<ApiState>,
) -> actix_web::Result<HttpResponse> {
    let subnets = sqlx::query!(
        r#"
        SELECT id, name, network,
               start_ip as "start_ip: std::net::Ipv4Addr",
               end_ip as "end_ip: std::net::Ipv4Addr",
               gateway as "gateway: std::net::Ipv4Addr",
               dns_servers, domain_name, lease_duration, vlan_id, enabled
        FROM dhcp_subnets
        ORDER BY name
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;

    let responses: Vec<SubnetResponse> = subnets
        .into_iter()
        .map(|subnet| {
            let dns_servers: Vec<Ipv4Addr> = serde_json::from_value(subnet.dns_servers)
                .unwrap_or_default();

            SubnetResponse {
                id: subnet.id,
                name: subnet.name,
                network: subnet.network.to_string(),
                start_ip: subnet.start_ip,
                end_ip: subnet.end_ip,
                gateway: subnet.gateway,
                dns_servers,
                domain_name: subnet.domain_name,
                lease_duration: subnet.lease_duration,
                vlan_id: subnet.vlan_id,
                enabled: subnet.enabled,
            }
        })
        .collect();

    Ok(HttpResponse::Ok().json(responses))
}

pub async fn get_subnet(
    state: web::Data<ApiState>,
    path: web::Path<Uuid>,
) -> actix_web::Result<HttpResponse> {
    let subnet_id = path.into_inner();

    let subnet = sqlx::query!(
        r#"
        SELECT id, name, network,
               start_ip as "start_ip: std::net::Ipv4Addr",
               end_ip as "end_ip: std::net::Ipv4Addr",
               gateway as "gateway: std::net::Ipv4Addr",
               dns_servers, domain_name, lease_duration, vlan_id, enabled
        FROM dhcp_subnets
        WHERE id = $1
        "#,
        subnet_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;

    match subnet {
        Some(subnet) => {
            let dns_servers: Vec<Ipv4Addr> = serde_json::from_value(subnet.dns_servers)
                .unwrap_or_default();

            let response = SubnetResponse {
                id: subnet.id,
                name: subnet.name,
                network: subnet.network.to_string(),
                start_ip: subnet.start_ip,
                end_ip: subnet.end_ip,
                gateway: subnet.gateway,
                dns_servers,
                domain_name: subnet.domain_name,
                lease_duration: subnet.lease_duration,
                vlan_id: subnet.vlan_id,
                enabled: subnet.enabled,
            };
            Ok(HttpResponse::Ok().json(response))
        }
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "not_found",
            "message": "Subnet not found"
        }))),
    }
}

pub async fn create_subnet(
    state: web::Data<ApiState>,
    req: web::Json<CreateSubnetRequest>,
) -> actix_web::Result<HttpResponse> {
    // Validate network
    if !validate_ipv4_network(&req.network) {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "invalid_network",
            "message": "Invalid network format"
        })));
    }

    // Validate IP range
    if !validate_ip_in_range(req.start_ip, req.start_ip, req.end_ip) {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "invalid_range",
            "message": "Invalid IP range"
        })));
    }

    let network: ipnet::Ipv4Net = req.network.parse()
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid network"))?;

    let dns_servers_json = serde_json::to_value(&req.dns_servers)
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("JSON error: {}", e)))?;

    let subnet = sqlx::query!(
        r#"
        INSERT INTO dhcp_subnets (name, network, start_ip, end_ip, gateway,
                                 dns_servers, domain_name, lease_duration, vlan_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id
        "#,
        req.name,
        network,
        std::net::IpAddr::V4(req.start_ip),
        std::net::IpAddr::V4(req.end_ip),
        std::net::IpAddr::V4(req.gateway),
        dns_servers_json,
        req.domain_name,
        req.lease_duration.unwrap_or(86400),
        req.vlan_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;

    info!("Created subnet: {} ({})", req.name, subnet.id);

    Ok(HttpResponse::Created().json(serde_json::json!({
        "id": subnet.id,
        "message": "Subnet created successfully"
    })))
}

pub async fn update_subnet(
    state: web::Data<ApiState>,
    path: web::Path<Uuid>,
    req: web::Json<UpdateSubnetRequest>,
) -> actix_web::Result<HttpResponse> {
    let subnet_id = path.into_inner();

    // Build dynamic update query based on provided fields
    // TODO: Implement dynamic SQL update

    info!("Updating subnet: {}", subnet_id);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Subnet updated successfully"
    })))
}

pub async fn delete_subnet(
    state: web::Data<ApiState>,
    path: web::Path<Uuid>,
) -> actix_web::Result<HttpResponse> {
    let subnet_id = path.into_inner();

    let result = sqlx::query!(
        r#"
        DELETE FROM dhcp_subnets
        WHERE id = $1
        "#,
        subnet_id
    )
    .execute(&state.db)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;

    if result.rows_affected() > 0 {
        info!("Deleted subnet: {}", subnet_id);
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "Subnet deleted successfully"
        })))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "not_found",
            "message": "Subnet not found"
        })))
    }
}

pub async fn list_reservations(
    state: web::Data<ApiState>,
) -> actix_web::Result<HttpResponse> {
    let reservations = sqlx::query!(
        r#"
        SELECT id, subnet_id, mac_address,
               ip_address as "ip_address: std::net::Ipv4Addr",
               hostname, description, created_at
        FROM dhcp_reservations
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;

    let responses: Vec<ReservationResponse> = reservations
        .into_iter()
        .map(|res| ReservationResponse {
            id: res.id,
            subnet_id: res.subnet_id,
            mac_address: bytes_to_mac_string(&res.mac_address),
            ip_address: res.ip_address,
            hostname: res.hostname,
            description: res.description,
            created_at: res.created_at,
        })
        .collect();

    Ok(HttpResponse::Ok().json(responses))
}

pub async fn create_reservation(
    state: web::Data<ApiState>,
    req: web::Json<CreateReservationRequest>,
) -> actix_web::Result<HttpResponse> {
    // Validate MAC address
    if !validate_mac_address(&req.mac_address) {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "invalid_mac",
            "message": "Invalid MAC address format"
        })));
    }

    let mac_bytes = mac_string_to_bytes(&req.mac_address)
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Invalid MAC address"))?;

    let reservation = sqlx::query!(
        r#"
        INSERT INTO dhcp_reservations (subnet_id, mac_address, ip_address, hostname, description)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
        req.subnet_id,
        &mac_bytes[..],
        std::net::IpAddr::V4(req.ip_address),
        req.hostname,
        req.description
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;

    info!("Created reservation: {} -> {}", req.mac_address, req.ip_address);

    Ok(HttpResponse::Created().json(serde_json::json!({
        "id": reservation.id,
        "message": "Reservation created successfully"
    })))
}

pub async fn delete_reservation(
    state: web::Data<ApiState>,
    path: web::Path<Uuid>,
) -> actix_web::Result<HttpResponse> {
    let reservation_id = path.into_inner();

    let result = sqlx::query!(
        r#"
        DELETE FROM dhcp_reservations
        WHERE id = $1
        "#,
        reservation_id
    )
    .execute(&state.db)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;

    if result.rows_affected() > 0 {
        info!("Deleted reservation: {}", reservation_id);
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "Reservation deleted successfully"
        })))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "not_found",
            "message": "Reservation not found"
        })))
    }
}

pub async fn get_stats(
    state: web::Data<ApiState>,
) -> actix_web::Result<HttpResponse> {
    // Get DHCP statistics
    let stats = sqlx::query!(
        r#"
        SELECT
            (SELECT COUNT(*) FROM dhcp_subnets) as total_subnets,
            (SELECT COUNT(*) FROM dhcp_leases WHERE state = 'active') as active_leases,
            (SELECT COUNT(*) FROM dhcp_leases WHERE state = 'expired') as expired_leases,
            (SELECT COUNT(*) FROM dhcp_reservations) as total_reservations
        "#
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "total_subnets": stats.total_subnets,
        "active_leases": stats.active_leases,
        "expired_leases": stats.expired_leases,
        "total_reservations": stats.total_reservations
    })))
}