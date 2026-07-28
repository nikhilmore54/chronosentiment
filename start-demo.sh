#!/usr/bin/env bash
# =============================================================================
# Coralys Platform Demo Launcher
# Starts the shared Rust backend plus both Solution Engine frontends:
#   • UltraCrew  (airline crew scheduling)   → http://localhost:3000
#   • UltraRoster (nurse rostering)          → http://localhost:5173
#
# Usage:
#   chmod +x start-demo.sh && ./start-demo.sh [--airline-only | --healthcare-only]
#
# Requirements:
#   - Rust toolchain (cargo)  https://rustup.rs
#   - Node.js >= 16            https://nodejs.org
# =============================================================================

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_DIR="$REPO_ROOT/services/ultracrew_server"
ULTRACREW_DIR="$REPO_ROOT/apps/ultracrew-pilot-portal"
ULTRAROSTER_DIR="$REPO_ROOT/ui/ultracrew"
BACKEND_PORT="${PORT:-3001}"
ULTRACREW_PORT=3000
ULTRAROSTER_PORT=5173
HEALTH_URL="http://localhost:$BACKEND_PORT/api/health"

# ── CLI flags ────────────────────────────────────────────────────────────────
START_ULTRACREW=true
START_ULTRAROSTER=true
for arg in "$@"; do
  case "$arg" in
    --airline-only)     START_ULTRAROSTER=false ;;
    --healthcare-only)  START_ULTRACREW=false ;;
  esac
done

GREEN='\033[0;32m'; YELLOW='\033[1;33m'; RED='\033[0;31m'; NC='\033[0m'
info()  { echo -e "${GREEN}[demo]${NC} $*"; }
warn()  { echo -e "${YELLOW}[demo]${NC} $*"; }
error() { echo -e "${RED}[demo]${NC} $*" >&2; }

kill_port() {
    local port="$1"
    local pids
    pids=$(lsof -ti:"$port" 2>/dev/null || true)
    if [[ -n "$pids" ]]; then
        warn "Killing existing process(es) on port $port: $pids"
        echo "$pids" | xargs kill -9 2>/dev/null || true
        sleep 1
    fi
}

check_deps() {
    local missing=0
    if ! command -v cargo &>/dev/null; then
        error "cargo not found. Install Rust: https://rustup.rs"
        missing=1
    fi
    if ! command -v node &>/dev/null; then
        error "node not found. Install Node.js: https://nodejs.org"
        missing=1
    fi
    if ! command -v npm &>/dev/null; then
        error "npm not found. Install Node.js: https://nodejs.org"
        missing=1
    fi
    [[ $missing -ne 0 ]] && exit 1
    info "cargo $(cargo --version 2>/dev/null | head -1 | cut -d' ' -f2), node $(node --version)"
}

