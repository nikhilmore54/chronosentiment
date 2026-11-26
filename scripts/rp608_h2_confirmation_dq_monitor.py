#!/usr/bin/env python3
"""
RP-608 H2 — Confirmation data-quality monitor (waiting-period infrastructure).

States per confirmation session: ELIGIBLE | INCOMPLETE | INTEGRITY_FAILURE
Does NOT: open hypotheses, change confirmation floors, or declare H2 confirmed.

Protocol refs:
  docs/research/rp608_h2_prospective_confirmation_protocol.md
  docs/research/rp608_h2_prospective_confirmation_accumulation.md

Usage (prefer Pi canonical outcomes):
  python3 scripts/research/rp608_h2_confirmation_dq_monitor.py \
    --outcomes live_capture/shadow_validation/crypto/trade_plan_overlay_v1/trade_plan_outcomes.jsonl \
    --out-dir artifacts/rp608_econ_edge_diag
"""

from __future__ import annotations

import argparse
import hashlib
import json
from collections import defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence, Tuple

IST = timezone(timedelta(hours=5, minutes=30))
PROTOCOL = "RP-608-H2-CONFIRMATION-DQ-MONITOR-001"
CONFIRMATION_START = "2026-11-06"
DISCOVERY_END = "2026-10-31"
VALIDATION_END = "2026-11-05"
POWERED_N = 5
READY_POWERED_SESSIONS = 5
READY_TOTAL_TRADES = 200
CONCENTRATION_FLAG_PCT = 60.0

# Decision-time fields useful for integrity (absence → incomplete, not a new hypothesis)
DECISION_TIME_FIELDS = (
    "entry_time_ms",
    "direction",
    "entry_price",
    "reference_price",
    "atr_14",
    "plan_id",
)


def load_jsonl(path: Path) -> Tuple[List[Dict[str, Any]], int]:
    rows: List[Dict[str, Any]] = []
    bad = 0
    if not path.exists():
        return rows, 0
    for line in path.read_text(encoding="utf-8", errors="ignore").splitlines():
        if not line.strip():
            continue
        try:
            rows.append(json.loads(line))
        except Exception:
            bad += 1
    return rows, bad


def file_fingerprint(path: Path) -> Dict[str, Any]:
    if not path.exists():
        return {"path": str(path), "exists": False}
    data = path.read_bytes()
    return {
        "path": str(path.resolve()),
        "exists": True,
        "bytes": len(data),
        "sha256": hashlib.sha256(data).hexdigest(),
        "mtime_utc": datetime.fromtimestamp(path.stat().st_mtime, tz=timezone.utc).isoformat(),
    }


def session_day_ist(ms: Any) -> Optional[str]:
    if ms is None:
        return None
    try:
        v = int(ms)
    except Exception:
        return None
    return datetime.fromtimestamp(v / 1000.0, tz=timezone.utc).astimezone(IST).date().isoformat()


def classify_row(r: Dict[str, Any]) -> str:
    """Return ok | missing_entry | missing_net | bad_day."""
    if r.get("entry_time_ms") is None:
        return "missing_entry"
    if r.get("net_pnl_usdt") is None:
        return "missing_net"
    day = session_day_ist(r.get("entry_time_ms"))
    if not day:
        return "bad_day"
    return "ok"


