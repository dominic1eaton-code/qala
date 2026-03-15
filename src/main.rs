//! # qala-kernel
//!
//! The Kernel is the platform OS coordinator for the qala SFOS.
//! It runs on port 7000 and provides:
//!
//! - **Service Registry** — tracks all registered services and their heartbeat status
//! - **Command Bus**      — executes platform-level commands (subsystem/service status changes)
//! - **Event Aggregator** — counts and stores recent events by topic and source service
//!
//! ## API surface (port 7000)
//!
//! | Method | Path                        | Description                     |
//! |--------|-----------------------------|---------------------------------|
//! | GET    | /health                     | Liveness probe                  |
//! | GET    | /v1/kernel/status           | Full kernel status snapshot     |
//! | POST   | /v1/kernel/register-service | Register a service with kernel  |
//! | POST   | /v1/kernel/events           | Ingest an event envelope        |
//! | GET    | /v1/kernel/events           | Query recent events             |
//! | POST   | /v1/kernel/command          | Execute a kernel command        |

mod commands;
mod events;
mod handlers;
mod registry;
mod state;

use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use tokio::net::TcpListener;
use tower_http::{
    cors::CorsLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use state::KernelState;

/// Default port the kernel listens on.
const DEFAULT_PORT: u16 = 7000;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env if present
    let _ = dotenvy::dotenv();

    // Initialise structured tracing (JSON in production, pretty in dev)
    init_tracing();

    info!("qala-kernel starting up");

    // Construct shared kernel state
    let state = Arc::new(KernelState::new());

    // Build the Axum router
    let app = build_router(state);

    // Determine bind address
    let port = std::env::var("KERNEL_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(DEFAULT_PORT);

    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await?;
    info!(address = %addr, "qala-kernel listening");

    axum::serve(listener, app).await?;
    Ok(())
}

/// Construct the application router with all routes and middleware.
fn build_router(state: Arc<KernelState>) -> Router {
    let api = Router::new()
        .route("/v1/kernel/status",           get(handlers::get_status))
        .route("/v1/kernel/register-service", post(handlers::register_service))
        .route("/v1/kernel/events",           post(handlers::ingest_event))
        .route("/v1/kernel/events",           get(handlers::query_events))
        .route("/v1/kernel/command",          post(handlers::execute_command))
        .with_state(state);

    Router::new()
        .route("/health", get(handlers::health))
        .merge(api)
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::new(std::time::Duration::from_secs(30)))
        .layer(CorsLayer::permissive())
}

/// Initialise the tracing subscriber from the `RUST_LOG` env var.
fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,qala_kernel=debug"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}
