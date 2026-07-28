# UltraCrew Pilot Experience v1.0 — Security Readiness Review

**Date:** 2026-07-28  
**Branch:** governance-hardening  
**Scope:** Pre-public-launch security review against OWASP Top 10 and deployment best practices  
**Rating:** 8.5–9/10 for public demonstration (10 must-do items before go-live)

---

## Status Summary

| Area | Status | Notes |
|------|--------|-------|
| Secrets management | ✅ Pass | No secrets in Git; env vars only |
| Source code exposure | ✅ Pass | Backend IP never sent to browser |
| CORS | ✅ Pass (demo) | `Any` origin acceptable for demo; restrict before production |
| CSRF protection | ✅ Pass | Double-submit token on all state-changing endpoints |
| Input validation | ⚠ Partial | Type validation only; no length/size limits on optimizer inputs |
| Rate limiting | ❌ Must-do | `/api/schedule` unprotected — demo availability risk |
| Source maps | ❌ Must-do | CRA default includes source maps; disable for production |
| Body size limit | ❌ Must-do | No HTTP body size cap; add `DefaultBodyLimit::max(1MB)` |
| Security headers | ⚠ Should-do | HSTS, X-Frame-Options, nosniff present; CSP missing |
| Dependency audit | ❌ Must-do | `cargo audit` and `npm audit` not yet run |
| Repository visibility | ❌ Must-do | Confirm private + check history for leaked secrets |
| Supabase key type | ⚠ Trade-off | Anon key used (acceptable for demo with RLS); use service role key in production |
| PII in logs | ⚠ Partial | `println!` logs session IDs; no customer PII logged |
| Demo data | ✅ Pass | GERAD synthetic data; no real customer data |
| Optimization timeouts | ❌ Should-do | No timeout on optimizer; should return 503 after ~60s |
| Panic handling | ⚠ Should-do | Unhandled panics may expose internal details |
| Demo mode | ⚠ Should-do | No `DEMO_MODE` env var to disable destructive ops |

---

## 1. Source Code Exposure

**Frontend (React / Vercel):**  
JavaScript, HTML, and CSS are downloaded by every visitor. This is unavoidable for any web application. The frontend contains no secrets, no business logic, and no IP. The Coralys optimisation engine is entirely server-side.

**Action (must-do):** Disable source maps in production.

Create `apps/ultracrew-pilot-portal/.env.production`:
```
GENERATE_SOURCEMAP=false
```

**Backend (Rust / Render):**  
Only the compiled binary is deployed. Source code, Coralys algorithms, and UltraCrew optimisation logic are never transmitted to clients. All IP stays server-side.

---

## 2. Secrets Management

**Current state:** ✅ Clean

- No `.env` files committed to Git
- `SUPABASE_URL`, `SUPABASE_ANON_KEY` read from environment variables at runtime
- `REACT_APP_API_URL` is a non-secret configuration value (safe to expose)
- No JWT signing keys, SMTP credentials, or service role keys in the codebase

**Action (must-do):** Set env vars in Render and Vercel dashboards only. Never commit a `.env` file.

**Action (must-do):** Check repository history for any previously committed secrets:
```bash
git log --all --full-history -- "*.env"
git log --all -p | grep -i "supabase\|api_key\|secret\|password" | head -50
```
If any secrets appear in history, rotate those credentials immediately even if the repository is now private.

---

## 3. Supabase Key Type — Conscious Trade-off

**Current state:** Backend uses `SUPABASE_ANON_KEY`.

This is acceptable for the demo if RLS policies are correctly configured. The anon key constrains the backend to the same RLS rules as any anonymous browser client.

**For production**, the correct pattern is:

```
Browser → Rust API → Service Role Key → Supabase
```

The service role key bypasses RLS and gives the backend full administrative access. It must never reach the browser — but using it inside the Rust backend is the standard server-side pattern.

**Recommended Supabase RLS policy (demo — insert-only, no public read):**
```sql
alter table pilot_sessions enable row level security;

-- Allow inserts (backend uses anon key)
create policy "backend_insert" on pilot_sessions
  for insert with check (true);

-- Deny all reads from anon role (admin reads via Supabase dashboard only)
create policy "no_public_read" on pilot_sessions
  for select using (false);
```

**Migration path:** When moving to production, add `SUPABASE_SERVICE_ROLE_KEY` as a Render env var and switch the backend to use it. Never expose the service role key to the browser.

---

## 4. CORS Configuration

**Current state:** `CorsLayer::new().allow_origin(Any)` — permits all origins.

Acceptable for demo. Before production, restrict to the Vercel domain:

