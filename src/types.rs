//! Canonical primitive enums and type aliases used across the qala platform.
//!
//! Every service in the qala SFOS works with the same typed vocabulary defined
//! here so that cross-service communication is unambiguous.

use serde::{Deserialize, Serialize};

// ── Solution types ─────────────────────────────────────────────────────────────

/// The six canonical solution types recognised by the qala platform.
///
/// Every artifact managed in qala is classified as exactly one of these types.
/// The type drives governance rules, maturity gate criteria, and UI presentation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SolutionType {
    /// A software application with defined processes and user interactions.
    Application,
    /// A composition of applications with coordinated behaviour and shared infrastructure.
    System,
    /// A tangible or digital deliverable produced as the output of a process.
    Good,
    /// A market-facing offering combining software, services, and/or goods.
    Product,
    /// A capability or function delivered to a consumer on-demand or continuously.
    Service,
    /// A substrate on which other solutions are built, deployed, and operated.
    Platform,
}

impl std::fmt::Display for SolutionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SolutionType::Application => "application",
            SolutionType::System      => "system",
            SolutionType::Good        => "good",
            SolutionType::Product     => "product",
            SolutionType::Service     => "service",
            SolutionType::Platform    => "platform",
        };
        write!(f, "{s}")
    }
}

// ── Maturity lifecycle ─────────────────────────────────────────────────────────

/// Solution maturity lifecycle stages.
///
/// Promotion between stages requires passing the configured gate criteria for
/// that stage. Only `Cm`-maturity solutions may produce immutable release artifacts.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Maturity {
    /// Open exploration. No stability guarantees; no gate criteria.
    Sandbox,
    /// Active feature development. Nightly builds may break.
    Dev,
    /// Automated nightly build and test cycle.
    Nightly,
    /// Feature-complete. Full test suites running. Pre-release hardening.
    Test,
    /// Release-qualified. Under Configuration Management. Immutable artifacts only.
    Cm,
}

impl Maturity {
    /// Returns the next maturity stage, or `None` if already at `Cm`.
    pub fn next(&self) -> Option<Maturity> {
        match self {
            Maturity::Sandbox => Some(Maturity::Dev),
            Maturity::Dev     => Some(Maturity::Nightly),
            Maturity::Nightly => Some(Maturity::Test),
            Maturity::Test    => Some(Maturity::Cm),
            Maturity::Cm      => None,
        }
    }

    /// Returns the gate description for advancing to the next stage.
    pub fn gate_description(&self) -> &'static str {
        match self {
            Maturity::Sandbox => "No gate — open creation",
            Maturity::Dev     => "Basic CI build success",
            Maturity::Nightly => "CI green on main branch",
            Maturity::Test    => "All test suites passing",
            Maturity::Cm      => "CM board approval and sign-off",
        }
    }
}

impl std::fmt::Display for Maturity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Maturity::Sandbox => "SANDBOX",
            Maturity::Dev     => "DEV",
            Maturity::Nightly => "NIGHTLY",
            Maturity::Test    => "TEST",
            Maturity::Cm      => "CM",
        };
        write!(f, "{s}")
    }
}

// ── Data type system ───────────────────────────────────────────────────────────

/// The canonical set of data types used across all qala solution data structures.
///
/// Custom types are composed from these primitives. Every `DataField` inside a
/// `DataStructure` declares one of these variants as its type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataType {
    // Primitives
    Bool,
    String,
    Char,
    Varchar,
    Date,
    Null,
    // Numeric
    Int,
    Float,
    Double,
    // Collections
    Array,
    Tuple,
    Set,
    Map,
    // References
    Object,
    Pointer,
    /// User-defined composite type; `name` holds the fully-qualified type identifier.
    Custom { name: String },
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Custom { name } => write!(f, "custom:{name}"),
            other => write!(f, "{other:?}").map(|_| ()).and_then(|_| {
                // lowercase the debug repr
                write!(f, "{}", format!("{other:?}").to_lowercase())
            }),
        }
    }
}

