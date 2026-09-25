#!/usr/bin/env bash
# =============================================================================
# ChronoSentiment Platform Launcher
# Starts the Coralys Decision Server backend + Gate 5 Observability UI:
#   • Backend  (coralys_decision_server) → http://localhost:3001
#   • Frontend (services/ui)             → http://localhost:5173
# =============================================================================

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_DIR="$REPO_ROOT/services/coralys_decision_server"
UI_DIR="$REPO_ROOT/services/ui"
BACKEND_PORT="${PORT:-3001}"
UI_PORT=5173

GREEN='\033[0;32m'; YELLOW='\033[1;33m'; RED='\033[0;31m'; NC='\033[0m'
info()  { echo -e "${GREEN}[chronosentiment]${NC} $*"; }
warn()  { echo -e "${YELLOW}[chronosentiment]${NC} $*"; }
error() { echo -e "${RED}[chronosentiment]${NC} $*" >&2; }

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
    info "Starting Coralys Decision Server on port $BACKEND_PORT..."
    cargo run --manifest-path "$BACKEND_DIR/Cargo.toml" --release &
    BACKEND_PID=$!
    info "Backend PID $BACKEND_PID"
}

start_ui() {
    kill_port "$UI_PORT"
    info "Starting ChronoSentiment UI on port $UI_PORT..."
    (cd "$UI_DIR" && npm run dev) &
    UI_PID=$!
    info "Frontend PID $UI_PID"
}

cleanup() {
    info "Shutting down platform services..."
    kill_port "$BACKEND_PORT"
    kill_port "$UI_PORT"
}
trap cleanup EXIT INT TERM

check_deps
start_backend
start_ui

info "ChronoSentiment Platform is running!"
info "  Backend API:  http://localhost:$BACKEND_PORT"
info "  Frontend UI: http://localhost:$UI_PORT"
info "Press Ctrl+C to stop."
wait