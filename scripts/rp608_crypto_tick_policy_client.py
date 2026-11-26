#!/usr/bin/env python3
"""Thin client for Rust crypto tick policy (Track C).

Env:
  RP608_CRYPTO_TICK_POLICY=python|rust|compare  (default python until soak)
  RP608_CRYPTO_TICK_POLICY_BIN / RP608_POSITION_DECIDE_BIN / RP608_PLAN_LIFECYCLE_BIN

`rust` → `rp608_position_decide --tick-eval-loop` for features + B/C actions.
`compare` → Rust primary when bin ready; also logs divergence vs Python (soak).
Missing bin → Python fallback (fail-soft). Emission gate / ATR / overlay stay Python.

Aligned with `.cursor/rules/chronosentiment-core.mdc` · Track C ABC plan.
"""

from __future__ import annotations

import json
import os
import subprocess
import threading
from pathlib import Path
from typing import Any, Dict, List, Optional, TextIO, Tuple

_SESSIONS: Dict[str, "TickEvalSession"] = {}
_LOCK = threading.Lock()
_FALLBACK_LOGGED = False


def tick_policy_backend() -> str:
    raw = (os.environ.get("RP608_CRYPTO_TICK_POLICY") or "python").strip().lower()
    if raw in {"rust", "rs"}:
        return "rust"
    if raw in {"compare", "side_by_side", "sbs"}:
        return "compare"
    return "python"


def _bin_supports_tick_eval(path: str) -> bool:
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
    return "--tick-eval-loop" in blob


def resolve_tick_eval_bin() -> Optional[str]:
    for key in (
        "RP608_CRYPTO_TICK_POLICY_BIN",
        "RP608_POSITION_DECIDE_BIN",
        "RP608_PLAN_LIFECYCLE_BIN",
        "RP608_GATE2X_MANAGE_BIN",
    ):
        p = (os.environ.get(key) or "").strip()
        if p and Path(p).is_file() and os.access(p, os.X_OK) and _bin_supports_tick_eval(p):
            return p
    # Local deploy artifact fallback (dev / unit tests).
    repo = Path(__file__).resolve().parents[1]
    for rel in (
        "artifacts/deploy/chronosentiment/rp608_position_decide.darwin-arm64",
        "artifacts/deploy/chronosentiment/rp608_position_decide.aarch64-linux",
    ):
        cand = repo / rel
        if cand.is_file() and os.access(cand, os.X_OK) and _bin_supports_tick_eval(str(cand)):
            return str(cand)
    return None


def rust_tick_ready() -> bool:
    return tick_policy_backend() in {"rust", "compare"} and resolve_tick_eval_bin() is not None


def resolve_tick_eval_mode() -> str:
    """Effective mode for evaluate_on_tick telemetry."""
    backend = tick_policy_backend()
    if backend == "python":
        return "python"
    if not resolve_tick_eval_bin():
        return "python_fallback_missing_bin"
    if backend == "compare":
        return "compare_rust_primary"
    return "rust"


def quote_to_dict(quote: Any) -> Dict[str, Any]:
    if isinstance(quote, dict):
        return {
            "update_id": int(quote["update_id"]),
            "event_time_ms": int(quote["event_time_ms"]),
            "receive_time_ms": int(quote["receive_time_ms"]),
            "bid_price": float(quote["bid_price"]),
            "bid_qty": float(quote["bid_qty"]),
            "ask_price": float(quote["ask_price"]),
            "ask_qty": float(quote["ask_qty"]),
            "symbol": str(quote.get("symbol") or "BTCUSDT"),
        }
    return {
        "update_id": int(quote.update_id),
        "event_time_ms": int(quote.event_time_ms),
        "receive_time_ms": int(quote.receive_time_ms),
        "bid_price": float(quote.bid_price),
        "bid_qty": float(quote.bid_qty),
        "ask_price": float(quote.ask_price),
        "ask_qty": float(quote.ask_qty),
        "symbol": str(getattr(quote, "symbol", None) or "BTCUSDT"),
    }


