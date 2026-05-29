"""
tests/test_candle_substrate.py
==============================
Pytest suite for scripts/candle_substrate.py.

Covers the current API (symbol_path / write_symbol_candles / read_symbol_candles /
df_to_records / build_timeline_fingerprint / frozen_batch_dir /
download_ticker_with_stderr / incremental_update_cohort) and all four bug-fixes
applied in the governance-hardening session:

  Fix 1 — read_symbol_candles() deduplicates on read (keep='first')
  Fix 2 — download_ticker_with_stderr() normalises tz to UTC, flattens MultiIndex,
           materialises flat structure via df.copy()
  Fix 3 — _update_one() normalises df_new tz before pd.concat
  Fix 4 — incremental_update_cohort() defaults max_workers=1 (no race condition)
"""

from __future__ import annotations

import gzip
import json
import sys
from pathlib import Path
from unittest.mock import MagicMock, patch

import pandas as pd
import pytest

# ── path bootstrap ────────────────────────────────────────────────────────────
sys.path.insert(0, str(Path(__file__).parent.parent / "scripts"))

from candle_substrate import (
    build_timeline_fingerprint,
    compute_substrate_hash,
    df_to_records,
    download_ticker_with_stderr,
    freeze_cohort,
    frozen_batch_dir,
    incremental_update_cohort,
    read_symbol_candles,
    symbol_path,
    write_symbol_candles,
)


# ── fixtures ──────────────────────────────────────────────────────────────────

@pytest.fixture
def sample_df():
    """Small UTC-aware OHLCV DataFrame — the canonical test input."""
    idx = pd.to_datetime(
        ["2024-01-01 00:00", "2024-01-01 00:05", "2024-01-01 00:10"], utc=True
    )
    return pd.DataFrame(
        {
            "Open":   [100.0, 101.0, 102.0],
            "High":   [105.0, 106.0, 107.0],
            "Low":    [99.0,  100.0, 101.0],
            "Close":  [103.0, 104.0, 105.0],
            "Volume": [1000.0, 1100.0, 1200.0],
        },
        index=idx,
    )


@pytest.fixture
def tmp_batch_dir(tmp_path):
    """Temporary directory standing in for a batch_dir."""
    d = tmp_path / "batch_test"
    d.mkdir()
    return d


@pytest.fixture
def tmp_sym_path(tmp_batch_dir):
    """Resolved symbol path inside tmp_batch_dir for BTC-USD."""
    return symbol_path(tmp_batch_dir, "BTC-USD")


# ── TestFrozenBatchDir ────────────────────────────────────────────────────────

class TestFrozenBatchDir:
    def test_zero_padded_three_digits(self, tmp_path):
        root = tmp_path / "candles"
        p = frozen_batch_dir(3, root)
        assert p.name == "batch_003"

    def test_large_batch_id(self, tmp_path):
        root = tmp_path / "candles"
        p = frozen_batch_dir(42, root)
        assert p.name == "batch_042"

    def test_returns_path_under_root(self, tmp_path):
        root = tmp_path / "candles"
        p = frozen_batch_dir(1, root)
        assert p.parent == root


# ── TestSymbolPath ────────────────────────────────────────────────────────────

class TestSymbolPath:
    def test_slash_replaced(self, tmp_batch_dir):
        p = symbol_path(tmp_batch_dir, "BTC/USD")
        assert "BTC_USD" in p.name

    def test_dash_preserved(self, tmp_batch_dir):
        """Dashes in symbol names are preserved (only / is replaced)."""
        p = symbol_path(tmp_batch_dir, "BTC-USD")
        assert "BTC-USD" in p.name

    def test_extension_is_jsonl_gz(self, tmp_batch_dir):
        p = symbol_path(tmp_batch_dir, "ETH-USD")
        assert p.suffix == ".gz"
        assert p.stem.endswith(".jsonl")

    def test_under_symbols_subdir(self, tmp_batch_dir):
        p = symbol_path(tmp_batch_dir, "SOL-USD")
        assert p.parent.name == "symbols"


# ── TestWriteReadSymbolCandles ────────────────────────────────────────────────

