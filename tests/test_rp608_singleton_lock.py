#!/usr/bin/env python3
"""Unit tests for RP-608 singleton flock (duplicate shadow prevention)."""

from __future__ import annotations

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from scripts.rp608_singleton_lock import (  # noqa: E402
    SingletonLockError,
    acquire_or_exit,
    acquire_singleton_lock,
)


class TestRp608SingletonLock(unittest.TestCase):
    def test_second_acquire_raises(self):
        with tempfile.TemporaryDirectory() as td:
            lock = Path(td) / "shadow.singleton.lock"
            hold: list = []
            fh1 = acquire_singleton_lock(lock, label="test", hold=hold)
            self.assertTrue(lock.exists())
            with self.assertRaises(SingletonLockError):
                acquire_singleton_lock(lock, label="test")
            fh1.close()

    def test_acquire_or_exit_code_3(self):
        """Second holder must exit 3 (daemon contract)."""
        with tempfile.TemporaryDirectory() as td:
            lock = Path(td) / "rp608_crypto_shadow.singleton.lock"
            hold: list = []
            acquire_singleton_lock(lock, label="parent", hold=hold)
            code = (
                "import sys\n"
                f"sys.path.insert(0, {str(WORKSPACE)!r})\n"
                "from scripts.rp608_singleton_lock import acquire_or_exit\n"
                "from pathlib import Path\n"
                f"acquire_or_exit(Path({str(lock)!r}), label='child')\n"
            )
            proc = subprocess.run(
                [sys.executable, "-c", code],
                capture_output=True,
                text=True,
                timeout=10,
            )
            self.assertEqual(proc.returncode, 3, msg=proc.stderr)
            self.assertIn("another instance already holds", proc.stderr)


if __name__ == "__main__":
    unittest.main()
