 # GERAD G-2014-22 Benchmark Dataset

## Source

Kasirzadeh, M., Saddoune, M., & Soumis, F. (2014). *Airline crew scheduling: models,
algorithms, and data sets*. GERAD Technical Report G-2014-22. HEC Montréal.

Report URL: https://www.gerad.ca/en/papers/G-2014-22

## About This Directory

This directory contains the **real** G-2014-22 benchmark data, converted from the
original `G1422-DataSets.zip` distribution into the UltraCrew adapter CSV schema.

The original zip contains 7 instances in a proprietary per-day CSV format
(`day_N.csv`, `listOfBases.csv`, `crew_avail_const.csv`, `initialSolution.in`).
The converter script `convert_raw_to_csv.py` transforms each instance into the
5-file schema consumed by the GERAD adapter.

## Instance Summary (real data from G1422-DataSets.zip)

| Instance | Crew | Flights | Duties | Pairings | Swaps | Horizon |
|----------|------|---------|--------|----------|-------|---------|
| 1        | 33   | 1,013   | 385    | 172      | 86    | 31 days |
| 2        | 34   | 1,500   | 506    | 303      | 151   | 31 days |
| 3        | 47   | 1,855   | 584    | 274      | 136   | 31 days |
| 4        | 145  | 5,613   | 1,789  | 1,079    | 539   | 31 days |
| 5        | 247  | 5,743   | 2,970  | 1,497    | 748   | 31 days |
| 6        | 223  | 5,886   | 2,476  | 1,187    | 593   | 31 days |
| 7        | 305  | 7,766   | 3,584  | 1,648    | 823   | 31 days |

The top-level CSV files (`crew.csv`, `flights.csv`, etc.) are copies of **Instance 1**,
used as the default benchmark scenario in the portal.

Per-instance directories (`instance1/` through `instance7/`) contain the full converted
data for each instance.

## Dataset Structure

| File | Description |
|------|-------------|
| `crew.csv` | Crew members with base, qualifications, contract type |
| `flights.csv` | Flight legs with origin, destination, departure/arrival UTC, block minutes |
| `duties.csv` | Duty periods derived from the initial solution pairings |
| `pairings.csv` | Multi-day pairings from `initialSolution.in` |
| `swap_exchanges.csv` | Adjacent pairing swap pairs per base |

## Raw Data

The original GERAD format files are preserved in `raw/instance1/` through
`raw/instance7/`. Each contains:

- `day_N.csv` — flight legs for day N (1-31)
- `listOfBases.csv` — airport codes with base status and crew counts
- `crew_avail_const.csv` — crew available per base per day
- `initialSolution.in` — reference pairing solution
- `creditedHours` — credited hours and cost per schedule

## Conversion

To re-run the conversion (e.g. after updating the raw data):

```bash
cd benchmarks/gerad-g2014-22
python3 convert_raw_to_csv.py --all
# Then copy instance1 to top-level:
cp instance1/*.csv .
```

To convert a single instance:

```bash
python3 convert_raw_to_csv.py --instance raw/instance2/instance2 --out instance2
```

## Usage

```bash
# Run the benchmark via CLI (uses top-level CSVs = Instance 1)
cargo run --bin ultracrew-cli -- --benchmark

# Or load via the portal
# Navigate to Step 2 -> Select "GERAD Benchmark" scenario
```

## Relationship to the GERAD Fixture

| | GERAD Fixture | GERAD Benchmark |
|---|---|---|
| Source | `adapters/gerad/tests/fixtures/` | `G1422-DataSets.zip` (real data) |
| Purpose | Adapter unit/integration tests | Optimizer validation vs. published results |
| Scale | 8 crew, 10 legs, 5 duties | 33-305 crew, 1013-7766 flights (7 instances) |
| Available | Always | Converted from real zip |

## Compliance Note

The GERAD dataset is distributed for research purposes. Verify the licence terms on
the GERAD publication page before using it in any commercial context.

## Related Files

- [`adapters/gerad/`](../../adapters/gerad/) - Rust adapter crate (parser, mapper, validator, importer)
- [`adapters/gerad/tests/fixtures/`](../../adapters/gerad/tests/fixtures/) - Synthetic test fixture
- [`apps/ultracrew-pilot-portal/src/App.js`](../../apps/ultracrew-pilot-portal/src/App.js) - Portal scenario selector