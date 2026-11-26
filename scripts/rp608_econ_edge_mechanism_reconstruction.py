#!/usr/bin/env python3
"""
RP-608-ECON-EDGE-DIAG-001 — Step 2 row-level mechanism reconstruction.

Answers the four postmortem questions against individual closed rows.
Assigns primary_mechanism ONLY when row evidence supports it; otherwise UNRESOLVED.
Counterfactual exits are recorded separately and are NOT blended into realized outcomes.

Authority 0%. Rules frozen. Books never pooled.
"""

from __future__ import annotations

import argparse
import json
from collections import Counter, defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

BOOKS = (
    ("nse_estimand_a", "INR", "₹"),
    ("nse_of_paper", "INR", "₹"),
    ("crypto_recommendation_plan", "USDT", "$"),
    ("crypto_of_paper", "USDT", "$"),
)

MFE_EPS = 1e-9
IST = timezone(timedelta(hours=5, minutes=30))

# Closed-book cohort for session_date_ist: trades whose *exit* falls on that IST date.
# Entry may be prior-session (carryover). This must be stated explicitly in artifacts.
SESSION_COHORT_DEFINITION = "CLOSED_TRADES_WITH_EXIT_DATE_IST"

# Mechanisms that may be assigned from realized-row evidence alone.
# Exit-policy counterfactuals and OF-incremental tests are out of scope here
# unless an explicit counterfactual artifact for THIS session is supplied.
SUPPORTED_FROM_REALIZED = {
    "FRICTION_DOMINANCE",
    "ENTRY_PATH_NEVER_FAVORABLE",
}


def fnum(r: Dict[str, Any], *keys: str) -> Optional[float]:
    for k in keys:
        if r.get(k) is None:
            continue
        try:
            return float(r[k])
        except (TypeError, ValueError):
            continue
    return None


def trade_id(r: Dict[str, Any]) -> str:
    for k in ("trade_id", "of_trade_id", "plan_id"):
        if r.get(k) is not None:
            return str(r[k])
    return "?"


def pnl_fields(r: Dict[str, Any], currency: str) -> Tuple[Optional[float], Optional[float], Optional[float]]:
    if currency == "INR":
        g = fnum(r, "gross_pnl_inr", "gross_pnl", "touch_gross_pnl_inr")
        n = fnum(r, "net_pnl_inr", "net_pnl", "all_in_net_pnl_inr")
        fr = fnum(r, "friction_inr", "statutory_overhead_inr", "fees_inr")
    else:
        g = fnum(r, "gross_pnl_usdt", "gross_pnl")
        n = fnum(r, "net_pnl_usdt", "net_pnl")
        fr = fnum(r, "friction_usdt", "fees_usdt", "est_friction_usdt")
    if fr is None and g is not None and n is not None:
        fr = g - n
    return g, fr, n


def primary_accounting(gross: Optional[float], net: Optional[float]) -> str:
    if gross is None or net is None:
        return "UNRESOLVED_PNL_FIELDS"
    if net > 0:
        return "NET_POSITIVE"
    if net == 0:
        return "NET_FLAT"
    if gross >= 0:
        return "FRICTION_DOMINATED"
    return "GROSS_ADVERSE"


def path_from_mfe_bps(mfe_bps: Optional[float]) -> str:
    if mfe_bps is None:
        return "PATH_METRICS_UNAVAILABLE"
    if mfe_bps <= MFE_EPS:
        return "PATH_NEVER_FAVORABLE"
    return "PATH_HAD_FAVORABLE"


def path_from_exec_prices(r: Dict[str, Any]) -> Optional[str]:
    """Crypto plan stores mfe_exec_price / mae_exec_price rather than mfe_bps."""
    entry = fnum(r, "entry_price")
    mfe_px = fnum(r, "mfe_exec_price")
    if entry is None or mfe_px is None:
        return None
    side = str(r.get("side") or r.get("direction") or "").upper()
    if side == "LONG":
        return "PATH_HAD_FAVORABLE" if mfe_px > entry + MFE_EPS else "PATH_NEVER_FAVORABLE"
    if side == "SHORT":
        return "PATH_HAD_FAVORABLE" if mfe_px < entry - MFE_EPS else "PATH_NEVER_FAVORABLE"
    return None


