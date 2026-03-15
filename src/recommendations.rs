//! Recommendation types and in-memory store for the AI agents service.
//!
//! A Recommendation is the primary output unit of the AI intelligence plane.
//! Each recommendation carries a typed category, a title, a human-readable body,
//! a confidence score, an optional suggested remediation, and lifecycle metadata.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ── Recommendation category ───────────────────────────────────────────────────

/// The functional category of an AI recommendation.
///
/// Used for filtering, prioritisation, and routing to the appropriate team.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationCategory {
    /// CI/CD pipeline slowness or failure patterns detected.
    PipelineBottleneck,
    /// SDE resource or configuration inefficiency.
    SdeOptimisation,
    /// Vulnerability or dependency risk identified.
    SecurityRisk,
    /// Test coverage gap or quality degradation.
    TestCoverage,
    /// AI-generated test cases ready for import.
    TestGeneration,
    /// Compute or capacity spike predicted.
    ResourcePrediction,
    /// Documentation structure or completeness issue.
    DocumentationQuality,
    /// Backup/snapshot schedule recommendation.
    BackupSchedule,
    /// Prototype or POC analysis feedback.
    PrototypeFeedback,
    /// General platform advisory.
    General,
}

impl std::fmt::Display for RecommendationCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::PipelineBottleneck   => "pipeline_bottleneck",
            Self::SdeOptimisation      => "sde_optimisation",
            Self::SecurityRisk         => "security_risk",
            Self::TestCoverage         => "test_coverage",
            Self::TestGeneration       => "test_generation",
            Self::ResourcePrediction   => "resource_prediction",
            Self::DocumentationQuality => "documentation_quality",
            Self::BackupSchedule       => "backup_schedule",
            Self::PrototypeFeedback    => "prototype_feedback",
            Self::General              => "general",
        };
        write!(f, "{s}")
    }
}

// ── Recommendation status ─────────────────────────────────────────────────────

/// Lifecycle state of an individual recommendation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationStatus {
    /// Active — awaiting human review or automated action.
    Active,
    /// Acknowledged by a user but not yet acted upon.
    Acknowledged,
    /// Applied — the remediation was implemented.
    Applied,
    /// Dismissed — deliberately rejected by a user.
    Dismissed,
    /// Expired — the underlying condition is no longer present.
    Expired,
}

// ── Recommendation ─────────────────────────────────────────────────────────────

/// A single AI-generated insight or advisory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Unique identifier.
    pub id: Uuid,
    /// Category of the recommendation.
    pub category: RecommendationCategory,
    /// Short human-readable title (≤ 80 chars).
    pub title: String,
    /// Full explanation of the finding and its impact.
    pub body: String,
    /// The SDE this recommendation relates to, if applicable.
    pub sde_id: Option<Uuid>,
    /// The solution this recommendation relates to, if applicable.
    pub solution_id: Option<Uuid>,
    /// Confidence score in [0.0, 1.0]. Values ≥ 0.80 are high-confidence.
    pub confidence: f32,
    /// Structured remediation suggestion (optional).
    pub remediation: Option<Remediation>,
    /// Current lifecycle status.
    pub status: RecommendationStatus,
    /// When this recommendation was generated.
    pub generated_at: DateTime<Utc>,
    /// When the status was last changed.
    pub updated_at: DateTime<Utc>,
}

impl Recommendation {
    /// Construct a new active recommendation.
    pub fn new(
        category: RecommendationCategory,
        title: impl Into<String>,
        body: impl Into<String>,
        confidence: f32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id:           Uuid::new_v4(),
            category,
            title:        title.into(),
            body:         body.into(),
            sde_id:       None,
            solution_id:  None,
            confidence:   confidence.clamp(0.0, 1.0),
            remediation:  None,
            status:       RecommendationStatus::Active,
            generated_at: now,
            updated_at:   now,
        }
    }

    pub fn with_sde(mut self, sde_id: Uuid) -> Self {
        self.sde_id = Some(sde_id);
        self
    }

    pub fn with_remediation(mut self, r: Remediation) -> Self {
        self.remediation = Some(r);
        self
    }

    /// Returns `true` if this is a high-confidence recommendation (≥ 0.80).
    pub fn is_high_confidence(&self) -> bool {
        self.confidence >= 0.80
    }
}

// ── Remediation ───────────────────────────────────────────────────────────────

/// A structured remediation suggestion attached to a recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Remediation {
    /// Short label for the action (e.g. `"Apply parallelisation fix"`).
    pub action_label: String,
    /// Human-readable steps to implement the remediation.
    pub steps: Vec<String>,
    /// Optional configuration diff or code snippet (YAML, TOML, etc.).
    pub config_snippet: Option<String>,
    /// Estimated effort to implement (e.g. `"< 1 hour"`, `"2–4 hours"`).
    pub estimated_effort: Option<String>,
    /// Projected improvement after applying (e.g. `"65% reduction in stage 3 duration"`).
    pub projected_impact: Option<String>,
}

