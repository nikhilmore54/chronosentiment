"""Gate 2X executor client — Rust adapter owns decisions (in-process manage).

Authority: `Gate2xPositionManager` / `gate2x_decide` → frozen RecommendationEngineV1.
Python never invents Protect/Exit. Prefer persistent `--manage-loop` (one REC001h
load); one-shot CLI is fallback only. Fail-closed Hold if Rust binary missing.
Adaptive TP (Gate 5) may extend post-touch; Gate 2X still owns continue-vs-exit.
See .cursor/rules/chronosentiment-core.mdc §3c.
"""

from __future__ import annotations

import json
import os
import subprocess
import threading
from pathlib import Path
from typing import Any, Dict, Optional, TextIO, Tuple

_REPO_ROOT = Path(__file__).resolve().parents[1]

DEFAULT_CRYPTO_EVIDENCE = str(
    _REPO_ROOT / "datasets/recommendation/historical_crypto"
)
DEFAULT_NSE_EVIDENCE = str(_REPO_ROOT / "datasets/recommendation/historical")

# Process-wide manage sessions keyed by (bin, evidence_dir, book).
_MANAGE_SESSIONS: Dict[Tuple[str, str, str], "Gate2xManageSession"] = {}
_MANAGE_LOCK = threading.Lock()


def _bin_supports_manage_loop(path: str) -> bool:
    """Reject stale Pi binaries that predate --manage-loop / Gate2xPositionManager."""
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
    return "--manage-loop" in blob


def resolve_decide_bin() -> Optional[str]:
    candidates: list[str] = []
    for key in ("RP608_GATE2X_MANAGE_BIN", "RP608_POSITION_DECIDE_BIN"):
        env = os.environ.get(key, "").strip()
        if env:
            candidates.append(env)
    # Prefer explicit aarch64 stage / home bin before any stale unsuffixed copy.
    for rel in (
        "adapters/chronosentiment/bin/rp608_position_decide.aarch64-linux",
        "adapters/chronosentiment/bin/rp608_position_decide",
        "adapters/chronosentiment/target/aarch64-unknown-linux-gnu/release/rp608_position_decide",
        "adapters/chronosentiment/target/release/rp608_position_decide",
        "adapters/chronosentiment/target/debug/rp608_position_decide",
        "target/release/rp608_position_decide",
        "target/debug/rp608_position_decide",
    ):
        candidates.append(str(_REPO_ROOT / rel))
    home_bin = Path.home() / "bin" / "rp608_position_decide"
    candidates.append(str(home_bin))
    which = subprocess.run(
        ["which", "rp608_position_decide"], capture_output=True, text=True
    )
    if which.returncode == 0 and which.stdout.strip():
        candidates.append(which.stdout.strip())

    seen = set()
    for path in candidates:
        if not path or path in seen:
            continue
        seen.add(path)
        if not Path(path).exists():
            continue
        if _bin_supports_manage_loop(path):
            return path
    return None


