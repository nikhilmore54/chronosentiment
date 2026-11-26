#!/usr/bin/env python3
"""
LIVE Pipeline Health Check — live_health.py

End-to-end health and smoke test for the LIVE-001 → LIVE-006 pipeline.
Read-only: does NOT modify any files or server state.
Exits non-zero if any critical check fails.

Checks performed:
  1. latest.json freshness (staleness detection)
  2. Disk ledger count matches expected (no silent drops)
  3. Duplicate decision_id detection in disk ledger
  4. Server reachability: GET /recommendations/v1/latest
  5. Server decision count matches disk ledger count
  6. Every disk decision_id is present on the server
  7. TIME-009 observation artifact coverage (every ledger entry has an artifact)
  8. No TIME-009 artifacts in ERROR state

Usage:
    python3 scripts/live_health.py \
        --ledger       live_capture/ledger/entries \
        --observations time_machine/analysis/TIME009/observations \
        --server-url   http://localhost:3001 \
        --latest-json  live_capture/certifications/latest.json
"""

import argparse
import json
import sys
from collections import Counter
from datetime import datetime, timedelta, timezone
from pathlib import Path
from urllib import error as url_error
from urllib import request

# Staleness threshold: latest.json older than this is flagged
STALENESS_HOURS = 26  # one trading day + buffer


def parse_args():
    p = argparse.ArgumentParser(description="LIVE pipeline health check (read-only)")
    p.add_argument(
        "--ledger",
        default="live_capture/ledger/entries",
        help="Directory containing LIVE-005 ledger entry JSON files",
    )
    p.add_argument(
        "--observations",
        default="time_machine/analysis/TIME009/observations",
        help="Directory containing TIME009-OBS-*.json artifacts",
    )
    p.add_argument(
        "--server-url",
        default="http://localhost:3001",
        help="Base URL of the recommendations server",
    )
    p.add_argument(
        "--latest-json",
        default=None,
        help="Path to latest.json (certification or recommendation)",
    )
    p.add_argument(
        "--strict",
        action="store_true",
        help="Treat warnings as failures",
    )
    return p.parse_args()


class HealthReport:
    def __init__(self):
        self.failures = []
        self.warnings = []
        self.passes = []

    def fail(self, check, detail):
        msg = f"  FAIL  [{check}] {detail}"
        self.failures.append(msg)
        print(msg)

    def warn(self, check, detail):
        msg = f"  WARN  [{check}] {detail}"
        self.warnings.append(msg)
        print(msg)

    def ok(self, check, detail=""):
        msg = f"  OK    [{check}] {detail}"
        self.passes.append(msg)
        print(msg)

    def section(self, title):
        print()
        print(f"── {title} " + "─" * max(0, 56 - len(title)))

    def summary(self, strict=False):
        print()
        print("=" * 60)
        print("LIVE PIPELINE HEALTH SUMMARY")
        print(f"  Passed  : {len(self.passes)}")
        print(f"  Warnings: {len(self.warnings)}")
        print(f"  Failures: {len(self.failures)}")
        print("=" * 60)
        n_fail = len(self.failures) + (len(self.warnings) if strict else 0)
        if n_fail == 0:
            print("RESULT: HEALTHY")
        else:
            print("RESULT: UNHEALTHY")
        return n_fail


def load_ledger_entries(ledger_dir: Path, report: HealthReport):
    entries = []
    if not ledger_dir.exists():
        report.fail("LEDGER_DIR", f"ledger directory not found: {ledger_dir}")
        return entries
    for f in sorted(ledger_dir.iterdir()):
        if f.suffix != ".json":
            continue
        try:
            data = json.loads(f.read_text())
            entries.append(data)
        except Exception as e:
            report.fail("LEDGER_PARSE", f"cannot parse {f.name}: {e}")
    return entries


def load_observations(obs_dir: Path, report: HealthReport):
    obs = []
    if not obs_dir.exists():
        report.warn("OBS_DIR", f"observations directory not found: {obs_dir}")
        return obs
    for f in sorted(obs_dir.iterdir()):
        if f.suffix != ".json" or f.name == "latest_run.json":
            continue
        try:
            obs.append(json.loads(f.read_text()))
        except Exception as e:
            report.warn("OBS_PARSE", f"cannot parse {f.name}: {e}")
    return obs


