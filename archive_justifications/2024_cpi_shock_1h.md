# Archive Ingestion Justification: 2024 CPI Shock (Wave 1)

**Universe Target:** `2024_cpi_shock_1h`
**Date:** 2026-05-23
**Time Bounds:** 2024-02-13 13:00:00 UTC to 2024-02-13 14:00:00 UTC (1707829200000 to 1707832800000)

## 1. What pressure class?
`macro_shock`

## 2. What recurrence axis?
Recurrence of localized vertical rupture penetration through smoothing kernels.

## 3. What existing assumption does it pressure?
Pressures the finding from `2026_intraday_impulse_shock_0730_0800_utc` that vertical velocity can penetrate `rolling_50` immunity. We need to verify if deterministic macroeconomic shocks produce the same continuity penetration geometry as organic market shocks.

## 4. What makes it phenomenologically distinct?
Unlike organic cascades or localized organic impulses, a CPI shock is deterministic, meaning market-maker liquidity withdrawal and repositioning occurs specifically around a known absolute timestamp. This tests impulse compression recurrence under coordinated rather than chaotic conditions.
