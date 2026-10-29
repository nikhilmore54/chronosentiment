"""
tests/test_rp608_shadow_daemon.py

Standard Library Unit Test Suite for RP-608 Dedicated Shadow Observer Daemon.
Zero Third-Party Test Dependencies.
Validates:
  1. Atomic Checkpoint Creation & Checksum Verification
  2. Fail-Closed Boundary on Checkpoint Tampering or Corruption
  3. Fail-Closed Boundary on Incompatible Schema / Window ID
  4. Resumption & Explicit Outage Accounting (Isolated vs Consecutive Breach)
  5. Opportunity Accounting Safeguard (Drops Excluded from Trade Denominator)
  6. Zero Order Authority Assertion on Environment Secrets
  7. Causal Feature & Factor Parity with Ground Truth Telemetry Fixtures
"""

import asyncio
import json
import os
import shutil
import tempfile
import time
import unittest
import sys
from pathlib import Path

WORKSPACE_DIR = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE_DIR))

from scripts.run_rp608_shadow_daemon import (
    DEFAULT_CHECKPOINT_PATH,
    DEFAULT_OUTPUT_DIR,
    Candle15m,
    CausalFeatureVector,
    DurableCheckpointManager,
    ExchangeClockFilter,
    OpportunityAccountingRegistry,
    RP608ShadowDaemon,
    UnexecutableDrop,
    compute_causal_features,
    compute_payload_checksum,
    evaluate_policy_rules,
    features_to_evidence,
    predicate_matches,
    resolve_output_dir,
    sha256_file,
)


