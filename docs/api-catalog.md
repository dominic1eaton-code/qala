# API Catalog

## Gateway (Go, `:8080`)

- `GET /health`
- `GET /routes`
- `ANY /api/*` proxy to downstream services

## User Identity (Go, `:8081`)

- `POST /users`
- `GET /users`
- `GET /users/{id}`
- `PUT /users/{id}`
- `DELETE /users/{id}`

## SDE Management (Go, `:8082`)

- `POST /sdes`
- `GET /sdes`
- `GET /sdes/{id}`
- `PATCH /sdes/{id}`
- `DELETE /sdes/{id}`
- `POST /sdes/{id}/snapshot`
- `POST /sdes/{id}/rollback`

## Workspace CMS (Go, `:8083`)

- `POST /workspaces`
- `GET /workspaces`
- `GET /workspaces/{id}`
- `PUT /workspaces/{id}`
- `DELETE /workspaces/{id}`
- `POST /workspaces/{id}/content`
- `GET /workspaces/{id}/content/{filePath}`

## Workflow CI/CD (Scala, `:8084`)

- `POST /pipelines/run`
- `GET /pipelines/{id}/status`

## Artifact Package (Go, `:8085`)

- `POST /artifacts/upload`
- `GET /artifacts`
- `GET /artifacts/{id}`
- `GET /artifacts/{id}/download`
- `DELETE /artifacts/{id}`

## Data Platform (Scala, `:8086`)

- `POST /data/pipeline/run`
- `GET /data/analytics`

## AI Agents (Rust, `:8087`)

- `GET /recommendations`
- `POST /analyze_sde`

## Security SEM (Rust, `:8088`)

- `GET /threats`
- `POST /policy/update`
- `POST /scan_sde`

## Notifications (Scala, `:8089`)

- `POST /notify`
- `GET /notifications`
- `GET /notifications/{id}`

## Kernel (Rust, `:7000`)

- `GET /health`
- `GET /v1/kernel/status`
- `POST /v1/kernel/register-service`
- `POST /v1/kernel/events`
- `GET /v1/kernel/events`
- `POST /v1/kernel/command`
