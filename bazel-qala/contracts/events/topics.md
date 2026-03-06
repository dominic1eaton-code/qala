# Event Topics

| Topic | Producers | Consumers |
|---|---|---|
| `USER_EVENTS` | user-identity-service | kernel, security-sem, notifications |
| `SDE_EVENTS` | sde-management-service | kernel, workflow, security-sem |
| `CMS_EVENTS` | workspace-cms-service | kernel, ai-agents |
| `BUILD_EVENTS` | workflow-ci-cd-service | kernel, data-platform, notifications |
| `ARTIFACT_EVENTS` | artifact-package-service | kernel, workflow, data-platform |
| `DATA_EVENTS` | data-platform-service | kernel, ai-agents |
| `SECURITY_EVENTS` | security-sem-service | kernel, notifications |
| `NOTIFICATIONS` | notifications-service | kernel |
| `AI_RECOMMENDATIONS` | ai-agents-service | kernel, notifications |
