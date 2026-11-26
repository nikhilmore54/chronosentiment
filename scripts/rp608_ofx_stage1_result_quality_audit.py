#!/usr/bin/env python3
"""
RP-608-OFX Stage 1 — result-quality audit (not Stage 2).

Diagnostics:
1. Event denominator + deduplication
2. Inferred-aggression assignment / ambiguous handling
3. Response magnitude distributions
4. Coverage by instrument and time window
5. Concentration (symbol / time domination)
6. Flat-band documentation (±2 bps observational)
7. Price-movement baseline comparison (sign-shuffle)

Authority 0%. No OF/execution changes. Stage 2 not authorized.
"""

from __future__ import annotations

import argparse
import json
import math
import random
from collections import Counter, defaultdict
from datetime import datetime, timezone, timedelta
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence, Tuple

IST = timezone(timedelta(hours=5, minutes=30))
HORIZONS = ("1000", "5000", "15000", "30000")


def pctile(xs: Sequence[float], p: float) -> Optional[float]:
    if not xs:
        return None
    ys = sorted(xs)
    idx = min(len(ys) - 1, max(0, int(round((p / 100.0) * (len(ys) - 1)))))
    return ys[idx]


def summarize_dist(xs: Sequence[float]) -> Dict[str, Any]:
    if not xs:
        return {"n": 0}
    return {
        "n": len(xs),
        "mean": round(sum(xs) / len(xs), 6),
        "p10": round(pctile(xs, 10) or 0.0, 6),
        "p25": round(pctile(xs, 25) or 0.0, 6),
        "p50": round(pctile(xs, 50) or 0.0, 6),
        "p75": round(pctile(xs, 75) or 0.0, 6),
        "p90": round(pctile(xs, 90) or 0.0, 6),
        "p95": round(pctile(xs, 95) or 0.0, 6),
        "p99": round(pctile(xs, 99) or 0.0, 6),
        "min": round(min(xs), 6),
        "max": round(max(xs), 6),
    }


def load_events(path: Path) -> List[Dict[str, Any]]:
    rows: List[Dict[str, Any]] = []
    with path.open(encoding="utf-8") as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            try:
                rows.append(json.loads(line))
            except json.JSONDecodeError:
                continue
    return rows


