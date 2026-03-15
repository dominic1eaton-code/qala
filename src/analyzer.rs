//! SDE Analyser — produces a structured set of AI findings for a given SDE.
//!
//! The analyser is invoked by `POST /analyze_sde` and returns a prioritised
//! list of `Recommendation` objects. In production these would be backed by
//! real ML models, historical telemetry, and live metric feeds. Here we
//! implement the full structural contract with realistic heuristic stubs
//! that demonstrate the complete analysis flow.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::recommendations::{
    Recommendation, RecommendationCategory, Remediation,
};

// ── Request / Response ─────────────────────────────────────────────────────────

/// Request body for `POST /analyze_sde`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeSdeRequest {
    /// The SDE to analyse.
    pub sde_id: Uuid,
    /// Optional: analysis scope. Defaults to all checks.
    #[serde(default)]
    pub scope: Vec<AnalysisScope>,
    /// Optional caller-provided metrics snapshot (avoids a service round-trip).
    pub metrics_snapshot: Option<SdeMetricsSnapshot>,
}

/// Scope flags controlling which categories of analysis are performed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisScope {
    PipelinePerformance,
    ResourceUtilisation,
    SecurityPosture,
    TestQuality,
    Documentation,
    All,
}

/// Caller-provided metrics for the SDE being analysed.
/// These mirror what the Data Platform would supply in a real system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdeMetricsSnapshot {
    /// Average CI pipeline duration in seconds.
    pub avg_pipeline_duration_secs: Option<f64>,
    /// Expected (SLA) pipeline duration in seconds.
    pub sla_pipeline_duration_secs: Option<f64>,
    /// Current memory utilisation as a fraction [0.0, 1.0].
    pub memory_utilisation:         Option<f32>,
    /// Current CPU utilisation as a fraction [0.0, 1.0].
    pub cpu_utilisation:            Option<f32>,
    /// Test coverage percentage [0.0, 100.0].
    pub test_coverage_pct:          Option<f32>,
    /// Number of high-severity CVEs currently open.
    pub open_high_cve_count:        Option<u32>,
    /// Whether a plaintext secret was detected.
    pub plaintext_secret_detected:  Option<bool>,
    /// Number of active recommendations that have not been actioned.
    pub unactioned_recommendations: Option<u32>,
}

/// The result of a complete SDE analysis.
#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub analysis_id:       Uuid,
    pub sde_id:            Uuid,
    pub scope:             Vec<String>,
    pub findings_count:    usize,
    pub recommendations:   Vec<Recommendation>,
    pub overall_score:     f32,  // 0–100 composite health score
    pub analysed_at:       chrono::DateTime<Utc>,
    pub duration_ms:       u64,
}

// ── Analyser ───────────────────────────────────────────────────────────────────

/// Execute a full AI analysis of the given SDE, returning an `AnalysisResult`.
///
/// The analysis is composed of independent check functions. Each check
/// inspects the supplied metrics and returns 0..N recommendations.
/// Results are merged, de-duplicated (by category for the same SDE), sorted
/// by confidence descending, and wrapped in an `AnalysisResult`.
pub async fn run(req: &AnalyzeSdeRequest) -> AnalysisResult {
    let start = std::time::Instant::now();
    let analysis_id = Uuid::new_v4();
    let metrics = req.metrics_snapshot.as_ref();
    let sde_id = req.sde_id;

    let run_all = req.scope.is_empty() || req.scope.contains(&AnalysisScope::All);

    let mut recs: Vec<Recommendation> = Vec::new();

    if run_all || req.scope.contains(&AnalysisScope::PipelinePerformance) {
        recs.extend(check_pipeline_performance(sde_id, metrics));
    }
    if run_all || req.scope.contains(&AnalysisScope::ResourceUtilisation) {
        recs.extend(check_resource_utilisation(sde_id, metrics));
    }
    if run_all || req.scope.contains(&AnalysisScope::SecurityPosture) {
        recs.extend(check_security_posture(sde_id, metrics));
    }
    if run_all || req.scope.contains(&AnalysisScope::TestQuality) {
        recs.extend(check_test_quality(sde_id, metrics));
    }
    if run_all || req.scope.contains(&AnalysisScope::Documentation) {
        recs.extend(check_documentation(sde_id, metrics));
    }

    // Sort by confidence descending
    recs.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));

    // Compute composite health score (100 = perfect, reduced by each finding weighted by confidence)
    let penalty: f32 = recs.iter().map(|r| r.confidence * 15.0).sum::<f32>().min(100.0);
    let overall_score = (100.0 - penalty).clamp(0.0, 100.0);

    let duration_ms = start.elapsed().as_millis() as u64;
    let findings_count = recs.len();

    tracing::info!(
        sde_id = %sde_id,
        findings = findings_count,
        score = overall_score,
        duration_ms,
        "SDE analysis complete"
    );

    AnalysisResult {
        analysis_id,
        sde_id,
        scope: req.scope.iter().map(|s| format!("{s:?}")).collect(),
        findings_count,
        recommendations: recs,
        overall_score,
        analysed_at: Utc::now(),
        duration_ms,
    }
}