def resolve_path(r: Dict[str, Any]) -> Tuple[str, str]:
    mfe_bps = fnum(r, "mfe_bps", "mfe")
    if mfe_bps is not None:
        return path_from_mfe_bps(mfe_bps), "mfe_bps"
    exec_path = path_from_exec_prices(r)
    if exec_path is not None:
        return exec_path, "mfe_exec_price_vs_entry"
    return "PATH_METRICS_UNAVAILABLE", "none"


def parse_dt(v: Any) -> Optional[datetime]:
    if v is None:
        return None
    try:
        if isinstance(v, (int, float)):
            ms = int(v)
            if ms > 10_000_000_000:
                ms //= 1000
            return datetime.fromtimestamp(ms, tz=timezone.utc)
        dt = datetime.fromisoformat(str(v).replace("Z", "+00:00"))
        if dt.tzinfo is None:
            dt = dt.replace(tzinfo=timezone.utc)
        return dt
    except Exception:
        return None


def entry_hour_ist(r: Dict[str, Any]) -> Optional[str]:
    for k in (
        "entry_utc",
        "entry_timestamp_utc_ms",
        "paper_open_timestamp",
        "activated_ms",
        "entry_time_ms",
    ):
        dt = parse_dt(r.get(k))
        if dt is None:
            continue
        return dt.astimezone(IST).strftime("%H")
    return None


def ist_date(v: Any) -> Optional[str]:
    dt = parse_dt(v)
    if dt is None:
        return None
    return dt.astimezone(IST).date().isoformat()


def path_comparison_block(
    r: Dict[str, Any], *, path: str, path_source: str
) -> Dict[str, Any]:
    """Inputs required to independently reproduce path_descriptor."""
    entry = fnum(r, "entry_price")
    mfe_px = fnum(r, "mfe_exec_price")
    mae_px = fnum(r, "mae_exec_price")
    mfe_bps = fnum(r, "mfe_bps", "mfe")
    side = str(r.get("side") or r.get("direction") or "?").upper()
    out: Dict[str, Any] = {
        "path_descriptor": path,
        "path_evidence_source": path_source,
        "side": side,
        "entry_price": entry,
        "exit_price": fnum(r, "exit_price"),
        "mfe_bps": mfe_bps,
        "mfe_exec_price": mfe_px,
        "mae_exec_price": mae_px,
        "eps": MFE_EPS,
    }
    if path_source == "mfe_exec_price_vs_entry":
        out["rule"] = (
            "LONG: PATH_HAD_FAVORABLE iff mfe_exec_price > entry_price + eps; else PATH_NEVER_FAVORABLE. "
            "SHORT: PATH_HAD_FAVORABLE iff mfe_exec_price < entry_price - eps; else PATH_NEVER_FAVORABLE."
        )
        if entry is not None and mfe_px is not None and side in ("LONG", "SHORT"):
            if side == "LONG":
                out["reproducible_result"] = (
                    "PATH_HAD_FAVORABLE" if mfe_px > entry + MFE_EPS else "PATH_NEVER_FAVORABLE"
                )
            else:
                out["reproducible_result"] = (
                    "PATH_HAD_FAVORABLE" if mfe_px < entry - MFE_EPS else "PATH_NEVER_FAVORABLE"
                )
            out["matches_path_descriptor"] = out["reproducible_result"] == path
        else:
            out["reproducible_result"] = None
            out["matches_path_descriptor"] = False
            out["gap"] = "entry_price or mfe_exec_price missing — path not independently reproducible"
    elif path_source == "mfe_bps":
        out["rule"] = "PATH_NEVER_FAVORABLE iff mfe_bps <= eps; else PATH_HAD_FAVORABLE"
        if mfe_bps is not None:
            out["reproducible_result"] = path_from_mfe_bps(mfe_bps)
            out["matches_path_descriptor"] = out["reproducible_result"] == path
        else:
            out["reproducible_result"] = None
            out["matches_path_descriptor"] = False
    else:
        out["rule"] = None
        out["reproducible_result"] = "PATH_METRICS_UNAVAILABLE"
        out["matches_path_descriptor"] = path == "PATH_METRICS_UNAVAILABLE"
    return out


