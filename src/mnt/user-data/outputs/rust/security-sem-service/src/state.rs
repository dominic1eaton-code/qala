//! Shared state for the Security SEM service.

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    policies::PolicyStore,
    threats::ThreatStore,
};

/// Shared, thread-safe state for the security SEM service.
#[derive(Debug)]
pub struct SemState {
    /// In-memory threat store.
    pub threats:     Arc<RwLock<ThreatStore>>,
    /// In-memory policy store.
    pub policies:    Arc<RwLock<PolicyStore>>,
    /// HTTP client for kernel event emission and inter-service calls.
    pub http_client: reqwest::Client,
    /// Base URL of the kernel.
    pub kernel_url:  String,
}

impl SemState {
    pub fn new() -> Self {
        let kernel_url = std::env::var("KERNEL_URL")
            .unwrap_or_else(|_| "http://localhost:7000".to_string());
        Self {
            threats:     Arc::new(RwLock::new(ThreatStore::new())),
            policies:    Arc::new(RwLock::new(PolicyStore::new())),
            http_client: reqwest::Client::new(),
            kernel_url,
        }
    }

    /// Seed the policy store with the default baseline policies.
    pub async fn seed_baseline_policies(&self) {
        use crate::policies::{Policy, PolicyEnforcement, PolicyScope};

        let baseline = vec![
            Policy::new(
                "enterprise-baseline-v2",
                "1.0.0",
                "Enterprise baseline governance policy — applied to all SDEs",
            ),
            Policy::with_rule(
                "hermetic-build-required",
                "1.0.0",
                "All builds must execute in hermetic, isolated environments",
                "hermetic_build_required",
                PolicyScope::AllSdes,
                PolicyEnforcement::Block,
                "All build pipelines must use BUILD_MODE=hermetic",
            ),
            Policy::with_rule(
                "no-plain-secrets-in-env",
                "1.0.0",
                "Credentials must be stored in Vault, not as plaintext env vars",
                "no_plain_secrets_in_env",
                PolicyScope::AllSdes,
                PolicyEnforcement::Block,
                "Plaintext credential detected in environment variable",
            ),
            Policy::with_rule(
                "artifact-signing-required",
                "1.0.0",
                "All published artifacts must be signed via cosign",
                "artifact_signing_required",
                PolicyScope::Artifacts,
                PolicyEnforcement::Block,
                "Artifact publication requires valid cosign signature",
            ),
            Policy::with_rule(
                "sast-on-every-build",
                "1.0.0",
                "SAST scanning must run as part of every CI build pipeline",
                "sast_on_every_build",
                PolicyScope::BuildPipelines,
                PolicyEnforcement::Block,
                "Build pipeline must include a SAST scan stage",
            ),
            Policy::with_rule(
                "cve-high-block-deploy",
                "1.0.0",
                "High-severity CVEs block deployment to production",
                "cve_high_block_deploy",
                PolicyScope::ProductionSdes,
                PolicyEnforcement::Alert,
                "CVSS ≥ 7.0 CVE detected — deployment blocked",
            ),
            Policy::with_rule(
                "mfa-required-admin",
                "1.0.0",
                "All admin users must authenticate with MFA",
                "mfa_required_admin",
                PolicyScope::Users,
                PolicyEnforcement::Block,
                "Admin role requires MFA authentication",
            ),
        ];

        let mut store = self.policies.write().await;
        for p in baseline {
            store.upsert(p);
        }

        tracing::info!(count = store.count(), "baseline policies seeded");
    }
}

impl Default for SemState {
    fn default() -> Self {
        Self::new()
    }
}
