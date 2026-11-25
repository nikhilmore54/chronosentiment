"""Rust plan-lifecycle client — Adaptive TP + SL/TP + Gate 2X (cutover).

Python shadows remain I/O + state persistence; lifecycle authority is
`rp608_position_decide --plan-lifecycle-loop` when available.

Default fail-closed: return None so caller may keep Python Adaptive/Gate2X path.
With `RP608_PLAN_LIFECYCLE_REQUIRE_RUST=1`: callers must NOT mutate plans via
Python Adaptive TP / Gate 2X giveback / hard SL-TP manage — log and skip
(TIME_EXIT / NSE bar horizon may remain Python per cutover doc).
Aligned with .cursor/rules/chronosentiment-core.mdc §3c.
"""

from __future__ import annotations

import json
import os
import subprocess
import threading
import time
from pathlib import Path
from typing import Any, Dict, Optional, TextIO, Tuple

_REPO_ROOT = Path(__file__).resolve().parents[1]

DEFAULT_CRYPTO_EVIDENCE = str(
    _REPO_ROOT / "datasets/recommendation/historical_crypto"
)
DEFAULT_NSE_EVIDENCE = str(_REPO_ROOT / "datasets/recommendation/historical")

_SESSIONS: Dict[Tuple[str, str, str], "PlanLifecycleSession"] = {}
_LOCK = threading.Lock()
_REQUIRE_RUST_LOG_TS: Dict[str, float] = {}
_REQUIRE_RUST_LOG_INTERVAL_S = 30.0


def lifecycle_enabled() -> bool:
    return os.environ.get("RP608_PLAN_LIFECYCLE", "1").strip().lower() not in {
        "0",
        "false",
        "no",
        "off",
    }


def require_rust_lifecycle() -> bool:
    """Hard fail-closed: no silent Python Adaptive/Gate2X manage when Rust is down."""
    return os.environ.get("RP608_PLAN_LIFECYCLE_REQUIRE_RUST", "").strip().lower() in {
        "1",
        "true",
        "yes",
        "on",
    }


def log_require_rust_block(book: str, reason: str) -> None:
    """Rate-limited FATAL log when REQUIRE_RUST blocks Python manage fallback."""
    if not require_rust_lifecycle():
        return
    now = time.monotonic()
    key = f"{book}:{reason}"
    last = _REQUIRE_RUST_LOG_TS.get(key, 0.0)
    if now - last < _REQUIRE_RUST_LOG_INTERVAL_S:
        return
    _REQUIRE_RUST_LOG_TS[key] = now
    print(
        f"[FATAL][plan_lifecycle] REQUIRE_RUST book={book} — {reason}; "
        "skipping Python Adaptive/Gate2X manage mutations (fail-closed)",
        flush=True,
    )


def _bin_supports_plan_lifecycle(path: str) -> bool:
    try:
        proc = subprocess.run(
            [path, "--help"],
            capture_output=True,
            text=True,
            timeout=3.0,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired):
        return False
    blob = (proc.stderr or "") + (proc.stdout or "")
    return "--plan-lifecycle-loop" in blob


def resolve_lifecycle_bin() -> Optional[str]:
    from scripts.rp608_gate2x_adapter_client import resolve_decide_bin

    for key in ("RP608_PLAN_LIFECYCLE_BIN", "RP608_GATE2X_MANAGE_BIN", "RP608_POSITION_DECIDE_BIN"):
        env = os.environ.get(key, "").strip()
        if env and Path(env).exists() and _bin_supports_plan_lifecycle(env):
            return env
    base = resolve_decide_bin()
    if base and _bin_supports_plan_lifecycle(base):
        return base
    return None


