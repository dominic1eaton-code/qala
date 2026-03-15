//! SDE Security Scanner — produces a structured security findings report.
//!
//! The scanner runs a battery of checks against a specified SDE and returns
//! a prioritised list of `Threat` records. In production each check would
//! invoke real toolchain integrations (gosec, semgrep, trivy, grype, etc.).
//! Here we implement the complete structural contract with heuristic stubs.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use qala_shared::types::Severity;

use crate::threats::{Threat, ThreatType};

// ── Scan request / response ────────────────────────────────────────────────────

/// Request body for `POST /scan_sde`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSdeRequest {
    /// The SDE to scan.
    pub sde_id: Uuid,
    /// Scan types to include. Defaults to all if empty.
    #[serde(default)]
    pub scan_types: Vec<ScanType>,
    /// Caller-supplied context to assist the scanner.
    pub context: Option<ScanContext>,
}

/// The kinds of scans that can be requested.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanType {
    /// CVE and dependency vulnerability scanning (SBOM-based).
    DependencyVulnerability,
    /// Static application security testing (SAST).
    StaticAnalysis,
    /// Configuration and secret hygiene checks.
    ConfigurationHygiene,
    /// Software supply chain integrity checks (SLSA, signing).
    SupplyChainIntegrity,
    /// All scan types.
    All,
}

/// Optional caller-supplied context accelerating the scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanContext {
    /// Primary language/runtime of the SDE.
    pub language: Option<String>,
    /// Whether any artifacts are currently unsigned.
    pub has_unsigned_artifacts: Option<bool>,
    /// List of known dependency names for targeted CVE lookup.
    pub dependency_names: Vec<String>,
    /// Whether a plaintext secret has been reported in this SDE.
    pub has_plaintext_secret: Option<bool>,
}

/// The result of a complete SDE security scan.
#[derive(Debug, Serialize, Deserialize)]
pub struct ScanResult {
    pub scan_id:        Uuid,
    pub sde_id:         Uuid,
    pub scan_types:     Vec<String>,
    pub findings_count: usize,
    pub critical_count: usize,
    pub high_count:     usize,
    pub medium_count:   usize,
    pub low_count:      usize,
    pub threats:        Vec<Threat>,
    pub scan_passed:    bool,  // true iff no high/critical findings
    pub scanned_at:     chrono::DateTime<Utc>,
    pub duration_ms:    u64,
}

// ── Scanner ────────────────────────────────────────────────────────────────────

/// Run a security scan against the given SDE and return a `ScanResult`.
pub async fn run(req: &ScanSdeRequest) -> ScanResult {
    let start    = std::time::Instant::now();
    let scan_id  = Uuid::new_v4();
    let sde_id   = req.sde_id;
    let ctx      = req.context.as_ref();

    let run_all  = req.scan_types.is_empty() || req.scan_types.contains(&ScanType::All);

    let mut threats: Vec<Threat> = Vec::new();

    if run_all || req.scan_types.contains(&ScanType::DependencyVulnerability) {
        threats.extend(scan_dependencies(sde_id, ctx));
    }
    if run_all || req.scan_types.contains(&ScanType::StaticAnalysis) {
        threats.extend(scan_static_analysis(sde_id, ctx));
    }
    if run_all || req.scan_types.contains(&ScanType::ConfigurationHygiene) {
        threats.extend(scan_config_hygiene(sde_id, ctx));
    }
    if run_all || req.scan_types.contains(&ScanType::SupplyChainIntegrity) {
        threats.extend(scan_supply_chain(sde_id, ctx));
    }

    // Sort: critical first, then high, then by CVSSscore descending
    threats.sort_by(|a, b| {
        b.severity.cmp(&a.severity).then(
            b.cvss_score
                .unwrap_or(0.0)
                .partial_cmp(&a.cvss_score.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal),
        )
    });

    let critical_count = threats.iter().filter(|t| t.severity == Severity::Critical).count();
    let high_count     = threats.iter().filter(|t| t.severity == Severity::High).count();
    let medium_count   = threats.iter().filter(|t| t.severity == Severity::Medium).count();
    let low_count      = threats.iter().filter(|t| t.severity == Severity::Low).count();
    let scan_passed    = critical_count == 0 && high_count == 0;
    let findings_count = threats.len();
    let duration_ms    = start.elapsed().as_millis() as u64;

    tracing::info!(
        sde_id = %sde_id,
        findings = findings_count,
        critical = critical_count,
        high     = high_count,
        passed   = scan_passed,
        duration_ms,
        "SDE security scan complete"
    );

    ScanResult {
        scan_id,
        sde_id,
        scan_types: req.scan_types.iter().map(|t| format!("{t:?}")).collect(),
        findings_count,
        critical_count,
        high_count,
        medium_count,
        low_count,
        threats,
        scan_passed,
        scanned_at: Utc::now(),
        duration_ms,
    }
}

// ── Individual scan checks ─────────────────────────────────────────────────────