```rust
use tower_http::cors::CorsLayer;
use axum::http::HeaderValue;

let cors = CorsLayer::new()
    .allow_origin("https://ultracrew-demo.vercel.app".parse::<HeaderValue>().unwrap())
    .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
    .allow_headers([axum::http::header::CONTENT_TYPE, axum::http::header::AUTHORIZATION]);
```

---

## 5. CSRF Protection

**Current state:** ✅ Double-submit cookie pattern implemented.

- `GET /api/csrf-token` issues a token stored in `AppState`
- All state-changing POST endpoints validate `X-CSRF-Token` header against stored token
- Returns `403 FORBIDDEN` on mismatch

No changes required.

---

## 6. Rate Limiting — Must-do

**Current state:** ❌ Missing. `/api/schedule` triggers a full MOGA optimisation run with no rate limit.

This is a **demo availability risk**, not just a production concern. A malicious actor (or a misconfigured client) can repeatedly call this endpoint and exhaust Render's CPU allocation, taking the demo offline during a customer presentation.

**Fix:** Add `tower_governor` rate limiting.

Add to `services/ultracrew_server/Cargo.toml`:
```toml
tower_governor = "0.4"
```

Add to router setup in `main.rs`:
```rust
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use std::sync::Arc;

let governor_conf = Arc::new(
    GovernorConfigBuilder::default()
        .per_second(2)    // 2 requests per second per IP
        .burst_size(5)    // burst of 5
        .finish()
        .unwrap()
);

let app = Router::new()
    // ... routes ...
    .layer(GovernorLayer { config: governor_conf });
```

---

## 7. Input Validation

**Current state:** ⚠ Partial — type validation only.

**Action (must-do):** Add HTTP body size limit:
```rust
use axum::extract::DefaultBodyLimit;

let app = Router::new()
    // ... routes ...
    .layer(DefaultBodyLimit::max(1 * 1024 * 1024)); // 1 MB
```

**Action (should-do):** Add optimizer payload validation in the schedule handler:
```rust
const MAX_CREW: usize = 1000;
const MAX_LEGS: usize = 10_000;
const MAX_HORIZON_DAYS: u32 = 90;

if input.crew.len() > MAX_CREW {
    return Err((StatusCode::BAD_REQUEST, format!("crew count exceeds maximum ({})", MAX_CREW)));
}
if input.flight_legs.len() > MAX_LEGS {
    return Err((StatusCode::BAD_REQUEST, format!("leg count exceeds maximum ({})", MAX_LEGS)));
}
```

Add string length validation in `pilot_session_handler`:
```rust
if input.dispatcher_comments.len() > 10_000 {
    return Err((StatusCode::BAD_REQUEST, "dispatcher_comments too long".to_string()));
}
```

---

## 8. Optimization Timeouts

**Current state:** ❌ Missing. Long-running optimization requests are not bounded.

**Action (should-do):** Wrap the optimizer call in a `tokio::time::timeout`:
```rust
use tokio::time::{timeout, Duration};

let result = timeout(
    Duration::from_secs(60),
    run_optimizer(input)
).await;

match result {
    Ok(Ok(solution)) => Ok(Json(solution)),
    Ok(Err(e)) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    Err(_) => Err((StatusCode::SERVICE_UNAVAILABLE,
        "Optimization timed out after 60 seconds. Try a smaller scenario.".to_string())),
}
```

---

## 9. Security Headers

**Current state:** ❌ Missing.

**CSP (Content-Security-Policy)** is the most valuable missing header. Add to `apps/ultracrew-pilot-portal/vercel.json`:

```json
{
  "buildCommand": "npm run build",
  "outputDirectory": "build",
  "framework": "create-react-app",
  "headers": [
    {
      "source": "/(.*)",
      "headers": [
        { "key": "X-Content-Type-Options", "value": "nosniff" },
        { "key": "X-Frame-Options", "value": "DENY" },
        { "key": "Referrer-Policy", "value": "strict-origin-when-cross-origin" },
        { "key": "Strict-Transport-Security", "value": "max-age=63072000; includeSubDomains" },
        { "key": "Content-Security-Policy", "value": "default-src 'self'; script-src 'self'; connect-src 'self' https://ultracrew-api.onrender.com; img-src 'self' data:; style-src 'self' 'unsafe-inline'" }
      ]
    }
  ]
}
```

Adjust `connect-src` to match the actual Render backend URL once deployed.

Add to Axum router for backend responses:
```rust
use tower_http::set_header::SetResponseHeaderLayer;
use axum::http::{header, HeaderValue};

let app = Router::new()
    // ... routes ...
    .layer(SetResponseHeaderLayer::if_not_present(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    ))
    .layer(SetResponseHeaderLayer::if_not_present(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY"),
    ));
```

