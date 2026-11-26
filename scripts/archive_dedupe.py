#!/usr/bin/env python3
"""Gzip JSONL reader helper for remaining Python cert/compare scripts.

Write-path dedupe (`DedupeIndex`, `GzipWriterPool`) lives in Rust
(`cs-ingest` / `financial/ese`). Prefer `cs-ingest verify` for new checks.
"""

from __future__ import annotations

import gzip
import json
from pathlib import Path


def iter_gzip_jsonl(path: Path):
    """Yield (line_no, record|None, error|None) from a gzip jsonl file."""
    try:
        with gzip.open(path, "rt", encoding="utf-8") as f:
            for line_no, line in enumerate(f, 1):
                line = line.strip()
                if line:
                    try:
                        yield line_no, json.loads(line), None
                    except json.JSONDecodeError as e:
                        yield line_no, None, str(e)
    except (OSError, EOFError, Exception) as e:
        yield 0, None, str(e)
        return
