#!/usr/bin/env python3
"""
RP-608 P4 — H2 Crypto Plan Session Regime (frozen observational test).

Protocol freeze: docs/research/rp608_hypothesis_p3_freeze.md
Book: crypto plan only. No thr scan. No H1/H3/H4. Gate 2 closed.

Usage (Pi):
  python3 scripts/research/rp608_h2_crypto_plan_session_regime_p4.py \
    --outcomes live_capture/shadow_validation/crypto/trade_plan_overlay_v1/trade_plan_outcomes.jsonl \
    --out-dir artifacts/rp608_econ_edge_diag
"""

from __future__ import annotations

import argparse
import json
import math
import statistics
from collections import defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence, Tuple

IST = timezone(timedelta(hours=5, minutes=30))
PROTOCOL = "RP-608-H2-CRYPTO-PLAN-SESSION-REGIME-P4"
BOOK = "crypto_recommendation_plan"

DISCOVERY_END = "2026-10-31"
VALIDATION_START = "2026-11-01"
VALIDATION_END = "2026-11-05"
CONFIRMATION_START = "2026-11-06"


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


def session_day_ist(ms: Any) -> Optional[str]:
    if ms is None:
        return None
    try:
        v = int(ms)
    except Exception:
        return None
    return datetime.fromtimestamp(v / 1000.0, tz=timezone.utc).astimezone(IST).date().isoformat()


def window_for_day(day: str) -> str:
    if day <= DISCOVERY_END:
        return "discovery"
    if VALIDATION_START <= day <= VALIDATION_END:
        return "validation"
    if day >= CONFIRMATION_START:
        return "confirmation"
    return "out_of_window"


def of_stratum(r: Dict[str, Any]) -> str:
    v = r.get("order_flow_supported")
    if v is True:
        return "True"
    if v is False:
        return "False"
    return "Missing"


def econ(rows: Sequence[Dict[str, Any]]) -> Dict[str, Any]:
    usable = [r for r in rows if r.get("net_pnl_usdt") is not None]
    n = len(usable)
    wins = [float(r["net_pnl_usdt"]) for r in usable if float(r["net_pnl_usdt"]) > 0]
    losses = [float(r["net_pnl_usdt"]) for r in usable if float(r["net_pnl_usdt"]) <= 0]
    fric = [float(r["friction_usdt"]) for r in usable if r.get("friction_usdt") is not None]
    avg_win = sum(wins) / len(wins) if wins else None
    avg_loss_abs = (sum(abs(x) for x in losses) / len(losses)) if losses else None
    payoff = (
        round(avg_win / avg_loss_abs, 4)
        if avg_win is not None and avg_loss_abs is not None and avg_loss_abs > 0
        else None
    )
    wr = round(100.0 * len(wins) / n, 2) if n else None
    return {
        "n": n,
        "wins": len(wins),
        "losses": len(losses),
        "wr_pct": wr,
        "payoff": payoff,
        "net_usdt": round(sum(float(r["net_pnl_usdt"]) for r in usable), 4) if usable else None,
        "friction_usdt": round(sum(fric), 4) if fric else None,
        "avg_win_usdt": None if avg_win is None else round(avg_win, 4),
        "avg_loss_abs_usdt": None if avg_loss_abs is None else round(avg_loss_abs, 4),
    }


def session_table(rows: Sequence[Dict[str, Any]]) -> List[Dict[str, Any]]:
    by: Dict[str, List[Dict[str, Any]]] = defaultdict(list)
    for r in rows:
        d = r.get("session_day_ist")
        if d:
            by[str(d)].append(r)
    out = []
    for day in sorted(by):
        s = econ(by[day])
        out.append({"session_day_ist": day, "window": window_for_day(day), **s})
    return out


def concentration(sessions: Sequence[Dict[str, Any]]) -> Dict[str, Any]:
    if not sessions:
        return {"top_session_by_n": None, "top_n_share_pct": None, "top_session_by_wins": None, "top_wins_share_pct": None, "top_session_by_net": None}
    total_n = sum(int(s["n"]) for s in sessions) or 1
    total_wins = sum(int(s["wins"]) for s in sessions) or 1
    by_n = max(sessions, key=lambda s: s["n"])
    by_w = max(sessions, key=lambda s: s["wins"])
    by_net = max(sessions, key=lambda s: (s["net_usdt"] if s["net_usdt"] is not None else float("-inf")))
    return {
        "top_session_by_n": by_n["session_day_ist"],
        "top_n_share_pct": round(100.0 * by_n["n"] / total_n, 2),
        "top_session_by_wins": by_w["session_day_ist"],
        "top_wins_share_pct": round(100.0 * by_w["wins"] / total_wins, 2),
        "top_session_by_net": by_net["session_day_ist"],
        "top_session_net_usdt": by_net["net_usdt"],
    }


