# HDV-001-B Integrity Verification Report

**Generated:** 2026-08-17
**Cache directory:** `datasets/hdv001/hdv001_price_cache_v1/`
**Required window:** 2026-07-14 → 2026-08-13
**Fetch window:** 2026-07-01 → 2026-08-17

## Summary

| Check | Result |
|-------|--------|
| CHECK-1: Universe match | ✓ PASS |
| CHECK-2: Duplicate sessions | ✓ PASS |
| CHECK-3: NaN / null OHLCV | ✓ PASS |
| CHECK-4: NSE holiday calendar | ✓ PASS |
| CHECK-5: Corporate-action adj | ✓ PASS |

**Overall: ✓ ALL CHECKS PASSED**

## CHECK-1: Universe match

```
Expected instruments : 52
Present in cache     : 52
RESULT: PASS — universe matches exactly 52 instruments
```

## CHECK-2: Duplicate sessions

```
Checked 52 instruments — 0 duplicate sessions found
RESULT: PASS
```

## CHECK-3: NaN / null OHLCV

```
Checked 52 instruments — 0 NaN/null OHLCV values
RESULT: PASS
```

## CHECK-4: NSE holiday calendar

```
Expected trading days in required window: 23
Known NSE holidays in window: [datetime.date(2026, 8, 15)]

Reference instrument: TCS.NS
  Bars in required window : 23
  Missing weekday sessions: []
  Unexplained absences    : []

Session counts in required window across 52 instruments:
  Unique counts: [23]

Total bars per instrument (full fetch window): [34]

RESULT: PASS — all missing weekday sessions are known NSE holidays
```

## CHECK-5: Corporate-action adj

```

  TCS.NS — TCS ₹12 interim dividend
    Ex-date       : 2026-07-15
    Expected div  : ₹12.00
    Recorded div  : ₹12.00
    Div match     : PASS
    Pre-ex close  : 2188.6001  (2026-07-14)
    Ex-date close : 2189.2000
    Post-ex close : 2201.0000  (2026-07-16)

  HCLTECH.NS — HCLTECH ₹12 dividend
    Ex-date       : 2026-07-17
    Expected div  : ₹12.00
    Recorded div  : ₹12.00
    Div match     : PASS
    Pre-ex close  : 1175.4000  (2026-07-16)
    Ex-date close : 1203.9000
    Post-ex close : 1221.3000  (2026-07-20)

  WIPRO.NS — WIPRO ₹2 dividend
    Ex-date       : 2026-07-27
    Expected div  : ₹2.00
    Recorded div  : ₹2.00
    Div match     : PASS
    Pre-ex close  : 175.1200  (2026-07-24)
    Ex-date close : 178.5300
    Post-ex close : 181.1200  (2026-07-28)

  ULTRACEMCO.NS — ULTRACEMCO ₹240 dividend
    Ex-date       : 2026-07-30
    Expected div  : ₹240.00
    Recorded div  : ₹240.00
    Div match     : PASS
    Pre-ex close  : 11758.0000  (2026-07-29)
    Ex-date close : 11847.0000
    Post-ex close : 11903.0000  (2026-07-31)

  MARUTI.NS — MARUTI ₹140 dividend
    Ex-date       : 2026-08-07
    Expected div  : ₹140.00
    Recorded div  : ₹140.00
    Div match     : PASS
    Pre-ex close  : 13940.0000  (2026-08-06)
    Ex-date close : 14037.0000
    Post-ex close : 14097.0000  (2026-08-10)

Spot-check summary: 5/5 passed
RESULT: PASS — all 5 corporate action events verified
```