class TestRP608ShadowDaemon(unittest.TestCase):
    def setUp(self):
        self.test_dir = tempfile.mkdtemp()
        self.temp_path = Path(self.test_dir)

    def tearDown(self):
        shutil.rmtree(self.test_dir, ignore_errors=True)

    def test_01_atomic_checkpoint_integrity(self):
        ckpt_path = self.temp_path / "test_checkpoint.json"
        mgr = DurableCheckpointManager(ckpt_path)

        state = {
            "schema_version": "1.0.0",
            "window_id": "RP-608-SHADOW-001B",
            "last_durable_bar_idx": 42,
            "last_heartbeat_utc_ms": 1792900000000,
            "payload": "valid_state_data",
        }

        mgr.save_checkpoint_atomic(state)
        self.assertTrue(ckpt_path.exists())
        self.assertFalse(ckpt_path.with_suffix(".tmp").exists())

        loaded = mgr.load_checkpoint()
        self.assertEqual(loaded["last_durable_bar_idx"], 42)
        self.assertIn("checksum", loaded)

        # Verify checksum matches
        expected_hash = compute_payload_checksum(loaded)
        self.assertEqual(loaded["checksum"], expected_hash)

    def test_02_fail_closed_on_checkpoint_tampering(self):
        ckpt_path = self.temp_path / "tampered_checkpoint.json"
        mgr = DurableCheckpointManager(ckpt_path)

        state = {
            "schema_version": "1.0.0",
            "window_id": "RP-608-SHADOW-001B",
            "last_durable_bar_idx": 10,
        }
        mgr.save_checkpoint_atomic(state)

        # Tamper with the file contents behind the manager's back
        with open(ckpt_path, "r", encoding="utf-8") as f:
            data = json.load(f)
        data["last_durable_bar_idx"] = 99  # Tampered field without updating checksum
        with open(ckpt_path, "w", encoding="utf-8") as f:
            json.dump(data, f)

        # Loading tampered checkpoint MUST raise ValueError
        with self.assertRaises(ValueError) as ctx:
            mgr.load_checkpoint()
        self.assertIn("FATAL: Checkpoint checksum mismatch", str(ctx.exception))

    def test_03_fail_closed_on_incompatible_schema(self):
        ckpt_path = self.temp_path / "invalid_schema.json"
        mgr = DurableCheckpointManager(ckpt_path)

        state = {
            "schema_version": "2.0.0",  # Incompatible version
            "window_id": "RP-608-SHADOW-001B",
        }
        mgr.save_checkpoint_atomic(state)

        with self.assertRaises(ValueError) as ctx:
            mgr.load_checkpoint()
        self.assertIn("FATAL: Incompatible checkpoint schema", str(ctx.exception))

    def test_04_resumption_downtime_accounting_and_idempotency(self):
        ckpt_path = self.temp_path / "resumption_checkpoint.json"
        daemon = RP608ShadowDaemon(checkpoint_path=ckpt_path, window_start_utc_ms=0)

        # 1. Initial run: Create clean initial checkpoint
        daemon.initialize_or_resume_state()
        self.assertEqual(daemon.state["last_durable_bar_idx"], 0)
        self.assertEqual(daemon.state["last_accounted_bar_idx"], 0)

        # 2. Simulate 35-minute power cut (2,100,000 ms)
        simulated_crash_time_ms = int(time.time() * 1000) - (35 * 60 * 1000)
        daemon.state["last_heartbeat_utc_ms"] = simulated_crash_time_ms
        daemon.state["last_durable_bar_idx"] = 100
        daemon.state["last_accounted_bar_idx"] = 100
        daemon.checkpoint_manager.save_checkpoint_atomic(daemon.state)

        # 3. First resumption across 35 minutes outage (2 missed intervals)
        daemon_r1 = RP608ShadowDaemon(checkpoint_path=ckpt_path, window_start_utc_ms=0)
        daemon_r1.initialize_or_resume_state()

        # Invariants after First Resumption:
        # last_durable_bar_idx remains 100 (genuinely processed candle count)
        self.assertEqual(daemon_r1.state["last_durable_bar_idx"], 100)
        # last_accounted_bar_idx advanced by 2 to 102
        self.assertEqual(daemon_r1.state["last_accounted_bar_idx"], 102)

        outages_r1 = daemon_r1.state.get("outage_history", [])
        self.assertEqual(len(outages_r1), 1)
        outage1 = outages_r1[0]
        self.assertEqual(outage1["missed_15m_intervals"], 2)
        self.assertGreaterEqual(outage1["duration_s"], 2100.0)

        # Quality Gates updated
        gates_r1 = daemon_r1.state["telemetry_gates"]
        self.assertEqual(gates_r1["total_missing_bars"], 2)
        self.assertGreaterEqual(gates_r1["consecutive_missing_bars"], 2)
        self.assertTrue(gates_r1["window_tainted_by_breach"])

        # Opportunity registry recorded exactly 2 drops (bars 101 and 102)
        self.assertEqual(daemon_r1.opportunity_registry.unexecutable_drops_count, 2)
        self.assertEqual(daemon_r1.opportunity_registry.total_signal_opportunities, 2)
        self.assertEqual(daemon_r1.opportunity_registry.executed_trades_count, 0)
        drop_bars_r1 = [d["bar_idx"] for d in daemon_r1.opportunity_registry.unexecutable_drops]
        self.assertEqual(drop_bars_r1, [101, 102])

        # 4. Immediate Second Resumption (simulate immediate restart without new candle)
        daemon_r2 = RP608ShadowDaemon(checkpoint_path=ckpt_path, window_start_utc_ms=0)
        daemon_r2.initialize_or_resume_state()

        # Invariants after Second Resumption (Idempotency Assertions):
        # Must NOT re-accrue the 2 missed intervals!
        self.assertEqual(daemon_r2.state["last_durable_bar_idx"], 100)
        self.assertEqual(daemon_r2.state["last_accounted_bar_idx"], 102)

        # Outage count and dropped opportunities must be identical to r1
        outages_r2 = daemon_r2.state.get("outage_history", [])
        self.assertEqual(len(outages_r2), 1)
        self.assertEqual(outages_r2[0]["outage_id"], outage1["outage_id"])

        self.assertEqual(daemon_r2.opportunity_registry.unexecutable_drops_count, 2)
        self.assertEqual(daemon_r2.opportunity_registry.total_signal_opportunities, 2)
        drop_bars_r2 = [d["bar_idx"] for d in daemon_r2.opportunity_registry.unexecutable_drops]
        self.assertEqual(drop_bars_r2, [101, 102])

    def test_05_opportunity_accounting_safeguard(self):
        registry = OpportunityAccountingRegistry()

        # Record 8 executed signals
        for _ in range(8):
            registry.record_executed()

        # Record 2 dropped signals (stale quote and execution quote timeout)
        registry.record_drop(UnexecutableDrop(
            opportunity_id=9,
            bar_idx=142,
            decision_time_ms=1792900000000,
            intended_action="ENTER_LONG",
            drop_reason="StaleQuoteExceeded",
            disposition="UNEXECUTABLE_STALE_DROP",
            details={"quote_age_ms": 1200},
        ))
        registry.record_drop(UnexecutableDrop(
            opportunity_id=10,
            bar_idx=198,
            decision_time_ms=1792950000000,
            intended_action="ENTER_LONG",
            drop_reason="ExecutionQuoteTimeout",
            disposition="UNEXECUTABLE_TIMEOUT_DROP",
            details={"timeout_ms": 5000},
        ))

        # Strict Invariant: Total = Executed + Dropped
        self.assertEqual(registry.total_signal_opportunities, 10)
        self.assertEqual(registry.executed_trades_count, 8)
        self.assertEqual(registry.unexecutable_drops_count, 2)
        self.assertEqual(registry.execution_drop_rate_pct, 20.0)

    def test_06_zero_order_authority_assertion(self):
        ckpt_path = self.temp_path / "auth_test.json"
        daemon = RP608ShadowDaemon(checkpoint_path=ckpt_path)

        os.environ["BINANCE_API_SECRET"] = "super_secret_forbidden_key"
        try:
            with self.assertRaises(PermissionError) as ctx:
                daemon.preflight_integrity_check()
            self.assertIn("FATAL: Forbidden API secret", str(ctx.exception))
        finally:
            del os.environ["BINANCE_API_SECRET"]

    def test_07_frozen_policy_parity_full_sweep(self):
        """Gate A: 100.0% Parity Sweep of Policy B & C against Rust Ground Truth Telemetry Fixtures."""
        policy_b_path = WORKSPACE_DIR / "datasets" / "crypto_15m" / "frozen_policy_b_champion.json"
        policy_c_path = WORKSPACE_DIR / "datasets" / "crypto_15m" / "frozen_policy_c_champion.json"
        fixture_b_path = WORKSPACE_DIR / "datasets" / "crypto_15m" / "policy_b_decision_telemetry_sample.json"
        fixture_c_path = WORKSPACE_DIR / "datasets" / "crypto_15m" / "policy_c_decision_telemetry_sample.json"

        with open(policy_b_path, "r", encoding="utf-8") as f:
            policy_b_genome = json.load(f)["champion_genome"]
        with open(policy_c_path, "r", encoding="utf-8") as f:
            policy_c_genome = json.load(f)["champion_genome"]
        with open(fixture_b_path, "r", encoding="utf-8") as f:
            records_b = json.load(f)
        with open(fixture_c_path, "r", encoding="utf-8") as f:
            records_c = json.load(f)

        self.assertEqual(len(records_b), 500, "Policy B fixture must contain exactly 500 records")
        self.assertEqual(len(records_c), 500, "Policy C fixture must contain exactly 500 records")

        action_map = {"Long": "LONG", "Short": "SHORT", "NoTrade": "NO_TRADE"}

        # 1. Sweep Policy B
        mismatches_b = []
        for r in records_b:
            bar_idx = r["bar_idx"]
            inputs = r["causal_inputs"]
            expected_action = action_map[r["proposed_action"]]

            feat = CausalFeatureVector(
                bar_idx=bar_idx,
                open_time_ms=r["timestamp_ms"] - (15 * 60 * 1000) + 1,
                close_time_ms=r["timestamp_ms"],
                reference_price=inputs["reference_price"],
                ret_15m_bps=inputs["ret_15m_bps"],
                atr_14=inputs["atr_14"],
                donchian_breakout_20=inputs["donchian_breakout"],
                price_range_pos_20=inputs["price_range_pos"],
                rolling_vwap_96=43000.0,
                dist_vwap_bps=inputs["dist_vwap_bps"],
                vol_burst_20=inputs["vol_burst"],
            )

            factors = features_to_evidence(feat, "BaseOnly")
            daemon_action = evaluate_policy_rules(policy_b_genome, factors)

            if daemon_action != expected_action:
                mismatches_b.append({
                    "bar_idx": bar_idx,
                    "expected": expected_action,
                    "actual": daemon_action,
                    "factors": factors,
                })

        self.assertEqual(len(mismatches_b), 0, f"Policy B Parity Mismatches: {mismatches_b}")

        # 2. Sweep Policy C
        mismatches_c = []
        for r in records_c:
            bar_idx = r["bar_idx"]
            inputs = r["causal_inputs"]
            expected_action = action_map[r["proposed_action"]]

            feat = CausalFeatureVector(
                bar_idx=bar_idx,
                open_time_ms=r["timestamp_ms"] - (15 * 60 * 1000) + 1,
                close_time_ms=r["timestamp_ms"],
                reference_price=inputs["reference_price"],
                ret_15m_bps=inputs["ret_15m_bps"],
                atr_14=inputs["atr_14"],
                donchian_breakout_20=inputs["donchian_breakout"],
                price_range_pos_20=inputs["price_range_pos"],
                rolling_vwap_96=43000.0,
                dist_vwap_bps=inputs["dist_vwap_bps"],
                vol_burst_20=inputs["vol_burst"],
            )

            factors = features_to_evidence(feat, "WithExhaustion")
            daemon_action = evaluate_policy_rules(policy_c_genome, factors)

            if daemon_action != expected_action:
                mismatches_c.append({
                    "bar_idx": bar_idx,
                    "expected": expected_action,
                    "actual": daemon_action,
                    "factors": factors,
                })

        self.assertEqual(len(mismatches_c), 0, f"Policy C Parity Mismatches: {mismatches_c}")

    def test_08_default_paths_configuration(self):
        """Validates that default output paths point strictly to production directories."""
        self.assertEqual(
            DEFAULT_OUTPUT_DIR,
            WORKSPACE_DIR / "live_capture" / "shadow_validation" / "crypto",
        )
        daemon = RP608ShadowDaemon()
        self.assertEqual(daemon.output_dir, DEFAULT_OUTPUT_DIR)
        self.assertEqual(daemon.checkpoint_path, DEFAULT_OUTPUT_DIR / "rp608_checkpoint.json")
        self.assertEqual(daemon.telemetry_log_path, DEFAULT_OUTPUT_DIR / "rp608_shadow_telemetry.jsonl")
        self.assertEqual(daemon.decision_cards_path, DEFAULT_OUTPUT_DIR / "rp608_decision_cards.jsonl")
        self.assertEqual(daemon.trade_journal_path, DEFAULT_OUTPUT_DIR / "rp608_trade_journal.jsonl")
        self.assertEqual(daemon.opportunity_log_path, DEFAULT_OUTPUT_DIR / "rp608_opportunity_registry.jsonl")

    def test_09_output_dir_redirection_and_safety_rejection(self):
        """Validates that --output-dir cleanly redirects every artifact and rejects unsafe configurations."""
        # 1. Gate C canonical test path
        gate_c_rel = "live_capture/shadow_validation/crypto/gate_c_test"
        resolved_gate_c = resolve_output_dir(gate_c_rel)
        self.assertEqual(resolved_gate_c, (WORKSPACE_DIR / gate_c_rel).resolve())

        # 2. Complete redirection to temp directory
        test_dir = self.temp_path / "custom_isolation"
        daemon = RP608ShadowDaemon(output_dir=test_dir)
        self.assertEqual(daemon.output_dir, test_dir.resolve())
        self.assertEqual(daemon.checkpoint_path, test_dir.resolve() / "rp608_checkpoint.json")
        self.assertEqual(daemon.telemetry_log_path, test_dir.resolve() / "rp608_shadow_telemetry.jsonl")
        self.assertEqual(daemon.decision_cards_path, test_dir.resolve() / "rp608_decision_cards.jsonl")
        self.assertEqual(daemon.trade_journal_path, test_dir.resolve() / "rp608_trade_journal.jsonl")
        self.assertEqual(daemon.opportunity_log_path, test_dir.resolve() / "rp608_opportunity_registry.jsonl")

        # 3. Unsafe path rejections
        with self.assertRaises(ValueError):
            resolve_output_dir("")
        with self.assertRaises(ValueError):
            resolve_output_dir("   ")
        with self.assertRaises(ValueError):
            resolve_output_dir("/")
        with self.assertRaises(ValueError):
            resolve_output_dir(WORKSPACE_DIR)

        # File instead of directory
        dummy_file = self.temp_path / "dummy_file.txt"
        dummy_file.write_text("hello")
        with self.assertRaises(ValueError):
            resolve_output_dir(dummy_file)

        # Mismatched checkpoint path outside output_dir
        with self.assertRaises(ValueError):
            RP608ShadowDaemon(
                output_dir=test_dir,
                checkpoint_path=self.temp_path / "other_dir" / "rp608_checkpoint.json",
            )

    def test_10_no_production_mutation_in_isolated_mode(self):
        """Verifies that running in isolated mode leaves production files strictly untouched."""
        prod_checkpoint = DEFAULT_OUTPUT_DIR / "rp608_checkpoint.json"
        prod_cards = DEFAULT_OUTPUT_DIR / "rp608_decision_cards.jsonl"

        prod_files_before = {}
        for p in [prod_checkpoint, prod_cards]:
            if p.exists():
                prod_files_before[p] = sha256_file(p)
            else:
                prod_files_before[p] = None

        # Execute isolated daemon lifecycle
        iso_dir = self.temp_path / "isolated_execution"
        daemon = RP608ShadowDaemon(output_dir=iso_dir)
        daemon.initialize_or_resume_state()
        daemon.update_heartbeat()

        # Assert isolated artifacts were created
        self.assertTrue((iso_dir / "rp608_checkpoint.json").exists())
        self.assertTrue((iso_dir / "rp608_shadow_telemetry.jsonl").exists())

        # Assert production artifacts were 100% untouched
        for p, before_hash in prod_files_before.items():
            if before_hash is None:
                self.assertFalse(p.exists(), f"Production file {p} was created by isolated test!")
            else:
                self.assertTrue(p.exists())
                after_hash = sha256_file(p)
                self.assertEqual(
                    before_hash,
                    after_hash,
                    f"Production file {p} was modified during isolated test!",
                )

    def test_11_corrupted_test_checkpoint_fails_closed_production_unaffected(self):
        """Verifies that a corrupted test checkpoint triggers fail-closed boundary without touching production."""
        iso_dir = self.temp_path / "corrupted_isolation"
        daemon = RP608ShadowDaemon(output_dir=iso_dir)
        daemon.initialize_or_resume_state()

        # Corrupt test checkpoint
        test_ckpt = iso_dir / "rp608_checkpoint.json"
        with open(test_ckpt, "r", encoding="utf-8") as f:
            data = json.load(f)
        data["last_durable_bar_idx"] = 9999
        with open(test_ckpt, "w", encoding="utf-8") as f:
            json.dump(data, f)

        # Second instance targeting corrupted test directory must fail closed
        daemon_corrupt = RP608ShadowDaemon(output_dir=iso_dir)
        with self.assertRaises(ValueError) as ctx:
            daemon_corrupt.initialize_or_resume_state()
        self.assertIn("FATAL: Checkpoint checksum mismatch", str(ctx.exception))

        # Check production checkpoint was not touched
        prod_checkpoint = DEFAULT_OUTPUT_DIR / "rp608_checkpoint.json"
        if prod_checkpoint.exists():
            mgr = DurableCheckpointManager(prod_checkpoint)
            prod_data = mgr.load_checkpoint()
            self.assertIsNotNone(prod_data)

    def test_12_single_candle_test_exits_and_persists_test_result(self):
        """Verifies that a single-candle evaluation persists its card, advances state, and stops."""
        iso_dir = self.temp_path / "single_candle_run"
        daemon = RP608ShadowDaemon(output_dir=iso_dir, single_candle_test=True)
        daemon.preflight_integrity_check()
        daemon.initialize_or_resume_state()
        daemon.running = True

        # Mock a warm history of 120 candles so causal feature extraction succeeds
        base_ms = 1792900000000
        for i in range(120):
            c = Candle15m(
                open_time_ms=base_ms + (i * 900_000),
                close_time_ms=base_ms + ((i + 1) * 900_000) - 1,
                open=65000.0 + (i * 2.0),
                high=65010.0 + (i * 2.0),
                low=64990.0 + (i * 2.0),
                close=65005.0 + (i * 2.0),
                volume=100.0,
            )
            daemon.recent_candles.append(c)

        # Mock a fresh top of book quote
        eval_candle = Candle15m(
            open_time_ms=base_ms + (120 * 900_000),
            close_time_ms=base_ms + ((121) * 900_000) - 1,
            open=65240.0,
            high=65250.0,
            low=65235.0,
            close=65245.0,
            volume=150.0,
        )

        async def mock_post_close_ticker():
            await asyncio.sleep(0.04)
            now_wall = int(time.time() * 1000)
            daemon.latest_book_ticker = {
                "exchange_event_time_ms": now_wall,
                "local_wallclock_utc_ms": now_wall,
                "local_receipt_mono_ns": time.monotonic_ns(),
                "sequence_update_id": 1001,
                "best_bid_price": 65245.0,
                "best_bid_qty": 1.5,
                "best_ask_price": 65246.0,
                "best_ask_qty": 2.0,
                "spread_bps": 0.15,
            }

        async def run_concurrent():
            t = asyncio.create_task(mock_post_close_ticker())
            await daemon.evaluate_completed_candle(eval_candle)
            await t

        asyncio.run(run_concurrent())


        # Check durable writes in isolated directory
        self.assertTrue(daemon.decision_cards_path.exists())
        with open(daemon.decision_cards_path, "r", encoding="utf-8") as f:
            lines = f.readlines()
        self.assertEqual(len(lines), 1)
        card = json.loads(lines[0])
        self.assertEqual(card["bar_idx"], 1)
        self.assertTrue(card["valid_execution_quote"])
        self.assertEqual(card["reference_price"], 65245.0)

        # Check checkpoint advanced
        reloaded = daemon.checkpoint_manager.load_checkpoint()
        self.assertEqual(reloaded["last_durable_bar_idx"], 1)
        self.assertEqual(reloaded["last_accounted_bar_idx"], 1)

        # Check telemetry logged
        self.assertTrue(daemon.telemetry_log_path.exists())

    def test_13_quote_freshness_boundary_and_staleness_rejection(self):
        """Verifies that quotes exceeding 500ms freshness boundary or with inverted book are rejected."""
        iso_dir = self.temp_path / "freshness_test_run"
        daemon = RP608ShadowDaemon(output_dir=iso_dir)
        daemon.preflight_integrity_check()
        daemon.initialize_or_resume_state()

        base_ms = 1792900000000
        for i in range(120):
            daemon.recent_candles.append(Candle15m(
                open_time_ms=base_ms + (i * 900_000),
                close_time_ms=base_ms + ((i + 1) * 900_000) - 1,
                open=65000.0, high=65010.0, low=64990.0, close=65000.0, volume=100.0,
            ))

        # 1. Stale quote test: Exchange event timestamp is 800 ms behind arrival wallclock
        now_wall = int(time.time() * 1000)
        daemon.latest_book_ticker = {
            "exchange_event_time_ms": now_wall - 800,  # 800 ms old (> 500 ms threshold)
            "local_wallclock_utc_ms": now_wall,
            "local_receipt_mono_ns": time.monotonic_ns() + 100_000_000,
            "sequence_update_id": 2001,
            "best_bid_price": 65000.0,
            "best_bid_qty": 1.0,
            "best_ask_price": 65001.0,
            "best_ask_qty": 1.0,
            "spread_bps": 0.15,
        }

        candle_stale = Candle15m(
            open_time_ms=base_ms + (120 * 900_000),
            close_time_ms=base_ms + (121 * 900_000) - 1,
            open=65000.0, high=65010.0, low=64990.0, close=65000.0, volume=100.0,
        )

        async def eval_stale():
            await daemon.evaluate_completed_candle(candle_stale)

        asyncio.run(eval_stale())

        with open(daemon.decision_cards_path, "r", encoding="utf-8") as f:
            cards = [json.loads(line) for line in f]
        self.assertEqual(len(cards), 1)
        self.assertFalse(cards[0]["valid_execution_quote"])
        self.assertEqual(cards[0]["drop_reason"], "StaleQuoteExceeded")
        self.assertGreater(cards[0]["quote_age_ms"], 500)
        self.assertEqual(daemon.state["telemetry_gates"]["stale_quote_count"], 1)

        # 2. Inverted book test: best_bid >= best_ask
        now_wall = int(time.time() * 1000)
        daemon.latest_book_ticker = {
            "exchange_event_time_ms": now_wall,
            "local_wallclock_utc_ms": now_wall,
            "local_receipt_mono_ns": time.monotonic_ns() + 100_000_000,
            "sequence_update_id": 2002,
            "best_bid_price": 65005.0,  # Bid higher than ask!
            "best_ask_price": 65001.0,
            "best_bid_qty": 1.0,
            "best_ask_qty": 1.0,
            "spread_bps": -0.6,
        }

        candle_inv = Candle15m(
            open_time_ms=base_ms + (121 * 900_000),
            close_time_ms=base_ms + (122 * 900_000) - 1,
            open=65000.0, high=65010.0, low=64990.0, close=65000.0, volume=100.0,
        )

        async def eval_inv():
            await daemon.evaluate_completed_candle(candle_inv)

        asyncio.run(eval_inv())

        with open(daemon.decision_cards_path, "r", encoding="utf-8") as f:
            cards = [json.loads(line) for line in f]
        self.assertEqual(len(cards), 2)
        self.assertFalse(cards[1]["valid_execution_quote"])
        self.assertEqual(cards[1]["drop_reason"], "InvertedBookError")

    def test_14_graceful_shutdown_lifecycle(self):
        """Verifies that shutdown prompt-wakes the loop, awaits tasks, and flushes final checkpoint."""
        iso_dir = self.temp_path / "shutdown_lifecycle_run"
        daemon = RP608ShadowDaemon(output_dir=iso_dir)

        # Mock clock filter so test doesn't depend on live network
        daemon.clock_filter.sample_offset = lambda samples=5: (0, 45)
        daemon.clock_filter.is_synchronized = lambda: True

        # Mock feed tasks to simulate active streaming
        feed_tasks_cancelled = {"ticker": False, "klines": False}

        async def mock_feed(name: str):
            try:
                while True:
                    await asyncio.sleep(0.05)
            except asyncio.CancelledError:
                feed_tasks_cancelled[name] = True
                raise

        daemon.ingest_book_ticker = lambda: mock_feed("ticker")
        daemon.ingest_klines = lambda: mock_feed("klines")

        async def run_lifecycle():
            task = asyncio.create_task(daemon.run())
            # Allow daemon to initialize and enter wait loop
            await asyncio.sleep(0.05)
            self.assertTrue(daemon.running)

            # Trigger graceful shutdown (mirrors SIGTERM / SIGINT)
            start_mono = time.monotonic()
            daemon.request_shutdown()

            # Wait for daemon to finalize
            await asyncio.wait_for(task, timeout=2.0)
            elapsed = time.monotonic() - start_mono

            # Prompt wake must complete in < 0.5s (not waiting full 5.0s tick)
            self.assertLess(elapsed, 0.5, "Shutdown must wake main loop promptly")

        asyncio.run(run_lifecycle())

        self.assertFalse(daemon.running)
        self.assertTrue(feed_tasks_cancelled["ticker"])
        self.assertTrue(feed_tasks_cancelled["klines"])

        # Verify durable final checkpoint was persisted
        self.assertTrue(daemon.checkpoint_path.exists())
        mgr = DurableCheckpointManager(daemon.checkpoint_path)
        final_state = mgr.load_checkpoint()
        self.assertIsNotNone(final_state)
        self.assertEqual(final_state["schema_version"], "1.0.0")

        # Verify telemetry log contains the shutdown heartbeat
        self.assertTrue(daemon.telemetry_log_path.exists())

    def test_15_activation_boundary_pre_start_exclusion(self):
        """Verifies that pre-activation buffer downtime is excluded while post-start gaps are accounted."""
        ckpt_path = self.temp_path / "boundary_checkpoint.json"
        now_ms = int(time.time() * 1000)

        # 1. Pre-start simulation: Window start is 2 hours in the future
        future_window_start = now_ms + (2 * 3600 * 1000)
        daemon_pre = RP608ShadowDaemon(
            checkpoint_path=ckpt_path,
            window_start_utc_ms=future_window_start,
        )
        daemon_pre.initialize_or_resume_state()

        # Simulate 1 hour elapsed between heartbeats before window start
        daemon_pre.state["last_heartbeat_utc_ms"] = now_ms - (3600 * 1000)
        daemon_pre.checkpoint_manager.save_checkpoint_atomic(daemon_pre.state)

        # Resuming while still before window start must NOT accrue missing intervals
        daemon_resume_pre = RP608ShadowDaemon(
            checkpoint_path=ckpt_path,
            window_start_utc_ms=future_window_start,
        )
        daemon_resume_pre.initialize_or_resume_state()
        self.assertEqual(daemon_resume_pre.state.get("outage_history", []), [])
        self.assertEqual(daemon_resume_pre.state["telemetry_gates"]["total_missing_bars"], 0)
        self.assertEqual(daemon_resume_pre.opportunity_registry.unexecutable_drops_count, 0)

        # 2. Post-start gap simulation:
        # Window start was 45 minutes ago. Last heartbeat was 2 hours ago (predated window start).
        past_window_start = now_ms - (45 * 60 * 1000)
        pre_start_heartbeat = now_ms - (120 * 60 * 1000)

        ckpt_post_path = self.temp_path / "boundary_post_checkpoint.json"
        daemon_post = RP608ShadowDaemon(
            checkpoint_path=ckpt_post_path,
            window_start_utc_ms=past_window_start,
        )
        daemon_post.initialize_or_resume_state()
        daemon_post.state["last_heartbeat_utc_ms"] = pre_start_heartbeat
        daemon_post.checkpoint_manager.save_checkpoint_atomic(daemon_post.state)

        # Resumption after window start must calculate downtime strictly from past_window_start:
        # 45 minutes / 15 minutes = exactly 3 missed intervals (not 8 intervals!)
        daemon_resume_post = RP608ShadowDaemon(
            checkpoint_path=ckpt_post_path,
            window_start_utc_ms=past_window_start,
        )
        daemon_resume_post.initialize_or_resume_state()

        outages = daemon_resume_post.state.get("outage_history", [])
        self.assertEqual(len(outages), 1)
        self.assertEqual(outages[0]["missed_15m_intervals"], 3)
        self.assertEqual(outages[0]["downtime_start_ms"], past_window_start)
        self.assertEqual(daemon_resume_post.state["telemetry_gates"]["total_missing_bars"], 3)
        self.assertEqual(daemon_resume_post.opportunity_registry.unexecutable_drops_count, 3)
        self.assertEqual(daemon_resume_post.state["last_accounted_bar_idx"], 3)
        self.assertEqual(daemon_resume_post.state["last_durable_bar_idx"], 0)


if __name__ == "__main__":
    unittest.main()




