#!/usr/bin/env python3
"""
RP-608-SCI-ADAPTER-REC-001 — P2 apply client (Python I/O only).

Rust owns decide via `rp608_position_decide --sci-adapter-rec-loop` with the
frozen P1 BTC pack. Fail-closed STANDASIDE/INSUFFICIENT if bin/pack missing.
Coralys untouched. Gate 2E B + adapter_select baselines untouched.
Env arm: RP608_SCI_ADAPTER_REC_TWIN (default off).
"""

from __future__ import annotations

import json
import os
import subprocess
import threading
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence

PROTOCOL_ID = "RP-608-SCI-ADAPTER-REC-001"
SCHEMA_VERSION = "1"
FEATURE_SCHEMA_ID = "price_volume_15m_v1"

_REPO_ROOT = Path(__file__).resolve().parents[1]
_DEFAULT_PACK = (
    _REPO_ROOT / "docs/research/rp608_sci_adapter_rec_p1/btc_p1_side_models.json"
)
_SESSION_LOCK = threading.Lock()
_SESSION: Optional["SciAdapterRecSession"] = None


def twin_enabled(default: bool = False) -> bool:
    raw = os.environ.get("RP608_SCI_ADAPTER_REC_TWIN", "1" if default else "0")
    return str(raw).strip().lower() not in {"0", "false", "no", "off"}


def clean_twin_enabled(default: bool = False) -> bool:
    """P1a SCI-clean twin (no DI_SKEW/regime block). Anuj-gated; default off."""
    raw = os.environ.get("RP608_SCI_ADAPTER_REC_CLEAN_TWIN", "1" if default else "0")
    return str(raw).strip().lower() not in {"0", "false", "no", "off"}


def frozen_pack_path() -> Path:
    env = os.environ.get("RP608_SCI_ADAPTER_REC_PACK", "").strip()
    if env:
        return Path(env)
    return _DEFAULT_PACK


def _fail_closed(reason: str) -> Dict[str, Any]:
    return {
        "protocol_id": PROTOCOL_ID,
        "schema_version": SCHEMA_VERSION,
        "feature_schema_id": FEATURE_SCHEMA_ID,
        "schema_hash": "",
        "ladder_stage": "",
        "side": "FLAT",
        "p_hat": 0.0,
        "p_lb_wilson": 0.0,
        "p_long": 0.0,
        "p_short": 0.0,
        "p_lb_long": 0.0,
        "p_lb_short": 0.0,
        "band": "INSUFFICIENT" if reason.startswith("RUST_") else "STANDASIDE",
        "blocks_entry": True,
        "block_reason": reason,
        "p_min": 0.0,
        "p_lb_cut": 0.0,
        "authority": "rust_sci_adapter_rec",
        "rust_ok": False,
    }


def resolve_decide_bin() -> Optional[str]:
    candidates: List[str] = []
    for key in (
        "RP608_SCI_ADAPTER_REC_BIN",
        "RP608_POSITION_DECIDE_BIN",
        "RP608_ADAPTER_SELECT_15M_BIN",
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
        if "--sci-adapter-rec-loop" in blob:
            return path
    return None


class SciAdapterRecSession:
    def __init__(self, bin_path: str, pack_path: Path) -> None:
        self.bin_path = bin_path
        self.pack_path = pack_path
        self._proc: Optional[subprocess.Popen] = None
        self._lock = threading.Lock()

    def start(self) -> None:
        if self._proc is not None:
            return
        self._proc = subprocess.Popen(
            [self.bin_path, "--sci-adapter-rec-loop", "--sci-pack", str(self.pack_path)],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
            cwd=str(_REPO_ROOT),
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

    def evaluate(self, closes: Sequence[float], volumes: Sequence[float]) -> Dict[str, Any]:
        with self._lock:
            if self._proc is None or self._proc.poll() is not None:
                self.close()
                self.start()
            assert self._proc is not None and self._proc.stdin and self._proc.stdout
            req = {
                "closes": [float(x) for x in closes],
                "volumes": [float(x) for x in volumes],
            }
            try:
                self._proc.stdin.write(json.dumps(req, separators=(",", ":")) + "\n")
                self._proc.stdin.flush()
                line = self._proc.stdout.readline()
            except Exception:
                self.close()
                return _fail_closed("RUST_SCI_ADAPTER_REC_IO_ERROR")
            if not line:
                self.close()
                return _fail_closed("RUST_SCI_ADAPTER_REC_EOF")
            try:
                stamp = json.loads(line)
            except json.JSONDecodeError:
                self.close()
                return _fail_closed("RUST_SCI_ADAPTER_REC_BAD_JSON")
            if not isinstance(stamp, dict):
                return _fail_closed("RUST_SCI_ADAPTER_REC_BAD_SHAPE")
            stamp["authority"] = "rust_sci_adapter_rec"
            stamp["rust_ok"] = True
            band = stamp.get("band")
            if isinstance(band, str):
                stamp["band"] = band.upper()
            return stamp


def _session() -> Optional[SciAdapterRecSession]:
    global _SESSION
    with _SESSION_LOCK:
        if _SESSION is not None:
            return _SESSION
        bin_path = resolve_decide_bin()
        pack = frozen_pack_path()
        if not bin_path:
            return None
        if not pack.exists():
            return None
        _SESSION = SciAdapterRecSession(bin_path, pack)
        try:
            _SESSION.start()
        except Exception:
            _SESSION = None
            return None
        return _SESSION


def recommend(closes: Sequence[float], volumes: Sequence[float]) -> Dict[str, Any]:
    """Rust-owned generator stamp; fail-closed if bin/pack unavailable."""
    sess = _session()
    if sess is None:
        if not frozen_pack_path().exists():
            return _fail_closed("RUST_SCI_ADAPTER_REC_PACK_MISSING")
        return _fail_closed("RUST_SCI_ADAPTER_REC_BIN_MISSING")
    return sess.evaluate(closes, volumes)