def audit(events: List[Dict[str, Any]], *, flat_bps: float = 2.0, seed: int = 42) -> Dict[str, Any]:
    n = len(events)
    # 1. denominator + dedup
    keys = [(e.get("symbol"), e.get("event_receipt_utc_ms")) for e in events]
    key_counts = Counter(keys)
    n_dup_keys = sum(1 for _, c in key_counts.items() if c > 1)
    n_dup_rows = sum(c - 1 for _, c in key_counts.items() if c > 1)
    # near-dup: same symbol + receipt within 0ms already covered; also consecutive identical mid0+label
    near = 0
    by_sym_sorted: Dict[str, List[Dict[str, Any]]] = defaultdict(list)
    for e in events:
        by_sym_sorted[str(e.get("symbol") or "")].append(e)
    for sym, rows in by_sym_sorted.items():
        rows.sort(key=lambda r: int(r.get("event_receipt_utc_ms") or 0))
        for a, b in zip(rows, rows[1:]):
            ta = int(a.get("event_receipt_utc_ms") or 0)
            tb = int(b.get("event_receipt_utc_ms") or 0)
            if 0 < tb - ta <= 50:
                aa = a.get("aggression") or {}
                bb = b.get("aggression") or {}
                if aa.get("label") == bb.get("label") and aa.get("mid") == bb.get("mid") and aa.get("ltq") == bb.get("ltq"):
                    near += 1

    # 2. aggression assignment
    labels = Counter((e.get("aggression") or {}).get("label") for e in events)
    conf = Counter((e.get("aggression") or {}).get("confidence") for e in events)
    update_class = Counter((e.get("aggression") or {}).get("update_class") for e in events)
    ambiguous_in_stage1 = labels.get("AMBIGUOUS", 0) + labels.get("UNAVAILABLE", 0)

    # 3. magnitude distributions
    mag: Dict[str, Any] = {}
    for h in HORIZONS:
        raw: List[float] = []
        absv: List[float] = []
        signed: List[float] = []
        classes = Counter()
        for e in events:
            resp = (e.get("price_response") or {}).get(h) or {}
            classes[resp.get("response_class") or "MISSING"] += 1
            if not resp.get("forward_available"):
                continue
            rb = resp.get("return_bps")
            sb = resp.get("signed_return_bps")
            if rb is not None:
                raw.append(float(rb))
                absv.append(abs(float(rb)))
            if sb is not None:
                signed.append(float(sb))
        mag[h] = {
            "response_class_counts": dict(classes),
            "return_bps": summarize_dist(raw),
            "abs_return_bps": summarize_dist(absv),
            "signed_return_bps": summarize_dist(signed),
            "share_abs_gt_flat": round(sum(1 for x in absv if x > flat_bps) / len(absv), 4) if absv else None,
            "share_abs_gt_5bps": round(sum(1 for x in absv if x > 5.0) / len(absv), 4) if absv else None,
            "share_abs_gt_10bps": round(sum(1 for x in absv if x > 10.0) / len(absv), 4) if absv else None,
        }

    # 4–5. coverage + concentration
    sym_counts = Counter(str(e.get("symbol") or "UNKNOWN") for e in events)
    minute_counts: Counter = Counter()
    hour_counts: Counter = Counter()
    receipts = []
    for e in events:
        ms = int(e.get("event_receipt_utc_ms") or 0)
        if ms <= 0:
            continue
        receipts.append(ms)
        dt = datetime.fromtimestamp(ms / 1000.0, tz=timezone.utc).astimezone(IST)
        minute_counts[dt.strftime("%Y-%m-%d %H:%M")] += 1
        hour_counts[dt.strftime("%Y-%m-%d %H")] += 1

    def top_share(counter: Counter, k: int) -> Dict[str, Any]:
        top = counter.most_common(k)
        tot = sum(counter.values()) or 1
        return {
            "top": [{ "key": a, "n": b, "share": round(b / tot, 4)} for a, b in top],
            f"top{k}_share": round(sum(b for _, b in top) / tot, 4),
            "n_keys": len(counter),
            "hhi": round(sum((c / tot) ** 2 for c in counter.values()), 6),
        }

    # 6. flat band
    flat_band = {
        "flat_bps": flat_bps,
        "role": "OBSERVATIONAL_CLASSIFICATION_BOUNDARY",
        "not": "TUNED_TRADING_THRESHOLD",
        "definition": (
            f"|mid return| < {flat_bps} bps ⇒ FLAT; "
            "direction classes only when |return| ≥ flat_bps"
        ),
    }

    # 7. baseline: sign-shuffle of absolute returns at 5s / 15s
    rng = random.Random(seed)
    baseline = {}
    for h in ("5000", "15000", "30000"):
        abs_rets = []
        true_signed = []
        for e in events:
            resp = (e.get("price_response") or {}).get(h) or {}
            if not resp.get("forward_available"):
                continue
            rb = resp.get("return_bps")
            sb = resp.get("signed_return_bps")
            if rb is None or sb is None:
                continue
            abs_rets.append(abs(float(rb)))
            true_signed.append(float(sb))
        if not abs_rets:
            baseline[h] = {"available": False}
            continue
        true_mean = sum(true_signed) / len(true_signed)
        # null: random ± on same magnitudes
        null_means = []
        for _ in range(200):
            s = sum(a * rng.choice((-1.0, 1.0)) for a in abs_rets) / len(abs_rets)
            null_means.append(s)
        null_means.sort()
        # two-sided empirical p: how often |null| >= |true|
        p = sum(1 for m in null_means if abs(m) >= abs(true_mean)) / len(null_means)
        baseline[h] = {
            "available": True,
            "n": len(true_signed),
            "observed_mean_signed_bps": round(true_mean, 6),
            "null_sign_shuffle_mean_abs_bps_p50": round(pctile([abs(m) for m in null_means], 50) or 0.0, 6),
            "null_sign_shuffle_mean_signed_p2_5": round(pctile(null_means, 2.5) or 0.0, 6),
            "null_sign_shuffle_mean_signed_p97_5": round(pctile(null_means, 97.5) or 0.0, 6),
            "empirical_two_sided_p_abs_mean": round(p, 4),
            "interpretation": (
                "Reject random-sign null at ~5% if empirical_two_sided_p_abs_mean < 0.05; "
                "still not a trading edge or Stage-2 authorization."
            ),
        }

    span = None
    if receipts:
        span = [
            datetime.fromtimestamp(min(receipts) / 1000.0, tz=timezone.utc).isoformat(),
            datetime.fromtimestamp(max(receipts) / 1000.0, tz=timezone.utc).isoformat(),
        ]

    # verdict helpers
    top10_sym = top_share(sym_counts, 10)
    top10_min = top_share(minute_counts, 10)
    short_window = len(minute_counts) < 30
    # Minute concentration is only meaningful on multi-period samples.
    concentration_flag = top10_sym["top10_share"] >= 0.40 or (
        (not short_window) and top10_min["top10_share"] >= 0.40
    )

    return {
        "protocol_id": "RP-608-OFX",
        "phase": "STAGE_1_RESULT_QUALITY_AUDIT",
        "generated_utc": datetime.now(timezone.utc).isoformat(),
        "authority_pct": 0.0,
        "stage1_conclusion_preserved": "DOMINANTLY_FLAT_SHORT_HORIZON",
        "stage2_authorized": False,
        "n_events_denominator": n,
        "event_span_utc": span,
        "diagnostic_1_denominator_dedup": {
            "n_events": n,
            "n_unique_symbol_receipt_keys": len(key_counts),
            "n_keys_with_duplicates": n_dup_keys,
            "n_duplicate_extra_rows": n_dup_rows,
            "near_duplicate_pairs_le_50ms_same_mid_label_ltq": near,
            "denominator_ok": n_dup_rows == 0,
        },
        "diagnostic_2_aggression_assignment": {
            "labels_in_stage1_events": dict(labels),
            "confidence": dict(conf),
            "update_class": dict(update_class),
            "ambiguous_or_unavailable_in_stage1_file": ambiguous_in_stage1,
            "ambiguous_handling": (
                "Stage 1 observer emits ONLY trade-like BUY_INITIATED_CANDIDATE / "
                "SELL_INITIATED_CANDIDATE. AMBIGUOUS and UNAVAILABLE ticks are excluded "
                "from the 24,714-event denominator (filtered at observe_event)."
            ),
            "inference_source": (
                "aggressor_inference_primary from Stage 0D (prior-quote preferred, "
                "same-packet fallback); not exchange aggressor."
            ),
        },
        "diagnostic_3_magnitude_distributions": mag,
        "diagnostic_4_coverage": {
            "n_symbols": len(sym_counts),
            "n_ist_minutes": len(minute_counts),
            "n_ist_hours": len(hour_counts),
            "symbols_top10": top10_sym,
            "ist_minutes_top10": top10_min,
            "ist_hours": dict(hour_counts),
        },
        "diagnostic_5_concentration": {
            "symbol_top10_share": top10_sym["top10_share"],
            "symbol_hhi": top10_sym["hhi"],
            "minute_top10_share": top10_min["top10_share"],
            "minute_hhi": top10_min["hhi"],
            "n_ist_minutes": len(minute_counts),
            "short_sample_window": short_window,
            "dominated_by_few_symbols_or_minutes": concentration_flag,
            "note": (
                "Symbol flag if top10 share ≥ 40%. Minute flag only if sample spans ≥30 IST minutes "
                "and top10 minute share ≥ 40%. Short windows make minute_top10_share near 1.0 by construction."
            ),
        },
        "diagnostic_6_flat_band": flat_band,
        "diagnostic_7_baseline_sign_shuffle": baseline,
        "synthesis": {
            "flat_result_likely_reflects": [],
            "does_not_establish": [
                "order_flow_has_no_value",
                "absorption_or_failed_auction",
                "of_mechanism_change",
                "stage2_authorization",
            ],
        },
        "governance": {
            "baseline_four_books_frozen": True,
            "of_execution_unchanged": True,
            "step3_plus": "HELD",
            "gate_r": "UNREACHED",
            "inspection_class": "agent_audit_of_reported_stage1_artifacts",
        },
    }


