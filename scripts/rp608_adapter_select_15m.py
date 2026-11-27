#!/usr/bin/env python3
"""
RP-608-ADAPTER-SELECT-15M-V1 — Python I/O only.

Logic lives in Rust (`adapter_select_15m` via `rp608_position_decide
--adapter-select-15m-loop`). This module is a thin client + env helpers.
Fail-closed STANDASIDE/INSUFFICIENT if the Rust binary is missing/unusable.
Coralys untouched. traj15 / OF ±0.30 / stops unchanged.

Frozen D1–D5 (2026-11-27):
  D1=a filter on Coralys candidate side
  D2 enter |S_side| ≥ 0.35
  D3 BTC+ETH observational first (NSE not armed)
  D4 leave Gate 2E B soak running; exclude from promote path
  D5 design approved as coding charter
"""

from __future__ import annotations

import json
import os
import subprocess
import threading
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence

PROTOCOL_ID = "RP-608-ADAPTER-SELECT-15M-V1"
SCHEMA_VERSION = "1"
ENTER_ABS = 0.35
LOOKBACKS = (3, 8, 20)

_REPO_ROOT = Path(__file__).resolve().parents[1]
_SESSION_LOCK = threading.Lock()
_SESSION: Optional["AdapterSelect15mSession"] = None


def twin_enabled(default: bool = False) -> bool:
    """Observational twin arm. Default off until Anuj explicitly arms Pi."""
    raw = os.environ.get("RP608_ADAPTER_SELECT_15M_TWIN", "1" if default else "0")
    return str(raw).strip().lower() not in {"0", "false", "no", "off"}


def _fail_closed(candidate_side: str, reason: str) -> Dict[str, Any]:
    side = str(candidate_side or "").strip().upper()
    if side not in {"LONG", "SHORT", "FLAT"}:
        side = "FLAT"
    return {
        "protocol_id": PROTOCOL_ID,
        "schema_version": SCHEMA_VERSION,
        "candidate_side": side,
        "score": 0.0,
        "side_score": 0.0,
        "enter_abs": ENTER_ABS,
        "band": "INSUFFICIENT" if reason.startswith("RUST_") else "STANDASIDE",
        "blocks_entry": True,
        "block_reason": reason,
        "path_consistency": None,
        "features": [],
        "n_closes": 0,
        "authority": "rust_adapter_select_15m",
        "rust_ok": False,
    }


def resolve_decide_bin() -> Optional[str]:
    candidates: List[str] = []
    for key in (
        "RP608_ADAPTER_SELECT_15M_BIN",
        "RP608_POSITION_DECIDE_BIN",
        "RP608_GATE2X_MANAGE_BIN",
    ):
        env = os.environ.get(key, "").strip()
        if env:
            candidates.append(env)
    for rel in (
        "artifacts/deploy/chronosentiment/rp608_position_decide.darwin-arm64",
        "artifacts/deploy/chronosentiment/rp608_position_decide.aarch64-linux",
        "artifacts/deploy/chronosentiment/rp608_position_decide",
        "adapters/chronosentiment/target/release/rp608_position_decide",
        "adapters/chronosentiment/target/debug/rp608_position_decide",
        "target/release/rp608_position_decide",
        "target/debug/rp608_position_decide",
    ):
        candidates.append(str(_REPO_ROOT / rel))
    candidates.append(str(Path.home() / "bin" / "rp608_position_decide"))
    which = subprocess.run(["which", "rp608_position_decide"], capture_output=True, text=True)
    if which.returncode == 0 and which.stdout.strip():
        candidates.append(which.stdout.strip())

    seen = set()
    for path in candidates:
        if not path or path in seen:
            continue
        seen.add(path)
        p = Path(path)
        if not p.exists():
            continue
        try:
            proc = subprocess.run(
                [path, "--help"],
                capture_output=True,
                text=True,
                timeout=3.0,
                check=False,
            )
        except (OSError, subprocess.TimeoutExpired):
            continue
        blob = (proc.stderr or "") + (proc.stdout or "")
        if "--adapter-select-15m-loop" in blob:
            return path
    return None


class AdapterSelect15mSession:
    """Persistent JSONL session to Rust --adapter-select-15m-loop."""

    def __init__(self, bin_path: str) -> None:
        self.bin_path = bin_path
        self._proc: Optional[subprocess.Popen] = None
        self._lock = threading.Lock()

    def start(self) -> None:
        if self._proc is not None:
            return
        self._proc = subprocess.Popen(
            [self.bin_path, "--adapter-select-15m-loop"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
        )

    def close(self) -> None:
        if self._proc is None:
            return
        try:
            if self._proc.stdin:
                self._proc.stdin.close()
        except Exception:
            pass
        try:
            self._proc.terminate()
        except Exception:
            pass
        self._proc = None

    def evaluate(self, closes: Sequence[float], candidate_side: str) -> Dict[str, Any]:
        with self._lock:
            if self._proc is None or self._proc.poll() is not None:
                self.close()
                self.start()
            assert self._proc is not None and self._proc.stdin and self._proc.stdout
            req = {
                "closes": [float(x) for x in closes],
                "candidate_side": str(candidate_side or ""),
            }
            try:
                self._proc.stdin.write(json.dumps(req, separators=(",", ":")) + "\n")
                self._proc.stdin.flush()
                line = self._proc.stdout.readline()
            except Exception:
                self.close()
                return _fail_closed(candidate_side, "RUST_ADAPTER_SELECT_15M_IO_ERROR")
            if not line:
                self.close()
                return _fail_closed(candidate_side, "RUST_ADAPTER_SELECT_15M_EOF")
            try:
                stamp = json.loads(line)
            except json.JSONDecodeError:
                self.close()
                return _fail_closed(candidate_side, "RUST_ADAPTER_SELECT_15M_BAD_JSON")
            if not isinstance(stamp, dict):
                return _fail_closed(candidate_side, "RUST_ADAPTER_SELECT_15M_BAD_SHAPE")
            stamp["authority"] = "rust_adapter_select_15m"
            stamp["rust_ok"] = True
            # Normalize band enum if Rust emitted SCREAMING_SNAKE via serde.
            band = stamp.get("band")
            if isinstance(band, str):
                stamp["band"] = band.upper()
            return stamp


def _session() -> Optional[AdapterSelect15mSession]:
    global _SESSION
    with _SESSION_LOCK:
        if _SESSION is not None:
            return _SESSION
        bin_path = resolve_decide_bin()
        if not bin_path:
            return None
        _SESSION = AdapterSelect15mSession(bin_path)
        try:
            _SESSION.start()
        except Exception:
            _SESSION = None
            return None
        return _SESSION


def recommend(closes: Sequence[float], candidate_side: str) -> Dict[str, Any]:
    """Rust-owned recommendation; fail-closed if bin unavailable."""
    sess = _session()
    if sess is None:
        return _fail_closed(candidate_side, "RUST_ADAPTER_SELECT_15M_BIN_MISSING")
    return sess.evaluate(closes, candidate_side)


# Alias used by overlay / status scripts
select_15m = recommend