def econ(rows: Sequence[Dict[str, Any]]) -> Dict[str, Any]:
    n = len(rows)
    wins = [float(r["net_pnl_usdt"]) for r in rows if float(r["net_pnl_usdt"]) > 0]
    losses = [float(r["net_pnl_usdt"]) for r in rows if float(r["net_pnl_usdt"]) <= 0]
    fric = [float(r["friction_usdt"]) for r in rows if r.get("friction_usdt") is not None]
    avg_w = sum(wins) / len(wins) if wins else None
    avg_l = sum(abs(x) for x in losses) / len(losses) if losses else None
    payoff = (
        round(avg_w / avg_l, 4) if avg_w is not None and avg_l is not None and avg_l > 0 else None
    )
    return {
        "n": n,
        "wins": len(wins),
        "losses": len(losses),
        "wr_pct": round(100.0 * len(wins) / n, 2) if n else None,
        "payoff": payoff,
        "net_usdt": round(sum(float(r["net_pnl_usdt"]) for r in rows), 4) if rows else None,
        "friction_usdt": round(sum(fric), 4) if fric else None,
        "payoff_net_complete": all(r.get("net_pnl_usdt") is not None for r in rows) and n > 0,
        "friction_complete_pct": round(100.0 * len(fric) / n, 2) if n else None,
    }


def decision_time_completeness(rows: Sequence[Dict[str, Any]]) -> Dict[str, Any]:
    out = {}
    n = len(rows) or 1
    for f in DECISION_TIME_FIELDS:
        present = sum(1 for r in rows if r.get(f) is not None and r.get(f) != "")
        out[f] = {"present": present, "pct": round(100.0 * present / n, 2)}
    return out


def session_state(
    day: str,
    ok_rows: List[Dict[str, Any]],
    excluded: Dict[str, int],
    parse_failures_global: int,
    already_recorded: bool,
) -> Dict[str, Any]:
    """Classify one confirmation session."""
    reasons: List[str] = []
    # Integrity failures: parse issues attributed at file level; per-session if exclusions dominate
    excl_n = sum(excluded.values())
    if parse_failures_global > 0 and not ok_rows and excl_n == 0:
        state = "INTEGRITY_FAILURE"
        reasons.append("file_parse_failures_and_no_ok_rows")
    elif excl_n > 0 and not ok_rows:
        state = "INTEGRITY_FAILURE"
        reasons.append("all_rows_failed_entry_or_net_join")
    elif not ok_rows:
        state = "INCOMPLETE"
        reasons.append("no_eligible_closed_trades")
    else:
        e = econ(ok_rows)
        dt = decision_time_completeness(ok_rows)
        # Incomplete if core decision-time fields materially missing
        core_miss = any(dt[f]["pct"] < 100.0 for f in ("entry_time_ms", "direction", "entry_price", "plan_id"))
        if not e["payoff_net_complete"]:
            state = "INCOMPLETE"
            reasons.append("net_pnl_incomplete")
        elif core_miss:
            state = "INCOMPLETE"
            reasons.append("decision_time_fields_incomplete")
        elif day < CONFIRMATION_START:
            state = "INTEGRITY_FAILURE"
            reasons.append("session_before_confirmation_window")
        elif DISCOVERY_END < day <= VALIDATION_END or day <= DISCOVERY_END:
            # Should never appear in confirmation scan
            state = "INTEGRITY_FAILURE"
            reasons.append("consumed_discovery_or_validation_day")
        else:
            state = "ELIGIBLE"
            if e["n"] < POWERED_N:
                reasons.append("thin_session_n_lt_5_still_eligible_for_accumulation")
    size_class = (
        "powered"
        if len(ok_rows) >= POWERED_N
        else ("thin" if ok_rows else "empty")
    )
    return {
        "session_day_ist": day,
        "state": state,
        "size_class": size_class,
        "already_recorded": already_recorded,
        "qualifies_under_frozen_confirmation_protocol": state == "ELIGIBLE",
        "reasons": reasons,
        "excluded": excluded,
        "economics": econ(ok_rows) if ok_rows else econ([]),
        "decision_time_completeness": decision_time_completeness(ok_rows) if ok_rows else {},
        "outcome_join_ok": all(r.get("net_pnl_usdt") is not None for r in ok_rows) if ok_rows else False,
    }


