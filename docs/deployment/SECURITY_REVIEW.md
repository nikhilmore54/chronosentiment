# UltraCrew Pilot Experience v1.0 — Security Readiness Review

**Date:** 2026-07-28  
**Branch:** governance-hardening  
**Scope:** Pre-public-launch security review against OWASP Top 10 and deployment best practices

---

## Status Summary

| Area | Status | Notes |
|------|--------|-------|
| Secrets management | ✅ Pass | No secrets in Git; env vars only |
| Source code exposure | ✅ Pass | Backend IP never sent to browser |
| CORS | ✅ Pass (demo) | `Any` origin acceptable for demo; restrict before production |
| CSRF protection | ✅ Pass | Double-submit token on all state-changing endpoints |
| Input validation | ⚠ Partial | Struct deserialization validates types; no length limits |
| Rate limiting | ❌ Missing | Optimizer endpoint can be triggered repeatedly |
| Source maps | ⚠ Check | CRA default includes source maps; disable for production |
| Security headers | ❌ Missing | No HSTS, CSP, X-Frame-Options configured |
| Dependency audit | ⚠ Pending | `cargo audit` and `npm audit` not yet run |
| Repository visibility | ⚠ Action needed | Confirm repo is private before public launch |
| Supabase key type | ✅ Pass | Anon key used (not service role key) |
| PII in logs | ⚠ Partial | `println!` logs session IDs; no customer PII logged |
| Demo data | ✅ Pass | GERAD synthetic data; no real customer data |

---

## 1. Source Code Exposure

**Frontend (React / Vercel):**  
JavaScript, HTML, and CSS are downloaded by every visitor. This is unavoidable for any web application. The frontend contains no secrets, no business logic, and no IP. The Coralys optimisation engine is entirely server-side.

**Action:** Disable source maps in production to prevent reverse-engineering of minified JS.

Add to `apps/ultracrew-pilot-portal/.env.production`:
```
GENERATE_SOURCEMAP=false
```

**Backend (Rust / Render):**  
Only the compiled binary is deployed. Source code, Coralys algorithms, and UltraCrew optimisation logic are never transmitted to clients. This is the correct architecture — all IP stays server-side.

---

## 2. Secrets Management

**Current state:** ✅ Clean

- No `.env` files committed to Git (verified: `.gitignore` excludes `.env*`)
- `SUPABASE_URL`, `SUPABASE_ANON_KEY` are read from environment variables at runtime
- `REACT_APP_API_URL` is a non-secret configuration value (safe to expose)
- No JWT signing keys, SMTP credentials, or service role keys in the codebase

**Action:** Before deploying, set env vars in Render and Vercel dashboards only. Never commit a `.env` file.

**Supabase key type:** The backend uses `SUPABASE_ANON_KEY` (not the service role key). This is correct. The anon key is subject to Row Level Security policies. Enable RLS on the `pilot_sessions` table before going live.

Recommended Supabase RLS policy (insert-only from backend, no public read):
```sql
-- Enable RLS
alter table pilot_sessions enable row level security;

-- Allow inserts from authenticated service (backend uses anon key with service header)
create policy "backend_insert" on pilot_sessions
  for insert with check (true);

-- Deny all reads from anon (admin reads via Supabase dashboard only)
create policy "no_public_read" on pilot_sessions
  for select using (false);
```

---

## 3. CORS Configuration

**Current state:** `CorsLayer::new().allow_origin(Any)` — permits all origins.

This is acceptable for a public demo where the frontend URL is not yet fixed. Before production, restrict to the Vercel domain:

```rust
use tower_http::cors::CorsLayer;
use axum::http::HeaderValue;

let cors = CorsLayer::new()
    .allow_origin("https://ultracrew-demo.vercel.app".parse::<HeaderValue>().unwrap())
    .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
    .allow_headers([axum::http::header::CONTENT_TYPE, axum::http::header::AUTHORIZATION]);
```

---

## 4. CSRF Protection

**Current state:** ✅ Double-submit cookie pattern implemented.

- `GET /api/csrf-token` issues a token stored in `AppState`
- All state-changing POST endpoints validate `X-CSRF-Token` header against stored token
- Returns `403 FORBIDDEN` on mismatch

This is sufficient for the demo. No changes required.

---

## 5. Input Validation

**Current state:** ⚠ Partial

