#!/usr/bin/env python3
"""Shared archive-reading helpers for Python cert/compare scripts.

Stage-1 baseline CLI moved to `cs-ingest verify`. Keep these helpers only
for `compare_replay_equivalence.py` until that script is ported.
"""

from __future__ import annotations

import json
from pathlib import Path

from archive_dedupe import iter_gzip_jsonl


def load_cohort_symbols(cohort_file: Path) -> set[str]:
    return {line.strip() for line in cohort_file.read_text().splitlines() if line.strip()}


def cohort_symbol_names(cohort: set[str]) -> set[str]:
    return set(cohort)


def load_manifest(archive_dir: Path, run_label: str | None) -> dict | None:
    if run_label and (archive_dir / "manifests" / f"ingestion_{run_label}.json").exists():
        path = archive_dir / "manifests" / f"ingestion_{run_label}.json"
    else:
        path = archive_dir / "ingestion_manifest.json"
    if not path.exists():
        return None
    with open(path) as f:
        return json.load(f)


def iter_archive_records(archive_dir: Path, cohort: set[str] | None):
    """Yield per-record dicts from Layer-1 barriers or Layer-2 gzip streams."""
    raw = archive_dir / "raw"
    if not raw.exists():
        return
    for sym_dir in sorted(raw.iterdir()):
        if not sym_dir.is_dir():
            continue
        symbol = sym_dir.name
        if cohort and symbol not in cohort:
            continue

        barriers_dir = sym_dir / "barriers"
        if barriers_dir.exists():
            barrier_files = sorted(barriers_dir.glob("*.json"), key=lambda p: int(p.stem))
            for bf in barrier_files:
                try:
                    rec = json.loads(bf.read_text())
                    yield {"ok": True, "symbol": symbol, "record": rec, "file": str(bf)}
                except Exception as e:
                    yield {
                        "ok": False,
                        "symbol": symbol,
                        "file": str(bf),
                        "line": 0,
                        "error": str(e),
                    }
            continue

        for gz in sorted(sym_dir.glob("telemetry_stream_*.jsonl.gz")):
            for line_no, rec, err in iter_gzip_jsonl(gz):
                if err and rec is None:
                    yield {
                        "ok": False,
                        "symbol": symbol,
                        "file": str(gz),
                        "line": line_no,
                        "error": err,
                        "corrupt_file": True,
                    }
                    break
                if rec is None:
                    yield {
                        "ok": False,
                        "symbol": symbol,
                        "file": str(gz),
                        "line": line_no,
                        "error": err or "json error",
                    }
                    continue
                yield {"ok": True, "symbol": symbol, "record": rec, "file": str(gz)}
