use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EventTopic {
    #[serde(rename = "USER_EVENTS")]
    UserEvents,
    #[serde(rename = "SDE_EVENTS")]
    SDEEvents,
    #[serde(rename = "CMS_EVENTS")]
    CMSEvents,
    #[serde(rename = "BUILD_EVENTS")]
    BuildEvents,
    #[serde(rename = "ARTIFACT_EVENTS")]
    ArtifactEvents,
    #[serde(rename = "DATA_EVENTS")]
    DataEvents,
    #[serde(rename = "SECURITY_EVENTS")]
    SecurityEvents,
    #[serde(rename = "NOTIFICATIONS")]
    Notifications,
    #[serde(rename = "AI_RECOMMENDATIONS")]
    AIRecommendations,
}

impl EventTopic {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventTopic::UserEvents => "USER_EVENTS",
            EventTopic::SDEEvents => "SDE_EVENTS",
            EventTopic::CMSEvents => "CMS_EVENTS",
            EventTopic::BuildEvents => "BUILD_EVENTS",
            EventTopic::ArtifactEvents => "ARTIFACT_EVENTS",
            EventTopic::DataEvents => "DATA_EVENTS",
            EventTopic::SecurityEvents => "SECURITY_EVENTS",
            EventTopic::Notifications => "NOTIFICATIONS",
            EventTopic::AIRecommendations => "AI_RECOMMENDATIONS",
        }
    }
}

impl fmt::Display for EventTopic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: Uuid,
    pub event_type: String,
    pub topic: EventTopic,
    pub source_service: String,
    pub correlation_id: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub payload: Value,
}

impl EventEnvelope {
    pub fn new(topic: EventTopic, event_type: impl Into<String>, source_service: impl Into<String>, payload: Value) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type: event_type.into(),
            topic,
            source_service: source_service.into(),
            correlation_id: None,
            occurred_at: Utc::now(),
            payload,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistration {
    pub service_name: String,
    pub system_name: String,
    pub subsystem_name: String,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelCommand {
    pub command: String,
    pub target: String,
    pub value: String,
}
