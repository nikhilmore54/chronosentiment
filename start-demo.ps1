# =============================================================================
# UltraCrew Demo Launcher (Windows PowerShell)
# Starts the Rust backend and React frontend, waits for both to be healthy,
# then opens the browser.
#
# Usage (from repo root in PowerShell):
#   Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
#   .\start-demo.ps1
#
# Requirements:
#   - Rust toolchain (cargo)  https://rustup.rs
#   - Node.js >= 16            https://nodejs.org
# =============================================================================

$ErrorActionPreference = "Stop"

$RepoRoot    = Split-Path -Parent $MyInvocation.MyCommand.Path
$BackendDir  = Join-Path $RepoRoot "services\ultracrew_server"
$FrontendDir = Join-Path $RepoRoot "apps\ultracrew-pilot-portal"
$BackendPort = if ($env:PORT) { $env:PORT } else { "3001" }
$FrontendPort = "3000"
$HealthUrl   = "http://localhost:$BackendPort/api/health"
$FrontendUrl = "http://localhost:$FrontendPort"

function Write-Info  { param($msg) Write-Host "[demo] $msg" -ForegroundColor Green }
function Write-Warn  { param($msg) Write-Host "[demo] $msg" -ForegroundColor Yellow }
function Write-Err   { param($msg) Write-Host "[demo] $msg" -ForegroundColor Red }

# ── Dependency checks ─────────────────────────────────────────────────────────
function Check-Deps {
    $missing = $false
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Err "cargo not found. Install Rust: https://rustup.rs"
        $missing = $true
    }
    if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
        Write-Err "node not found. Install Node.js: https://nodejs.org"
        $missing = $true
    }
    if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
        Write-Err "npm not found. Install Node.js: https://nodejs.org"
        $missing = $true
    }
    if ($missing) { exit 1 }
    $cargoVer = (cargo --version 2>$null) -replace "cargo ",""
    $nodeVer  = node --version 2>$null
    Write-Info "cargo $cargoVer, node $nodeVer"
}

# ── Install frontend dependencies if needed ───────────────────────────────────
function Install-FrontendDeps {
    $nodeModules = Join-Path $FrontendDir "node_modules"
    if (-not (Test-Path $nodeModules)) {
        Write-Info "Installing frontend dependencies (first run)..."
        Push-Location $FrontendDir
        npm install --silent
        Pop-Location
    }
}

# ── Start backend ─────────────────────────────────────────────────────────────
$BackendProcess  = $null
$FrontendProcess = $null

function Start-Backend {
    Write-Info "Building and starting backend on port $BackendPort..."
    Write-Info "(First build may take 1-2 minutes)"
    $env:PORT = $BackendPort
    $script:BackendProcess = Start-Process -FilePath "cargo" `
        -ArgumentList "run","--manifest-path","$BackendDir\Cargo.toml","--release" `
        -RedirectStandardOutput "$env:TEMP\ultracrew-backend.log" `
        -RedirectStandardError  "$env:TEMP\ultracrew-backend-err.log" `
        -PassThru -NoNewWindow
    Write-Info "Backend PID $($script:BackendProcess.Id)  (logs: $env:TEMP\ultracrew-backend.log)"
}

