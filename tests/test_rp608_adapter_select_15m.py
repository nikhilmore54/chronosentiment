"""Tests for RP-608-ADAPTER-SELECT-15M-V1 (Rust-owned via CLI)."""

from __future__ import annotations

import pytest

from scripts.rp608_adapter_select_15m import (
    ENTER_ABS,
    PROTOCOL_ID,
    recommend,
    resolve_decide_bin,
    twin_enabled,
)


def test_protocol_id():
    assert PROTOCOL_ID == "RP-608-ADAPTER-SELECT-15M-V1"
    assert ENTER_ABS == 0.35


def test_twin_default_off(monkeypatch):
    monkeypatch.delenv("RP608_ADAPTER_SELECT_15M_TWIN", raising=False)
    assert twin_enabled(default=False) is False


def test_fail_closed_without_bin(monkeypatch):
    monkeypatch.setattr(
        "scripts.rp608_adapter_select_15m.resolve_decide_bin",
        lambda: None,
    )
    import scripts.rp608_adapter_select_15m as mod

    monkeypatch.setattr(mod, "_SESSION", None)
    stamp = recommend([100.0 + i for i in range(80)], "LONG")
    assert stamp["blocks_entry"] is True
    assert stamp["block_reason"] == "RUST_ADAPTER_SELECT_15M_BIN_MISSING"
    assert stamp["rust_ok"] is False


@pytest.mark.skipif(
    resolve_decide_bin() is None,
    reason="rp608_position_decide with --adapter-select-15m-loop not built",
)
def test_rust_rising_enters_long():
    import scripts.rp608_adapter_select_15m as mod

    mod._SESSION = None
    closes = [100.0 + i for i in range(80)]
    stamp = recommend(closes, "LONG")
    assert stamp.get("rust_ok") is True
    assert stamp["blocks_entry"] is False
    assert stamp["band"] == "ENTER"
    assert float(stamp["side_score"]) >= ENTER_ABS


@pytest.mark.skipif(
    resolve_decide_bin() is None,
    reason="rp608_position_decide with --adapter-select-15m-loop not built",
)
def test_rust_rising_blocks_short():
    import scripts.rp608_adapter_select_15m as mod

    mod._SESSION = None
    closes = [100.0 + i for i in range(80)]
    stamp = recommend(closes, "SHORT")
    assert stamp.get("rust_ok") is True
    assert stamp["blocks_entry"] is True
    assert stamp["band"] == "STANDASIDE"


@pytest.mark.skipif(
    resolve_decide_bin() is None,
    reason="rp608_position_decide with --adapter-select-15m-loop not built",
)
def test_rust_short_history_insufficient():
    import scripts.rp608_adapter_select_15m as mod

    mod._SESSION = None
    stamp = recommend([100.0, 101.0], "LONG")
    assert stamp.get("rust_ok") is True
    assert stamp["blocks_entry"] is True
    assert stamp["band"] == "INSUFFICIENT"


@pytest.mark.skipif(
    resolve_decide_bin() is None,
    reason="rp608_position_decide with --adapter-select-15m-loop not built",
)
def test_rust_deterministic():
    import scripts.rp608_adapter_select_15m as mod

    mod._SESSION = None
    closes = [100.0 + i for i in range(80)]
    a = recommend(closes, "LONG")
    b = recommend(closes, "LONG")
    assert a["score"] == b["score"]
    assert a["side_score"] == b["side_score"]
    assert a["band"] == b["band"]
