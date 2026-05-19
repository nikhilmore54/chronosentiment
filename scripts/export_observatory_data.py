"""
Export replay log data into structured JSON for the Observatory Demo.
Parses TELEMETRY, AUDIT_TRADE, REC_STATUS, and PAPER_SUMMARY lines.
"""
import re, json, sys
from collections import defaultdict

log_file = sys.argv[1] if len(sys.argv) > 1 else "archive/replay_1m_gen11.log"
out_file = sys.argv[2] if len(sys.argv) > 2 else "observatory/data.json"

telemetry_buffer = None
trade_telemetry = {}   # rec_id -> telemetry dict
trades = []            # Finalized trades with full lifecycle
all_telemetry = []     # Every telemetry event (for timeline)

print(f"Exporting observatory data from {log_file}...")

with open(log_file, "r") as f:
    for line in f:
        # --- TELEMETRY ---
        if "[TELEMETRY]" in line:
            m_sym = re.search(r"sym=([A-Z\-]+)", line)
            m_sig = re.search(r"sig=(\w+)", line)
            m_margin = re.search(r"margin=([-\d\.]+)", line)
            m_conv = re.search(r"conv=([-\d\.]+)", line)

            m_atlas = re.search(
                r"atlas_eff=([-\d\.]+)\s+atlas_den=([-\d\.]+)\s+atlas_res=([-\d\.]+)\s+"
                r"shadow_fert=([-\d\.]+)\s+atlas_age=(\d+)", line
            )
            m_genesis = re.search(
                r"genesis_comp=([-\d\.]+)\s+genesis_range=([-\d\.]+)\s+genesis_bias=([-\d\.]+)", line
            )

            if m_sym and m_atlas:
                entry = {
                    "sym": m_sym.group(1),
                    "sig": m_sig.group(1) if m_sig else "?",
                    "margin": float(m_margin.group(1)) if m_margin else 0,
                    "conv": float(m_conv.group(1)) if m_conv else 0,
                    "eff": float(m_atlas.group(1)),
                    "den": float(m_atlas.group(2)),
                    "res": float(m_atlas.group(3)),
                    "fert": float(m_atlas.group(4)),
                    "age": int(m_atlas.group(5)),
                }
                if m_genesis:
                    entry["comp"] = float(m_genesis.group(1))
                    entry["range"] = float(m_genesis.group(2))
                    entry["bias"] = float(m_genesis.group(3))

                telemetry_buffer = entry
                all_telemetry.append(entry)

        # --- REC_STATUS ACTIVE (link telemetry to rec_id) ---
        elif "[REC_STATUS]" in line and "status=ACTIVE" in line:
            if telemetry_buffer:
                m = re.search(r"rec_id=(\d+)", line)
                if m:
                    trade_telemetry[m.group(1)] = telemetry_buffer
                telemetry_buffer = None

        # --- AUDIT_TRADE (final trade outcome) ---
        elif "[AUDIT_TRADE]" in line:
            m_id = re.search(r"rec_id=(\d+)", line)
            m_sym = re.search(r"sym=([A-Z\-]+)", line)
            m_dir = re.search(r"dir=(\w+)", line)
            m_entry = re.search(r"entry=([-\d\.]+)", line)
            m_tp = re.search(r"tp=([-\d\.]+)", line)
            m_sl = re.search(r"sl=([-\d\.]+)", line)
            m_exit = re.search(r"exit=([-\d\.]+)", line)
            m_pnl = re.search(r"realized_pnl=([-\d\.]+)", line)
            m_exit_type = re.search(r"exit_type=(\w+)", line)
            m_dur = re.search(r"dur=(\d+)", line)

            if m_id and m_pnl and m_exit_type:
                rec_id = m_id.group(1)
                pnl = float(m_pnl.group(1))
                pnl_bps = pnl * 10000

                trade = {
                    "rec_id": int(rec_id),
                    "sym": m_sym.group(1) if m_sym else "?",
                    "dir": m_dir.group(1) if m_dir else "?",
                    "entry_price": float(m_entry.group(1)) if m_entry else 0,
                    "tp": float(m_tp.group(1)) if m_tp else 0,
                    "sl": float(m_sl.group(1)) if m_sl else 0,
                    "exit_price": float(m_exit.group(1)) if m_exit else 0,
                    "pnl": pnl,
                    "pnl_bps": round(pnl_bps, 2),
                    "exit_type": m_exit_type.group(1),
                    "duration": int(m_dur.group(1)) if m_dur else 0,
                }

                # Merge telemetry if available
                if rec_id in trade_telemetry:
                    t = trade_telemetry[rec_id]
                    trade["eff"] = t["eff"]
                    trade["den"] = t["den"]
                    trade["res"] = t["res"]
                    trade["fert"] = t["fert"]
                    trade["age"] = t["age"]
                    trade["comp"] = t.get("comp", 1.0)
                    trade["range"] = t.get("range", 0.0)
                    trade["bias"] = t.get("bias", 0.0)

                trades.append(trade)