# ── Parallel npm install ──────────────────────────────────────────────────────
install_frontend_deps() {
    local pids=()
    if $START_ULTRACREW && [[ ! -d "$ULTRACREW_DIR/node_modules" ]]; then
        info "Installing UltraCrew dependencies in background..."
        (cd "$ULTRACREW_DIR" && npm install --silent 2>/dev/null) &
        pids+=($!)
    fi
    if $START_ULTRAROSTER && [[ ! -d "$ULTRAROSTER_DIR/node_modules" ]]; then
        info "Installing UltraRoster dependencies in background..."
        (cd "$ULTRAROSTER_DIR" && npm install --silent 2>/dev/null) &
        pids+=($!)
    fi
    if [[ ${#pids[@]} -gt 0 ]]; then
        info "Waiting for npm installs to complete..."
        for pid in "${pids[@]}"; do wait "$pid" || true; done
        info "npm installs done."
    fi
}

# ── Service launchers (each pipes through tee: log file + merged log) ─────────
MERGED_LOG=/tmp/coralys-demo.log
: >"$MERGED_LOG"   # truncate/create merged log

start_backend() {
    kill_port "$BACKEND_PORT"
    info "Starting backend on port $BACKEND_PORT..."
    if [[ -f "$REPO_ROOT/target/release/ultracrew_server" ]]; then
        PORT="$BACKEND_PORT" "$REPO_ROOT/target/release/ultracrew_server" 2>&1 \
            | tee -a /tmp/ultracrew-backend.log >>"$MERGED_LOG" &
    else
        info "(No pre-compiled binary — running cargo build, this may take 1-2 min)"
        PORT="$BACKEND_PORT" cargo run --manifest-path "$BACKEND_DIR/Cargo.toml" \
            --bin ultracrew_server --release 2>&1 \
            | tee -a /tmp/ultracrew-backend.log >>"$MERGED_LOG" &
    fi
    BACKEND_PID=$!
    echo "$BACKEND_PID" >/tmp/ultracrew-backend.pid
    info "Backend PID $BACKEND_PID"
}

start_ultracrew() {
    if ! $START_ULTRACREW; then ULTRACREW_PID=""; return; fi
    kill_port "$ULTRACREW_PORT"
    info "Starting UltraCrew (airline) on port $ULTRACREW_PORT..."
    (cd "$ULTRACREW_DIR" && BROWSER=none npm start 2>&1) \
        | tee -a /tmp/ultracrew-frontend.log >>"$MERGED_LOG" &
    ULTRACREW_PID=$!
    echo "$ULTRACREW_PID" >/tmp/ultracrew-frontend.pid
    info "UltraCrew PID $ULTRACREW_PID"
}

start_ultraroster() {
    if ! $START_ULTRAROSTER; then ULTRAROSTER_PID=""; return; fi
    kill_port "$ULTRAROSTER_PORT"
    info "Starting UltraRoster (healthcare) on port $ULTRAROSTER_PORT..."
    (cd "$ULTRAROSTER_DIR" && BROWSER=none npm run dev 2>&1) \
        | tee -a /tmp/ultraroster-frontend.log >>"$MERGED_LOG" &
    ULTRAROSTER_PID=$!
    echo "$ULTRAROSTER_PID" >/tmp/ultraroster-frontend.pid
    info "UltraRoster PID $ULTRAROSTER_PID"
}

# ── Parallel readiness checks ─────────────────────────────────────────────────
# Each check runs in a background subshell and writes "ok" or "fail" to a
# status file. The main script polls until all expected status files appear.

_wait_backend() {
    local attempts=0 max=120
    while [[ $attempts -lt $max ]]; do
        if curl -sf "$HEALTH_URL" &>/dev/null; then
            echo "ok" >/tmp/.coralys-backend-ready
            return 0
        fi
        if ! kill -0 "$BACKEND_PID" 2>/dev/null; then
            echo "fail" >/tmp/.coralys-backend-ready
            return 1
        fi
        sleep 1; ((attempts++))
    done
    echo "timeout" >/tmp/.coralys-backend-ready
}

_wait_url() {
    local url="$1" flag="$2" max="${3:-60}"
    local attempts=0
    while [[ $attempts -lt $max ]]; do
        if curl -sf "$url" &>/dev/null; then
            echo "ok" >"$flag"; return 0
        fi
        sleep 1; ((attempts++))
    done
    echo "timeout" >"$flag"
}

wait_for_all() {
    rm -f /tmp/.coralys-backend-ready /tmp/.coralys-ultracrew-ready /tmp/.coralys-ultraroster-ready

    _wait_backend &
    $START_ULTRACREW   && _wait_url "http://localhost:$ULTRACREW_PORT"   /tmp/.coralys-ultracrew-ready   60 &
    $START_ULTRAROSTER && _wait_url "http://localhost:$ULTRAROSTER_PORT" /tmp/.coralys-ultraroster-ready 60 &

    info "All services starting concurrently — waiting for readiness..."
    local spinner=('⠋' '⠙' '⠹' '⠸' '⠼' '⠴' '⠦' '⠧' '⠇' '⠏')
    local i=0
    while true; do
        local done=true
        [[ ! -f /tmp/.coralys-backend-ready ]]    && done=false
        $START_ULTRACREW   && [[ ! -f /tmp/.coralys-ultracrew-ready ]]   && done=false
        $START_ULTRAROSTER && [[ ! -f /tmp/.coralys-ultraroster-ready ]] && done=false
        $done && break
        printf "\r  %s  waiting..." "${spinner[$((i % ${#spinner[@]}))]}"
        i=$((i+1)); sleep 0.2
    done
    printf "\r                        \r"

    local backend_status
    backend_status=$(cat /tmp/.coralys-backend-ready 2>/dev/null || echo "unknown")
    if [[ "$backend_status" == "fail" ]]; then
        error "Backend process exited. Last 20 lines:"
        tail -20 /tmp/ultracrew-backend.log >&2
        exit 1
    elif [[ "$backend_status" == "timeout" ]]; then
        error "Backend did not become healthy within 120s. Last 20 lines:"
        tail -20 /tmp/ultracrew-backend.log >&2
        exit 1
    fi
    info "Backend healthy."

    if $START_ULTRACREW; then
        local uc_status; uc_status=$(cat /tmp/.coralys-ultracrew-ready 2>/dev/null || echo "unknown")
        [[ "$uc_status" == "ok" ]] && info "UltraCrew ready." \
            || warn "UltraCrew did not respond in time — it may still be starting."
    fi
    if $START_ULTRAROSTER; then
        local ur_status; ur_status=$(cat /tmp/.coralys-ultraroster-ready 2>/dev/null || echo "unknown")
        [[ "$ur_status" == "ok" ]] && info "UltraRoster ready." \
            || warn "UltraRoster did not respond in time — it may still be starting."
    fi

    rm -f /tmp/.coralys-backend-ready /tmp/.coralys-ultracrew-ready /tmp/.coralys-ultraroster-ready
}

open_browser() {
    local opener=""
    if command -v open &>/dev/null; then opener="open"
    elif command -v xdg-open &>/dev/null; then opener="xdg-open"
    fi

    if [[ -n "$opener" ]]; then
        $START_ULTRACREW    && "$opener" "http://localhost:$ULTRACREW_PORT"
        $START_ULTRAROSTER  && "$opener" "http://localhost:$ULTRAROSTER_PORT"
    else
        $START_ULTRACREW   && info "Open UltraCrew:    http://localhost:$ULTRACREW_PORT"
        $START_ULTRAROSTER && info "Open UltraRoster:  http://localhost:$ULTRAROSTER_PORT"
    fi
}

cleanup() {
    info "Shutting down..."
    [[ -f /tmp/ultracrew-backend.pid   ]] && kill "$(cat /tmp/ultracrew-backend.pid)"   2>/dev/null || true
    [[ -f /tmp/ultracrew-frontend.pid  ]] && kill "$(cat /tmp/ultracrew-frontend.pid)"  2>/dev/null || true
    [[ -f /tmp/ultraroster-frontend.pid ]] && kill "$(cat /tmp/ultraroster-frontend.pid)" 2>/dev/null || true
    rm -f /tmp/ultracrew-backend.pid /tmp/ultracrew-frontend.pid /tmp/ultraroster-frontend.pid
}
trap cleanup EXIT INT TERM

# =============================================================================
echo ""
echo "  ██████╗ ██████╗ ██████╗  █████╗ ██╗  ██╗   ██╗███████╗"
echo "  ██╔════╝██╔═══██╗██╔══██╗██╔══██╗██║  ╚██╗ ██╔╝██╔════╝"
echo "  ██║     ██║   ██║██████╔╝███████║██║   ╚████╔╝ ███████╗"
echo "  ██║     ██║   ██║██╔══██╗██╔══██║██║    ╚██╔╝  ╚════██║"
echo "  ╚██████╗╚██████╔╝██║  ██║██║  ██║███████╗██║   ███████║"
echo "   ╚═════╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝╚═╝   ╚══════╝"
echo ""
echo "  Workforce Optimisation Platform  |  Demo Launcher"
echo "  UltraCrew (Airline) + UltraRoster (Healthcare)"
echo ""

load_inrc_scenario() {
    if $START_ULTRAROSTER; then
        info "Loading default INRC scenario into backend..."
        local response
        response=$(curl -sf -X POST \
            -H "Content-Type: application/json" \
            -d '{}' \
            "http://localhost:$BACKEND_PORT/api/load-scenario" 2>/dev/null || true)
        if [[ -n "$response" ]]; then
            info "INRC scenario loaded: $response"
        else
            warn "INRC scenario load returned no response — UltraRoster simulation endpoints may return 503 until loaded manually."
        fi
    fi
}

check_deps
install_frontend_deps   # parallel npm install (skipped if node_modules exist)

# ── Sequenced startup: backend first, scenario seeded, then frontends ──────────
# The UltraRoster frontend fetches /api/nurses on first render.
# The scenario must be loaded before the frontend starts to avoid a 503 on that
# first fetch which would leave the dashboard showing "No workforce loaded".
start_backend
info "Waiting for backend to be ready before seeding scenario..."
local_wait=0
while ! curl -sf "http://localhost:$BACKEND_PORT/api/health" >/dev/null 2>&1; do
    sleep 1
    local_wait=$((local_wait + 1))
    if [[ $local_wait -ge 30 ]]; then
        warn "Backend did not start within 30s — skipping scenario seed"
        break
    fi
done
load_inrc_scenario      # POST /api/load-scenario before frontends start

start_ultracrew         # ─┐ frontends start after scenario is seeded
start_ultraroster       # ─┘
wait_for_all            # parallel readiness checks with spinner (frontends only now)
open_browser

echo ""
info "=== Demos running ==="
$START_ULTRACREW   && info "  UltraCrew  (Airline)     → http://localhost:$ULTRACREW_PORT"
$START_ULTRAROSTER && info "  UltraRoster (Healthcare) → http://localhost:$ULTRAROSTER_PORT"
info "  Backend health           → http://localhost:$BACKEND_PORT/api/health"
info ""
info "  Logs:"
info "    Backend     /tmp/ultracrew-backend.log"
$START_ULTRACREW   && info "    UltraCrew   /tmp/ultracrew-frontend.log"
$START_ULTRAROSTER && info "    UltraRoster /tmp/ultraroster-frontend.log"
info "    Merged      /tmp/coralys-demo.log"
info ""
info "Press Ctrl+C to stop all processes."
wait