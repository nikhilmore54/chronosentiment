"""
Replay Attestation Contract v1 enforcement.

Authority chain:
  docs/contracts/REPLAY_ATTESTATION_CONTRACT_v1.md
    → fixtures/contracts/attestation_record.example.json

Run:
  pytest tests/test_replay_attestation_contract.py -q
"""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURES_DIR = ROOT / "fixtures" / "contracts"
ATTESTATION_FIXTURE = FIXTURES_DIR / "attestation_record.example.json"
CONTRACT_DOC = ROOT / "docs" / "contracts" / "REPLAY_ATTESTATION_CONTRACT_v1.md"
SIGNATURES_RS = (
    ROOT / "infrastructure" / "observatory" / "api" / "src" / "signatures.rs"
)
CONTRACT_VERSION = "1.0"

REQUIRED_ATTESTATION_FIELDS = (
    "scenario_id",
    "substrate_reference",
    "event_count",
    "expected_event_hash",
    "result_hash",
    "attestation_level",
    "attestation_timestamp",
    "engine_mode",
)


def load_attestation_example() -> dict:
    return json.loads(ATTESTATION_FIXTURE.read_text(encoding="utf-8"))


class TestAttestationFixture:
    def test_fixture_exists_and_version(self) -> None:
        assert ATTESTATION_FIXTURE.is_file()
        data = load_attestation_example()
        assert data["contract_version"] == CONTRACT_VERSION

    def test_required_diagnostic_fields(self) -> None:
        data = load_attestation_example()
        for field in REQUIRED_ATTESTATION_FIELDS:
            assert field in data, f"missing AttestationRecord field: {field}"

    def test_attestation_level_in_range(self) -> None:
        level = load_attestation_example()["attestation_level"]
        assert 0 <= level <= 4


class TestAttestationCodeInvariants:
    def test_contract_doc_exists(self) -> None:
        assert CONTRACT_DOC.is_file()
        text = CONTRACT_DOC.read_text(encoding="utf-8")
        assert "event_hash" in text
        assert "result_hash" in text
        assert "METADATA_SIGNATURE" in text

    def test_replay_signature_inputs_documented_in_code(self) -> None:
        text = SIGNATURES_RS.read_text(encoding="utf-8")
        assert "session_id" in text
        assert "event_count" in text
        assert "compute_replay_signature" in text

    def test_event_hash_not_named_replay_signature_in_contract(self) -> None:
        text = CONTRACT_DOC.read_text(encoding="utf-8")
        assert "MUST NOT relabel this field as `event_hash`" in text