class PlanLifecycleSession:
    def __init__(self, bin_path: str, evidence_dir: str, book: str) -> None:
        self.bin_path = bin_path
        self.evidence_dir = evidence_dir
        self.book = book
        self._proc: Optional[subprocess.Popen] = None
        self._stdin: Optional[TextIO] = None
        self._stdout: Optional[TextIO] = None
        self._lock = threading.Lock()

    def start(self) -> None:
        if self._proc is not None:
            return
        self._proc = subprocess.Popen(
            [
                self.bin_path,
                "--plan-lifecycle-loop",
                "--book",
                self.book,
                "--evidence-dir",
                self.evidence_dir,
            ],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
        )
        assert self._proc.stdin and self._proc.stdout
        self._stdin = self._proc.stdin
        self._stdout = self._proc.stdout

    def tick(
        self, plan: Dict[str, Any], quote: Dict[str, Any], *, reset_latch: bool = False
    ) -> Optional[Dict[str, Any]]:
        with self._lock:
            try:
                self.start()
                assert self._stdin and self._stdout and self._proc
                payload = {
                    "plan": plan,
                    "quote": quote,
                    "reset_latch": bool(reset_latch),
                }
                self._stdin.write(json.dumps(payload, default=str) + "\n")
                self._stdin.flush()
                line = self._stdout.readline()
                if not line:
                    err = ""
                    try:
                        if self._proc.stderr:
                            err = (self._proc.stderr.readline() or "")[:400]
                    except Exception:
                        pass
                    if err:
                        print(
                            f"[plan_lifecycle] tick empty stdout book={self.book}: {err}",
                            flush=True,
                        )
                    self.close()
                    return None
                return json.loads(line)
            except Exception as exc:
                print(
                    f"[plan_lifecycle] tick failed book={self.book}: {type(exc).__name__}: {exc}",
                    flush=True,
                )
                self.close()
                return None

    def close(self) -> None:
        if self._proc is None:
            return
        try:
            if self._stdin:
                self._stdin.close()
        except Exception:
            pass
        try:
            self._proc.terminate()
        except Exception:
            pass
        self._proc = None
        self._stdin = None
        self._stdout = None


def get_session(book: str = "crypto", evidence_dir: Optional[str] = None) -> Optional[PlanLifecycleSession]:
    if not lifecycle_enabled():
        return None
    bin_path = resolve_lifecycle_bin()
    if not bin_path:
        return None
    if evidence_dir:
        ev = evidence_dir
    elif book == "crypto":
        ev = os.environ.get("RP608_CRYPTO_REC001H_DIR", DEFAULT_CRYPTO_EVIDENCE)
    else:
        ev = os.environ.get("RP608_NSE_REC001H_DIR", DEFAULT_NSE_EVIDENCE)
    key = (bin_path, ev, book)
    with _LOCK:
        sess = _SESSIONS.get(key)
        if sess is None:
            sess = PlanLifecycleSession(bin_path, ev, book)
            _SESSIONS[key] = sess
        return sess