// ── Query parameters ──────────────────────────────────────────────────────────

/// Query filters for `GET /recommendations`.
#[derive(Debug, Default, Deserialize)]
pub struct RecommendationQuery {
    /// Filter by SDE ID.
    pub sde_id: Option<Uuid>,
    /// Filter by category (e.g. `"pipeline_bottleneck"`).
    pub category: Option<String>,
    /// Filter by status (default: `active`).
    pub status: Option<String>,
    /// Only return high-confidence recommendations (≥ 0.80).
    pub high_confidence_only: Option<bool>,
    /// Maximum results (default 20, max 200).
    pub limit: Option<usize>,
}

// ── Store ─────────────────────────────────────────────────────────────────────

/// In-memory store of all recommendations.
#[derive(Debug)]
pub struct RecommendationStore {
    records: HashMap<Uuid, Recommendation>,
}

impl RecommendationStore {
    pub fn new() -> Self {
        Self { records: HashMap::new() }
    }

    /// Insert or replace a recommendation.
    pub fn upsert(&mut self, rec: Recommendation) -> Uuid {
        let id = rec.id;
        self.records.insert(id, rec);
        id
    }

    /// Retrieve a recommendation by ID.
    pub fn get(&self, id: &Uuid) -> Option<&Recommendation> {
        self.records.get(id)
    }

    /// Update the status of a recommendation.
    /// Returns `false` if the ID is not found.
    pub fn set_status(&mut self, id: &Uuid, status: RecommendationStatus) -> bool {
        if let Some(rec) = self.records.get_mut(id) {
            rec.status = status;
            rec.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Query recommendations with optional filters.
    pub fn query(&self, q: &RecommendationQuery) -> Vec<&Recommendation> {
        let limit = q.limit.unwrap_or(20).min(200);

        let target_status: RecommendationStatus = q
            .status
            .as_deref()
            .and_then(parse_status)
            .unwrap_or(RecommendationStatus::Active);

        let mut results: Vec<&Recommendation> = self
            .records
            .values()
            .filter(|r| r.status == target_status)
            .filter(|r| {
                if let Some(sde_id) = q.sde_id {
                    r.sde_id == Some(sde_id)
                } else {
                    true
                }
            })
            .filter(|r| {
                if let Some(ref cat) = q.category {
                    r.category.to_string() == *cat
                } else {
                    true
                }
            })
            .filter(|r| {
                if q.high_confidence_only.unwrap_or(false) {
                    r.is_high_confidence()
                } else {
                    true
                }
            })
            .collect();

        // Sort: highest confidence first, then newest
        results.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(b.generated_at.cmp(&a.generated_at))
        });

        results.into_iter().take(limit).collect()
    }

    /// Total number of stored recommendations.
    pub fn count(&self) -> usize {
        self.records.len()
    }

    /// Count of recommendations in `Active` status.
    pub fn active_count(&self) -> usize {
        self.records
            .values()
            .filter(|r| r.status == RecommendationStatus::Active)
            .count()
    }
}

impl Default for RecommendationStore {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_status(s: &str) -> Option<RecommendationStatus> {
    match s {
        "active"       => Some(RecommendationStatus::Active),
        "acknowledged" => Some(RecommendationStatus::Acknowledged),
        "applied"      => Some(RecommendationStatus::Applied),
        "dismissed"    => Some(RecommendationStatus::Dismissed),
        "expired"      => Some(RecommendationStatus::Expired),
        _              => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rec(category: RecommendationCategory, confidence: f32) -> Recommendation {
        Recommendation::new(category, "Test", "Test body", confidence)
    }

    #[test]
    fn high_confidence_threshold() {
        assert!(make_rec(RecommendationCategory::General, 0.80).is_high_confidence());
        assert!(make_rec(RecommendationCategory::General, 0.95).is_high_confidence());
        assert!(!make_rec(RecommendationCategory::General, 0.79).is_high_confidence());
    }

    #[test]
    fn store_query_filters_by_status() {
        let mut store = RecommendationStore::new();
        let active_id = store.upsert(make_rec(RecommendationCategory::General, 0.9));
        store.upsert(make_rec(RecommendationCategory::General, 0.7));
        store.set_status(&active_id, RecommendationStatus::Dismissed);

        let q = RecommendationQuery { status: Some("dismissed".into()), ..Default::default() };
        assert_eq!(store.query(&q).len(), 1);

        let q2 = RecommendationQuery::default();
        assert_eq!(store.query(&q2).len(), 1); // only the remaining active one
    }

    #[test]
    fn confidence_clamped_to_unit_interval() {
        let rec = make_rec(RecommendationCategory::General, 1.5);
        assert_eq!(rec.confidence, 1.0);
        let rec2 = make_rec(RecommendationCategory::General, -0.5);
        assert_eq!(rec2.confidence, 0.0);
    }
}