def fetch_url_json(url: str, report: HealthReport, check_name: str):
    """Fetch a JSON endpoint. Returns parsed body or None on failure."""
    try:
        req = request.Request(url, headers={"Accept": "application/json"})
        with request.urlopen(req, timeout=10) as resp:
            return json.loads(resp.read().decode())
    except url_error.URLError as e:
        report.warn(check_name, f"cannot reach {url}: {e}")
        return None
    except Exception as e:
        report.warn(check_name, f"unexpected error fetching {url}: {e}")
        return None


def fetch_server_decisions(server_url: str, report: HealthReport):
    """Fetch /decisions (raw ledger) from server. Returns list or None.

    The server deduplicates by effective_session for /recommendations/v1/latest,
    but /decisions returns all stored decisions. We use /decisions for count
    and ID checks so that multiple runs on the same day are all accounted for.
    """
    url = server_url.rstrip("/") + "/decisions"
    data = fetch_url_json(url, report, "SERVER_REACH")
    if data is None:
        return None
    if isinstance(data, list):
        return data
    if isinstance(data, dict):
        # Envelope: {"decisions": [...], "total": N}
        for key in ("decisions", "items", "data"):
            if key in data:
                return data[key]
    return []


def check_latest_json_freshness(latest_json_path, report: HealthReport):
    report.section("CHECK 1: latest.json freshness")
    if latest_json_path is None:
        # Try common locations
        candidates = [
            Path("live_capture/certifications/latest.json"),
            Path("live_capture/recommendations/latest.json"),
            Path("live_capture/ledger/latest.json"),
        ]
        for c in candidates:
            if c.exists():
                latest_json_path = c
                break

    if latest_json_path is None:
        report.warn("FRESHNESS", "latest.json not found in common locations")
        return

    path = Path(latest_json_path)
    if not path.exists():
        report.fail("FRESHNESS", f"latest.json not found at {path}")
        return

    try:
        data = json.loads(path.read_text())
        ts_str = (
            data.get("generated_at")
            or data.get("timestamp")
            or data.get("created_at")
            or data.get("certified_at")
            or data.get("run_at")
        )
        if ts_str is None:
            report.warn("FRESHNESS", f"{path}: no timestamp field found")
            return
        ts = datetime.fromisoformat(ts_str.replace("Z", "+00:00"))
        age_h = (datetime.now(timezone.utc) - ts).total_seconds() / 3600
        if age_h > STALENESS_HOURS:
            report.fail(
                "FRESHNESS",
                f"{path.name} is {age_h:.1f}h old (threshold={STALENESS_HOURS}h, ts={ts_str[:19]})"
            )
        else:
            report.ok("FRESHNESS", f"{path.name} is {age_h:.1f}h old (ts={ts_str[:19]})")
    except Exception as e:
        report.warn("FRESHNESS", f"cannot parse {path}: {e}")


def check_ledger_duplicates(entries, report: HealthReport):
    report.section("CHECK 2: Disk ledger duplicate decision_ids")
    ids = [e.get("decision_id", "") for e in entries]
    counts = Counter(ids)
    dups = {did: n for did, n in counts.items() if n > 1}
    if dups:
        for did, n in sorted(dups.items()):
            report.fail("LEDGER_DUP", f"decision_id={did} appears {n} times")
    else:
        report.ok("LEDGER_DUP", f"no duplicates in {len(entries)} ledger entries")


def check_server_reachability(server_url: str, server_decisions, report: HealthReport):
    report.section("CHECK 3: Server reachability")
    if server_decisions is None:
        report.fail("SERVER_REACH", f"server at {server_url} is unreachable")
    else:
        report.ok("SERVER_REACH", f"server returned {len(server_decisions)} decisions")