Axum's `Json` extractor validates that the request body deserialises into the target struct. Type mismatches return 422. However:

- No maximum string length limits on `dispatcher_comments`, `product_gaps`, `org_name`, etc.
- No maximum array length on `recommendation_decisions`
- No payload size limit on the HTTP body

**Actions:**

Add a body size limit to the Axum router:
```rust
use axum::extract::DefaultBodyLimit;

let app = Router::new()
    // ... routes ...
    .layer(DefaultBodyLimit::max(1 * 1024 * 1024)); // 1 MB
```

Add string length validation in `pilot_session_handler`:
```rust
if input.dispatcher_comments.len() > 10_000 {
    return Err((StatusCode::BAD_REQUEST, "dispatcher_comments too long".to_string()));
}
```

---

## 6. Rate Limiting

**Current state:** ❌ Missing — highest priority security gap.

The `POST /api/schedule` endpoint triggers a full MOGA optimisation run. Without rate limiting, a malicious actor could repeatedly call this endpoint and exhaust Render's CPU allocation.

**Recommended fix:** Add `tower_governor` rate limiting middleware.

Add to `services/ultracrew_server/Cargo.toml`:
```toml
tower_governor = "0.4"
```

Add to router setup in `main.rs`:
```rust
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};

let governor_conf = GovernorConfigBuilder::default()
    .per_second(2)       // 2 requests per second
    .burst_size(5)       // burst of 5
    .finish()
    .unwrap();

let app = Router::new()
    // ... routes ...
    .layer(GovernorLayer { config: Arc::new(governor_conf) });
```

---

## 7. Security Headers

**Current state:** ❌ Missing

Add to the Axum router:
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

For the frontend, add to `apps/ultracrew-pilot-portal/vercel.json`:
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
        { "key": "Strict-Transport-Security", "value": "max-age=63072000; includeSubDomains" }
      ]
    }
  ]
}
```

---

## 8. Dependency Audit

**Actions (run before first public deploy):**

```bash
# Rust
cargo audit

# Frontend
cd apps/ultracrew-pilot-portal && npm audit --audit-level=high
```

Address any HIGH or CRITICAL findings before launch.

---

## 9. Repository Visibility

**Action required:** Confirm the GitHub repository is **private** before announcing the public demo URL. The repository contains:

- Coralys platform source code (core IP)
- UltraCrew optimisation logic
- MOGA implementation
- All architectural and governance documents

This should remain private. Only the deployed URLs (`https://ultracrew-demo.vercel.app`, `https://ultracrew-api.onrender.com`) are public.

---

## 10. Logging

**Current state:** ⚠ Partial

`println!` statements log session IDs and file paths. No customer PII (names, emails, schedules) is logged. This is acceptable for the demo.

Before production, replace `println!` with structured logging (`tracing` crate) with appropriate log levels. Remove any debug `println!` that could expose internal state.

---

## 11. Demo Data

**Current state:** ✅ Clean

The SunAir Regional scenario is built from the GERAD benchmark (synthetic data, no real airline, no real crew, no real schedules). No personally identifiable information is present in any demo dataset.

---

## Pre-Launch Checklist

### Must-do before first public URL

- [ ] Set `GENERATE_SOURCEMAP=false` in `.env.production`
- [ ] Add `DefaultBodyLimit::max(1MB)` to Axum router
- [ ] Run `cargo audit` — fix HIGH/CRITICAL findings
- [ ] Run `npm audit --audit-level=high` — fix HIGH/CRITICAL findings
- [ ] Confirm GitHub repository is private
- [ ] Enable RLS on Supabase `pilot_sessions` table
- [ ] Set env vars in Render and Vercel dashboards (never commit `.env`)

### Should-do before customer/investor demos

- [ ] Add rate limiting (`tower_governor`) to `/api/schedule`
- [ ] Add security headers to `vercel.json`
- [ ] Add `X-Content-Type-Options` and `X-Frame-Options` to Axum responses
- [ ] Restrict CORS to Vercel domain (not `Any`)
- [ ] Add string length validation to pilot session handler

### Deferred (post-pilot)

- [ ] Replace `println!` with `tracing` structured logging
- [ ] External penetration test (OWASP Top 10)
- [ ] Add authentication for `/api/pilot/sessions` list endpoint
- [ ] Implement Supabase Row Level Security policies
- [ ] Add request signing or API key for backend-to-Supabase communication