#!/usr/bin/env python3
"""
RP-608-WL-IMPROVE-001 — Cross-session replication of Crypto plan
decision-time entry-quality associations beyond 2026-11-03…05.

Discovery associations are fixed from Phase 2 (direction, OF support,
imbalance, direction×OF). Holdout windows do not re-tune buckets.

Decision-time predictors only. Baseline frozen. Gate 3 unauthorized.
"""

from __future__ import annotations

import argparse
import json
import math
from collections import defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence, Tuple

IST = timezone(timedelta(hours=5, minutes=30))

DISCOVERY = ["2026-11-03", "2026-11-04", "2026-11-05"]
# Pre-discovery holdout (journal history). Oct-29 flagged as high-n anomaly.
HOLDOUT_PRE = [
    "2026-10-26",
    "2026-10-27",
    "2026-10-28",
    "2026-10-29",
    "2026-10-30",
    "2026-10-31",
    "2026-11-01",
    "2026-11-02",
]
HOLDOUT_PRE_EX_ANOMALY = [d for d in HOLDOUT_PRE if d != "2026-10-29"]
HOLDOUT_FWD = ["2026-11-06"]  # partial day; small-n caution
OF_ERA_START = "2026-11-01"  # first day with material OF field coverage


def fnum(r: Dict[str, Any], *keys: str) -> Optional[float]:
    for k in keys:
        if r.get(k) is None:
            continue
        try:
            return float(r[k])
        except (TypeError, ValueError):
            continue
    return None


def pdt(v: Any) -> Optional[datetime]:
    if v is None:
        return None
    if isinstance(v, (int, float)):
        ms = int(v)
        if ms > 10_000_000_000:
            ms //= 1000
        try:
            return datetime.fromtimestamp(ms, tz=timezone.utc)
        except Exception:
            return None
    try:
        dt = datetime.fromisoformat(str(v).replace("Z", "+00:00"))
        if dt.tzinfo is None:
            dt = dt.replace(tzinfo=timezone.utc)
        return dt
    except Exception:
        return None


def exit_day_ist(r: Dict[str, Any]) -> Optional[str]:
    dt = pdt(r.get("exit_time_ms"))
    if not dt:
        return None
    return dt.astimezone(IST).strftime("%Y-%m-%d")


def wilson_low(wins: int, n: int, z: float = 1.96) -> Optional[float]:
    if n <= 0:
        return None
    p = wins / n
    denom = 1 + z * z / n
    centre = p + z * z / (2 * n)
    adj = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n))
    return round(100 * max(0.0, (centre - adj) / denom), 2)


def atr_bucket(atr: Optional[float], entry: Optional[float]) -> str:
    if atr is None or entry is None or entry <= 0:
        return "ATR_REL_UNAVAILABLE"
    rel = atr / entry
    if rel < 0.002:
        return "ATR_REL_<20bps"
    if rel < 0.004:
        return "ATR_REL_20_40bps"
    if rel < 0.008:
        return "ATR_REL_40_80bps"
    return "ATR_REL_≥80bps"


def imb_bucket(imb: Optional[float]) -> str:
    if imb is None:
        return "IMB_UNAVAILABLE"
    a = abs(imb)
    if a < 0.30:
        return "ABS_IMB_<0.30"
    if a < 0.50:
        return "ABS_IMB_0.30_0.50"
    if a < 0.70:
        return "ABS_IMB_0.50_0.70"
    return "ABS_IMB_≥0.70"


def enrich(r: Dict[str, Any]) -> Optional[Dict[str, Any]]:
    net = fnum(r, "net_pnl_usdt", "net_pnl")
    if net is None:
        return None
    day = exit_day_ist(r)
    if not day:
        return None
    entry = fnum(r, "entry_price", "reference_price")
    atr = fnum(r, "atr_14")
    imb = fnum(r, "order_flow_imbalance")
    # OF support: missing field => UNAVAILABLE (pre-OF era), not False
    if "order_flow_supported" not in r and r.get("order_flow_imbalance") is None:
        of_sup = "OF_SUP_UNAVAILABLE"
        of_bool = None
    else:
        of_bool = bool(r.get("order_flow_supported"))
        of_sup = "OF_SUP=True" if of_bool else "OF_SUP=False"
    direction = str(r.get("direction") or "?").upper()
    return {
        "day": day,
        "net": net,
        "win": net > 0,
        "direction": direction,
        "of_supported": of_sup,
        "imb_bucket": imb_bucket(imb),
        "atr_bucket": atr_bucket(atr, entry),
        "dir_x_of": f"{direction}|{of_sup}",
        "plan_id": r.get("plan_id"),
        "of_era": day >= OF_ERA_START,
    }


