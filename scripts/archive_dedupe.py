#!/usr/bin/env python3
"""Persistent (symbol, ts) dedupe index for cohort ingestion archives."""

from __future__ import annotations

import gzip
import json
from pathlib import Path


class GzipWriterPool:
    """
    Keep one gzip handle per file for the whole ingest run.

    Opening gzip in append mode per line corrupts streams (invalid block type).
    """

    def __init__(self) -> None:
        self._writers: dict[str, gzip.GzipFile] = {}

    def writeln(self, path: Path, text: str) -> None:
        if not text.endswith("\n"):
            text += "\n"
        key = str(path.resolve())
        if key not in self._writers:
            path.parent.mkdir(parents=True, exist_ok=True)
            if path.exists() and path.stat().st_size > 0:
                self._writers[key] = gzip.open(path, "at", encoding="utf-8")
            else:
                self._writers[key] = gzip.open(path, "wt", encoding="utf-8")
        self._writers[key].write(text)

    def flush_all(self) -> None:
        for fh in self._writers.values():
            fh.flush()

    def close_all(self) -> None:
        for fh in self._writers.values():
            fh.close()
        self._writers.clear()


def iter_gzip_jsonl(path: Path):
    """Yield (line_no, record|None, error|None) from a gzip jsonl file."""
    try:
        with gzip.open(path, "rt", encoding="utf-8") as f:
            for line_no, line in enumerate(f, 1):
                line = line.strip()
                if not line:
                    continue
                try:
                    yield line_no, json.loads(line), None
                except json.JSONDecodeError as e:
                    yield line_no, None, str(e)
    except (OSError, EOFError, Exception) as e:
        yield 0, None, str(e)
        return


class DedupeIndex:
    """Tracks persisted (symbol, ts) keys to prevent append duplication."""

    def __init__(self, index_path: Path):
        self.index_path = index_path
        self.seen: set[tuple[str, int]] = set()
        self.skipped = 0

    def load(self) -> int:
        if self.index_path.exists():
            with open(self.index_path) as f:
                data = json.load(f)
            self.seen = {tuple(k) for k in data.get("keys", [])}
            return len(self.seen)
        return 0

    def rebuild_from_archive(self, archive_dir: Path, cohort: set[str] | None = None) -> int:
        raw = archive_dir / "raw"
        if not raw.exists():
            return 0
        for sym_dir in raw.iterdir():
            if not sym_dir.is_dir():
                continue
            symbol = sym_dir.name
            if cohort and symbol not in cohort:
                continue
            for gz in sym_dir.glob("telemetry_stream_*.jsonl.gz"):
                for _line_no, rec, err in iter_gzip_jsonl(gz):
                    if err or rec is None:
                        continue
                    ts = rec.get("ts")
                    if ts is not None:
                        self.seen.add((symbol, int(ts)))
        return len(self.seen)

    def check_and_add(self, symbol: str, ts: int) -> bool:
        key = (symbol, int(ts))
        if key in self.seen:
            self.skipped += 1
            return False
        self.seen.add(key)
        return True

    def save(self) -> None:
        self.index_path.parent.mkdir(parents=True, exist_ok=True)
        payload = {
            "version": 1,
            "count": len(self.seen),
            "keys": [[s, t] for s, t in sorted(self.seen)],
        }
        tmp = self.index_path.with_suffix(".tmp")
        with open(tmp, "w") as f:
            json.dump(payload, f)
        tmp.replace(self.index_path)
