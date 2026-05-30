"""
UI API Contract v1 enforcement.

Authority chain:
  docs/contracts/UI_API_CONTRACT_v1.md → fixtures/contracts/ → this module

Layers:
  1. Required-field presence on canonical fixtures
  2. Forbidden-field absence
  3. Live API conformance (optional; set UI_CONTRACT_TEST_API_URL)

Run:
  pytest tests/test_ui_api_contract.py -q
  UI_CONTRACT_TEST_API_URL=http://localhost:8000 pytest tests/test_ui_api_contract.py -q
"""

from __future__ import annotations

import json
import os
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any

import pytest

ROOT = Path(__file__).resolve().parents[1]
FIXTURES_DIR = ROOT / "fixtures" / "contracts"
UI_SRC_DIR = ROOT / "my-chrono-sentiment-ui" / "src"
CONTRACT_VERSION = "1.0"
LIVE_API_URL = os.environ.get("UI_CONTRACT_TEST_API_URL", "http://localhost:8000").rstrip("/")

FORBIDDEN_HOST_PATTERNS = (
    "http://localhost:8000",
    "http://localhost:8080",
    "https://staging-api",
)

# ---------------------------------------------------------------------------
# Contract rules (derived from UI_API_CONTRACT_v1.md — not from mock payloads)
# ---------------------------------------------------------------------------

STRATEGY_EVALUATION_REQUIRED = ("strategy_id", "execution_fitness")

ENDPOINT_RULES = {
    "run_ga": {
        "fixture": "run_ga.response.json",
        "required_top_level": ("global_best",),
        "required_nested": {
            "global_best": STRATEGY_EVALUATION_REQUIRED,
        },
        "forbidden_top_level": ("results",),
    },
    "global_ranking": {
        "fixture": "global_ranking.response.json",
        "required_top_level": ("rankings", "total"),
        "required_nested": {
            "rankings[]": STRATEGY_EVALUATION_REQUIRED,
        },
        "forbidden_top_level": ("ranking",),
    },
    "compare_strategies_minimal": {
        "fixture": "compare_strategies.response.json",
        "required_top_level": ("ranking", "comparison_summary"),
        "required_nested": {
            "ranking[]": STRATEGY_EVALUATION_REQUIRED,
            "comparison_summary": ("reason",),
        },
        "forbidden_top_level": ("results", "analytical_conclusion"),
    },
    "compare_strategies_extended": {
        "fixture": "compare_strategies.response.extended.json",
        "required_top_level": ("ranking", "comparison_summary"),
        "required_nested": {
            "ranking[]": STRATEGY_EVALUATION_REQUIRED,
            "comparison_summary": ("reason",),
        },
        "optional_nested_present": {
            "comparison_summary": (
                "replay_integrity",
                "timestamp_cohesion",
                "sync_state",
                "governor_action",
                "replay_certified",
                "metrics",
            ),
        },
        "forbidden_top_level": ("results", "analytical_conclusion"),
    },
    "inspect_strategy": {
        "fixture": "inspect_strategy.response.json",
        "required_top_level": ("strategy_id", "narrative_blocks", "execution_trace"),
        "required_nested": {
            "narrative_blocks[]": ("sequence_id", "group", "narrative"),
            "execution_trace[]": ("sequence_id",),
        },
        "forbidden_top_level": (),
    },
}


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def load_json(path: Path) -> Any:
    with path.open(encoding="utf-8") as fh:
        return json.load(fh)


def load_fixture(name: str) -> Any:
    return load_json(FIXTURES_DIR / name)


def _get_nested(obj: dict[str, Any], dotted: str) -> Any:
    if dotted.endswith("[]"):
        key = dotted[:-2]
        items = obj.get(key)
        assert isinstance(items, list), f"Expected list at {key!r}, got {type(items).__name__}"
        return items
    value = obj
    for part in dotted.split("."):
        assert isinstance(value, dict), f"Expected dict traversing {dotted!r}"
        value = value[part]
    return value


def assert_required_fields(payload: dict[str, Any], rules: dict[str, Any]) -> None:
    for key in rules.get("required_top_level", ()):
        assert key in payload, f"Missing required top-level field {key!r}"

    for path, fields in rules.get("required_nested", {}).items():
        if path.endswith("[]"):
            items = _get_nested(payload, path)
            assert items, f"Expected at least one item in {path}"
            for item in items:
                for field in fields:
                    assert field in item, f"Missing {field!r} in {path} item"
                    assert isinstance(item[field], (str, int, float)), (
                        f"{path}.{field} must be scalar, got {type(item[field]).__name__}"
                    )
        else:
            nested = _get_nested(payload, path)
            for field in fields:
                assert field in nested, f"Missing {field!r} in {path}"