def summarize(rows: Sequence[Dict[str, Any]]) -> Dict[str, Any]:
    n = len(rows)
    w = sum(1 for r in rows if r["win"])
    return {
        "n": n,
        "wins": w,
        "losses": n - w,
        "win_rate_pct": round(100 * w / n, 2) if n else None,
        "wilson_low_95_pct": wilson_low(w, n),
        "net_sum": round(sum(r["net"] for r in rows), 4) if rows else None,
    }


def slice_wr(rows: Sequence[Dict[str, Any]], key: str, value: str) -> Dict[str, Any]:
    sub = [r for r in rows if r.get(key) == value]
    s = summarize(sub)
    s["key"] = value
    s["field"] = key
    return s


def contrast(rows: Sequence[Dict[str, Any]], key: str, a: str, b: str) -> Dict[str, Any]:
    sa, sb = slice_wr(rows, key, a), slice_wr(rows, key, b)
    gap = None
    if sa["win_rate_pct"] is not None and sb["win_rate_pct"] is not None:
        gap = round(sa["win_rate_pct"] - sb["win_rate_pct"], 2)
    return {
        "field": key,
        "a": sa,
        "b": sb,
        "wr_gap_pp": gap,
        "direction_matches_discovery": None,  # filled by caller for signed tests
    }


def session_table(rows: Sequence[Dict[str, Any]], key: str, value: str) -> List[Dict[str, Any]]:
    out = []
    by_day: Dict[str, List[Dict[str, Any]]] = defaultdict(list)
    for r in rows:
        if r.get(key) == value:
            by_day[r["day"]].append(r)
    for d in sorted(by_day):
        s = summarize(by_day[d])
        s["day"] = d
        out.append(s)
    return out


def load_journal(path: Path) -> List[Dict[str, Any]]:
    rows = []
    seen = set()
    for line in path.read_text(encoding="utf-8", errors="ignore").splitlines():
        if not line.strip():
            continue
        try:
            r = json.loads(line)
        except Exception:
            continue
        e = enrich(r)
        if not e:
            continue
        pid = str(e.get("plan_id") or "")
        dedupe = pid or f"{e['day']}:{e['direction']}:{e['net']}:{id(r)}"
        if pid and pid in seen:
            continue
        if pid:
            seen.add(pid)
        rows.append(e)
    return rows


def eval_hypothesis(
    name: str,
    discovery: Sequence[Dict[str, Any]],
    holdout: Sequence[Dict[str, Any]],
    key: str,
    high: str,
    low: str,
    *,
    min_n_side: int = 20,
    of_era_only: bool = False,
) -> Dict[str, Any]:
    d_rows = [r for r in discovery if (r["of_era"] if of_era_only else True)]
    h_rows = [r for r in holdout if (r["of_era"] if of_era_only else True)]
    # For OF fields, drop UNAVAILABLE
    if key in ("of_supported", "dir_x_of") or "OF_SUP" in high:
        d_rows = [r for r in d_rows if "UNAVAILABLE" not in str(r.get(key))]
        h_rows = [r for r in h_rows if "UNAVAILABLE" not in str(r.get(key))]
        if key == "imb_bucket":
            pass
    if key == "imb_bucket":
        d_rows = [r for r in d_rows if r["imb_bucket"] != "IMB_UNAVAILABLE"]
        h_rows = [r for r in h_rows if r["imb_bucket"] != "IMB_UNAVAILABLE"]

    d_c = contrast(d_rows, key, high, low)
    h_c = contrast(h_rows, key, high, low)

    def pass_side(c: Dict[str, Any]) -> Tuple[bool, str]:
        if c["a"]["n"] < min_n_side or c["b"]["n"] < min_n_side:
            return False, "UNDERPOWERED"
        if c["wr_gap_pp"] is None:
            return False, "NO_DATA"
        if c["wr_gap_pp"] <= 0:
            return False, "GAP_FLIPPED_OR_ZERO"
        # require high side WR strictly above low side (already gap>0)
        return True, "GAP_SAME_SIGN"

    d_ok, d_lab = pass_side(d_c)
    h_ok, h_lab = pass_side(h_c)
    if h_lab == "UNDERPOWERED":
        verdict = "INCONCLUSIVE_UNDERPOWERED"
    elif not d_ok:
        verdict = "DISCOVERY_FAILED_RERUN"
    elif h_ok:
        verdict = "REPLICATES"
    elif h_lab == "GAP_FLIPPED_OR_ZERO":
        verdict = "FAILS_REPLICATION"
    else:
        verdict = f"FAILS_REPLICATION_{h_lab}"

    return {
        "name": name,
        "key": key,
        "high": high,
        "low": low,
        "of_era_only": of_era_only,
        "discovery": d_c,
        "holdout": h_c,
        "discovery_session_high": session_table(d_rows, key, high),
        "holdout_session_high": session_table(h_rows, key, high),
        "verdict": verdict,
        "coverage_note": (
            f"Selecting only `{high}` keeps "
            f"{d_c['a']['n']}/{summarize(d_rows)['n']} discovery trades "
            f"({round(100*d_c['a']['n']/max(1,summarize(d_rows)['n']),1)}% coverage)"
        ),
    }