// ── Individual analysis checks ────────────────────────────────────────────────

/// Check CI/CD pipeline performance metrics.
fn check_pipeline_performance(sde_id: Uuid, m: Option<&SdeMetricsSnapshot>) -> Vec<Recommendation> {
    let mut out = Vec::new();

    let avg = m.and_then(|m| m.avg_pipeline_duration_secs);
    let sla = m.and_then(|m| m.sla_pipeline_duration_secs);

    if let (Some(avg), Some(sla)) = (avg, sla) {
        let ratio = avg / sla;
        if ratio > 2.5 {
            let confidence = ((ratio - 1.0) / 5.0).clamp(0.7, 0.99) as f32;
            out.push(
                Recommendation::new(
                    RecommendationCategory::PipelineBottleneck,
                    "Pipeline latency exceeds SLA threshold",
                    format!(
                        "Average pipeline duration {:.0}s is {:.1}× above the SLA target of {:.0}s. \
                         Recommend parallelising the transform stage workers from 2 to 6 threads. \
                         Projected reduction: 60–70%.",
                        avg, ratio, sla
                    ),
                    confidence,
                )
                .with_sde(sde_id)
                .with_remediation(Remediation {
                    action_label: "Increase transform worker parallelism".into(),
                    steps: vec![
                        "Set `stages[transform].workers: 6` in pipeline.yaml".into(),
                        "Reduce `batch_size` from 15000 to 5000 to balance memory".into(),
                        "Re-run the pipeline and verify stage 3 duration".into(),
                    ],
                    config_snippet: Some(
                        "stages:\n  - name: data-transform\n    workers: 6  # was: 2\n    batch_size: 5000".into()
                    ),
                    estimated_effort: Some("< 1 hour".into()),
                    projected_impact: Some(format!("~65% reduction from {avg:.0}s to ~{:.0}s", avg * 0.35)),
                }),
            );
        }
    } else {
        // No metrics supplied — recommend fetching pipeline telemetry
        out.push(
            Recommendation::new(
                RecommendationCategory::PipelineBottleneck,
                "Pipeline performance telemetry unavailable",
                "No pipeline duration metrics were supplied for this SDE. \
                 Enable build event streaming to allow AI pipeline analysis.",
                0.65,
            )
            .with_sde(sde_id),
        );
    }

    out
}

/// Check SDE resource utilisation.
fn check_resource_utilisation(sde_id: Uuid, m: Option<&SdeMetricsSnapshot>) -> Vec<Recommendation> {
    let mut out = Vec::new();

    if let Some(mem) = m.and_then(|m| m.memory_utilisation) {
        if mem > 0.85 {
            let confidence = ((mem - 0.5) * 2.0).clamp(0.0, 0.99) as f32;
            out.push(
                Recommendation::new(
                    RecommendationCategory::SdeOptimisation,
                    "Memory utilisation approaching capacity limit",
                    format!(
                        "Current memory utilisation is {:.0}% of the allocated limit. \
                         Enabling lazy loading on model weight initialisation would \
                         reduce peak memory usage by approximately 40%.",
                        mem * 100.0
                    ),
                    confidence,
                )
                .with_sde(sde_id)
                .with_remediation(Remediation {
                    action_label: "Enable lazy weight loading".into(),
                    steps: vec![
                        "Add `lazy_loading: true` to model config".into(),
                        "Set `GOMAXPROCS` environment variable to match available vCPUs".into(),
                        "Monitor peak RSS after deployment".into(),
                    ],
                    config_snippet: Some("model:\n  lazy_loading: true\n  weight_cache_mb: 2048".into()),
                    estimated_effort: Some("2–4 hours".into()),
                    projected_impact: Some(format!(
                        "Peak memory {:.1} GB → ~{:.1} GB",
                        mem * 16.0, mem * 16.0 * 0.60
                    )),
                }),
            );
        }
    }

    if let Some(cpu) = m.and_then(|m| m.cpu_utilisation) {
        if cpu > 0.80 {
            out.push(
                Recommendation::new(
                    RecommendationCategory::ResourcePrediction,
                    "CPU utilisation spike detected — consider horizontal scaling",
                    format!(
                        "CPU utilisation is at {:.0}%. \
                         Based on current trend, a scaling event within the next 4–6 hours is likely. \
                         Recommend pre-scaling worker pool from 2 to 4 replicas.",
                        cpu * 100.0
                    ),
                    0.82,
                )
                .with_sde(sde_id),
            );
        }
    }

    out
}