def enrich_synthesis(report: Dict[str, Any]) -> None:
    reasons = []
    mag5 = (report.get("diagnostic_3_magnitude_distributions") or {}).get("5000") or {}
    abs5 = mag5.get("abs_return_bps") or {}
    if abs5.get("p50") is not None and abs5["p50"] < report["diagnostic_6_flat_band"]["flat_bps"]:
        reasons.append("median_abs_mid_move_below_flat_band_at_5s")
    if (mag5.get("share_abs_gt_flat") or 0) < 0.25:
        reasons.append("minority_of_events_exceed_flat_band_at_5s")
    b5 = (report.get("diagnostic_7_baseline_sign_shuffle") or {}).get("5000") or {}
    p5 = b5.get("empirical_two_sided_p_abs_mean")
    mean5 = b5.get("observed_mean_signed_bps")
    if b5.get("available") and p5 is not None and p5 >= 0.05:
        reasons.append("mean_signed_response_not_distinct_from_sign_shuffle_null_at_5s")
    elif b5.get("available") and p5 is not None and p5 < 0.05:
        if mean5 is not None and abs(float(mean5)) < 1.0:
            reasons.append(
                "mean_signed_response_statistically_nonzero_vs_sign_shuffle_but_economically_tiny_lt_1bps"
            )
        else:
            reasons.append("mean_signed_response_statistically_nonzero_vs_sign_shuffle")
    d5 = report.get("diagnostic_5_concentration") or {}
    if d5.get("short_sample_window"):
        reasons.append("short_intraday_window_limits_time_diversification")
    if d5.get("dominated_by_few_symbols_or_minutes"):
        reasons.append("coverage_concentration_present_interpret_with_care")
    elif not d5.get("short_sample_window"):
        reasons.append("not_obviously_dominated_by_top10_symbols_or_minutes")
    if (d5.get("symbol_top10_share") or 0) < 0.40:
        reasons.append("symbol_coverage_reasonably_dispersed_top10_lt_40pct")
    if report.get("diagnostic_1_denominator_dedup", {}).get("denominator_ok"):
        reasons.append("event_key_denominator_unique_symbol_receipt")
    report["synthesis"]["flat_result_likely_reflects"] = reasons