class TestWriteReadSymbolCandles:
    def test_roundtrip_shape(self, tmp_sym_path, sample_df):
        recs = df_to_records(sample_df)
        write_symbol_candles(tmp_sym_path, recs)
        result = read_symbol_candles(tmp_sym_path)
        assert len(result) == 3

    def test_roundtrip_columns(self, tmp_sym_path, sample_df):
        recs = df_to_records(sample_df)
        write_symbol_candles(tmp_sym_path, recs)
        result = read_symbol_candles(tmp_sym_path)
        assert list(result.columns) == ["Open", "High", "Low", "Close", "Volume"]

    def test_index_is_utc(self, tmp_sym_path, sample_df):
        recs = df_to_records(sample_df)
        write_symbol_candles(tmp_sym_path, recs)
        result = read_symbol_candles(tmp_sym_path)
        assert result.index.tz is not None
        assert str(result.index.tz) == "UTC"

    def test_missing_file_returns_empty(self, tmp_batch_dir):
        p = symbol_path(tmp_batch_dir, "MISSING-SYM")
        result = read_symbol_candles(p)
        assert result.empty

    def test_creates_parent_dirs(self, tmp_path, sample_df):
        deep = tmp_path / "a" / "b" / "symbols" / "BTC-USD.jsonl.gz"
        recs = df_to_records(sample_df)
        write_symbol_candles(deep, recs)
        assert deep.exists()

    def test_records_sorted_by_ts(self, tmp_sym_path):
        """write_symbol_candles must emit records in ascending ts order."""
        idx = pd.to_datetime(
            ["2024-01-01 00:10", "2024-01-01 00:00", "2024-01-01 00:05"], utc=True
        )
        df = pd.DataFrame(
            {"Open": [3.0, 1.0, 2.0], "High": [3.0, 1.0, 2.0],
             "Low": [3.0, 1.0, 2.0], "Close": [3.0, 1.0, 2.0], "Volume": [30.0, 10.0, 20.0]},
            index=idx,
        )
        recs = df_to_records(df)
        write_symbol_candles(tmp_sym_path, recs)
        result = read_symbol_candles(tmp_sym_path)
        ts_vals = result.index.tolist()
        assert ts_vals == sorted(ts_vals)


# ── TestDedupOnRead (Fix 1) ───────────────────────────────────────────────────

class TestDedupOnRead:
    """Fix 1: read_symbol_candles() must deduplicate on read (keep='first')."""

    def test_duplicate_rows_removed(self, tmp_sym_path):
        idx = pd.to_datetime(
            ["2024-01-01 00:00", "2024-01-01 00:00", "2024-01-01 00:05"], utc=True
        )
        df = pd.DataFrame(
            {"Open": [1.0, 2.0, 3.0], "High": [1.0, 2.0, 3.0],
             "Low":  [1.0, 2.0, 3.0], "Close": [1.0, 2.0, 3.0],
             "Volume": [10.0, 20.0, 30.0]},
            index=idx,
        )
        # Write raw duplicate records bypassing df_to_records dedup
        records = [
            {"ts": int(pd.Timestamp(ts).timestamp()),
             "open": float(row["Open"]), "high": float(row["High"]),
             "low": float(row["Low"]), "close": float(row["Close"]),
             "volume": float(row["Volume"])}
            for ts, row in df.iterrows()
        ]
        tmp_sym_path.parent.mkdir(parents=True, exist_ok=True)
        with gzip.open(tmp_sym_path, "wt", encoding="utf-8") as fh:
            for rec in records:
                fh.write(json.dumps(rec) + "\n")

        result = read_symbol_candles(tmp_sym_path)
        assert len(result) == 2, f"Expected 2 rows after dedup, got {len(result)}"
        # keep='first' → first duplicate (Open=1.0) is retained
        assert result.iloc[0]["Open"] == 1.0

    def test_index_unique_after_read(self, tmp_sym_path, sample_df):
        recs = df_to_records(sample_df)
        write_symbol_candles(tmp_sym_path, recs)
        result = read_symbol_candles(tmp_sym_path)
        assert result.index.is_unique


# ── TestDfToRecords ───────────────────────────────────────────────────────────

class TestDfToRecords:
    def test_ts_is_unix_int(self, sample_df):
        recs = df_to_records(sample_df)
        for r in recs:
            assert isinstance(r["ts"], int)

    def test_ohlcv_keys_present(self, sample_df):
        recs = df_to_records(sample_df)
        for r in recs:
            assert set(r.keys()) == {"ts", "open", "high", "low", "close", "volume"}

    def test_sorted_ascending(self, sample_df):
        recs = df_to_records(sample_df)
        ts_list = [r["ts"] for r in recs]
        assert ts_list == sorted(ts_list)

    def test_values_are_float(self, sample_df):
        recs = df_to_records(sample_df)
        for r in recs:
            for k in ("open", "high", "low", "close", "volume"):
                assert isinstance(r[k], float)

    def test_empty_df_returns_empty_list(self):
        result = df_to_records(pd.DataFrame())
        assert result == []