---

## 10. Panic Handling

**Current state:** ⚠ Unhandled panics in Axum handlers may expose internal details.

**Action (should-do):** Add a panic handler layer:
```rust
use tower_http::catch_panic::CatchPanicLayer;

let app = Router::new()
    // ... routes ...
    .layer(CatchPanicLayer::new());
```

This returns a generic `500 Internal Server Error` response instead of propagating the panic. Log the panic details server-side only.

---

## 11. Demo Mode Configuration

**Action (should-do):** Add a `DEMO_MODE` environment variable:

```rust
let demo_mode = std::env::var("DEMO_MODE")
    .map(|v| v == "true" || v == "1")
    .unwrap_or(false);
```

When `DEMO_MODE=true`:
- Disable any destructive operations (data deletion, bulk exports)
- Restrict file uploads if not needed for the demo
- Use only synthetic data (SunAir Regional / GERAD)
- Return generic error messages (no stack traces)
- Prevent accidental access to production integrations

Set `DEMO_MODE=true` in the Render environment for the public demo deployment.

---

## 12. Dependency Audit

**Actions (must-do before first public deploy):**

```bash
# Rust
cargo audit

# Frontend
cd apps/ultracrew-pilot-portal && npm audit --audit-level=high
```

Address any HIGH or CRITICAL findings before launch. Pin dependency versions — do not run `npm update` or `cargo update` immediately before a demo.

---

## 13. Repository Visibility

**Action (must-do):** Confirm the GitHub repository is **private** before announcing the public demo URL.

**Action (must-do):** Check history for leaked secrets (see Section 2). If any appear, rotate those credentials immediately.

The repository contains Coralys platform source code, UltraCrew optimisation logic, MOGA implementation, and all architectural/governance documents. This must remain private.

---

## 14. Logging

**Current state:** ⚠ `println!` statements log session IDs and file paths. No customer PII logged.

Acceptable for demo. Before production, replace `println!` with `tracing` structured logging at appropriate log levels.

---

## 15. Documentation Endpoints

If any `/docs`, `/swagger`, or `/openapi.json` endpoints exist, verify they are either disabled or intentionally public. The current Axum router does not expose these — no action required unless added in future.

---

## Pre-Launch Checklist

### Must-do before first public URL

- [ ] Set `GENERATE_SOURCEMAP=false` in `apps/ultracrew-pilot-portal/.env.production`
- [ ] Add `DefaultBodyLimit::max(1MB)` to Axum router
- [ ] Add `tower_governor` rate limiting to `/api/schedule` (2 req/s per IP, burst 5)
- [ ] Run `cargo audit` — fix HIGH/CRITICAL findings
- [ ] Run `npm audit --audit-level=high` — fix HIGH/CRITICAL findings
- [ ] Confirm GitHub repository is private
- [ ] Check git history for leaked secrets; rotate any found
- [ ] Enable RLS on Supabase `pilot_sessions` table (DDL in Section 3)
- [ ] Set env vars in Render and Vercel dashboards (never commit `.env`)
- [ ] Set `DEMO_MODE=true` in Render environment

### Should-do before customer/investor demos

- [ ] Add CSP header to `vercel.json` (template in Section 9)
- [ ] Add `X-Content-Type-Options` and `X-Frame-Options` to Axum responses
- [ ] Restrict CORS to Vercel domain (not `Any`)
- [ ] Add optimizer payload validation (crew ≤ 1000, legs ≤ 10,000, horizon ≤ 90 days)
- [ ] Add optimization timeout (60s → 503)
- [ ] Add `CatchPanicLayer` for generic 500 responses
- [ ] Add string length validation to pilot session handler
- [ ] Set `DEMO_MODE=true` in Render environment

### Deferred — after pilot feedback

- [ ] Replace `println!` with `tracing` structured logging
- [ ] External penetration test (OWASP Top 10)
- [ ] Authentication for `/api/pilot/sessions` list endpoint
- [ ] Switch backend to Supabase service role key (see Section 3)
---

## Milestone P0 — Release Decision Gate

This gate must be passed before announcing any public URL. It is not a checklist to work through gradually — it is a binary GO / NO-GO decision made at a single point in time.

### GO criteria (all must be true)

**Functional**
- [ ] Public frontend deployed and accessible over HTTPS
- [ ] Public backend deployed and accessible over HTTPS
- [ ] `GET /api/health` returns `{"status":"ok"}` from the public Render URL
- [ ] Supabase persistence verified: complete a demo session and confirm the record appears in the Supabase dashboard
- [ ] End-to-end workflow tested: SunAir scenario → optimizer → Step 6 evidence → submit → Supabase record