# ── Start frontend ────────────────────────────────────────────────────────────
function Start-Frontend {
    Write-Info "Starting frontend on port $FrontendPort..."
    $env:BROWSER = "none"
    $script:FrontendProcess = Start-Process -FilePath "npm" `
        -ArgumentList "start" `
        -WorkingDirectory $FrontendDir `
        -RedirectStandardOutput "$env:TEMP\ultracrew-frontend.log" `
        -RedirectStandardError  "$env:TEMP\ultracrew-frontend-err.log" `
        -PassThru -NoNewWindow
    Write-Info "Frontend PID $($script:FrontendProcess.Id)  (logs: $env:TEMP\ultracrew-frontend.log)"
}

# ── Wait for backend health check ─────────────────────────────────────────────
function Wait-ForBackend {
    Write-Info "Waiting for backend health check..."
    $attempts = 0; $max = 120
    while ($attempts -lt $max) {
        try {
            $resp = Invoke-WebRequest -Uri $HealthUrl -UseBasicParsing -TimeoutSec 2 -ErrorAction Stop
            if ($resp.StatusCode -eq 200) {
                Write-Info "Backend healthy."
                return
            }
        } catch {}
        if ($script:BackendProcess.HasExited) {
            Write-Err "Backend process exited unexpectedly."
            Get-Content "$env:TEMP\ultracrew-backend-err.log" -Tail 20 | ForEach-Object { Write-Err $_ }
            exit 1
        }
        Start-Sleep -Seconds 1
        $attempts++
    }
    Write-Err "Backend did not become healthy within ${max}s."
    Get-Content "$env:TEMP\ultracrew-backend-err.log" -Tail 20 | ForEach-Object { Write-Err $_ }
    exit 1
}

# ── Wait for frontend ─────────────────────────────────────────────────────────
function Wait-ForFrontend {
    Write-Info "Waiting for frontend..."
    $attempts = 0; $max = 60
    while ($attempts -lt $max) {
        try {
            $resp = Invoke-WebRequest -Uri $FrontendUrl -UseBasicParsing -TimeoutSec 2 -ErrorAction Stop
            if ($resp.StatusCode -eq 200) {
                Write-Info "Frontend ready."
                return
            }
        } catch {}
        Start-Sleep -Seconds 1
        $attempts++
    }
    Write-Warn "Frontend did not respond within ${max}s - opening browser anyway."
}

# ── Cleanup ───────────────────────────────────────────────────────────────────
function Cleanup {
    Write-Info "Shutting down..."
    if ($script:BackendProcess  -and -not $script:BackendProcess.HasExited)  { $script:BackendProcess.Kill()  }
    if ($script:FrontendProcess -and -not $script:FrontendProcess.HasExited) { $script:FrontendProcess.Kill() }
}

# ── Main ──────────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "  ██╗   ██╗██╗  ████████╗██████╗  █████╗  ██████╗██████╗ ███████╗██╗    ██╗" -ForegroundColor Cyan
Write-Host "  ██║   ██║██║  ╚══██╔══╝██╔══██╗██╔══██╗██╔════╝██╔══██╗██╔════╝██║    ██║" -ForegroundColor Cyan
Write-Host "  ██║   ██║██║     ██║   ██████╔╝███████║██║     ██████╔╝█████╗  ██║ █╗ ██║" -ForegroundColor Cyan
Write-Host "  ██║   ██║██║     ██║   ██╔══██╗██╔══██║██║     ██╔══██╗██╔══╝  ██║███╗██║" -ForegroundColor Cyan
Write-Host "  ╚██████╔╝███████╗██║   ██║  ██║██║  ██║╚██████╗██║  ██║███████╗╚███╔███╔╝" -ForegroundColor Cyan
Write-Host "   ╚═════╝ ╚══════╝╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝╚══════╝ ╚══╝╚══╝" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Workforce Decision Platform  |  Demo Launcher" -ForegroundColor Cyan
Write-Host ""

try {
    Check-Deps
    Install-FrontendDeps
    Start-Backend
    Start-Frontend
    Wait-ForBackend
    Wait-ForFrontend
    Start-Process $FrontendUrl

    Write-Info "Demo running."
    Write-Info "  Frontend : $FrontendUrl"
    Write-Info "  Backend  : http://localhost:$BackendPort"
    Write-Info "  Logs     : $env:TEMP\ultracrew-backend.log  $env:TEMP\ultracrew-frontend.log"
    Write-Info ""
    Write-Info "Press Ctrl+C to stop both processes."

    # Keep script alive
    while ($true) { Start-Sleep -Seconds 5 }
} finally {
    Cleanup
}