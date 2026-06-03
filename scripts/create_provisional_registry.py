"""Generate provisional candidate registry for Q1.
Select eight seed sessions (date, symbol) and extract their metrics from the session catalog.
Output: candidate_event_registry_q1_provisional.json
"""

import json
from pathlib import Path

CATALOG_PATH = Path("phase1/analysis/coordinate_audit/session_catalog_q1.json")
OUT_PATH = Path("candidate_event_registry_q1_provisional.json")

# Seed sessions as (date, symbol) pairs
SEED = [
    ("2025-01-21", "NIFTY"),
    ("2025-01-21", "BANKNIFTY"),
    ("2025-03-05", "NIFTY"),
    ("2025-03-24", "BANKNIFTY"),
    ("2025-01-28", "BANKNIFTY"),
    ("2025-02-03", "BANKNIFTY"),
    ("2025-02-27", "NIFTY"),
    ("2025-01-06", "BANKNIFTY"),
]

catalog = json.loads(CATALOG_PATH.read_text())
# Build lookup dictionary keyed by (date, symbol)
lookup = {(entry["date"], entry["symbol"]): entry for entry in catalog}

provisional = []
for key in SEED:
    entry = lookup.get(key)
    if entry:
        provisional.append(entry)
    else:
        print(f"Warning: session {key} not found in catalog")

OUT_PATH.write_text(json.dumps(provisional, indent=2))
print(f"Provisional registry written to {OUT_PATH}")
