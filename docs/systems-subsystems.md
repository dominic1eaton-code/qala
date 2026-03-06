# Systems And Subsystems

## Kernel System

- `Registry`: tracks registered services and heartbeat status.
- `CommandBus`: executes kernel commands (`set_subsystem_status`, `set_service_status`).
- `EventAggregator`: counts and stores recent topic events.

## Control Plane

- `Gateway`: ingress and routing to microservices.
- `Identity`: users and RBAC metadata.
- `SDE`: software development environment lifecycle.
- `Workspace`: workspace/file content management.
- `Artifact`: package and binary metadata.

## Execution Plane

- `Workflow`: CI/CD pipeline trigger and status.
- `Data`: metrics and analytics computation.
- `Notifications`: user/system notification dispatch.

## Intelligence And Security Plane

- `AI`: recommendation and analysis hooks.
- `SEM`: security events, policy updates, SDE scanning/quarantine decisions.
