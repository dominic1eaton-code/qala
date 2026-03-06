$ErrorActionPreference = "Stop"

$kernel = "http://localhost:7000/v1/kernel/register-service"
$registrations = @(
    @{ service_name = "api-gateway"; system_name = "ControlPlane"; subsystem_name = "Gateway" },
    @{ service_name = "user-identity-service"; system_name = "ControlPlane"; subsystem_name = "Identity" },
    @{ service_name = "sde-management-service"; system_name = "ControlPlane"; subsystem_name = "SDE" },
    @{ service_name = "workspace-cms-service"; system_name = "ControlPlane"; subsystem_name = "Workspace" },
    @{ service_name = "artifact-package-service"; system_name = "ControlPlane"; subsystem_name = "Artifact" },
    @{ service_name = "workflow-ci-cd-service"; system_name = "ExecutionPlane"; subsystem_name = "Workflow" },
    @{ service_name = "data-platform-service"; system_name = "ExecutionPlane"; subsystem_name = "Data" },
    @{ service_name = "notifications-service"; system_name = "ExecutionPlane"; subsystem_name = "Notifications" },
    @{ service_name = "ai-agents-service"; system_name = "Intelligence"; subsystem_name = "AI" },
    @{ service_name = "security-sem-service"; system_name = "Security"; subsystem_name = "SEM" }
)

foreach ($reg in $registrations) {
    $json = $reg | ConvertTo-Json -Compress
    Invoke-RestMethod -Method Post -Uri $kernel -ContentType "application/json" -Body $json | Out-Null
    Write-Host "Registered $($reg.service_name)"
}
