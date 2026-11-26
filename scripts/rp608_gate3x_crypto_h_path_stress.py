#!/usr/bin/env python3
"""
RP-608-GATE3X-CRYPTO-H-PATH-001 — Crypto plan H60/H120 path-stress.

Frozen entries from trade_plan_outcomes.jsonl. Authority 0%.
No live overlay / OF ±0.30 / ATR edits.

CF policies (predeclared):
  A          — production nets
  H60_OPT    — early SL (<60s) → scratch@entry − RT friction (optimistic Gate2X bound)
  H120_OPT   — early SL (<120s) → scratch@entry − RT friction
  H60_PATH   — early SL → scratch only if 1s path revisits entry±friction inside
               [entry, entry+60s]; else keep production SL net
  H120_PATH  — same with 120s window

Promote (PATH policies only): holdout Δnet>0, paired session share≥0.5,
n_path_scratch≥5, n_sessions≥3, and PATH does not worsen payoff into
W/L-only destruction of net vs A.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import time
import urllib.error
import urllib.parse
import urllib.request
from collections import defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

PROTOCOL_ID = "RP-608-GATE3X-CRYPTO-H-PATH-001"
IST = timezone(timedelta(hours=5, minutes=30))
FRICTION_BPS_SIDE = 6.0  # 6 bps / side → 12 bps RT (matches Gate2X/3X-G)
BINANCE_KLINES = "https://api.binance.com/api/v3/klines"


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def load_jsonl(path: Path) -> List[Dict[str, Any]]:
    return [json.loads(l) for l in path.open() if l.strip()]


def day_ist(ms: Any) -> str:
    try:
        return (
            datetime.fromtimestamp(int(ms) / 1000, tz=timezone.utc)
            .astimezone(IST)
            .strftime("%Y-%m-%d")
        )
    except Exception:
        return "UNKNOWN"


def chrono_split_days(days: List[str]) -> Tuple[List[str], List[str]]:
    days = sorted(d for d in days if d != "UNKNOWN")
    cut = max(2, int(math.floor(len(days) * 0.60))) if len(days) >= 4 else max(1, len(days) // 2)
    return days[:cut], days[cut:]


def crypto_fric(entry: float, exit_px: float, qty: float) -> float:
    return (entry * qty + exit_px * qty) * (FRICTION_BPS_SIDE / 10000.0)


def scratch_net(entry: float, qty: float) -> float:
    return round(0.0 - crypto_fric(entry, entry, qty), 2)


def stats(nets: List[float]) -> Dict[str, Any]:
    n = len(nets)
    wins = sum(1 for x in nets if x > 0)
    losses = sum(1 for x in nets if x < 0)
    avg_w = (sum(x for x in nets if x > 0) / wins) if wins else 0.0
    avg_l = (sum(abs(x) for x in nets if x < 0) / losses) if losses else 0.0
    payoff = (avg_w / avg_l) if avg_l > 0 else None
    return {
        "n": n,
        "W": wins,
        "L": losses,
        "wr_pct": round(100.0 * wins / n, 2) if n else None,
        "net": round(sum(nets), 4),
        "payoff": round(payoff, 4) if payoff is not None else None,
        "avg_win": round(avg_w, 4),
        "avg_loss_abs": round(avg_l, 4),
    }


def fetch_klines_1s(
    start_ms: int,
    end_ms: int,
    *,
    cache: Dict[Tuple[int, int], List[List[Any]]],
    sleep_s: float = 0.08,
) -> List[List[Any]]:
    """Fetch BTCUSDT 1s klines covering [start_ms, end_ms]. Cached by window."""
    if end_ms <= start_ms:
        return []
    key = (start_ms, end_ms)
    if key in cache:
        return cache[key]
    out: List[List[Any]] = []
    cursor = start_ms
    while cursor < end_ms:
        params = urllib.parse.urlencode(
            {
                "symbol": "BTCUSDT",
                "interval": "1s",
                "startTime": cursor,
                "endTime": end_ms,
                "limit": 1000,
            }
        )
        url = f"{BINANCE_KLINES}?{params}"
        try:
            with urllib.request.urlopen(url, timeout=20) as resp:
                batch = json.loads(resp.read().decode("utf-8"))
        except urllib.error.HTTPError as e:
            if e.code == 400:
                # 1s may be unsupported on some edges — fall back empty
                cache[key] = []
                return []
            raise
        if not batch:
            break
        out.extend(batch)
        last_open = int(batch[-1][0])
        nxt = last_open + 1000
        if nxt <= cursor:
            break
        cursor = nxt
        time.sleep(sleep_s)
        if len(batch) < 1000:
            break
    cache[key] = out
    return out


def path_revisits_entry(
    direction: str,
    entry: float,
    klines: List[List[Any]],
    *,
    friction_bps: float = FRICTION_BPS_SIDE,
) -> Dict[str, Any]:
    """
    Path rule (predeclared):
      1) Require an adverse excursion beyond entry±friction (left the scratch band).
      2) Thereafter, mid must re-enter the scratch band around entry before window end.
    Mid = (H+L)/2 per 1s bar.
    """
    if not klines:
        return {
            "ok": False,
            "reason": "NO_KLINES",
            "n_bars": 0,
            "adverse_seen": False,
            "revisit_seen": False,
        }
    band = entry * (friction_bps / 10000.0)
    adverse = False
    revisit = False
    revisit_ms = None
    for row in klines:
        o_ms = int(row[0])
        high = float(row[2])
        low = float(row[3])
        mid = 0.5 * (high + low)
        if direction == "LONG":
            if mid < entry - band:
                adverse = True
            if adverse and mid >= entry - band:
                revisit = True
                revisit_ms = o_ms
                break
        else:  # SHORT
            if mid > entry + band:
                adverse = True
            if adverse and mid <= entry + band:
                revisit = True
                revisit_ms = o_ms
                break
    return {
        "ok": bool(adverse and revisit),
        "reason": "REVISIT" if (adverse and revisit) else ("ADVERSE_ONLY" if adverse else "NO_ADVERSE"),
        "n_bars": len(klines),
        "adverse_seen": adverse,
        "revisit_seen": revisit,
        "revisit_ms": revisit_ms,
    }


def build_rows(
    trades: List[Dict[str, Any]],
    *,
    kline_cache: Dict[Tuple[int, int], List[List[Any]]],
    fetch: bool,
    fetch_allow: Optional[set] = None,
) -> List[Dict[str, Any]]:
    rows: List[Dict[str, Any]] = []
    fetched = 0
    for t in trades:
        direction = str(t.get("direction") or "").upper()
        entry = float(t["entry_price"])
        qty = float(t.get("simulated_quantity") or 0.0)
        prod = float(t["net_pnl_usdt"])
        reason = str(t.get("exit_reason") or "")
        hold = float(t.get("holding_duration_seconds") or 0.0)
        entry_ms = int(t.get("entry_time_ms") or 0)
        exit_ms = int(t.get("exit_time_ms") or entry_ms)
        sess = day_ist(entry_ms)
        early60 = reason == "STOP_LOSS_HIT" and hold < 60.0
        early120 = reason == "STOP_LOSS_HIT" and hold < 120.0
        plan_id = t.get("plan_id")

        path60: Dict[str, Any] = {"ok": False, "reason": "NOT_EARLY"}
        path120: Dict[str, Any] = {"ok": False, "reason": "NOT_EARLY"}

        do_fetch = bool(fetch and (early60 or early120))
        if do_fetch and fetch_allow is not None and plan_id not in fetch_allow:
            path60 = {"ok": False, "reason": "FETCH_SKIPPED"} if early60 else path60
            path120 = {"ok": False, "reason": "FETCH_SKIPPED"} if early120 else path120
            do_fetch = False

        if do_fetch:
            fetched += 1
            if fetched == 1 or fetched % 25 == 0:
                print(f"[path] fetching early SL #{fetched} plan={plan_id} hold={hold:.1f}s", flush=True)
            win_end = max(entry_ms + (120 if early120 else 60) * 1000, exit_ms + 1000)
            kl_all = fetch_klines_1s(entry_ms, win_end, cache=kline_cache)
            if early60:
                cut = entry_ms + 60 * 1000
                kl60 = [row for row in kl_all if int(row[0]) <= cut]
                path60 = path_revisits_entry(direction, entry, kl60)
            if early120:
                cut = entry_ms + 120 * 1000
                kl120 = [row for row in kl_all if int(row[0]) <= cut]
                path120 = path_revisits_entry(direction, entry, kl120)

        scratch = scratch_net(entry, qty)
        h60_opt = scratch if early60 else prod
        h120_opt = scratch if early120 else prod
        h60_path = scratch if (early60 and path60.get("ok")) else prod
        h120_path = scratch if (early120 and path120.get("ok")) else prod

        rows.append(
            {
                "plan_id": plan_id,
                "session_ist": sess,
                "direction": direction,
                "entry_time_ms": entry_ms,
                "exit_time_ms": exit_ms,
                "hold_s": hold,
                "reason": reason,
                "early60": early60,
                "early120": early120,
                "prod": prod,
                "h60_opt": h60_opt,
                "h120_opt": h120_opt,
                "h60_path": h60_path,
                "h120_path": h120_path,
                "path60": path60,
                "path120": path120,
                "h60_path_applied": bool(early60 and path60.get("ok")),
                "h120_path_applied": bool(early120 and path120.get("ok")),
            }
        )
    print(f"[path] done fetches={fetched} cache_windows={len(kline_cache)}", flush=True)
    return rows


def paired_share(rows: List[Dict[str, Any]], key: str) -> Dict[str, Any]:
    c_day: Dict[str, float] = defaultdict(float)
    x_day: Dict[str, float] = defaultdict(float)
    for r in rows:
        d = r["session_ist"]
        if d == "UNKNOWN":
            continue
        c_day[d] += float(r["prod"])
        x_day[d] += float(r[key])
    days = sorted(set(c_day) | set(x_day))
    ok = sum(1 for d in days if x_day[d] >= c_day[d])
    return {
        "n_sessions": len(days),
        "delta_nonneg": ok,
        "share": round(ok / len(days), 4) if days else None,
    }


def block(rows: List[Dict[str, Any]], label: str) -> Dict[str, Any]:
    out: Dict[str, Any] = {"label": label, "n": len(rows)}
    for key in ("prod", "h60_opt", "h120_opt", "h60_path", "h120_path"):
        st = stats([float(r[key]) for r in rows])
        out[f"stats_{key}"] = st
        if key != "prod":
            out[f"delta_{key}"] = round(st["net"] - out["stats_prod"]["net"], 4)
            out[f"paired_{key}"] = paired_share(rows, key)
    out["n_early60"] = sum(1 for r in rows if r["early60"])
    out["n_early120"] = sum(1 for r in rows if r["early120"])
    out["n_h60_path_scratch"] = sum(1 for r in rows if r["h60_path_applied"])
    out["n_h120_path_scratch"] = sum(1 for r in rows if r["h120_path_applied"])
    # path diagnostics among early
    early60 = [r for r in rows if r["early60"]]
    reasons = defaultdict(int)
    for r in early60:
        reasons[str(r.get("path60", {}).get("reason"))] += 1
    out["path60_reason_counts"] = dict(reasons)
    return out


def promote_path(hold_b: Dict[str, Any], key: str) -> Tuple[bool, List[str]]:
    """Return (ok, fail_reasons)."""
    fails: List[str] = []
    delta = hold_b.get(f"delta_{key}")
    paired = hold_b.get(f"paired_{key}") or {}
    n_key = "n_h60_path_scratch" if key == "h60_path" else "n_h120_path_scratch"
    n_app = int(hold_b.get(n_key) or 0)
    if delta is None or delta <= 0:
        fails.append(f"holdout_delta_{key}_not_positive")
    if (paired.get("share") or 0) < 0.5:
        fails.append(f"paired_share_{key}_lt_0.5")
    if (paired.get("n_sessions") or 0) < 3:
        fails.append("n_sessions_lt_3")
    if n_app < 5:
        fails.append(f"{n_key}_lt_5")
    st_a = hold_b["stats_prod"]
    st_x = hold_b[f"stats_{key}"]
    if st_x["net"] < st_a["net"]:
        fails.append("net_worse_than_A")
    return (len(fails) == 0, fails)


def write_md(report: Dict[str, Any], path: Path) -> None:
    hold = report["holdout"]
    lines = [
        f"# {PROTOCOL_ID} — Results",
        "",
        f"**Disposition:** `{report['disposition']}`",
        f"**Generated (UTC):** {report['generated_utc']}",
        f"**Journal:** `{report['journal_path']}` sha256=`{report['journal_sha256'][:16]}…`",
        f"**n trades:** {report['n_trades']} · discovery days {len(report['discovery_days'])} · holdout days {len(report['holdout_days'])}",
        "",
        "## Holdout (primary)",
        "",
        f"| Policy | net | Δ vs A | WR% | payoff | path scratches | paired share |",
        f"| --- | ---: | ---: | ---: | ---: | ---: | ---: |",
    ]
    for key, label in (
        ("prod", "A production"),
        ("h60_opt", "H60_OPT"),
        ("h120_opt", "H120_OPT"),
        ("h60_path", "H60_PATH"),
        ("h120_path", "H120_PATH"),
    ):
        st = hold[f"stats_{key}"]
        if key == "prod":
            lines.append(
                f"| {label} | {st['net']} | — | {st['wr_pct']} | {st['payoff']} | — | — |"
            )
        else:
            paired = hold[f"paired_{key}"]
            n_sc = hold["n_h60_path_scratch"] if "60" in key and "path" in key else (
                hold["n_h120_path_scratch"] if "path" in key else "—"
            )
            lines.append(
                f"| {label} | {st['net']} | {hold[f'delta_{key}']} | {st['wr_pct']} | {st['payoff']} | {n_sc} | {paired.get('share')} |"
            )
    lines.extend(
        [
            "",
            f"**path60 reasons (early SL, all rows in holdout block scope):** "
            f"`{hold.get('path60_reason_counts')}`",
            "",
            "## Promote checks",
            "",
            f"- H60_PATH: `{report['promote_h60_path']}` fails=`{report['promote_h60_path_fails']}`",
            f"- H120_PATH: `{report['promote_h120_path']}` fails=`{report['promote_h120_path_fails']}`",
            "",
            "## Full-sample deltas (disclosure)",
            "",
            f"- Δ H60_OPT: {report['full']['delta_h60_opt']}",
            f"- Δ H120_OPT: {report['full']['delta_h120_opt']}",
            f"- Δ H60_PATH: {report['full']['delta_h60_path']}",
            f"- Δ H120_PATH: {report['full']['delta_h120_path']}",
            "",
            "## Non-claims",
            "",
            "- No live daemon / overlay edit",
            "- No OF ±0.30 / ATR / authority change",
            "- OPT bounds are optimistic; PATH is the Gate3X stress",
            "",
        ]
    )
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    ap = argparse.ArgumentParser(description=PROTOCOL_ID)
    ap.add_argument(
        "--journal",
        type=Path,
        default=Path(
            "live_capture/shadow_validation/crypto/trade_plan_overlay_v1/trade_plan_outcomes.jsonl"
        ),
    )
    ap.add_argument(
        "--out-dir",
        type=Path,
        default=Path("artifacts/rp608_gate3x_crypto_h_path"),
    )
    ap.add_argument("--no-fetch", action="store_true", help="Skip Binance 1s fetch (OPT-only)")
    ap.add_argument("--max-early", type=int, default=0, help="Debug: cap early SL path fetches")
    args = ap.parse_args()

    journal = args.journal.resolve()
    out_dir = args.out_dir.resolve()
    out_dir.mkdir(parents=True, exist_ok=True)

    trades = load_jsonl(journal)
    kline_cache: Dict[Tuple[int, int], List[List[Any]]] = {}
    fetch = not args.no_fetch
    allow = None
    if args.max_early > 0 and fetch:
        early_ids = []
        for t in trades:
            hold = float(t.get("holding_duration_seconds") or 0)
            if t.get("exit_reason") == "STOP_LOSS_HIT" and hold < 120:
                early_ids.append(t.get("plan_id"))
        allow = set(early_ids[: args.max_early])

    rows = build_rows(trades, kline_cache=kline_cache, fetch=fetch, fetch_allow=allow)

    days = sorted({r["session_ist"] for r in rows if r["session_ist"] != "UNKNOWN"})
    disc_d, hold_d = chrono_split_days(days)
    disc_set, hold_set = set(disc_d), set(hold_d)
    full_b = block(rows, "full")
    disc_b = block([r for r in rows if r["session_ist"] in disc_set], "discovery")
    hold_b = block([r for r in rows if r["session_ist"] in hold_set], "holdout")

    ok60, fail60 = promote_path(hold_b, "h60_path")
    ok120, fail120 = promote_path(hold_b, "h120_path")

    if ok60 or ok120:
        disposition = "PASS_GATE3X_DRAFT_GATE5_H_PACKET"
        if ok60 and ok120:
            disposition = "PASS_GATE3X_DRAFT_GATE5_H_PACKET_BOTH"
        elif ok60:
            disposition = "PASS_GATE3X_DRAFT_GATE5_H60_ONLY"
        else:
            disposition = "PASS_GATE3X_DRAFT_GATE5_H120_ONLY"
    elif hold_b["delta_h60_opt"] > 0 and (hold_b["paired_h60_opt"]["share"] or 0) >= 0.5:
        disposition = "DO_NOT_PROMOTE_PATH_FAILS_OPTIMISTIC_ONLY"
    else:
        disposition = "DO_NOT_PROMOTE"

    report = {
        "protocol_id": PROTOCOL_ID,
        "generated_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "authority_pct": 0.0,
        "journal_path": str(journal),
        "journal_sha256": sha256_file(journal),
        "n_trades": len(trades),
        "fetch_1s": fetch,
        "kline_windows_cached": len(kline_cache),
        "discovery_days": disc_d,
        "holdout_days": hold_d,
        "full": full_b,
        "discovery": disc_b,
        "holdout": hold_b,
        "promote_h60_path": ok60,
        "promote_h60_path_fails": fail60,
        "promote_h120_path": ok120,
        "promote_h120_path_fails": fail120,
        "disposition": disposition,
        "non_goals": [
            "no_live_overlay_edit",
            "no_OF_pm_30_change",
            "no_ATR_change",
            "no_authority_raise",
        ],
    }

    json_path = out_dir / "gate3x_crypto_h_path_results.json"
    md_path = out_dir / "gate3x_crypto_h_path_results.md"
    rows_path = out_dir / "gate3x_crypto_h_path_rows.jsonl"
    json_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    write_md(report, md_path)
    with rows_path.open("w", encoding="utf-8") as f:
        for r in rows:
            # slim path blobs
            slim = {k: v for k, v in r.items()}
            f.write(json.dumps(slim) + "\n")

    print(json.dumps({"disposition": disposition, "out": str(out_dir), "holdout_delta_h60_path": hold_b["delta_h60_path"], "holdout_delta_h120_path": hold_b["delta_h120_path"], "n_h60_path_scratch": hold_b["n_h60_path_scratch"], "n_h120_path_scratch": hold_b["n_h120_path_scratch"], "path60_reasons": hold_b["path60_reason_counts"]}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
