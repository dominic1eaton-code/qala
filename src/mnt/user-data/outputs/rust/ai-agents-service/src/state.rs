//! Shared state for the AI Agents service.

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::recommendations::RecommendationStore;

/// Shared, thread-safe state for the AI agents service.
#[derive(Debug)]
pub struct AgentState {
    /// Store of active and historical recommendations.
    pub recommendations: Arc<RwLock<RecommendationStore>>,
    /// HTTP client for outbound calls (kernel registration, SDE data fetching).
    pub http_client: reqwest::Client,
    /// Base URL of the kernel service.
    pub kernel_url: String,
    /// Base URL of the SDE management service.
    pub sde_service_url: String,
}

impl AgentState {
    pub fn new() -> Self {
        let kernel_url = std::env::var("KERNEL_URL")
            .unwrap_or_else(|_| "http://localhost:7000".to_string());
        let sde_service_url = std::env::var("SDE_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8082".to_string());

        Self {
            recommendations: Arc::new(RwLock::new(RecommendationStore::new())),
            http_client:     reqwest::Client::new(),
            kernel_url,
            sde_service_url,
        }
    }
}

impl Default for AgentState {
    fn default() -> Self {
        Self::new()
    }
}