def assert_forbidden_fields(payload: dict[str, Any], forbidden: tuple[str, ...]) -> None:
    for key in forbidden:
        assert key not in payload, f"Forbidden top-level field present: {key!r}"


def assert_optional_fields_present(payload: dict[str, Any], optional: dict[str, tuple[str, ...]]) -> None:
    for path, fields in optional.items():
        nested = _get_nested(payload, path)
        for field in fields:
            assert field in nested, f"Expected optional field {path}.{field} in extended fixture"


def structural_superset(actual: Any, template: Any, path: str = "$") -> None:
    """Actual response must contain template keys with compatible types."""
    if isinstance(template, dict):
        assert isinstance(actual, dict), f"{path}: expected object, got {type(actual).__name__}"
        for key, tmpl_val in template.items():
            assert key in actual, f"{path}: missing key {key!r}"
            structural_superset(actual[key], tmpl_val, f"{path}.{key}")
        return

    if isinstance(template, list):
        assert isinstance(actual, list), f"{path}: expected array, got {type(actual).__name__}"
        if template:
            assert actual, f"{path}: expected non-empty array"
            structural_superset(actual[0], template[0], f"{path}[0]")
        return

    assert isinstance(actual, type(template)), (
        f"{path}: expected {type(template).__name__}, got {type(actual).__name__}"
    )


def api_reachable() -> bool:
    try:
        with urllib.request.urlopen(f"{LIVE_API_URL}/health", timeout=2) as resp:
            return 200 <= resp.status < 300
    except (urllib.error.URLError, TimeoutError, OSError):
        return False


def http_get(path: str) -> Any:
    with urllib.request.urlopen(f"{LIVE_API_URL}{path}", timeout=10) as resp:
        return json.loads(resp.read().decode("utf-8"))


def http_post(path: str, body: dict[str, Any]) -> Any:
    data = json.dumps(body).encode("utf-8")
    req = urllib.request.Request(
        f"{LIVE_API_URL}{path}",
        data=data,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=10) as resp:
        return json.loads(resp.read().decode("utf-8"))


# ---------------------------------------------------------------------------
# Layer 0 — manifest & fixture authority
# ---------------------------------------------------------------------------


class TestContractManifest:
    def test_manifest_contract_version(self) -> None:
        manifest = load_fixture("manifest.json")
        assert manifest["contract_version"] == CONTRACT_VERSION

    def test_manifest_fixture_paths_exist(self) -> None:
        manifest = load_fixture("manifest.json")
        for _key, filename in manifest["fixtures"].items():
            assert (FIXTURES_DIR / filename).is_file(), f"Missing fixture file: {filename}"

    def test_manifest_endpoint_inventory_complete(self) -> None:
        manifest = load_fixture("manifest.json")
        paths = {entry["path"] for entry in manifest["endpoint_inventory"]}
        expected = {
            "/observatory",
            "/health",
            "/run_ga",
            "/signals/latest",
            "/ga/strategy-store",
            "/inspect_strategy",
            "/compare_strategies",
            "/ga/global-ranking",
        }
        assert expected <= paths


# ---------------------------------------------------------------------------
# Layer 1 & 2 — fixture shape (required + forbidden)
# ---------------------------------------------------------------------------


@pytest.mark.parametrize("rule_name", list(ENDPOINT_RULES.keys()))
def test_fixture_required_and_forbidden_fields(rule_name: str) -> None:
    rules = ENDPOINT_RULES[rule_name]
    payload = load_fixture(rules["fixture"])
    assert_required_fields(payload, rules)
    assert_forbidden_fields(payload, rules.get("forbidden_top_level", ()))


def test_extended_compare_fixture_includes_optional_surfaces() -> None:
    rules = ENDPOINT_RULES["compare_strategies_extended"]
    payload = load_fixture(rules["fixture"])
    assert_optional_fields_present(payload, rules["optional_nested_present"])


def test_minimal_compare_fixture_omits_optional_certification_surfaces() -> None:
    payload = load_fixture("compare_strategies.response.json")
    summary = payload["comparison_summary"]
    optional = {
        "replay_integrity",
        "timestamp_cohesion",
        "sync_state",
        "governor_action",
        "replay_certified",
        "metrics",
    }
    assert not optional.intersection(summary.keys())


