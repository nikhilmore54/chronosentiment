# RP-608 — Research Reproducibility Safeguards

**Date:** 2026-11-07  
**Authority:** 0% · Gate 2 CLOSED  
**Motivation:** P4 audit found Mac outcomes stale (1014) vs frozen Pi report (1037).  

## Rule

Every research report **must** identify:

| Field | Requirement |
| --- | --- |
| Exact source dataset path | Absolute or repo-relative path used at generation |
| Source fingerprint | `sha256` + byte size + mtime UTC |
| Source row count | Raw + eligible |
| Script id / path | Script filename + protocol id string |
| Hypothesis / protocol version | e.g. `RP-608-H2-…` |
| Disc / val / conf boundaries | Explicit dates |
| Output artifacts | `.md` / `.json` paths |
| Host | `pi` vs `mac` — Pi preferred for crypto journals / Kite |

## Canonical hosts

| Data class | Canonical host |
| --- | --- |
| Crypto plan / OF journals | **Pi** `live_capture/...` |
| NSE Kite joins | **Pi** |
| Frozen reports in `artifacts/rp608_econ_edge_diag/` | Treat as sealed once Gate/memo filed; regenerate only on canonical host |

**Do not** silently overwrite a sealed Pi-backed report using a stale Mac dataset.

## Implementation aids

- H2 confirmation DQ monitor embeds source `sha256` on every run:  
  `scripts/research/rp608_h2_confirmation_dq_monitor.py`
- Dataset fingerprint helper:  
  `scripts/research/rp608_research_source_fingerprint.py`
- Registry of sealed reports:  
  `artifacts/rp608_econ_edge_diag/rp608_sealed_report_registry.json` (updated when sealing)

## Sealing checklist (manual)

Before calling a report “frozen”:

1. Run on canonical host.  
2. Record fingerprint in report JSON `integrity` / `source`.  
3. Append entry to sealed registry.  
4. Docs/INDEX link the sealed paths only.  

## Non-goals

- Not a new trading hypothesis  
- Not automatic Gate 2 decisions  