def check_server_vs_disk_count(entries, server_decisions, report: HealthReport):
    report.section("CHECK 4: Server decision count vs disk ledger")
    if server_decisions is None:
        report.warn("SERVER_COUNT", "skipped — server unreachable")
        return

    n_disk = len(entries)
    n_server = len(server_decisions)

    if n_server < n_disk:
        report.fail(
            "SERVER_COUNT",
            f"server has {n_server} decisions but disk has {n_disk} — {n_disk - n_server} missing"
        )
    elif n_server > n_disk:
        report.warn(
            "SERVER_COUNT",
            f"server has {n_server} decisions but disk has {n_disk} — {n_server - n_disk} extra (may be from previous runs)"
        )
    else:
        report.ok("SERVER_COUNT", f"server and disk both have {n_disk} decisions")


def check_disk_ids_on_server(entries, server_decisions, report: HealthReport):
    report.section("CHECK 5: Every disk decision_id present on server")
    if server_decisions is None:
        report.warn("SERVER_IDS", "skipped — server unreachable")
        return

    server_ids = set()
    for d in server_decisions:
        did = d.get("decision_id") or d.get("id") or d.get("decisionId")
        if did:
            server_ids.add(did)

    disk_ids = {e.get("decision_id", "") for e in entries}
    missing = disk_ids - server_ids

    if missing:
        report.fail(
            "SERVER_IDS",
            f"{len(missing)} disk decision_ids not found on server: "
            f"{sorted(missing)[:3]}{'...' if len(missing) > 3 else ''}"
        )
    else:
        report.ok("SERVER_IDS", f"all {len(disk_ids)} disk decision_ids present on server")


def check_time009_coverage(entries, observations, report: HealthReport):
    report.section("CHECK 6: TIME-009 observation coverage")
    ledger_ids = {e.get("decision_id", "") for e in entries}
    obs_ids = {o.get("decision_id", "") for o in observations}

    missing = ledger_ids - obs_ids
    if missing:
        report.fail(
            "T009_COVERAGE",
            f"{len(missing)} ledger entries have no TIME-009 artifact: "
            f"{sorted(missing)[:3]}{'...' if len(missing) > 3 else ''}"
        )
    else:
        report.ok("T009_COVERAGE", f"all {len(ledger_ids)} ledger entries have TIME-009 artifacts")

    # Check for ERROR state
    error_obs = [o for o in observations if o.get("observation_status") not in ("PENDING", "COMPLETE")]
    if error_obs:
        for o in error_obs:
            report.fail(
                "T009_STATUS",
                f"observation {o.get('decision_id', '?')} has unexpected status={o.get('observation_status')!r}"
            )
    else:
        n_pending = sum(1 for o in observations if o.get("observation_status") == "PENDING")
        n_complete = sum(1 for o in observations if o.get("observation_status") == "COMPLETE")
        report.ok("T009_STATUS", f"PENDING={n_pending} COMPLETE={n_complete} ERROR=0")


def main():
    args = parse_args()
    report = HealthReport()

    print("=" * 60)
    print("LIVE PIPELINE HEALTH CHECK")
    print(f"Run time: {datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M UTC')}")
    print(f"Ledger      : {args.ledger}")
    print(f"Observations: {args.observations}")
    print(f"Server      : {args.server_url}")
    print("=" * 60)

    # Load data
    entries = load_ledger_entries(Path(args.ledger), report)
    observations = load_observations(Path(args.observations), report)
    server_decisions = fetch_server_decisions(args.server_url, report)

    print(f"\nLoaded: {len(entries)} ledger entries, {len(observations)} observations")
    if server_decisions is not None:
        print(f"Server: {len(server_decisions)} decisions")

    # Run checks
    check_latest_json_freshness(args.latest_json, report)
    check_ledger_duplicates(entries, report)
    check_server_reachability(args.server_url, server_decisions, report)
    check_server_vs_disk_count(entries, server_decisions, report)
    check_disk_ids_on_server(entries, server_decisions, report)
    check_time009_coverage(entries, observations, report)

    return 1 if report.summary(strict=args.strict) > 0 else 0


if __name__ == "__main__":
    sys.exit(main())