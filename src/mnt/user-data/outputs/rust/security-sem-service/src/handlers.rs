//! Axum HTTP route handlers for the Security SEM service.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use serde_json::json;

use qala_shared::events::{EventEnvelope, EventTopic, ThreatDetectedEvent, PolicyViolatedEvent};

use crate::{
    policies::PolicyUpdateRequest,
    scanner::ScanSdeRequest,
    state::SemState,
    threats::ThreatQuery,
};

// ── GET /health ────────────────────────────────────────────────────────────────

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({
        "service": "security-sem-service",
        "status":  "ok",
        "ts":      Utc::now(),
    })))
}

// ── GET /threats ───────────────────────────────────────────────────────────────

pub async fn list_threats(
    State(state): State<Arc<SemState>>,
    Query(q): Query<ThreatQuery>,
) -> impl IntoResponse {
    let store   = state.threats.read().await;
    let threats = store.query(&q);

    (StatusCode::OK, Json(json!({
        "count":        threats.len(),
        "active_total": store.active_count(),
        "stored_total": store.count(),
        "threats":      threats,
    })))
}

// ── POST /policy/update ───────────────────────────────────────────────────────

pub async fn update_policy(
    State(state): State<Arc<SemState>>,
    Json(req): Json<PolicyUpdateRequest>,
) -> impl IntoResponse {
    tracing::info!(policy = %req.name, "policy update requested");

    let policy_id = {
        let mut store = state.policies.write().await;
        store.apply_update(&req)
    };

    // Emit a policy violation event to the kernel if status isn't active
    // (in production we'd emit policy.updated events on all changes)
    emit_event(
        &state,
        EventTopic::SecurityEvents,
        "security.policy_updated",
        None,
        json!({
            "policy_id":   policy_id,
            "policy_name": req.name,
        }),
    )
    .await;

    (StatusCode::OK, Json(json!({
        "policy_id": policy_id,
        "name":      req.name,
        "message":   "policy updated successfully",
    })))
}

// ── POST /scan_sde ────────────────────────────────────────────────────────────

pub async fn scan_sde(
    State(state): State<Arc<SemState>>,
    Json(req): Json<ScanSdeRequest>,
) -> impl IntoResponse {
    let sde_id = req.sde_id;
    tracing::info!(sde_id = %sde_id, "SDE security scan requested");

    // Run the scanner
    let result = crate::scanner::run(&req).await;

    // Persist newly discovered threats
    let mut threat_ids = Vec::new();
    {
        let mut store = state.threats.write().await;
        for threat in &result.threats {
            let id = store.insert(threat.clone());
            threat_ids.push(id);
        }
    }

    // Emit security events for each finding
    for threat in &result.threats {
        let payload = ThreatDetectedEvent {
            threat_id:   threat.id,
            sde_id:      threat.sde_id,
            cve_id:      threat.cve_id.clone(),
            severity:    threat.severity.to_string(),
            description: threat.description.clone(),
        };
        emit_event(
            &state,
            EventTopic::SecurityEvents,
            "security.threat_detected",
            threat.sde_id,
            serde_json::to_value(&payload).unwrap_or_default(),
        )
        .await;
    }

    // Emit policy violation events for any policy-enforcement-class threats
    {
        let policies  = state.policies.read().await;
        let all_rules = policies.all_active();
        for rule_policy in all_rules {
            for rule in &rule_policy.rules {
                let violation_detected = match rule.rule_key.as_str() {
                    "no_plain_secrets_in_env" => result.threats.iter().any(|t| {
                        matches!(t.threat_type, crate::threats::ThreatType::CredentialExposure)
                    }),
                    "cve_high_block_deploy" => result.high_count > 0 || result.critical_count > 0,
                    _ => false,
                };

                if violation_detected {
                    let payload = PolicyViolatedEvent {
                        policy_name: rule.rule_key.clone(),
                        sde_id:      Some(sde_id),
                        detail:      rule.description.clone(),
                    };
                    emit_event(
                        &state,
                        EventTopic::SecurityEvents,
                        "security.policy_violated",
                        Some(sde_id),
                        serde_json::to_value(&payload).unwrap_or_default(),
                    )
                    .await;
                }
            }
        }
    }

    (StatusCode::OK, Json(result))
}

// ── Event emission helper ──────────────────────────────────────────────────────

async fn emit_event(
    state: &SemState,
    topic: EventTopic,
    event_type: &'static str,
    resource_id: Option<uuid::Uuid>,
    payload: serde_json::Value,
) {
    let envelope = EventEnvelope::new(
        topic,
        event_type,
        "security-sem-service",
        resource_id,
        payload,
    );

    let url    = format!("{}/v1/kernel/events", state.kernel_url);
    let client = state.http_client.clone();

    tokio::spawn(async move {
        if let Err(e) = client.post(&url).json(&envelope).send().await {
            tracing::warn!(error = %e, event_type, "failed to emit event to kernel");
        }
    });
}