def test_global_ranking_fixture_uses_envelope_not_bare_array() -> None:
    payload = load_fixture("global_ranking.response.json")
    assert isinstance(payload, dict)
    assert "rankings" in payload and "total" in payload


# ---------------------------------------------------------------------------
# Layer 3 — structural superset (template ⊆ response)
# ---------------------------------------------------------------------------


@pytest.mark.parametrize(
    "fixture_name",
    [
        "run_ga.response.json",
        "global_ranking.response.json",
        "compare_strategies.response.json",
        "inspect_strategy.response.json",
    ],
)
def test_minimal_fixture_is_valid_json_object(fixture_name: str) -> None:
    payload = load_fixture(fixture_name)
    assert isinstance(payload, dict)


# ---------------------------------------------------------------------------
# Step 2 — transport authority (no hardcoded hosts in components)
# ---------------------------------------------------------------------------


class TestTransportAuthority:
    def test_api_base_url_only_in_config(self) -> None:
        config_file = UI_SRC_DIR / "config" / "api.js"
        assert config_file.is_file()
        for pattern in FORBIDDEN_HOST_PATTERNS:
            assert pattern in config_file.read_text(encoding="utf-8")

    def test_components_do_not_embed_api_hosts(self) -> None:
        allowed = {UI_SRC_DIR / "config" / "api.js"}
        offenders: list[str] = []
        for path in UI_SRC_DIR.rglob("*.js"):
            if path in allowed:
                continue
            text = path.read_text(encoding="utf-8")
            for pattern in FORBIDDEN_HOST_PATTERNS:
                if pattern in text:
                    offenders.append(f"{path.relative_to(ROOT)} contains {pattern!r}")
        assert not offenders, "\n".join(offenders)

    def test_services_api_is_single_routing_helper(self) -> None:
        services_api = UI_SRC_DIR / "services" / "api.js"
        text = services_api.read_text(encoding="utf-8")
        assert "API_BASE_URL" in text
        assert "apiUrl" in text


# ---------------------------------------------------------------------------
# Live API conformance (mock or Rust — same validation path)
# ---------------------------------------------------------------------------


@pytest.mark.skipif(not api_reachable(), reason=f"API not reachable at {LIVE_API_URL}")
class TestLiveApiContractConformance:
    def test_get_global_ranking_shape(self) -> None:
        template = load_fixture("global_ranking.response.json")
        actual = http_get("/ga/global-ranking")
        if isinstance(actual, list):
            pytest.fail("Deprecated bare-array envelope; use { rankings, total }")
        assert_required_fields(actual, ENDPOINT_RULES["global_ranking"])
        assert_forbidden_fields(actual, ENDPOINT_RULES["global_ranking"]["forbidden_top_level"])
        structural_superset(actual, template)

    def test_get_run_ga_shape(self) -> None:
        template = load_fixture("run_ga.response.json")
        actual = http_get("/run_ga")
        assert_required_fields(actual, ENDPOINT_RULES["run_ga"])
        structural_superset(actual, template)

    def test_post_compare_strategies_shape(self) -> None:
        template = load_fixture("compare_strategies.response.json")
        body = {
            "strategies": [
                {"strategy_config": {"strategy_id": "strat_200_5_4_2"}},
                {"strategy_config": {"strategy_id": "strat_150_3_3_1"}},
            ],
            "scenarios": [],
            "seed": 42,
        }
        actual = http_post("/compare_strategies", body)
        assert_required_fields(actual, ENDPOINT_RULES["compare_strategies_minimal"])
        assert_forbidden_fields(
            actual, ENDPOINT_RULES["compare_strategies_minimal"]["forbidden_top_level"]
        )
        assert "results" not in actual
        assert "analytical_conclusion" not in actual
        structural_superset(actual, template)

    def test_post_inspect_strategy_wire_shape(self) -> None:
        template = load_fixture("inspect_strategy.response.json")
        body = {"strategy_id": "strat_200_5_4_2", "seed": 42}
        actual = http_post("/inspect_strategy", body)
        assert_required_fields(actual, ENDPOINT_RULES["inspect_strategy"])
        structural_superset(actual, template)

    def test_contract_version_header_when_present(self) -> None:
        req = urllib.request.Request(f"{LIVE_API_URL}/health", method="GET")
        with urllib.request.urlopen(req, timeout=5) as resp:
            version = resp.headers.get("X-Contract-Version")
            if version is not None:
                assert version == CONTRACT_VERSION
