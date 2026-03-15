//! Axum HTTP route handlers for the AI Agents service.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use serde_json::json;

use qala_shared::events::{EventEnvelope, EventTopic, RecommendationGeneratedEvent};

use crate::{
    analyzer::{AnalyzeSdeRequest, AnalysisResult},
    recommendations::RecommendationQuery,
    state::AgentState,
};

// ── GET /health ────────────────────────────────────────────────────────────────

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({
        "service": "ai-agents-service",
        "status":  "ok",
        "ts":      Utc::now(),
    })))
}

// ── GET /recommendations ──────────────────────────────────────────────────────

pub async fn list_recommendations(
    State(state): State<Arc<AgentState>>,
    Query(q): Query<RecommendationQuery>,
) -> impl IntoResponse {
    let store = state.recommendations.read().await;
    let recs = store.query(&q);

    let count = recs.len();
    let active = store.active_count();
    let total  = store.count();

    (StatusCode::OK, Json(json!({
        "count":         count,
        "active_total":  active,
        "stored_total":  total,
        "recommendations": recs,
    })))
}

// ── POST /analyze_sde ─────────────────────────────────────────────────────────

pub async fn analyze_sde(
    State(state): State<Arc<AgentState>>,
    Json(req): Json<AnalyzeSdeRequest>,
) -> impl IntoResponse {
    let sde_id = req.sde_id;
    tracing::info!(sde_id = %sde_id, "SDE analysis requested");

    // Run the analysis
    let result: AnalysisResult = crate::analyzer::run(&req).await;

    // Persist generated recommendations
    {
        let mut store = state.recommendations.write().await;
        for rec in &result.recommendations {
            store.upsert(rec.clone());
        }
    }

    // Emit events to kernel for each high-confidence recommendation
    emit_recommendation_events(&state, &result).await;

    (StatusCode::OK, Json(result))
}

// ── Event emission helper ─────────────────────────────────────────────────────

async fn emit_recommendation_events(state: &AgentState, result: &AnalysisResult) {
    for rec in result.recommendations.iter().filter(|r| r.is_high_confidence()) {
        let payload = RecommendationGeneratedEvent {
            recommendation_id: rec.id,
            sde_id:            rec.sde_id,
            category:          rec.category.to_string(),
            title:             rec.title.clone(),
            confidence:        rec.confidence,
        };

        let envelope = EventEnvelope::new(
            EventTopic::AiRecommendations,
            "ai.recommendation_generated",
            "ai-agents-service",
            rec.sde_id,
            serde_json::to_value(&payload).unwrap_or_default(),
        );

        if let Ok(body) = serde_json::to_value(&envelope) {
            let kernel_url = &state.kernel_url;
            let client = state.http_client.clone();
            let url = format!("{kernel_url}/v1/kernel/events");

            tokio::spawn(async move {
                if let Err(e) = client.post(&url).json(&body).send().await {
                    tracing::warn!(error = %e, "failed to emit event to kernel");
                }
            });
        }
    }
}
