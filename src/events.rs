//! Platform-wide event contract for the qala SFOS.
//!
//! All async communication between services uses `EventEnvelope` as the
//! standard wire format. Events are published to Kafka topics following the
//! topic taxonomy defined in `EventTopic`.
//!
//! Event flow:
//!   Service action  →  emit EventEnvelope  →  Kafka topic  →  consumers
//!   Kernel aggregates counters from all topics via POST /v1/kernel/events

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Topic taxonomy ─────────────────────────────────────────────────────────────

/// All platform event stream topics.
///
/// Maps 1:1 with the Kafka topic names used across the qala infrastructure.
/// Each service publishes only to its own topic(s); consumers may subscribe
/// to multiple topics.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventTopic {
    /// Published by: User & Identity Service
    UserEvents,
    /// Published by: SDE Management Service
    SdeEvents,
    /// Published by: Workspace & CMS Service
    CmsEvents,
    /// Published by: Workflow & CI/CD Service
    BuildEvents,
    /// Published by: Artifact & Package Management Service
    ArtifactEvents,
    /// Published by: Data Platform Service
    DataEvents,
    /// Published by: Security & SEM Service
    SecurityEvents,
    /// Published by: Notifications Service
    Notifications,
    /// Published by: AI Agents Service
    AiRecommendations,
}

impl std::fmt::Display for EventTopic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            EventTopic::UserEvents        => "USER_EVENTS",
            EventTopic::SdeEvents         => "SDE_EVENTS",
            EventTopic::CmsEvents         => "CMS_EVENTS",
            EventTopic::BuildEvents       => "BUILD_EVENTS",
            EventTopic::ArtifactEvents    => "ARTIFACT_EVENTS",
            EventTopic::DataEvents        => "DATA_EVENTS",
            EventTopic::SecurityEvents    => "SECURITY_EVENTS",
            EventTopic::Notifications     => "NOTIFICATIONS",
            EventTopic::AiRecommendations => "AI_RECOMMENDATIONS",
        };
        write!(f, "{s}")
    }
}

// ── Event envelope ─────────────────────────────────────────────────────────────

/// Standard event envelope used for all async inter-service communication.
///
/// The `payload` field carries the topic-specific JSON event body. Receivers
/// deserialise `payload` based on `event_type` after parsing the envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// Globally unique identifier for this event.
    pub event_id: Uuid,
    /// The Kafka topic this event belongs to.
    pub topic: EventTopic,
    /// The machine-readable event type discriminant (e.g. `"sde.created"`).
    pub event_type: String,
    /// The service that emitted this event (e.g. `"sde-management-service"`).
    pub source_service: String,
    /// ISO 8601 timestamp when the event was produced.
    pub occurred_at: DateTime<Utc>,
    /// The subject resource's ID (optional — not all events have a resource).
    pub resource_id: Option<Uuid>,
    /// The event-specific payload as a raw JSON value.
    pub payload: serde_json::Value,
    /// Schema version for forward compatibility. Increment on breaking changes.
    pub schema_version: u8,
}

impl EventEnvelope {
    /// Construct a new envelope, generating a fresh event_id and timestamp.
    pub fn new(
        topic: EventTopic,
        event_type: impl Into<String>,
        source_service: impl Into<String>,
        resource_id: Option<Uuid>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            topic,
            event_type: event_type.into(),
            source_service: source_service.into(),
            occurred_at: Utc::now(),
            resource_id,
            payload,
            schema_version: 1,
        }
    }

    /// Serialise this envelope to a JSON string suitable for Kafka publication.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialise an envelope from a Kafka message payload.
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

// ── Typed event payloads ───────────────────────────────────────────────────────

// SDE_EVENTS ──────────────────────────────────────────────────────────────────

/// Payload for `sde.created`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdeCreatedEvent {
    pub sde_id:     Uuid,
    pub owner_id:   Uuid,
    pub name:       String,
    pub factory_id: Option<Uuid>,
}

/// Payload for `sde.state_changed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdeStateChangedEvent {
    pub sde_id:    Uuid,
    pub from_state: String,
    pub to_state:   String,
}

/// Payload for `sde.snapshot_created`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdeSnapshotCreatedEvent {
    pub sde_id:      Uuid,
    pub snapshot_id: Uuid,
    pub version:     u32,
    pub label:       String,
}

/// Payload for `sde.rollback_completed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdeRollbackCompletedEvent {
    pub sde_id:            Uuid,
    pub restored_version:  u32,
    pub previous_version:  u32,
}

// BUILD_EVENTS ─────────────────────────────────────────────────────────────────

/// Payload for `build.started`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildStartedEvent {
    pub pipeline_id: Uuid,
    pub sde_id:      Uuid,
    pub commit_sha:  String,
    pub branch:      String,
}

/// Payload for `build.completed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildCompletedEvent {
    pub pipeline_id:    Uuid,
    pub sde_id:         Uuid,
    pub success:        bool,
    pub duration_secs:  u32,
    pub coverage_pct:   Option<f32>,
    pub artifact_ids:   Vec<Uuid>,
}

// SECURITY_EVENTS ─────────────────────────────────────────────────────────────

/// Payload for `security.threat_detected`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatDetectedEvent {
    pub threat_id:   Uuid,
    pub sde_id:      Option<Uuid>,
    pub cve_id:      Option<String>,
    pub severity:    String,
    pub description: String,
}

/// Payload for `security.policy_violated`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolatedEvent {
    pub policy_name: String,
    pub sde_id:      Option<Uuid>,
    pub detail:      String,
}

// AI_RECOMMENDATIONS ──────────────────────────────────────────────────────────

/// Payload for `ai.recommendation_generated`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationGeneratedEvent {
    pub recommendation_id: Uuid,
    pub sde_id:            Option<Uuid>,
    pub category:          String,
    pub title:             String,
    pub confidence:        f32,
}

// ARTIFACT_EVENTS ─────────────────────────────────────────────────────────────

/// Payload for `artifact.uploaded`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactUploadedEvent {
    pub artifact_id:  Uuid,
    pub sde_id:       Uuid,
    pub name:         String,
    pub version:      String,
    pub size_bytes:   u64,
    pub signed:       bool,
}

// USER_EVENTS ─────────────────────────────────────────────────────────────────

/// Payload for `user.created`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCreatedEvent {
    pub user_id: Uuid,
    pub email:   String,
    pub name:    String,
}