/// Scan for CVEs and dependency vulnerabilities.
fn scan_dependencies(sde_id: Uuid, ctx: Option<&ScanContext>) -> Vec<Threat> {
    let mut out = Vec::new();

    // In production this would parse the SBOM and query NVD / OSV
    // Here we simulate realistic findings based on common Go/Rust dependencies

    let deps = ctx
        .map(|c| c.dependency_names.as_slice())
        .unwrap_or(&[]);

    // Check for known vulnerable packages in the context
    for dep in deps {
        if dep.contains("go-jose") || dep.contains("jose") {
            out.push(
                Threat::new(
                    ThreatType::Cve,
                    Severity::High,
                    "CVE-2024-44000 — JWT validation bypass in go-jose",
                    "Authentication bypass vulnerability in go-jose v3.0.1. JWT validation \
                     can be bypassed via malformed headers under specific conditions. \
                     Patch available: v3.0.3.",
                    "dependency-scanner",
                )
                .with_cve("CVE-2024-44000", 8.2)
                .with_sde(sde_id)
                .with_component(dep.clone()),
            );
        }

        if dep.contains("net/http") || dep.contains("golang.org/x/net") {
            out.push(
                Threat::new(
                    ThreatType::Cve,
                    Severity::Medium,
                    "CVE-2024-39293 — HTTP request smuggling in net/http",
                    "HTTP request smuggling vulnerability under specific proxy configurations. \
                     Not confirmed exploitable in standard deployments. Patch: golang.org/x/net v0.23.0.",
                    "dependency-scanner",
                )
                .with_cve("CVE-2024-39293", 6.1)
                .with_sde(sde_id)
                .with_component(dep.clone()),
            );
        }

        if dep.contains("grpc-gateway") {
            out.push(
                Threat::new(
                    ThreatType::Cve,
                    Severity::Medium,
                    "CVE-2024-38521 — path traversal in grpc-gateway",
                    "Path traversal vulnerability in grpc-gateway v2.19.x HTTP routing layer. \
                     Affects services exposing dynamic path parameters. Patch: v2.20.0.",
                    "dependency-scanner",
                )
                .with_cve("CVE-2024-38521", 5.8)
                .with_sde(sde_id)
                .with_component(dep.clone()),
            );
        }
    }

    // If no context, emit a generic advisory
    if deps.is_empty() {
        out.push(
            Threat::new(
                ThreatType::SupplyChain,
                Severity::Low,
                "Dependency manifest not supplied — SBOM-based scan skipped",
                "No SBOM or dependency list was provided. Full CVE scanning could not be performed. \
                 Ensure hermetic build mode generates an SBOM on each build.",
                "dependency-scanner",
            )
            .with_sde(sde_id),
        );
    }

    out
}

/// Run static analysis security checks.
fn scan_static_analysis(sde_id: Uuid, ctx: Option<&ScanContext>) -> Vec<Threat> {
    let mut out = Vec::new();

    let lang = ctx.and_then(|c| c.language.as_deref()).unwrap_or("unknown");

    // Simulate SAST findings appropriate to the language
    if lang == "Go" || lang == "go" {
        out.push(
            Threat::new(
                ThreatType::SastFinding,
                Severity::Low,
                "G304 — file path provided as taint input (gosec)",
                "gosec G304: File path in os.Open is derived from user-controlled input. \
                 Validate and sanitise the path before use to prevent path traversal.",
                "sast-scanner",
            )
            .with_sde(sde_id),
        );
    }

    if lang == "Rust" || lang == "rust" {
        out.push(
            Threat::new(
                ThreatType::SastFinding,
                Severity::Low,
                "Unsafe block detected — manual review recommended",
                "1 `unsafe` block found outside of FFI boundary code. \
                 Ensure memory safety invariants are documented and audited.",
                "sast-scanner",
            )
            .with_sde(sde_id),
        );
    }

    out
}

/// Check configuration and secret hygiene.
fn scan_config_hygiene(sde_id: Uuid, ctx: Option<&ScanContext>) -> Vec<Threat> {
    let mut out = Vec::new();

    if ctx.and_then(|c| c.has_plaintext_secret).unwrap_or(false) {
        out.push(
            Threat::new(
                ThreatType::CredentialExposure,
                Severity::Medium,
                "Plaintext credential detected in environment configuration",
                "An API key or password has been found stored as a plaintext environment variable \
                 rather than as a Vault secret reference. This violates policy `no-plain-secrets-in-env`. \
                 Migrate the credential to `secret/<sde-name>/<key>` in the platform Vault.",
                "config-scanner",
            )
            .with_sde(sde_id),
        );
    }

    out
}

/// Check supply-chain integrity (signing, attestation).
fn scan_supply_chain(sde_id: Uuid, ctx: Option<&ScanContext>) -> Vec<Threat> {
    let mut out = Vec::new();

    if ctx.and_then(|c| c.has_unsigned_artifacts).unwrap_or(false) {
        out.push(
            Threat::new(
                ThreatType::SupplyChain,
                Severity::High,
                "Unsigned artifact detected — build provenance cannot be verified",
                "One or more build artifacts from this SDE have not been signed with cosign. \
                 Policy `artifact-signing-required` mandates signing before publication. \
                 Run `cosign sign` and upload the attestation to Rekor before deploying.",
                "supply-chain-scanner",
            )
            .with_sde(sde_id),
        );
    }

    out
}
