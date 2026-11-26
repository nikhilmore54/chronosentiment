# UltraCrew Pilot Experience v1.0 — Deployment Audit

**Date:** 2026-07-28  
**Branch:** governance-hardening  
**Target stack:** Vercel (frontend) + Render (backend) + Supabase (persistent storage)

---

## 1. Frontend

**Directory:** `apps/ultracrew-pilot-portal/`

| Item | Value |
|------|-------|
| Framework | Create React App 5.0.1 (React 18) |
| Build command | `npm run build` |
| Output directory | `build/` |
| Node version | ≥ 16 |
| Entry point | `src/App.js` |
| Static assets | `public/` (index.html, favicon, manifest) |
| Routing | Single-page app — no client-side router, no 404 rewrite needed |

**Environment variables required:**

| Variable | Purpose | Example |
|----------|---------|---------|
| `REACT_APP_API_URL` | Backend base URL | `https://ultracrew-api.onrender.com` |

**⚠ Blocking issue — hardcoded proxy:**

All API calls use relative paths (`/api/csrf-token`, `/api/schedule`, etc.) relying on the CRA `"proxy": "http://localhost:3001"` in `package.json`. This proxy is a development-only feature and does not work in production builds.

**Fix required:** Add a `getApiBase()` helper that reads `REACT_APP_API_URL` and prefix all `fetch()` calls with it. See Section 8 (Missing Items).

---

## 2. Backend

**Directory:** `services/ultracrew_server/`  
**Workspace root:** repo root (all `coralys-*` crates are path dependencies)

| Item | Value |
|------|-------|
| Language | Rust (edition 2024) |
| Build command | `cargo build --release --bin ultracrew_server` (from repo root) |
| Start command | `./target/release/ultracrew_server` |
| Port | `$PORT` (default 3001) |
| Health endpoint | `GET /api/health` → `{"status":"ok"}` |
| CORS | `CorsLayer::new().allow_origin(Any)` — permits all origins ✅ |

**Workspace dependencies (must build from repo root):**

```
coralys-moga
coralys-ecology
coralys-recommendation
coralys-policy
coralys-core
adapters/chronosentiment
infrastructure/optimization
adapters/ultracrew
```

Render must be configured with **root directory = `.`** (repo root), not `services/ultracrew_server/`.

**Environment variables required:**

| Variable | Purpose | Default |
|----------|---------|---------|
| `PORT` | Listening port | `3001` |
| `SUPABASE_URL` | Supabase project URL (after migration) | — |
| `SUPABASE_ANON_KEY` | Supabase anon key (after migration) | — |

---

## 3. Runtime Data

| Asset | Location | Notes |
|-------|----------|-------|
| SunAir scenario | Embedded in `App.js` (`buildSunairScenario()`) | No external file needed |
| GERAD fixture | Embedded in `src/geradInstance1.js` | No external file needed |
| GERAD benchmark | `benchmarks/gerad-g2014-22/` | Gated — not needed for demo |
| Pilot session records | `pilot_sessions/*.json` | **Ephemeral on Render — must migrate** |

No images, fonts, or logos are currently used. The UI is entirely CSS-in-JS.

---

## 4. Persistent Storage

**Current behaviour:** `pilot_session_handler` writes to `std::path::Path::new("pilot_sessions")` — a relative path on the server's local filesystem.

**Problem:** Render's free tier uses ephemeral filesystems. Files written to disk are lost on every deploy or restart.

**Recommended migration: Supabase PostgreSQL**

Create a `pilot_sessions` table:

```sql
create table pilot_sessions (
  id text primary key,
  timestamp timestamptz not null default now(),
  dispatcher_id text,
  dispatcher_role text,
  scenario_id text,
  coverage_pct float,
  hard_violations int,
  rest_violations int,
  fitness float,
  runtime_secs float,
  disruption_recovery_secs float,
  manual_edits int,
  recommendations_presented int,
  recommendations_accepted int,
  recommendations_rejected int,
  recommendations_modified int,
  explanation_usefulness int,
  overall_satisfaction int,
  adoption_signal text,
  dispatcher_comments text,
  org_name text,
  baseline_scheduling_mins float,
  baseline_disruption_mins float,
  product_gaps text,
  next_steps text,
  willing_to_pilot text,
  override_rate text,
  avg_explanation_rating float,
  recommendation_decisions jsonb,
  manual_edit_reasons jsonb,
  session_complete boolean
);
```