def wr_dispersion(sessions: Sequence[Dict[str, Any]]) -> Dict[str, Any]:
    wrs = [float(s["wr_pct"]) for s in sessions if s.get("wr_pct") is not None and int(s["n"]) >= 5]
    if len(wrs) < 2:
        return {
            "n_sessions_ge5": len(wrs),
            "wr_min": None if not wrs else wrs[0],
            "wr_max": None if not wrs else wrs[0],
            "wr_range_pp": None,
            "wr_stdev": None,
            "high_dispersion": False,
            "note": "insufficient sessions with n>=5",
        }
    return {
        "n_sessions_ge5": len(wrs),
        "wr_min": round(min(wrs), 2),
        "wr_max": round(max(wrs), 2),
        "wr_range_pp": round(max(wrs) - min(wrs), 2),
        "wr_stdev": round(statistics.pstdev(wrs), 2),
        "high_dispersion": (max(wrs) - min(wrs)) >= 15.0,
        "note": "high_dispersion if WR range across sessions with n>=5 is >= 15 pp",
    }


def of_secondary(rows: Sequence[Dict[str, Any]]) -> List[Dict[str, Any]]:
    by: Dict[str, List[Dict[str, Any]]] = defaultdict(list)
    for r in rows:
        by[of_stratum(r)].append(r)
    out = []
    for k in ("True", "False", "Missing"):
        out.append({"order_flow_supported": k, **econ(by.get(k, []))})
    return out


def integrity(raw_n: int, bad: int, eligible: Sequence[Dict[str, Any]], path: Path) -> Dict[str, Any]:
    return {
        "outcomes_path": str(path),
        "raw_rows": raw_n,
        "json_parse_failures": bad,
        "eligible_n": len(eligible),
        "book": BOOK,
        "protocol": PROTOCOL,
        "checks": {
            "book_identity": "crypto_plan_outcomes_only",
            "chronological_boundaries": {
                "discovery": f"session_day_ist <= {DISCOVERY_END}",
                "validation": f"{VALIDATION_START} .. {VALIDATION_END}",
                "confirmation": f">= {CONFIRMATION_START}",
            },
            "decision_time_eligibility": "closed plan outcomes with entry_time_ms + net_pnl_usdt; no post-entry predictors",
            "outcome_join": "native net_pnl_usdt on outcomes row",
            "missing_net_or_entry_excluded": True,
            "no_threshold_scan": True,
            "h1_h3_h4_not_opened": True,
        },
    }


def dispose(
    disc_disp: Dict[str, Any],
    val_disp: Dict[str, Any],
    conf_n: int,
    disc_sessions: Sequence[Dict[str, Any]],
    val_sessions: Sequence[Dict[str, Any]],
) -> Dict[str, Any]:
    """
    H2 disposition: clarity on session-regime dominance — not a tradable rule.
    """
    limitations = [
        "Observational only; does not authorize filters, urgency, or live authority.",
        "H2 documents session heterogeneity; it is not an entry policy.",
        "Confirmation window currently thin if n sessions/trades small.",
        "Optional OF stratum is descriptive only (not H3).",
    ]
    if not disc_sessions or not val_sessions:
        return {
            "disposition": "BLOCKED",
            "reason": "Missing discovery or validation sessions after eligibility filter.",
            "limitations": limitations,
            "gate2": "DO_NOT_ADVANCE",
        }

    d_high = bool(disc_disp.get("high_dispersion"))
    v_high = bool(val_disp.get("high_dispersion"))
    d_conc = concentration(disc_sessions)
    v_conc = concentration(val_sessions)

    # Structural story: high WR dispersion in discovery AND validation
    if d_high and v_high:
        label = "SUPPORTED_SESSION_REGIME_DOMINANT"
        detail = (
            "Discovery and validation both show high cross-session WR dispersion "
            f"(disc range {disc_disp.get('wr_range_pp')} pp; val range {val_disp.get('wr_range_pp')} pp). "
            "Session-level conditions dominate the WR narrative; not a transferable entry rule."
        )
    elif d_high and not v_high:
        label = "INCONCLUSIVE"
        detail = (
            "Discovery shows high session WR dispersion, but validation does not meet the "
            "predeclared high-dispersion criterion. Session-regime account not validated."
        )
    elif not d_high and v_high:
        label = "INCONCLUSIVE"
        detail = (
            "Validation shows high dispersion without the same discovery pattern strength. "
            "Do not promote a session-regime policy."
        )
    else:
        label = "NOT_SUPPORTED"
        detail = (
            "Neither discovery nor validation shows high cross-session WR dispersion "
            "under the predeclared >=15 pp range (n>=5 sessions)."
        )

    conf_status = "NEED_MORE_SESSIONS" if conf_n < 100 else "ELIGIBLE_TO_EVALUATE_SEPARATELY"
    if conf_n == 0:
        conf_status = "NO_CONFIRMATION_ROWS"

    return {
        "disposition": label,
        "detail": detail,
        "confirmation_status": conf_status,
        "confirmation_n": conf_n,
        "discovery_dispersion": disc_disp,
        "validation_dispersion": val_disp,
        "discovery_concentration": d_conc,
        "validation_concentration": v_conc,
        "limitations": limitations,
        "gate2": "DO_NOT_ADVANCE",
        "h4": "PARKED_PERMANENTLY_UNAVAILABLE_UNDER_CURRENT_CRYPTO_ENGINES",
        "note": "SUPPORTED here means descriptive session-regime clarity — not Gate 2 advance.",
    }


