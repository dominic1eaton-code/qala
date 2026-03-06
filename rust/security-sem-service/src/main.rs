use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use qala_shared::{EventEnvelope, EventTopic};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::HashMap,
    env,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Threat {
    id: String,
    sde_id: String,
    threat_type: String,
    severity: String,
}

#[derive(Debug, Deserialize)]
struct ScanRequest {
    sde_id: String,
}

#[derive(Debug, Deserialize)]
struct PolicyUpdateRequest {
    name: String,
    rules: serde_json::Value,
}

#[derive(Default)]
struct SEMState {
    threats: Mutex<Vec<Threat>>,
    policies: Mutex<HashMap<String, serde_json::Value>>,
    events: Mutex<Vec<EventEnvelope>>,
}

#[tokio::main]
async fn main() {
    let state = Arc::new(SEMState::default());
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8088);

    let app = Router::new()
        .route("/health", get(health))
        .route("/threats", get(get_threats))
        .route("/policy/update", post(update_policy))
        .route("/scan_sde", post(scan_sde))
        .route("/events", get(get_events))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("[security-sem-service] listening on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({"service":"security-sem-service","status":"ok"}))
}

async fn get_threats(State(state): State<Arc<SEMState>>) -> Json<serde_json::Value> {
    let threats = state.threats.lock().unwrap().clone();
    Json(json!({
      "count": threats.len(),
      "items": threats
    }))
}

async fn update_policy(
    State(state): State<Arc<SEMState>>,
    Json(req): Json<PolicyUpdateRequest>,
) -> Json<serde_json::Value> {
    state
        .policies
        .lock()
        .unwrap()
        .insert(req.name.clone(), req.rules.clone());

    let event = EventEnvelope::new(
        EventTopic::SecurityEvents,
        "POLICY_UPDATED",
        "security-sem-service",
        json!({
          "policy_name": req.name
        }),
    );
    state.events.lock().unwrap().push(event.clone());

    Json(json!({
      "status": "updated",
      "event_id": event.event_id
    }))
}

async fn scan_sde(
    State(state): State<Arc<SEMState>>,
    Json(req): Json<ScanRequest>,
) -> Json<serde_json::Value> {
    let compromised = req.sde_id.ends_with("7");
    let mut response = json!({
      "sde_id": req.sde_id,
      "status": "safe"
    });

    if compromised {
        let threat = Threat {
            id: format!("thr-{}", chrono::Utc::now().timestamp_millis()),
            sde_id: req.sde_id.clone(),
            threat_type: "suspicious_dependency".to_string(),
            severity: "high".to_string(),
        };
        state.threats.lock().unwrap().push(threat.clone());
        let event = EventEnvelope::new(
            EventTopic::SecurityEvents,
            "THREAT_DETECTED",
            "security-sem-service",
            json!({
              "sde_id": threat.sde_id,
              "severity": threat.severity,
              "containment": "quarantine_sde"
            }),
        );
        state.events.lock().unwrap().push(event.clone());
        response = json!({
          "sde_id": req.sde_id,
          "status": "quarantined",
          "threat": threat,
          "event_id": event.event_id
        });
    }

    Json(response)
}

async fn get_events(State(state): State<Arc<SEMState>>) -> Json<serde_json::Value> {
    let events = state.events.lock().unwrap().clone();
    Json(json!({
      "count": events.len(),
      "items": events
    }))
}
