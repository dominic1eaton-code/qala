//! Platform-wide error types for the qala SFOS.
//!
//! All services should return `QalaResult<T>` and map lower-level errors
//! into the appropriate `QalaError` variant before propagating to HTTP handlers.

use thiserror::Error;
use uuid::Uuid;

/// Canonical result type used throughout the qala platform.
pub type QalaResult<T> = Result<T, QalaError>;

/// Top-level error enum covering all failure modes in the qala platform.
#[derive(Debug, Error)]
pub enum QalaError {
    // ── Domain errors ──────────────────────────────────────────────────────

    /// A requested resource was not found.
    #[error("not found: {resource_type} with id {id}")]
    NotFound { resource_type: &'static str, id: Uuid },

    /// A requested resource was not found by name.
    #[error("not found: {resource_type} '{name}'")]
    NotFoundByName { resource_type: &'static str, name: String },

    /// The caller does not have permission to perform this action.
    #[error("forbidden: {reason}")]
    Forbidden { reason: String },

    /// The request payload or parameters are invalid.
    #[error("invalid request: {reason}")]
    InvalidRequest { reason: String },

    /// The requested state transition is not allowed from the current state.
    #[error("invalid state transition: cannot move {resource_type} from {from} to {to}")]
    InvalidStateTransition {
        resource_type: &'static str,
        from: String,
        to: String,
    },

    /// A resource with the given identity already exists.
    #[error("conflict: {resource_type} '{name}' already exists")]
    AlreadyExists { resource_type: &'static str, name: String },

    // ── SDE / Factory errors ───────────────────────────────────────────────

    /// Snapshot creation failed.
    #[error("snapshot failed for SDE {sde_id}: {reason}")]
    SnapshotFailed { sde_id: Uuid, reason: String },

    /// Rollback to the requested snapshot version failed.
    #[error("rollback failed for SDE {sde_id} to version {version}: {reason}")]
    RollbackFailed { sde_id: Uuid, version: u32, reason: String },

    /// The SDE is in a lifecycle state that prevents this operation.
    #[error("SDE {sde_id} is in state '{state}' which does not allow '{operation}'")]
    SdeStateConflict {
        sde_id: Uuid,
        state: String,
        operation: &'static str,
    },

    /// Factory capacity exceeded.
    #[error("factory {factory_id} has reached its SDE capacity of {max_sdes}")]
    FactoryCapacityExceeded { factory_id: Uuid, max_sdes: usize },

    // ── Kernel errors ──────────────────────────────────────────────────────

    /// A kernel command was not recognised.
    #[error("unknown kernel command: '{command}'")]
    UnknownKernelCommand { command: String },

    /// Service registration failed.
    #[error("service registration failed for '{service_name}': {reason}")]
    ServiceRegistrationFailed { service_name: String, reason: String },

    // ── Security errors ────────────────────────────────────────────────────

    /// A policy rule rejected the operation.
    #[error("policy violation: {policy_name} — {detail}")]
    PolicyViolation { policy_name: String, detail: String },

    /// A plaintext secret was detected where vault storage is required.
    #[error("secret hygiene violation: plaintext credential detected in {location}")]
    PlaintextSecretDetected { location: String },

    // ── Infrastructure / transport errors ──────────────────────────────────

    /// An internal error occurred (unexpected / unrecoverable).
    #[error("internal error: {0}")]
    Internal(String),

    /// A downstream service call failed.
    #[error("upstream service error from '{service}': {reason}")]
    UpstreamError { service: String, reason: String },

    /// JSON serialisation / deserialisation failed.
    #[error("serialisation error: {0}")]
    Serialisation(#[from] serde_json::Error),
}

impl QalaError {
    /// HTTP status code appropriate for this error variant.
    pub fn status_code(&self) -> u16 {
        match self {
            QalaError::NotFound { .. } | QalaError::NotFoundByName { .. } => 404,
            QalaError::Forbidden { .. } => 403,
            QalaError::InvalidRequest { .. } => 400,
            QalaError::InvalidStateTransition { .. } => 422,
            QalaError::AlreadyExists { .. } => 409,
            QalaError::PolicyViolation { .. } => 403,
            QalaError::FactoryCapacityExceeded { .. } => 422,
            QalaError::SdeStateConflict { .. } => 409,
            QalaError::PlaintextSecretDetected { .. } => 400,
            QalaError::UnknownKernelCommand { .. } => 400,
            _ => 500,
        }
    }

    /// Machine-readable error code string.
    pub fn code(&self) -> &'static str {
        match self {
            QalaError::NotFound { .. } | QalaError::NotFoundByName { .. } => "NOT_FOUND",
            QalaError::Forbidden { .. } => "FORBIDDEN",
            QalaError::InvalidRequest { .. } => "INVALID_REQUEST",
            QalaError::InvalidStateTransition { .. } => "INVALID_STATE_TRANSITION",
            QalaError::AlreadyExists { .. } => "ALREADY_EXISTS",
            QalaError::SnapshotFailed { .. } => "SNAPSHOT_FAILED",
            QalaError::RollbackFailed { .. } => "ROLLBACK_FAILED",
            QalaError::SdeStateConflict { .. } => "SDE_STATE_CONFLICT",
            QalaError::FactoryCapacityExceeded { .. } => "FACTORY_CAPACITY_EXCEEDED",
            QalaError::UnknownKernelCommand { .. } => "UNKNOWN_KERNEL_COMMAND",
            QalaError::ServiceRegistrationFailed { .. } => "SERVICE_REGISTRATION_FAILED",
            QalaError::PolicyViolation { .. } => "POLICY_VIOLATION",
            QalaError::PlaintextSecretDetected { .. } => "PLAINTEXT_SECRET_DETECTED",
            QalaError::UpstreamError { .. } => "UPSTREAM_ERROR",
            QalaError::Serialisation(_) => "SERIALISATION_ERROR",
            QalaError::Internal(_) => "INTERNAL_ERROR",
        }
    }
}
