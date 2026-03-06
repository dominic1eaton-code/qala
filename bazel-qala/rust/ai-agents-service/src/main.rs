use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use qala_shared::{EventEnvelope, EventTopic};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    env,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

#[derive(Default)]
struct AIState {
    events: Mutex<Vec<EventEnvelope>>,
}

#[derive(Debug, Deserialize)]
struct AnalyzeSDERequest {
    sde_id: String,
    context: Option<String>,
}

#[derive(Debug, Serialize)]
struct Recommendation {
    id: String,
    category: String,
    message: String,
    confidence: f32,
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AIState::default());
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8087);

    let app = Router::new()
        .route("/health", get(health))
        .route("/recommendations", get(recommendations))
        .route("/analyze_sde", post(analyze_sde))
        .route("/events", get(list_events))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("[ai-agents-service] listening on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({"service":"ai-agents-service","status":"ok"}))
}

async fn recommendations() -> Json<serde_json::Value> {
    let items = vec![
        Recommendation {
            id: "rec-1".to_string(),
            category: "sde".to_string(),
            message: "Increase snapshot cadence for active SDEs.".to_string(),
            confidence: 0.83,
        },
        Recommendation {
            id: "rec-2".to_string(),
            category: "workflow".to_string(),
            message: "Split long-running pipeline stages in parallel groups.".to_string(),
            confidence: 0.78,
        },
    ];
    Json(json!({ "items": items }))
}

async fn analyze_sde(
    State(state): State<Arc<AIState>>,
    Json(req): Json<AnalyzeSDERequest>,
) -> Json<serde_json::Value> {
    let score = if req.context.as_deref().unwrap_or("").contains("hotfix") {
        0.61
    } else {
        0.84
    };
    let event = EventEnvelope::new(
        EventTopic::AIRecommendations,
        "AI_RECOMMENDATION",
        "ai-agents-service",
        json!({
            "sde_id": req.sde_id,
            "score": score
        }),
    );
    state.events.lock().unwrap().push(event.clone());

    Json(json!({
      "sde_id": req.sde_id,
      "score": score,
      "event_id": event.event_id
    }))
}

async fn list_events(State(state): State<Arc<AIState>>) -> Json<serde_json::Value> {
    let events = state.events.lock().unwrap().clone();
    Json(json!({
      "count": events.len(),
      "items": events
    }))
}
