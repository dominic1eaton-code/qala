//! # ai-agents-service
//!
//! The AI Agents service is the intelligence plane of the qala SFOS.
//! It listens on port 8087 and provides:
//!
//! - **Recommendation engine** — generates typed recommendations across all
//!   platform concerns: pipeline bottlenecks, SDE optimisation, test coverage,
//!   security hardening, resource capacity forecasting, and documentation quality.
//!
//! - **SDE Analyser** — accepts an SDE analysis request and returns a structured
//!   set of findings with confidence scores and suggested remediations.
//!
//! ## API surface (port 8087)
//!
//! | Method | Path               | Description                            |
//! |--------|--------------------|----------------------------------------|
//! | GET    | /health            | Liveness probe                         |
//! | GET    | /recommendations   | List active recommendations            |
//! | POST   | /analyze_sde       | Trigger AI analysis of a specific SDE  |

mod analyzer;
mod handlers;
mod recommendations;
mod state;

use std::sync::Arc;

use axum::{Router, routing::{get, post}};
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use state::AgentState;

const DEFAULT_PORT: u16 = 8087;
const SERVICE_NAME: &str = "ai-agents-service";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    init_tracing();

    info!(service = SERVICE_NAME, "starting up");

    let state = Arc::new(AgentState::new());

    // Register with the kernel on startup (best-effort)
    if let Err(e) = register_with_kernel(&state).await {
        tracing::warn!(error = %e, "kernel registration failed (continuing)");
    }

    let app = build_router(state);

    let port = std::env::var("AI_AGENTS_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(DEFAULT_PORT);

    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await?;
    info!(address = %addr, service = SERVICE_NAME, "listening");

    axum::serve(listener, app).await?;
    Ok(())
}

fn build_router(state: Arc<AgentState>) -> Router {
    Router::new()
        .route("/health",          get(handlers::health))
        .route("/recommendations", get(handlers::list_recommendations))
        .route("/analyze_sde",     post(handlers::analyze_sde))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::new(std::time::Duration::from_secs(60)))
        .layer(CorsLayer::permissive())
}

/// Attempt to register this service with the qala kernel.
async fn register_with_kernel(state: &AgentState) -> anyhow::Result<()> {
    let kernel_url = std::env::var("KERNEL_URL")
        .unwrap_or_else(|_| "http://localhost:7000".to_string());

    let port = std::env::var("AI_AGENTS_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(DEFAULT_PORT);

    let body = serde_json::json!({
        "name":      SERVICE_NAME,
        "subsystem": "ai",
        "address":   format!("http://ai-agents-service:{port}"),
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
        .unwrap_or_else(|_| EnvFilter::new("info,ai_agents_service=debug"));
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}
