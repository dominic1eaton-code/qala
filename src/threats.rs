//! Threat management — types, store, and query support for security findings.
//!
//! The `ThreatStore` is the authoritative record of all active and historical
//! security threats detected across the qala platform. Threats are sourced from:
//! - SDE scan results (CVE detection, SAST findings, config violations)
//! - Real-time security event stream correlation
//! - External feed ingestion (future: NVD, GitHub Advisory)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use qala_shared::types::Severity;

// ── Threat types ───────────────────────────────────────────────────────────────

/// Classification of the security threat.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreatType {
    /// Known software vulnerability (CVE).
    Cve,
    /// Exposed credential or secret.
    CredentialExposure,
    /// Static analysis security finding.
    SastFinding,
    /// Dependency supply-chain risk.
    SupplyChain,
    /// Misconfiguration detected.
    Misconfiguration,
    /// Policy violation.
    PolicyViolation,
    /// Runtime anomaly detected by AI correlation.
    RuntimeAnomaly,
}

/// Lifecycle status of a threat record.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreatStatus {
    /// Newly detected, not yet reviewed.
    Active,
    /// Under investigation.
    Investigating,
    /// A fix is in progress.
    Remediating,
    /// The threat has been fully resolved.
    Resolved,
    /// Accepted risk — documented and acknowledged.
    AcceptedRisk,
    /// False positive confirmed.
    FalsePositive,
}

// ── Threat ─────────────────────────────────────────────────────────────────────

/// A single security threat or finding in the qala platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Threat {
    pub id:          Uuid,
    /// Human-readable threat identifier (e.g. `"THR-0041"`).
    pub threat_ref:  String,
    /// The type of threat.
    pub threat_type: ThreatType,
    /// Severity level.
    pub severity:    Severity,
    /// Short human-readable title.
    pub title:       String,
    /// Full description including impact and context.
    pub description: String,
    /// CVE identifier, if applicable (e.g. `"CVE-2024-44000"`).
    pub cve_id:      Option<String>,
    /// CVSS score [0.0, 10.0], if applicable.
    pub cvss_score:  Option<f32>,
    /// The SDE the threat was detected in, if applicable.
    pub sde_id:      Option<Uuid>,
    /// Affected package/dependency, if applicable.
    pub affected_component: Option<String>,
    /// Current status.
    pub status:      ThreatStatus,
    /// When the threat was first detected.
    pub detected_at: DateTime<Utc>,
    /// When the threat status was last updated.
    pub updated_at:  DateTime<Utc>,
    /// Who or what detected this threat.
    pub detected_by: String,
}

impl Threat {
    pub fn new(
        threat_type: ThreatType,
        severity: Severity,
        title: impl Into<String>,
        description: impl Into<String>,
        detected_by: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id:                 Uuid::new_v4(),
            threat_ref:         String::new(), // set by store on insert
            threat_type,
            severity,
            title:              title.into(),
            description:        description.into(),
            cve_id:             None,
            cvss_score:         None,
            sde_id:             None,
            affected_component: None,
            status:             ThreatStatus::Active,
            detected_at:        now,
            updated_at:         now,
            detected_by:        detected_by.into(),
        }
    }

    pub fn with_cve(mut self, cve_id: impl Into<String>, cvss: f32) -> Self {
        self.cve_id     = Some(cve_id.into());
        self.cvss_score = Some(cvss.clamp(0.0, 10.0));
        self
    }

    pub fn with_sde(mut self, sde_id: Uuid) -> Self {
        self.sde_id = Some(sde_id);
        self
    }

    pub fn with_component(mut self, component: impl Into<String>) -> Self {
        self.affected_component = Some(component.into());
        self
    }
}

// ── Query parameters ───────────────────────────────────────────────────────────

/// Query filters for `GET /threats`.
#[derive(Debug, Default, Deserialize)]
pub struct ThreatQuery {
    pub sde_id:   Option<Uuid>,
    pub severity: Option<String>,
    pub status:   Option<String>,
    pub limit:    Option<usize>,
}

// ── Threat store ───────────────────────────────────────────────────────────────

/// In-memory threat store with auto-incrementing threat reference numbers.
#[derive(Debug)]
pub struct ThreatStore {
    records:     HashMap<Uuid, Threat>,
    next_ref_id: u32,
}

impl ThreatStore {
    pub fn new() -> Self {
        Self {
            records:     HashMap::new(),
            next_ref_id: 1,
        }
    }

    /// Insert a new threat record, assigning a `threat_ref` automatically.
    pub fn insert(&mut self, mut threat: Threat) -> Uuid {
        let id = threat.id;
        threat.threat_ref = format!("THR-{:04}", self.next_ref_id);
        self.next_ref_id += 1;
        self.records.insert(id, threat);
        id
    }

    /// Update the status of a threat.
    pub fn set_status(&mut self, id: &Uuid, status: ThreatStatus) -> bool {
        if let Some(t) = self.records.get_mut(id) {
            t.status     = status;
            t.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Retrieve a threat by ID.
    pub fn get(&self, id: &Uuid) -> Option<&Threat> {
        self.records.get(id)
    }

    /// Query threats with optional filters.
    pub fn query(&self, q: &ThreatQuery) -> Vec<&Threat> {
        let limit = q.limit.unwrap_or(50).min(500);

        let target_status = q.status.as_deref()
            .and_then(parse_threat_status)
            .unwrap_or(ThreatStatus::Active);

        let target_severity = q.severity.as_deref().and_then(parse_severity);

        let mut results: Vec<&Threat> = self
            .records
            .values()
            .filter(|t| t.status == target_status)
            .filter(|t| {
                if let Some(sde_id) = q.sde_id {
                    t.sde_id == Some(sde_id)
                } else {
                    true
                }
            })
            .filter(|t| {
                if let Some(ref sev) = target_severity {
                    &t.severity == sev
                } else {
                    true
                }
            })
            .collect();

        // Sort: highest severity first, then newest
        results.sort_by(|a, b| {
            b.severity
                .cmp(&a.severity)
                .then(b.detected_at.cmp(&a.detected_at))
        });

        results.into_iter().take(limit).collect()
    }

    pub fn count(&self) -> usize {
        self.records.len()
    }

    pub fn active_count(&self) -> usize {
        self.records
            .values()
            .filter(|t| t.status == ThreatStatus::Active)
            .count()
    }
}

impl Default for ThreatStore {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_threat_status(s: &str) -> Option<ThreatStatus> {
    match s {
        "active"        => Some(ThreatStatus::Active),
        "investigating" => Some(ThreatStatus::Investigating),
        "remediating"   => Some(ThreatStatus::Remediating),
        "resolved"      => Some(ThreatStatus::Resolved),
        "accepted_risk" => Some(ThreatStatus::AcceptedRisk),
        "false_positive"=> Some(ThreatStatus::FalsePositive),
        _               => None,
    }
}

fn parse_severity(s: &str) -> Option<Severity> {
    match s {
        "low"      => Some(Severity::Low),
        "medium"   => Some(Severity::Medium),
        "high"     => Some(Severity::High),
        "critical" => Some(Severity::Critical),
        _          => None,
    }
}
