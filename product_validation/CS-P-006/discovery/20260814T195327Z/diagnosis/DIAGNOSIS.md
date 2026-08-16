# CS-P-006-C Search #1 — post-search diagnosis

Diagnosis of the **already sealed** Search #1 artifact. Coralys was not re-run. Evaluation figures are holdout diagnosis, not search feedback.

- artifact_hash: `9a887827e8f41988987208f13e4ccbac507b3241692026c55f38d11f85971ac0`
- genome identity: `d8363a93e5afe518b7a4cbb8f5c3ac59efcf396f0d318ccdae0dd683e9d730d3`

## 1. Search-space utilization

The factory can sample Trend, Momentum, Volatility (presence), conjunctions, LONG, SHORT, and NO_TRADE. The **selected** artifact uses only Trend=Bearish → LONG; unmatched NO_TRADE.

| Capability | Factory can sample | Selected artifact uses |
|------------|--------------------|------------------------|
| Trend | true | true |
| Momentum | true | false |
| Volatility | true | false |
| Conjunctions | true | false |
| LONG | true | true |
| SHORT | true | false |
| NO_TRADE | true | true |

## 2. Population archive limitation

Search #1 recorded generation-best identity and fitness only. Median, worst, and population diversity were not persisted. The development-best genome rules were not serialized.

- generation bests recorded: 12
- unique generation-best genomes: 2
- candidates presented to selection: 2
- population median recorded: false
- population worst recorded: false
- population diversity recorded: false

## 3. Fitness trajectory (from Search #1 evidence)

See `search_evidence.json`. Recorded best jumped once (generation 2) and then stayed flat. Average rose from ~0.0018 toward ~0.012 and never reached the recorded best. Median/worst/diversity were not persisted, so early population collapse cannot be proven or disproven from the archive.

## 4–6. Sealed-policy decomposition

### development (search-visible)

- actions: LONG 49 / SHORT 0 / NO_TRADE 42
- mean signed traded return: 0.016325
- Trend occupancy: {"Bearish": 49, "Bullish": 42}
- bearish n=49 mean raw 20D Some(0.014038605742496534) (LONG payoff); SHORT payoff Some(-0.014038605742496534)
- other n=42 mean raw 20D Some(-0.003082083990946934)

| instrument | n | LONG | SHORT | NO_TRADE | mean signed traded | n_bearish | mean raw when bearish |
|------------|---|------|-------|----------|--------------------|-----------|-----------------------|
| HDFCBANK.NS | 13 | 7 | 0 | 6 | 0.001401 | 7 | Some(0.0014013540312884773) |
| ICICIBANK.NS | 13 | 5 | 0 | 8 | 0.024771 | 5 | Some(0.024770852362467635) |
| INFY.NS | 13 | 5 | 0 | 8 | 0.018261 | 5 | Some(0.018261163805460105) |
| RELIANCE.NS | 13 | 8 | 0 | 5 | 0.009300 | 8 | Some(0.009300390022776307) |
| TCS.NS | 13 | 10 | 0 | 3 | -0.001157 | 10 | Some(-0.0011567352597549314) |
| IDEA.NS | 13 | 9 | 0 | 4 | 0.022897 | 9 | Some(0.022897227673831975) |
| MAHABANK.NS | 13 | 5 | 0 | 8 | 0.038802 | 5 | Some(0.03880226113490463) |

### selection (search-visible)

- actions: LONG 39 / SHORT 0 / NO_TRADE 52
- mean signed traded return: 0.019938
- Trend occupancy: {"Bearish": 39, "Bullish": 52}
- bearish n=39 mean raw 20D Some(0.01568844964017212) (LONG payoff); SHORT payoff Some(-0.01568844964017212)
- other n=52 mean raw 20D Some(0.012065395220397013)

| instrument | n | LONG | SHORT | NO_TRADE | mean signed traded | n_bearish | mean raw when bearish |
|------------|---|------|-------|----------|--------------------|-----------|-----------------------|
| HDFCBANK.NS | 13 | 7 | 0 | 6 | 0.019642 | 7 | Some(0.019641916650481372) |
| ICICIBANK.NS | 13 | 7 | 0 | 6 | 0.010712 | 7 | Some(0.010712333168244107) |
| INFY.NS | 13 | 7 | 0 | 6 | 0.007676 | 7 | Some(0.007676300643305533) |
| RELIANCE.NS | 13 | 5 | 0 | 8 | 0.004786 | 5 | Some(0.0047860475451376614) |
| TCS.NS | 13 | 3 | 0 | 10 | 0.032206 | 3 | Some(0.03220584739876848) |
| IDEA.NS | 13 | 6 | 0 | 7 | -0.016543 | 6 | Some(-0.01654273674682241) |
| MAHABANK.NS | 13 | 4 | 0 | 9 | 0.081086 | 4 | Some(0.08108608082285906) |

### evaluation (holdout diagnosis only)

- actions: LONG 33 / SHORT 0 / NO_TRADE 58
- mean signed traded return: -0.000229
- Trend occupancy: {"Bearish": 33, "Bullish": 58}
- bearish n=33 mean raw 20D Some(-0.016064089302286366) (LONG payoff); SHORT payoff Some(0.016064089302286366)
- other n=58 mean raw 20D Some(-0.0017303870535318592)

| instrument | n | LONG | SHORT | NO_TRADE | mean signed traded | n_bearish | mean raw when bearish |
|------------|---|------|-------|----------|--------------------|-----------|-----------------------|
| HDFCBANK.NS | 13 | 4 | 0 | 9 | 0.033417 | 4 | Some(0.0334167702038247) |
| ICICIBANK.NS | 13 | 1 | 0 | 12 | 0.075415 | 1 | Some(0.07541493229465113) |
| INFY.NS | 13 | 5 | 0 | 8 | 0.026974 | 5 | Some(0.0269744161647746) |
| RELIANCE.NS | 13 | 6 | 0 | 7 | -0.025813 | 6 | Some(-0.02581332658148444) |
| TCS.NS | 13 | 5 | 0 | 8 | 0.029178 | 5 | Some(0.029177915196766435) |
| IDEA.NS | 13 | 7 | 0 | 6 | -0.080608 | 7 | Some(-0.080607659740059) |
| MAHABANK.NS | 13 | 5 | 0 | 8 | -0.060165 | 5 | Some(-0.060165007844757104) |

Do not retune the genome from these tables. Search #2 is not authorized by this diagnosis.
