#!/usr/bin/env python3
"""Verify V-006 chronology byte fixture integrity.

Evidence harness only — not a producer authority. Validates that frozen
fixture bytes, excerpt hashes, and reference chronology hashes remain stable.
"""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
if (REPO_ROOT / "RELEASE_INFO.json").exists():
    FIXTURE_ROOT = REPO_ROOT / "replay" / "fixtures" / "chronology_serialization"
else:
    FIXTURE_ROOT = REPO_ROOT / "fixtures" / "chronology_serialization"


def streaming_chronology_hash(data: bytes) -> str:
    hasher = hashlib.sha256()
    for line in data.splitlines(keepends=True):
        hasher.update(line)
    return hasher.hexdigest()


def load_json(path: Path) -> dict:
    return json.loads(path.read_text())


def verify_fixture_dir(path: Path) -> list[str]:
    errors: list[str] = []
    meta_path = path / "fixture_meta.json"
    excerpt_path = path / "substrate_excerpt.jsonl"
    manifest_path = path / "manifest.json"
    first_line_hex_path = path / "first_line.hex"

    if not meta_path.exists():
        return [f"{path.name}: missing fixture_meta.json"]

    meta = load_json(meta_path)
    fixture_id = meta.get("fixture_id", path.name)
    dialect = meta.get("dialect")

    if not excerpt_path.exists():
        errors.append(f"{fixture_id}: missing substrate_excerpt.jsonl")
        return errors

    excerpt_bytes = excerpt_path.read_bytes()
    computed = streaming_chronology_hash(excerpt_bytes)
    expected = meta.get("excerpt_chronology_hash")
    if computed != expected:
        errors.append(
            f"{fixture_id}: excerpt hash mismatch expected={expected} actual={computed}"
        )

    lines = excerpt_bytes.splitlines(keepends=True)
    if meta.get("excerpt_line_count") != len(lines):
        errors.append(
            f"{fixture_id}: excerpt_line_count mismatch meta={meta.get('excerpt_line_count')} actual={len(lines)}"
        )

    if first_line_hex_path.exists() and lines:
        expected_hex = first_line_hex_path.read_text().strip()
        actual_hex = lines[0].hex()
        if expected_hex != actual_hex:
            errors.append(f"{fixture_id}: first_line.hex mismatch")

    if manifest_path.exists():
        manifest = load_json(manifest_path)
        required_common = {"substrate", "chronology_hash", "gaps"}
        missing = required_common - set(manifest.keys())
        if missing:
            errors.append(f"{fixture_id}: manifest missing keys {sorted(missing)}")

        if dialect == "A":
            required = {
                "source",
                "resolution",
                "capture_method",
                "import_timestamp",
                "capture_start",
                "capture_end",
                "total_ticks",
            }
            missing_a = required - set(manifest.keys())
            if missing_a:
                errors.append(f"{fixture_id}: dialect A manifest missing {sorted(missing_a)}")
        elif dialect == "B":
            if "provenance" in manifest:
                errors.append(f"{fixture_id}: dialect B manifest must not include provenance")
        elif dialect == "C":
            if "provenance" not in manifest:
                errors.append(f"{fixture_id}: dialect C manifest missing provenance")

    # tick schema checks
    for idx, line in enumerate(lines, start=1):
        try:
            tick = json.loads(line.decode("utf-8"))
        except json.JSONDecodeError:
            errors.append(f"{fixture_id}: invalid JSON on line {idx}")
            continue
        expected_keys = {"symbol", "timestamp", "price", "volume", "is_buyer_maker"}
        if set(tick.keys()) != expected_keys:
            errors.append(
                f"{fixture_id}: line {idx} keys {sorted(tick.keys())} != expected 5-field tick schema"
            )

    return errors


def verify_reference_hashes() -> list[str]:
    errors: list[str] = []
    ref_path = FIXTURE_ROOT / "reference_hashes.json"
    if not ref_path.exists():
        return ["reference_hashes.json missing"]

    payload = load_json(ref_path)
    for entry in payload.get("references", []):
        if "path" not in entry:
            continue
        source = REPO_ROOT / entry["path"]
        if not source.exists():
            errors.append(f"reference source missing: {entry['path']}")
            continue
        if "streaming_line_sha256" in entry:
            computed = streaming_chronology_hash(source.read_bytes())
            expected = entry["streaming_line_sha256"]
            if computed != expected:
                errors.append(
                    f"reference streaming hash mismatch for {entry['path']}"
                )
            manifest_expected = entry.get("manifest_chronology_hash")
            if manifest_expected and computed != manifest_expected:
                errors.append(
                    f"reference manifest chronology_hash mismatch for {entry['path']}"
                )
            if entry.get("match") is False:
                errors.append(f"reference entry marked match=false for {entry['path']}")
    return errors


def main() -> int:
    if not FIXTURE_ROOT.exists():
        print(f"Fixture root missing: {FIXTURE_ROOT}")
        return 1

    fixture_dirs = sorted(
        p for p in FIXTURE_ROOT.iterdir() if p.is_dir() and (p / "fixture_meta.json").exists()
    )
    if not fixture_dirs:
        print("No chronology byte fixtures found.")
        return 1

    all_errors: list[str] = []
    for fixture_dir in fixture_dirs:
        all_errors.extend(verify_fixture_dir(fixture_dir))

    if not (REPO_ROOT / "RELEASE_INFO.json").exists():
        all_errors.extend(verify_reference_hashes())

    if all_errors:
        print("FAIL: chronology byte fixture verification")
        for err in all_errors:
            print(f"  - {err}")
        return 1

    print(f"PASS: verified {len(fixture_dirs)} chronology byte fixture(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