def render(payload: Dict[str, Any]) -> str:
    lines = [
        "# RP-608-WL-IMPROVE-001 — Crypto Plan Entry-Quality Replication",
        "",
        "**Authority:** 0% · baseline frozen · decision-time predictors only",
        "**Discovery:** 2026-11-03 … 2026-11-05 (recomputed from live plan journal)",
        "**Holdout pre:** 2026-10-26 … 2026-11-02",
        "**Holdout forward:** 2026-11-06 (partial; caution)",
        "**Anomaly flag:** 2026-10-29 n≈379 — reported with and without this day",
        "",
        "Replication does **not** authorize filters. Gate 2 still requires a falsifiable hypothesis.",
        "Gate 3 unauthorized. Other books deferred.",
        "",
        "## Cohort sizes",
        "",
        f"| Window | n | WR% |",
        f"| --- | ---: | ---: |",
        f"| Discovery Nov3–5 | {payload['cohorts']['discovery']['n']} | {payload['cohorts']['discovery']['win_rate_pct']} |",
        f"| Holdout pre (all) | {payload['cohorts']['holdout_pre']['n']} | {payload['cohorts']['holdout_pre']['win_rate_pct']} |",
        f"| Holdout pre ex-1029 | {payload['cohorts']['holdout_pre_ex_anomaly']['n']} | {payload['cohorts']['holdout_pre_ex_anomaly']['win_rate_pct']} |",
        f"| Holdout forward Nov6 | {payload['cohorts']['holdout_fwd']['n']} | {payload['cohorts']['holdout_fwd']['win_rate_pct']} |",
        "",
        "### Per-day WR (journal)",
        "",
        "| Day | Window | n | WR% |",
        "| --- | --- | ---: | ---: |",
    ]
    for row in payload["per_day"]:
        lines.append(f"| {row['day']} | {row['window']} | {row['n']} | {row['win_rate_pct']} |")

    lines += ["", "## Fixed discovery associations tested", ""]

    def block(title: str, tests: List[Dict[str, Any]]) -> None:
        lines.append(f"### {title}")
        lines.append("")
        lines.append(
            "| Hypothesis | Discovery WR high/low (n) | Holdout WR high/low (n) | Gap disc→hold pp | Verdict |"
        )
        lines.append("| --- | --- | --- | ---: | --- |")
        for t in tests:
            d, h = t["discovery"], t["holdout"]
            lines.append(
                f"| {t['name']} | "
                f"{d['a']['win_rate_pct']}%/{d['b']['win_rate_pct']}% "
                f"({d['a']['n']}/{d['b']['n']}) | "
                f"{h['a']['win_rate_pct']}%/{h['b']['win_rate_pct']}% "
                f"({h['a']['n']}/{h['b']['n']}) | "
                f"{d['wr_gap_pp']} → {h['wr_gap_pp']} | **{t['verdict']}** |"
            )
        lines.append("")

    block("Primary holdout (pre, all days)", payload["tests_holdout_pre"])
    block("Sensitivity: holdout pre excluding 2026-10-29", payload["tests_holdout_pre_ex"])
    block("Forward holdout 2026-11-06 (partial)", payload["tests_holdout_fwd"])

    lines += [
        "## Replication summary",
        "",
    ]
    for line in payload["summary_bullets"]:
        lines.append(f"- {line}")
    lines += [
        "",
        "## Governance",
        "",
        "No execution change. No OF ±0.30 change. No hour/direction ban.",
        "Next: hypothesis specification only for associations that **REPLICATE** "
        "(or clearly fail → drop). Then Gate 2 review.",
        "",
        f"Source journal: `{payload['journal']}`",
        "",
    ]
    return "\n".join(lines)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--journal",
        default="live_capture/shadow_validation/crypto/trade_plan_overlay_v1/trade_plan_outcomes.jsonl",
    )
    ap.add_argument("--out-dir", default="artifacts/rp608_econ_edge_diag")
    args = ap.parse_args()

    all_rows = load_journal(Path(args.journal))
    discovery = [r for r in all_rows if r["day"] in DISCOVERY]
    hold_pre = [r for r in all_rows if r["day"] in HOLDOUT_PRE]
    hold_pre_x = [r for r in all_rows if r["day"] in HOLDOUT_PRE_EX_ANOMALY]
    hold_fwd = [r for r in all_rows if r["day"] in HOLDOUT_FWD]

    # Fixed Phase 2 associations (same bucket defs; not re-tuned)
    specs = [
        ("H_DIR: LONG > SHORT", "direction", "LONG", "SHORT", False),
        ("H_OF: OF_SUP=True > False", "of_supported", "OF_SUP=True", "OF_SUP=False", True),
        (
            "H_DIR_OF: LONG|OF_SUP=True > SHORT|OF_SUP=True",
            "dir_x_of",
            "LONG|OF_SUP=True",
            "SHORT|OF_SUP=True",
            True,
        ),
        (
            "H_IMB: ABS_IMB_≥0.70 > ABS_IMB_<0.30",
            "imb_bucket",
            "ABS_IMB_≥0.70",
            "ABS_IMB_<0.30",
            True,
        ),
        (
            "H_ATR: ATR_REL_<20bps > ATR_REL_40_80bps",
            "atr_bucket",
            "ATR_REL_<20bps",
            "ATR_REL_40_80bps",
            False,
        ),
    ]

    def run_all(hold: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
        out = []
        for name, key, high, low, of_era in specs:
            out.append(
                eval_hypothesis(
                    name, discovery, hold, key, high, low, of_era_only=of_era, min_n_side=15
                )
            )
        return out

    tests_pre = run_all(hold_pre)
    tests_pre_x = run_all(hold_pre_x)
    # forward: lower min_n because partial day
    tests_fwd = []
    for name, key, high, low, of_era in specs:
        tests_fwd.append(
            eval_hypothesis(
                name, discovery, hold_fwd, key, high, low, of_era_only=of_era, min_n_side=5
            )
        )

    per_day = []
    for d in sorted({r["day"] for r in all_rows}):
        sub = [r for r in all_rows if r["day"] == d]
        if d in DISCOVERY:
            w = "discovery"
        elif d in HOLDOUT_FWD:
            w = "holdout_fwd"
        elif d in HOLDOUT_PRE:
            w = "holdout_pre" + ("_ANOMALY" if d == "2026-10-29" else "")
        else:
            w = "other"
        s = summarize(sub)
        per_day.append({"day": d, "window": w, **s})

    # Summary bullets
    bullets = []
    for t in tests_pre_x:
        bullets.append(f"{t['name']}: **{t['verdict']}** on holdout pre ex-1029 "
                       f"(disc gap {t['discovery']['wr_gap_pp']}pp → hold {t['holdout']['wr_gap_pp']}pp).")
    replicates = [t["name"] for t in tests_pre_x if t["verdict"] == "REPLICATES"]
    fails = [t["name"] for t in tests_pre_x if t["verdict"] == "FAILS_REPLICATION"]
    under = [t["name"] for t in tests_pre_x if "UNDERPOWERED" in t["verdict"]]
    bullets.append(
        f"Count (ex-1029): REPLICATES={len(replicates)} · FAILS={len(fails)} · UNDERPOWERED={len(under)}."
    )
    if replicates:
        bullets.append(
            "Only REPLICATES associations are eligible for hypothesis specification (step 4)."
        )
    else:
        bullets.append(
            "No association cleanly REPLICATES on holdout ex-1029 — do not invent a Gate 2 hypothesis from discovery alone."
        )
    bullets.append(
        "Oct-29 anomaly: include-all holdout results are sensitivity-only; primary read is ex-1029."
    )
    bullets.append(
        "Forward Nov-6 is incomplete; treat as smoke check only."
    )

    payload = {
        "protocol": "RP-608-WL-IMPROVE-001",
        "step": "cross_session_replication",
        "journal": args.journal,
        "cohorts": {
            "discovery": summarize(discovery),
            "holdout_pre": summarize(hold_pre),
            "holdout_pre_ex_anomaly": summarize(hold_pre_x),
            "holdout_fwd": summarize(hold_fwd),
        },
        "per_day": per_day,
        "tests_holdout_pre": tests_pre,
        "tests_holdout_pre_ex": tests_pre_x,
        "tests_holdout_fwd": tests_fwd,
        "summary_bullets": bullets,
        "note_vs_phase2_export": (
            "Discovery n may differ from Phase 2 session_export_* files if the live "
            "journal accumulated additional closed plans after those exports."
        ),
    }

    out_dir = Path(args.out_dir)
    stem = out_dir / "wl_phase2_crypto_plan_replication_beyond_2026-11-03_to_05"
    stem.with_suffix(".json").write_text(json.dumps(payload, indent=2, default=str), encoding="utf-8")
    stem.with_suffix(".md").write_text(render(payload), encoding="utf-8")
    print(json.dumps({
        "discovery_n": payload["cohorts"]["discovery"]["n"],
        "holdout_pre_ex_n": payload["cohorts"]["holdout_pre_ex_anomaly"]["n"],
        "verdicts_ex1029": {t["name"]: t["verdict"] for t in tests_pre_x},
        "wrote": str(stem) + ".md",
    }, indent=2))


if __name__ == "__main__":
    main()
