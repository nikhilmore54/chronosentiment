V-011 Documentation Alignment Ledger Entry
---
**Type:** Documentation alignment
**Class:** Non-authoritative correction
**Replay impact:** None
**Authority impact:** None
**Purpose:** Align replay‑evidence descriptions with post‑ratification evidence posture.

**What changed**
- Updated the “Replay Evidence / Verification Tooling” table in `AUTHORITY_MAP.md` to reflect:
  - `fixtures/strategy_identity/differential_report.json` – Primary retained V‑001 evidence artifact.
  - `fixtures/strategy_identity/archive/*` – Historical lineage evidence.
  - `fixtures/strategy_identity/*.jsonl` – Optional historical parser‑evidence corpus (no longer required).

**What did not change**
- No JSONL fixtures were restored.
- No verifier behavior was modified.
- No replay semantics or authority surfaces were altered.
- No new governance phase was introduced.
- No constitutional concepts were added.

**Governance rationale**
- This is a documentation‑alignment event, not a semantic‑authority event.
- It reduces drift by ensuring the authority map matches the operational reality established by V‑001 ratification.

**Next steps**
- The Phase 5 draft remains pending until a formal transition document defines entry, success, and exit criteria.
---
