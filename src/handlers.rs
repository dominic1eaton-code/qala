//! Axum HTTP route handlers for the qala kernel.
//!
//! All handlers receive a shared `Arc<KernelState>` via Axum's `State` extractor.
//! Errors are mapped to JSON responses using the `QalaError` status codes.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;

use qala_shared::events::EventEnvelope;

use crate::{
    commands::{KernelCommandRequest, KernelCommandResponse},
    events::{EventAggregatorSummary, EventQuery},
    registry::{RegisterServiceRequest, RegisterServiceResponse},
    state::KernelState,
};

// ── Error response helper ─────────────────────────────────────────────────────

/// JSON error body returned on all failed responses.
#[derive(Serialize)]
struct ErrorBody {
    code:    &'static str,
    message: String,
}

fn err_response(status: StatusCode, code: &'static str, msg: impl Into<String>) -> impl IntoResponse {
    (status, Json(ErrorBody { code, message: msg.into() }))
}

macro_rules! map_err {
    ($res:expr) => {
        match $res {
            Ok(v)  => v,
            Err(e) => {
                let status = StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
                return err_response(status, e.code(), e.to_string()).into_response();
            }
        }
    };
}

// ── GET /health ────────────────────────────────────────────────────────────────

/// Liveness probe — always returns 200 OK if the process is running.
pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({
        "service":  "qala-kernel",
        "status":   "ok",
        "ts":       Utc::now(),
    })))
}

// ── GET /v1/kernel/status ─────────────────────────────────────────────────────

/// Full kernel status snapshot: subsystems, registered services, event summary.
#[derive(Serialize)]
pub struct KernelStatusResponse {
    pub kernel_version:  String,
    pub ts:              chrono::DateTime<Utc>,
    pub subsystems:      std::collections::HashMap<String, String>,
    pub services:        Vec<ServiceSummary>,
    pub events:          EventAggregatorSummary,
}

#[derive(Serialize)]
pub struct ServiceSummary {
    pub name:            String,
    pub subsystem:       String,
    pub address:         String,
    pub status:          String,
    pub version:         String,
    pub last_heartbeat:  chrono::DateTime<Utc>,
    pub missed_heartbeats: u32,
}

pub async fn get_status(State(state): State<Arc<KernelState>>) -> impl IntoResponse {
    let subsystems = {
        let map = state.subsystems.read().await;
        map.all()
            .iter()
            .map(|(k, v)| (k.clone(), v.to_string()))
            .collect()
    };

    let services = {
        let registry = state.registry.read().await;
        registry
            .all()
            .map(|svc| ServiceSummary {
                name:             svc.name.clone(),
                subsystem:        svc.subsystem.clone(),
                address:          svc.address.clone(),
                status:           svc.status.to_string(),
                version:          svc.version.clone(),
                last_heartbeat:   svc.last_heartbeat,
                missed_heartbeats: svc.missed_heartbeats,
            })
            .collect()
    };

    let events = {
        let agg = state.events.read().await;
        agg.summary()
    };

    (StatusCode::OK, Json(KernelStatusResponse {
        kernel_version: env!("CARGO_PKG_VERSION").to_string(),
        ts:             Utc::now(),
        subsystems,
        services,
        events,
    }))
}

// ── POST /v1/kernel/register-service ─────────────────────────────────────────

pub async fn register_service(
    State(state): State<Arc<KernelState>>,
    Json(req): Json<RegisterServiceRequest>,
) -> impl IntoResponse {
    tracing::info!(
        service = %req.name,
        subsystem = %req.subsystem,
        address = %req.address,
        "service registration request"
    );

    let record = {
        let mut registry = state.registry.write().await;
        registry.register(&req)
    };

    // Transition the corresponding subsystem to Healthy
    {
        let mut subsystems = state.subsystems.write().await;
        subsystems.set_status(&req.subsystem, qala_shared::types::SubsystemStatus::Healthy);
    }

    let resp = RegisterServiceResponse {
        service_id:    record.id,
        name:          record.name,
        registered_at: record.registered_at,
        message:       "service registered successfully".to_string(),
    };

    (StatusCode::CREATED, Json(resp))
}

// ── POST /v1/kernel/events ────────────────────────────────────────────────────

pub async fn ingest_event(
    State(state): State<Arc<KernelState>>,
    Json(envelope): Json<EventEnvelope>,
) -> impl IntoResponse {
    tracing::debug!(
        topic        = %envelope.topic,
        event_type   = %envelope.event_type,
        source       = %envelope.source_service,
        event_id     = %envelope.event_id,
        "event ingested"
    );

    {
        let mut agg = state.events.write().await;
        agg.ingest(envelope);
    }

    (StatusCode::ACCEPTED, Json(json!({ "accepted": true })))
}

// ── GET /v1/kernel/events ─────────────────────────────────────────────────────

pub async fn query_events(
    State(state): State<Arc<KernelState>>,
    Query(q): Query<EventQuery>,
) -> impl IntoResponse {
    let events: Vec<serde_json::Value> = {
        let agg = state.events.read().await;
        agg.query(&q)
            .into_iter()
            .map(|e| serde_json::to_value(e).unwrap_or(serde_json::Value::Null))
            .collect()
    };

    let count = events.len();

    (StatusCode::OK, Json(json!({
        "count":  count,
        "events": events,
    })))
}

// ── POST /v1/kernel/command ───────────────────────────────────────────────────

pub async fn execute_command(
    State(state): State<Arc<KernelState>>,
    Json(req): Json<KernelCommandRequest>,
) -> impl IntoResponse {
    tracing::info!(command = %req.command, "kernel command received");

    let result = map_err!(crate::commands::execute(&state, req).await);

    let status = if result.success {
        StatusCode::OK
    } else {
        StatusCode::UNPROCESSABLE_ENTITY
    };

    (status, Json(result)).into_response()
}