def assign_mechanism(
    *,
    accounting: str,
    path: str,
    path_source: str,
) -> Tuple[str, str, str, List[str]]:
    """
    Returns: primary_mechanism, confidence, evidence_basis, unresolved_questions
    """
    unresolved = [
        "signal_quality",
        "exit_quality",
        "friction_as_cause",
        "concentration_as_cause",
    ]

    if accounting == "NET_POSITIVE":
        return (
            "NOT_APPLICABLE_NET_POSITIVE",
            "N/A",
            "net>0; loss-mechanism questions do not apply to this row",
            [],
        )
    if accounting == "NET_FLAT":
        return (
            "NOT_APPLICABLE_NET_FLAT",
            "N/A",
            "net==0",
            [],
        )
    if accounting == "UNRESOLVED_PNL_FIELDS":
        return (
            "UNRESOLVED",
            "NONE",
            "gross or net missing",
            unresolved,
        )

    if accounting == "FRICTION_DOMINATED":
        # Definitional for this row's net composition: gross≥0 and net<0.
        # Does NOT claim the entry should/shouldn't have been taken, nor that an
        # alternative exit would have improved the row.
        return (
            "FRICTION_DOMINANCE",
            "HIGH",
            "primary_accounting=FRICTION_DOMINATED (gross≥0, net<0); net loss equals friction−gross on this row",
            ["signal_quality", "exit_quality", "concentration_as_cause"],
        )

    # GROSS_ADVERSE
    if path == "PATH_NEVER_FAVORABLE" and path_source != "none":
        return (
            "ENTRY_PATH_NEVER_FAVORABLE",
            "MEDIUM",
            f"gross<0 and {path_source} shows no favorable excursion before exit; "
            "does not establish which entry rule (e.g. G3) caused the path, "
            "and does not establish that an alternative exit would have helped",
            ["exit_quality", "friction_as_cause", "concentration_as_cause"],
        )

    if path == "PATH_HAD_FAVORABLE":
        return (
            "UNRESOLVED",
            "NONE",
            "gross<0 after a favorable excursion — signal fade vs exit timing "
            "not separable without a session-matched exit counterfactual",
            unresolved,
        )

    return (
        "UNRESOLVED",
        "NONE",
        "gross<0 with path metrics unavailable — signal vs exit not separable from realized row alone; "
        "observed exit_reason is metadata only (reverse-OF / stop / time exit ≠ established cause)",
        unresolved,
    )


def load_jsonl(path: Path) -> List[Dict[str, Any]]:
    if not path.exists():
        return []
    out: List[Dict[str, Any]] = []
    for line in path.read_text(encoding="utf-8", errors="ignore").splitlines():
        if not line.strip():
            continue
        try:
            out.append(json.loads(line))
        except Exception:
            continue
    return out


