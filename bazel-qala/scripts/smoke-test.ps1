$ErrorActionPreference = "Stop"

function Invoke-Json {
    param(
        [string]$Method,
        [string]$Uri,
        [string]$Body = "{}"
    )
    Invoke-RestMethod -Method $Method -Uri $Uri -ContentType "application/json" -Body $Body
}

Write-Host "Checking health endpoints..."
$ports = 7000,8080,8081,8082,8083,8084,8085,8086,8087,8088,8089
foreach ($port in $ports) {
    try {
        $h = Invoke-RestMethod -Method Get -Uri "http://localhost:$port/health"
        Write-Host "  :$port -> $($h.status)"
    } catch {
        Write-Host "  :$port -> unavailable"
    }
}

Write-Host "`nRunning control-plane flow..."
$user = Invoke-Json -Method Post -Uri "http://localhost:8081/users" -Body '{"name":"Dev One","email":"dev1@qala.local"}'
$sde = Invoke-Json -Method Post -Uri "http://localhost:8082/sdes" -Body "{`"owner_id`":`"$($user.id)`",`"template_id`":`"tmpl-base`"}"
$workspace = Invoke-Json -Method Post -Uri "http://localhost:8083/workspaces" -Body "{`"owner_id`":`"$($user.id)`",`"name`":`"main`"}"
Invoke-Json -Method Post -Uri "http://localhost:8083/workspaces/$($workspace.id)/content" -Body '{"file_path":"README.md","content":"hello"}' | Out-Null
Invoke-Json -Method Post -Uri "http://localhost:8084/pipelines/run" -Body "{`"pipeline_id`":`"pipe-1`",`"sde_id`":`"$($sde.id)`"}" | Out-Null
Invoke-Json -Method Post -Uri "http://localhost:8085/artifacts/upload" -Body "{`"name`":`"api`",`"version`":`"v1`",`"sde_id`":`"$($sde.id)`",`"file_path`":`"dist/api.zip`"}" | Out-Null
Invoke-Json -Method Post -Uri "http://localhost:8089/notify" -Body "{`"user_id`":`"$($user.id)`",`"channel`":`"in-app`",`"message`":`"pipeline started`"}" | Out-Null

Write-Host "Smoke test completed."
