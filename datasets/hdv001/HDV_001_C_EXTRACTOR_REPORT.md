# HDV-001-C Price Path Extractor Report

**Generated:** 2026-08-17
**Source dataset:** `datasets/stop_research_dataset_v01.json`
**Price cache:** `datasets/hdv001/hdv001_price_cache_v1/`
**Output:** `datasets/hdv001/hdv001_price_paths_v1.json`
**Observation horizon:** 10 NSE sessions

## Temporal Rule

First eligible bar satisfies: `bar_date > decision_date_in_IST`

No bar from the decision date itself enters the path.
decision_time (UTC) is converted to IST (+05:30) to determine the decision date.

## Statistics

| Metric | Value |
|--------|-------|
| Total decisions | 1144 |
| COMPLETE (>= 10 sessions) | 728 |
| MATURING (< 10 sessions) | 416 |
| NO_CACHE | 0 |
| Zero sessions | 0 |

## Notes

MATURING decisions are those whose 10-session observation window has not
yet completed as of the cache build date (2026-08-17). These will become
COMPLETE as future sessions are added to the cache.

All 1,144 decisions receive a path record regardless of whether Config B
or C actually realized the trade. HDV-001 evaluates the Coralys decision,
not the historical portfolio execution mechanics.