def reconstruct_book(
    *,
    book_key: str,
    rows: List[Dict[str, Any]],
    currency: str,
    reported_book_net: Optional[float],
    session_date_ist: str,
) -> Dict[str, Any]:
    evidence: List[Dict[str, Any]] = []
    mech_counts: Counter = Counter()
    mech_net: Dict[str, float] = defaultdict(float)
    symbol_counts: Counter = Counter()
    hour_counts: Counter = Counter()
    side_counts: Counter = Counter()
    carryovers: List[str] = []
    identity_nonzero: List[Dict[str, Any]] = []

    for r in rows:
        sym = str(r.get("symbol") or r.get("tradingsymbol") or "?")
        symbol_counts[sym] += 1
    # second pass after we know repeat exposures
    for r in rows:
        g, fr, n = pnl_fields(r, currency)
        accounting = primary_accounting(g, n)
        path, path_source = resolve_path(r)
        mech, conf, basis, unresolved_q = assign_mechanism(
            accounting=accounting, path=path, path_source=path_source
        )
        side = str(r.get("side") or r.get("direction") or "?").upper()
        hour = entry_hour_ist(r)
        sym = str(r.get("symbol") or r.get("tradingsymbol") or "?")
        exit_r = str(r.get("exit_reason") or r.get("close_reason") or "UNRESOLVED_EXIT_REASON")
        tid = trade_id(r)
        entry_event = (
            r.get("entry_utc")
            or r.get("paper_open_timestamp")
            or r.get("activated_ms")
            or r.get("entry_time_ms")
        )
        exit_event = (
            r.get("exit_utc")
            or r.get("paper_close_timestamp")
            or r.get("exited_ms")
            or r.get("exit_time_ms")
        )
        entry_d = ist_date(entry_event)
        exit_d = ist_date(exit_event)
        carryover = bool(entry_d and exit_d and entry_d != session_date_ist and exit_d == session_date_ist)
        if carryover:
            carryovers.append(tid)

        identity_residual = None
        if g is not None and fr is not None and n is not None:
            identity_residual = round(g - fr - n, 10)
            if abs(identity_residual) > 0:
                identity_nonzero.append(
                    {
                        "trade_id": tid,
                        "gross": g,
                        "friction": fr,
                        "net": n,
                        "gross_minus_friction_minus_net": identity_residual,
                    }
                )

        path_cmp = path_comparison_block(r, path=path, path_source=path_source)

        row = {
            "trade_id": tid,
            "book": book_key,
            "symbol": sym,
            "side": side,
            "entry_hour_ist": hour,
            "entry_event": entry_event,
            "exit_event": exit_event,
            "entry_price": fnum(r, "entry_price"),
            "exit_price": fnum(r, "exit_price"),
            "observed_exit_reason": exit_r,
            "gross": None if g is None else round(g, 6),
            "friction": None if fr is None else round(fr, 6),
            "net": None if n is None else round(n, 6),
            "accounting_identity": {
                "gross_minus_friction_minus_net": identity_residual,
                "exact_at_exported_precision": identity_residual == 0 or identity_residual is None,
                "note": (
                    "Preserve nonzero residuals; do not silently normalize. "
                    "Crypto exports may show cent/aggregation drift between displayed fields."
                ),
            },
            "session_cohort": {
                "definition": SESSION_COHORT_DEFINITION,
                "session_date_ist": session_date_ist,
                "entry_date_ist": entry_d,
                "exit_date_ist": exit_d,
                "carryover_entry_prior_session": carryover,
                "note": (
                    "Cohort = closed trades with exit_date_ist == session_date_ist. "
                    "Not exclusively same-session entries."
                ),
            },
            "mae_bps": fnum(r, "mae_bps", "mae"),
            "mfe_bps": fnum(r, "mfe_bps", "mfe"),
            "mfe_exec_price": fnum(r, "mfe_exec_price"),
            "mae_exec_price": fnum(r, "mae_exec_price"),
            "primary_accounting": accounting,
            "path_descriptor": path,
            "path_evidence_source": path_source,
            "path_comparison": path_cmp,
            "concentration_descriptors": {
                "symbol": sym,
                "entry_hour_ist": hour,
                "side": side,
                "symbol_closed_trade_count_session": symbol_counts[sym],
            },
            "primary_mechanism": mech,
            "confidence": conf,
            "evidence_basis": basis,
            "unresolved_questions": unresolved_q,
            "mechanism_status": "ASSIGNED" if mech in SUPPORTED_FROM_REALIZED else (
                "NOT_APPLICABLE" if mech.startswith("NOT_APPLICABLE") else "UNRESOLVED"
            ),
            "counterfactual_exit": {
                "status": "NOT_COMPUTED_FOR_THIS_SESSION",
                "note": (
                    "Phase-2 A–E PATH_ONLY exists for 2026-11-02 only and must not be "
                    "transferred onto 2026-11-03 rows. Realized exit remains authoritative."
                ),
                "realized_exit_reason": exit_r,
                "counterfactual_net": None,
                "delta_vs_realized_net": None,
            },
            "explicitly_not_claimed": [
                "g3_policy_c_caused_loss",
                "reverse_of_caused_loss",
                "stop_caused_negative_expectancy",
                "time_exit_caused_loss",
                "friction_alone_explains_book",
                "concentration_caused_loss",
                "entry_path_never_favorable_equals_signal_failure",
            ],
        }
        evidence.append(row)
        mech_counts[mech] += 1
        if n is not None:
            mech_net[mech] += n
        if hour:
            hour_counts[hour] += 1
        side_counts[side] += 1

    total_net = sum(r["net"] or 0.0 for r in evidence)
    rounding_note = None
    if reported_book_net is not None:
        delta = round(total_net - reported_book_net, 6)
        if abs(delta) > 0:
            rounding_note = {
                "reconstructed_net_sum": round(total_net, 6),
                "reported_book_net": reported_book_net,
                "delta": delta,
                "treatment": "PRESERVE_AS_ROUNDING_OR_SOURCE_AGGREGATION_NOTE_DO_NOT_SILENTLY_NORMALIZE",
            }

    # Integer display drift note for NSE OF style cases
    display_int_buckets = sum(int(round(v)) for v in mech_net.values())
    book_int = None if reported_book_net is None else int(round(reported_book_net))
    display_note = None
    if book_int is not None and display_int_buckets != book_int:
        display_note = {
            "sum_of_rounded_mechanism_nets": display_int_buckets,
            "rounded_reported_book_net": book_int,
            "delta_inr_or_usd": display_int_buckets - book_int,
            "treatment": "DISPLAY_ROUNDING_ONLY_FULL_PRECISION_IDENTITY_GOVERNS",
        }

    by_sym_net: Dict[str, float] = defaultdict(float)
    for r in evidence:
        by_sym_net[r["symbol"]] += r["net"] or 0.0
    top_loss = sorted(by_sym_net.items(), key=lambda kv: kv[1])[:8]

    total_g = sum(r["gross"] or 0.0 for r in evidence)
    total_f = sum(r["friction"] or 0.0 for r in evidence)
    return {
        "book": book_key,
        "currency": currency,
        "n_closed": len(evidence),
        "session_cohort_definition": SESSION_COHORT_DEFINITION,
        "session_date_ist": session_date_ist,
        "n_carryover_entry_prior_session": len(carryovers),
        "carryover_trade_ids": carryovers,
        "reconstructed_net_sum": round(total_net, 6),
        "reconstructed_gross_sum": round(total_g, 6),
        "reconstructed_friction_sum": round(total_f, 6),
        "aggregate_gross_minus_friction_minus_net": round(total_g - total_f - total_net, 10),
        "reported_book_net": reported_book_net,
        "rounding_or_source_note": rounding_note,
        "integer_display_rounding_note": display_note,
        "accounting_precision": {
            "n_rows_with_nonzero_identity_residual": len(identity_nonzero),
            "rows_with_nonzero_identity_residual": identity_nonzero[:20],
            "treatment": "PRESERVE_DO_NOT_SILENTLY_NORMALIZE",
            "note": (
                "If gross−friction≠net at exported precision, document as source drift. "
                "Mechanism labels use signs, not exact identity."
            ),
        },
        "mechanism_counts": dict(mech_counts),
        "mechanism_net_sum": {k: round(v, 4) for k, v in sorted(mech_net.items())},
        "assigned_supported_n": sum(1 for r in evidence if r["mechanism_status"] == "ASSIGNED"),
        "unresolved_n": sum(1 for r in evidence if r["mechanism_status"] == "UNRESOLVED"),
        "not_applicable_n": sum(1 for r in evidence if r["mechanism_status"] == "NOT_APPLICABLE"),
        "concentration_distributional": {
            "entry_hour_ist_counts": dict(sorted(hour_counts.items())),
            "side_counts": dict(side_counts),
            "top_symbols_by_net_loss": [{"symbol": s, "net": round(v, 4)} for s, v in top_loss],
            "note": "Distributional descriptors only — never primary_mechanism.",
        },
        "rows": evidence,
    }


