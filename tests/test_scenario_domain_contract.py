"""
Scenario Domain Contract v1 enforcement.

Authority chain:
  docs/contracts/SCENARIO_DOMAIN_CONTRACT_v1.md → fixtures/contracts/scenario_registry.json

Run:
  pytest tests/test_scenario_domain_contract.py -q
"""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURES_DIR = ROOT / "fixtures" / "contracts"
REGISTRY_FIXTURE = FIXTURES_DIR / "scenario_registry.json"
CONTRACT_VERSION = "1.0"


def load_registry() -> dict:
    return json.loads(REGISTRY_FIXTURE.read_text(encoding="utf-8"))


class TestScenarioRegistryFixture:
    def test_fixture_exists_and_version(self) -> None:
        assert REGISTRY_FIXTURE.is_file()
        data = load_registry()
        assert data["contract_version"] == CONTRACT_VERSION
        assert "domains" in data

    def test_eligible_domain_count_v1(self) -> None:
        data = load_registry()
        eligible = [d for d in data["domains"] if d.get("evaluation_eligible", False)]
        assert len(eligible) >= 2, "Phase C v1 requires multi-domain evaluation substrate"

    def test_domains_have_declared_provenance(self) -> None:
        data = load_registry()
        for domain in data["domains"]:
            assert domain.get("id"), "each domain must declare id"
            assert domain.get("domain_class"), "each domain must declare domain_class"
            substrate = domain.get("substrate_source") or {}
            assert substrate.get("kind"), f"{domain['id']} missing substrate_source.kind"
            assert substrate.get("reference"), f"{domain['id']} missing substrate_source.reference"

    def test_aggregation_isolated_from_registry(self) -> None:
        data = load_registry()
        agg = data.get("aggregation_v1") or {}
        assert agg.get("module") == "ScenarioAggregator"
        assert agg.get("reducer"), "aggregation policy must be explicit when declared"


class TestScenarioAuthorityInvariants:
    """Document-level invariants mirrored as tests (registry = declaration only)."""

    FORBIDDEN_REGISTRY_METHODS = (
        "select_best_domains",
        "select_domains",
        "rank_domains",
        "filter_domains",
    )

    def test_registry_rust_module_has_no_policy_methods(self) -> None:
        registry_rs = (
            ROOT
            / "infrastructure"
            / "observatory"
            / "api"
            / "src"
            / "scenario"
            / "registry.rs"
        )
        text = registry_rs.read_text(encoding="utf-8")
        for forbidden in self.FORBIDDEN_REGISTRY_METHODS:
            assert forbidden not in text, (
                f"ScenarioRegistry must remain declarative; found {forbidden!r}"
            )

    def test_aggregator_module_exists_separately(self) -> None:
        aggregator_rs = (
            ROOT
            / "infrastructure"
            / "observatory"
            / "api"
            / "src"
            / "scenario"
            / "aggregator.rs"
        )
        assert aggregator_rs.is_file()
        text = aggregator_rs.read_text(encoding="utf-8")
        assert "ScenarioAggregator" in text
        assert "robust_min" in text
