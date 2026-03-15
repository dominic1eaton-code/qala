//! Solution Factory domain model.
//!
//! A Solution Factory is a coordinated, networked collection of SDEs that
//! produces solutions in a consistent, repeatable, and scalable way.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{FactoryTier, SubsystemStatus};

// ── Solution Factory ──────────────────────────────────────────────────────────

/// A Solution Factory — a governed, hierarchical collection of networked SDEs.
///
/// Factories are hierarchical: qala itself is the root factory and produces
/// child factories that inherit governance from their parent while allowing
/// scoped customisation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolutionFactory {
    /// Globally unique identifier.
    pub id: Uuid,
    /// Human-readable name (e.g. `"alpha-factory-01"`).
    pub name: String,
    /// The user or organisation that owns this factory.
    pub owner_id: Uuid,
    /// Parent factory ID, if this is a child factory. `None` for the root factory.
    pub parent_factory_id: Option<Uuid>,
    /// Tier determines governance scope and SDE capacity.
    pub tier: FactoryTier,
    /// Current operational status.
    pub status: SubsystemStatus,
    /// Human-readable description of this factory's purpose.
    pub description: String,
    /// IDs of SDEs networked into this factory.
    pub sde_ids: Vec<Uuid>,
    /// IDs of child factories produced by this factory.
    pub child_factory_ids: Vec<Uuid>,
    /// Active governance policy name (e.g. `"enterprise-baseline-v2"`).
    pub governance_policy: String,
    /// Factory-level configuration.
    pub config: FactoryConfig,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SolutionFactory {
    pub fn new(
        name: impl Into<String>,
        owner_id: Uuid,
        tier: FactoryTier,
        description: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            owner_id,
            parent_factory_id: None,
            tier,
            status: SubsystemStatus::Initialising,
            description: description.into(),
            sde_ids: Vec::new(),
            child_factory_ids: Vec::new(),
            governance_policy: "default-v1".to_string(),
            config: FactoryConfig::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Returns `true` if this factory has capacity for another SDE.
    pub fn has_sde_capacity(&self) -> bool {
        match self.tier.max_sdes() {
            None      => true,
            Some(max) => self.sde_ids.len() < max,
        }
    }

    /// Remaining SDE slots, or `None` if unlimited.
    pub fn remaining_sde_capacity(&self) -> Option<usize> {
        self.tier.max_sdes().map(|max| max.saturating_sub(self.sde_ids.len()))
    }
}

// ── Factory Configuration ─────────────────────────────────────────────────────

/// Configuration parameters governing a Solution Factory's behaviour.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoryConfig {
    /// Build isolation enforced across all SDEs in this factory.
    pub build_isolation: FactoryBuildIsolation,
    /// Whether build attestation (SLSA provenance) is required for all artifacts.
    pub require_build_attestation: bool,
    /// Whether artifact signing (cosign) is required before publication.
    pub require_artifact_signing: bool,
    /// Event streaming broker address (e.g. `"kafka:9092"`).
    pub event_broker: String,
    /// Secrets vault address (e.g. `"http://vault:8200"`).
    pub secrets_vault_addr: String,
    /// Networking mode between SDEs in this factory.
    pub network_mode: SdeNetworkMode,
}

impl Default for FactoryConfig {
    fn default() -> Self {
        Self {
            build_isolation: FactoryBuildIsolation::Hermetic,
            require_build_attestation: true,
            require_artifact_signing: true,
            event_broker: "kafka:9092".to_string(),
            secrets_vault_addr: "http://vault:8200".to_string(),
            network_mode: SdeNetworkMode::Mesh,
        }
    }
}

/// Build isolation policy applied factory-wide.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactoryBuildIsolation {
    /// All builds in all SDEs must be hermetic. Recommended.
    Hermetic,
    /// Standard isolation enforced but full hermeticity not guaranteed.
    Isolated,
    /// No factory-level isolation policy — deferred to individual SDE config.
    PerSde,
}

/// Networking topology for SDEs within a factory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SdeNetworkMode {
    /// Full mesh — all SDEs can communicate with all others.
    Mesh,
    /// Hub-and-spoke — SDEs communicate only through the factory coordinator.
    HubAndSpoke,
    /// Isolated — no direct SDE-to-SDE communication; all events via broker.
    Isolated,
}

// ── Governance Policy ─────────────────────────────────────────────────────────

/// A governance policy applied to a factory or SDE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernancePolicy {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub description: String,
    /// The set of rules this policy enforces.
    pub rules: Vec<PolicyRule>,
    /// Factory IDs this policy is applied to.
    pub applied_to_factories: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A single rule within a governance policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub id: Uuid,
    /// Machine-readable rule identifier (e.g. `"no-plain-secrets-in-env"`).
    pub rule_key: String,
    /// Human-readable description.
    pub description: String,
    /// Scope of this rule.
    pub scope: PolicyScope,
    /// Action to take when this rule is violated.
    pub enforcement: PolicyEnforcement,
}

/// The scope a policy rule applies to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyScope {
    AllSdes,
    ProductionSdes,
    BuildPipelines,
    Artifacts,
    Users,
}

/// How a policy violation is handled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyEnforcement {
    /// Block the operation and return an error.
    Block,
    /// Allow the operation but emit a SECURITY_EVENTS alert.
    Alert,
    /// Record the violation in the audit log silently.
    Audit,
}