# ── TestBuildTimelineFingerprint ──────────────────────────────────────────────

class TestBuildTimelineFingerprint:
    def test_deterministic(self):
        ts = [1000, 2000, 3000]
        assert build_timeline_fingerprint(ts) == build_timeline_fingerprint(ts)

    def test_order_independent(self):
        ts = [3000, 1000, 2000]
        ts_sorted = [1000, 2000, 3000]
        assert build_timeline_fingerprint(ts) == build_timeline_fingerprint(ts_sorted)

    def test_different_data_different_fingerprint(self):
        assert build_timeline_fingerprint([1000, 2000]) != build_timeline_fingerprint([1000, 3000])

    def test_returns_16_hex_chars(self):
        fp = build_timeline_fingerprint([1000, 2000, 3000])
        assert len(fp) == 16
        assert all(c in "0123456789abcdef" for c in fp)

    def test_empty_list(self):
        fp = build_timeline_fingerprint([])
        assert len(fp) == 16


# ── TestComputeSubstrateHash ──────────────────────────────────────────────────

class TestComputeSubstrateHash:
    def test_returns_16_hex_chars(self, tmp_batch_dir, sample_df):
        p = symbol_path(tmp_batch_dir, "BTC-USD")
        write_symbol_candles(p, df_to_records(sample_df))
        h = compute_substrate_hash(tmp_batch_dir, ["BTC-USD"])
        assert len(h) == 16
        assert all(c in "0123456789abcdef" for c in h)

    def test_missing_symbol_ignored(self, tmp_batch_dir):
        h = compute_substrate_hash(tmp_batch_dir, ["MISSING-SYM"])
        assert len(h) == 16

    def test_deterministic(self, tmp_batch_dir, sample_df):
        p = symbol_path(tmp_batch_dir, "BTC-USD")
        write_symbol_candles(p, df_to_records(sample_df))
        h1 = compute_substrate_hash(tmp_batch_dir, ["BTC-USD"])
        h2 = compute_substrate_hash(tmp_batch_dir, ["BTC-USD"])
        assert h1 == h2


# ── TestDownloadTickerWithStderr (Fix 2) ──────────────────────────────────────

class TestDownloadTickerWithStderr:
    """Fix 2: download_ticker_with_stderr() must flatten MultiIndex, copy(),
    and normalise index to UTC."""

    def _mock_ticker(self, df):
        mock = MagicMock()
        with patch("candle_substrate.yf.download", return_value=df):
            result, stderr = download_ticker_with_stderr("BTC-USD")
        return result, stderr

    def test_multiindex_columns_flattened(self):
        idx = pd.to_datetime(["2024-01-01 00:00", "2024-01-01 00:05"], utc=True)
        multi_cols = pd.MultiIndex.from_tuples(
            [("Open", "BTC-USD"), ("High", "BTC-USD"), ("Low", "BTC-USD"),
             ("Close", "BTC-USD"), ("Volume", "BTC-USD")]
        )
        mock_df = pd.DataFrame(
            [[100.0, 105.0, 99.0, 103.0, 1000.0],
             [101.0, 106.0, 100.0, 104.0, 1100.0]],
            index=idx, columns=multi_cols,
        )
        result, _ = self._mock_ticker(mock_df)
        assert not isinstance(result.columns, pd.MultiIndex)
        assert "Open" in result.columns

    def test_tz_naive_index_localized_to_utc(self):
        idx = pd.to_datetime(["2024-01-01 00:00", "2024-01-01 00:05"])  # naive
        mock_df = pd.DataFrame(
            {"Open": [100.0, 101.0], "High": [105.0, 106.0],
             "Low": [99.0, 100.0], "Close": [103.0, 104.0], "Volume": [1000.0, 1100.0]},
            index=idx,
        )
        result, _ = self._mock_ticker(mock_df)
        assert result.index.tz is not None
        assert str(result.index.tz) == "UTC"

    def test_tz_aware_non_utc_converted_to_utc(self):
        idx = pd.to_datetime(["2024-01-01 00:00", "2024-01-01 00:05"]).tz_localize("US/Eastern")
        mock_df = pd.DataFrame(
            {"Open": [100.0, 101.0], "High": [105.0, 106.0],
             "Low": [99.0, 100.0], "Close": [103.0, 104.0], "Volume": [1000.0, 1100.0]},
            index=idx,
        )
        result, _ = self._mock_ticker(mock_df)
        assert str(result.index.tz) == "UTC"

    def test_empty_df_returned_as_is(self):
        result, _ = self._mock_ticker(pd.DataFrame())
        assert result.empty

    def test_index_unique_after_download(self):
        """Duplicate timestamps in yfinance output must be deduplicated."""
        idx = pd.to_datetime(
            ["2024-01-01 00:00", "2024-01-01 00:00", "2024-01-01 00:05"], utc=True
        )
        mock_df = pd.DataFrame(
            {"Open": [1.0, 2.0, 3.0], "High": [1.0, 2.0, 3.0],
             "Low": [1.0, 2.0, 3.0], "Close": [1.0, 2.0, 3.0], "Volume": [10.0, 20.0, 30.0]},
            index=idx,
        )
        result, _ = self._mock_ticker(mock_df)
        assert result.index.is_unique

    def test_returns_tuple_df_and_str(self):
        idx = pd.to_datetime(["2024-01-01 00:00"], utc=True)
        mock_df = pd.DataFrame(
            {"Open": [100.0], "High": [105.0], "Low": [99.0],
             "Close": [103.0], "Volume": [1000.0]},
            index=idx,
        )
        result = self._mock_ticker(mock_df)
        assert isinstance(result, tuple)
        assert len(result) == 2
        assert isinstance(result[1], str)


