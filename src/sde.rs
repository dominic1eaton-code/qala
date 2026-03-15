//! Solution Development Environment (SDE) domain models.
//!
//! An SDE is the atomic operational unit of the qala platform.
//! It is deployable, configurable, distributable, version-controlled,
//! composable, and scalable.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::types::{Maturity, SdeState};

// ── SDE ───────────────────────────────────────────────────────────────────────

/// A Solution Development Environment — the atomic operational unit of qala.
///
/// One SDE can host a collection of solutions. It owns its toolchain, environment
/// configuration, content workspace, and build pipeline configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sde {
    /// Globally unique identifier.
    pub id: Uuid,
    /// Human-readable name (e.g. `"sde-core-api"`).
    pub name: String,
    /// The user who owns this SDE.
    pub owner_id: Uuid,
    /// The factory this SDE belongs to (if any).
    pub factory_id: Option<Uuid>,
    /// Current lifecycle state.
    pub state: SdeState,
    /// Maturity level aligned to the solutions being produced.
    pub maturity: Maturity,
    /// Primary programming language / runtime.
    pub primary_language: String,
    /// Runtime or compiler version (e.g. `"1.22.3"` for Go).
    pub language_version: String,
    /// Human-readable description.
    pub description: String,
    /// Tags for search and classification.
    pub tags: Vec<String>,
    /// Environment variables (non-secret). Secrets live in the vault.
    pub env_vars: HashMap<String, String>,
    /// Configuration parameters controlling SDE behaviour.
    pub config: SdeConfig,
    /// IDs of solutions currently hosted in this SDE.
    pub solution_ids: Vec<Uuid>,
    /// Monotonically incrementing snapshot version counter.
    pub snapshot_version: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Sde {
    /// Create a new SDE in `Provisioning` state.
    pub fn new(
        name: impl Into<String>,
        owner_id: Uuid,
        primary_language: impl Into<String>,
        language_version: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            owner_id,
            factory_id: None,
            state: SdeState::Provisioning,
            maturity: Maturity::Sandbox,
            primary_language: primary_language.into(),
            language_version: language_version.into(),
            description: description.into(),
            tags: Vec::new(),
            env_vars: HashMap::new(),
            config: SdeConfig::default(),
            solution_ids: Vec::new(),
            snapshot_version: 0,
            created_at: now,
            updated_at: now,
        }
    }
}

// ── SDE Configuration ──────────────────────────────────────────────────────────

/// Configuration parameters governing SDE behaviour.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdeConfig {
    /// Build isolation mode.
    pub build_mode: BuildMode,
    /// Whether to automatically create a snapshot after each successful build.
    pub auto_snapshot: bool,
    /// Whether build artifact signing (via cosign) is enforced.
    pub enforce_artifact_signing: bool,
    /// Whether SAST scanning is required on every build.
    pub require_sast: bool,
    /// Minimum test coverage percentage required to pass the build gate.
    /// `None` means no coverage gate is enforced.
    pub min_coverage_pct: Option<f32>,
    /// Maximum number of solutions this SDE may host.
    pub max_solutions: Option<usize>,
}

impl Default for SdeConfig {
    fn default() -> Self {
        Self {
            build_mode: BuildMode::Hermetic,
            auto_snapshot: true,
            enforce_artifact_signing: true,
            require_sast: true,
            min_coverage_pct: None,
            max_solutions: None,
        }
    }
}

/// Build environment isolation mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildMode {
    /// Fully isolated, reproducible, dependency-locked. Recommended.
    Hermetic,
    /// Isolated from the host but not fully reproducible.
    Isolated,
    /// Standard (no isolation guarantee). Only permitted in `Sandbox` maturity.
    Standard,
}

// ── SDE Snapshot ──────────────────────────────────────────────────────────────

/// A point-in-time capture of a complete SDE state.
///
/// Snapshots are the basis for rollback, branching, and audit. They are
/// immutable once created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdeSnapshot {
    pub id: Uuid,
    /// The SDE this snapshot belongs to.
    pub sde_id: Uuid,
    /// Monotonically increasing version number within this SDE.
    pub version: u32,
    /// Human-readable label (e.g. `"Post auth-module refactor"`).
    pub label: String,
    /// Human-readable notes about this snapshot.
    pub notes: Option<String>,
    /// Who triggered this snapshot (user ID or `"ci-system"`).
    pub created_by: String,
    /// The SDE state at the time of the snapshot.
    pub sde_state_at_snapshot: SdeState,
    /// The snapshot's own toolchain manifest (serialised JSON).
    pub toolchain_manifest: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl SdeSnapshot {
    pub fn new(
        sde_id: Uuid,
        version: u32,
        label: impl Into<String>,
        created_by: impl Into<String>,
        sde_state: SdeState,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            sde_id,
            version,
            label: label.into(),
            notes: None,
            created_by: created_by.into(),
            sde_state_at_snapshot: sde_state,
            toolchain_manifest: None,
            created_at: Utc::now(),
        }
    }
}

// ── SDE Tool / Toolchain ───────────────────────────────────────────────────────

/// An individual tool registered in an SDE's toolbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdeTool {
    pub id: Uuid,
    pub sde_id: Uuid,
    /// Tool name (e.g. `"golangci-lint"`, `"cosign"`, `"docker"`).
    pub name: String,
    /// Pinned semantic version.
    pub version: String,
    /// Classification of the tool's function.
    pub tool_type: ToolType,
    /// Vendor or open-source project name.
    pub vendor: String,
    /// SPDX license identifier (e.g. `"MIT"`, `"Apache-2.0"`).
    pub license: String,
    /// SHA-256 hash of the tool binary for supply-chain verification.
    pub verification_hash: Option<String>,
    /// Whether this tool is part of the active toolchain.
    pub in_toolchain: bool,
    pub added_at: DateTime<Utc>,
}

/// Classification of a tool's functional role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolType {
    Compiler,
    Linter,
    Formatter,
    BuildSystem,
    TestRunner,
    Container,
    InfrastructureAsCode,
    Ide,
    VersionControl,
    Scanner,
    Signing,
    SecretsManager,
    Monitoring,
    Other,
}

/// An ordered toolchain — a linked sequence of tools where each tool's output
/// feeds the next.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdeToolchain {
    pub id: Uuid,
    pub sde_id: Uuid,
    /// Name of this chain (e.g. `"build-chain"`, `"release-chain"`).
    pub name: String,
    /// Ordered list of tool IDs in pipeline order.
    pub tool_ids: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
}
