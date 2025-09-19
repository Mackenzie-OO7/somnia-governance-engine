use axum::Router;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use somnia_governance_engine::{
    api::{
        routes::{auth_routes, governance_routes, health_routes, websocket_routes},
    },
    config::Config,
    AppState,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "somnia_governance_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env()?;
    
    // Initialize clients
    let blockchain_client = somnia_governance_engine::blockchain::client::SomniaClient::new(&config).await?;
    let ipfs_client = somnia_governance_engine::ipfs::client::IpfsClient::new(&config).await?;
    let governance_engine = somnia_governance_engine::governance::engine::GovernanceEngine::new(
        blockchain_client.clone(),
        ipfs_client.clone(),
    ).await?;

    // Create application state
    let app_state = AppState {
        config: config.clone(),
        blockchain_client,
        ipfs_client,
        governance_engine,
    };

    // Build application routes
    let app = Router::new()
        .nest("/api/health", health_routes())
        .nest("/api/auth", auth_routes())
        .nest("/api/governance", governance_routes())
        .nest("/ws", websocket_routes())
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        )
        .with_state(app_state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    tracing::info!("🚀 Somnia Governance Engine starting on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
