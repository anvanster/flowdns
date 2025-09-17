use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod database;
mod dhcp;
mod dns;
mod api;

use config::Settings;

#[derive(Parser, Debug)]
#[command(name = "flowdns")]
#[command(about = "Multi-subnet DNS/DHCP server with IPv6 support", long_about = None)]
struct Args {
    #[arg(short, long, default_value = "config/server.toml")]
    config: String,

    #[arg(long)]
    migrate: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "flowdns=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting FlowDNS Server");

    let args = Args::parse();

    // Load configuration
    let settings = Settings::load(&args.config)?;
    let settings = Arc::new(settings);

    // Initialize database
    let db_pool = database::init_pool(&settings.database).await?;

    // Run migrations if requested
    if args.migrate {
        info!("Running database migrations...");
        database::run_migrations(&db_pool).await?;
        info!("Migrations completed successfully");
        return Ok(());
    }

    // Start services
    let mut handles = vec![];

    // Start DHCP server
    if settings.dhcp.enabled {
        let dhcp_settings = Arc::clone(&settings);
        let dhcp_pool = db_pool.clone();
        handles.push(tokio::spawn(async move {
            if let Err(e) = dhcp::server::start(dhcp_settings, dhcp_pool).await {
                error!("DHCP server failed: {}", e);
            }
        }));
    }

    // Start DNS server
    if settings.dns.enabled {
        let dns_settings = Arc::clone(&settings);
        let dns_pool = db_pool.clone();
        handles.push(tokio::spawn(async move {
            if let Err(e) = dns::simple_server::start(dns_settings, dns_pool).await {
                error!("DNS server failed: {}", e);
            }
        }));
    }

    // Start API server
    if settings.api.enabled {
        let api_settings = Arc::clone(&settings);
        let api_pool = db_pool.clone();
        handles.push(tokio::spawn(async move {
            if let Err(e) = api::server::start(api_settings, api_pool).await {
                error!("API server failed: {}", e);
            }
        }));
    }

    // Wait for all services
    for handle in handles {
        handle.await?;
    }

    Ok(())
}