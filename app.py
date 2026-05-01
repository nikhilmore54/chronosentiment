"""
ChronoSentiment demo: combined live experiment registry + engine diagnostics.

Read-only, deterministic data sources only (.cursor/rules/chronosentiment-core.mdc).
"""

from __future__ import annotations

import html
import json
import importlib
import re
import statistics
import sys
import time
from collections import defaultdict
from datetime import datetime
from pathlib import Path
from typing import Any

import pandas as pd
import streamlit as st

st_autorefresh = None
try:  # pragma: no cover - optional dependency
    _autorefresh_mod = importlib.import_module("streamlit_autorefresh")
    st_autorefresh = getattr(_autorefresh_mod, "st_autorefresh", None)
except Exception:
    st_autorefresh = None

_ROOT = Path(__file__).resolve().parent
if str(_ROOT) not in sys.path:
    sys.path.insert(0, str(_ROOT))

from scripts.short_side_diagnostic_runner import (  # noqa: E402
    collect_diagnostic_logs,
    run_diagnostics,
)

REGISTRY_PATH = "data/experiments.jsonl"
DEFAULT_DIAG_DIR = "analysis/awr_grid"
DEFAULT_DIAG_GLOB = "live_run.log"
LIVE_MULTI_DIAG_DIR = "analysis/live_multi"
LIVE_MULTI_DIAG_GLOB = "live_*.log*"
REPLAY_DIAG_DIR = "analysis/awr_grid"
REPLAY_DIAG_GLOB = "diag_final_distribution_limit400_offset*.log"
# Frozen replay snapshot for regression reference (see scripts/snapshot_baseline.sh).
BASELINE_V1_DIR = "analysis/baselines/baseline_v1"
BASELINE_V1_DIAG_DIR = f"{BASELINE_V1_DIR}/diag_logs"
BASELINE_V1_REGISTRY = f"{BASELINE_V1_DIR}/experiments.jsonl"
BASELINE_V1_METADATA = f"{BASELINE_V1_DIR}/metadata.json"

st.set_page_config(
    page_title="ChronoSentiment — Live + Diagnostics",
    layout="wide",
    initial_sidebar_state="expanded",
)

OFFSET_RE = re.compile(r"offset(\d+)", re.IGNORECASE)
SYMBOL_TS_LINE_RE = re.compile(r"\[SYMBOL_TS\]\s+(.+)$")
SYMBOL_LOG_RE = re.compile(r"^live_([A-Z0-9]+(?:_[A-Z0-9]+)+)\.log(?:_[AB])?$")
# blue_green_log_writer.py uses live_{stem}_A.log, not live_{stem}.log_A
SYMBOL_BG_STEM_RE = re.compile(r"^live_(.+)_([AB])\.log$")
PAPER_SKETCH_FILL_TAG = "[PAPER_SKETCH_FILL]"
PAPER_SKETCH_EXIT_TAG = "[PAPER_SKETCH_EXIT]"


def infer_default_live_diag_paths() -> tuple[str, str]:
    """Prefer multi-lane log dir when present (read-only check; .cursor/rules/chronosentiment-core.mdc)."""
    live_multi = _ROOT / LIVE_MULTI_DIAG_DIR
    if live_multi.is_dir():
        return (LIVE_MULTI_DIAG_DIR, LIVE_MULTI_DIAG_GLOB)
    return (DEFAULT_DIAG_DIR, DEFAULT_DIAG_GLOB)


def extract_offset_label(filename: str) -> str:
    m = OFFSET_RE.search(filename)
    return m.group(1) if m else filename


