# SDD Traceability Matrix

This maps the recurring implementation sections in `qala-sdd.md` (CodeForge implementation table, API catalog, and microservices skeleton) to concrete code in this repository.

| SDD Capability | Implemented In |
|---|---|
| API Gateway routing (`/api/*`) | `go/cmd/api-gateway/main.go` |
| User CRUD + `USER_EVENTS` | `go/cmd/user-identity-service/main.go` |
| SDE lifecycle + snapshot/rollback + `SDE_EVENTS` | `go/cmd/sde-management-service/main.go` |
| Workspace CRUD + content operations + `CMS_EVENTS` | `go/cmd/workspace-cms-service/main.go` |
| Pipeline run/status + `BUILD_EVENTS` | `scala/workflow-ci-cd-service/src/main/scala/com/qala/workflow/Main.scala` |
| Artifact upload/download + `ARTIFACT_EVENTS` | `go/cmd/artifact-package-service/main.go` |
| Data pipeline + analytics + `DATA_EVENTS` | `scala/data-platform-service/src/main/scala/com/qala/data/Main.scala` |
| AI recommendations + SDE analysis + `AI_RECOMMENDATIONS` | `rust/ai-agents-service/src/main.rs` |
| Security threats/policies/scan + `SECURITY_EVENTS` | `rust/security-sem-service/src/main.rs` |
| Notification dispatch + `NOTIFICATIONS` | `scala/notifications-service/src/main/scala/com/qala/notifications/Main.scala` |
| Kernel OS coordination (systems/subsystems/events/commands) | `rust/qala-kernel/src/main.rs`, `rust/qala-kernel/src/state.rs` |
| Shared event contract | `contracts/events/schema.json`, `rust/qala-shared/src/lib.rs`, `go/internal/platform/events/events.go` |
| DB schema blueprint | `db/postgres/*.sql` |
