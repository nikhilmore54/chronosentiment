"""
RP-608-OFX — executed-quantity directional Order Flow (research-only).

Construct: signed aggressor volume ΔQ = Q_buy − Q_sell.
Not baseline L1 OF. Not candle colour. Authority 0%.
"""

from .aggregate import (
    DEFAULT_WINDOW_MS,
    CandleContext,
    ExecutedFlowWindowResult,
    ExecutedTrade,
    SideClass,
    aggregate_executed_flow,
    candle_direction_from_ohlc,
)
from .crypto_aggtrade import parse_binance_agg_trade, trades_from_aggtrade_records
from .nse_inferred import trades_from_nse_inferred_ticks
from .volume_mark import (
    OFX_ENTRY_SURGE_LOOKBACK_TICKS,
    OFX_ENTRY_SURGE_MULTIPLE,
    OFX_VOLUME_IMBALANCE_THRESHOLD,
    classify_ofx_volume_support,
    evaluate_ofx_entry_volume_surge,
    inspect_ofx_volume_book,
    is_ofx_volume_exit_signal,
    ofx_entry_surge_from_trades,
)

__all__ = [
    "DEFAULT_WINDOW_MS",
    "CandleContext",
    "ExecutedFlowWindowResult",
    "ExecutedTrade",
    "SideClass",
    "aggregate_executed_flow",
    "candle_direction_from_ohlc",
    "parse_binance_agg_trade",
    "trades_from_aggtrade_records",
    "trades_from_nse_inferred_ticks",
    "OFX_VOLUME_IMBALANCE_THRESHOLD",
    "OFX_ENTRY_SURGE_MULTIPLE",
    "OFX_ENTRY_SURGE_LOOKBACK_TICKS",
    "classify_ofx_volume_support",
    "evaluate_ofx_entry_volume_surge",
    "ofx_entry_surge_from_trades",
    "inspect_ofx_volume_book",
    "is_ofx_volume_exit_signal",
]
