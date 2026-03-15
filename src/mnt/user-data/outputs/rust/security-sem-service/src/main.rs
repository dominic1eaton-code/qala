//! # security-sem-service
//!
//! The Security & SEM (Security Event Management) service is the security
//! plane of the qala SFOS. It listens on port 8088 and provides:
//!
//! - **Threat management** — tracks all active threats, CVEs, and security findings.
//! - **Policy engine**    — stores and enforces governance policies platform-wide.
//! - **SDE scanner**      — executes security scans against a specified SDE and
//!                          produces a structured findings report.
//!
//! ## API surface (port 8088)
//!
//! | Method | Path              | Description                                    |
//! |--------|-------------------|------------------------------------------------|
//! | GET    | /health           | Liveness probe                                 |
//! | GET    | /threats          | List active threats / security findings        |
//! | POST   | /policy/update    | Create or update a governance policy           |
//! | POST   | /scan_sde         | Trigger a security scan of a specific SDE      |

mod handlers;
mod policies;
mod scanner;
mod state;
mod threats;

use std::sync::Arc;

use axum::{Router, routing::{get, post}};
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use state::SemState;

const DEFAULT_PORT: u16 = 8088;
const SERVICE_NAME: &str = "security-sem-service";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    init_tracing();

    info!(service = SERVICE_NAME, "starting up");

    let state = Arc::new(SemState::new());

    // Seed baseline governance policies
    state.seed_baseline_policies().await;

    // Register with kernel (best-effort)
    if let Err(e) = register_with_kernel(&state).await {
        tracing::warn!(error = %e, "kernel registration failed (continuing)");
    }

    let app = build_router(state);

    let port = std::env::var("SECURITY_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(DEFAULT_PORT);

    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await?;
    info!(address = %addr, service = SERVICE_NAME, "listening");

    axum::serve(listener, app).await?;
    Ok(())
}

fn build_router(state: Arc<SemState>) -> Router {
    Router::new()
        .route("/health",        get(handlers::health))
        .route("/threats",       get(handlers::list_threats))
        .route("/policy/update", post(handlers::update_policy))
        .route("/scan_sde",      post(handlers::scan_sde))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::new(std::time::Duration::from_secs(120)))
        .layer(CorsLayer::permissive())
}

async fn register_with_kernel(state: &SemState) -> anyhow::Result<()> {
    let kernel_url = &state.kernel_url;
    let port = std::env::var("SECURITY_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(DEFAULT_PORT);

    let body = serde_json::json!({
        "name":      SERVICE_NAME,
        "subsystem": "sem",
        "address":   format!("http://security-sem-service:{port}"),
        "port":      port,
        "language":  "rust",
        "version":   env!("CARGO_PKG_VERSION"),
    });

    state
        .http_client
        .post(format!("{kernel_url}/v1/kernel/register-service"))
        .json(&body)
        .send()
        .await?;

    info!("registered with kernel");
    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,security_sem_service=debug"));
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}