def plan_from_overlay(plan: Dict[str, Any], *, book: str = "crypto") -> Dict[str, Any]:
    peak = plan.get("gate2x_peak_mfe_usdt")
    if peak is None:
        peak = plan.get("gate2x_peak_mfe_inr")
    notional = plan.get("simulated_notional_usdt")
    if notional is None:
        notional = plan.get("notional_inr") or plan.get("simulated_notional") or 0
    qty = plan.get("simulated_quantity")
    if qty is None:
        qty = plan.get("quantity") or plan.get("qty") or 0
    tp = plan.get("profit_target_price")
    if tp is None:
        tp = plan.get("target_price") or plan.get("tp") or 0
    mfe = plan.get("mfe_exec_price")
    if mfe is None:
        mfe = plan.get("mfe_price") or plan.get("mfe_px")
    mae = plan.get("mae_exec_price")
    if mae is None:
        mae = plan.get("mae_price") or plan.get("mae_px")
    # NSE Estimand A is LONG; crypto overlay default SHORT.
    default_dir = "LONG" if book == "nse" else "SHORT"
    entry_ms = int(
        plan.get("entry_time_ms")
        or plan.get("entry_timestamp_utc_ms")
        or plan.get("opened_ms")
        or 0
    )
    expiry = plan.get("expiry_time_ms") or plan.get("expires_ms")
    if expiry is None and book == "nse":
        # Bar horizon managed in Python; give Rust a far expiry so tick path
        # does not TIME_EXIT — bar path still owns MAX_HOLD_HORIZON_180M.
        expiry = entry_ms + 7 * 24 * 3600 * 1000 if entry_ms else 2**62
    if notional in (None, 0, 0.0) and book == "nse":
        try:
            notional = float(qty or 0) * float(
                plan.get("entry_price") or plan.get("entry_px") or 0
            )
        except (TypeError, ValueError):
            notional = 0
    return {
        "plan_id": str(plan.get("plan_id") or plan.get("trade_id") or ""),
        "instrument": str(
            plan.get("symbol") or plan.get("instrument") or plan.get("tradingsymbol") or ""
        ),
        "direction": str(
            plan.get("direction") or plan.get("side") or default_dir
        ).upper(),
        "entry_price": float(plan.get("entry_price") or plan.get("entry_px") or 0),
        "stop_loss_price": float(plan.get("stop_loss_price") or plan.get("sl") or 0),
        "original_stop_loss_price": (
            float(plan["original_stop_loss_price"])
            if plan.get("original_stop_loss_price") is not None
            else float(plan.get("stop_loss_price") or plan.get("sl") or 0)
        ),
        "profit_target_price": float(tp or 0),
        "atr_14": float(plan.get("atr_14") or 0),
        "simulated_quantity": float(qty or 0),
        "simulated_notional_usdt": float(notional or 0),
        "entry_time_ms": entry_ms,
        "expiry_time_ms": int(expiry or 0),
        "tp_level": int(plan.get("tp_level") or 0),
        "adaptive_tp_armed": bool(plan.get("adaptive_tp_armed")),
        "adaptive_partial_latch_armed": bool(plan.get("adaptive_partial_latch_armed")),
        "trailing_sl_armed": bool(plan.get("trailing_sl_armed")),
        "mfe_exec_price": mfe,
        "mae_exec_price": mae,
        "gate2x_peak_mfe": peak,
        "gate2x_peak_tp_frac": plan.get("gate2x_peak_tp_frac"),
        "gate2x_mfe_armed": bool(plan.get("gate2x_mfe_armed")),
        "gate2x_soft_breach_since_ms": plan.get("gate2x_soft_breach_since_ms"),
        "path_h_adverse_seen": bool(plan.get("path_h_adverse_seen")),
        "path_h_revisit_seen": bool(plan.get("path_h_revisit_seen")),
        "exit_vol_surge": (
            plan.get("exit_vol_surge")
            or plan.get("gate2x_exit_vol_surge")
            or plan.get("vol_surge")
        ),
        "book": book,
        # R4: named preset preferred; legacy field overrides still accepted.
        "gate2x_preset": (
            str(plan["gate2x_preset"]).strip() if plan.get("gate2x_preset") else None
        )
        or None,
        "gate2x_floor_lock_immediate": bool(plan.get("gate2x_floor_lock_immediate")),
        "gate2x_ratchet_peak_frac": (
            float(plan["gate2x_ratchet_peak_frac"])
            if plan.get("gate2x_ratchet_peak_frac") is not None
            else None
        ),
        "gate2x_protocol": (
            str(plan.get("gate2x_protocol") or plan.get("gate2x_mfe_giveback_protocol") or "")
            or None
        ),
    }