def to_markdown(rep: Dict[str, Any]) -> str:
    day = rep["session_date_ist"]
    lines = [
        f"# RP-608 Mechanism Reconstruction — {day}",
        "",
        f"**Generated:** {rep['generated_utc']}  ",
        "**Authority:** 0.0% · rules frozen · books not pooled  ",
        f"**Source export:** `{rep['source_export']}`  ",
        f"**Accounting layer:** `{rep['accounting_attribution']}`  ",
        f"**Method:** `{rep['method']}`  ",
        "",
        "## Scope",
        "",
        "Row-level answers to: (1) signal quality, (2) exit quality, (3) friction, (4) concentration.",
        "",
        f"- **Session cohort definition:** `{SESSION_COHORT_DEFINITION}` — closed trades with "
        f"**exit_date_ist == {day}**. Entry may be a prior session (carryover). "
        "Counts are **not** exclusively same-session entries.",
        "- **Primary mechanism** assigned only when realized-row evidence supports it.",
        "- Path assignments using `mfe_exec_price_vs_entry` include `entry_price`, `mfe_exec_price`, "
        "  and a `path_comparison` block so the descriptor is independently reproducible.",
        "- `ENTRY_PATH_NEVER_FAVORABLE` ≠ signal failure / bad entry (explicit non-claim).",
        "- **Counterfactual exits** are separately identified and are **not computed** for this session here",
        "  (Phase-2 A–E PATH_ONLY is 2026-11-02 only — not transferable).",
        "- Observed `exit_reason` (including reverse-OF / stops / time exits) is **metadata**, not a cause.",
        "- Concentration is **distributional**, never `primary_mechanism`.",
        "- Crypto accounting identity residuals are **preserved** (not silently normalized).",
        "",
        "## Supported mechanism labels (from realized rows only)",
        "",
        "| Label | When assigned | Confidence |",
        "| --- | --- | --- |",
        "| `FRICTION_DOMINANCE` | `FRICTION_DOMINATED` accounting (gross≥0, net<0) | HIGH |",
        "| `ENTRY_PATH_NEVER_FAVORABLE` | `GROSS_ADVERSE` + never-favorable path evidence | MEDIUM |",
        "| `UNRESOLVED` | Otherwise for losing rows | NONE |",
        "| `NOT_APPLICABLE_*` | Net-positive / flat rows | N/A |",
        "",
    ]

    for key, _c, sym in BOOKS:
        b = rep["books"][key]
        lines.append(f"## {key}")
        lines.append(
            f"- n={b['n_closed']} · reconstructed net {sym}{b['reconstructed_net_sum']} · "
            f"reported book net {sym}{b['reported_book_net']}"
        )
        if b.get("rounding_or_source_note"):
            lines.append(f"- **Rounding/source note (preserved):** `{b['rounding_or_source_note']}`")
        if b.get("integer_display_rounding_note"):
            lines.append(
                f"- **Integer display rounding note (preserved, not normalized):** "
                f"`{b['integer_display_rounding_note']}`"
            )
        lines.append(
            f"- mechanism status: assigned={b['assigned_supported_n']} · "
            f"unresolved={b['unresolved_n']} · not_applicable={b['not_applicable_n']}"
        )
        lines.append(
            f"- carryovers (entry prior session, exit on {day}): "
            f"**{b['n_carryover_entry_prior_session']}** · ids `{b['carryover_trade_ids']}`"
        )
        ap = b.get("accounting_precision") or {}
        lines.append(
            f"- accounting identity: aggregate gross−friction−net = "
            f"`{b.get('aggregate_gross_minus_friction_minus_net')}` · "
            f"rows with nonzero residual = **{ap.get('n_rows_with_nonzero_identity_residual', 0)}** "
            f"(preserved, not normalized)"
        )
        lines.append("")
        lines.append("| primary_mechanism | n | net sum |")
        lines.append("| --- | ---: | ---: |")
        for mech, n in sorted(b["mechanism_counts"].items(), key=lambda kv: (-kv[1], kv[0])):
            lines.append(
                f"| `{mech}` | {n} | {sym}{b['mechanism_net_sum'].get(mech, 0)} |"
            )
        lines.append("")
        c = b["concentration_distributional"]
        lines.append(
            f"- concentration (distributional): hours `{c['entry_hour_ist_counts']}` · "
            f"sides `{c['side_counts']}` · top losses `{c['top_symbols_by_net_loss'][:5]}`"
        )
        lines.append("")
        # Evidence excerpt: assigned rows first, then a few unresolved
        assigned = [r for r in b["rows"] if r["mechanism_status"] == "ASSIGNED"]
        unresolved = [r for r in b["rows"] if r["mechanism_status"] == "UNRESOLVED"]
        lines.append("### Evidence excerpt (full table in JSONL)")
        lines.append("")
        lines.append(
            "| trade_id | symbol | acct | path | observed_exit | mech | conf | net |"
        )
        lines.append("| --- | --- | --- | --- | --- | --- | --- | ---: |")
        sample = assigned[:8] + unresolved[:4]
        for r in sample:
            lines.append(
                f"| `{r['trade_id']}` | {r['symbol']} | `{r['primary_accounting']}` | "
                f"`{r['path_descriptor']}` | `{r['observed_exit_reason']}` | "
                f"`{r['primary_mechanism']}` | {r['confidence']} | {sym}{r['net']} |"
            )
        lines.append("")
        lines.append(
            f"Full evidence rows: `mechanism_reconstruction_{day}_{key}_rows.jsonl` "
            f"({len(b['rows'])} rows)."
        )
        lines.append("")

    lines.extend(
        [
            "## Exit counterfactual status (separate from realized)",
            "",
            "| Session | Artifact | Usable on 2026-11-03 rows? |",
            "| --- | --- | --- |",
            "| 2026-11-02 | `artifacts/rp608_exit_falsification_001/phase2_path_only_report_2026-11-02.md` | **No** — different session entries |",
            "| 2026-11-03 | *(not computed in this deliverable)* | Pending separate PATH_ONLY run if authorized as research-only |",
            "",
            "Gate R remains **not reached**. No production exit change is authorized.",
            "",
            "## What remains unresolved by design",
            "",
            "- Whether G3 Policy C caused NSE A losses.",
            "- Whether reverse-OF exits caused NSE OF / crypto OF losses.",
            "- Whether stops caused crypto-plan negative expectancy.",
            "- Whether any alternative exit would have improved 3 Nov (requires session-matched CF).",
            "- Whether concentration *caused* losses (only distributional shares reported).",
            "",
            "## Governance",
            "",
            "- Rules frozen · authority 0% · books separate · no rule proposals from this artifact.",
            "",
        ]
    )
    return "\n".join(lines)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--day", default="2026-11-03")
    ap.add_argument("--export-dir", type=Path, required=True)
    ap.add_argument("--out-dir", type=Path, required=True)
    args = ap.parse_args()
    args.out_dir.mkdir(parents=True, exist_ok=True)

    export_path = args.export_dir / f"session_export_{args.day}.json"
    export = json.loads(export_path.read_text()) if export_path.exists() else {"books": {}}

    books_out: Dict[str, Any] = {}
    for key, currency, _sym in BOOKS:
        closed = load_jsonl(args.export_dir / f"session_export_{args.day}_{key}_closed.jsonl")
        reported = None
        try:
            reported = export["books"][key]["summary_closed"]["realized_net_sum"]
        except Exception:
            reported = None
        # User-facing reported nets (preserve their table figures where relevant)
        if key == "nse_of_paper" and reported is not None:
            # Keep full precision; also stash the commonly quoted integer −4688
            pass
        books_out[key] = reconstruct_book(
            book_key=key,
            rows=closed,
            currency=currency,
            reported_book_net=reported,
            session_date_ist=args.day,
        )

    # Explicit NSE OF display note the user called out (−4689 vs −4688)
    of = books_out["nse_of_paper"]
    of["user_reported_integer_net"] = -4688
    of["bucket_integer_sum_example"] = {
        "note": (
            "If primary-accounting bucket nets are rounded to whole rupees independently, "
            "their sum can read −₹4,689 while the book net rounds to −₹4,688. "
            "Full-precision reconstructed net is authoritative; do not silently normalize."
        ),
        "full_precision_net": of["reconstructed_net_sum"],
        "quoted_book_net_inr": -4688,
    }

    rep = {
        "protocol_id": "RP-608-ECON-EDGE-DIAG-001",
        "deliverable_id": f"mechanism_reconstruction_{args.day}",
        "session_date_ist": args.day,
        "generated_utc": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
        "live_order_authority": 0.0,
        "rules_frozen": True,
        "books_pooled": False,
        "source_export": str(export_path),
        "accounting_attribution": str(
            args.export_dir / f"accounting_attribution_{args.day}.md"
        ),
        "method": "docs/research/rp608_econ_edge_session_postmortem_method.md",
        "causal_policy": "ASSIGN_ONLY_WHEN_SUPPORTED_ELSE_UNRESOLVED",
        "counterfactual_policy": "SEPARATE_FROM_REALIZED_NOT_COMPUTED_FOR_THIS_SESSION",
        "session_cohort_definition": SESSION_COHORT_DEFINITION,
        "session_cohort_note": (
            f"All books: closed trades with exit_date_ist == {args.day}. "
            "Carryover entries from a prior IST date are included and flagged."
        ),
        "evidence_sufficiency_gate_status": "BLOCKED_PENDING_RE_REVIEW",
        "books": books_out,
    }

    (args.out_dir / f"mechanism_reconstruction_{args.day}.json").write_text(
        json.dumps(
            {
                **{k: v for k, v in rep.items() if k != "books"},
                "books": {
                    k: {kk: vv for kk, vv in b.items() if kk != "rows"}
                    for k, b in books_out.items()
                },
            },
            indent=2,
            default=str,
        )
    )
    (args.out_dir / f"mechanism_reconstruction_{args.day}.md").write_text(to_markdown(rep))
    for key, _c, _s in BOOKS:
        path = args.out_dir / f"mechanism_reconstruction_{args.day}_{key}_rows.jsonl"
        with path.open("w", encoding="utf-8") as f:
            for r in books_out[key]["rows"]:
                f.write(json.dumps(r, default=str) + "\n")

    print(to_markdown(rep))
    print(f"WROTE {args.out_dir}")


if __name__ == "__main__":
    main()
