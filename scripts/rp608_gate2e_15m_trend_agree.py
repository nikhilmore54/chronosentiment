#!/usr/bin/env python3
"""
RP-608-GATE2E-15M-OHLCV-TREND-AGREE-001 — Python I/O only.

Logic lives in Rust (`ohlcv_trend_agreement` via `rp608_position_decide
--trend-agree-loop`). This module is a thin client + env helpers.
Fail-closed STANDASIDE if the Rust binary is missing/unusable.
Coralys untouched. traj15 / OF ±0.30 / stops unchanged.
"""

from __future__ import annotations

import json
import os
import subprocess
import threading
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence, TextIO

PROTOCOL_ID = "RP-608-GATE2E-15M-OHLCV-TREND-AGREE-001"
LOOKBACKS = (3, 8, 20)  # documented contract; evaluated in Rust

_REPO_ROOT = Path(__file__).resolve().parents[1]
_SESSION_LOCK = threading.Lock()
_SESSION: Optional["TrendAgreeSession"] = None


def twin_enabled(default: bool = True) -> bool:
    raw = os.environ.get("RP608_GATE2E_TREND_AGREE_TWIN", "1" if default else "0")
    return str(raw).strip().lower() not in {"0", "false", "no", "off"}


def nse_twin_enabled(default: bool = True) -> bool:
    raw = os.environ.get("RP608_NSE_GATE2E_TREND_AGREE_TWIN", "1" if default else "0")
    return str(raw).strip().lower() not in {"0", "false", "no", "off"}


def _fail_closed(candidate_side: str, reason: str) -> Dict[str, Any]:
    side = str(candidate_side or "").strip().upper()
    if side not in {"LONG", "SHORT", "FLAT"}:
        side = "FLAT"
    return {
        "protocol_id": PROTOCOL_ID,
        "candidate_side": side,
        "votes": [],
        "agree_count": 0,
        "oppose_count": 0,
        "flat_or_missing": 3,
        "agreement": "STANDASIDE",
        "blocks_entry": True,
        "block_reason": reason,
        "authority": "rust_ohlcv_trend_agreement",
        "rust_ok": False,
    }


def resolve_decide_bin() -> Optional[str]:
    candidates: List[str] = []
    for key in ("RP608_TREND_AGREE_BIN", "RP608_POSITION_DECIDE_BIN", "RP608_GATE2X_MANAGE_BIN"):
        env = os.environ.get(key, "").strip()
        if env:
            candidates.append(env)
    for rel in (
        "artifacts/deploy/chronosentiment/rp608_position_decide.aarch64-linux",
        "artifacts/deploy/chronosentiment/rp608_position_decide",
        "adapters/chronosentiment/target/aarch64-unknown-linux-gnu/release/rp608_position_decide",
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
        if "--trend-agree-loop" in blob:
            return path
    return None


class TrendAgreeSession:
    """Persistent JSONL session to Rust --trend-agree-loop."""

    def __init__(self, bin_path: str) -> None:
        self.bin_path = bin_path
        self._proc: Optional[subprocess.Popen] = None
        self._lock = threading.Lock()

    def start(self) -> None:
        if self._proc is not None:
            return
        self._proc = subprocess.Popen(
            [self.bin_path, "--trend-agree-loop"],
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
                return _fail_closed(candidate_side, "RUST_TREND_AGREE_IO_ERROR")
            if not line:
                self.close()
                return _fail_closed(candidate_side, "RUST_TREND_AGREE_EOF")
            try:
                stamp = json.loads(line)
            except json.JSONDecodeError:
                self.close()
                return _fail_closed(candidate_side, "RUST_TREND_AGREE_BAD_JSON")
            if not isinstance(stamp, dict):
                return _fail_closed(candidate_side, "RUST_TREND_AGREE_BAD_SHAPE")
            stamp["authority"] = "rust_ohlcv_trend_agreement"
            stamp["rust_ok"] = True
            # Normalize enum-ish vote sides if Rust emitted objects
            votes = stamp.get("votes")
            if isinstance(votes, list):
                norm_votes = []
                for v in votes:
                    if not isinstance(v, dict):
                        continue
                    side = v.get("side")
                    if isinstance(side, dict):
                        # unexpected
                        side = "FLAT"
                    elif side is not None:
                        side = str(side).upper()
                    nv = dict(v)
                    nv["side"] = side
                    norm_votes.append(nv)
                stamp["votes"] = norm_votes
            return stamp


def _session() -> Optional[TrendAgreeSession]:
    global _SESSION
    with _SESSION_LOCK:
        if _SESSION is not None:
            return _SESSION
        bin_path = resolve_decide_bin()
        if not bin_path:
            return None
        _SESSION = TrendAgreeSession(bin_path)
        try:
            _SESSION.start()
        except Exception:
            _SESSION = None
            return None
        return _SESSION


def agreement(closes: Sequence[float], candidate_side: str) -> Dict[str, Any]:
    """Rust-owned agreement; fail-closed STANDASIDE if bin unavailable."""
    sess = _session()
    if sess is None:
        return _fail_closed(candidate_side, "RUST_TREND_AGREE_BIN_MISSING")
    return sess.evaluate(closes, candidate_side)


def label_lookbacks(closes: Sequence[float]) -> List[Dict[str, Any]]:
    """Convenience: run agreement and return votes (still Rust-owned)."""
    stamp = agreement(closes, "LONG")
    votes = stamp.get("votes")
    return list(votes) if isinstance(votes, list) else []