def run(outcomes_path: Path) -> Dict[str, Any]:
    raw, bad = load_jsonl(outcomes_path)
    eligible: List[Dict[str, Any]] = []
    excluded = {"missing_entry_time_ms": 0, "missing_net_pnl_usdt": 0}
    for r in raw:
        if r.get("entry_time_ms") is None:
            excluded["missing_entry_time_ms"] += 1
            continue
        if r.get("net_pnl_usdt") is None:
            excluded["missing_net_pnl_usdt"] += 1
            continue
        row = dict(r)
        day = session_day_ist(row.get("entry_time_ms"))
        row["session_day_ist"] = day
        row["window"] = window_for_day(day) if day else "unknown"
        eligible.append(row)

    by_win = {
        "discovery": [r for r in eligible if r.get("window") == "discovery"],
        "validation": [r for r in eligible if r.get("window") == "validation"],
        "confirmation": [r for r in eligible if r.get("window") == "confirmation"],
    }

    disc_sessions = session_table(by_win["discovery"])
    val_sessions = session_table(by_win["validation"])
    conf_sessions = session_table(by_win["confirmation"])

    disc_disp = wr_dispersion(disc_sessions)
    val_disp = wr_dispersion(val_sessions)
    conf_disp = wr_dispersion(conf_sessions)

    integ = integrity(len(raw), bad, eligible, outcomes_path)
    integ["excluded"] = excluded
    integ["window_counts"] = {k: len(v) for k, v in by_win.items()}

    disposition = dispose(
        disc_disp,
        val_disp,
        len(by_win["confirmation"]),
        disc_sessions,
        val_sessions,
    )

    return {
        "protocol": PROTOCOL,
        "book": BOOK,
        "authority": "0.0%",
        "gate2_disposition": "GATE2_DO_NOT_ADVANCE",
        "baseline_frozen": True,
        "of_frozen": True,
        "hypotheses_opened": ["H2"],
        "hypotheses_not_opened": ["H1", "H3", "H4"],
        "freeze_ref": "docs/research/rp608_hypothesis_p3_freeze.md",
        "integrity": integ,
        "discovery": {
            "window": f"<= {DISCOVERY_END}",
            "aggregate": econ(by_win["discovery"]),
            "sessions": disc_sessions,
            "dispersion": disc_disp,
            "concentration": concentration(disc_sessions),
            "of_supported_secondary_descriptive": of_secondary(by_win["discovery"]),
        },
        "validation": {
            "window": f"{VALIDATION_START} .. {VALIDATION_END}",
            "aggregate": econ(by_win["validation"]),
            "sessions": val_sessions,
            "dispersion": val_disp,
            "concentration": concentration(val_sessions),
            "of_supported_secondary_descriptive": of_secondary(by_win["validation"]),
        },
        "confirmation": {
            "window": f">= {CONFIRMATION_START}",
            "merged_into_validation": False,
            "aggregate": econ(by_win["confirmation"]),
            "sessions": conf_sessions,
            "dispersion": conf_disp,
            "concentration": concentration(conf_sessions),
            "status": disposition["confirmation_status"],
            "note": "Reported separately; not confirmation of H2 unless status allows and governance reviews.",
        },
        "h2_assessment": disposition,
    }


