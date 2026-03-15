//! Command Bus — executes platform-level kernel commands.
//!
//! Commands are synchronous, state-mutating operations issued to the kernel.
//! They differ from events (which are async telemetry) in that they directly
//! change kernel state and return an acknowledgement.
//!
//! ## Supported commands
//!
//! | Command                 | Parameters                     | Effect                                |
//! |-------------------------|--------------------------------|---------------------------------------|
//! | `set_subsystem_status`  | `subsystem`, `status`          | Update a subsystem's reported status  |
//! | `set_service_status`    | `service_name`, `status`       | Update a specific service's status    |
//! | `service_heartbeat`     | `service_name`                 | Record a liveness heartbeat           |
//! | `reset_event_counters`  | (none)                         | Clear all event counters              |

use serde::{Deserialize, Serialize};

use qala_shared::{
    errors::{QalaError, QalaResult},
    types::SubsystemStatus,
};

use crate::state::KernelState;

// ── Command request / response wire types ─────────────────────────────────────

/// The incoming command payload posted to `POST /v1/kernel/command`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelCommandRequest {
    /// The command discriminant.
    pub command: String,
    /// Command-specific parameters as a JSON object.
    #[serde(default)]
    pub params: serde_json::Value,
}

/// Standard response from any successfully executed kernel command.
#[derive(Debug, Serialize, Deserialize)]
pub struct KernelCommandResponse {
    pub command:    String,
    pub success:    bool,
    pub message:    String,
    /// Optional structured result data (command-specific).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data:       Option<serde_json::Value>,
}

impl KernelCommandResponse {
    pub fn ok(command: &str, message: impl Into<String>) -> Self {
        Self {
            command: command.to_string(),
            success: true,
            message: message.into(),
            data:    None,
        }
    }

    pub fn ok_with_data(
        command: &str,
        message: impl Into<String>,
        data: serde_json::Value,
    ) -> Self {
        Self {
            command: command.to_string(),
            success: true,
            message: message.into(),
            data:    Some(data),
        }
    }
}

// ── Command router ─────────────────────────────────────────────────────────────

/// Route and execute a kernel command against the shared state.
///
/// This is the single entry point for all command execution. The `command`
/// field of the request is used to dispatch to the appropriate handler.
pub async fn execute(
    state: &KernelState,
    req: KernelCommandRequest,
) -> QalaResult<KernelCommandResponse> {
    tracing::debug!(command = %req.command, "executing kernel command");

    match req.command.as_str() {
        "set_subsystem_status" => cmd_set_subsystem_status(state, &req.params).await,
        "set_service_status"   => cmd_set_service_status(state, &req.params).await,
        "service_heartbeat"    => cmd_service_heartbeat(state, &req.params).await,
        "reset_event_counters" => cmd_reset_event_counters(state).await,
        other => Err(QalaError::UnknownKernelCommand {
            command: other.to_string(),
        }),
    }
}

// ── Command handlers ──────────────────────────────────────────────────────────

/// `set_subsystem_status` — update the status of a named subsystem.
///
/// Parameters: `{ "subsystem": "<name>", "status": "<status>" }`
async fn cmd_set_subsystem_status(
    state: &KernelState,
    params: &serde_json::Value,
) -> QalaResult<KernelCommandResponse> {
    let subsystem = require_str(params, "subsystem")?;
    let status_str = require_str(params, "status")?;
    let status = parse_subsystem_status(status_str)?;

    {
        let mut subsystems = state.subsystems.write().await;
        subsystems.set_status(subsystem, status.clone());
    }

    tracing::info!(subsystem, status = %status, "subsystem status updated");

    Ok(KernelCommandResponse::ok(
        "set_subsystem_status",
        format!("subsystem '{subsystem}' set to '{status}'"),
    ))
}

/// `set_service_status` — update the reported status of a named service.
///
/// Parameters: `{ "service_name": "<name>", "status": "<status>" }`
async fn cmd_set_service_status(
    state: &KernelState,
    params: &serde_json::Value,
) -> QalaResult<KernelCommandResponse> {
    let service_name = require_str(params, "service_name")?;
    let status_str   = require_str(params, "status")?;
    let status       = parse_subsystem_status(status_str)?;

    let updated = {
        let mut registry = state.registry.write().await;
        registry.set_status(service_name, status.clone())
    };

    if !updated {
        return Err(QalaError::NotFoundByName {
            resource_type: "service",
            name: service_name.to_string(),
        });
    }

    tracing::info!(service = service_name, status = %status, "service status updated");

    Ok(KernelCommandResponse::ok(
        "set_service_status",
        format!("service '{service_name}' set to '{status}'"),
    ))
}

/// `service_heartbeat` — record a liveness heartbeat from a named service.
///
/// Parameters: `{ "service_name": "<name>" }`
async fn cmd_service_heartbeat(
    state: &KernelState,
    params: &serde_json::Value,
) -> QalaResult<KernelCommandResponse> {
    let service_name = require_str(params, "service_name")?;

    let found = {
        let mut registry = state.registry.write().await;
        registry.heartbeat(service_name)
    };

    if !found {
        return Err(QalaError::NotFoundByName {
            resource_type: "service",
            name: service_name.to_string(),
        });
    }

    tracing::debug!(service = service_name, "heartbeat received");

    Ok(KernelCommandResponse::ok(
        "service_heartbeat",
        format!("heartbeat recorded for '{service_name}'"),
    ))
}

/// `reset_event_counters` — clear all event counters (useful for testing / reset).
///
/// Parameters: (none)
async fn cmd_reset_event_counters(
    state: &KernelState,
) -> QalaResult<KernelCommandResponse> {
    // Re-initialise the aggregator with the same capacity
    let capacity = {
        let agg = state.events.read().await;
        agg.capacity()
    };

    {
        let mut agg = state.events.write().await;
        *agg = crate::events::EventAggregator::new(capacity);
    }

    tracing::info!("event counters reset");

    Ok(KernelCommandResponse::ok(
        "reset_event_counters",
        "all event counters cleared",
    ))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Extract a required string field from a JSON params object.
fn require_str<'a>(params: &'a serde_json::Value, key: &str) -> QalaResult<&'a str> {
    params
        .get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| QalaError::InvalidRequest {
            reason: format!("missing required parameter '{key}'"),
        })
}

/// Parse a `SubsystemStatus` from a string, returning `InvalidRequest` on failure.
fn parse_subsystem_status(s: &str) -> QalaResult<SubsystemStatus> {
    match s {
        "initialising" | "Initialising" => Ok(SubsystemStatus::Initialising),
        "healthy"      | "Healthy"      => Ok(SubsystemStatus::Healthy),
        "degraded"     | "Degraded"     => Ok(SubsystemStatus::Degraded),
        "offline"      | "Offline"      => Ok(SubsystemStatus::Offline),
        "unknown"      | "Unknown"      => Ok(SubsystemStatus::Unknown),
        other => Err(QalaError::InvalidRequest {
            reason: format!(
                "unknown subsystem status '{other}'; \
                 valid values: initialising, healthy, degraded, offline, unknown"
            ),
        }),
    }
}