// ── SDE lifecycle states ───────────────────────────────────────────────────────

/// Lifecycle state of a Solution Development Environment (SDE).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SdeState {
    /// Environment is being assembled from template or definition.
    Provisioning,
    /// Fully operational; development and build activities may occur.
    Active,
    /// State captured; environment continues to run.
    Snapshotted,
    /// Resources released; state preserved for resumption.
    Suspended,
    /// Environment restored to a prior snapshot version.
    RolledBack,
    /// Moved to long-term storage; retrievable on demand.
    Archived,
    /// Permanently decommissioned; backups retained per policy.
    Terminated,
}

impl SdeState {
    /// Returns `true` if the SDE can receive build traffic in this state.
    pub fn is_buildable(&self) -> bool {
        matches!(self, SdeState::Active | SdeState::Snapshotted)
    }

    /// Returns `true` if the SDE can be snapshotted from this state.
    pub fn can_snapshot(&self) -> bool {
        matches!(self, SdeState::Active | SdeState::Snapshotted)
    }

    /// Returns `true` if rollback is a valid operation from this state.
    pub fn can_rollback(&self) -> bool {
        matches!(
            self,
            SdeState::Active | SdeState::Snapshotted | SdeState::Suspended
        )
    }
}

impl std::fmt::Display for SdeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SdeState::Provisioning => "provisioning",
            SdeState::Active       => "active",
            SdeState::Snapshotted  => "snapshotted",
            SdeState::Suspended    => "suspended",
            SdeState::RolledBack   => "rolled_back",
            SdeState::Archived     => "archived",
            SdeState::Terminated   => "terminated",
        };
        write!(f, "{s}")
    }
}

// ── Factory tier ───────────────────────────────────────────────────────────────

/// Tier of a Solution Factory, determining governance scope and SDE capacity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactoryTier {
    /// The qala platform root factory — immutable policies.
    Root,
    /// Multi-team, multi-product factory with full governance suite.
    Enterprise,
    /// Single-team factory scoped to a product or service line.
    Team,
    /// Single-user factory for personal and hobby projects.
    Personal,
}

impl FactoryTier {
    /// Maximum number of SDEs permitted at this tier.
    /// `None` means unlimited (Enterprise and Root).
    pub fn max_sdes(&self) -> Option<usize> {
        match self {
            FactoryTier::Root       => None,
            FactoryTier::Enterprise => None,
            FactoryTier::Team       => Some(50),
            FactoryTier::Personal   => Some(2),
        }
    }
}

// ── Message types ──────────────────────────────────────────────────────────────

/// Whether a message is dynamic (event-driven) or static (state snapshot).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageKind {
    /// Dynamic — published when something changes (e.g. `ProductCreated`).
    Event,
    /// Static — represents the current state of a resource (e.g. `ProductState`).
    State,
}

/// Whether an interface message flows inbound or outbound.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowDirection {
    Import,
    Export,
}

// ── Severity / risk levels ─────────────────────────────────────────────────────

/// Risk/severity level used by the Security SEM and AI agents.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Severity::Low      => "low",
            Severity::Medium   => "medium",
            Severity::High     => "high",
            Severity::Critical => "critical",
        };
        write!(f, "{s}")
    }
}

// ── Subsystem status ───────────────────────────────────────────────────────────

/// Operational status reported by subsystems to the kernel.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubsystemStatus {
    Initialising,
    Healthy,
    Degraded,
    Offline,
    Unknown,
}

impl std::fmt::Display for SubsystemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SubsystemStatus::Initialising => "initialising",
            SubsystemStatus::Healthy      => "healthy",
            SubsystemStatus::Degraded     => "degraded",
            SubsystemStatus::Offline      => "offline",
            SubsystemStatus::Unknown      => "unknown",
        };
        write!(f, "{s}")
    }
}