# ── TestIncrementalUpdateCohort (Fix 3 + Fix 4) ───────────────────────────────

class TestIncrementalUpdateCohort:
    """Fix 3: tz-normalization before concat.
    Fix 4: max_workers defaults to 1."""

    def _make_cohort_file(self, tmp_path, symbols):
        cohort_dir = tmp_path / "cohorts"
        cohort_dir.mkdir(exist_ok=True)
        f = cohort_dir / "test_cohort.txt"
        f.write_text("\n".join(symbols))
        return f

    def _seed_substrate(self, tmp_path, batch_id, symbols, sample_df):
        """Create the symbols/ directory so incremental_update_cohort can run."""
        from candle_substrate import frozen_batch_dir, symbol_path, write_symbol_candles, df_to_records
        batch_dir = frozen_batch_dir(batch_id, tmp_path / "state_archive" / "candles")
        sym_dir = batch_dir / "symbols"
        sym_dir.mkdir(parents=True, exist_ok=True)
        for sym in symbols:
            p = symbol_path(batch_dir, sym)
            write_symbol_candles(p, df_to_records(sample_df))
        return batch_dir

    def _mock_download(self, symbol, interval="5m", period="5d"):
        idx = pd.to_datetime(["2024-01-01 00:00", "2024-01-01 00:05"], utc=True)
        df = pd.DataFrame(
            {"Open": [100.0, 101.0], "High": [105.0, 106.0],
             "Low": [99.0, 100.0], "Close": [103.0, 104.0], "Volume": [1000.0, 1100.0]},
            index=idx,
        )
        return df, ""

    def test_requires_existing_substrate(self, tmp_path):
        cohort_file = self._make_cohort_file(tmp_path, ["BTC-USD"])
        with pytest.raises(FileNotFoundError, match="Substrate does not exist"):
            incremental_update_cohort(
                cohort_file=cohort_file,
                batch_id=99,
                root=tmp_path / "state_archive" / "candles",
            )

    def test_serial_execution_produces_valid_result(self, tmp_path, sample_df):
        symbols = ["BTC-USD", "ETH-USD"]
        cohort_file = self._make_cohort_file(tmp_path, symbols)
        self._seed_substrate(tmp_path, 1, symbols, sample_df)

        with patch("candle_substrate.download_ticker_with_stderr",
                   side_effect=self._mock_download):
            manifest_path, sym_ts, stats = incremental_update_cohort(
                cohort_file=cohort_file,
                batch_id=1,
                root=tmp_path / "state_archive" / "candles",
                max_workers=1,
            )

        assert manifest_path.exists()
        assert "BTC-USD" in sym_ts
        assert "ETH-USD" in sym_ts

    def test_manifest_written_with_fingerprint(self, tmp_path, sample_df):
        symbols = ["BTC-USD"]
        cohort_file = self._make_cohort_file(tmp_path, symbols)
        self._seed_substrate(tmp_path, 2, symbols, sample_df)

        with patch("candle_substrate.download_ticker_with_stderr",
                   side_effect=self._mock_download):
            manifest_path, _, _ = incremental_update_cohort(
                cohort_file=cohort_file,
                batch_id=2,
                root=tmp_path / "state_archive" / "candles",
            )

        manifest = json.loads(manifest_path.read_text())
        assert "timeline_fingerprint" in manifest
        fp = manifest["timeline_fingerprint"]
        assert len(fp) == 16
        assert all(c in "0123456789abcdef" for c in fp)

    def test_tz_naive_df_new_does_not_raise(self, tmp_path, sample_df):
        """Fix 3: tz-naive df_new must be normalised before concat with tz-aware df_old."""
        symbols = ["BTC-USD"]
        cohort_file = self._make_cohort_file(tmp_path, symbols)
        self._seed_substrate(tmp_path, 3, symbols, sample_df)

        def tz_naive_download(symbol, interval="5m", period="5d"):
            idx = pd.to_datetime(["2024-01-02 00:00", "2024-01-02 00:05"])  # tz-naive
            df = pd.DataFrame(
                {"Open": [200.0, 201.0], "High": [205.0, 206.0],
                 "Low": [199.0, 200.0], "Close": [203.0, 204.0], "Volume": [2000.0, 2100.0]},
                index=idx,
            )
            return df, ""

        # Must not raise "Cannot compare tz-naive and tz-aware timestamps"
        with patch("candle_substrate.download_ticker_with_stderr",
                   side_effect=tz_naive_download):
            manifest_path, sym_ts, _ = incremental_update_cohort(
                cohort_file=cohort_file,
                batch_id=3,
                root=tmp_path / "state_archive" / "candles",
            )
        assert "BTC-USD" in sym_ts

    def test_max_workers_default_is_one(self):
        """Fix 4: default max_workers must be 1 to prevent race conditions."""
        import inspect
        sig = inspect.signature(incremental_update_cohort)
        assert sig.parameters["max_workers"].default == 1

    def test_stats_dict_returned(self, tmp_path, sample_df):
        symbols = ["BTC-USD"]
        cohort_file = self._make_cohort_file(tmp_path, symbols)
        self._seed_substrate(tmp_path, 4, symbols, sample_df)

        with patch("candle_substrate.download_ticker_with_stderr",
                   side_effect=self._mock_download):
            _, _, stats = incremental_update_cohort(
                cohort_file=cohort_file,
                batch_id=4,
                root=tmp_path / "state_archive" / "candles",
            )

        assert isinstance(stats, dict)
        assert "attempted" in stats
        assert "empty_responses" in stats