def merge_lifecycle_into_plan(
    plan: Dict[str, Any], result: Dict[str, Any], *, book: str = "crypto"
) -> Dict[str, Any]:
    """Apply Rust lifecycle plan fields back onto overlay / NSE trade dict."""
    rp = result.get("plan") or {}
    out = dict(plan)
    peak_dst = "gate2x_peak_mfe_inr" if book == "nse" else "gate2x_peak_mfe_usdt"
    for k_src, k_dst in (
        ("stop_loss_price", "stop_loss_price"),
        ("original_stop_loss_price", "original_stop_loss_price"),
        ("profit_target_price", "profit_target_price"),
        ("tp_level", "tp_level"),
        ("adaptive_tp_armed", "adaptive_tp_armed"),
        ("adaptive_partial_latch_armed", "adaptive_partial_latch_armed"),
        ("trailing_sl_armed", "trailing_sl_armed"),
        ("mfe_exec_price", "mfe_exec_price"),
        ("mae_exec_price", "mae_exec_price"),
        ("gate2x_peak_mfe", peak_dst),
        ("gate2x_peak_tp_frac", "gate2x_peak_tp_frac"),
        ("gate2x_mfe_armed", "gate2x_mfe_armed"),
        ("gate2x_soft_breach_since_ms", "gate2x_soft_breach_since_ms"),
        ("path_h_adverse_seen", "path_h_adverse_seen"),
        ("path_h_revisit_seen", "path_h_revisit_seen"),
        ("exit_vol_surge", "exit_vol_surge"),
    ):
        if k_src in rp and rp[k_src] is not None:
            out[k_dst] = rp[k_src]
    if book == "nse":
        if "profit_target_price" in rp and rp["profit_target_price"] is not None:
            out["target_price"] = rp["profit_target_price"]
        if "mfe_exec_price" in rp and rp["mfe_exec_price"] is not None:
            out["mfe_price"] = rp["mfe_exec_price"]
        if "mae_exec_price" in rp and rp["mae_exec_price"] is not None:
            out["mae_price"] = rp["mae_exec_price"]
        if "exit_vol_surge" in rp and rp["exit_vol_surge"] is not None:
            out["gate2x_exit_vol_surge"] = rp["exit_vol_surge"]
    if result.get("gate2x_action"):
        out["gate2x_adapter_action"] = result["gate2x_action"]
    if result.get("ambiguous_resolved_as"):
        out["ambiguous_resolved_as"] = result["ambiguous_resolved_as"]
    out["plan_lifecycle_protocol"] = result.get("protocol")
    out["plan_lifecycle_authority"] = "rust_rp608_plan_lifecycle"
    return out


_EXIT_REASON_MAP_CRYPTO = {
    "PROFIT_TARGET_HIT": ("CLOSED_TP", "PROFIT_TARGET_HIT"),
    "STOP_LOSS_HIT": ("CLOSED_SL", "STOP_LOSS_HIT"),
    # Industry/OMS style: trailed stop fill (IBKR/TT/etc. "Trailing Stop")
    "TRAILING_STOP_HIT": ("CLOSED_TRAIL", "TRAILING_STOP_HIT"),
    "EARLY_SL_PATH_SCRATCH": ("CLOSED_SCRATCH", "EARLY_SL_PATH_SCRATCH"),
    "GATE2X_MFE_GIVEBACK_PROTECT": ("CLOSED_GIVEBACK", "GATE2X_MFE_GIVEBACK_PROTECT"),
    "MAX_HORIZON_EXPIRED": ("TIME_EXIT", "MAX_HORIZON_60M_EXPIRED"),
    "AMBIGUOUS_STOP_TARGET": ("CLOSED_SL", "AMBIGUOUS_STOP_TARGET"),
}

# NSE daemon uses STOP_LOSS / TAKE_PROFIT / GATE2X_MFE_GIVEBACK_PROTECT strings.
_EXIT_REASON_MAP_NSE = {
    "PROFIT_TARGET_HIT": ("TAKE_PROFIT", "TAKE_PROFIT"),
    "STOP_LOSS_HIT": ("STOP_LOSS", "STOP_LOSS"),
    "TRAILING_STOP_HIT": ("TRAILING_STOP", "TRAILING_STOP_HIT"),
    "EARLY_SL_PATH_SCRATCH": ("STOP_LOSS", "EARLY_SL_PATH_SCRATCH"),
    "GATE2X_MFE_GIVEBACK_PROTECT": (
        "GATE2X_MFE_GIVEBACK_PROTECT",
        "GATE2X_MFE_GIVEBACK_PROTECT",
    ),
    "MAX_HORIZON_EXPIRED": ("MAX_HOLD_HORIZON_180M", "MAX_HOLD_HORIZON_180M"),
    "AMBIGUOUS_STOP_TARGET": ("STOP_LOSS", "STOP_LOSS"),
}


def map_lifecycle_close(
    result: Dict[str, Any], *, book: str = "crypto"
) -> Tuple[str, str]:
    """Map Rust SCREAMING close reason → (terminal_state, exit_reason)."""
    raw = str(result.get("exit_reason") or "").upper()
    table = _EXIT_REASON_MAP_NSE if book == "nse" else _EXIT_REASON_MAP_CRYPTO
    return table.get(raw, ("CLOSED_SL", raw or "RUST_LIFECYCLE_CLOSE"))
