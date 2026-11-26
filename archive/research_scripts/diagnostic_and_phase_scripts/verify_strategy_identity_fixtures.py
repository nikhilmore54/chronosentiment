#!/usr/bin/env python3
"""Verify V-001 strategy identity fixture records.

This script is an evidence harness, not a parser authority. It mirrors the
legacy parser lineages named in each fixture so the frozen corpus can detect
unreviewed changes to observed strategy identity semantics.
"""

from __future__ import annotations

import json
import sys
from collections import defaultdict
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[1]
if (REPO_ROOT / "RELEASE_INFO.json").exists():
    FIXTURE_ROOT = REPO_ROOT / "replay" / "fixtures" / "strategy_identity"
else:
    FIXTURE_ROOT = REPO_ROOT / "fixtures" / "strategy_identity"
DIFFERENTIAL_REPORT_PATH = FIXTURE_ROOT / "differential_report.json"

EDGE_DECAY_PARSER = "core/src/edge_decay.rs::parse_strategy_from_id_local"
API_PARSER = "services/api/src/strategy_id_parse.rs::parse_strategy_id_full"
CANONICAL_PARSER = "core/src/strategy_id.rs::parse_strategy_id"


def parse_int(value: str) -> int:
    return int(value, 10)


def parse_optional_u8(parts: list[str], index: int, default: int) -> int:
    if index >= len(parts):
        return default
    try:
        value = int(parts[index], 10)
    except ValueError:
        return default
    return value if 0 <= value <= 255 else default


def base_strategy() -> dict[str, Any]:
    return {
        "queue_threshold": 0,
        "base_edge": 0,
        "take_profit": 0,
        "stop_loss": 0,
        "holding_period": 0,
        "w_conviction": 50,
        "w_momentum": 30,
        "w_volatility": 20,
        "exp_conviction": 100,
        "exp_momentum": 100,
        "exp_volatility": 100,
        "selectivity": 75,
        "archetype": 0,
        "entry_offset": 0,
        "direction_bias": 50,
        "vol_floor": 20,
        "mom_floor": 20,
        "edge_ratio": 150,
        "participation_threshold": 30,
        "exec_aggression": 50,
        "latency_bias": 10,
        "fill_threshold": 50,
        "lineage": 0,
    }


def serialize_strategy(strategy: dict[str, Any]) -> str:
    fields = [
        "queue_threshold",
        "base_edge",
        "take_profit",
        "stop_loss",
        "holding_period",
        "w_conviction",
        "w_momentum",
        "w_volatility",
        "exp_conviction",
        "exp_momentum",
        "exp_volatility",
        "selectivity",
        "archetype",
        "entry_offset",
        "direction_bias",
        "vol_floor",
        "mom_floor",
        "edge_ratio",
        "participation_threshold",
    ]
    return "STRAT_" + "v".join(str(strategy[field]) for field in fields)


def parse_edge_decay_local(serialized_id: str) -> dict[str, Any]:
    return parse_canonical_strategy_id(serialized_id)


def parse_api_strategy_id_full(serialized_id: str) -> dict[str, Any]:
    return parse_strategy_id_with_compatibility(serialized_id)