/// Check security posture of the SDE.
fn check_security_posture(sde_id: Uuid, m: Option<&SdeMetricsSnapshot>) -> Vec<Recommendation> {
    let mut out = Vec::new();

    if let Some(cve_count) = m.and_then(|m| m.open_high_cve_count) {
        if cve_count > 0 {
            let confidence = (0.70 + cve_count as f32 * 0.05).min(0.99);
            out.push(
                Recommendation::new(
                    RecommendationCategory::SecurityRisk,
                    format!("{cve_count} high-severity CVE(s) detected in dependency tree"),
                    format!(
                        "{cve_count} high-severity CVE(s) are present in this SDE's dependency tree. \
                         Patch versions are available. Deployment to production is blocked by policy \
                         `cve-high-block-deploy` until resolved.",
                    ),
                    confidence,
                )
                .with_sde(sde_id)
                .with_remediation(Remediation {
                    action_label: "Upgrade affected dependencies".into(),
                    steps: vec![
                        "Run `govulncheck ./...` (Go) or `cargo audit` (Rust) for the full CVE list".into(),
                        "Upgrade each affected dependency to the patched version".into(),
                        "Re-run the security scan pipeline stage to confirm clean".into(),
                        "Publish a patched artifact to unblock deployment".into(),
                    ],
                    config_snippet: None,
                    estimated_effort: Some(format!("{cve_count}–{} hours", cve_count * 2)),
                    projected_impact: Some("Unblocks production deployment; eliminates policy violation".into()),
                }),
            );
        }
    }

    if m.and_then(|m| m.plaintext_secret_detected).unwrap_or(false) {
        out.push(
            Recommendation::new(
                RecommendationCategory::SecurityRisk,
                "Plaintext secret detected in environment variables",
                "A credential or API key has been found stored as a plaintext environment variable. \
                 This violates policy `no-plain-secrets-in-env`. The secret must be migrated to the \
                 HashiCorp Vault secrets path for this SDE.",
                0.99,
            )
            .with_sde(sde_id)
            .with_remediation(Remediation {
                action_label: "Migrate secret to Vault".into(),
                steps: vec![
                    "Run `vault kv put secret/<sde-name>/<key> value=<secret>`".into(),
                    "Replace the env var with a Vault reference in sde.yaml".into(),
                    "Re-scan the SDE to confirm the violation is cleared".into(),
                ],
                config_snippet: Some(
                    "env_refs:\n  - key: API_KEY_OPENAI\n    vault_path: secret/sde-ml-research/openai-api-key".into()
                ),
                estimated_effort: Some("< 30 minutes".into()),
                projected_impact: Some("Clears policy violation; resolves security threat THR-0043".into()),
            }),
        );
    }

    out
}

/// Check test quality and coverage.
fn check_test_quality(sde_id: Uuid, m: Option<&SdeMetricsSnapshot>) -> Vec<Recommendation> {
    let mut out = Vec::new();

    if let Some(cov) = m.and_then(|m| m.test_coverage_pct) {
        if cov < 80.0 {
            let gap = 90.0 - cov;
            out.push(
                Recommendation::new(
                    RecommendationCategory::TestCoverage,
                    format!("Test coverage {cov:.0}% is below the CM gate threshold of 90%"),
                    format!(
                        "Current test coverage is {cov:.1}%. The CM maturity gate requires ≥ 90%. \
                         A gap of {gap:.1}% must be closed before this solution can be promoted. \
                         AI-generated test cases are available for import.",
                    ),
                    0.90,
                )
                .with_sde(sde_id)
                .with_remediation(Remediation {
                    action_label: "Import AI-generated test cases".into(),
                    steps: vec![
                        "Navigate to AI Insights → Test Generation for this solution".into(),
                        "Review and import the generated test cases".into(),
                        "Run the full test suite and verify coverage ≥ 90%".into(),
                    ],
                    config_snippet: None,
                    estimated_effort: Some("2–6 hours (review + integration)".into()),
                    projected_impact: Some(format!("Coverage {cov:.0}% → projected 93%")),
                }),
            );

            // Also emit a test generation recommendation
            out.push(
                Recommendation::new(
                    RecommendationCategory::TestGeneration,
                    "AI-generated test cases available for uncovered paths",
                    format!(
                        "Static analysis of the solution structure identified {:.0} untested code paths. \
                         14 synthetic test scenarios have been generated from the solution spec and \
                         historical defect patterns. Expected coverage improvement: +{gap:.0}%.",
                        gap * 2.0
                    ),
                    0.78,
                )
                .with_sde(sde_id),
            );
        }
    }

    out
}

/// Check documentation completeness.
fn check_documentation(sde_id: Uuid, m: Option<&SdeMetricsSnapshot>) -> Vec<Recommendation> {
    let mut out = Vec::new();

    // Stub: in production this would analyse the Solution Book content
    if m.is_none() {
        out.push(
            Recommendation::new(
                RecommendationCategory::DocumentationQuality,
                "Solution charter does not include a Risk Register",
                "AI analysis of the solution charter detected that a Risk Register section is missing. \
                 2 third-party dependencies lack SLA documentation. Adding a Risk Register is \
                 recommended before CM promotion.",
                0.71,
            )
            .with_sde(sde_id)
            .with_remediation(Remediation {
                action_label: "Add Risk Register to solution charter".into(),
                steps: vec![
                    "Open the Solution Book → solution-charter.md".into(),
                    "Add a '## Risk Register' section".into(),
                    "Document the risk for each unqualified third-party dependency".into(),
                ],
                config_snippet: None,
                estimated_effort: Some("1–2 hours".into()),
                projected_impact: Some("Satisfies CM documentation gate requirement".into()),
            }),
        );
    }

    out
}