def run(outcomes: Path, registry_path: Path) -> Dict[str, Any]:
    raw, bad = load_jsonl(outcomes)
    fp = file_fingerprint(outcomes)

    # Prior registry of recorded sessions
    prior: Dict[str, Any] = {}
    if registry_path.exists():
        try:
            prior = json.loads(registry_path.read_text(encoding="utf-8"))
        except Exception:
            prior = {}
    previously = set((prior.get("recorded_eligible_sessions") or []))

    by_day_ok: Dict[str, List[Dict[str, Any]]] = defaultdict(list)
    by_day_excl: Dict[str, Dict[str, int]] = defaultdict(lambda: defaultdict(int))
    consumed_leak: List[str] = []

    for r in raw:
        status = classify_row(r)
        day = session_day_ist(r.get("entry_time_ms")) if r.get("entry_time_ms") is not None else None
        if status != "ok":
            if day and day >= CONFIRMATION_START:
                by_day_excl[day][status] += 1
            continue
        assert day is not None
        if day < CONFIRMATION_START:
            # track leak attempts into confirmation monitor (should be ignored)
            if day not in consumed_leak:
                consumed_leak.append(day)
            continue
        by_day_ok[day].append(r)

    all_conf_days = sorted(set(by_day_ok) | set(by_day_excl))
    sessions = []
    for day in all_conf_days:
        sessions.append(
            session_state(
                day,
                by_day_ok.get(day, []),
                dict(by_day_excl.get(day, {})),
                bad,
                day in previously,
            )
        )

    eligible = [s for s in sessions if s["state"] == "ELIGIBLE"]
    powered = [s for s in eligible if s["size_class"] == "powered"]
    total_eligible_trades = sum(int(s["economics"]["n"]) for s in eligible)
    top_share = None
    if eligible and total_eligible_trades > 0:
        top = max(eligible, key=lambda s: s["economics"]["n"])
        top_share = round(100.0 * top["economics"]["n"] / total_eligible_trades, 2)

    readiness = {
        "powered_sessions": len(powered),
        "powered_sessions_floor": READY_POWERED_SESSIONS,
        "total_eligible_trades": total_eligible_trades,
        "total_trades_floor": READY_TOTAL_TRADES,
        "top_session_share_pct": top_share,
        "concentration_flag_pct": CONCENTRATION_FLAG_PCT,
        "concentration_flagged": bool(top_share is not None and top_share >= CONCENTRATION_FLAG_PCT),
        "assessment_ready": len(powered) >= READY_POWERED_SESSIONS
        and total_eligible_trades >= READY_TOTAL_TRADES,
        "status": (
            "ASSESSMENT_READY_TO_SCHEDULE"
            if (
                len(powered) >= READY_POWERED_SESSIONS
                and total_eligible_trades >= READY_TOTAL_TRADES
            )
            else "NEED_MORE_SESSIONS"
        ),
        "note": (
            "ASSESSMENT_READY_TO_SCHEDULE means floors met — still requires a separate "
            "confirmation assessment run under the frozen protocol. This monitor never "
            "declares H2 confirmed or PERSISTS."
        ),
    }

    recorded_eligible = sorted({s["session_day_ist"] for s in eligible} | previously)

    return {
        "protocol": PROTOCOL,
        "authority": "0.0%",
        "gate2": "CLOSED",
        "hypotheses_opened": [],
        "does_not_declare_h2_confirmed": True,
        "confirmation_window_start": CONFIRMATION_START,
        "source": fp,
        "parse_failures": bad,
        "raw_rows": len(raw),
        "sessions": sessions,
        "state_counts": {
            "ELIGIBLE": sum(1 for s in sessions if s["state"] == "ELIGIBLE"),
            "INCOMPLETE": sum(1 for s in sessions if s["state"] == "INCOMPLETE"),
            "INTEGRITY_FAILURE": sum(1 for s in sessions if s["state"] == "INTEGRITY_FAILURE"),
        },
        "readiness": readiness,
        "ignored_pre_confirmation_days_present_in_file": sorted(consumed_leak)[:20],
        "recorded_eligible_sessions": recorded_eligible,
        "new_eligible_since_prior_registry": sorted(
            {s["session_day_ist"] for s in eligible} - previously
        ),
    }