Replace the `save_item` / filesystem write in `pilot_session_handler` with a Supabase REST insert using the `reqwest` crate and `SUPABASE_URL` / `SUPABASE_ANON_KEY` environment variables.

**Interim option (zero code change):** Keep filesystem writes but add a Render persistent disk ($7/month). Not recommended for long term.

---

## 5. Environment Variables

**`apps/ultracrew-pilot-portal/.env.example`:**
```
# Backend API base URL (no trailing slash)
# Development: leave empty (CRA proxy handles it)
# Production: set to your Render backend URL
REACT_APP_API_URL=https://ultracrew-api.onrender.com
```

**`services/ultracrew_server/.env.example`:**
```
# Server port
PORT=3001

# Supabase (required after storage migration)
SUPABASE_URL=https://your-project.supabase.co
SUPABASE_ANON_KEY=your-anon-key
```

---

## 6. Deployment Target Compatibility

### Vercel (frontend)

| Check | Status |
|-------|--------|
| React / CRA support | ✅ Native |
| Build command | `npm run build` ✅ |
| Output dir | `build/` ✅ |
| HTTPS | ✅ Automatic |
| Custom domain | ✅ Supported |
| SPA routing | ✅ No router used — no rewrite needed |
| Environment variables | ✅ Set in Vercel dashboard |
| `REACT_APP_API_URL` support | ✅ CRA reads `REACT_APP_*` at build time |
| **Blocking: hardcoded proxy** | ❌ Must fix before deploy |

### Render (backend)

| Check | Status |
|-------|--------|
| Rust support | ✅ Native |
| Build from repo root | ✅ Set root dir to `.` |
| Build command | `cargo build --release --bin ultracrew_server` ✅ |
| Start command | `./target/release/ultracrew_server` ✅ |
| PORT env var | ✅ Render injects `$PORT` automatically |
| HTTPS | ✅ Automatic |
| CORS `Any` | ✅ Works for demo |
| Ephemeral filesystem | ⚠ Pilot sessions lost on restart — migrate to Supabase |
| Free tier build time | ⚠ First Rust build ~10–15 min (large workspace) |
| **Blocking: filesystem writes** | ❌ Must migrate or add persistent disk |

### Railway (alternative)

| Check | Status |
|-------|--------|
| Rust support | ✅ |
| Monorepo support | ✅ |
| Persistent volumes | ✅ Available |
| Free allowance | ⚠ Changes periodically — verify before use |

---

## 7. Startup Commands

### Frontend (Vercel)

```
Build command:   npm run build
Output dir:      build
Install command: npm install
Node version:    18
Root directory:  apps/ultracrew-pilot-portal
```

### Backend (Render)

```
Build command:   cargo build --release --bin ultracrew_server
Start command:   ./target/release/ultracrew_server
Root directory:  . (repo root)
Environment:     PORT (auto-injected), SUPABASE_URL, SUPABASE_ANON_KEY
```

---

## 8. Missing Items (Blocking)

### 8.1 Frontend API URL — BLOCKING

All `fetch()` calls use relative paths. In production, Vercel serves the React app from a different domain than the Render backend, so relative paths fail.

**Fix:** Add to `apps/ultracrew-pilot-portal/src/App.js` (top of file):

```js
const API_BASE = process.env.REACT_APP_API_URL || '';
```

Replace every `fetch('/api/...')` with `fetch(\`${API_BASE}/api/...\`)`.

Affected lines: `fetchCsrfToken` (line 53), `runOptimizer` (line 235), `submitPilotSession` (line 248), pairings/duties calls (lines 351–352).

