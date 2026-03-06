# Go Control Plane Services

## Services

- `cmd/api-gateway` (`:8080`)
- `cmd/user-identity-service` (`:8081`)
- `cmd/sde-management-service` (`:8082`) - SDE lifecycle, solutions, and solution factories
- `cmd/workspace-cms-service` (`:8083`)
- `cmd/artifact-package-service` (`:8085`)

## Run

```powershell
cd go
go run .\cmd\user-identity-service
go run .\cmd\sde-management-service
go run .\cmd\workspace-cms-service
go run .\cmd\artifact-package-service
go run .\cmd\api-gateway
```
