"""RP-608 crypto shadow multi-symbol wiring (ETH twin; BTC defaults locked)."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
import sys

WORKSPACE_DIR = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE_DIR))

from scripts.run_rp608_shadow_daemon import (
    ALLOWED_CRYPTO_SHADOW_SYMBOLS,
    DEFAULT_CRYPTO_SHADOW_SYMBOL,
    DEFAULT_OUTPUT_DIR,
    RP608ShadowDaemon,
    binance_ws_stream_url,
    default_output_dir_for_symbol,
    normalize_crypto_shadow_symbol,
)
from scripts.rp608_ofx_crypto_aggtrade_journal import OfxCryptoAggTradeJournal
from scripts.rp608_trade_plan_overlay import (
    calculate_trade_plan,
    fetch_live_book_ticker,
    fetch_live_btcusdt_book,
)


class TestCryptoShadowSymbol(unittest.TestCase):
    def test_normalize_and_paths(self):
        self.assertEqual(normalize_crypto_shadow_symbol(None), "BTCUSDT")
        self.assertEqual(normalize_crypto_shadow_symbol("ethusdt"), "ETHUSDT")
        self.assertEqual(default_output_dir_for_symbol("BTCUSDT"), DEFAULT_OUTPUT_DIR)
        eth = default_output_dir_for_symbol("ETHUSDT")
        self.assertTrue(str(eth).endswith("crypto_eth"))
        sol = default_output_dir_for_symbol("SOLUSDT")
        self.assertTrue(str(sol).endswith("crypto_sol"))
        with self.assertRaises(ValueError):
            normalize_crypto_shadow_symbol("DOGEUSDT")
        self.assertEqual(ALLOWED_CRYPTO_SHADOW_SYMBOLS, ("BTCUSDT", "ETHUSDT", "SOLUSDT"))

    def test_ws_urls(self):
        self.assertEqual(
            binance_ws_stream_url("ETHUSDT", "bookTicker"),
            "wss://stream.binance.com:9443/ws/ethusdt@bookTicker",
        )
        self.assertEqual(
            binance_ws_stream_url("BTCUSDT", "kline_15m"),
            "wss://stream.binance.com:9443/ws/btcusdt@kline_15m",
        )

    def test_ofx_journal_ws_url_follows_symbol(self):
        with tempfile.TemporaryDirectory() as td:
            j = OfxCryptoAggTradeJournal(Path(td), enabled=False, symbol="ETHUSDT")
            self.assertEqual(j.symbol, "ETHUSDT")
            self.assertEqual(
                j.ws_url, "wss://stream.binance.com:9443/ws/ethusdt@aggTrade"
            )

    def test_daemon_eth_isolates_books_and_forces_gate_off(self):
        with tempfile.TemporaryDirectory() as td:
            out = Path(td) / "crypto_eth"
            d = RP608ShadowDaemon(output_dir=out, symbol="ETHUSDT")
            try:
                self.assertEqual(d.symbol, "ETHUSDT")
                self.assertEqual(d.trade_plan_overlay.symbol, "ETHUSDT")
                self.assertEqual(d.trade_plan_overlay.book_id, "CRYPTO_PLAN_ETHUSDT")
                self.assertEqual(
                    d.trade_plan_overlay_baseline.book_id,
                    "CRYPTO_PLAN_BASELINE_PARALLEL_ETHUSDT",
                )
                self.assertEqual(d.trade_plan_overlay._pattern_entry_gate_mode_override, "0")
                self.assertEqual(
                    d.trade_plan_overlay_baseline._pattern_entry_gate_mode_override, "0"
                )
                self.assertEqual(d.ofx_aggtrade_journal.symbol, "ETHUSDT")
                self.assertEqual(
                    d.trade_plan_overlay.base_dir.resolve(),
                    (out / "trade_plan_overlay_v1").resolve(),
                )
            finally:
                d.ofx_aggtrade_journal.close()

    def test_daemon_btc_defaults_unchanged(self):
        with tempfile.TemporaryDirectory() as td:
            out = Path(td) / "crypto"
            d = RP608ShadowDaemon(output_dir=out, symbol="BTCUSDT")
            try:
                self.assertEqual(d.symbol, DEFAULT_CRYPTO_SHADOW_SYMBOL)
                self.assertEqual(d.trade_plan_overlay.book_id, "CRYPTO_PLAN")
                self.assertEqual(d.ofx_aggtrade_journal.symbol, "BTCUSDT")
            finally:
                d.ofx_aggtrade_journal.close()

    def test_plan_stamps_card_symbol(self):
        card = {
            "policy_b_action": "LONG",
            "reference_price": 3000.0,
            "atr_14": 30.0,
            "close_time_ms": 1_800_000_000_000,
            "decision_event_idx": 1,
            "bar_idx": 1,
            "symbol": "ETHUSDT",
            "execution_quote": {
                "best_bid_price": 2999.0,
                "best_ask_price": 3001.0,
                "exchange_event_time_ms": 1_800_000_000_000,
            },
        }
        plan = calculate_trade_plan(card, quote=card["execution_quote"])
        self.assertIsNotNone(plan)
        assert plan is not None
        self.assertEqual(plan["symbol"], "ETHUSDT")

    def test_fetch_alias(self):
        self.assertIs(fetch_live_btcusdt_book, fetch_live_btcusdt_book)
        # Signature smoke: alias callable
        self.assertTrue(callable(fetch_live_book_ticker))
        self.assertTrue(callable(fetch_live_btcusdt_book))


if __name__ == "__main__":
    unittest.main()