def render_md(rep: Dict[str, Any]) -> str:
    lines = [
        "# RP-608 H2 — Confirmation Data-Quality Monitor",
        "",
        f"**Protocol:** `{rep['protocol']}` · Authority {rep['authority']} · Gate 2 {rep['gate2']}",
        "",
        "Does **not** open hypotheses, change floors, or declare H2 confirmed.",
        "",
        "## Source fingerprint",
        "",
        f"```json\n{json.dumps(rep['source'], indent=2)}\n```",
        "",
        f"Raw rows: {rep['raw_rows']} · parse failures: {rep['parse_failures']}",
        "",
        "## Readiness (frozen floors)",
        "",
        f"```json\n{json.dumps(rep['readiness'], indent=2)}\n```",
        "",
        f"**Status:** `{rep['readiness']['status']}`",
        "",
        "## Session states",
        "",
        f"Counts: `{rep['state_counts']}`",
        "",
        "| Day (IST) | State | Size | n | WR% | W/L | Payoff | Net | Already recorded |",
        "| --- | --- | --- | ---: | ---: | --- | ---: | ---: | --- |",
    ]
    for s in rep["sessions"]:
        e = s["economics"]
        lines.append(
            f"| {s['session_day_ist']} | {s['state']} | {s['size_class']} | {e['n']} | "
            f"{e['wr_pct']} | {e['wins']}/{e['losses']} | {e['payoff']} | {e['net_usdt']} | "
            f"{s['already_recorded']} |"
        )
    lines.extend(
        [
            "",
            f"New eligible since prior registry: `{rep['new_eligible_since_prior_registry']}`",
            "",
            "## Governance",
            "",
            "- Confirmation criteria unchanged",
            "- Discovery/validation days ignored for eligibility",
            "- No automatic H2 confirmation",
            "",
        ]
    )
    return "\n".join(lines)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--outcomes",
        default="live_capture/shadow_validation/crypto/trade_plan_overlay_v1/trade_plan_outcomes.jsonl",
    )
    ap.add_argument("--out-dir", default="artifacts/rp608_econ_edge_diag")
    ap.add_argument(
        "--registry",
        default="artifacts/rp608_econ_edge_diag/rp608_h2_confirmation_session_registry.json",
    )
    args = ap.parse_args()
    outcomes = Path(args.outcomes)
    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    registry_path = Path(args.registry)
    if not outcomes.exists():
        print(json.dumps({"status": "MISSING_OUTCOMES", "path": str(outcomes)}))
        return 2
    rep = run(outcomes, registry_path)
    stem = out_dir / "rp608_h2_confirmation_dq_monitor"
    stem.with_suffix(".json").write_text(json.dumps(rep, indent=2), encoding="utf-8")
    stem.with_suffix(".md").write_text(render_md(rep), encoding="utf-8")
    # Update registry with eligible sessions only (idempotent record-keeping)
    registry_path.write_text(
        json.dumps(
            {
                "protocol": PROTOCOL,
                "updated_utc": datetime.now(tz=timezone.utc).isoformat(),
                "source_sha256": rep["source"].get("sha256"),
                "recorded_eligible_sessions": rep["recorded_eligible_sessions"],
                "last_readiness_status": rep["readiness"]["status"],
            },
            indent=2,
        ),
        encoding="utf-8",
    )
    print(
        json.dumps(
            {
                "status": "OK",
                "readiness": rep["readiness"]["status"],
                "state_counts": rep["state_counts"],
                "eligible_sessions": [s["session_day_ist"] for s in rep["sessions"] if s["state"] == "ELIGIBLE"],
                "source_sha256": rep["source"].get("sha256"),
                "md": str(stem.with_suffix(".md")),
            },
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
