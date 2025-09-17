use crate::config::Settings;
use sqlx::PgPool;
use actix_web::{web, App, HttpServer, middleware};
use actix_web_httpauth::middleware::HttpAuthentication;
use std::sync::Arc;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use anyhow::Result;
use tracing::{info, error};

use crate::api::{auth, handlers, models, validators};

pub struct ApiState {
    pub db: PgPool,
    pub settings: Arc<Settings>,
}

pub async fn start(settings: Arc<Settings>, db: PgPool) -> Result<()> {
    let api_addr = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        settings.api.port,
    );

    info!("Starting API server on {}", api_addr);

    let state = web::Data::new(ApiState {
        db: db.clone(),
        settings: settings.clone(),
    });

    let server = HttpServer::new(move || {
        let auth_middleware = HttpAuthentication::bearer(auth::validator);

        App::new()
            .app_data(state.clone())
            .wrap(middleware::Logger::default())
            .wrap(middleware::NormalizePath::trim())
            .service(
                web::scope("/api/v1")
                    .service(
                        // Public endpoints (no auth required)
                        web::scope("/auth")
                            .route("/login", web::post().to(handlers::auth::login))
                            .route("/refresh", web::post().to(handlers::auth::refresh))
                    )
                    .service(
                        // API Documentation endpoints (no auth required)
                        web::scope("/docs")
                            .route("/openapi.json", web::get().to(handlers::docs::openapi_spec))
                            .route("", web::get().to(handlers::docs::swagger_ui))
                    )
                    .service(
                        // System health and metrics endpoints (no auth required for monitoring)
                        web::scope("/system")
                            .route("/health", web::get().to(handlers::system::health))
                            .route("/metrics", web::get().to(handlers::system::metrics))
                    )
                    .service(
                        // Protected endpoints (auth required)
                        web::scope("")
                            .wrap(auth_middleware)
                            // DHCP endpoints
                            .service(
                                web::scope("/dhcp")
                                    .route("/leases", web::get().to(handlers::dhcp::list_leases))
                                    .route("/leases", web::post().to(handlers::dhcp::create_lease))
                                    .route("/leases/{id}", web::get().to(handlers::dhcp::get_lease))
                                    .route("/leases/{id}", web::delete().to(handlers::dhcp::release_lease))
                                    .route("/subnets", web::get().to(handlers::dhcp::list_subnets))
                                    .route("/subnets", web::post().to(handlers::dhcp::create_subnet))
                                    .route("/subnets/{id}", web::get().to(handlers::dhcp::get_subnet))
                                    .route("/subnets/{id}", web::put().to(handlers::dhcp::update_subnet))
                                    .route("/subnets/{id}", web::delete().to(handlers::dhcp::delete_subnet))
                                    .route("/reservations", web::get().to(handlers::dhcp::list_reservations))
                                    .route("/reservations", web::post().to(handlers::dhcp::create_reservation))
                                    .route("/reservations/{id}", web::delete().to(handlers::dhcp::delete_reservation))
                                    .route("/stats", web::get().to(handlers::dhcp::get_stats))
                            )
                            // DNS endpoints
                            .service(
                                web::scope("/dns")
                                    .route("/zones", web::get().to(handlers::dns::list_zones))
                                    .route("/zones", web::post().to(handlers::dns::create_zone))
                                    .route("/zones/{id}", web::get().to(handlers::dns::get_zone))
                                    .route("/zones/{id}", web::put().to(handlers::dns::update_zone))
                                    .route("/zones/{id}", web::delete().to(handlers::dns::delete_zone))
                                    .route("/zones/{zone_id}/records", web::get().to(handlers::dns::list_records))
                                    .route("/zones/{zone_id}/records", web::post().to(handlers::dns::create_record))
                                    .route("/records/{id}", web::put().to(handlers::dns::update_record))
                                    .route("/records/{id}", web::delete().to(handlers::dns::delete_record))
                            )
                            // Protected system endpoints
                            .service(
                                web::scope("/system")
                                    .route("/config", web::get().to(handlers::system::get_config))
                            )
                    )
            )
    })
    .bind(&api_addr)?
    .run();

    info!("API server listening on {}", api_addr);

    match server.await {
        Ok(_) => {
            info!("API server shutdown gracefully");
            Ok(())
        }
        Err(e) => {
            error!("API server error: {}", e);
            Err(anyhow::anyhow!("API server failed: {}", e))
        }
    }
}