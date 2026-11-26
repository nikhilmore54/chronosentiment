#!/usr/bin/env python3
"""One-shot: historical Estimand A post-10:45 vs pre-10:45; gate readiness."""

from __future__ import annotations

import json
from collections import defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path

IST = timezone(timedelta(hours=5, minutes=30))
ROOT = Path(
    "/home/ultraspeed/ChronoSentiment_MEGA_FINAL/live_capture/shadow_validation/nse"
)
TEST_RUN = ROOT / "test_run"


def as_float(x, default=None):
    try:
        if isinstance(x, dict):
            for k in ("net_pnl", "net", "value"):
                if k in x:
                    return float(x[k])
            return default
        return float(x)
    except Exception:
        return default


def entry_ist(r):
    ms = r.get("entry_timestamp_utc_ms")
    if ms:
        return datetime.fromtimestamp(int(ms) / 1000, tz=timezone.utc).astimezone(IST)
    for key in ("entry_timestamp_ist", "entry_ts_ist", "entry_timestamp_iso"):
        iso = r.get(key)
        if not iso:
            continue
        try:
            return datetime.fromisoformat(str(iso).replace("Z", "+00:00")).astimezone(IST)
        except Exception:
            continue
    return None


def summarize(rows, label):
    n = len(rows)
    wins = sum(1 for r in rows if r["win"] is True)
    losses = sum(1 for r in rows if r["win"] is False)
    decided = wins + losses
    wr = (wins / decided) if decided else None
    nets = [r["net"] for r in rows if r["net"] is not None]
    gross = [r["gross"] for r in rows if r["gross"] is not None]
    print(f"== {label} ==")
    print(
        f"n={n} decided={decided} W={wins} L={losses} "
        f"WR={None if wr is None else round(100 * wr, 1)}%"
    )
    if nets:
        print(f"sum_net={round(sum(nets), 2)} avg_net={round(sum(nets) / len(nets), 2)}")
    if gross:
        print(
            f"sum_gross={round(sum(gross), 2)} "
            f"avg_gross={round(sum(gross) / len(gross), 2)}"
        )
    by_day = defaultdict(list)
    for r in rows:
        by_day[r["day"]].append(r)
    print(f"days={len(by_day)}")


def load_closed(journal: Path):
    closed = []
    with journal.open() as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            try:
                r = json.loads(line)
            except json.JSONDecodeError:
                continue
            if r.get("exit_reason") is None and r.get("exit_price") is None:
                if str(r.get("status", "")).upper() == "OPEN":
                    continue
                if r.get("net_pnl") is None and r.get("gross_pnl") is None:
                    continue
            est = str(r.get("estimand") or "ESTIMAND_A").upper()
            if "OF" in est and "ESTIMAND" not in est:
                continue
            if est not in ("ESTIMAND_A", "A", "ARM_A", ""):
                if r.get("estimand") and "A" not in est:
                    continue
            ts = entry_ist(r)
            if not ts:
                continue
            hm = ts.hour * 60 + ts.minute
            net = as_float(r.get("net_pnl"), as_float(r.get("net_pnl_inr")))
            gross = as_float(r.get("gross_pnl"), as_float(r.get("gross_pnl_inr")))
            win = None
            if net is not None:
                win = net > 0
            elif gross is not None:
                win = gross > 0
            closed.append(
                {
                    "day": ts.date().isoformat(),
                    "hm": hm,
                    "post": hm >= 10 * 60 + 45,
                    "win": win,
                    "gross": gross,
                    "net": net,
                    "sym": r.get("symbol"),
                }
            )
    return closed


def main():
    candidates = [
        TEST_RUN / "rp608_nse_trade_journal.jsonl",
        ROOT / "rp608_nse_trade_journal.jsonl",
    ]
    journal = next((p for p in candidates if p.exists() and p.stat().st_size > 0), None)
    if journal is None:
        print("NO_JOURNAL")
        return
    closed = load_closed(journal)
    print("journal", journal, "bytes", journal.stat().st_size, "rows", len(closed))
    summarize(closed, "ALL_CLOSED_A_LIKE")
    summarize([r for r in closed if not r["post"]], "ENTRY_BEFORE_1045")
    summarize([r for r in closed if r["post"]], "ENTRY_AT_OR_AFTER_1045")
    today = datetime.now(IST).date().isoformat()
    td = [r for r in closed if r["day"] == today]
    summarize(td, f"TODAY_{today}_ALL")
    summarize([r for r in td if not r["post"]], f"TODAY_{today}_PRE_1045")
    summarize([r for r in td if r["post"]], f"TODAY_{today}_POST_1045")

    watch = TEST_RUN / "rp608_nse_next_session_recommendations.json"
    print("watch_exists", watch.exists())
    if watch.exists():
        doc = json.loads(watch.read_text())
        print(
            "watch_next",
            doc.get("next_session_date"),
            "from",
            doc.get("from_session_date"),
            "n",
            doc.get("n_recommendations"),
        )
    day = today.replace("-", "")
    preopen = sorted(TEST_RUN.glob(f"preopen_discovery_snapshot_{day}*.jsonl"))
    print("preopen_today", [p.name for p in preopen])

    latest = TEST_RUN / "rp608_nse_latest_evaluations.json"
    print("latest_eval_exists", latest.exists())
    if latest.exists():
        doc = json.loads(latest.read_text())
        rows = []
        if isinstance(doc, list):
            rows = doc
        elif isinstance(doc, dict):
            rows = doc.get("evaluations") or doc.get("symbols") or []
            if not rows and all(isinstance(v, dict) for v in doc.values()):
                rows = list(doc.values())
        stamped = blocked = 0
        codes = defaultdict(int)
        sample = None
        for r in rows:
            if not isinstance(r, dict):
                continue
            if "coralys_input_context" in r:
                stamped += 1
            if r.get("coralys_input_gate_blocks"):
                blocked += 1
            codes[r.get("decision_code")] += 1
            if sample is None and r.get("decision_code") == "STANDASIDE_CORALYS_INPUT_GATE":
                sample = r
        print("latest_n", len(rows), "stamped", stamped, "blocked", blocked)
        print("top_codes", sorted(codes.items(), key=lambda x: -x[1])[:12])
        if sample:
            ctx = sample.get("coralys_input_context") or {}
            print(
                "sample_standaside",
                sample.get("symbol"),
                sample.get("explanation"),
                ctx.get("standaside_reason"),
            )


if __name__ == "__main__":
    main()