def main() -> int:
    ap = argparse.ArgumentParser(description="RP-608-OFX Stage 1 result-quality audit")
    ap.add_argument(
        "--events",
        type=Path,
        default=Path("artifacts/rp608_ofx/stage1_aggression_response_events.jsonl"),
    )
    ap.add_argument(
        "--out",
        type=Path,
        default=Path("artifacts/rp608_ofx/stage1_result_quality_audit.json"),
    )
    ap.add_argument("--flat-bps", type=float, default=2.0)
    ap.add_argument("--seed", type=int, default=42)
    args = ap.parse_args()
    events = load_events(args.events)
    report = audit(events, flat_bps=args.flat_bps, seed=args.seed)
    enrich_synthesis(report)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    # concise stdout
    d1 = report["diagnostic_1_denominator_dedup"]
    d5 = report["diagnostic_5_concentration"]
    d7 = report["diagnostic_7_baseline_sign_shuffle"]
    print(
        json.dumps(
            {
                "n_events": report["n_events_denominator"],
                "dedup_ok": d1["denominator_ok"],
                "near_dups_50ms": d1["near_duplicate_pairs_le_50ms_same_mid_label_ltq"],
                "ambiguous_in_file": report["diagnostic_2_aggression_assignment"][
                    "ambiguous_or_unavailable_in_stage1_file"
                ],
                "symbol_top10_share": d5["symbol_top10_share"],
                "minute_top10_share": d5["minute_top10_share"],
                "abs_return_5s_p50": (report["diagnostic_3_magnitude_distributions"]["5000"]["abs_return_bps"].get("p50")),
                "share_abs_gt_2bps_5s": report["diagnostic_3_magnitude_distributions"]["5000"].get("share_abs_gt_flat"),
                "baseline_5s_p": (d7.get("5000") or {}).get("empirical_two_sided_p_abs_mean"),
                "baseline_5s_mean_signed": (d7.get("5000") or {}).get("observed_mean_signed_bps"),
                "synthesis": report["synthesis"]["flat_result_likely_reflects"],
                "stage2_authorized": False,
                "out": str(args.out),
            },
            indent=2,
        )
    )
    print(f"WROTE {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
