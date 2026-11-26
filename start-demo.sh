#!/usr/bin/env bash
# =============================================================================
# Coralys Workforce & ChronoSentiment Demo Launcher
# Starts backend and frontend services:
#   • UltraCrew  (airline crew scheduling)   → http://localhost:3000
#   • UltraRoster (nurse rostering)          → http://localhost:5173
#   • ChronoSentiment UI (paper gate 5)       → http://localhost:5174
# =============================================================================

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_DIR="$REPO_ROOT/services/ultracrew_server"
CORALYS_DIR="$REPO_ROOT/services/coralys_decision_server"
ULTRACREW_DIR="$REPO_ROOT/apps/ultracrew-pilot-portal"
ULTRAROSTER_DIR="$REPO_ROOT/ui/ultracrew"
CHRONO_UI_DIR="$REPO_ROOT/services/ui"

BACKEND_PORT="${PORT:-3001}"
ULTRACREW_PORT=3000
ULTRAROSTER_PORT=5173
CHRONO_UI_PORT=5174

GREEN='\033[0;32m'; YELLOW='\033[1;33m'; RED='\033[0;31m'; NC='\033[0m'
info()  { echo -e "${GREEN}[demo]${NC} $*"; }
warn()  { echo -e "${YELLOW}[demo]${NC} $*"; }
error() { error_msg="$1"; echo -e "${RED}[demo]${NC} $error_msg" >&2; }

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
    if ! command -v cargo &>/dev/null; then
        error "cargo not found. Install Rust: https://rustup.rs"
        exit 1
    fi
    if ! command -v node &>/dev/null; then
        error "node not found. Install Node.js: https://nodejs.org"
        exit 1
    fi
    info "cargo $(cargo --version | head -1 | cut -d' ' -f2), node $(node --version)"
}

start_backend() {
    kill_port "$BACKEND_PORT"
    info "Starting backend on port $BACKEND_PORT..."
    cargo run --manifest-path "$BACKEND_DIR/Cargo.toml" --bin ultracrew_server --release &
    BACKEND_PID=$!
    info "Backend PID $BACKEND_PID"
}

start_ultracrew() {
    if [[ -d "$ULTRACREW_DIR" ]]; then
        kill_port "$ULTRACREW_PORT"
        info "Starting UltraCrew (airline) on port $ULTRACREW_PORT..."
        (cd "$ULTRACREW_DIR" && BROWSER=none npm start 2>&1) &
    fi
}

start_ultraroster() {
    if [[ -d "$ULTRAROSTER_DIR" ]]; then
        kill_port "$ULTRAROSTER_PORT"
        info "Starting UltraRoster (healthcare) on port $ULTRAROSTER_PORT..."
        (cd "$ULTRAROSTER_DIR" && BROWSER=none npm run dev 2>&1) &
    fi
}

start_chrono_ui() {
    if [[ -d "$CHRONO_UI_DIR" ]]; then
        kill_port "$CHRONO_UI_PORT"
        info "Starting ChronoSentiment UI on port $CHRONO_UI_PORT..."
        (cd "$CHRONO_UI_DIR" && BROWSER=none npm run dev -- --port $CHRONO_UI_PORT 2>&1) &
    fi
}

cleanup() {
    info "Shutting down..."
    kill_port "$BACKEND_PORT"
    kill_port "$ULTRACREW_PORT"
    kill_port "$ULTRAROSTER_PORT"
    kill_port "$CHRONO_UI_PORT"
}
trap cleanup EXIT INT TERM

check_deps
start_backend
start_ultracrew
start_ultraroster
start_chrono_ui

info "=== Applications Running ==="
info "  UltraCrew   (Airline)     → http://localhost:$ULTRACREW_PORT"
info "  UltraRoster (Healthcare) → http://localhost:$ULTRAROSTER_PORT"
info "  ChronoUI    (Gate 5)      → http://localhost:$CHRONO_UI_PORT"
info "Press Ctrl+C to stop all."
wait