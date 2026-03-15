//! Service Registry — tracks all services that have registered with the kernel,
//! their addresses, heartbeat timestamps, and reported health.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use qala_shared::types::SubsystemStatus;

// ── Service record ─────────────────────────────────────────────────────────────

/// A service that has registered itself with the kernel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredService {
    pub id: Uuid,
    /// Machine name (e.g. `"sde-management-service"`).
    pub name: String,
    /// Which qala subsystem this service implements.
    pub subsystem: String,
    /// Base URL the service can be reached at (e.g. `"http://sde-svc:8082"`).
    pub address: String,
    /// Port the service listens on.
    pub port: u16,
    /// Programming language of the service implementation.
    pub language: String,
    /// Semantic version of the running service binary.
    pub version: String,
    /// Current reported status.
    pub status: SubsystemStatus,
    /// Time the service first registered.
    pub registered_at: DateTime<Utc>,
    /// Time of the most recent heartbeat (or `registered_at` if never updated).
    pub last_heartbeat: DateTime<Utc>,
    /// Number of heartbeat cycles missed (0 = healthy).
    pub missed_heartbeats: u32,
}

impl RegisteredService {
    pub fn new(req: &RegisterServiceRequest) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: req.name.clone(),
            subsystem: req.subsystem.clone(),
            address: req.address.clone(),
            port: req.port,
            language: req.language.clone(),
            version: req.version.clone(),
            status: SubsystemStatus::Healthy,
            registered_at: now,
            last_heartbeat: now,
            missed_heartbeats: 0,
        }
    }

    /// Mark a heartbeat received now.
    pub fn heartbeat(&mut self) {
        self.last_heartbeat = Utc::now();
        self.missed_heartbeats = 0;
        self.status = SubsystemStatus::Healthy;
    }

    /// Seconds since last heartbeat.
    pub fn seconds_since_heartbeat(&self) -> i64 {
        (Utc::now() - self.last_heartbeat).num_seconds()
    }
}

// ── Request / Response types ───────────────────────────────────────────────────

/// Request body for `POST /v1/kernel/register-service`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterServiceRequest {
    pub name:      String,
    pub subsystem: String,
    pub address:   String,
    pub port:      u16,
    pub language:  String,
    pub version:   String,
}

/// Response for a successful registration.
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterServiceResponse {
    pub service_id:  Uuid,
    pub name:        String,
    pub registered_at: DateTime<Utc>,
    pub message:     String,
}

// ── Registry ───────────────────────────────────────────────────────────────────

/// In-memory service registry.
///
/// Keyed by service name so that re-registration from a restarted pod
/// updates the existing record rather than duplicating it.
#[derive(Debug)]
pub struct ServiceRegistry {
    services: HashMap<String, RegisteredService>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self { services: HashMap::new() }
    }

    /// Register or update a service. Returns a clone of the stored record.
    pub fn register(&mut self, req: &RegisterServiceRequest) -> RegisteredService {
        let entry = self
            .services
            .entry(req.name.clone())
            .or_insert_with(|| RegisteredService::new(req));

        // Update mutable fields on re-registration
        entry.address = req.address.clone();
        entry.port    = req.port;
        entry.version = req.version.clone();
        entry.heartbeat();

        entry.clone()
    }

    /// Record a heartbeat for the named service.
    /// Returns `false` if the service is not registered.
    pub fn heartbeat(&mut self, service_name: &str) -> bool {
        if let Some(svc) = self.services.get_mut(service_name) {
            svc.heartbeat();
            true
        } else {
            false
        }
    }

    /// Update the status of a named service.
    pub fn set_status(&mut self, service_name: &str, status: SubsystemStatus) -> bool {
        if let Some(svc) = self.services.get_mut(service_name) {
            svc.status = status;
            true
        } else {
            false
        }
    }

    /// Retrieve a service by name.
    pub fn get(&self, name: &str) -> Option<&RegisteredService> {
        self.services.get(name)
    }

    /// Iterate over all registered services.
    pub fn all(&self) -> impl Iterator<Item = &RegisteredService> {
        self.services.values()
    }

    /// Count of registered services.
    pub fn count(&self) -> usize {
        self.services.len()
    }

    /// Mark services that have exceeded the heartbeat threshold as `Degraded`.
    /// Returns the list of service names that were degraded.
    pub fn check_heartbeats(&mut self, threshold_secs: i64) -> Vec<String> {
        let mut degraded = Vec::new();
        for svc in self.services.values_mut() {
            if svc.seconds_since_heartbeat() > threshold_secs
                && svc.status == SubsystemStatus::Healthy
            {
                svc.status = SubsystemStatus::Degraded;
                svc.missed_heartbeats += 1;
                degraded.push(svc.name.clone());
            }
        }
        degraded
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
