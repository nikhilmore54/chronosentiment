#!/usr/bin/env python3
"""
RP-608 Gate 2 v2 — historical run (2E + 2X) on frozen journals.

Authority 0%. Books never pooled. Live baseline unchanged.
2E: decision-time contrasts with chronological discovery/holdout + coverage floor.
2X: delegates to rp608_gate2x_001_multisession (or loads existing results).
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import subprocess
import sys
from collections import defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Callable, Dict, List, Optional, Tuple

WORKSPACE = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(WORKSPACE))

IST = timezone(timedelta(hours=5, minutes=30))
PROTOCOL = "RP-608-GATE2-HISTORICAL-001"
# Gate 2E v2 coverage floor
COV_FLOOR_FRAC = 0.40
COV_FLOOR_N = 80


def _sha(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def _load_jsonl(path: Path) -> List[Dict[str, Any]]:
    out = []
    with path.open() as f:
        for line in f:
            if line.strip():
                out.append(json.loads(line))
    return out


def _day_ist(ms: Any) -> str:
    try:
        return (
            datetime.fromtimestamp(int(ms) / 1000, tz=timezone.utc)
            .astimezone(IST)
            .strftime("%Y-%m-%d")
        )
    except Exception:
        return "UNKNOWN"


def _hour_ist(ms: Any) -> Optional[int]:
    try:
        return (
            datetime.fromtimestamp(int(ms) / 1000, tz=timezone.utc)
            .astimezone(IST)
            .hour
        )
    except Exception:
        return None


def _cohort_stats(rows: List[Dict[str, Any]], net_key: str) -> Dict[str, Any]:
    nets = [float(r[net_key]) for r in rows]
    n = len(nets)
    wins = sum(1 for x in nets if x > 0)
    losses = sum(1 for x in nets if x < 0)
    avg_w = (sum(x for x in nets if x > 0) / wins) if wins else 0.0
    avg_l = (sum(abs(x) for x in nets if x < 0) / losses) if losses else 0.0
    payoff = (avg_w / avg_l) if avg_l > 0 else None
    return {
        "n": n,
        "wins": wins,
        "losses": losses,
        "wr_pct": round(100.0 * wins / n, 2) if n else None,
        "net": round(sum(nets), 4),
        "payoff": round(payoff, 4) if payoff is not None else None,
    }


def _split_chrono(
    rows: List[Dict[str, Any]], day_key: str = "session_ist"
) -> Tuple[List[Dict[str, Any]], List[Dict[str, Any]], List[str], List[str]]:
    days = sorted({r[day_key] for r in rows if r.get(day_key) and r[day_key] != "UNKNOWN"})
    if len(days) < 4:
        cut = max(1, len(days) // 2)
    else:
        cut = max(2, int(math.floor(len(days) * 0.60)))
    disc_days = set(days[:cut])
    hold_days = set(days[cut:])
    disc = [r for r in rows if r[day_key] in disc_days]
    hold = [r for r in rows if r[day_key] in hold_days]
    return disc, hold, sorted(disc_days), sorted(hold_days)


def _coverage_ok(n_pref: int, n_book: int) -> bool:
    if n_book <= 0:
        return False
    return n_pref >= COV_FLOOR_N or (n_pref / n_book) >= COV_FLOOR_FRAC


def _eval_contrast(
    disc: List[Dict[str, Any]],
    hold: List[Dict[str, Any]],
    pred: Callable[[Dict[str, Any]], bool],
    net_key: str,
    name: str,
) -> Dict[str, Any]:
    def side(rows: List[Dict[str, Any]], which: str) -> Dict[str, Any]:
        pref = [r for r in rows if pred(r)]
        ctrl = [r for r in rows if not pred(r)]
        sp = _cohort_stats(pref, net_key)
        sc = _cohort_stats(ctrl, net_key) if ctrl else None
        wr_gap = None
        if sp["wr_pct"] is not None and sc and sc["wr_pct"] is not None:
            wr_gap = round(sp["wr_pct"] - sc["wr_pct"], 2)
        return {
            "preferred": sp,
            "complement": sc,
            "wr_gap_pp": wr_gap,
            "coverage_ok": _coverage_ok(sp["n"], len(rows)),
            "coverage_frac": round(sp["n"] / len(rows), 4) if rows else 0.0,
        }

    d = side(disc, "disc")
    h = side(hold, "hold")
    # Replication: same sign WR gap and preferred WR > complement on holdout
    replicates = False
    if (
        d["wr_gap_pp"] is not None
        and h["wr_gap_pp"] is not None
        and d["preferred"]["n"] >= 30
        and h["preferred"]["n"] >= 30
        and d["coverage_ok"]
        and h["coverage_ok"]
    ):
        replicates = (d["wr_gap_pp"] > 0 and h["wr_gap_pp"] > 0) and (
            h["wr_gap_pp"] >= 0.5 * min(abs(d["wr_gap_pp"]), 20.0) or h["wr_gap_pp"] >= 3.0
        )
    # Gate 2E v2: W/L primary; payoff/net disclosure; coverage floor
    advance = False
    label = "DO_NOT_ADVANCE"
    reason = ""
    if not replicates:
        label = "DO_NOT_ADVANCE"
        reason = "WR gap fails replication or coverage/n floors"
    else:
        # supporting econ on holdout preferred
        pay = h["preferred"].get("payoff")
        net = h["preferred"].get("net")
        if pay is not None and pay >= 1.0 and net is not None and net > 0:
            advance = True
            label = "ADVANCE_CANDIDATE"
            reason = (
                f"WR gap replicates; holdout preferred payoff={pay} net={net} "
                f"cov={h['coverage_frac']}"
            )
        elif h["wr_gap_pp"] and h["wr_gap_pp"] > 0 and h["coverage_ok"]:
            # v2: may still be PARTIAL if W/L replicates but econ weak — not ADVANCE
            label = "REPLICATES_WL_ECON_WEAK"
            reason = (
                f"WR gap replicates on holdout (+{h['wr_gap_pp']}pp) but "
                f"payoff={pay} net={net} — disclose only; no 2E ADVANCE"
            )
        else:
            reason = "replication edge cases"
    return {
        "contrast": name,
        "discovery": d,
        "holdout": h,
        "replicates_wl": replicates,
        "label": label,
        "eligible_for_advance": advance,
        "reason": reason,
    }


def run_2e_crypto(rows: List[Dict[str, Any]]) -> Dict[str, Any]:
    enriched = []
    for r in rows:
        ms = r.get("entry_time_ms")
        imb = r.get("order_flow_imbalance")
        try:
            imb_f = abs(float(imb)) if imb is not None else None
        except (TypeError, ValueError):
            imb_f = None
        atr = float(r.get("atr_14") or 0.0)
        entry = float(r.get("entry_price") or 0.0)
        atr_rel_bps = (atr / entry * 1e4) if entry > 0 and atr > 0 else None
        enriched.append(
            {
                **r,
                "session_ist": _day_ist(ms),
                "hour_ist": _hour_ist(ms),
                "of_sup": str(r.get("order_flow_supported")),
                "abs_imb": imb_f,
                "atr_rel_bps": atr_rel_bps,
                "net_pnl_usdt": float(r["net_pnl_usdt"]),
            }
        )
    disc, hold, dd, hd = _split_chrono(enriched)
    contrasts = [
        ("LONG_vs_not", lambda r: str(r.get("direction") or "").upper() == "LONG"),
        ("SHORT_vs_not", lambda r: str(r.get("direction") or "").upper() == "SHORT"),
        ("OF_SUP_True", lambda r: r.get("of_sup") == "True"),
        ("ABS_IMB_ge_0.70", lambda r: r.get("abs_imb") is not None and r["abs_imb"] >= 0.70),
        ("ATR_REL_lt_20bps", lambda r: r.get("atr_rel_bps") is not None and r["atr_rel_bps"] < 20),
        (
            "HOUR_NOT_19",
            lambda r: r.get("hour_ist") is not None and int(r["hour_ist"]) != 19,
        ),
    ]
    results = [
        _eval_contrast(disc, hold, pred, "net_pnl_usdt", name) for name, pred in contrasts
    ]
    return {
        "book": "crypto_recommendation_plan",
        "n": len(enriched),
        "discovery_days": dd,
        "holdout_days": hd,
        "n_discovery": len(disc),
        "n_holdout": len(hold),
        "contrasts": results,
        "best_label": max(
            results, key=lambda x: (x["eligible_for_advance"], x["replicates_wl"], x["holdout"]["wr_gap_pp"] or -999)
        )["label"],
    }


def run_2e_nse(rows: List[Dict[str, Any]]) -> Dict[str, Any]:
    enriched = []
    for r in rows:
        ms = r.get("entry_timestamp_utc_ms")
        stop_bps = r.get("stop_loss_bps")
        tgt_bps = r.get("target_bps")
        try:
            rr = float(tgt_bps) / float(stop_bps) if stop_bps and float(stop_bps) > 0 else None
        except (TypeError, ValueError):
            rr = None
        bar = r.get("entry_bar_idx")
        enriched.append(
            {
                **r,
                "session_ist": _day_ist(ms),
                "hour_ist": _hour_ist(ms),
                "entry_bar": int(bar) if bar is not None else None,
                "struct_rr": rr,
                "exit_policy": str(r.get("exit_policy") or "NONE"),
                "net_pnl_inr": float(r["net_pnl_inr"]),
            }
        )
    disc, hold, dd, hd = _split_chrono(enriched)
    contrasts = [
        ("BAR_le_8", lambda r: r.get("entry_bar") is not None and r["entry_bar"] <= 8),
        ("BAR_ge_12", lambda r: r.get("entry_bar") is not None and r["entry_bar"] >= 12),
        ("HOUR_10_13", lambda r: r.get("hour_ist") in (10, 11, 12, 13)),
        ("STRUCT_RR_ge_1.3", lambda r: r.get("struct_rr") is not None and r["struct_rr"] >= 1.3),
        (
            "DYNAMIC_POLICY",
            lambda r: "DYNAMIC" in str(r.get("exit_policy") or "").upper(),
        ),
    ]
    results = [
        _eval_contrast(disc, hold, pred, "net_pnl_inr", name) for name, pred in contrasts
    ]
    return {
        "book": "nse_estimand_a",
        "n": len(enriched),
        "discovery_days": dd,
        "holdout_days": hd,
        "n_discovery": len(disc),
        "n_holdout": len(hold),
        "contrasts": results,
        "note": "LONG-only book; side contrast unavailable",
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--nse-journal",
        type=Path,
        default=WORKSPACE / "live_capture/shadow_validation/nse/rp608_nse_trade_journal.jsonl",
    )
    ap.add_argument(
        "--crypto-journal",
        type=Path,
        default=WORKSPACE
        / "live_capture/shadow_validation/crypto/trade_plan_overlay_v1/trade_plan_outcomes.jsonl",
    )
    ap.add_argument(
        "--out-dir",
        type=Path,
        default=WORKSPACE / "artifacts/rp608_gate2_historical",
    )
    ap.add_argument("--skip-2x-rerun", action="store_true")
    args = ap.parse_args()
    out: Path = args.out_dir
    out.mkdir(parents=True, exist_ok=True)

    # ── 2X ──
    x_json = WORKSPACE / "artifacts/rp608_gate2x_001/gate2x_001_multisession_results.json"
    if not args.skip_2x_rerun:
        subprocess.check_call(
            [sys.executable, str(WORKSPACE / "scripts/research/rp608_gate2x_001_multisession.py")],
            cwd=str(WORKSPACE),
        )
    x_report = json.loads(x_json.read_text()) if x_json.exists() else {}

    # ── 2E ──
    crypto = _load_jsonl(args.crypto_journal)
    nse = _load_jsonl(args.nse_journal)
    e_crypto = run_2e_crypto(crypto)
    e_nse = run_2e_nse(nse)

    report = {
        "protocol_id": PROTOCOL,
        "gate2_amendment": "docs/research/rp608_gate2_amendment_v2.md",
        "generated_utc": datetime.now(timezone.utc).isoformat(),
        "authority_pct": 0.0,
        "fingerprints": {
            "crypto": {
                "path": str(args.crypto_journal),
                "sha256": _sha(args.crypto_journal),
                "n": len(crypto),
            },
            "nse": {
                "path": str(args.nse_journal),
                "sha256": _sha(args.nse_journal),
                "n": len(nse),
            },
        },
        "coverage_floor": {"frac": COV_FLOOR_FRAC, "n": COV_FLOOR_N},
        "gate_2e": {"crypto_recommendation_plan": e_crypto, "nse_estimand_a": e_nse},
        "gate_2x": x_report.get("books", x_report),
    }

    # Headline dispositions
    def flatten_2e(block: Dict[str, Any]) -> List[Dict[str, Any]]:
        return [
            {
                "contrast": c["contrast"],
                "label": c["label"],
                "replicates_wl": c["replicates_wl"],
                "eligible_for_advance": c["eligible_for_advance"],
                "reason": c["reason"],
                "holdout_wr_gap_pp": c["holdout"]["wr_gap_pp"],
                "holdout_pref_n": c["holdout"]["preferred"]["n"],
                "holdout_pref_net": c["holdout"]["preferred"]["net"],
                "holdout_pref_payoff": c["holdout"]["preferred"]["payoff"],
            }
            for c in block["contrasts"]
        ]

    headlines = {
        "gate_2e_crypto": flatten_2e(e_crypto),
        "gate_2e_nse": flatten_2e(e_nse),
        "gate_2x_crypto": (x_report.get("books") or {}).get("crypto_recommendation_plan", {}).get(
            "verdicts", {}
        ),
        "gate_2x_nse": (x_report.get("books") or {}).get("nse_estimand_a", {}).get("verdicts", {}),
    }
    report["headlines"] = headlines

    json_path = out / "gate2_historical_run.json"
    json_path.write_text(json.dumps(report, indent=2))

    # Markdown review
    lines = [
        "# RP-608 Gate 2 v2 — Historical Run",
        "",
        f"**Generated:** {report['generated_utc']}  ",
        f"**Protocol:** `{PROTOCOL}`  ",
        f"**Amendment:** Gate 2 v2 (`rp608_gate2_amendment_v2.md`)  ",
        f"**Authority:** 0% · live baseline frozen  ",
        "",
        "## Fingerprints",
        "",
        f"- Crypto: n={len(crypto)} sha256=`{report['fingerprints']['crypto']['sha256']}`",
        f"- NSE A: n={len(nse)} sha256=`{report['fingerprints']['nse']['sha256']}`",
        "",
        f"Chronological split ~60/40 by IST session day. Coverage floor: "
        f"≥{int(COV_FLOOR_FRAC*100)}% or n≥{COV_FLOOR_N}.",
        "",
        "## Gate 2E — Crypto plan",
        "",
        f"Discovery days: `{e_crypto['discovery_days']}`  ",
        f"Holdout days: `{e_crypto['holdout_days']}`",
        "",
        "| Contrast | label | repl WL | hold WR gap | hold n | hold net | hold payoff |",
        "| --- | --- | --- | ---: | ---: | ---: | ---: |",
    ]
    for c in headlines["gate_2e_crypto"]:
        lines.append(
            f"| `{c['contrast']}` | `{c['label']}` | {c['replicates_wl']} | "
            f"{c['holdout_wr_gap_pp']} | {c['holdout_pref_n']} | {c['holdout_pref_net']} | "
            f"{c['holdout_pref_payoff']} |"
        )

    lines += [
        "",
        "## Gate 2E — NSE Estimand A",
        "",
        f"Discovery days: `{e_nse['discovery_days']}`  ",
        f"Holdout days: `{e_nse['holdout_days']}`",
        "",
        "| Contrast | label | repl WL | hold WR gap | hold n | hold net | hold payoff |",
        "| --- | --- | --- | ---: | ---: | ---: | ---: |",
    ]
    for c in headlines["gate_2e_nse"]:
        lines.append(
            f"| `{c['contrast']}` | `{c['label']}` | {c['replicates_wl']} | "
            f"{c['holdout_wr_gap_pp']} | {c['holdout_pref_n']} | {c['holdout_pref_net']} | "
            f"{c['holdout_pref_payoff']} |"
        )

    lines += ["", "## Gate 2X — Crypto plan", ""]
    for pol, v in (headlines["gate_2x_crypto"] or {}).items():
        lines.append(f"- **{pol}**: `{v.get('label')}` — {v.get('reason')}")

    lines += ["", "## Gate 2X — NSE Estimand A", ""]
    for pol, v in (headlines["gate_2x_nse"] or {}).items():
        lines.append(f"- **{pol}**: `{v.get('label')}` — {v.get('reason')}")

    lines += [
        "",
        "## Overall Gate 2 disposition (this run)",
        "",
        "| Track | Book | Disposition |",
        "| --- | --- | --- |",
    ]
    any_2e_adv = any(c["eligible_for_advance"] for c in headlines["gate_2e_crypto"] + headlines["gate_2e_nse"])
    lines.append(
        f"| 2E | Crypto | `{'ADVANCE_CANDIDATE present' if any(c['eligible_for_advance'] for c in headlines['gate_2e_crypto']) else 'NO_ADVANCE'}` |"
    )
    lines.append(
        f"| 2E | NSE A | `{'ADVANCE_CANDIDATE present' if any(c['eligible_for_advance'] for c in headlines['gate_2e_nse']) else 'NO_ADVANCE'}` |"
    )
    x_c_adv = any(
        (v or {}).get("eligible_for_advance") for v in (headlines["gate_2x_crypto"] or {}).values()
    )
    x_n_adv = any(
        (v or {}).get("eligible_for_advance") for v in (headlines["gate_2x_nse"] or {}).values()
    )
    lines.append(f"| 2X | Crypto | `{'ADVANCE_CANDIDATE (G/H*)' if x_c_adv else 'NO_ADVANCE'}` |")
    lines.append(f"| 2X | NSE A | `{'ADVANCE_CANDIDATE' if x_n_adv else 'NO_ADVANCE (F scratch failed)'}` |")
    lines += [
        "",
        "**Live daemon:** still frozen (Gate 5).  ",
        f"**2E any ADVANCE:** {any_2e_adv}  ",
        f"**2X crypto ADVANCE:** {x_c_adv}  ",
        "",
        "Full JSON: `artifacts/rp608_gate2_historical/gate2_historical_run.json`  ",
        "2X detail: `artifacts/rp608_gate2x_001/` · `docs/research/rp608_gate2x_001_review.md`",
        "",
    ]
    md_path = out / "gate2_historical_run.md"
    md_path.write_text("\n".join(lines))
    print(f"wrote {json_path}")
    print(f"wrote {md_path}")
    print(json.dumps({"2e_any_advance": any_2e_adv, "2x_crypto_advance": x_c_adv, "2x_nse_advance": x_n_adv}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
