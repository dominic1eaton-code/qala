# Qala (CodeForge) Software Factory OS

Qala is a polyglot implementation baseline of the CodeForge software factory operating system described in [`qala-sdd.md`](./qala-sdd.md). It provides:

- A Rust kernel (`qala-kernel`) that coordinates systems, subsystems, and event streams.
- Go control-plane services (gateway, identity, SDE, workspace, artifact).
- Scala execution and analytics services (workflow CI/CD, data platform, notifications).
- Rust intelligence and security services (AI agents, SEM).
- Shared event contracts, SQL schemas, and local orchestration.

## Systems And Subsystems

- `Kernel System` (Rust): service registry, subsystem health, command bus, event aggregation.
- `Control Plane` (Go):
  - `API Gateway`
  - `User Identity`
  - `SDE Management`
  - `Workspace/CMS`
  - `Artifact/Package`
- `Execution Plane` (Scala):
  - `Workflow/CI-CD`
  - `Data Platform`
  - `Notifications`
- `Intelligence & Security Plane` (Rust):
  - `AI Agents`
  - `Security/SEM`

## Repository Layout

```text
qala/
├── contracts/
│   ├── events/
│   └── openapi/
├── db/postgres/
├── docs/
├── go/
├── rust/
├── scala/
└── infra/
```

## Core API Surface

The implemented endpoints follow the SDD’s implementation table:

- `POST /users`, `GET/PUT/DELETE /users/{id}`
- `POST /sdes`, `GET/PATCH/DELETE /sdes/{id}`, `POST /sdes/{id}/snapshot`, `POST /sdes/{id}/rollback`
- `POST /workspaces`, `GET/PUT/DELETE /workspaces/{id}`, `POST /workspaces/{id}/content`
- `POST /pipelines/run`, `GET /pipelines/{id}/status`
- `POST /artifacts/upload`, `GET /artifacts/{id}`, `GET /artifacts/{id}/download`, `DELETE /artifacts/{id}`
- `POST /data/pipeline/run`, `GET /data/analytics`
- `GET /recommendations`, `POST /analyze_sde`
- `GET /threats`, `POST /policy/update`, `POST /scan_sde`
- `POST /notify`, `GET /notifications/{id}`

## Event Topics

The platform uses the SDD-aligned topic set:

- `USER_EVENTS`
- `SDE_EVENTS`
- `CMS_EVENTS`
- `BUILD_EVENTS`
- `ARTIFACT_EVENTS`
- `DATA_EVENTS`
- `SECURITY_EVENTS`
- `NOTIFICATIONS`
- `AI_RECOMMENDATIONS`

See [`contracts/events/schema.json`](./contracts/events/schema.json).

## Run Locally

Use the stack in [`infra/docker-compose.yml`](./infra/docker-compose.yml) for Kafka and Postgres, then run each language workspace:

- Go services from `./go`
- Rust kernel/services from `./rust`
- Scala services from `./scala`

Service default ports are documented in [`docs/architecture.md`](./docs/architecture.md).
