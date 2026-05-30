#!/usr/bin/env python3
"""
ChronoSentiment UI E2E verification (transport layer).

Maps to Pass 6 acceptance tests where automatable without a browser.
Authority: docs/contracts/UI_API_CONTRACT_v1.md, .cursor/rules/chronosentiment-core.mdc

Usage:
  UI_E2E_API_URL=http://localhost:8000 python3 scripts/verify_ui_e2e.py
"""

from __future__ import annotations

import json
import os
import sys
import urllib.error
import urllib.request
from typing import Any

BASE = os.environ.get("UI_E2E_API_URL", "http://localhost:8000").rstrip("/")

STRATEGY_1 = "strat_200_5_4_2"
STRATEGY_2 = "strat_150_3_3_1"
SEED = 42


def get(path: str) -> Any:
    req = urllib.request.Request(f"{BASE}{path}", method="GET")
    with urllib.request.urlopen(req, timeout=120) as resp:
        return json.loads(resp.read().decode())


def post(path: str, body: dict) -> Any:
    data = json.dumps(body).encode()
    req = urllib.request.Request(
        f"{BASE}{path}",
        data=data,
        method="POST",
        headers={"Content-Type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=120) as resp:
        return json.loads(resp.read().decode())


def parse_strategy_params(strategy_id: str) -> dict[str, int]:
    """Mirror my-chrono-sentiment-ui/src/utils/strategyId.js short form."""
    parts = strategy_id.split("_")
    if parts[0] != "strat" or len(parts) < 5:
        raise ValueError(f"unsupported strategy_id: {strategy_id}")
    return {
        "queue_threshold": int(parts[1]),
        "base_edge": int(parts[2]),
        "take_profit": int(parts[3]),
        "stop_loss": int(parts[4]),
    }


class Check:
    def __init__(self, name: str) -> None:
        self.name = name
        self.ok = False
        self.detail = ""

    def pass_(self, detail: str) -> None:
        self.ok = True
        self.detail = detail

    def fail(self, detail: str) -> None:
        self.ok = False
        self.detail = detail


def main() -> int:
    checks: list[Check] = []

    # ── 1. Observatory schema alignment ─────────────────────────────────────
    c = Check("1 Observatory cohort_id")
    try:
        obs = get("/observatory")
        gs = obs.get("governor_state") or {}
        cid = gs.get("cohort_id")
        legacy = "cohort" in gs
        if cid and not legacy:
            c.pass_(f"cohort_id={cid!r}")
        else:
            c.fail(f"cohort_id={cid!r} legacy_cohort={legacy}")
    except Exception as exc:
        c.fail(str(exc))
    checks.append(c)

    # ── 2. Narrative timestamp_ns ───────────────────────────────────────────
    c = Check("2 Inspect narrative timestamp_ns")
    try:
        insp = post("/inspect_strategy", {"strategy_id": STRATEGY_1, "seed": SEED})
        blocks = insp.get("narrative_blocks") or []
        with_ts = [b for b in blocks if b.get("timestamp_ns") is not None]
        legacy_ts = [b for b in blocks if "timestamp" in b and "timestamp_ns" not in b]
        if blocks and len(with_ts) == len(blocks) and not legacy_ts:
            c.pass_(f"{len(blocks)} blocks, all carry timestamp_ns")
        elif blocks:
            c.fail(
                f"{len(with_ts)}/{len(blocks)} blocks have timestamp_ns; "
                f"legacy timestamp-only={len(legacy_ts)}"
            )
        else:
            c.fail("no narrative_blocks returned")
    except Exception as exc:
        c.fail(str(exc))
    checks.append(c)

    # ── 3. Group enum (backend emits canonical groups) ──────────────────────
    c = Check("3 Narrative group enum")
    try:
        insp = post("/inspect_strategy", {"strategy_id": STRATEGY_1, "seed": SEED})
        groups = {b.get("group") for b in (insp.get("narrative_blocks") or [])}
        allowed = {"INTENT", "QUEUE", "EXECUTION", "SETTLEMENT", "GOVERNANCE"}
        bad = groups - allowed
        if bad:
            c.fail(f"unexpected groups: {sorted(bad)}")
        else:
            c.pass_(f"groups={sorted(groups)}")
    except Exception as exc:
        c.fail(str(exc))
    checks.append(c)

    # ── Workspace 1: Run GA ─────────────────────────────────────────────────
    c = Check("W1 Run GA")
    try:
        ga = get("/run_ga")
        gb = ga.get("global_best") or {}
        has_best = isinstance(gb, dict) and gb.get("strategy_id")
        legacy = "results" in ga
        c.pass_(
            f"global_best={gb.get('strategy_id')} "
            f"legacy_results_field={legacy} (normalizeGaResult still needed={legacy})"
        ) if has_best else c.fail(f"missing global_best: keys={list(ga.keys())}")
    except Exception as exc:
        c.fail(str(exc))
    checks.append(c)

    # ── Workspace 2: Inspect Strategy ───────────────────────────────────────
    c = Check("W2 Inspect Strategy")
    try:
        insp = post("/inspect_strategy", {"strategy_id": STRATEGY_1, "seed": SEED})
        ok = (
            insp.get("strategy_id")
            and isinstance(insp.get("narrative_blocks"), list)
            and isinstance(insp.get("execution_trace"), list)
            and insp.get("certification_state")
        )
        c.pass_(
            f"cert={insp.get('certification_state')} "
            f"blocks={len(insp.get('narrative_blocks') or [])} "
            f"trace={len(insp.get('execution_trace') or [])}"
        ) if ok else c.fail(f"missing required inspect surfaces: keys={list(insp.keys())[:8]}")
    except Exception as exc:
        c.fail(str(exc))
    checks.append(c)

    # ── Workspace 3: Compare (UI payload) ───────────────────────────────────
    c = Check("W3 Compare Strategies")
    try:
        body = {
            "strategies": [
                {"strategy_config": parse_strategy_params(STRATEGY_1)},
                {"strategy_config": parse_strategy_params(STRATEGY_2)},
            ],
            "scenarios": [],
            "seed": SEED,
        }
        cmp = post("/compare_strategies", body)
        ranking = cmp.get("ranking") or []
        summary = cmp.get("comparison_summary") or {}
        if len(ranking) >= 2 and summary.get("reason"):
            c.pass_(f"ranking={len(ranking)} reason_present=True")
        else:
            c.fail(f"ranking={len(ranking)} summary_keys={list(summary.keys())}")
    except urllib.error.HTTPError as exc:
        c.fail(f"HTTP {exc.code}: {exc.read().decode()[:240]}")
    except Exception as exc:
        c.fail(str(exc))
    checks.append(c)

    # ── Workspace 4: Global Ranking ─────────────────────────────────────────
    c = Check("W4 Global Ranking")
    try:
        rank = get("/ga/global-ranking")
        if isinstance(rank, dict) and "rankings" in rank and "total" in rank:
            c.pass_(f"total={rank.get('total')} rankings={len(rank.get('rankings') or [])}")
        else:
            c.fail(f"expected envelope {{rankings, total}}, got {type(rank).__name__}")
    except Exception as exc:
        c.fail(str(exc))
    checks.append(c)

    # ── 9. Law One (transport): certification from backend ──────────────────
    c = Check("9 Law One certification_state from backend")
    try:
        insp = post("/inspect_strategy", {"strategy_id": STRATEGY_1, "seed": SEED})
        if insp.get("certification_state"):
            c.pass_(f"certification_state={insp.get('certification_state')!r}")
        else:
            c.fail("certification_state missing from inspect payload")
    except Exception as exc:
        c.fail(str(exc))
    checks.append(c)

    # ── Report ────────────────────────────────────────────────────────────────
    print(f"UI E2E verification against {BASE}\n")
    passed = 0
    for chk in checks:
        status = "PASS" if chk.ok else "FAIL"
        print(f"[{status}] {chk.name}: {chk.detail}")
        if chk.ok:
            passed += 1

    browser_only = [
        "4 Strategy comparison sidebar UI",
        "5 Error surface separation (error1/error2)",
        "6 Raw event toggle",
        "7 ComparisonPanels React warnings",
        "8 Backend offline strip/footer",
        "10 Console integrity (DevTools)",
    ]
    print("\nBrowser-only (Pass 6): manual verification required:")
    for item in browser_only:
        print(f"  - {item}")

    print(f"\nAutomated: {passed}/{len(checks)} passed")
    return 0 if passed == len(checks) else 1


if __name__ == "__main__":
    sys.exit(main())