**Security**
- [ ] All 10 must-do items in the Pre-Launch Checklist above are complete
- [ ] `cargo audit` shows zero HIGH or CRITICAL findings
- [ ] `npm audit --audit-level=high` shows zero HIGH or CRITICAL findings
- [ ] Rate limiting confirmed active (verify 429 response after rapid repeated calls to `/api/schedule`)
- [ ] Repository confirmed private on GitHub

**Operational**
- [ ] Render service starts from a cold boot without manual intervention
- [ ] Vercel frontend connects to the correct Render backend (not localhost)
- [ ] Browser refresh on any step does not break the session
- [ ] Error pages are user-friendly (no stack traces visible)

### NO-GO conditions (any one blocks release)

- Any must-do security item incomplete
- `cargo audit` or `npm audit` contains HIGH or CRITICAL findings
- Rate limiting disabled or unverified
- Repository public
- Supabase persistence unverified
- End-to-end workflow untested on the public URLs

---

## Operational Readiness — Pre-Demo Verification

Run this checklist before every customer or investor demonstration. It takes approximately 10 minutes.

### Environment check

1. Open `https://ultracrew-demo.vercel.app` in a fresh browser tab (incognito/private mode)
2. Verify the portal loads without errors
3. Open browser DevTools → Console — confirm no red errors on load
4. Call `GET https://ultracrew-api.onrender.com/api/health` — confirm `{"status":"ok"}`

### Smoke test (full workflow)

5. Start a new demo session — select the SunAir scenario
6. Run the optimizer — confirm it completes within 30 seconds and returns a schedule
7. Step through Steps 1–5 — verify all UI elements render correctly
8. Complete Step 6 commercial evidence fields
9. Submit the session — confirm the success screen appears
10. Open the Supabase dashboard → `pilot_sessions` table — confirm the new record is present

### Recovery check

11. Refresh the browser mid-session — confirm the app recovers gracefully
12. Confirm the Render service is not in a sleeping state (free tier sleeps after 15 min inactivity — trigger a wake-up call to `/api/health` before the meeting)

---

## Disaster Recovery

If something goes wrong during a live demonstration, follow this procedure in order.

### Frontend failure (Vercel)

1. Open the Vercel dashboard → Deployments
2. Identify the last successful deployment
3. Click "Promote to Production" on that deployment
4. Wait 60–90 seconds for propagation
5. Verify `https://ultracrew-demo.vercel.app` loads correctly

### Backend failure (Render)

1. Open the Render dashboard → `ultracrew-api` service
2. Click "Manual Deploy" → "Deploy latest commit"
3. Wait for the build to complete (first build ~15 min; subsequent builds ~2–3 min)
4. Verify `GET /api/health` returns `{"status":"ok"}`
5. Run the smoke test (Steps 5–10 above)

### Environment variable loss

If env vars are lost (e.g. after a service recreation):

1. Render: set `PORT=10000`, `SUPABASE_URL`, `SUPABASE_ANON_KEY`, `DEMO_MODE=true`
2. Vercel: set `REACT_APP_API_URL=https://ultracrew-api.onrender.com`
3. Redeploy both services
4. Run the smoke test

### Supabase unavailable

If Supabase is unreachable, the backend automatically falls back to local disk writes (`pilot_sessions/*.json`). The demo continues to function. Session records will not appear in the Supabase dashboard until connectivity is restored, but the demo workflow is unaffected.

### During a live meeting — immediate fallback

If the public URL fails during a meeting:

1. Switch to the local demo: run `./start-demo.sh` (macOS/Linux) or `start-demo.ps1` (Windows)
2. Share screen showing `http://localhost:3000`
3. The local demo is functionally identical to the public deployment
4. Apologise briefly, continue the demonstration — do not dwell on the technical issue

---

## Feature Freeze Declaration

**Pilot Experience v1.0 is feature frozen as of the date of Milestone P0 achievement.**

After P0, only the following changes are permitted without a formal review:

- Bug fixes that affect demo reliability
- Security patches (HIGH or CRITICAL findings from audits)
- Deployment and infrastructure changes
- Stability improvements

New capabilities (WOA Report Generator, Executive Dashboard, WDX Scorecard, Pilot Proposal Generator) are deferred until at least three customer demonstrations have been completed and their feedback has been reviewed. This protects the integrity of evidence gathering and ensures future development is driven by observed customer needs rather than assumptions.
- [ ] Production-grade monitoring and alerting
- [ ] Advanced API signing or mTLS between backend and Supabase