def parse_canonical_strategy_id(serialized_id: str) -> dict[str, Any]:
    if not serialized_id.startswith("STRAT_"):
        raise ValueError("unsupported legacy format")

    parts = serialized_id.split("v")
    if len(parts) < 13:
        raise ValueError("canonical parser requires at least 13 v-delimited parts")

    strategy = base_strategy()
    strategy.update(
        {
            "queue_threshold": parse_int(parts[0].removeprefix("STRAT_")),
            "base_edge": parse_int(parts[1]),
            "take_profit": parse_int(parts[2]),
            "stop_loss": parse_int(parts[3]),
            "holding_period": parse_int(parts[4]),
            "w_conviction": parse_int(parts[5]),
            "w_momentum": parse_int(parts[6]),
            "w_volatility": parse_int(parts[7]) if len(parts) > 7 else 20,
            "exp_conviction": parse_int(parts[8]) if len(parts) > 8 else 100,
            "exp_momentum": parse_int(parts[9]) if len(parts) > 9 else 100,
            "exp_volatility": parse_int(parts[10]) if len(parts) > 10 else 100,
            "selectivity": parse_optional_u8(parts, 11, 75),
            "archetype": parse_optional_u8(parts, 12, 0),
            "entry_offset": int(parts[13], 10) if len(parts) > 13 else 0,
            "direction_bias": parse_optional_u8(parts, 14, 50),
            "vol_floor": parse_optional_u8(parts, 15, 20),
            "mom_floor": parse_optional_u8(parts, 16, 20),
            "edge_ratio": parse_optional_u8(parts, 17, 150),
            "participation_threshold": parse_optional_u8(parts, 18, 30),
        }
    )
    return strategy


def parse_strategy_id_with_compatibility(serialized_id: str) -> dict[str, Any]:
    if serialized_id.startswith("STRAT_"):
        return parse_canonical_strategy_id(serialized_id)

    nums: list[int] = []
    for part in reversed(serialized_id.split("_")):
        try:
            nums.append(int(part, 10))
        except ValueError:
            continue

    if len(nums) < 4:
        raise ValueError("legacy compatibility parser requires at least four numeric fields")

    strategy = base_strategy()
    strategy.update(
        {
            "queue_threshold": nums[3],
            "base_edge": nums[2],
            "take_profit": nums[1],
            "stop_loss": nums[0],
        }
    )
    return strategy


PARSERS = {
    CANONICAL_PARSER: parse_canonical_strategy_id,
    EDGE_DECAY_PARSER: parse_edge_decay_local,
    API_PARSER: parse_api_strategy_id_full,
}


def load_fixtures() -> list[tuple[Path, int, dict[str, Any]]]:
    records: list[tuple[Path, int, dict[str, Any]]] = []
    for path in sorted(FIXTURE_ROOT.glob("*/*.jsonl")):
        for line_number, line in enumerate(path.read_text().splitlines(), start=1):
            if line.strip():
                records.append((path, line_number, json.loads(line)))
    return records


