#!/usr/bin/env bash
# =============================================================================
# UltraCrew Demo Launcher
# Starts the Rust backend and React frontend, waits for both to be healthy,
# then opens the browser.
#
# Usage:
#   chmod +x start-demo.sh && ./start-demo.sh
#
# Requirements:
#   - Rust toolchain (cargo)  https://rustup.rs
#   - Node.js >= 16            https://nodejs.org
# =============================================================================

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_DIR="$REPO_ROOT/services/ultracrew_server"
FRONTEND_DIR="$REPO_ROOT/apps/ultracrew-pilot-portal"
BACKEND_PORT="${PORT:-3001}"
FRONTEND_PORT=3000
HEALTH_URL="http://localhost:$BACKEND_PORT/api/health"
FRONTEND_URL="http://localhost:$FRONTEND_PORT"

GREEN='\033[0;32m'; YELLOW='\033[1;33m'; RED='\033[0;31m'; NC='\033[0m'
info()  { echo -e "${GREEN}[demo]${NC} $*"; }
warn()  { echo -e "${YELLOW}[demo]${NC} $*"; }
error() { echo -e "${RED}[demo]${NC} $*" >&2; }

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

install_frontend_deps() {
    if [[ ! -d "$FRONTEND_DIR/node_modules" ]]; then
        info "Installing frontend dependencies (first run)..."
        (cd "$FRONTEND_DIR" && npm install --silent)
    fi
}

start_backend() {
    info "Building and starting backend on port $BACKEND_PORT..."
    info "(First build may take 1-2 minutes)"
    PORT="$BACKEND_PORT" cargo run --manifest-path "$BACKEND_DIR/Cargo.toml" --bin ultracrew_server --release \
        >/tmp/ultracrew-backend.log 2>&1 &
    BACKEND_PID=$!
    echo "$BACKEND_PID" >/tmp/ultracrew-backend.pid
    info "Backend PID $BACKEND_PID  (logs: /tmp/ultracrew-backend.log)"
}

start_frontend() {
    info "Starting frontend on port $FRONTEND_PORT..."
    (cd "$FRONTEND_DIR" && BROWSER=none npm start) \
        >/tmp/ultracrew-frontend.log 2>&1 &
    FRONTEND_PID=$!
    echo "$FRONTEND_PID" >/tmp/ultracrew-frontend.pid
    info "Frontend PID $FRONTEND_PID  (logs: /tmp/ultracrew-frontend.log)"
}

wait_for_backend() {
    info "Waiting for backend health check..."
    local attempts=0 max=120
    while [[ $attempts -lt $max ]]; do
        if curl -sf "$HEALTH_URL" &>/dev/null; then
            info "Backend healthy."
            return 0
        fi
        if ! kill -0 "$BACKEND_PID" 2>/dev/null; then
            error "Backend process exited. Last 20 lines:"
            tail -20 /tmp/ultracrew-backend.log >&2
            exit 1
        fi
        sleep 1
        ((attempts++))
    done
    error "Backend did not become healthy within ${max}s. Last 20 lines:"
    tail -20 /tmp/ultracrew-backend.log >&2
    exit 1
}

wait_for_frontend() {
    info "Waiting for frontend..."
    local attempts=0 max=60
    while [[ $attempts -lt $max ]]; do
        if curl -sf "$FRONTEND_URL" &>/dev/null; then
            info "Frontend ready."
            return 0
        fi
        sleep 1
        ((attempts++))
    done
    warn "Frontend did not respond within ${max}s - opening browser anyway."
}

open_browser() {
    info "Opening $FRONTEND_URL ..."
    if command -v open &>/dev/null; then
        open "$FRONTEND_URL"
    elif command -v xdg-open &>/dev/null; then
        xdg-open "$FRONTEND_URL"
    else
        info "Open your browser at: $FRONTEND_URL"
    fi
}

cleanup() {
    info "Shutting down..."
    [[ -f /tmp/ultracrew-backend.pid  ]] && kill "$(cat /tmp/ultracrew-backend.pid)"  2>/dev/null || true
    [[ -f /tmp/ultracrew-frontend.pid ]] && kill "$(cat /tmp/ultracrew-frontend.pid)" 2>/dev/null || true
    rm -f /tmp/ultracrew-backend.pid /tmp/ultracrew-frontend.pid
}
trap cleanup EXIT INT TERM

# =============================================================================
echo ""
echo "  ██╗   ██╗██╗  ████████╗██████╗  █████╗  ██████╗██████╗ ███████╗██╗    ██╗"
echo "  ██║   ██║██║  ╚══██╔══╝██╔══██╗██╔══██╗██╔════╝██╔══██╗██╔════╝██║    ██║"
echo "  ██║   ██║██║     ██║   ██████╔╝███████║██║     ██████╔╝█████╗  ██║ █╗ ██║"
echo "  ██║   ██║██║     ██║   ██╔══██╗██╔══██║██║     ██╔══██╗██╔══╝  ██║███╗██║"
echo "  ╚██████╔╝███████╗██║   ██║  ██║██║  ██║╚██████╗██║  ██║███████╗╚███╔███╔╝"
echo "   ╚═════╝ ╚══════╝╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝╚══════╝ ╚══╝╚══╝"
echo ""
echo "  Workforce Decision Platform  |  Demo Launcher"
echo ""

check_deps
install_frontend_deps
start_backend
start_frontend
wait_for_backend
wait_for_frontend
open_browser

info "Demo running."
info "  Frontend : $FRONTEND_URL"
info "  Backend  : http://localhost:$BACKEND_PORT"
info "  Logs     : /tmp/ultracrew-backend.log  /tmp/ultracrew-frontend.log"
info ""
info "Press Ctrl+C to stop both processes."
wait