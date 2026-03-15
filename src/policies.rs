//! Policy engine — governance policy types and in-memory store.
//!
//! Policies define the rules that must be satisfied across all SDEs and
//! artifacts in a solution factory. They are enforced at build time, during
//! SDE scans, and at artifact publication.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ── Policy enums ───────────────────────────────────────────────────────────────

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

impl std::fmt::Display for PolicyScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PolicyScope::AllSdes         => "all_sdes",
            PolicyScope::ProductionSdes  => "production_sdes",
            PolicyScope::BuildPipelines  => "build_pipelines",
            PolicyScope::Artifacts       => "artifacts",
            PolicyScope::Users           => "users",
        };
        write!(f, "{s}")
    }
}

/// How a policy violation is handled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyEnforcement {
    /// Block the operation and return an error immediately.
    Block,
    /// Allow the operation but emit a SECURITY_EVENTS alert.
    Alert,
    /// Record the violation silently in the audit log.
    Audit,
}

/// Status of a policy record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyStatus {
    Active,
    Disabled,
    Draft,
    Deprecated,
}

// ── Policy rule ────────────────────────────────────────────────────────────────

/// A single rule within a governance policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub id:          Uuid,
    /// Machine-readable rule key (e.g. `"hermetic_build_required"`).
    pub rule_key:    String,
    /// Human-readable description.
    pub description: String,
    /// Scope of this rule.
    pub scope:       PolicyScope,
    /// How violations are handled.
    pub enforcement: PolicyEnforcement,
}

// ── Policy ─────────────────────────────────────────────────────────────────────

/// A governance policy applied to a factory or its SDEs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id:          Uuid,
    pub name:        String,
    pub version:     String,
    pub description: String,
    pub status:      PolicyStatus,
    pub rules:       Vec<PolicyRule>,
    pub created_at:  DateTime<Utc>,
    pub updated_at:  DateTime<Utc>,
}

impl Policy {
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id:          Uuid::new_v4(),
            name:        name.into(),
            version:     version.into(),
            description: description.into(),
            status:      PolicyStatus::Active,
            rules:       Vec::new(),
            created_at:  now,
            updated_at:  now,
        }
    }

    /// Convenience constructor that creates a policy with a single rule.
    pub fn with_rule(
        name: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
        rule_key: impl Into<String>,
        scope: PolicyScope,
        enforcement: PolicyEnforcement,
        rule_description: impl Into<String>,
    ) -> Self {
        let mut p = Self::new(name, version, description);
        p.rules.push(PolicyRule {
            id:          Uuid::new_v4(),
            rule_key:    rule_key.into(),
            description: rule_description.into(),
            scope,
            enforcement,
        });
        p
    }
}

// ── Request / Response ─────────────────────────────────────────────────────────

/// Request body for `POST /policy/update`.
#[derive(Debug, Deserialize)]
pub struct PolicyUpdateRequest {
    pub name:        String,
    pub version:     Option<String>,
    pub description: Option<String>,
    pub status:      Option<String>,
    pub rules:       Option<Vec<PolicyRuleSpec>>,
}

/// Specification for a single policy rule in an update request.
#[derive(Debug, Deserialize)]
pub struct PolicyRuleSpec {
    pub rule_key:    String,
    pub description: String,
    pub scope:       String,
    pub enforcement: String,
}

// ── Policy store ───────────────────────────────────────────────────────────────

/// In-memory governance policy store.
#[derive(Debug)]
pub struct PolicyStore {
    /// Policies keyed by name for fast lookup by name.
    by_name: HashMap<String, Policy>,
    /// Policies keyed by ID.
    by_id:   HashMap<Uuid, String>, // id → name
}

impl PolicyStore {
    pub fn new() -> Self {
        Self {
            by_name: HashMap::new(),
            by_id:   HashMap::new(),
        }
    }

    /// Insert or replace a policy by name.
    pub fn upsert(&mut self, mut policy: Policy) -> Uuid {
        policy.updated_at = Utc::now();
        let id   = policy.id;
        let name = policy.name.clone();
        self.by_id.insert(id, name.clone());
        self.by_name.insert(name, policy);
        id
    }

    pub fn get_by_name(&self, name: &str) -> Option<&Policy> {
        self.by_name.get(name)
    }

    pub fn get_by_id(&self, id: &Uuid) -> Option<&Policy> {
        self.by_id.get(id).and_then(|name| self.by_name.get(name))
    }

    pub fn all_active(&self) -> Vec<&Policy> {
        self.by_name
            .values()
            .filter(|p| p.status == PolicyStatus::Active)
            .collect()
    }

    pub fn all(&self) -> Vec<&Policy> {
        self.by_name.values().collect()
    }

    pub fn count(&self) -> usize {
        self.by_name.len()
    }

    /// Apply a `PolicyUpdateRequest` to create or update a policy.
    pub fn apply_update(&mut self, req: &PolicyUpdateRequest) -> Uuid {
        let now = Utc::now();

        let existing = self.by_name.get(&req.name).cloned();

        let mut policy = existing.unwrap_or_else(|| Policy {
            id:          Uuid::new_v4(),
            name:        req.name.clone(),
            version:     "1.0.0".to_string(),
            description: String::new(),
            status:      PolicyStatus::Active,
            rules:       Vec::new(),
            created_at:  now,
            updated_at:  now,
        });

        if let Some(v) = &req.version     { policy.version     = v.clone(); }
        if let Some(d) = &req.description { policy.description = d.clone(); }
        if let Some(s) = &req.status {
            policy.status = match s.as_str() {
                "active"     => PolicyStatus::Active,
                "disabled"   => PolicyStatus::Disabled,
                "draft"      => PolicyStatus::Draft,
                "deprecated" => PolicyStatus::Deprecated,
                _            => PolicyStatus::Active,
            };
        }

        if let Some(rules) = &req.rules {
            policy.rules = rules
                .iter()
                .map(|r| PolicyRule {
                    id:          Uuid::new_v4(),
                    rule_key:    r.rule_key.clone(),
                    description: r.description.clone(),
                    scope:       parse_scope(&r.scope),
                    enforcement: parse_enforcement(&r.enforcement),
                })
                .collect();
        }

        policy.updated_at = now;
        self.upsert(policy)
    }
}

impl Default for PolicyStore {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_scope(s: &str) -> PolicyScope {
    match s {
        "production_sdes"  => PolicyScope::ProductionSdes,
        "build_pipelines"  => PolicyScope::BuildPipelines,
        "artifacts"        => PolicyScope::Artifacts,
        "users"            => PolicyScope::Users,
        _                  => PolicyScope::AllSdes,
    }
}

fn parse_enforcement(s: &str) -> PolicyEnforcement {
    match s {
        "alert" => PolicyEnforcement::Alert,
        "audit" => PolicyEnforcement::Audit,
        _       => PolicyEnforcement::Block,
    }
}
