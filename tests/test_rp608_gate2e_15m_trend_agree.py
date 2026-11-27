"""Tests for RP-608-GATE2E-15M-OHLCV-TREND-AGREE-001 (Rust-owned via CLI)."""

from __future__ import annotations

import pytest

from scripts.rp608_gate2e_15m_trend_agree import (
    PROTOCOL_ID,
    agreement,
    resolve_decide_bin,
)


def test_protocol_id():
    assert PROTOCOL_ID == "RP-608-GATE2E-15M-OHLCV-TREND-AGREE-001"


def test_fail_closed_without_bin(monkeypatch):
    monkeypatch.setattr(
        "scripts.rp608_gate2e_15m_trend_agree.resolve_decide_bin",
        lambda: None,
    )
    # Reset cached session
    import scripts.rp608_gate2e_15m_trend_agree as mod

    monkeypatch.setattr(mod, "_SESSION", None)
    stamp = agreement([100.0 + i for i in range(25)], "LONG")
    assert stamp["blocks_entry"] is True
    assert stamp["block_reason"] == "RUST_TREND_AGREE_BIN_MISSING"
    assert stamp["rust_ok"] is False


@pytest.mark.skipif(resolve_decide_bin() is None, reason="rp608_position_decide with --trend-agree-loop not built")
def test_rust_rising_allows_long():
    import scripts.rp608_gate2e_15m_trend_agree as mod

    mod._SESSION = None
    closes = [100.0 + i for i in range(25)]
    stamp = agreement(closes, "LONG")
    assert stamp.get("rust_ok") is True
    assert stamp["blocks_entry"] is False
    assert stamp["agreement"] == "ALLOW"
    assert stamp["agree_count"] == 3


@pytest.mark.skipif(resolve_decide_bin() is None, reason="rp608_position_decide with --trend-agree-loop not built")
def test_rust_rising_blocks_short():
    import scripts.rp608_gate2e_15m_trend_agree as mod

    mod._SESSION = None
    closes = [100.0 + i for i in range(25)]
    stamp = agreement(closes, "SHORT")
    assert stamp.get("rust_ok") is True
    assert stamp["blocks_entry"] is True
    assert stamp["block_reason"] == "LOOKBACK_OPPOSE"


@pytest.mark.skipif(resolve_decide_bin() is None, reason="rp608_position_decide with --trend-agree-loop not built")
def test_rust_short_history():
    import scripts.rp608_gate2e_15m_trend_agree as mod

    mod._SESSION = None
    stamp = agreement([100.0, 101.0], "LONG")
    assert stamp.get("rust_ok") is True
    assert stamp["blocks_entry"] is True
    assert stamp["block_reason"] == "INSUFFICIENT_AGREE_VOTES"
