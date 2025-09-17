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
    // Return realistic mock data for demo purposes
    let dhcp_metrics = DhcpMetrics {
        total_subnets: 2,
        active_leases: 15,
        expired_leases: 3,
        reserved_addresses: 10,
        available_addresses: 180,
    };

    let dns_metrics = DnsMetrics {
        total_zones: 3,
        total_records: 42,
        dynamic_records: 15,
    };

    // Get system metrics (simplified - mock data for now)
    let system_metrics = SystemMetrics {
        uptime_seconds: 3600,  // 1 hour uptime
        memory_usage_mb: 256.5,  // Mock memory usage
        cpu_usage_percent: 12.5,  // Mock CPU usage
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