#!/usr/bin/env python3
"""
scripts/rp608_singleton_lock.py

Exclusive process lock for RP-608 shadow daemons (fcntl flock).
Prevents duplicate crypto/NSE shadow instances from racing shared overlay state.
Authority 0% — ops hygiene only. Aligned with .cursor/rules chronosentiment-core.
"""

from __future__ import annotations

import atexit
import fcntl
import os
import sys
from pathlib import Path
from typing import Any, Optional, TextIO


class SingletonLockError(RuntimeError):
    """Raised when another live holder already owns the lock."""


def acquire_singleton_lock(
    lock_path: Path,
    *,
    label: str = "rp608",
    hold: Optional[list] = None,
) -> TextIO:
    """
    Acquire non-blocking exclusive flock on ``lock_path``.

    Keep the returned file handle open for the process lifetime (or store it
    in ``hold`` so it is not GC'd). Writes pid into the lockfile for operators.
    """
    lock_path = Path(lock_path)
    lock_path.parent.mkdir(parents=True, exist_ok=True)
    fh = open(lock_path, "a+", encoding="utf-8")
    try:
        fcntl.flock(fh.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
    except BlockingIOError as exc:
        holder = ""
        try:
            fh.seek(0)
            holder = (fh.read() or "").strip()
        except Exception:
            pass
        fh.close()
        detail = f" (holder pid file: {holder})" if holder else ""
        raise SingletonLockError(
            f"{label}: another instance already holds {lock_path}{detail}"
        ) from exc

    fh.seek(0)
    fh.truncate()
    fh.write(f"{os.getpid()}\n")
    fh.flush()
    os.fsync(fh.fileno())

    def _release() -> None:
        try:
            fcntl.flock(fh.fileno(), fcntl.LOCK_UN)
        except Exception:
            pass
        try:
            fh.close()
        except Exception:
            pass

    atexit.register(_release)
    if hold is not None:
        hold.append(fh)
    return fh


def acquire_or_exit(
    lock_path: Path,
    *,
    label: str = "rp608",
    hold: Optional[list] = None,
) -> TextIO:
    """Same as acquire_singleton_lock but exits process with code 3 on conflict."""
    try:
        return acquire_singleton_lock(lock_path, label=label, hold=hold)
    except SingletonLockError as exc:
        print(f"[RP-608 FATAL] {exc}", file=sys.stderr)
        print(
            "[RP-608 FATAL] Refusing duplicate shadow — stop the other process "
            "(prefer: systemctl restart rp608-shadow-observer) before relaunching.",
            file=sys.stderr,
        )
        sys.exit(3)