class TickEvalSession:
    def __init__(self, bin_path: str) -> None:
        self.bin_path = bin_path
        self._proc: Optional[subprocess.Popen] = None
        self._stdin: Optional[TextIO] = None
        self._stdout: Optional[TextIO] = None
        self._lock = threading.Lock()

    def start(self) -> None:
        if self._proc is not None:
            return
        self._proc = subprocess.Popen(
            [self.bin_path, "--tick-eval-loop", "--book", "crypto"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
        )
        assert self._proc.stdin and self._proc.stdout
        self._stdin = self._proc.stdin
        self._stdout = self._proc.stdout

    def eval_tick(
        self,
        *,
        quote: Any,
        buffer_quotes: List[Any],
        closes: List[float],
        policy_b_genome: Dict[str, Any],
        policy_c_genome: Dict[str, Any],
    ) -> Optional[Dict[str, Any]]:
        with self._lock:
            try:
                self.start()
                assert self._stdin and self._stdout and self._proc
                payload = {
                    "quote": quote_to_dict(quote),
                    "buffer_quotes": [quote_to_dict(q) for q in buffer_quotes],
                    "closes": [float(c) for c in closes],
                    "policy_b_genome": policy_b_genome,
                    "policy_c_genome": policy_c_genome,
                }
                self._stdin.write(json.dumps(payload, default=str) + "\n")
                self._stdin.flush()
                line = self._stdout.readline()
                if not line:
                    self.close()
                    return None
                out = json.loads(line)
                if not isinstance(out, dict):
                    return None
                return out
            except Exception as exc:
                print(
                    f"[crypto_tick_policy] rust eval failed: {type(exc).__name__}: {exc}",
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
        try:
            self._proc.wait(timeout=2.0)
        except Exception:
            try:
                self._proc.kill()
            except Exception:
                pass
        self._proc = None
        self._stdin = None
        self._stdout = None


def get_tick_eval_session() -> Optional[TickEvalSession]:
    bin_path = resolve_tick_eval_bin()
    if not bin_path:
        return None
    with _LOCK:
        sess = _SESSIONS.get(bin_path)
        if sess is None:
            sess = TickEvalSession(bin_path)
            _SESSIONS[bin_path] = sess
        return sess


def rust_eval_tick(
    *,
    quote: Any,
    buffer_quotes: List[Any],
    closes: List[float],
    policy_b_genome: Dict[str, Any],
    policy_c_genome: Dict[str, Any],
) -> Optional[Tuple[str, str, Dict[str, Any], Dict[str, Any]]]:
    """
    Returns (action_b, action_c, features_tick, rec_hygiene) or None on failure.
    Hygiene is applied inside the persistent Rust --tick-eval-loop process.
    """
    sess = get_tick_eval_session()
    if sess is None:
        return None
    # Prior quotes only — loop appends `quote` after buffer_quotes.
    prior = list(buffer_quotes)
    if prior and quote_to_dict(prior[-1])["update_id"] == quote_to_dict(quote)["update_id"]:
        prior = prior[:-1]
    out = sess.eval_tick(
        quote=quote,
        buffer_quotes=prior,
        closes=closes,
        policy_b_genome=policy_b_genome,
        policy_c_genome=policy_c_genome,
    )
    if not out:
        return None
    action_b = str(out.get("policy_b_action") or "NO_TRADE")
    action_c = str(out.get("policy_c_action") or "NO_TRADE")
    feats = out.get("features_tick")
    if not isinstance(feats, dict):
        feats = {
            "mid_price": out.get("mid_price"),
            "imbalance": out.get("imbalance"),
            "update_id": out.get("update_id"),
        }
    hygiene = out.get("rec_hygiene") if isinstance(out.get("rec_hygiene"), dict) else {}
    return action_b, action_c, feats, hygiene


def log_python_fallback_once(reason: str) -> None:
    global _FALLBACK_LOGGED
    if _FALLBACK_LOGGED:
        return
    _FALLBACK_LOGGED = True
    print(
        f"[crypto_tick_policy] Track C falling back to Python ({reason})",
        flush=True,
    )
