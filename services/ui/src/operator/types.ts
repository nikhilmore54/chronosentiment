/**
 * Read-only operator DTOs mapped from Pi `/api/state`.
 * Presentation only — no ChronoSentiment behavioural logic.
 */

export type FetchStatus = 'ok' | 'error' | 'unavailable';

export type OfCandleMark = {
  bar_idx?: number | null;
  decision_event_idx?: number | null;
  candle_open_time_ms?: number | null;
  candle_close_time_ms?: number | null;
  candle_open_utc?: string | null;
  candle_close_utc?: string | null;
  mark_time_ms?: number | null;
  mark_time_utc?: string | null;
  candle_label?: string | null;
};

export type OfRecommendation = {
  active: boolean;
  symbol?: string | null;
  direction?: string | null;
  order_flow_label?: string | null;
  order_flow_supported?: boolean;
  order_flow_imbalance?: number | null;
  order_flow_source?: string | null;
  plan_id?: string | null;
  entry_price?: number | null;
  entry_candle?: OfCandleMark | null;
  exit_candle?: OfCandleMark | null;
  exit_reason?: string | null;
  exit_imbalance?: number | null;
  exit_opposite_side?: string | null;
  note?: string | null;
  live_order_authority?: number | null;
};

export type TradePlanView = {
  plan_id?: string | null;
  direction?: string | null;
  state?: string | null;
  entry_price?: number | null;
  stop_loss_price?: number | null;
  profit_target_price?: number | null;
  invalidation_price?: number | null;
  atr_14?: number | null;
  expiry_time_ms?: number | null;
  max_holding_seconds?: number | null;
  is_active?: boolean;
  order_authority?: string | null;
  order_flow_supported?: boolean | null;
  valuation?: {
    unrealized_net_pnl_usdt?: number | null;
    unrealized_net_return_bps?: number | null;
    current_bid?: number | null;
    current_ask?: number | null;
    executable_exit_price?: number | null;
    time_remaining_seconds?: number | null;
  } | null;
};

export type DecisionCardView = {
  bar_idx?: number | null;
  decision_event_idx?: number | null;
  timestamp_utc?: string | null;
  close_time_ms?: number | null;
  reference_price?: number | null;
  policy_b_action?: string | null;
  policy_c_action?: string | null;
  ret_15m_bps?: number | null;
  atr_14?: number | null;
  quote_age_ms?: number | null;
  spread_bps?: number | null;
  order_flow_supported?: boolean | null;
  order_flow_label?: string | null;
};

export type OfTradeOutcome = {
  of_trade_id?: string | null;
  plan_id?: string | null;
  direction?: string | null;
  entry_price?: number | null;
  exit_price?: number | null;
  net_pnl_usdt?: number | null;
  net_return_bps?: number | null;
  gross_pnl_usdt?: number | null;
  friction_usdt?: number | null;
  spread_cost_usdt?: number | null;
  fees_usdt?: number | null;
  slippage_usdt?: number | null;
  holding_duration_seconds?: number | null;
  exit_reason?: string | null;
  activated_ms?: number | null;
  exited_ms?: number | null;
  paper_open_timestamp?: string | null;
  signal_exit_ms?: number | null;
  signal_exit_timestamp?: string | null;
  paper_close_ms?: number | null;
  paper_close_timestamp?: string | null;
  paper_close_lag_ms?: number | null;
  execution_mode?: string | null;
  live_order_authority?: number | null;
};

export type OfShadowTrade = {
  open?: boolean;
  of_trade_id?: string | null;
  direction?: string | null;
  entry_price?: number | null;
  mark_price?: number | null;
  simulated_quantity?: number | null;
  simulated_notional_usdt?: number | null;
  entry_signal_qty?: number | null;
  participation_pct?: number | null;
  sizing_source?: string | null;
  holding_duration_seconds?: number | null;
  unrealized_gross_pnl_usdt?: number | null;
  unrealized_net_pnl_usdt?: number | null;
  unrealized_net_return_bps?: number | null;
  activated_ms?: number | null;
  spread_cost_usdt?: number | null;
  fees_usdt?: number | null;
  slippage_usdt?: number | null;
  friction_usdt?: number | null;
  execution_mode?: string | null;
  live_order_authority?: number | null;
  paper_open_timestamp?: string | null;
  exit_rule?: string | null;
  independent_of_trade_plan_ttl?: boolean;
};

export type OfTradePnl = {
  protocol_id?: string | null;
  execution_mode?: string | null;
  live_order_authority?: number | null;
  summary?: {
    n_trades?: number;
    n_wins?: number;
    n_losses?: number;
    cumulative_gross_pnl_usdt?: number;
    cumulative_net_pnl_usdt?: number;
    cumulative_net_return_bps?: number;
    cumulative_friction_usdt?: number;
  } | null;
  recent?: OfTradeOutcome[];
};

