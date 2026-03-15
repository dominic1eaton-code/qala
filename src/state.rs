//! `KernelState` — the root in-memory state of the qala kernel.
//!
//! A single `Arc<KernelState>` is shared across all Axum handler tasks.
//! All inner collections are protected by `tokio::sync::RwLock` for concurrent
//! read-heavy access patterns typical of status queries.

use std::sync::Arc;

use tokio::sync::RwLock;

use crate::{
    events::EventAggregator,
    registry::ServiceRegistry,
};

/// The shared, thread-safe state of the qala kernel.
///
/// This is the central coordination point for all subsystem data:
/// - Service registration and heartbeat tracking
/// - Event counters and recent event ring buffer
/// - Subsystem status (updated via the command bus)
#[derive(Debug)]
pub struct KernelState {
    /// Registry of all services that have registered with the kernel.
    pub registry: Arc<RwLock<ServiceRegistry>>,
    /// Aggregated event counters and recent events.
    pub events: Arc<RwLock<EventAggregator>>,
    /// Subsystem statuses keyed by subsystem name.
    pub subsystems: Arc<RwLock<SubsystemMap>>,
}

impl KernelState {
    /// Create a new, empty `KernelState` with all subsystems in `Initialising`.
    pub fn new() -> Self {
        let subsystems = SubsystemMap::with_known_subsystems();
        Self {
            registry: Arc::new(RwLock::new(ServiceRegistry::new())),
            events:   Arc::new(RwLock::new(EventAggregator::new(500))),
            subsystems: Arc::new(RwLock::new(subsystems)),
        }
    }
}

impl Default for KernelState {
    fn default() -> Self {
        Self::new()
    }
}

// ── Subsystem map ─────────────────────────────────────────────────────────────

use std::collections::HashMap;
use qala_shared::types::SubsystemStatus;

/// Map of subsystem name → current status.
#[derive(Debug)]
pub struct SubsystemMap(pub HashMap<String, SubsystemStatus>);

impl SubsystemMap {
    /// Initialise with all known qala subsystems set to `Initialising`.
    pub fn with_known_subsystems() -> Self {
        let mut map = HashMap::new();
        for name in KNOWN_SUBSYSTEMS {
            map.insert(name.to_string(), SubsystemStatus::Initialising);
        }
        Self(map)
    }

    pub fn set_status(&mut self, subsystem: &str, status: SubsystemStatus) {
        self.0.insert(subsystem.to_string(), status);
    }

    pub fn get_status(&self, subsystem: &str) -> SubsystemStatus {
        self.0
            .get(subsystem)
            .cloned()
            .unwrap_or(SubsystemStatus::Unknown)
    }

    pub fn all(&self) -> &HashMap<String, SubsystemStatus> {
        &self.0
    }
}

/// The canonical set of subsystem names tracked by the kernel.
pub const KNOWN_SUBSYSTEMS: &[&str] = &[
    // Kernel
    "registry", "command-bus", "event-aggregator",
    // Control Plane
    "gateway", "identity", "sde", "workspace", "artifact",
    // Execution Plane
    "workflow", "data", "notifications",
    // Intelligence & Security
    "ai", "sem",
];