### 8.2 Supabase storage migration — BLOCKING

`pilot_session_handler` writes to local disk. Must be replaced with a Supabase insert before deploying to Render.

### 8.3 `vercel.json` — RECOMMENDED

Create `apps/ultracrew-pilot-portal/vercel.json`:

```json
{
  "buildCommand": "npm run build",
  "outputDirectory": "build",
  "framework": "create-react-app"
}
```

### 8.4 `render.yaml` — RECOMMENDED

Create at repo root:

```yaml
services:
  - type: web
    name: ultracrew-api
    runtime: rust
    buildCommand: cargo build --release --bin ultracrew_server
    startCommand: ./target/release/ultracrew_server
    envVars:
      - key: PORT
        value: 10000
      - key: SUPABASE_URL
        fromDatabase:
          name: ultracrew-db
          property: connectionString
      - key: SUPABASE_ANON_KEY
        sync: false
```

### 8.5 `.env.example` files — RECOMMENDED

Create `apps/ultracrew-pilot-portal/.env.example` and `services/ultracrew_server/.env.example` (content in Section 5).

---

## 9. Deployment Checklist

### Phase 1 — Code changes (before any deploy)

- [ ] Add `API_BASE` constant to `App.js` reading `REACT_APP_API_URL`
- [ ] Replace all relative `fetch('/api/...')` calls with `fetch(\`${API_BASE}/api/...\`)`
- [ ] Add Supabase client (`reqwest` + `serde_json`) to backend `Cargo.toml`
- [ ] Replace `pilot_session_handler` filesystem write with Supabase REST insert
- [ ] Replace `list_pilot_sessions_handler` filesystem read with Supabase REST select
- [ ] Create `apps/ultracrew-pilot-portal/.env.example`
- [ ] Create `services/ultracrew_server/.env.example`
- [ ] Create `apps/ultracrew-pilot-portal/vercel.json`
- [ ] Create `render.yaml` at repo root
- [ ] Commit all changes

### Phase 2 — Supabase setup

- [ ] Create Supabase project at https://supabase.com
- [ ] Run `pilot_sessions` table DDL (Section 4)
- [ ] Copy `SUPABASE_URL` and `SUPABASE_ANON_KEY` from Supabase dashboard

### Phase 3 — Render backend deploy

- [ ] Connect GitHub repo to Render
- [ ] Set root directory to `.` (repo root)
- [ ] Set build command: `cargo build --release --bin ultracrew_server`
- [ ] Set start command: `./target/release/ultracrew_server`
- [ ] Add env vars: `SUPABASE_URL`, `SUPABASE_ANON_KEY`
- [ ] Deploy and verify `GET https://ultracrew-api.onrender.com/api/health` returns `{"status":"ok"}`
- [ ] Note the Render URL

### Phase 4 — Vercel frontend deploy

- [ ] Connect GitHub repo to Vercel
- [ ] Set root directory to `apps/ultracrew-pilot-portal`
- [ ] Set build command: `npm run build`
- [ ] Set output directory: `build`
- [ ] Add env var: `REACT_APP_API_URL=https://ultracrew-api.onrender.com`
- [ ] Deploy and verify the portal loads at `https://ultracrew-pilot-portal.vercel.app`
- [ ] Run a complete demo session end-to-end
- [ ] Verify pilot session record appears in Supabase dashboard

### Phase 5 — Custom domain (optional)

- [ ] Purchase `ultracrew.ai` or use `demo.ultracrew.ai` subdomain
- [ ] Configure DNS in Vercel
- [ ] Update `REACT_APP_API_URL` if backend domain also changes

---

## 10. Estimated Effort

| Item | Effort |
|------|--------|
| `API_BASE` frontend fix | 30 min |
| Supabase table creation | 15 min |
| Backend Supabase integration | 2–3 hours |
| Vercel deploy | 30 min |
| Render deploy (first build ~15 min) | 45 min |
| End-to-end verification | 30 min |
| **Total** | **~5 hours** |

The largest single item is the Supabase backend integration. Everything else is configuration.