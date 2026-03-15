# Qala Architecture

## Service Map

| Service | Language | Port | System | Subsystem |
|---|---|---:|---|---|
| `qala-kernel` | Rust | 7000 | Kernel | Registry, Health, Command Bus |
| `api-gateway` | Go | 8080 | Control Plane | API ingress + service routing |
| `user-identity-service` | Go | 8081 | Control Plane | Identity and RBAC seed |
| `sde-management-service` | Go | 8082 | Control Plane | SDE lifecycle + snapshots + solutions + factories |
| `workspace-cms-service` | Go | 8083 | Control Plane | Workspaces and file content |
| `workflow-ci-cd-service` | Scala | 8084 | Execution Plane | Pipeline run/status |
| `artifact-package-service` | Go | 8085 | Control Plane | Artifact metadata and retrieval |
| `data-platform-service` | Scala | 8086 | Execution Plane | Analytics and metrics |
| `ai-agents-service` | Rust | 8087 | Intelligence | Recommendation stubs |
| `security-sem-service` | Rust | 8088 | Security | Threat/policy stubs |
| `notifications-service` | Scala | 8089 | Execution Plane | Message dispatch |

## Kernel Responsibilities

- Maintains system-level state for all subsystems.
- Stores event counters by topic and source service.
- Accepts commands for subsystem status changes and service registration.
- Exposes `/v1/kernel/status`, `/v1/kernel/events`, and `/v1/kernel/command`.

## Integration Pattern

- HTTP APIs for direct control-plane actions.
- Event envelopes for async propagation with SDD topic taxonomy.
- Service-level in-memory repositories for baseline implementation.

## Persistence Model

- Initial implementation: in-memory stores + SQL DDL for migration-ready schemas.
- Schema files located under `db/postgres/*.sql`.
- Production extension: separate PostgreSQL DB per microservice.

## Operating Model

1. Developer/API client calls gateway or service API.
2. Service updates local state and emits an event envelope.
3. Kernel aggregates event telemetry and subsystem health.
4. Analytics/security/notification services consume and react.