# ── TestMergeDedup ────────────────────────────────────────────────────────────

class TestMergeDedup:
    """Merge strategy: keep='last' so newer download overwrites stale stored data."""

    def test_merge_keeps_last(self, tmp_path, sample_df):
        """When df_old and df_new share a timestamp, df_new value wins."""
        symbols = ["BTC-USD"]
        cohort_dir = tmp_path / "cohorts"
        cohort_dir.mkdir()
        cohort_file = cohort_dir / "cohort.txt"
        cohort_file.write_text("BTC-USD")

        root = tmp_path / "state_archive" / "candles"
        batch_dir = frozen_batch_dir(1, root)
        sym_dir = batch_dir / "symbols"
        sym_dir.mkdir(parents=True)

        # Seed with old data: Open=100 at ts 2024-01-01 00:00
        old_idx = pd.to_datetime(["2024-01-01 00:00"], utc=True)
        df_old = pd.DataFrame(
            {"Open": [100.0], "High": [105.0], "Low": [99.0],
             "Close": [103.0], "Volume": [1000.0]},
            index=old_idx,
        )
        p = symbol_path(batch_dir, "BTC-USD")
        write_symbol_candles(p, df_to_records(df_old))

        # New download: same ts but Open=999 (should win)
        new_idx = pd.to_datetime(["2024-01-01 00:00"], utc=True)
        df_new = pd.DataFrame(
            {"Open": [999.0], "High": [999.0], "Low": [999.0],
             "Close": [999.0], "Volume": [9999.0]},
            index=new_idx,
        )

        def mock_dl(symbol, interval="5m", period="5d"):
            return df_new.copy(), ""

        with patch("candle_substrate.download_ticker_with_stderr", side_effect=mock_dl):
            incremental_update_cohort(
                cohort_file=cohort_file,
                batch_id=1,
                root=root,
                max_workers=1,
            )

        result = read_symbol_candles(p)
        assert len(result) == 1
        assert result.iloc[0]["Open"] == 999.0, "keep='last' must let df_new overwrite df_old"