export type CryptoStatus = {
  available: boolean;
  policy_b?: string | null;
  policy_c?: string | null;
  reference_price?: number | null;
  imbalance?: number | null;
  quote_age_ms?: number | null;
  atr_14?: number | null;
  regime?: string | null;
  trade_plan?: TradePlanView | null;
  order_flow?: OfRecommendation | null;
  order_flow_last_exit?: OfRecommendation | null;
  of_candle_markers?: {
    entry?: OfCandleMark | null;
    exit?: OfCandleMark | null;
    active?: boolean;
  } | null;
  of_trade_pnl?: OfTradePnl | null;
  of_shadow_trade?: OfShadowTrade | null;
  recent_cards?: DecisionCardView[];
};

export type NextSessionLevels = {
  entry_low?: number | null;
  entry_high?: number | null;
  stop_price?: number | null;
  target_price?: number | null;
  risk_reward_display?: string | null;
  risk_reward_gross?: number | null;
  risk_reward_net?: number | null;
  reference_price?: number | null;
  level_status?: string | null;
  level_status_reason?: string | null;
  levels_calculated_at_utc?: string | null;
  policy_id?: string | null;
  levels_role?: string | null;
  not_executed_entry?: boolean;
  not_paper_trade_entry?: boolean;
};

export type NextSessionWatch = {
  symbol?: string | null;
  tradingsymbol?: string | null;
  side?: string | null;
  status?: string | null;
  score?: number | null;
  traj15_eod?: number | null;
  session_return_bps?: number | null;
  close_price?: number | null;
  order_flow_supported?: boolean;
  order_flow_label?: string | null;
  order_flow_imbalance?: number | null;
  thesis?: string | null;
  recommended_at_utc?: string | null;
  levels?: NextSessionLevels | null;
  level_status?: string | null;
  entry_low?: number | null;
  entry_high?: number | null;
  stop_price?: number | null;
  target_price?: number | null;
  risk_reward_display?: string | null;
  not_executed_entry?: boolean;
  not_paper_trade_entry?: boolean;
};

export type NseStatus = {
  available: boolean;
  status?: string | null;
  order_authority?: number | null;
  census_total?: number | null;
  classified_count?: number | null;
  quarantined_count?: number | null;
  next_session_date?: string | null;
  from_session_date?: string | null;
  n_recommendations?: number | null;
  n_order_flow_supported?: number | null;
  n_live_order_flow_supported?: number | null;
  n_live_order_flow_long?: number | null;
  n_live_order_flow_short?: number | null;
  universe_eligible_n?: number | null;
  universe_label?: string | null;
  next_session_note?: string | null;
  recommended_at_utc?: string | null;
  emitted_utc?: string | null;
  stamp_mtime_utc?: string | null;
  live_update?: boolean;
  live_poll_hint_sec?: number | null;
  recommendations?: NextSessionWatch[];
  total_evaluated?: number | null;
  n_qualifying?: number | null;
  n_high_target?: number | null;
  rejection_breakdown?: Record<string, number> | null;
  momentum_distribution?: {
    bullish_ge_005?: number;
    neutral?: number;
    bearish_le_005?: number;
  } | null;
  daemon_pid?: number | string | null;
  services?: Record<string, string> | null;
  scrips?: Array<Record<string, unknown>>;
};

export type SystemHealth = {
  available: boolean;
  live_order_authority: number;
  server_time_utc?: string | null;
  server_time_ist?: string | null;
  service_status?: {
    active?: boolean;
    status_text?: string;
  } | null;
  window_active?: boolean | null;
  window_elapsed_hours?: number | null;
  telemetry_gates?: {
    quote_coverage_pct?: number | null;
    max_drift_ms?: number | null;
    consecutive_missing_bars?: number | null;
    total_missing_bars?: number | null;
    reconnection_count_24h?: number | null;
    window_tainted?: boolean | null;
  } | null;
  checkpoint_integrity?: {
    recorded?: string | null;
    status?: string | null;
    last_durable_bar_idx?: number | null;
    last_heartbeat_utc?: string | null;
  } | null;
  accounting?: {
    total_cards?: number | null;
    prospective_count?: number | null;
    prewindow_excluded?: number | null;
  } | null;
};

export type Rp611ArmStatus = {
  available?: boolean;
  short_id?: string | null;
  disposition?: string | null;
  experiment_clock_started?: boolean;
  experiment_clock_started_utc?: string | null;
  observer_deployed?: boolean;
  phase?: string | null;
  phase1_deployed_utc?: string | null;
  pre_deployment_capture_gap_sec?: number | null;
  journal_counts?: Record<string, number>;
  volume_source?: string | null;
};

export type OperatorState = {
  fetch_status: FetchStatus;
  fetch_error?: string | null;
  fetched_at_ms: number;
  raw_available: boolean;
  live_order_authority: number;
  crypto: CryptoStatus;
  nse: NseStatus;
  system: SystemHealth;
  rp611?: {
    nse?: Rp611ArmStatus;
    crypto?: Rp611ArmStatus;
  } | null;
};
