#!/usr/bin/env python3
"""NSE Kite instrument-map cache / readiness seed (429 resilience)."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from scripts.run_rp608_nse_shadow_daemon import KiteDataFeedAdapter


class TestNseInstrumentMapCache(unittest.TestCase):
    def test_seed_from_readiness_and_persist(self):
        with tempfile.TemporaryDirectory() as td:
            td_path = Path(td)
            readiness = td_path / "kite_readiness_report.json"
            cache = td_path / "kite_nse_instrument_map.json"
            readiness.write_text(
                json.dumps(
                    {
                        "all_quotes": [
                            {
                                "symbol": "RELIANCE.NS",
                                "tradingsymbol": "RELIANCE",
                                "instrument_token": 738561,
                            },
                            {
                                "symbol": "TCS.NS",
                                "tradingsymbol": "TCS",
                                "instrument_token": 2953217,
                            },
                            {
                                "symbol": "WIPRO.NS",
                                "tradingsymbol": "WIPRO",
                                "instrument_token": 969473,
                            },
                        ]
                    }
                ),
                encoding="utf-8",
            )

            with mock.patch.object(
                KiteDataFeedAdapter, "_fetch_instruments_csv_raw_map", side_effect=RuntimeError("429")
            ):
                adapter = KiteDataFeedAdapter(
                    symbols=("RELIANCE.NS", "TCS.NS", "WIPRO.NS"),
                    credentials_file=td_path / "missing_creds.json",
                    instrument_cache_path=cache,
                    readiness_report_path=readiness,
                )
            self.assertEqual(len(adapter.instrument_map), 3)
            self.assertEqual(adapter._instrument_map_source, "readiness_report_seed")
            self.assertTrue(cache.exists())
            self.assertEqual(adapter.instrument_map["RELIANCE.NS"][1], "RELIANCE")

            with mock.patch.object(
                KiteDataFeedAdapter, "_fetch_instruments_csv_raw_map", side_effect=RuntimeError("429")
            ), mock.patch.object(
                KiteDataFeedAdapter, "_seed_instrument_map_from_readiness", return_value={}
            ):
                adapter2 = KiteDataFeedAdapter(
                    symbols=("RELIANCE.NS", "TCS.NS", "WIPRO.NS"),
                    credentials_file=td_path / "missing_creds.json",
                    instrument_cache_path=cache,
                    readiness_report_path=td_path / "no_readiness.json",
                )
            self.assertEqual(len(adapter2.instrument_map), 3)
            self.assertEqual(adapter2._instrument_map_source, "disk_cache")


if __name__ == "__main__":
    unittest.main()