def build_snapshot(
    *,
    decision_id: str,
    instrument: str,
    direction: str,
    reference_price: Optional[float],
    peak_mfe: float,
    current_mfe: float,
    peak_tp_frac: float,
    hold_s: float,
    arm_threshold: float,
    already_armed: bool,
    vol_surge: str,
    soft_breach_since_ms: Optional[int],
    now_ms: int,
    protocol: str,
    params: Optional[Dict[str, Any]] = None,
    tp_level: int = 0,
    adaptive_armed: bool = False,
) -> Dict[str, Any]:
    p = params or {}
    return {
        "decision_id": decision_id,
        "instrument": instrument,
        "direction": direction,
        "reference_price": reference_price,
        "peak_mfe": float(peak_mfe),
        "current_mfe": float(current_mfe),
        "peak_tp_frac": float(peak_tp_frac),
        "hold_s": float(hold_s),
        "arm_threshold": float(arm_threshold),
        "already_armed": bool(already_armed),
        "vol_surge": str(vol_surge or "UNK").upper(),
        "soft_breach_since_ms": soft_breach_since_ms,
        "now_ms": int(now_ms),
        "train_vol_median": float(p.get("train_vol_median") or 0.001),
        "tp_level": int(tp_level or 0),
        "adaptive_armed": bool(adaptive_armed),
        "params": {
            "enabled": bool(p.get("enabled", True)),
            "arm_min_tp_frac": float(p.get("arm_min_tp_frac", 0.40)),
            "min_hold_s_before_exit": float(p.get("min_hold_s_before_exit", 120.0)),
            "meaningful_floor_bps": float(p.get("meaningful_floor_bps", 5.0)),
            "protect_floor": str(p.get("protect_floor", "COST_CLEAR")).upper(),
            "reversal_volume_confirm": bool(p.get("reversal_volume_confirm", True)),
            "immediate_vol_surges": list(p.get("immediate_vol_surges") or ["HIGH"]),
            "med_vol_dwell_ms": int(p.get("med_vol_dwell_ms", 40_000)),
            "low_vol_dwell_ms": int(p.get("low_vol_dwell_ms", 90_000)),
            "ratchet_peak_frac": float(p.get("ratchet_peak_frac", 0.30)),
            "deep_giveback_frac": float(p.get("deep_giveback_frac", 0.50)),
            "deep_giveback_dwell_scale": float(p.get("deep_giveback_dwell_scale", 0.40)),
            "protocol": protocol,
        },
    }


def _fail_closed_hold(reason: str) -> Tuple[str, Optional[str], Dict[str, Any]]:
    """No Python Protect/Exit invent — executor holds until Rust authority returns."""
    return "Hold", reason, {"armed": False, "soft_breach_since_ms": None}


def _book_for_evidence(evidence_dir: str) -> str:
    low = evidence_dir.lower()
    if "historical_crypto" in low or low.rstrip("/").endswith("crypto"):
        return "crypto"
    return "nse"


class Gate2xManageSession:
    """Persistent Rust `--manage-loop` worker (in-process gate2x_decide)."""

    def __init__(self, bin_path: str, evidence_dir: str, book: str) -> None:
        self.bin_path = bin_path
        self.evidence_dir = evidence_dir
        self.book = book
        self._proc: Optional[subprocess.Popen] = None
        self._stdin: Optional[TextIO] = None
        self._stdout: Optional[TextIO] = None
        self._io_lock = threading.Lock()
        self._start()

    def _start(self) -> None:
        self._proc = subprocess.Popen(
            [
                self.bin_path,
                "--manage-loop",
                "--evidence-dir",
                self.evidence_dir,
                "--book",
                self.book,
            ],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
        )
        assert self._proc.stdin is not None and self._proc.stdout is not None
        self._stdin = self._proc.stdin
        self._stdout = self._proc.stdout

    def close(self) -> None:
        with self._io_lock:
            if self._proc and self._proc.poll() is None:
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

    def decide(self, snapshot: Dict[str, Any], *, timeout_s: float = 5.0) -> Tuple[str, Optional[str], Dict[str, Any]]:
        with self._io_lock:
            if self._proc is None or self._proc.poll() is not None:
                try:
                    self._start()
                except OSError as exc:
                    return _fail_closed_hold(f"GATE2X_MANAGE_START:{exc}")
            assert self._stdin is not None and self._stdout is not None
            try:
                self._stdin.write(json.dumps(snapshot) + "\n")
                self._stdin.flush()
            except BrokenPipeError as exc:
                return _fail_closed_hold(f"GATE2X_MANAGE_PIPE:{exc}")
            # Blocking readline; manage loop is local and deterministic.
            line = self._stdout.readline()
            if not line:
                return _fail_closed_hold("GATE2X_MANAGE_EOF")
            try:
                decision = json.loads(line)
            except json.JSONDecodeError:
                return _fail_closed_hold("GATE2X_MANAGE_BAD_JSON")
            action = str(decision.get("action") or "Hold")
            reason = str(decision.get("reason") or "") or None
            soft = decision.get("soft_breach") or {}
            if action not in ("Hold", "Protect", "Exit"):
                return _fail_closed_hold("GATE2X_MANAGE_BAD_ACTION")
            return action, reason, soft