# --- Build summary statistics ---
if trades:
    winners = [t for t in trades if t["pnl_bps"] > 0]
    losers = [t for t in trades if t["pnl_bps"] <= 0]
    exit_counts = defaultdict(int)
    exit_pnl = defaultdict(float)
    for t in trades:
        exit_counts[t["exit_type"]] += 1
        exit_pnl[t["exit_type"]] += t["pnl_bps"]

    # Asset breakdown
    asset_stats = {}
    for sym in set(t["sym"] for t in trades):
        sym_trades = [t for t in trades if t["sym"] == sym]
        sym_winners = [t for t in sym_trades if t["pnl_bps"] > 0]
        asset_stats[sym] = {
            "trades": len(sym_trades),
            "win_rate": round(len(sym_winners) / len(sym_trades) * 100, 1) if sym_trades else 0,
            "total_pnl": round(sum(t["pnl_bps"] for t in sym_trades), 1),
            "avg_pnl": round(sum(t["pnl_bps"] for t in sym_trades) / len(sym_trades), 1),
        }

    # Elastic recovery ratio
    ts_count = exit_counts.get("TrailingStop", 0)
    tp_count = exit_counts.get("TakeProfit", 0)
    sl_count = exit_counts.get("StopLoss", 0)
    mo_count = exit_counts.get("Mortality", 0)
    err = (ts_count + tp_count) / max(sl_count + mo_count, 1)

    summary = {
        "total_trades": len(trades),
        "winners": len(winners),
        "losers": len(losers),
        "win_rate": round(len(winners) / len(trades) * 100, 1),
        "avg_win_bps": round(sum(t["pnl_bps"] for t in winners) / max(len(winners), 1), 1),
        "avg_loss_bps": round(sum(t["pnl_bps"] for t in losers) / max(len(losers), 1), 1),
        "total_pnl_bps": round(sum(t["pnl_bps"] for t in trades), 1),
        "expectancy_bps": round(sum(t["pnl_bps"] for t in trades) / len(trades), 1),
        "elastic_recovery_ratio": round(err, 3),
        "exit_distribution": dict(exit_counts),
        "exit_pnl": {k: round(v, 1) for k, v in exit_pnl.items()},
        "asset_stats": asset_stats,
    }
else:
    summary = {}

output = {
    "summary": summary,
    "trades": trades,
    "telemetry_count": len(all_telemetry),
}

with open(out_file, "w") as f:
    json.dump(output, f, indent=2)

print(f"Exported {len(trades)} trades, {len(all_telemetry)} telemetry events → {out_file}")
print(f"Summary: {summary.get('total_trades', 0)} trades, {summary.get('win_rate', 0)}% win rate, {summary.get('expectancy_bps', 0)} bps expectancy")