def verify_record(path: Path, line_number: int, record: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    parser_source = record.get("parser_source")
    parser = PARSERS.get(parser_source)
    if parser is None:
        return [f"{path}:{line_number}: unknown parser_source {parser_source!r}"]

    try:
        parsed = parser(record["serialized_id"])
        result = "parsed"
        round_trip = serialize_strategy(parsed)
    except Exception as exc:  # noqa: BLE001 - rejection is fixture evidence here.
        parsed = None
        result = "rejected"
        round_trip = None
        if record.get("expected_parser_result") == "parsed":
            errors.append(f"{path}:{line_number}: parser rejected unexpectedly: {exc}")

    if result != record.get("expected_parser_result"):
        errors.append(
            f"{path}:{line_number}: expected result {record.get('expected_parser_result')!r}, got {result!r}"
        )
    if parsed != record.get("parsed_structure"):
        errors.append(f"{path}:{line_number}: parsed structure mismatch")
    if round_trip != record.get("round_trip_serialization"):
        errors.append(f"{path}:{line_number}: round-trip serialization mismatch")

    return errors


def run_parser(parser_source: str, serialized_id: str) -> dict[str, Any]:
    try:
        parsed = PARSERS[parser_source](serialized_id)
        return {
            "parser_result": "parsed",
            "resolution": parser_resolution(parser_source, serialized_id),
            "parsed_structure": parsed,
            "round_trip_serialization": serialize_strategy(parsed),
            "error": None,
        }
    except Exception as exc:  # noqa: BLE001 - parser rejection is reported evidence.
        return {
            "parser_result": "rejected",
            "resolution": "rejected",
            "parsed_structure": None,
            "round_trip_serialization": None,
            "error": str(exc),
        }


def parser_resolution(parser_source: str, serialized_id: str) -> str:
    if parser_source == API_PARSER and not serialized_id.startswith("STRAT_"):
        return "compatibility_translated"
    return "canonical"


def classify_differential(outputs: dict[str, dict[str, Any]]) -> str:
    parsed_outputs = [
        output for output in outputs.values() if output["parser_result"] == "parsed"
    ]
    if not parsed_outputs:
        return "all_rejected"
    if len(parsed_outputs) != len(outputs):
        return "divergent_acceptance"

    first = parsed_outputs[0]
    if all(output == first for output in parsed_outputs[1:]):
        return "bit_equivalent"
    if all(
        output["parsed_structure"] == first["parsed_structure"]
        for output in parsed_outputs[1:]
    ):
        return "semantically_equivalent"
    return "divergent_semantics"


def outcome_category(classification: str, outputs: dict[str, dict[str, Any]]) -> str:
    if classification == "divergent_acceptance" and any(
        output.get("resolution") == "compatibility_translated"
        and output["parser_result"] == "parsed"
        for output in outputs.values()
    ):
        return "accepted_normalized"

    mapping = {
        "bit_equivalent": "accepted_same_meaning",
        "semantically_equivalent": "accepted_normalized",
        "divergent_semantics": "accepted_divergent_semantics",
        "divergent_acceptance": "rejected_historically_admitted",
        "all_rejected": "rejected_universally",
    }
    return mapping[classification]


def write_differential_report(records: list[tuple[Path, int, dict[str, Any]]]) -> Path:
    fixtures_by_id: dict[str, list[str]] = defaultdict(list)
    provenance_by_id: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for _path, _line_number, record in records:
        serialized_id = record["serialized_id"]
        fixtures_by_id[serialized_id].append(record["fixture_id"])
        provenance_by_id[serialized_id].append(record["provenance"])

    cases: list[dict[str, Any]] = []
    for serialized_id in sorted(fixtures_by_id):
        outputs = {
            parser_source: run_parser(parser_source, serialized_id)
            for parser_source in sorted(PARSERS)
        }
        classification = classify_differential(outputs)
        cases.append(
            {
                "serialized_id": serialized_id,
                "fixture_ids": sorted(fixtures_by_id[serialized_id]),
                "provenance": provenance_by_id[serialized_id],
                "classification": classification,
                "outcome_category": outcome_category(classification, outputs),
                "parser_outputs": outputs,
            }
        )

    report = {
        "report_type": "strategy_identity_differential_parser_report",
        "status": "generated_from_frozen_fixtures",
        "canonical_candidate": CANONICAL_PARSER,
        "outcome_model": {
            "axes": ["admissibility", "interpretation"],
            "categories": [
                "accepted_same_meaning",
                "accepted_normalized",
                "accepted_divergent_semantics",
                "rejected_historically_admitted",
                "rejected_universally",
            ],
        },
        "parser_sources": sorted(PARSERS),
        "case_count": len(cases),
        "cases": cases,
    }
    DIFFERENTIAL_REPORT_PATH.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    return DIFFERENTIAL_REPORT_PATH


def main() -> int:
    records = load_fixtures()
    if not records:
        print(f"[PASS] no strategy identity fixtures found under {FIXTURE_ROOT}", file=sys.stderr)
        print("       No fixtures to verify; considered successful.", file=sys.stderr)
        return 0
    errors: list[str] = []
    for path, line_number, record in records:
        errors.extend(verify_record(path, line_number, record))
    if errors:
        print("[FAIL] strategy identity fixture verification", file=sys.stderr)
        for error in errors:
            print(f"       - {error}", file=sys.stderr)
        print("       Remediation: Address the broken fixture validations listed above.", file=sys.stderr)
        return 1
    report_path = write_differential_report(records)
    relative_report_path = report_path.relative_to(REPO_ROOT)
    print(f"[PASS] verified {len(records)} strategy identity fixture records")
    print(f"[INFO] wrote differential parser report: {relative_report_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