def render_md(rep: Dict[str, Any]) -> str:
    h2 = rep["h2_assessment"]
    lines = [
        "# RP-608 P4 — H2 Crypto Plan Session Regime",
        "",
        f"**Protocol:** `{rep['protocol']}`  ",
        f"**Book:** `{rep['book']}` · Authority {rep['authority']}  ",
        f"**Gate 2:** `{rep['gate2_disposition']}` · baseline/OF frozen  ",
        f"**Opened:** {rep['hypotheses_opened']} · **Not opened:** {rep['hypotheses_not_opened']}  ",
        f"**Freeze:** `{rep['freeze_ref']}`",
        "",
        "## Disposition",
        "",
        f"**H2:** `{h2['disposition']}`  ",
        f"{h2['detail']}  ",
        f"**Confirmation:** `{h2['confirmation_status']}` (n={h2['confirmation_n']})  ",
        f"**Gate 2:** `{h2['gate2']}` — descriptive clarity only; not an entry policy.",
        "",
        "## Integrity",
        "",
        f"- Outcomes: `{rep['integrity']['outcomes_path']}`",
        f"- Raw rows: {rep['integrity']['raw_rows']} · parse failures: {rep['integrity']['json_parse_failures']} · eligible: {rep['integrity']['eligible_n']}",
        f"- Excluded: `{rep['integrity']['excluded']}`",
        f"- Window counts: `{rep['integrity']['window_counts']}`",
        f"- Checks: `{json.dumps(rep['integrity']['checks'])}`",
        "",
    ]

    def block(title: str, part: Dict[str, Any]) -> None:
        lines.extend(
            [
                f"## {title}",
                "",
                f"Window: `{part['window']}`",
                "",
                f"Aggregate: n={part['aggregate']['n']} · WR={part['aggregate']['wr_pct']}% · "
                f"W/L={part['aggregate']['wins']}/{part['aggregate']['losses']} · "
                f"payoff={part['aggregate']['payoff']} · net={part['aggregate']['net_usdt']} USDT · "
                f"friction={part['aggregate']['friction_usdt']}",
                "",
                f"Dispersion: `{part['dispersion']}`",
                "",
                f"Concentration: `{part['concentration']}`",
                "",
                "| Session (IST) | n | WR% | W/L | Payoff | Net USDT | Friction |",
                "| --- | ---: | ---: | --- | ---: | ---: | ---: |",
            ]
        )
        for s in part["sessions"]:
            lines.append(
                f"| {s['session_day_ist']} | {s['n']} | {s['wr_pct']} | {s['wins']}/{s['losses']} | "
                f"{s['payoff']} | {s['net_usdt']} | {s['friction_usdt']} |"
            )
        lines.extend(
            [
                "",
                "OF-supported secondary (descriptive; not H3):",
                "",
                "| Stratum | n | WR% | W/L | Payoff | Net |",
                "| --- | ---: | ---: | --- | ---: | ---: |",
            ]
        )
        for row in part.get("of_supported_secondary_descriptive") or []:
            lines.append(
                f"| {row['order_flow_supported']} | {row['n']} | {row['wr_pct']} | "
                f"{row['wins']}/{row['losses']} | {row['payoff']} | {row['net_usdt']} |"
            )
        lines.append("")

    block("Discovery", rep["discovery"])
    block("Validation", rep["validation"])
    lines.extend(
        [
            "## Confirmation (separate — not merged into validation)",
            "",
            f"Status: `{rep['confirmation']['status']}`",
            "",
        ]
    )
    block("Confirmation window detail", rep["confirmation"])
    lines.extend(
        [
            "## Limitations",
            "",
        ]
    )
    for lim in h2["limitations"]:
        lines.append(f"- {lim}")
    lines.extend(
        [
            "",
            "## Governance",
            "",
            "- No threshold scan · no new features · no execution velocity inference",
            "- H4 remains parked / permanently unavailable under current crypto engines",
            "- Live authority 0% · Gate 2 closed",
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
    args = ap.parse_args()
    outcomes = Path(args.outcomes)
    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    if not outcomes.exists():
        print(json.dumps({"status": "MISSING_OUTCOMES", "path": str(outcomes)}))
        return 2
    rep = run(outcomes)
    stem = out_dir / "rp608_h2_crypto_plan_session_regime_p4"
    stem.with_suffix(".json").write_text(json.dumps(rep, indent=2), encoding="utf-8")
    stem.with_suffix(".md").write_text(render_md(rep), encoding="utf-8")
    print(
        json.dumps(
            {
                "status": "OK",
                "disposition": rep["h2_assessment"]["disposition"],
                "gate2": rep["gate2_disposition"],
                "discovery_n": rep["discovery"]["aggregate"]["n"],
                "validation_n": rep["validation"]["aggregate"]["n"],
                "confirmation_n": rep["confirmation"]["aggregate"]["n"],
                "discovery_wr_range_pp": rep["discovery"]["dispersion"].get("wr_range_pp"),
                "validation_wr_range_pp": rep["validation"]["dispersion"].get("wr_range_pp"),
                "confirmation_status": rep["confirmation"]["status"],
                "md": str(stem.with_suffix(".md")),
                "json": str(stem.with_suffix(".json")),
            },
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
