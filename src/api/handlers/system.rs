use actix_web::{web, HttpResponse};
use crate::api::models::{HealthResponse, MetricsResponse, DhcpMetrics, DnsMetrics, SystemMetrics};
use crate::api::server::ApiState;
use chrono::Utc;
use tracing::info;

pub async fn health(
    state: web::Data<ApiState>,
) -> actix_web::Result<HttpResponse> {
    // Check database connection (simplified - skip actual query for now)
    let db_status = "healthy";

    // Check service status
    let dhcp_status = if state.settings.dhcp.enabled {
        "enabled"
    } else {
        "disabled"
    };

    let dns_status = if state.settings.dns.enabled {
        "enabled"
    } else {
        "disabled"
    };

    let response = HealthResponse {
        status: "healthy".to_string(),
        database: db_status.to_string(),
        dhcp_server: dhcp_status.to_string(),
        dns_server: dns_status.to_string(),
        api_server: "healthy".to_string(),
        timestamp: Utc::now(),
    };

    Ok(HttpResponse::Ok().json(response))
}

pub async fn metrics(
    _state: web::Data<ApiState>,
) -> actix_web::Result<HttpResponse> {
    // Simplified metrics - return mock data for now
    let dhcp_metrics = DhcpMetrics {
        total_subnets: 0,
        active_leases: 0,
        expired_leases: 0,
        reserved_addresses: 0,
        available_addresses: 0,
    };

    let dns_metrics = DnsMetrics {
        total_zones: 0,
        total_records: 0,
        dynamic_records: 0,
    };

    // Get system metrics (simplified - in production, use actual system monitoring)
    let system_metrics = SystemMetrics {
        uptime_seconds: 0,  // TODO: Track actual uptime
        memory_usage_mb: 0.0,  // TODO: Get actual memory usage
        cpu_usage_percent: 0.0,  // TODO: Get actual CPU usage
    };

    let response = MetricsResponse {
        dhcp: dhcp_metrics,
        dns: dns_metrics,
        system: system_metrics,
    };

    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_config(
    state: web::Data<ApiState>,
) -> actix_web::Result<HttpResponse> {
    // Return non-sensitive configuration information
    let config = serde_json::json!({
        "dhcp": {
            "enabled": state.settings.dhcp.enabled,
            "port": state.settings.dhcp.port,
            "bind_address": state.settings.dhcp.bind_address,
        },
        "dns": {
            "enabled": state.settings.dns.enabled,
            "port": state.settings.dns.port,
            "bind_address": state.settings.dns.bind_address,
        },
        "api": {
            "enabled": state.settings.api.enabled,
            "port": state.settings.api.port,
        },
        "database": {
            "max_connections": state.settings.database.max_connections,
            "min_connections": state.settings.database.min_connections,
        },
    });

    info!("Configuration requested via API");

    Ok(HttpResponse::Ok().json(config))
}