def get_manage_session(evidence_dir: str, *, book: Optional[str] = None) -> Optional[Gate2xManageSession]:
    bin_path = resolve_decide_bin()
    if not bin_path:
        return None
    b = (book or _book_for_evidence(evidence_dir)).lower()
    key = (bin_path, evidence_dir, b)
    with _MANAGE_LOCK:
        sess = _MANAGE_SESSIONS.get(key)
        if sess is None:
            try:
                sess = Gate2xManageSession(bin_path, evidence_dir, b)
            except OSError:
                return None
            _MANAGE_SESSIONS[key] = sess
        return sess


def _oneshot_decide(
    snapshot: Dict[str, Any],
    *,
    evidence_dir: str,
    timeout_s: float,
) -> Tuple[str, Optional[str], Dict[str, Any]]:
    bin_path = resolve_decide_bin()
    if not bin_path:
        return _fail_closed_hold("GATE2X_NO_RUST_BIN")
    book = _book_for_evidence(evidence_dir)
    try:
        proc = subprocess.run(
            [bin_path, "--evidence-dir", evidence_dir, "--book", book],
            input=json.dumps(snapshot),
            capture_output=True,
            text=True,
            timeout=timeout_s,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        return _fail_closed_hold(f"GATE2X_ADAPTER_TRANSPORT:{exc}")
    if proc.returncode != 0:
        err = (proc.stderr or "").strip()[:200]
        return _fail_closed_hold(f"GATE2X_ADAPTER_ERROR:{err}")
    lines = (proc.stdout or "").strip().splitlines()
    if not lines:
        return _fail_closed_hold("GATE2X_ADAPTER_EMPTY")
    try:
        decision = json.loads(lines[-1])
    except json.JSONDecodeError:
        return _fail_closed_hold("GATE2X_ADAPTER_BAD_JSON")
    action = str(decision.get("action") or "Hold")
    reason = str(decision.get("reason") or "") or None
    soft = decision.get("soft_breach") or {}
    if action not in ("Hold", "Protect", "Exit"):
        return _fail_closed_hold("GATE2X_ADAPTER_BAD_ACTION")
    return action, reason, soft


def decide(
    snapshot: Dict[str, Any],
    *,
    evidence_dir: str,
    timeout_s: float = 5.0,
    prefer_manage_loop: bool = True,
) -> Tuple[str, Optional[str], Dict[str, Any]]:
    """
    Returns (action, detail_reason_or_None, soft_breach_state).

    action: Hold | Protect | Exit — from Rust only.
    """
    use_loop = prefer_manage_loop and os.environ.get("RP608_GATE2X_ONESHOT", "").strip() != "1"
    if use_loop:
        sess = get_manage_session(evidence_dir)
        if sess is not None:
            return sess.decide(snapshot, timeout_s=timeout_s)
    return _oneshot_decide(snapshot, evidence_dir=evidence_dir, timeout_s=timeout_s)


def apply_soft_breach_to_trade(trade: Dict[str, Any], soft: Dict[str, Any]) -> None:
    """Persist adapter soft-breach / arm fields onto the open trade/plan dict."""
    if not soft:
        return
    if "soft_breach_since_ms" in soft:
        trade["gate2x_soft_breach_since_ms"] = soft.get("soft_breach_since_ms")
    if soft.get("soft_breach_dwell_ms") is not None:
        trade["gate2x_soft_breach_dwell_ms"] = soft.get("soft_breach_dwell_ms")
    if soft.get("soft_breach_vol_surge") is not None:
        trade["gate2x_soft_breach_vol_surge"] = soft.get("soft_breach_vol_surge")
    if soft.get("giveback_frac") is not None:
        trade["gate2x_giveback_frac"] = soft.get("giveback_frac")
    if soft.get("protect_floor") is not None:
        trade["gate2x_protect_floor"] = soft.get("protect_floor")
    if soft.get("ratchet_floor") is not None:
        trade["gate2x_ratchet_floor"] = soft.get("ratchet_floor")
    if soft.get("armed"):
        trade["gate2x_mfe_armed"] = True
    if soft.get("protocol"):
        trade["gate2x_protocol"] = soft.get("protocol")
