mod state;

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use std::{env, net::SocketAddr, sync::Arc};

use qala_shared::{EventEnvelope, KernelCommand, ServiceRegistration};
use state::{KernelState, SharedKernelState};

#[tokio::main]
async fn main() {
    let shared = Arc::new(KernelState::default());
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(7000);

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/kernel/status", get(kernel_status))
        .route("/v1/kernel/register-service", post(register_service))
        .route("/v1/kernel/events", post(ingest_event).get(list_events))
        .route("/v1/kernel/command", post(run_command))
        .with_state(shared);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("[qala-kernel] listening on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({
        "service": "qala-kernel",
        "status": "ok"
    }))
}

async fn kernel_status(State(state): State<SharedKernelState>) -> Json<serde_json::Value> {
    Json(state.snapshot())
}

async fn register_service(
    State(state): State<SharedKernelState>,
    Json(reg): Json<ServiceRegistration>,
) -> Json<serde_json::Value> {
    let response = state.register_service(reg);
    Json(response)
}

async fn ingest_event(
    State(state): State<SharedKernelState>,
    Json(event): Json<EventEnvelope>,
) -> Json<serde_json::Value> {
    Json(state.ingest_event(event))
}

async fn list_events(State(state): State<SharedKernelState>) -> Json<serde_json::Value> {
    Json(state.list_recent_events())
}

async fn run_command(
    State(state): State<SharedKernelState>,
    Json(command): Json<KernelCommand>,
) -> Json<serde_json::Value> {
    Json(state.run_command(command))
}