@st.cache_data(ttl=5)
def load_registry(path: str) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    registry = Path(path)
    if not registry.exists():
        return records
    with registry.open("r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                records.append(json.loads(line))
            except json.JSONDecodeError:
                continue
    return records


@st.cache_data(ttl=5)
def cached_run_diagnostics(paths_tuple: tuple[str, ...]) -> dict[str, Any]:
    """Stable cache key: tuple of resolved paths."""
    return run_diagnostics(list(paths_tuple))


def latest_per_hypothesis(records: list[dict[str, Any]]) -> list[dict[str, Any]]:
    latest: dict[str, dict[str, Any]] = {}
    for record in records:
        hypothesis_id = record.get("hypothesis_id")
        ts_raw = record.get("timestamp")
        if not hypothesis_id or not isinstance(ts_raw, str) or not ts_raw:
            continue
        try:
            ts = datetime.fromisoformat(ts_raw.replace("Z", "+00:00"))
        except ValueError:
            continue
        prev = latest.get(hypothesis_id)
        if prev is None or ts > prev["_ts"]:
            row = dict(record)
            row["_ts"] = ts
            latest[hypothesis_id] = row
    return list(latest.values())


def to_dataframe(records: list[dict[str, Any]]) -> pd.DataFrame:
    rows: list[dict[str, Any]] = []
    for record in records:
        summary = record.get("batch_summary", {})
        rows.append(
            {
                "hypothesis_id": record.get("hypothesis_id"),
                "decision": record.get("decision"),
                "confidence": record.get("confidence"),
                "state": record.get("state", "active"),
                "avg_pnl_delta": summary.get("avg_delta_avg_pnl", 0.0),
                "hit_rate_delta": summary.get("avg_delta_hit_rate", 0.0),
                "drawdown_delta": summary.get("avg_delta_max_dd", 0.0),
                "retained_pct": summary.get("avg_retained_pct", 0.0),
                "positive_ratio": summary.get("positive_ratio", 0.0),
            }
        )
    return pd.DataFrame(rows)


def inject_page_style() -> None:
    st.markdown(
        """
<style>
/* Investor-grade dark shell (read-only UI; .cursor/rules/chronosentiment-core.mdc) */
html, body, [class*="css"] {
    font-family: -apple-system, BlinkMacSystemFont, "Inter", "Segoe UI", Roboto, sans-serif;
}
html, body, [data-testid="stAppViewContainer"] {
    background-color: #0b0f19;
    color: #e5e7eb;
}
section[data-testid="stSidebar"] {
    background-color: #111827;
    border-right: 1px solid #1f2937;
}
.block-container {
    padding-top: 1.5rem;
    padding-bottom: 1rem;
    background-color: #0b0f19;
    max-width: 100%;
}
h1, h2, h3, h4, h5, h6 {
    color: #f9fafb !important;
}
.stMarkdown p, .stMarkdown li, label, span[data-testid] {
    color: #e5e7eb;
}
/* Cards */
.cs-card {
    background: #111827;
    border: 1px solid #1f2937;
    border-radius: 12px;
    padding: 14px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.4);
    text-align: center;
    margin-bottom: 10px;
}
.cs-header {
    font-size: 12px;
    font-weight: 500;
    color: #9ca3af;
    margin-bottom: 6px;
}
.cs-value {
    font-size: 20px;
    font-weight: 600;
    color: #f9fafb;
}
.cs-subtle {
    font-size: 12px;
    color: #6b7280;
}
.cs-section {
    margin-top: 18px;
    margin-bottom: 8px;
    font-size: 16px;
    font-weight: 600;
    color: #e5e7eb;
}
[data-testid="stMetricValue"] {
    color: #f9fafb !important;
}
[data-testid="stMetricLabel"] label {
    color: #9ca3af !important;
}
[data-testid="stProgress"] > div {
    background-color: #1f2937 !important;
}
[data-testid="stProgress"] > div > div {
    background: linear-gradient(90deg, #2563eb, #60a5fa) !important;
}
/* Data frames */
[data-testid="stDataFrame"] {
    background-color: #111827;
    border-radius: 8px;
    border: 1px solid #1f2937;
}
[data-testid="stDataFrame"] div {
    color: #e5e7eb !important;
}
[data-testid="stDataFrame"] th {
    background-color: #1f2937 !important;
}
[data-testid="stDataFrame"] td {
    background-color: #111827 !important;
}
/* Section rule */
.cs-rule {
    border: 0;
    border-top: 1px solid #1f2937;
    margin: 12px 0 16px 0;
}
.streamlit-expanderHeader {
    background-color: #111827 !important;
    color: #e5e7eb !important;
    border-radius: 8px;
}
</style>
""",
        unsafe_allow_html=True,
    )


def metric_card(label: str, value: str) -> None:
    st.markdown(
        f"""
    <div class="cs-card">
        <div class="cs-header">{label}</div>
        <div class="cs-value">{value}</div>
    </div>
    """,
        unsafe_allow_html=True,
    )


def section_header(title: str) -> None:
    st.markdown(
        f"""
    <div class="cs-section">{title}</div>
    """,
        unsafe_allow_html=True,
    )


def recommendation_to_ui_mode(rec: str) -> str:
    if rec == "KEEP_LONG_ONLY":
        return "LONG_ONLY"
    if rec == "CONSIDER_SHADOW_EPSILON_MAPPING":
        return "MONITORING"
    if rec == "CHECK_DIRECTIONAL_MAPPING":
        return "READY"
    if rec == "SHORTS_OBSERVED_REVIEW_GATES":
        return "SHORT_ACTIVE"
    if rec == "PENDING_STREAM_SUMMARY":
        return "MONITORING"
    return "UNKNOWN"


def render_global_status_bar(
    rec: str,
    near_n: int,
    bear_n: int,
    n_sl: int,
) -> None:
    """Premium full-width status strip (identity layer)."""
    mode = recommendation_to_ui_mode(rec)
    color_map = {
        "LONG_ONLY": "#16a34a",
        "MONITORING": "#f59e0b",
        "READY": "#2563eb",
        "SHORT_ACTIVE": "#dc2626",
        "UNKNOWN": "#6b7280",
    }
    icon_map = {
        "LONG_ONLY": "🟢",
        "MONITORING": "🟡",
        "READY": "🔵",
        "SHORT_ACTIVE": "🔴",
        "UNKNOWN": "⚪",
    }
    color = color_map.get(mode, "#6b7280")
    label_map = {
        "KEEP_LONG_ONLY": "LONG-ONLY MODE (VALIDATED)",
        "CONSIDER_SHADOW_EPSILON_MAPPING": "MONITORING BEARISH EMERGENCE",
        "CHECK_DIRECTIONAL_MAPPING": "READY FOR SHORT-SIDE MAPPING REVIEW",
        "SHORTS_OBSERVED_REVIEW_GATES": "SHORT ACTIVITY DETECTED — REVIEW GATES",
        "PENDING_STREAM_SUMMARY": "LIVE STREAM ACTIVE — DIAGNOSTIC SUMMARY PENDING",
    }
    headline = label_map.get(rec, f"STATUS: {rec}")
    st.markdown(
        f"""
    <div style="
        padding: 14px 18px;
        border-radius: 12px;
        background: linear-gradient(90deg, {color}22, #111827);
        border: 1px solid {color}44;
        margin-bottom: 12px;
        box-shadow: 0 2px 12px rgba(0,0,0,0.35);
    ">
        <b style="color:{color}; font-size:16px;">{icon_map.get(mode, "⚪")} {headline}</b>
        <span style="margin-left:12px; color:#9ca3af; font-size:14px;">
        Slices: {n_sl} · Near-bearish offsets: {near_n} · Bearish offsets: {bear_n}
        </span>
    </div>
    """,
        unsafe_allow_html=True,
    )


def auto_explanation(summary: dict[str, Any]) -> str:
    bear = int(summary.get("offsets_with_bearish_events", 0) or 0)
    near = int(summary.get("offsets_with_near_bearish", 0) or 0)
    if bear == 0:
        return "No bearish structure detected across slices"
    if near < 3:
        return "Weak bearish pressure, below activation threshold"
    return "Emerging bearish structure — monitoring"


def rows_sell_activity(rows: list[dict[str, Any]]) -> tuple[int, int]:
    """Aggregate sell signals from parsed rows (live slice feed)."""
    t_cand = 0
    t_final = 0
    for r in rows:
        sd = r["side_distribution"]
        t_cand += int(sd["candidates_sell"])
        t_final += int(sd["final_sell"])
    return t_cand, t_final


def live_consistency_block(summary: dict[str, Any], rows: list[dict[str, Any]]) -> None:
    """Summary vs row-level recompute — self-validating dashboard."""
    st.markdown("##### Live consistency check")
    if not rows:
        st.caption("No diagnostic slices loaded — run `live_engine` logs or adjust path/glob.")
        return

    s_cand = int(summary.get("offsets_with_sell_candidates", 0) or 0)
    s_fin = int(summary.get("offsets_with_final_sell", 0) or 0)
    off_cand = sum(1 for r in rows if r["side_distribution"]["candidates_sell"] > 0)
    off_fin = sum(1 for r in rows if r["side_distribution"]["final_sell"] > 0)
    t_cand, t_final = rows_sell_activity(rows)

    expected_no_sell = s_cand == 0 and s_fin == 0
    actual_no_sell = t_cand == 0 and t_final == 0

    match_agg = s_cand == off_cand and s_fin == off_fin

    if expected_no_sell and actual_no_sell and match_agg:
        st.success("Live behavior consistent with diagnostics")
    elif not match_agg:
        st.error(
            "⚠ Summary vs parsed slices mismatch — re-run parse or check stale logs "
            f"(offsets cand/FINAL: summary {s_cand}/{s_fin} vs rows {off_cand}/{off_fin})."
        )
    elif expected_no_sell and not actual_no_sell:
        st.error("⚠ Unexpected SELL activity in slices — investigate gates and logs")
    else:
        st.warning("ℹ Mixed behavior — monitor short-side signals and registry")


def compute_phase(summary: dict[str, Any]) -> str:
    if int(summary.get("offsets_with_sell_candidates", 0) or 0) > 0:
        return "SHORT_ACTIVE"
    if int(summary.get("offsets_with_final_sell", 0) or 0) > 0:
        return "SHORT_ACTIVE"
    nb = int(summary.get("offsets_with_near_bearish", 0) or 0)
    if nb >= 4:
        return "READY_FOR_MAPPING"
    if nb > 0:
        return "MONITORING"
    return "LONG_ONLY"


def section_rule() -> None:
    st.markdown("<hr class='cs-rule'>", unsafe_allow_html=True)


def resolve_log_paths(
    mode: str,
    live_dir: str,
    live_pattern: str,
    max_files: int,
) -> list[Path]:
    """Deterministic log source switch (read-only)."""
    if mode == "Replay":
        paths = collect_diagnostic_logs(REPLAY_DIAG_DIR, REPLAY_DIAG_GLOB)
    elif mode == "Baseline":
        paths = collect_diagnostic_logs(BASELINE_V1_DIAG_DIR, REPLAY_DIAG_GLOB)
    else:
        paths = collect_diagnostic_logs(live_dir, live_pattern)
    n = max(1, min(int(max_files), len(paths))) if paths else 0
    return paths[-n:] if n > 0 else []


def extract_symbol_from_path(path: Path) -> str | None:
    """Extract validated symbol from lane log filename.

    Accepts:
      live_BTC_USD.log
      live_BTC_USD.log_A
      live_BTC_USD.log_B
      live_BTC_USD_A.log / live_BTC_USD_B.log (blue/green stem variants)
    Rejects aggregate files like live_run.log.
    """
    name = path.name
    m = SYMBOL_BG_STEM_RE.match(name)
    if m:
        return m.group(1).replace("_", "-")
    m = SYMBOL_LOG_RE.match(name)
    if not m:
        return None
    return m.group(1).replace("_", "-")


def resolve_latest_log_per_symbol(paths: list[Path]) -> dict[str, Path]:
    grouped: dict[str, list[Path]] = defaultdict(list)
    for p in paths:
        if not p.exists() or not p.is_file():
            continue
        symbol = extract_symbol_from_path(p)
        if symbol is None:
            continue
        grouped[symbol].append(p)
    latest: dict[str, Path] = {}
    for symbol, items in grouped.items():
        latest[symbol] = max(items, key=lambda x: x.stat().st_mtime)
    return latest


def summarize_multi_symbol(multi_results: dict[str, dict[str, Any]]) -> dict[str, Any]:
    summaries = [r.get("summary", {}) for r in multi_results.values() if r.get("summary")]
    n = len(summaries)
    near_offsets = sum(1 for s in summaries if int(s.get("offsets_with_near_bearish", 0) or 0) > 0)
    bearish_offsets = sum(1 for s in summaries if int(s.get("offsets_with_bearish_events", 0) or 0) > 0)
    sell_cand_offsets = sum(1 for s in summaries if int(s.get("offsets_with_sell_candidates", 0) or 0) > 0)
    sell_final_offsets = sum(1 for s in summaries if int(s.get("offsets_with_final_sell", 0) or 0) > 0)
    if sell_final_offsets > 0 or sell_cand_offsets > 0:
        recommendation = "SHORTS_OBSERVED_REVIEW_GATES"
    elif bearish_offsets > 0:
        recommendation = "CHECK_DIRECTIONAL_MAPPING"
    elif near_offsets >= max(1, n // 2):
        recommendation = "CONSIDER_SHADOW_EPSILON_MAPPING"
    else:
        recommendation = "KEEP_LONG_ONLY"
    return {
        "files_parsed": n,
        "offsets_with_near_bearish": near_offsets,
        "offsets_with_bearish_events": bearish_offsets,
        "offsets_with_sell_candidates": sell_cand_offsets,
        "offsets_with_final_sell": sell_final_offsets,
        "recommendation": recommendation,
    }


def run_multi_symbol_diagnostics(log_dir: str, pattern: str, max_files: int) -> dict[str, dict[str, Any]]:
    base_paths = resolve_log_paths("Live", log_dir, pattern, max_files)
    latest_by_symbol = resolve_latest_log_per_symbol(base_paths)
    out: dict[str, dict[str, Any]] = {}
    for symbol, p in latest_by_symbol.items():
        result = cached_run_diagnostics((str(p.resolve()),))
        # Skip empty/partial rotating files; keep UI deterministic and clean.
        if not result.get("rows") and not result.get("summary"):
            continue
        out[symbol] = {"source": str(p), **result}
    return out


def extract_symbol_ts_from_log_files(paths: list[Path]) -> dict[str, float]:
    """Extract symbol freshness from live log lines.

    Uses file mtime as freshness timestamp (feed recency), not candle timestamp.
    """
    latest: dict[str, float] = {}
    for p in paths:
        if not p.exists() or not p.is_file():
            continue
        try:
            file_seen_ts = float(p.stat().st_mtime)
        except OSError:
            file_seen_ts = 0.0
        try:
            with p.open("r", encoding="utf-8") as f:
                for line in f:
                    m = SYMBOL_TS_LINE_RE.search(line)
                    if not m:
                        continue
                    for part in m.group(1).split(","):
                        part = part.strip()
                        if ":" not in part:
                            continue
                        sym, _ts_str = part.rsplit(":", 1)
                        sym = sym.strip()
                        if not sym:
                            continue
                        prev = latest.get(sym)
                        if prev is None or file_seen_ts > prev:
                            latest[sym] = file_seen_ts
        except OSError:
            continue
    return latest


TRADE_SKETCH_SL_FRAC = 0.8
TRADE_SKETCH_TP_FRAC = 1.5
TRADE_SKETCH_MAX_JSON_BATCHES = 5
TRADE_BATCH_MAX_GAP_SEC = 10.0
TRADE_MIN_RISK_FRAC = 0.0005
TRADE_TAIL_BYTES = 64 * 1024
TRADE_MEDIAN_CLOSES = 3
TRADE_ENGINE_RISK_WINDOW = 5

SYMBOL_PRICE_LINE_RE = re.compile(r"\[SYMBOL_PRICE\]\s+(.+)$")


def read_log_tail_text(path: Path, max_bytes: int = TRADE_TAIL_BYTES) -> str:
    try:
        with path.open("rb") as f:
            f.seek(0, 2)
            sz = f.tell()
            f.seek(max(0, sz - max_bytes), 0)
            return f.read().decode("utf-8", errors="ignore")
    except OSError:
        return ""


def _parse_stream_json_batch_line(line: str) -> list[dict[str, Any]] | None:
    s = line.strip()
    if not s.startswith("["):
        return None
    try:
        v = json.loads(s)
    except json.JSONDecodeError:
        return None
    if not isinstance(v, list) or not v or not isinstance(v[0], dict):
        return None
    if "symbol" not in v[0] or "close" not in v[0]:
        return None
    return v


def stream_symbols_from_log_tail(log_path: Path, max_lines: int = 4000) -> list[str]:
    """Symbols appearing in recent JSON stream batches (tail scan)."""
    text = read_log_tail_text(log_path)
    seen: list[str] = []
    dup: set[str] = set()
    n = 0
    for line in reversed(text.splitlines()):
        n += 1
        if n > max_lines:
            break
        b = _parse_stream_json_batch_line(line)
        if not b:
            continue
        for it in b:
            s = str(it.get("symbol", "")).strip().upper()
            if s and s not in dup:
                dup.add(s)
                seen.append(s)
    return seen


def _parse_sketch_trade_events_from_log_tail(
    log_path: Path,
    max_lines: int = 4000,
) -> list[dict[str, Any]]:
    """Parse sketch fill/exit JSON events from a log tail (read-only, replayable source)."""
    text = read_log_tail_text(log_path)
    if not text.strip():
        return []
    events: list[dict[str, Any]] = []
    lines = text.splitlines()[-max_lines:]
    for seq, raw in enumerate(lines):
        line = raw.strip()
        if not line:
            continue
        if PAPER_SKETCH_FILL_TAG in line:
            payload = line.split(PAPER_SKETCH_FILL_TAG, 1)[1].strip()
            try:
                data = json.loads(payload)
            except json.JSONDecodeError:
                continue
            if not isinstance(data, dict):
                continue
            sym = str(data.get("symbol", "")).strip()
            side = str(data.get("side", "")).strip().upper()
            try:
                ts = int(data.get("ts", 0) or 0)
                entry = float(data.get("entry", 0.0) or 0.0)
                sl = float(data.get("sl", 0.0) or 0.0)
                tp = float(data.get("tp", 0.0) or 0.0)
            except (TypeError, ValueError):
                continue
            if not sym or side not in {"LONG", "SHORT"} or ts <= 0:
                continue
            events.append(
                {
                    "etype": "fill",
                    "symbol": sym,
                    "side": side,
                    "entry": entry,
                    "sl": sl,
                    "tp": tp,
                    "ts": ts,
                    "_seq": seq,
                    "_path": str(log_path),
                }
            )
            continue
        if PAPER_SKETCH_EXIT_TAG in line:
            payload = line.split(PAPER_SKETCH_EXIT_TAG, 1)[1].strip()
            try:
                data = json.loads(payload)
            except json.JSONDecodeError:
                continue
            if not isinstance(data, dict):
                continue
            sym = str(data.get("symbol", "")).strip()
            reason = str(data.get("reason", "")).strip()
            try:
                ts = int(data.get("ts", 0) or 0)
                pnl = float(data.get("pnl", 0.0) or 0.0)
            except (TypeError, ValueError):
                continue
            if not sym or ts <= 0:
                continue
            events.append(
                {
                    "etype": "exit",
                    "symbol": sym,
                    "reason": reason,
                    "pnl": pnl,
                    "ts": ts,
                    "_seq": seq,
                    "_path": str(log_path),
                }
            )
    return events


def parse_latest_symbol_prices_from_tail_text(tail_text: str) -> dict[str, tuple[float, int | None]]:
    """
    Latest symbol prices from `[SYMBOL_PRICE]` lines in one tail text.
    Supports JSON payloads or CSV `SYM:price,SYM:price` payloads.
    """
    latest: dict[str, tuple[float, int | None]] = {}
    for raw in reversed(tail_text.splitlines()):
        line = raw.strip()
        if not line:
            continue
        m = SYMBOL_PRICE_LINE_RE.search(line)
        if not m:
            continue
        payload = m.group(1).strip()
        if payload.startswith("{"):
            try:
                obj = json.loads(payload)
            except json.JSONDecodeError:
                continue
            if not isinstance(obj, dict):
                continue
            ts_raw = obj.get("ts")
            ts_val: int | None = None
            try:
                if ts_raw is not None:
                    ts_val = int(ts_raw)
            except (TypeError, ValueError):
                ts_val = None
            for k, v in obj.items():
                sym = str(k).strip().upper()
                if sym == "TS" or not sym:
                    continue
                if sym in latest:
                    continue
                try:
                    latest[sym] = (float(v), ts_val)
                except (TypeError, ValueError):
                    continue
            continue
        for part in payload.split(","):
            piece = part.strip()
            if ":" not in piece:
                continue
            sym, val = piece.rsplit(":", 1)
            sym_u = sym.strip().upper()
            if not sym_u or sym_u in latest:
                continue
            try:
                latest[sym_u] = (float(val.strip()), None)
            except ValueError:
                continue
    return latest


def collect_latest_symbol_prices(log_paths: list[Path]) -> dict[str, tuple[float, int | None]]:
    """Deterministically merge latest symbol prices across selected log tails."""
    merged: dict[str, tuple[float, int | None]] = {}
    order: dict[str, int] = {}
    for file_idx, p in enumerate(sorted(log_paths, key=lambda x: str(x.resolve()))):
        if not p.is_file():
            continue
        local = parse_latest_symbol_prices_from_tail_text(read_log_tail_text(p))
        for sym, (px, ts) in local.items():
            if sym not in merged:
                merged[sym] = (px, ts)
                order[sym] = file_idx
                continue
            prev_px, prev_ts = merged[sym]
            prev_idx = order.get(sym, -1)
            replace = False
            if ts is not None and prev_ts is not None:
                replace = ts >= prev_ts
            elif ts is not None and prev_ts is None:
                replace = True
            elif ts is None and prev_ts is None:
                replace = file_idx >= prev_idx
            if replace:
                merged[sym] = (px, ts if ts is not None else prev_ts)
                order[sym] = file_idx
    return merged


def compute_live_pnl(entry: float, price: float, side: str) -> float:
    if side == "LONG":
        return (price - entry) / max(entry, 1e-12)
    return (entry - price) / max(entry, 1e-12)


def get_latest_event_ts(events: list[dict[str, Any]]) -> int | None:
    ts_vals = [int(e.get("ts", 0) or 0) for e in events if int(e.get("ts", 0) or 0) > 0]
    if not ts_vals:
        return None
    return max(ts_vals)


def classify_staleness(price_ts: int | None, latest_ts: int | None) -> dict[str, Any]:
    """Raw staleness label + semantic execution-validity layer (deterministic timestamps only)."""
    if price_ts is None or latest_ts is None:
        return {"label": "unknown", "validity": "unknown", "delta_ms": None}
    delta = int(latest_ts) - int(price_ts)
    if delta <= 5_000:
        label = "fresh"
        validity = "execution-valid"
    elif delta <= 30_000:
        label = "warm"
        validity = "potentially drifting"
    else:
        label = "stale"
        validity = "invalid for decision"
    return {"label": label, "validity": validity, "delta_ms": delta}


def compute_event_gap(events: list[dict[str, Any]], price_ts: int | None) -> dict[str, float | int | None]:
    """Count events after the latest price event and normalize by total visible events."""
    if price_ts is None:
        return {"gap": None, "gap_ratio": None}
    total = len(events)
    gap = 0
    for e in events:
        ets = int(e.get("ts", 0) or 0)
        if ets > int(price_ts):
            gap += 1
    gap_ratio = (gap / total) if total > 0 else 0.0
    return {"gap": gap, "gap_ratio": gap_ratio}


def derive_warm_gap_threshold(
    events: list[dict[str, Any]],
    latest_prices: dict[str, tuple[float, int | None]],
    latest_event_ts: int | None,
    *,
    min_samples: int = 10,
    fallback: float = 0.4,
) -> float:
    """
    Deterministic threshold calibration from current warm samples.
    Uses p75 of warm `gap_ratio`; falls back when sample size is small.
    """
    if latest_event_ts is None:
        return fallback
    ratios: list[float] = []
    for _sym, (_px, pts) in latest_prices.items():
        st = classify_staleness(pts, latest_event_ts)
        if st.get("label") != "warm":
            continue
        gi = compute_event_gap(events, pts)
        gr = gi.get("gap_ratio")
        if gr is None:
            continue
        ratios.append(float(gr))
    if len(ratios) < min_samples:
        return fallback
    ratios.sort()
    idx = int((len(ratios) - 1) * 0.75)
    return ratios[idx]


def collect_sketch_trade_events(log_paths: list[Path]) -> list[dict[str, Any]]:
    """Collect and globally sort sketch events from selected logs."""
    events: list[dict[str, Any]] = []
    for file_idx, p in enumerate(sorted(log_paths, key=lambda x: str(x.resolve()))):
        if p.is_file():
            parsed = _parse_sketch_trade_events_from_log_tail(p)
            for ev in parsed:
                ev["_file_idx"] = file_idx
            events.extend(parsed)
    events.sort(
        key=lambda e: (
            int(e.get("ts", 0)),
            int(e.get("_file_idx", 0)),
            int(e.get("_seq", 0)),
            str(e.get("etype", "")),
        )
    )
    # Tail windows can overlap refresh-to-refresh; drop exact duplicate events deterministically.
    deduped: list[dict[str, Any]] = []
    seen: set[tuple[Any, ...]] = set()
    for ev in events:
        sig = (
            str(ev.get("etype", "")),
            str(ev.get("symbol", "")),
            int(ev.get("ts", 0) or 0),
            float(ev.get("entry", 0.0) or 0.0),
            float(ev.get("pnl", 0.0) or 0.0),
            str(ev.get("reason", "")),
            str(ev.get("side", "")),
        )
        if sig in seen:
            continue
        seen.add(sig)
        deduped.append(ev)
    return deduped


def build_sketch_trade_state(events: list[dict[str, Any]]) -> tuple[dict[str, dict[str, Any]], list[dict[str, Any]]]:
    """
    Deterministic state reducer for sketch fills/exits.
    Uses per-symbol LIFO matching so side-flip close + immediate reopen does not cross-join.
    """
    active: dict[tuple[str, str, int], dict[str, Any]] = {}
    open_stacks: dict[str, list[dict[str, Any]]] = defaultdict(list)
    closed: list[dict[str, Any]] = []
    for ev in events:
        et = ev.get("etype")
        sym = str(ev.get("symbol", "")).strip()
        if not sym:
            continue
        if et == "fill":
            f = dict(ev)
            side = str(f.get("side", "")).strip().upper()
            ts = int(f.get("ts", 0) or 0)
            if side not in {"LONG", "SHORT"} or ts <= 0:
                continue
            active[(sym, side, ts)] = f
            open_stacks[sym].append(f)
            continue
        if et == "exit":
            stack = open_stacks.get(sym) or []
            fill = stack.pop() if stack else None
            rec: dict[str, Any] = {
                "symbol": sym,
                "reason": str(ev.get("reason", "")),
                "pnl": float(ev.get("pnl", 0.0) or 0.0),
                "exit_ts": int(ev.get("ts", 0) or 0),
            }
            if fill is not None:
                f_key = (
                    sym,
                    str(fill.get("side", "")).strip().upper(),
                    int(fill.get("ts", 0) or 0),
                )
                active.pop(f_key, None)
                rec.update(
                    {
                        "side": str(fill.get("side", "")),
                        "entry": float(fill.get("entry", 0.0) or 0.0),
                        "sl": float(fill.get("sl", 0.0) or 0.0),
                        "tp": float(fill.get("tp", 0.0) or 0.0),
                        "fill_ts": int(fill.get("ts", 0) or 0),
                    }
                )
            closed.append(rec)
    # UI keeps one active row per symbol for compact display.
    active_by_symbol: dict[str, dict[str, Any]] = {}
    for (_sym, _side, _ts), f in active.items():
        sym = str(f.get("symbol", "")).strip()
        if not sym:
            continue
        prev = active_by_symbol.get(sym)
        if prev is None or int(f.get("ts", 0) or 0) >= int(prev.get("ts", 0) or 0):
            active_by_symbol[sym] = f
    return active_by_symbol, closed


def render_sketch_trade_panels(
    active: dict[str, dict[str, Any]],
    closed: list[dict[str, Any]],
    latest_prices: dict[str, tuple[float, int | None]],
    latest_event_ts: int | None,
    sketch_events: list[dict[str, Any]],
) -> None:
    warm_gap_threshold = derive_warm_gap_threshold(
        sketch_events, latest_prices, latest_event_ts, min_samples=10, fallback=0.4
    )
    st.markdown("### 📈 Active Trades")
    if not active:
        st.caption("No active sketch trades from log replay.")
    else:
        for sym in sorted(active.keys()):
            t = active[sym]
            st.markdown(
                f"**{html.escape(sym, quote=True)} · {html.escape(str(t.get('side', '')), quote=True)}** · "
                f"Entry **{float(t.get('entry', 0.0)):.4f}** · "
                f"SL **{float(t.get('sl', 0.0)):.4f}** · "
                f"TP **{float(t.get('tp', 0.0)):.4f}**"
            )
            px_info = latest_prices.get(sym.upper())
            if px_info is None:
                st.caption("PnL: — (no `[SYMBOL_PRICE]` in tail)")
                continue
            px, px_ts = px_info
            side = str(t.get("side", "")).strip().upper()
            entry = float(t.get("entry", 0.0) or 0.0)
            pnl = compute_live_pnl(entry, float(px), side if side in {"LONG", "SHORT"} else "LONG")
            color = "#16a34a" if pnl >= 0 else "#dc2626"
            stale_info = classify_staleness(px_ts, latest_event_ts)
            gap_info = compute_event_gap(sketch_events, px_ts)
            label = str(stale_info.get("label", "unknown"))
            validity = str(stale_info.get("validity", "unknown"))
            delta_ms = stale_info.get("delta_ms")
            gap = gap_info.get("gap")
            gap_ratio = gap_info.get("gap_ratio")
            badge = {
                "execution-valid": "🟢 execution-valid",
                "potentially drifting": "🟡 potentially drifting",
                "invalid for decision": "⚫ invalid for decision",
                "unknown": "⚪ unknown",
            }.get(validity, "⚪ unknown")
            execution_risk = False
            if label == "stale":
                execution_risk = True
            elif label == "warm" and gap_ratio is not None and float(gap_ratio) > warm_gap_threshold:
                execution_risk = True
            extra = ""
            if delta_ms is not None and gap is not None:
                delta_s = max(0.0, float(delta_ms) / 1000.0)
                ev_per_s = (float(gap) / delta_s) if delta_s > 0 else 0.0
                extra = f" (Δ={delta_s:.1f}s, gap={int(gap)}, {ev_per_s:.1f} ev/s)"
            ts_txt = f" · ts {px_ts}" if px_ts is not None else ""
            risk_txt = " ⚠ execution-risk" if execution_risk else ""
            st.markdown(
                f"<span style='color:{color};font-weight:600;'>PnL: {pnl:+.4%}</span>"
                f"<span style='opacity:0.75;'> {badge}{extra}{risk_txt}{ts_txt}</span>",
                unsafe_allow_html=True,
            )
    st.caption(
        f"Execution-risk warm gap threshold: {warm_gap_threshold:.2f} "
        f"(adaptive p75 when warm samples >= 10, else fallback 0.40)."
    )
    st.markdown("### 📊 Closed Trades")
    if not closed:
        st.caption("No closed sketch trades from log replay.")
        return
    total_pnl = sum(float(t.get("pnl", 0.0) or 0.0) for t in closed)
    wins = sum(1 for t in closed if float(t.get("pnl", 0.0) or 0.0) > 0.0)
    win_rate = wins / len(closed) if closed else 0.0
    st.caption(
        f"Trades: {len(closed)} · Total PnL: {total_pnl:+.6f} · Win rate: {win_rate:.0%}"
    )
    with st.expander("Recent closed sketch trades", expanded=False):
        for t in closed[-10:][::-1]:
            sym = html.escape(str(t.get("symbol", "")), quote=True)
            side = html.escape(str(t.get("side", "—")), quote=True)
            reason = html.escape(str(t.get("reason", "")), quote=True)
            pnl = float(t.get("pnl", 0.0) or 0.0)
            st.markdown(f"**{sym} · {side}** · PnL **{pnl:+.6f}** · `{reason}`")


def engine_close_sequence_from_tail(text: str, symbol: str) -> list[float]:
    """Chronological closes from `[SYMBOL_PRICE]` lines (engine-emitted)."""
    sym_u = symbol.strip().upper()
    out: list[float] = []
    for line in text.splitlines():
        m = SYMBOL_PRICE_LINE_RE.search(line)
        if not m:
            continue
        for part in m.group(1).split(","):
            part = part.strip()
            if ":" not in part:
                continue
            sym, val_str = part.rsplit(":", 1)
            if sym.strip().upper() != sym_u:
                continue
            try:
                out.append(float(val_str.strip()))
            except ValueError:
                continue
    return out


def json_contiguous_batches_for_symbol(
    text: str,
    symbol: str,
    max_batches: int,
    max_gap_sec: float,
) -> tuple[list[list[dict[str, Any]]], int] | None:
    """Newest-first walk with timestamp gap break; returns chronological batch list + count."""
    sym_u = symbol.strip().upper()
    batches: list[list[dict[str, Any]]] = []
    prev_ts: int | None = None
    for line in reversed(text.splitlines()):
        b = _parse_stream_json_batch_line(line)
        if not b:
            continue
        ts_cand: list[int] = []
        for it in b:
            if str(it.get("symbol", "")).strip().upper() != sym_u:
                continue
            try:
                ts_cand.append(int(it["timestamp"]))
            except (KeyError, TypeError, ValueError):
                continue
        if not ts_cand:
            continue
        ts_b = min(ts_cand)
        if prev_ts is not None and abs(ts_b - prev_ts) > max_gap_sec:
            break
        batches.append(b)
        prev_ts = ts_b
        if len(batches) >= max_batches:
            break
    if not batches:
        return None
    chrono = list(reversed(batches))
    return chrono, len(chrono)


def _median_last_k(vals: list[float], k: int) -> float | None:
    if not vals:
        return None
    tail = vals[-k:]
    return float(statistics.median(tail))


def extract_trade_sketch_price_inputs(log_path: Path, symbol: str) -> dict[str, Any] | None:
    """Entry (median of last closes) + risk span from JSON HL or engine closes; tail-only read."""
    text = read_log_tail_text(log_path)
    if not text.strip():
        return None
    sym_u = symbol.strip().upper()
    eng = engine_close_sequence_from_tail(text, symbol)
    jw = json_contiguous_batches_for_symbol(
        text, symbol, TRADE_SKETCH_MAX_JSON_BATCHES, TRADE_BATCH_MAX_GAP_SEC
    )
    entry: float | None = None
    risk: float | None = None
    n_json = 0
    labels: list[str] = []

    if eng:
        entry = _median_last_k(eng, TRADE_MEDIAN_CLOSES)
        labels.append("engine_close")

    if jw:
        batches_chrono, n_json = jw
        highs: list[float] = []
        lows: list[float] = []
        closes_pb: list[float] = []
        for bat in batches_chrono:
            last_c: float | None = None
            for it in bat:
                if str(it.get("symbol", "")).strip().upper() != sym_u:
                    continue
                try:
                    highs.append(float(it["high"]))
                    lows.append(float(it["low"]))
                    last_c = float(it["close"])
                except (KeyError, TypeError, ValueError):
                    continue
            if last_c is not None:
                closes_pb.append(last_c)
        if highs and lows:
            risk = max(highs) - min(lows)
            labels.append("json_hl")
        if entry is None and closes_pb:
            entry = _median_last_k(closes_pb, TRADE_MEDIAN_CLOSES)
            labels.append("json_close")

    if risk is None and len(eng) >= 2:
        win = eng[-TRADE_ENGINE_RISK_WINDOW :]
        risk = max(win) - min(win)
        labels.append("engine_range")

    if entry is None or entry <= 0:
        return None
    if risk is None or risk <= 0:
        return {"_state": "no_range", "entry": entry}
    if risk < entry * TRADE_MIN_RISK_FRAC:
        return {"_state": "no_range", "entry": entry, "risk": risk, "_reason": "min_risk"}
    range_pct = (risk / entry) * 100.0
    return {
        "entry": entry,
        "risk": risk,
        "n_json_batches": n_json,
        "range_pct": range_pct,
        "labels": labels,
    }


def aggregate_final_side_totals(rows: list[dict[str, Any]]) -> tuple[int, int]:
    fb = 0
    fs = 0
    for r in rows:
        sd = r.get("side_distribution") or {}
        fb += int(sd.get("final_buy", 0) or 0)
        fs += int(sd.get("final_sell", 0) or 0)
    return fb, fs


def trade_sketch_direction(fb: int, fs: int) -> str | None:
    if fb <= 0 and fs <= 0:
        return None
    if fb > fs:
        return "LONG"
    if fs > fb:
        return "SHORT"
    return None


def trade_sketch_slice_count(sym_summary: dict[str, Any], rows: list[dict[str, Any]]) -> int:
    n = int(sym_summary.get("total_slices", sym_summary.get("files_parsed", 0)) or 0)
    if n > 0:
        return n
    return len(rows)


def trade_sketch_confidence(slice_n: int, imbalance: int) -> str:
    if imbalance >= 3 and slice_n >= 10:
        return "HIGH"
    if imbalance >= 2 and slice_n >= 5:
        return "MEDIUM"
    return "LOW"


def construct_trade_sketch(
    symbol: str,
    rows: list[dict[str, Any]],
    sym_summary: dict[str, Any],
    log_path: Path | None,
) -> dict[str, Any] | None:
    """Deterministic entry/SL/target from `[SYMBOL_PRICE]` + contiguous JSON batches (no ML)."""
    fb, fs = aggregate_final_side_totals(rows)
    direction = trade_sketch_direction(fb, fs)
    if direction is None:
        return None
    if log_path is None or not log_path.is_file():
        return {"state": "no_log", "symbol": symbol}
    px = extract_trade_sketch_price_inputs(log_path, symbol)
    if px is None:
        return {"state": "no_price", "symbol": symbol, "direction": direction}
    if px.get("_state") == "no_range":
        return {
            "state": "no_range",
            "symbol": symbol,
            "direction": direction,
            "entry": float(px["entry"]) if px.get("entry") is not None else None,
            "reason": px.get("_reason"),
        }
    entry = float(px["entry"])
    risk = float(px["risk"])
    if direction == "LONG":
        sl = entry - TRADE_SKETCH_SL_FRAC * risk
        target = entry + TRADE_SKETCH_TP_FRAC * risk
    else:
        sl = entry + TRADE_SKETCH_SL_FRAC * risk
        target = entry - TRADE_SKETCH_TP_FRAC * risk
    sc = trade_sketch_slice_count(sym_summary, rows)
    hold = max(3, min(15, sc * 2))
    imb = abs(fb - fs)
    conf = trade_sketch_confidence(sc, imb)
    risk_abs = abs(entry - sl)
    reward_abs = abs(target - entry)
    rr = reward_abs / risk_abs if risk_abs > 1e-12 else 0.0
    n_j = int(px.get("n_json_batches", 0) or 0)
    rp = float(px.get("range_pct", 0.0) or 0.0)
    lbl = ",".join(str(x) for x in (px.get("labels") or []))
    provenance_caption = (
        f"Derived from tail (~{TRADE_TAIL_BYTES // 1024}KB): {n_j} contiguous JSON batches · "
        f"range {rp:.3f}% of entry · binding: {lbl}. "
        f"`final_buy={fb}` `final_sell={fs}`."
    )
    return {
        "state": "ok",
        "symbol": symbol,
        "direction": direction,
        "entry": entry,
        "stop_loss": sl,
        "target": target,
        "hold_minutes": hold,
        "confidence": conf,
        "risk_span": risk,
        "rr": rr,
        "final_buy": fb,
        "final_sell": fs,
        "provenance_caption": provenance_caption,
    }


def render_trade_setup_panel(
    mode: str,
    is_multi_view: bool,
    multi_results: dict[str, dict[str, Any]],
    rows: list[dict[str, Any]],
    summary: dict[str, Any],
    selected_paths: list[Path],
) -> None:
    st.markdown("### Trade setup (when available)")
    st.caption(
        "Read-only sketch: aggregate `final_buy`/`final_sell` across parsed slices, binding prices from "
        "`[SYMBOL_PRICE]` in the log, risk from contiguous JSON OHLC batches (gap ≤ "
        f"{TRADE_BATCH_MAX_GAP_SEC:.0f}s, up to {TRADE_SKETCH_MAX_JSON_BATCHES}) or engine close range. "
        "Not execution advice."
    )
    if is_multi_view and multi_results:
        for sym in sorted(multi_results.keys()):
            res = multi_results[sym]
            sym_rows = res.get("rows", []) or []
            sym_summary = res.get("summary", {}) or {}
            lp: Path | None = None
            try:
                lp = Path(str(res.get("source", "")))
            except (TypeError, ValueError):
                lp = None
            plan = construct_trade_sketch(sym, sym_rows, sym_summary, lp)
            _render_one_trade_plan(sym, plan)
        return
    if selected_paths:
        lp = selected_paths[0]
        stream_syms = stream_symbols_from_log_tail(lp)
        if stream_syms:
            for sym in stream_syms:
                plan = construct_trade_sketch(sym, rows, summary, lp)
                _render_one_trade_plan(sym, plan)
            return
        sym_fb: str | None = None
        for r in rows:
            stmap = r.get("symbol_timestamps")
            if isinstance(stmap, dict) and stmap:
                sym_fb = sorted(str(k) for k in stmap.keys())[0]
                break
        sym_use = sym_fb or "PORTFOLIO"
        plan = construct_trade_sketch(sym_use, rows, summary, lp)
        _render_one_trade_plan(sym_fb or "Diagnostics", plan)
        return
    st.caption("No log paths selected — trade sketch unavailable.")


def _render_one_trade_plan(label: str, plan: dict[str, Any] | None) -> None:
    st.markdown(f"**{html.escape(label, quote=True)}**")
    if plan is None:
        st.caption("No trade — insufficient structure (`final_buy` / `final_sell` totals tie at zero).")
        return
    if plan.get("state") == "no_log":
        st.caption("No source log path for price/risk extraction.")
        return
    if plan.get("state") == "no_price":
        st.caption(
            "No `[SYMBOL_PRICE]` line for this symbol in the log tail, and no usable JSON stream closes — "
            "cannot bind entry without inventing a price."
        )
        return
    if plan.get("state") == "no_range":
        reason = plan.get("reason")
        if reason == "min_risk":
            st.warning(
                f"Range too small vs entry for {plan.get('direction', '?')} "
                f"(below {TRADE_MIN_RISK_FRAC * 100:.2f}% of entry) — no trade."
            )
        else:
            st.warning(
                f"Flat or missing OHLC/close window for {plan.get('direction', '?')} — "
                "cannot derive positive risk span."
            )
        return
    if plan.get("state") != "ok":
        st.caption("No trade — insufficient structure.")
        return
    d = plan["direction"]
    icon = "📈" if d == "LONG" else "📉"
    st.markdown(
        f"{icon} **{d}** · Entry **{plan['entry']:.4f}** · SL **{plan['stop_loss']:.4f}** · "
        f"Target **{plan['target']:.4f}** · Hold **{plan['hold_minutes']}** min · "
        f"Confidence **{plan['confidence']}** · R:R **{plan['rr']:.2f}**"
    )
    prov = plan.get("provenance_caption")
    if prov:
        st.caption(prov)
    else:
        st.caption(
            f"From `final_buy={plan['final_buy']}` `final_sell={plan['final_sell']}` · "
            f"risk span **{plan['risk_span']:.6f}**."
        )


def is_feed_live(log_dir: str, log_pattern: str, threshold_seconds: int) -> bool:
    """True when at least one matching log file was updated recently."""
    latest_mtime = get_latest_log_mtime(log_dir, log_pattern)
    if latest_mtime <= 0:
        return False
    return (time.time() - latest_mtime) < float(threshold_seconds)


def get_latest_log_mtime(log_dir: str, log_pattern: str) -> float:
    """Latest mtime among matching log files, or 0 when unavailable."""
    d = Path(log_dir)
    if not d.is_dir():
        return 0.0
    matches = list(d.glob(log_pattern))
    if not matches:
        return 0.0
    return max((p.stat().st_mtime for p in matches), default=0.0)


def format_age_seconds(age_sec: int) -> str:
    if age_sec < 60:
        return f"{age_sec}s ago"
    if age_sec < 3600:
        return f"{age_sec // 60}m {age_sec % 60}s ago"
    return f"{age_sec // 3600}h {(age_sec % 3600) // 60}m ago"


def inject_phase_progress_color(phase: str) -> None:
    phase_colors = {
        "LONG_ONLY": "#16a34a",
        "MONITORING": "#f59e0b",
        "READY_FOR_MAPPING": "#2563eb",
        "SHORT_ACTIVE": "#dc2626",
    }
    color = phase_colors.get(phase, "#2563eb")
    st.markdown(
        f"""
<style>
[data-testid="stProgress"] > div > div {{
    background: linear-gradient(90deg, {color}, {color}) !important;
}}
</style>
""",
        unsafe_allow_html=True,
    )


def render_state_change_strip(history: list[dict[str, Any]]) -> None:
    st.markdown("### ⏱ Recent State Changes")
    if not history:
        st.caption("No state changes yet")
        return

    cols = st.columns(len(history))
    phase_colors = {
        "LONG_ONLY": "#16a34a",
        "MONITORING": "#f59e0b",
        "READY_FOR_MAPPING": "#2563eb",
        "SHORT_ACTIVE": "#dc2626",
    }
    for i, item in enumerate(history):
        phase = str(item.get("phase", "UNKNOWN"))
        border = phase_colors.get(phase, "#6b7280")
        with cols[i]:
            st.markdown(
                f"""
            <div style="
                padding:8px;
                border-radius:8px;
                background:#111827;
                border:1px solid {border};
                text-align:center;
            ">
                <div style="font-size:11px; color:#6b7280;">
                    {item.get("time", "—")}
                </div>
                <div style="font-size:13px; font-weight:600; color:{border};">
                    {phase}
                </div>
                <div style="font-size:11px; color:#9ca3af;">
                    near:{item.get("near", 0)} sell:{item.get("sell", 0)}
                </div>
            </div>
            """,
                unsafe_allow_html=True,
            )


def extract_ticker_freshness(rows: list[dict[str, Any]]) -> dict[str, float]:
    """Build symbol -> latest timestamp map from parsed rows."""
    latest: dict[str, float] = {}
    for r in rows:
        sym_ts = r.get("symbol_timestamps", {}) or {}
        if isinstance(sym_ts, dict):
            for sym_raw, ts_raw in sym_ts.items():
                try:
                    ts_f = float(ts_raw)
                except (TypeError, ValueError):
                    continue
                if ts_f > 1_000_000_000_000:  # milliseconds epoch
                    ts_f = ts_f / 1000.0
                sym = str(sym_raw)
                prev = latest.get(sym)
                if prev is None or ts_f > prev:
                    latest[sym] = ts_f

        sym = r.get("symbol")
        ts = r.get("timestamp")
        if sym is None or ts is None:
            continue
        try:
            ts_f = float(ts)
        except (TypeError, ValueError):
            continue
        if ts_f > 1_000_000_000_000:  # milliseconds epoch
            ts_f = ts_f / 1000.0
        prev = latest.get(str(sym))
        if prev is None or ts_f > prev:
            latest[str(sym)] = ts_f
    return latest


def classify_freshness(ts: float, now: float | None = None) -> str:
    now_ts = time.time() if now is None else now
    age = now_ts - ts
    if age < 5:
        return "fresh"
    if age < 15:
        return "warm"
    return "stale"


def format_pill_age(ts: float, now: float) -> str:
    age = int(max(0.0, now - ts))
    if age < 60:
        return f"{age}s"
    if age < 3600:
        return f"{age // 60}m"
    return f"{age // 3600}h"


def render_fresh_ticker_pills(ticker_ts_map: dict[str, float], stale_threshold: int) -> None:
    if not ticker_ts_map:
        return
    now = time.time()
    items = sorted(ticker_ts_map.items(), key=lambda x: x[1], reverse=True)
    max_visible = 3
    visible = items[:max_visible]
    remaining = max(0, len(items) - max_visible)
    color_map = {
        "fresh": "#16a34a",
        "warm": "#f59e0b",
        "stale": "#6b7280",
    }
    parts: list[str] = []
    for sym, ts in visible:
        age = format_pill_age(ts, now)
        age_sec = int(max(0.0, now - ts))
        state = classify_freshness(ts, now)
        color = color_map[state]
        full_ts = datetime.fromtimestamp(ts).strftime("%Y-%m-%d %H:%M:%S")
        degraded = age_sec > stale_threshold
        marker = " !" if degraded else ""
        opacity = "0.6" if degraded else "1.0"
        safe_sym = html.escape(sym, quote=True)
        safe_title = html.escape(f"Last update: {full_ts}", quote=True)
        parts.append(
            f'<span title="{safe_title}" style="display:inline-block;padding:4px 10px;'
            f"border-radius:999px;background:{color}22;color:{color};"
            f"border:1px solid {color}55;font-size:12px;opacity:{opacity};"
            f'">{safe_sym} · {age}{marker}</span>'
        )
    if remaining > 0:
        parts.append(
            '<span style="display:inline-block;padding:4px 10px;border-radius:999px;'
            f'background:#374151;color:#9ca3af;font-size:12px;">+{remaining}</span>'
        )
    inner = "".join(parts)
    pills_html = (
        f'<div style="display:flex;flex-wrap:wrap;gap:6px;align-items:center;">{inner}</div>'
    )
    st.markdown(pills_html, unsafe_allow_html=True)
    st.caption("🟢 fresh  ·  🟡 warm  ·  ⚫ stale")


def build_per_ticker_summary(rows: list[dict[str, Any]]) -> dict[str, dict[str, float | int]]:
    """Aggregate per-symbol trade snapshots from parsed rows."""
    out: dict[str, dict[str, float | int]] = {}
    for r in rows:
        sym_ts = r.get("symbol_timestamps", {}) or {}
        if isinstance(sym_ts, dict):
            for sym_raw, ts_raw in sym_ts.items():
                sym = str(sym_raw)
                try:
                    ts_f = float(ts_raw)
                except (TypeError, ValueError):
                    continue
                if ts_f > 1_000_000_000_000:
                    ts_f = ts_f / 1000.0
                d = out.setdefault(
                    sym,
                    {"latest_ts": ts_f, "events": 0, "closed_trades": 0, "wins": 0, "losses": 0, "total_pnl": 0.0},
                )
                d["latest_ts"] = max(float(d["latest_ts"]), ts_f)
                d["events"] = int(d["events"]) + 1

        tts = r.get("ticker_trade_summary", {}) or {}
        if isinstance(tts, dict):
            for sym_raw, stats in tts.items():
                if not isinstance(stats, dict):
                    continue
                sym = str(sym_raw)
                d = out.setdefault(
                    sym,
                    {"latest_ts": 0.0, "events": 0, "closed_trades": 0, "wins": 0, "losses": 0, "total_pnl": 0.0},
                )
                d["closed_trades"] = int(d["closed_trades"]) + int(stats.get("closed_trades", 0) or 0)
                d["wins"] = int(d["wins"]) + int(stats.get("wins", 0) or 0)
                d["losses"] = int(d["losses"]) + int(stats.get("losses", 0) or 0)
                d["total_pnl"] = float(d["total_pnl"]) + float(stats.get("total_pnl", 0.0) or 0.0)
    return out


def compute_formation_metrics(rows: list[dict[str, Any]]) -> dict[str, int]:
    """
    Deterministic formation tracker from parsed slice diagnostics.
    Distinguishes observed-but-not-promoted from no-observation windows.
    """
    total = len(rows)
    formation = 0
    triggered = 0
    dropped = 0
    for r in rows:
        sd = r.get("side_distribution") or {}
        c_buy = int(sd.get("candidates_buy", 0) or 0)
        c_sell = int(sd.get("candidates_sell", 0) or 0)
        f_buy = int(sd.get("final_buy", 0) or 0)
        f_sell = int(sd.get("final_sell", 0) or 0)
        has_formation = (c_buy + c_sell) > 0
        has_trigger = (f_buy + f_sell) > 0
        if has_formation:
            formation += 1
        if has_trigger:
            triggered += 1
        if has_formation and not has_trigger:
            dropped += 1
    return {
        "total_slices": total,
        "formation_events": formation,
        "triggered_events": triggered,
        "dropped_before_trigger": dropped,
    }


def compute_ticker_decision(d: dict[str, float | int]) -> tuple[str, str, str]:
    trades = int(d.get("closed_trades", 0) or 0)
    pnl = float(d.get("total_pnl", 0.0) or 0.0)
    n = int(d.get("events", 0) or 0)
    if trades > 0 and pnl < 0:
        label, color = "WATCH", "#f59e0b"
    elif trades > 0 and pnl >= 0:
        label, color = "ACTIVE", "#16a34a"
    else:
        label, color = "HOLD", "#6b7280"

    if n >= 8:
        conf = "HIGH"
    elif n >= 5:
        conf = "MEDIUM"
    else:
        conf = "LOW"
    return label, color, conf


def render_per_ticker_panel(ticker_map: dict[str, dict[str, float | int]], max_cards: int = 4) -> None:
    if not ticker_map:
        return
    items = sorted(ticker_map.items(), key=lambda kv: float(kv[1]["latest_ts"]), reverse=True)
    items = items[:max_cards]
    if not items:
        return

    st.markdown("### 🧩 Per-Ticker Trade Snapshot")
    cols = st.columns(len(items))
    for i, (sym, d) in enumerate(items):
        label, color, conf = compute_ticker_decision(d)
        trades = int(d.get("closed_trades", 0) or 0)
        total_pnl = float(d.get("total_pnl", 0.0) or 0.0)
        with cols[i]:
            st.markdown(
                f"""
            <div style="
                padding:12px;
                border-radius:10px;
                background:#111827;
                border:1px solid #1f2937;
                border-top:3px solid {color};
            ">
                <div style="font-size:12px; color:#9ca3af;">{sym}</div>
                <div style="font-size:14px; font-weight:600; color:{color};">
                    {label}
                </div>
                <div style="font-size:11px; color:#6b7280;">
                    Trades: {trades} · PnL: {total_pnl:+.4f}
                </div>
                <div style="font-size:11px; color:#6b7280;">
                    Conf: {conf} · n={int(d.get("events", 0))}
                </div>
            </div>
            """,
                unsafe_allow_html=True,
            )


def compute_lane_health(multi_results: dict[str, dict[str, Any]], stale_threshold: int) -> list[tuple[str, str, float | None]]:
    now = time.time()
    health: list[tuple[str, str, float | None]] = []
    for sym, res in sorted(multi_results.items()):
        rows = res.get("rows", []) or []
        latest_ts: float | None = None
        for r in rows:
            st_map = r.get("symbol_timestamps", {}) or {}
            if isinstance(st_map, dict) and sym in st_map:
                try:
                    ts = float(st_map[sym])
                except (TypeError, ValueError):
                    continue
                if ts > 1_000_000_000_000:
                    ts = ts / 1000.0
                if latest_ts is None or ts > latest_ts:
                    latest_ts = ts
        if latest_ts is None:
            status = "unknown"
        else:
            age = now - latest_ts
            status = "alive" if age <= stale_threshold else "stale"
        health.append((sym, status, latest_ts))
    return health


def render_missing_lanes_strip(missing: list[str]) -> None:
    if not missing:
        return
    parts: list[str] = []
    for sym in missing:
        safe = html.escape(sym, quote=True)
        parts.append(
            f'<span style="display:inline-block;padding:4px 10px;'
            f'border-radius:999px;background:#6b728022;color:#6b7280;'
            f'border:1px solid #6b728055;font-size:12px;">{safe} · missing</span>'
        )
    inner = "".join(parts)
    st.markdown(
        f'<div style="display:flex;flex-wrap:wrap;gap:6px;align-items:center;margin:8px 0 12px 0;">{inner}</div>',
        unsafe_allow_html=True,
    )
    st.caption("Expected lanes not observed in current logs.")


def render_lane_health_strip(
    health: list[tuple[str, str, float | None]],
    *,
    show_heading: bool = True,
) -> None:
    if not health:
        return
    color_map = {
        "alive": "#16a34a",
        "stale": "#f59e0b",
        "unknown": "#6b7280",
    }
    parts: list[str] = []
    for sym, status, ts in health:
        color = color_map.get(status, "#6b7280")
        safe_sym = html.escape(sym, quote=True)
        if ts is not None:
            full_ts = datetime.fromtimestamp(ts).strftime("%Y-%m-%d %H:%M:%S")
            title = html.escape(f"Last update: {full_ts}", quote=True)
        else:
            title = "No timestamp available yet"
        parts.append(
            f'<span title="{title}" style="display:inline-block;padding:4px 10px;'
            f"border-radius:999px;background:{color}22;color:{color};"
            f'border:1px solid {color}55;font-size:12px;">{safe_sym} · {status}</span>'
        )
    inner = "".join(parts)
    pill_markup = (
        f'<div style="display:flex;flex-wrap:wrap;gap:6px;align-items:center;margin:8px 0 12px 0;">{inner}</div>'
    )
    if show_heading:
        st.markdown("### 🧭 Lane Health")
    st.markdown(pill_markup, unsafe_allow_html=True)


def render_live_pipeline_observability(
    *,
    feed_live: bool,
    is_multi_view: bool,
    multi_results: dict[str, dict[str, Any]],
    expected_symbols: list[str],
    stale_threshold: int,
    diag_dir: str,
    diag_pattern: str,
) -> bool:
    """Single Live pipeline + lane block (read-only). Returns whether pipelines look active."""
    pipeline_alive = feed_live and (not is_multi_view or bool(multi_results))
    lm = get_latest_log_mtime(diag_dir, diag_pattern)
    age_sec = int(max(0.0, time.time() - lm)) if lm > 0 else None
    age_label = format_age_seconds(age_sec) if age_sec is not None else "— (no log mtime)"

    if pipeline_alive:
        st.success("📡 Live pipelines active")
        if multi_results:
            lane_health = compute_lane_health(multi_results, stale_threshold)
            render_lane_health_strip(lane_health, show_heading=False)
            if expected_symbols:
                observed = set(multi_results.keys())
                missing = sorted(set(expected_symbols) - observed)
                render_missing_lanes_strip(missing)
        return True

    lines: list[str] = []
    lines.append(
        f"No log updates for <span style='color:#e5e7eb;'>{html.escape(age_label, quote=False)}</span>."
    )
    if is_multi_view:
        missing = sorted(set(expected_symbols) - set(multi_results.keys()))
        if expected_symbols and missing:
            miss_txt = html.escape(", ".join(missing), quote=False)
            lines.append(f"Expected symbol lanes not observed: {miss_txt}.")
        elif not multi_results:
            lines.append("No symbol lanes matched the current <code>live_*</code> glob.")
    d_esc = html.escape(str(diag_dir), quote=True)
    p_esc = html.escape(str(diag_pattern), quote=True)
    lines.append("<br><strong>Start:</strong> <code>scripts/run_multi_engine.py</code>")
    lines.append(f"<strong>Check:</strong> log directory <code>{d_esc}</code> · glob <code>{p_esc}</code>")

    body = "<br>".join(lines)
    st.markdown(
        f"""
<div style="
    padding:14px 16px;
    border-radius:10px;
    border:1px solid #f59e0b66;
    background:#42200633;
    margin-bottom:12px;
">
    <div style="color:#fbbf24;font-weight:600;font-size:16px;margin-bottom:8px;">
        No active pipelines detected
    </div>
    <div style="font-size:14px;color:#d1d5db;line-height:1.55;">
        {body}
    </div>
</div>
""",
        unsafe_allow_html=True,
    )
    return False


def read_baseline_metadata() -> dict[str, Any] | None:
    p = _ROOT / BASELINE_V1_METADATA
    if not p.is_file():
        return None
    try:
        return json.loads(p.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None


def baseline_reference_counts(meta: dict[str, Any] | None) -> dict[str, int | str] | None:
    if not meta:
        return None
    s = meta.get("summary")
    if not isinstance(s, dict):
        return None
    cls = str(s.get("classification", meta.get("classification", "")) or "").strip()
    return {
        "near": int(s.get("offsets_with_near_bearish", s.get("near_bearish_offsets", 0)) or 0),
        "sell_c": int(s.get("offsets_with_sell_candidates", s.get("sell_candidates", 0)) or 0),
        "sell_f": int(s.get("offsets_with_final_sell", s.get("final_sell", 0)) or 0),
        "classification": cls,
    }


def baseline_alignment_badge(
    cur: dict[str, int],
    base: dict[str, int | str] | None,
    n_slices: int,
) -> tuple[str, str, str]:
    """Return (icon, title, detail) for read-only comparison vs frozen metadata (not 'truth')."""
    if base is None:
        return ("⚪", "No baseline reference", "Add or refresh `analysis/baselines/baseline_v1/metadata.json`.")
    bn = int(base["near"])
    bsc = int(base["sell_c"])
    bsf = int(base["sell_f"])
    cn, csc, csf = cur["near"], cur["sell_c"], cur["sell_f"]
    if bsf == 0 and csf > 0:
        return ("🔴", "Regression vs baseline reference", "Final SELL offsets appear now; baseline metadata recorded none.")
    if bsc == 0 and csc > 0:
        return ("🔴", "Regression vs baseline reference", "Sell-candidate offsets appear now; baseline metadata recorded none.")
    delta_near = cn - bn
    threshold = max(2, max(1, n_slices) // 5)
    if delta_near >= threshold and cn > bn:
        return (
            "🟡",
            "Deviation vs baseline reference",
            f"Near-bearish offsets {cn} vs baseline {bn} (Δ +{delta_near}).",
        )
    return (
        "🟢",
        "Within baseline reference envelope",
        f"Near-bearish {cn} vs {bn}; sell activity comparable to recorded snapshot.",
    )


def render_baseline_alignment_badge(icon: str, title: str, detail: str) -> None:
    safe_title = html.escape(title, quote=True)
    safe_detail = html.escape(detail, quote=True)
    st.markdown(
        f"""
<div style="padding:10px 14px;border-radius:10px;border:1px solid #374151;background:#111827;margin-bottom:10px;">
    <span style="font-size:16px;">{icon}</span>
    <b style="color:#e5e7eb;">{safe_title}</b>
    <span style="color:#9ca3af;font-size:13px;"> — {safe_detail}</span>
</div>
""",
        unsafe_allow_html=True,
    )


def render_baseline_comparison_panel(
    mode: str,
    near_n: int,
    sell_c_n: int,
    fin_n: int,
    rec: str,
    base: dict[str, int | str] | None,
) -> None:
    if base is None:
        return
    if mode == "Baseline":
        st.info(
            "Baseline mode: viewing the frozen snapshot. Switch to **Live** or **Replay** to compare "
            "other runs against this `metadata.json` reference."
        )
        return
    st.markdown("#### Baseline comparison")
    b_cls = str(base.get("classification") or "—")
    bn, bsc, bsf = int(base["near"]), int(base["sell_c"]), int(base["sell_f"])
    h0, h1, h2 = st.columns((2, 1, 1))
    h0.markdown("**Metric**")
    h1.markdown("**Current**")
    h2.markdown("**Baseline ref**")
    for label, cv, bv in (
        ("Near-bearish offsets", near_n, bn),
        ("Sell candidates", sell_c_n, bsc),
        ("Final SELL", fin_n, bsf),
    ):
        c0, c1, c2 = st.columns((2, 1, 1))
        c0.caption(label)
        c1.markdown(f"**{cv}**")
        c2.markdown(f"**{bv}**")
    r0, r1, r2 = st.columns((2, 1, 1))
    r0.caption("Runner recommendation")
    r1.markdown(f"`{rec}`")
    r2.markdown(f"`{b_cls}`")
    st.caption(
        "Right column: values from `metadata.json` (reference snapshot only, not ground truth)."
    )


def compute_recommendation(summary: dict[str, Any], phase: str) -> tuple[str, str, str]:
    near = int(summary.get("offsets_with_near_bearish", 0) or 0)
    sell = int(summary.get("offsets_with_sell_candidates", 0) or 0)
    if sell > 0:
        return ("SHORT ENABLED", "Bearish structure validated", "#dc2626")
    if near == 0:
        return ("HOLD", "No validated market structure", "#6b7280")
    if phase == "LONG_ONLY":
        return ("LONG BIAS", "Bullish structure, awaiting trigger", "#16a34a")
    return ("WAIT", "Structure forming, not confirmed", "#f59e0b")


def compute_recommendation_confidence(summary: dict[str, Any], near: int) -> tuple[str, int]:
    slices = int(summary.get("total_slices", summary.get("files_parsed", 0)) or 0)
    confidence = "LOW"
    if slices >= 5:
        confidence = "MEDIUM"
    if slices >= 8 and near == 0:
        confidence = "HIGH"
    return confidence, slices


def recommendation_hint(label: str) -> str:
    if label == "HOLD":
        return "Awaiting emergence of near-bearish or trigger conditions"
    if label == "LONG BIAS":
        return "Watch for trigger confirmation in current regime"
    if label == "WAIT":
        return "Structure forming, not yet actionable"
    return "Short-side conditions validated"


def render_recommendation_bar(
    summary: dict[str, Any],
    phase: str,
    *,
    mode: str = "Replay",
    feed_live: bool = True,
    pipeline_alive: bool = True,
    pending_first_slice: bool = False,
    is_per_lane: bool = False,
    is_multi_view: bool = False,
) -> None:
    label, reason, color = compute_recommendation(summary, phase)
    near = int(summary.get("offsets_with_near_bearish", 0) or 0)
    confidence, slices = compute_recommendation_confidence(summary, near)
    hint = recommendation_hint(label)
    icon_map = {
        "HOLD": "⏸",
        "WAIT": "⏳",
        "LONG BIAS": "📈",
        "SHORT ENABLED": "📉",
    }
    icon = icon_map.get(label, "•")
    if slices == 0:
        if mode == "Live" and not pipeline_alive and not is_per_lane:
            title = "No diagnostic slices"
            subtitle = "Pipeline and feed health are summarized in the status section above."
            accent = "#6b7280"
        elif mode == "Live" and not pipeline_alive and is_per_lane:
            title = "Lane quiet"
            subtitle = "See pipeline status above for feed and process health."
            accent = "#6b7280"
        elif mode == "Live" and pending_first_slice and feed_live:
            title = "⏳ Waiting for first diagnostic slice"
            subtitle = (
                "Feed is updating; aggregate diagnostic summary lines are not emitted yet."
            )
            accent = "#6b7280"
        elif mode == "Live":
            title = "⏳ Waiting for first diagnostic slice"
            subtitle = "Live mode: diagnostic rows are not loaded yet for this view."
            accent = "#6b7280"
        elif mode == "Baseline":
            title = "No diagnostic slices loaded"
            subtitle = (
                "Baseline `diag_logs/` is empty or glob mismatch — run `bash scripts/snapshot_baseline.sh`."
            )
            accent = "#6b7280"
        else:
            title = "No diagnostic slices loaded"
            subtitle = "Replay or path has no parsed aggregate rows yet."
            accent = "#6b7280"
        st.markdown(
            f"""
        <div style="
            padding:16px;
            border-radius:12px;
            background:#6b728022;
            border:1px solid #6b728055;
            border-top:3px solid {accent};
            margin-bottom:14px;
        ">
            <div style="font-size:14px; color:#9ca3af;">Current Decision</div>
            <div style="font-size:18px; font-weight:600; color:#9ca3af;">
                {title}
            </div>
            <div style="font-size:12px; color:#9ca3af;">
                {subtitle}
            </div>
        </div>
        """,
            unsafe_allow_html=True,
        )
        return
    st.markdown(
        f"""
    <div style="
        padding:16px;
        border-radius:12px;
        background:{color}22;
        border:1px solid {color}55;
        border-top: 3px solid {color};
        margin-bottom:14px;
    ">
        <div style="font-size:14px; color:#9ca3af;">Current Decision</div>
        <div style="font-size:18px; font-weight:600; color:{color};">
            {icon} {label}
        </div>
        <div style="font-size:12px; color:#9ca3af;">
            {reason}
        </div>
        <div style="font-size:11px; color:#6b7280; margin-top:4px;">
            Confidence: {confidence} · Slices: {slices}
        </div>
        <div style="font-size:11px; color:#6b7280;">
            {hint}
        </div>
    </div>
    """,
        unsafe_allow_html=True,
    )


def build_dashboard() -> None:
    inject_page_style()

    st.sidebar.title("Controls")
    mode = st.sidebar.radio(
        "Mode",
        options=["Live", "Replay", "Baseline"],
        horizontal=True,
        help="Live = current logs; Replay = bundled AWR grid slice logs; Baseline = frozen snapshot under analysis/baselines/baseline_v1/.",
    )
    refresh_mode = st.sidebar.selectbox(
        "Auto Refresh",
        options=["Off", "2s", "5s"],
        index=0,
        help="Live mode only; Replay and Baseline stay static.",
    )
    refresh_map = {
        "Off": None,
        "2s": 2000,
        "5s": 5000,
    }
    refresh_interval = refresh_map.get(refresh_mode)

    st.sidebar.subheader("Experiments")
    registry_path = st.sidebar.text_input("Registry path", value=REGISTRY_PATH)
    state_filter = st.sidebar.multiselect(
        "State",
        options=["active", "validated", "archived"],
        default=["active", "validated"],
    )
    decision_filter = st.sidebar.multiselect(
        "Decision",
        options=["PROMOTE", "HOLD", "REJECT"],
        default=["PROMOTE", "HOLD", "REJECT"],
    )
    min_retention = st.sidebar.slider("Min Retention %", 0, 100, 0)
    min_positive_ratio = st.sidebar.slider("Min Positive Ratio", 0.0, 1.0, 0.0)

    st.sidebar.subheader("Diagnostics (engine logs)")
    diag_default_dir, diag_default_glob = infer_default_live_diag_paths()
    diag_dir = st.sidebar.text_input(
        "Log directory",
        value=diag_default_dir,
        help=(
            "If `analysis/live_multi` exists on disk, defaults target multi-lane logs; "
            "otherwise `analysis/awr_grid` + single `live_run.log`."
        ),
    )
    diag_pattern = st.sidebar.text_input(
        "Glob pattern",
        value=diag_default_glob,
        help="Use `live_*.log*` for per-symbol lanes under `analysis/live_multi`.",
    )
    max_diag_files = st.sidebar.number_input("Max log files", min_value=1, value=50, step=1)
    st.sidebar.markdown("**Feed Health**")
    stale_threshold = st.sidebar.slider(
        "Feed stale threshold (sec)",
        min_value=2,
        max_value=30,
        value=5,
        step=1,
        help="Time since last log update before feed is considered stale",
    )
    expected_symbols_raw = st.sidebar.text_input(
        "Expected symbols (comma-separated)",
        value="BTC-USD,ETH-USD,SOL-USD",
        help="Compared to observed lanes when using live_* multi-lane logs (read-only check).",
    )
    expected_symbols = [
        s.strip().upper() for s in expected_symbols_raw.split(",") if s.strip()
    ]

    diag_dir_use = diag_dir
    diag_pattern_use = diag_pattern
    if mode == "Baseline":
        diag_dir_use = BASELINE_V1_DIAG_DIR
        diag_pattern_use = REPLAY_DIAG_GLOB
        st.sidebar.info(
            "Baseline uses frozen `analysis/baselines/baseline_v1/` (diag_logs + registry when present). "
            "Sidebar log directory / glob are ignored."
        )

    registry_load_path = registry_path
    if mode == "Baseline":
        br = _ROOT / BASELINE_V1_REGISTRY
        if br.is_file():
            registry_load_path = BASELINE_V1_REGISTRY
        else:
            st.sidebar.warning(
                "Baseline registry missing — run `bash scripts/snapshot_baseline.sh` to populate `baseline_v1/`."
            )

    st.title("ChronoSentiment")
    st.caption("Live experiment registry + engine signal readiness — one view")
    if mode == "Live" and refresh_interval:
        if st_autorefresh is not None:
            st_autorefresh(interval=refresh_interval, key="live_refresh")
            st.caption(f"🔄 Auto-refresh every {refresh_mode}")
        else:
            st.warning("Auto-refresh requested but `streamlit-autorefresh` is not installed.")
    if mode == "Replay":
        if refresh_interval:
            st.warning("Auto-refresh disabled in Replay mode (data is deterministic).")
        st.info("🔁 Replay Mode — deterministic historical diagnostics")
        st.caption("Source is fixed unless replay settings change.")
    elif mode == "Baseline":
        if refresh_interval:
            st.warning("Auto-refresh disabled in Baseline mode (frozen snapshot).")
        st.info("📌 Baseline Mode — frozen reference snapshot")
        st.caption(
            f"Diagnostics: `{BASELINE_V1_DIAG_DIR}` + `{REPLAY_DIAG_GLOB}` · "
            f"Registry: `{BASELINE_V1_REGISTRY}` when present."
        )
        meta_path = _ROOT / BASELINE_V1_METADATA
        if meta_path.is_file():
            try:
                meta = json.loads(meta_path.read_text(encoding="utf-8"))
                name = meta.get("name", "baseline_v1")
                created = meta.get("created_at", "—")
                st.caption(f"Metadata: {name} · created {created}")
            except (OSError, json.JSONDecodeError):
                pass
    else:
        st.markdown("**⚡ Live Mode**")
        st.caption(
            "Each refresh re-reads configured log paths; pipeline and lane status are shown after diagnostics load."
        )

    # --- Data loads ---
    records = load_registry(registry_load_path)
    latest = latest_per_hypothesis(records)
    df = to_dataframe(latest)
    df_filtered = pd.DataFrame()
    if not df.empty:
        df_filtered = df[
            (df["state"].isin(state_filter))
            & (df["decision"].isin(decision_filter))
            & (df["retained_pct"] >= min_retention)
            & (df["positive_ratio"] >= min_positive_ratio)
        ].sort_values(
            by=["avg_pnl_delta", "hit_rate_delta", "drawdown_delta"],
            ascending=[False, False, True],
        )

    selected_paths = resolve_log_paths(mode, diag_dir_use, diag_pattern_use, int(max_diag_files))
    diag_result: dict[str, Any] = {"rows": [], "summary": {}, "parse_errors": []}
    multi_results: dict[str, dict[str, Any]] = {}
    is_multi_view = mode == "Live" and "live_" in diag_pattern_use
    if selected_paths:
        with st.spinner("Analyzing diagnostic logs..."):
            if is_multi_view:
                multi_results = run_multi_symbol_diagnostics(
                    diag_dir_use, diag_pattern_use, int(max_diag_files)
                )
                all_rows: list[dict[str, Any]] = []
                all_parse_errors: list[str] = []
                for sym, res in multi_results.items():
                    for rr in res.get("rows", []):
                        row = dict(rr)
                        row["symbol"] = sym
                        all_rows.append(row)
                    all_parse_errors.extend(res.get("parse_errors", []))
                diag_result = {
                    "rows": all_rows,
                    "summary": summarize_multi_symbol(multi_results),
                    "parse_errors": all_parse_errors,
                }
            else:
                path_key = tuple(str(p.resolve()) for p in selected_paths)
                diag_result = cached_run_diagnostics(path_key)

    rows = diag_result.get("rows", [])
    summary = diag_result.get("summary", {})
    parse_errors = diag_result.get("parse_errors", [])
    ticker_ts = extract_ticker_freshness(rows)
    if mode == "Live" and selected_paths:
        live_ticker_ts = extract_symbol_ts_from_log_files(selected_paths)
        if live_ticker_ts:
            ticker_ts = live_ticker_ts
    live_summary_pending = mode == "Live" and not rows and bool(ticker_ts)
    if live_summary_pending:
        summary = {
            "files_parsed": 0,
            "offsets_with_near_bearish": 0,
            "offsets_with_bearish_events": 0,
            "offsets_with_sell_candidates": 0,
            "offsets_with_final_sell": 0,
            "recommendation": "PENDING_STREAM_SUMMARY",
        }
    ticker_map = build_per_ticker_summary(rows)
    formation_metrics = compute_formation_metrics(rows)

    n_sl = int(summary.get("files_parsed", 0) or 0)
    if n_sl < 1:
        n_sl = 1
    near_n = int(summary.get("offsets_with_near_bearish", 0) or 0)
    bear_n = int(summary.get("offsets_with_bearish_events", 0) or 0)
    sell_c_n = int(summary.get("offsets_with_sell_candidates", 0) or 0)
    fin_n = int(summary.get("offsets_with_final_sell", 0) or 0)
    rec = str(summary.get("recommendation", "UNKNOWN"))

    st.caption(f"Last refresh: {datetime.now().strftime('%H:%M:%S')}")
    b_meta = read_baseline_metadata()
    b_ref = baseline_reference_counts(b_meta)
    cur_snap = {"near": near_n, "sell_c": sell_c_n, "sell_f": fin_n}
    if b_ref is not None:
        ic, ttl, dtl = baseline_alignment_badge(cur_snap, b_ref, n_sl)
        render_baseline_alignment_badge(ic, ttl, dtl)
    render_baseline_comparison_panel(mode, near_n, sell_c_n, fin_n, rec, b_ref)
    feed_live = is_feed_live(diag_dir_use, diag_pattern_use, int(stale_threshold))
    pipeline_alive = True
    if mode == "Live":
        pipeline_alive = render_live_pipeline_observability(
            feed_live=feed_live,
            is_multi_view=is_multi_view,
            multi_results=multi_results,
            expected_symbols=expected_symbols,
            stale_threshold=int(stale_threshold),
            diag_dir=diag_dir_use,
            diag_pattern=diag_pattern_use,
        )

    if parse_errors and not live_summary_pending:
        st.warning(f"{len(parse_errors)} log path(s) could not be parsed — check paths or format.")
    if live_summary_pending:
        st.caption(
            "Log ticker activity is present; full aggregate diagnostics appear when summary lines are emitted."
        )
    if mode == "Live":
        if ticker_ts:
            render_fresh_ticker_pills(ticker_ts, int(stale_threshold))
        else:
            st.caption("No per-ticker freshness in this log format yet.")
        if ticker_map:
            render_per_ticker_panel(ticker_map, max_cards=4)
        if multi_results:
            st.markdown("### 📊 Per-Symbol Decisions")
            symbols_sorted = sorted(multi_results.keys())
            cols = st.columns(min(3, len(symbols_sorted)))
            for i, sym in enumerate(symbols_sorted):
                with cols[i % len(cols)]:
                    st.markdown(f"**{sym}**")
                    sym_summary = multi_results[sym].get("summary", {}) or {}
                    sym_phase = compute_phase(sym_summary) if sym_summary else "LONG_ONLY"
                    sym_near = int(sym_summary.get("offsets_with_near_bearish", 0) or 0)
                    _, sym_slices = compute_recommendation_confidence(sym_summary, sym_near)
                    lane_slice_pending = mode == "Live" and sym_slices == 0
                    render_recommendation_bar(
                        sym_summary,
                        sym_phase,
                        mode=mode,
                        feed_live=feed_live,
                        pipeline_alive=pipeline_alive,
                        pending_first_slice=lane_slice_pending,
                        is_per_lane=True,
                        is_multi_view=is_multi_view,
                    )

    render_trade_setup_panel(mode, is_multi_view, multi_results, rows, summary, selected_paths)
    if selected_paths:
        sketch_events = collect_sketch_trade_events(selected_paths)
        active_sketch, closed_sketch = build_sketch_trade_state(sketch_events)
        latest_prices = collect_latest_symbol_prices(selected_paths)
        latest_ts = get_latest_event_ts(sketch_events)
        render_sketch_trade_panels(
            active_sketch, closed_sketch, latest_prices, latest_ts, sketch_events
        )
    st.caption(
        "Formation events (window): "
        f"{formation_metrics['formation_events']} observed · "
        f"{formation_metrics['triggered_events']} triggered · "
        f"{formation_metrics['dropped_before_trigger']} dropped before trigger "
        f"(slices={formation_metrics['total_slices']})."
    )

    phase = compute_phase(summary) if summary else "LONG_ONLY"

    # --- Global identity + validation (read-only) ---
    render_global_status_bar(rec, near_n, bear_n, n_sl)
    render_recommendation_bar(
        summary,
        phase,
        mode=mode,
        feed_live=feed_live,
        pipeline_alive=pipeline_alive,
        pending_first_slice=live_summary_pending,
        is_multi_view=is_multi_view,
    )
    st.markdown(
        f"""
### 🧠 System Interpretation
{auto_explanation(summary)}  
Short-side logic remains gated until diagnostic thresholds are satisfied.
"""
    )
    if int(summary.get("offsets_with_near_bearish", 0) or 0) == 0:
        st.caption("⏳ Waiting for structural signal conditions...")
    section_rule()
    live_consistency_block(summary, rows)

    if "state_history" not in st.session_state:
        st.session_state.state_history = []
    current_state = {
        "time": datetime.now().strftime("%H:%M:%S"),
        "phase": phase,
        "near": near_n,
        "sell": sell_c_n,
    }

    if "prev_phase" not in st.session_state:
        st.session_state.prev_phase = phase
        st.session_state.state_history = [current_state]
    elif st.session_state.prev_phase != phase:
        st.markdown(
            f"""
    <div style="
        padding:14px;
        border-radius:12px;
        background: linear-gradient(90deg, #2563eb33, #111827);
        border:1px solid #2563eb66;
        margin-bottom:12px;
    ">
        <b style="color:#60a5fa;">⚡ Phase transition</b><br>
        <span style="color:#cbd5e1;">
        {st.session_state.prev_phase} → {phase}
        </span>
    </div>
    """,
            unsafe_allow_html=True,
        )
        st.caption("State updated based on latest diagnostic window.")
        st.session_state.prev_phase = phase
        history = list(st.session_state.state_history)
        history.append(current_state)
        st.session_state.state_history = history[-5:]

    phase_index = {
        "LONG_ONLY": 1,
        "MONITORING": 2,
        "READY_FOR_MAPPING": 3,
        "SHORT_ACTIVE": 4,
    }.get(phase, 1)
    phase_text = {
        "LONG_ONLY": "No bearish structure detected",
        "MONITORING": "Early bearish signals detected",
        "READY_FOR_MAPPING": "Consistent bearish pressure emerging",
        "SHORT_ACTIVE": "Short signals active in diagnostic slices",
    }
    st.caption(f"Phase: {phase_text.get(phase, phase)}")
    inject_phase_progress_color(phase)
    st.progress(phase_index / 4.0)
    if phase == "READY_FOR_MAPPING":
        st.markdown(
            """
    <div style="
        padding:10px;
        border-radius:10px;
        background:#1e3a8a33;
        border:1px solid #3b82f6;
        margin-bottom:10px;
        color:#bfdbfe;
    ">
    System ready for short-side evaluation (shadow mode) — governance applies.
    </div>
    """,
            unsafe_allow_html=True,
        )

    render_state_change_strip(st.session_state.state_history)
    section_rule()

    # --- Status ribbon (compact metrics) ---
    st.markdown("#### System status")
    r2, r3, r4, r5 = st.columns(4)
    with r2:
        st.metric("Hypotheses (registry)", len(df_filtered) if not df_filtered.empty else 0)
    with r3:
        top_dec = str(df_filtered.iloc[0]["decision"]) if not df_filtered.empty else "—"
        st.metric("Top filter decision", top_dec)
    with r4:
        st.metric("Diagnostic slices", n_sl)
    with r5:
        st.metric("Runner recommendation", rec)

    section_rule()

    # --- Two-column live + diagnostics ---
    left, right = st.columns(2, gap="large")

    with left:
        section_header("Experiment registry (live)")
        if df_filtered.empty:
            st.info("No rows after filters, or empty registry. Adjust sidebar or append batch runs.")
        else:
            m1, m2, m3, m4 = st.columns(4)
            m1.metric("Avg PnL Δ", f"{float(df_filtered['avg_pnl_delta'].mean()):+.5f}")
            m2.metric("Hit rate Δ", f"{float(df_filtered['hit_rate_delta'].mean()):+.2f}")
            m3.metric("DD Δ", f"{float(df_filtered['drawdown_delta'].mean()):+.5f}")
            m4.metric("Retention %", f"{float(df_filtered['retained_pct'].mean()):.2f}")
            st.caption("Top 5 hypotheses by filtered sort order")
            st.dataframe(
                df_filtered.head(5),
                use_container_width=True,
                height=260,
            )

    with right:
        section_header("Signal readiness (engine)")
        if not selected_paths:
            st.warning(
                f"No logs matched `{diag_pattern_use}` under `{diag_dir_use}`. "
                "Point to stderr captures from `live_engine`, or run `scripts/snapshot_baseline.sh` for Baseline."
            )
        else:
            prev_near = st.session_state.get("trend_near_n")
            prev_n_sl = st.session_state.get("trend_n_sl")
            st.session_state["trend_near_n"] = near_n
            st.session_state["trend_n_sl"] = n_sl
            if prev_near is not None and prev_n_sl is not None and prev_n_sl == n_sl:
                d_off = near_n - int(prev_near)
                if d_off > 0:
                    delta_s = f"+{d_off}"
                elif d_off < 0:
                    delta_s = str(d_off)
                else:
                    delta_s = "0"
            else:
                delta_s = "—"
            st.metric("Near-bearish (offsets)", f"{near_n} / {n_sl}", delta=delta_s)
            st.caption("Δ vs previous refresh (same slice count). Change “Max log files” resets trend.")

            d1, d2, d3, d4 = st.columns(4)
            fin_s = int(summary.get("offsets_with_final_sell", 0) or 0)
            with d1:
                metric_card("Near-bearish slices", f"{near_n} / {n_sl}")
            with d2:
                metric_card("Bearish events", f"{bear_n} / {n_sl}")
            with d3:
                metric_card("SELL candidates", f"{sell_c_n} / {n_sl}")
            with d4:
                metric_card("Final SELL", f"{fin_s} / {n_sl}")
            st.caption("Latest diagnostic slices (tail of sorted glob)")
            if rows:
                slim = []
                for r in rows[-5:]:
                    slim.append(
                        {
                            "Offset": extract_offset_label(r["file_name"]),
                            "Bear": r["raw_tendency"]["bearish_events"],
                            "Near": r["component_diagnostic"]["near_bearish"],
                            "SELL_c": r["side_distribution"]["candidates_sell"],
                            "Class": r["classification"],
                        }
                    )
                st.dataframe(pd.DataFrame(slim), use_container_width=True, height=260)

    section_rule()

    # --- Detail expanders ---
    ex_a, ex_b = st.columns(2)
    with ex_a:
        with st.expander("Full leaderboard", expanded=False):
            if df_filtered.empty:
                st.write("—")
            else:
                st.dataframe(df_filtered, use_container_width=True, height=400)
                if len(df_filtered) > 0:
                    best = df_filtered.iloc[0]
                    st.markdown(
                        f"**Spotlight:** `{best['hypothesis_id']}` · {best['decision']} · {best['state']}"
                    )
    with ex_b:
        with st.expander("Full diagnostic evidence", expanded=False):
            if not rows:
                st.write("—")
            else:
                table_rows = []
                for r in rows:
                    table_rows.append(
                        {
                            "Offset": extract_offset_label(r["file_name"]),
                            "Bull": r["raw_tendency"]["bullish_events"],
                            "Bear": r["raw_tendency"]["bearish_events"],
                            "Near": r["component_diagnostic"]["near_bearish"],
                            "BUY_c": r["side_distribution"]["candidates_buy"],
                            "SELL_c": r["side_distribution"]["candidates_sell"],
                            "Final SELL": r["side_distribution"]["final_sell"],
                            "Class": r["classification"],
                        }
                    )
                st.dataframe(pd.DataFrame(table_rows), use_container_width=True, height=400)
            if parse_errors:
                st.error("Some paths failed to parse:")
                for e in parse_errors:
                    st.text(e)

    st.markdown("#### Phase 2 unlock checklist")
    uc1, uc2, uc3 = st.columns(3)
    with uc1:
        st.checkbox(
            "Near-bearish ≥ 4 / N",
            value=near_n >= 4,
            disabled=True,
        )
    with uc2:
        st.checkbox(
            "Bearish events ≥ 2 slices",
            value=bear_n >= 2,
            disabled=True,
        )
    with uc3:
        st.checkbox(
            "SELL candidates observed",
            value=sell_c_n > 0,
            disabled=True,
        )

    st.caption(
        "Deterministic · Read-only · No strategy mutation · "
        "`chronosentiment-core.mdc`"
    )


build_dashboard()
