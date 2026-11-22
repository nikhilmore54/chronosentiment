#!/usr/bin/env python3
"""
Thin client for Rust crypto tick policy (Track C).

Env:
  RP608_CRYPTO_TICK_POLICY=python|rust  (default python until soak)
  RP608_CRYPTO_TICK_POLICY_BIN / RP608_POSITION_DECIDE_BIN — optional future loop bin

Today: library parity lives in Rust `product::crypto_tick_policy`; daemon stays on
Python evaluate path until `--tick-eval-loop` is wired. This module exposes the
env switch + helpers so daemon can flip without a second cutover.
"""

from __future__ import annotations

import os
from typing import Optional


def tick_policy_backend() -> str:
    raw = (os.environ.get("RP608_CRYPTO_TICK_POLICY") or "python").strip().lower()
    if raw in {"rust", "rs"}:
        return "rust"
    return "python"


def rust_tick_ready() -> bool:
    """True when rust backend requested AND a decide bin exists (future loop)."""
    if tick_policy_backend() != "rust":
        return False
    for key in (
        "RP608_CRYPTO_TICK_POLICY_BIN",
        "RP608_POSITION_DECIDE_BIN",
        "RP608_PLAN_LIFECYCLE_BIN",
    ):
        p = (os.environ.get(key) or "").strip()
        if p and os.path.isfile(p) and os.access(p, os.X_OK):
            return True
    return False


def resolve_tick_eval_mode() -> str:
    """
    Effective mode for evaluate_on_tick.
    Until tick-eval-loop is deployed, rust request falls back to python with log once.
    """
    if tick_policy_backend() == "rust" and rust_tick_ready():
        # Loop not yet in position_decide — keep python until C CLI lands.
        # Callers may check this string for telemetry.
        return "python_fallback_pending_tick_loop"
    return "python"
