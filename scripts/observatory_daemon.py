#!/usr/bin/env python3
"""Long-lived live_observatory process — preserves Rust warm state across commits."""

from __future__ import annotations

import json
import os
import subprocess
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OBSERVATORY_BIN = ROOT / "target/release/examples/live_observatory"


class ObservatoryDaemon:
  def __init__(self, source_type: str = "LIVE"):
    self.source_type = source_type
    self.proc: subprocess.Popen | None = None

  def start(self, timeout_sec: float = 30.0) -> None:
    if self.proc is not None:
      return
    env = {"SOURCE_TYPE": self.source_type}
    self.proc = subprocess.Popen(
      [str(OBSERVATORY_BIN)],
      stdin=subprocess.PIPE,
      stdout=subprocess.PIPE,
      stderr=subprocess.DEVNULL,
      text=True,
      bufsize=1,
      cwd=str(ROOT),
      env={**os.environ, **env},
    )
    deadline = time.time() + timeout_sec
    while time.time() < deadline:
      line = self.proc.stdout.readline()
      if not line:
        if self.proc.poll() is not None:
          err = self.proc.stderr.read() if self.proc.stderr else ""
          raise RuntimeError(f"Observatory exited during startup: {err}")
        continue
      if "[OBSERVATORY_READY]" in line:
        return
    raise TimeoutError("Observatory did not emit [OBSERVATORY_READY]")

  def send_batch(self, batch: list[dict], timeout_sec: float = 120.0) -> list[str]:
    if not batch:
      return []
    if self.proc is None or self.proc.stdin is None or self.proc.stdout is None:
      raise RuntimeError("Daemon not started")

    self.proc.stdin.write(json.dumps(batch) + "\n")
    self.proc.stdin.flush()

    telemetry: list[str] = []
    deadline = time.time() + timeout_sec
    while time.time() < deadline:
      line = self.proc.stdout.readline()
      if not line:
        if self.proc.poll() is not None:
          break
        continue
      if line.startswith("[TELEMETRY]"):
        telemetry.append(line)
      elif line.startswith("[BATCH_COMPLETE]"):
        break
    return telemetry

  def shutdown(self) -> None:
    if self.proc and self.proc.stdin:
      try:
        self.proc.stdin.close()
      except Exception:
        pass
    if self.proc:
      self.proc.terminate()
      try:
        self.proc.wait(timeout=5)
      except subprocess.TimeoutExpired:
        self.proc.kill()
    self.proc = None

  def __enter__(self):
    self.start()
    return self

  def __exit__(self, *args):
    self.shutdown()
