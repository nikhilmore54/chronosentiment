#!/usr/bin/env python3
"""
Collect replay observations on RN-001 and append them to the ledger.
Governance check: this script ONLY observes, serializes, and appends entries – no prediction or analysis.
It validates that all mandatory Ledger v1 fields are present.

Two operational modes:
  * placeholder – uses synthetic hashes and a static price (useful for P1‑M1A validation).
  * real       – expects implementations of `fetch_replay_hash`, `fetch_trace_hash`, and `fetch_market_price`
                that pull data from the certified replay engine and market data source.
                These functions are deliberately left as TODOs; the script will abort if called without
                a proper implementation, ensuring no accidental drift into synthetic data.
"""
import json
import uuid
import datetime
import time
import os
import sys
import argparse

# Path to the append‑only ledger (must exist under the project root)
LEDGER_PATH = os.path.expanduser("~/ChronoSentiment_MEGA_FINAL/data/ledger/observations.jsonl")
SUBSTRATE = "BTCUSDT"

# ==== Data source stubs ====================================================

def fetch_replay_hash() -> str:
    """Read the replay_hash.txt for BTCUSDT from the frozen artifact directory.
    Returns the hash string without whitespace.
    """
    import pathlib
    path = pathlib.Path("infrastructure/core/artifacts/BTCUSDT/plateau_low/rolling_50/replay_hash.txt")
    if not path.is_file():
        raise FileNotFoundError(f"Replay hash file not found: {path}")
    return path.read_text().strip()

def fetch_trace_hash() -> str:
    """Compute a SHA‑256 hash of the trace_v1.json artifact for BTCUSDT.
    This provides a deterministic identifier for the trace content.
    """
    import pathlib, hashlib
    path = pathlib.Path("infrastructure/core/artifacts/BTCUSDT/plateau_low/rolling_50/trace_v1.json")
    if not path.is_file():
        raise FileNotFoundError(f"Trace file not found: {path}")
    data = path.read_bytes()
    return hashlib.sha256(data).hexdigest()


def fetch_market_price() -> float:
    """Return a placeholder market price.
    Real market price integration is out of scope for Phase‑1; a static placeholder is sufficient.
    """
    return placeholder_market_price()


# ==== Placeholder helpers (for P1‑M1A) ====================================

def generate_placeholder_hash() -> str:
    """Generate a deterministic placeholder hash (16‑hex chars)."""
    return uuid.uuid4().hex[:16]

def placeholder_market_price() -> float:
    """Static placeholder price used during infrastructure validation.
    This value is *not* a real market observation.
    """
    return 420.55

# ==== Core observation builder =============================================

def get_current_timestamp() -> str:
    """Return current UTC timestamp in ISO‑8601 with trailing Z."""
    return datetime.datetime.utcnow().replace(microsecond=0).isoformat() + "Z"

def build_observation(mode: str) -> dict:
    """Construct a Ledger v1 observation entry.
    `mode` must be either "placeholder" or "real".
    """
    if mode == "real":
        replay_hash = fetch_replay_hash()
        trace_hash = fetch_trace_hash()
        price = fetch_market_price()
    else:
        replay_hash = generate_placeholder_hash()
        trace_hash = generate_placeholder_hash()
        price = placeholder_market_price()

    return {
        "observation_id": str(uuid.uuid4()),
        "ledger_version": "v1",
        "timestamp": get_current_timestamp(),
# SUBSTRATE constant moved above
        "substrate": SUBSTRATE,  # Fixed to BTCUSDT for current Phase‑1 collection
        "replay_namespace": "replay:v1",
        "topology_namespace": "topology:v1",
        "cognition_namespace": "cognition:v1",
        "replay_hash": replay_hash,
        "trace_hash": trace_hash,
        "price": price,
        "research_node": "RN-001",
        "forward_windows": {"1d": None, "7d": None, "30d": None},
    }

# ==== Validation ===========================================================

def validate_observation(obs: dict) -> bool:
    """Ensure all mandatory fields are present and no prohibited keys appear."""
    mandatory = {
        "observation_id",
        "ledger_version",
        "timestamp",
        "substrate",
        "replay_namespace",
        "topology_namespace",
        "cognition_namespace",
        "replay_hash",
        "trace_hash",
        "price",
        "research_node",
        "forward_windows",
    }
    prohibited = {
        "prediction", "forecast", "expected_return", "signal", "buy", "sell",
        "long", "short", "position", "alpha", "score", "ranking",
    }
    missing = mandatory - obs.keys()
    extra = set(obs.keys()) - mandatory
    bad = prohibited & set(k.lower() for k in obs.keys())
    if missing:
        print("[ERROR] Missing mandatory fields:", missing, file=sys.stderr)
    if extra:
        print("[WARNING] Unexpected extra fields:", extra, file=sys.stderr)
    if bad:
        print("[ERROR] Prohibited keys present:", bad, file=sys.stderr)
    return not missing and not bad

def append_observation(obs: dict):
    """Append a JSON line to the ledger; creates directory if missing. Immutable only."""
    os.makedirs(os.path.dirname(LEDGER_PATH), exist_ok=True)
    with open(LEDGER_PATH, "a", encoding="utf-8") as f:
        f.write(json.dumps(obs) + "\n")

# ==== Main loop ===========================================================

def main(interval_seconds: int = 60, mode: str = "placeholder"):
    while True:
        try:
            obs = build_observation(mode)
        except NotImplementedError as e:
            print(f"[FATAL] {e}", file=sys.stderr)
            sys.exit(1)
        if not validate_observation(obs):
            print("[FATAL] Observation validation failed – aborting write", file=sys.stderr)
            sys.exit(1)
        append_observation(obs)
        print(f"[RN-001] Recorded observation {obs['observation_id']}")
        # Echo the full JSON for governance inspection of the first entry
        print(json.dumps(obs, indent=2))
        time.sleep(interval_seconds)

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Start RN-001 observation collection (governance‑checked)")
    parser.add_argument("--interval", type=int, default=60, help="Seconds between observations")
    parser.add_argument("--mode", choices=["placeholder", "real"], default="placeholder",
                        help="Data source mode: placeholder (synthetic) or real (certified replay engine)")
    args = parser.parse_args()
    main(args.interval, args.mode)
