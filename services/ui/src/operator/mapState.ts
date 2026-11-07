/**
 * Presentation mapping from Pi `/api/state` JSON → operator DTOs.
 * Subset-only; no Policy B/C / OF / NSE ranking logic.
 */

import type {
  CryptoStatus,
  DecisionCardView,
  NseStatus,
  OfCandleMark,
  OfRecommendation,
  OperatorState,
  SystemHealth,
  TradePlanView,
} from './types';
import { numOrNull } from './freshness';

function asRecord(v: unknown): Record<string, unknown> | null {
  return v && typeof v === 'object' && !Array.isArray(v) ? (v as Record<string, unknown>) : null;
}

function mapCandle(raw: unknown): OfCandleMark | null {
  const o = asRecord(raw);
  if (!o) return null;
  return {
    bar_idx: numOrNull(o.bar_idx),
    decision_event_idx: numOrNull(o.decision_event_idx),
    candle_open_time_ms: numOrNull(o.candle_open_time_ms),
    candle_close_time_ms: numOrNull(o.candle_close_time_ms),
    candle_open_utc: (o.candle_open_utc as string) ?? null,
    candle_close_utc: (o.candle_close_utc as string) ?? null,
    mark_time_ms: numOrNull(o.mark_time_ms),
    mark_time_utc: (o.mark_time_utc as string) ?? null,
    candle_label: (o.candle_label as string) ?? null,
  };
}

function mapOfRec(raw: unknown, activeHint?: boolean): OfRecommendation | null {
  const o = asRecord(raw);
  if (!o) return null;
  const active =
    typeof activeHint === 'boolean'
      ? activeHint
      : Boolean(o.active ?? o.order_flow_active ?? false);
  return {
    active,
    symbol: (o.symbol as string) ?? null,
    direction: (o.direction as string) ?? null,
    order_flow_label: (o.order_flow_label as string) ?? null,
    order_flow_supported: Boolean(o.order_flow_supported),
    order_flow_imbalance: numOrNull(o.order_flow_imbalance),
    order_flow_source: (o.order_flow_source as string) ?? null,
    plan_id: (o.plan_id as string) ?? null,
    entry_price: numOrNull(o.entry_price),
    entry_candle: mapCandle(o.entry_candle ?? o.of_entry_candle),
    exit_candle: mapCandle(o.exit_candle ?? o.of_exit_candle),
    exit_reason: (o.exit_reason as string) ?? null,
    exit_imbalance: numOrNull(o.exit_imbalance),
    exit_opposite_side: (o.exit_opposite_side as string) ?? null,
    note: (o.note as string) ?? null,
    live_order_authority: numOrNull(o.live_order_authority),
  };
}

function mapTradePlan(raw: unknown): TradePlanView | null {
  const o = asRecord(raw);
  if (!o) return null;
  const val = asRecord(o.valuation);
  return {
    plan_id: (o.plan_id as string) ?? null,
    direction: (o.direction as string) ?? null,
    state: (o.state as string) ?? null,
    entry_price: numOrNull(o.entry_price),
    stop_loss_price: numOrNull(o.stop_loss_price),
    profit_target_price: numOrNull(o.profit_target_price),
    invalidation_price: numOrNull(o.invalidation_price),
    atr_14: numOrNull(o.atr_14),
    expiry_time_ms: numOrNull(o.expiry_time_ms),
    max_holding_seconds: numOrNull(o.max_holding_seconds),
    is_active: Boolean(o.is_active ?? (o.state === 'OPEN' || o.state === 'ACTIVE')),
    order_authority: (o.order_authority as string) ?? null,
    order_flow_supported:
      o.order_flow_supported === undefined || o.order_flow_supported === null
        ? null
        : Boolean(o.order_flow_supported),
    valuation: val
      ? {
          unrealized_net_pnl_usdt: numOrNull(val.unrealized_net_pnl_usdt),
          unrealized_net_return_bps: numOrNull(val.unrealized_net_return_bps),
          current_bid: numOrNull(val.current_bid),
          current_ask: numOrNull(val.current_ask),
          executable_exit_price: numOrNull(val.executable_exit_price),
          time_remaining_seconds: numOrNull(val.time_remaining_seconds),
        }
      : null,
  };
}

function mapCard(raw: unknown): DecisionCardView {
  const o = asRecord(raw) ?? {};
  return {
    bar_idx: numOrNull(o.bar_idx),
    decision_event_idx: numOrNull(o.decision_event_idx),
    timestamp_utc: (o.timestamp_utc as string) ?? null,
    close_time_ms: numOrNull(o.close_time_ms),
    reference_price: numOrNull(o.reference_price),
    policy_b_action: (o.policy_b_action as string) ?? null,
    policy_c_action: (o.policy_c_action as string) ?? null,
    ret_15m_bps: numOrNull(o.ret_15m_bps),
    atr_14: numOrNull(o.atr_14),
    quote_age_ms: numOrNull(o.quote_age_ms),
    spread_bps: numOrNull(o.spread_bps),
    order_flow_supported:
      o.order_flow_supported === undefined || o.order_flow_supported === null
        ? null
        : Boolean(o.order_flow_supported),
    order_flow_label: (o.order_flow_label as string) ?? null,
  };
}

export function mapCrypto(raw: Record<string, unknown>): CryptoStatus {
  const latest = asRecord(raw.latest_card);
  const ofActive = asRecord(raw.order_flow_recommendation);
  const ofExit = asRecord(raw.order_flow_last_exit);
  const markers = asRecord(raw.of_candle_markers);
  const ofPnlRaw = asRecord(raw.of_trade_pnl);
  const ofPnlSummary = asRecord(ofPnlRaw?.summary);
  const ofPnlRecent = Array.isArray(ofPnlRaw?.recent) ? ofPnlRaw!.recent : [];
  const ofShadowRaw = asRecord(raw.of_shadow_trade);
  const recentRaw = Array.isArray(raw.recent_cards) ? raw.recent_cards : [];

  const ofRec = ofActive ? mapOfRec(ofActive, Boolean(ofActive.active ?? true)) : null;
  const ofLastExit = ofExit ? mapOfRec(ofExit, false) : null;

  return {
    available: true,
    policy_b: (latest?.policy_b_action as string) ?? (raw.policy_b as string) ?? null,
    policy_c: (latest?.policy_c_action as string) ?? (raw.policy_c as string) ?? null,
    reference_price: numOrNull(latest?.reference_price ?? raw.reference_price ?? raw.mid),
    imbalance: numOrNull(latest?.imbalance ?? raw.imbalance),
    quote_age_ms: numOrNull(latest?.quote_age_ms ?? raw.quote_age_ms),
    atr_14: numOrNull(latest?.atr_14 ?? raw.atr_14),
    regime: (latest?.regime as string) ?? (raw.regime as string) ?? null,
    trade_plan: mapTradePlan(raw.trade_plan),
    order_flow: ofRec,
    order_flow_last_exit: ofLastExit,
    of_candle_markers: markers
      ? {
          entry: mapCandle(markers.entry),
          exit: mapCandle(markers.exit),
          active: Boolean(markers.active),
        }
      : ofRec
        ? {
            entry: ofRec.entry_candle ?? null,
            exit: ofRec.exit_candle ?? ofLastExit?.exit_candle ?? null,
            active: ofRec.active,
          }
        : null,
    of_trade_pnl: ofPnlRaw
      ? {
          protocol_id: (ofPnlRaw.protocol_id as string) ?? null,
          execution_mode: (ofPnlRaw.execution_mode as string) ?? null,
          live_order_authority: numOrNull(ofPnlRaw.live_order_authority),
          summary: ofPnlSummary
            ? {
                n_trades: numOrNull(ofPnlSummary.n_trades) ?? undefined,
                n_wins: numOrNull(ofPnlSummary.n_wins) ?? undefined,
                n_losses: numOrNull(ofPnlSummary.n_losses) ?? undefined,
                cumulative_gross_pnl_usdt:
                  numOrNull(ofPnlSummary.cumulative_gross_pnl_usdt) ?? undefined,
                cumulative_net_pnl_usdt:
                  numOrNull(ofPnlSummary.cumulative_net_pnl_usdt) ?? undefined,
                cumulative_net_return_bps:
                  numOrNull(ofPnlSummary.cumulative_net_return_bps) ?? undefined,
                cumulative_friction_usdt:
                  numOrNull(ofPnlSummary.cumulative_friction_usdt) ?? undefined,
              }
            : null,
          recent: ofPnlRecent.map((r) => {
            const o = asRecord(r) ?? {};
            return {
              of_trade_id: (o.of_trade_id as string) ?? null,
              plan_id: (o.plan_id as string) ?? null,
              direction: (o.direction as string) ?? null,
              entry_price: numOrNull(o.entry_price),
              exit_price: numOrNull(o.exit_price),
              net_pnl_usdt: numOrNull(o.net_pnl_usdt),
              net_return_bps: numOrNull(o.net_return_bps),
              gross_pnl_usdt: numOrNull(o.gross_pnl_usdt),
              friction_usdt: numOrNull(o.friction_usdt),
              spread_cost_usdt: numOrNull(o.spread_cost_usdt),
              fees_usdt: numOrNull(o.fees_usdt),
              slippage_usdt: numOrNull(o.slippage_usdt),
              holding_duration_seconds: numOrNull(o.holding_duration_seconds),
              exit_reason: (o.exit_reason as string) ?? null,
              activated_ms: numOrNull(o.activated_ms),
              exited_ms: numOrNull(o.exited_ms),
              paper_open_timestamp: (o.paper_open_timestamp as string) ?? null,
              signal_exit_ms: numOrNull(o.signal_exit_ms),
              signal_exit_timestamp: (o.signal_exit_timestamp as string) ?? null,
              paper_close_ms: numOrNull(o.paper_close_ms),
              paper_close_timestamp: (o.paper_close_timestamp as string) ?? null,
              paper_close_lag_ms: numOrNull(o.paper_close_lag_ms),
              execution_mode: (o.execution_mode as string) ?? null,
              live_order_authority: numOrNull(o.live_order_authority),
            };
          }),
        }
      : null,
    of_shadow_trade:
      ofShadowRaw && ofShadowRaw.open
        ? {
            open: true,
            of_trade_id: (ofShadowRaw.of_trade_id as string) ?? null,
            direction: (ofShadowRaw.direction as string) ?? null,
            entry_price: numOrNull(ofShadowRaw.entry_price),
            mark_price: numOrNull(ofShadowRaw.mark_price),
            simulated_quantity: numOrNull(ofShadowRaw.simulated_quantity),
            simulated_notional_usdt: numOrNull(ofShadowRaw.simulated_notional_usdt),
            entry_signal_qty: numOrNull(ofShadowRaw.entry_signal_qty),
            participation_pct: numOrNull(ofShadowRaw.participation_pct),
            sizing_source: (ofShadowRaw.sizing_source as string) ?? null,
            holding_duration_seconds: numOrNull(ofShadowRaw.holding_duration_seconds),
            unrealized_gross_pnl_usdt: numOrNull(ofShadowRaw.unrealized_gross_pnl_usdt),
            unrealized_net_pnl_usdt: numOrNull(ofShadowRaw.unrealized_net_pnl_usdt),
            unrealized_net_return_bps: numOrNull(ofShadowRaw.unrealized_net_return_bps),
            activated_ms: numOrNull(ofShadowRaw.activated_ms),
            spread_cost_usdt: numOrNull(ofShadowRaw.spread_cost_usdt),
            fees_usdt: numOrNull(ofShadowRaw.fees_usdt),
            slippage_usdt: numOrNull(ofShadowRaw.slippage_usdt),
            friction_usdt: numOrNull(ofShadowRaw.friction_usdt),
            execution_mode: (ofShadowRaw.execution_mode as string) ?? null,
            live_order_authority: numOrNull(ofShadowRaw.live_order_authority),
            paper_open_timestamp: (ofShadowRaw.paper_open_timestamp as string) ?? null,
            exit_rule: (ofShadowRaw.exit_rule as string) ?? null,
            independent_of_trade_plan_ttl: Boolean(
              ofShadowRaw.independent_of_trade_plan_ttl ?? true
            ),
          }
        : ofShadowRaw
          ? { open: false }
          : null,
    recent_cards: recentRaw.map(mapCard),
  };
}

export function mapNse(raw: Record<string, unknown> | null): NseStatus {
  if (!raw) {
    return { available: false };
  }
  const next = asRecord(raw.next_session_recommendations) ?? {};
  const evals = asRecord(raw.evaluations_summary) ?? {};
  const rp607 = asRecord(raw.rp607_summary) ?? {};
  const coverage = asRecord(raw.coverage) ?? {};
  const mom = asRecord(evals.momentum_distribution);

  const classified =
    numOrNull(rp607.classified_count) ??
    numOrNull(rp607.n_classified) ??
    numOrNull(next.universe_eligible_n);
  const census =
    numOrNull(coverage.total_universe) ??
    numOrNull(coverage.reconciled) ??
    numOrNull(rp607.census_total) ??
    119;
  const quarantined =
    numOrNull(rp607.quarantined_count) ??
    (classified != null && census != null ? Math.max(0, census - classified) : null);

  const recs = Array.isArray(next.recommendations) ? next.recommendations : [];

  return {
    available: true,
    status: (raw.status as string) ?? null,
    order_authority: numOrNull(raw.order_authority) ?? 0,
    census_total: census,
    classified_count: classified,
    quarantined_count: quarantined,
    next_session_date: (next.next_session_date as string) ?? null,
    from_session_date: (next.from_session_date as string) ?? null,
    n_recommendations: numOrNull(next.n_recommendations) ?? recs.length,
    n_order_flow_supported: numOrNull(next.n_order_flow_supported),
    n_live_order_flow_supported:
      numOrNull(raw.n_live_order_flow_supported) ??
      numOrNull(evals.n_live_order_flow_supported),
    n_live_order_flow_long:
      numOrNull(raw.n_live_order_flow_long) ??
      numOrNull(evals.n_live_order_flow_long),
    n_live_order_flow_short:
      numOrNull(raw.n_live_order_flow_short) ??
      numOrNull(evals.n_live_order_flow_short),
    universe_eligible_n: numOrNull(next.universe_eligible_n),
    universe_label: (next.universe_label as string) ?? null,
    next_session_note: (next.note as string) ?? null,
    recommended_at_utc:
      (next.recommended_at_utc as string) ?? (next.emitted_utc as string) ?? null,
    emitted_utc: (next.emitted_utc as string) ?? null,
    stamp_mtime_utc: (next.stamp_mtime_utc as string) ?? null,
    live_update: next.live_update !== false,
    live_poll_hint_sec: numOrNull(next.live_poll_hint_sec) ?? 5,
    recommendations: recs.map((r) => {
      const o = asRecord(r) ?? {};
      return {
        symbol: (o.symbol as string) ?? null,
        tradingsymbol: (o.tradingsymbol as string) ?? null,
        side: (o.side as string) ?? null,
        status: (o.status as string) ?? null,
        score: numOrNull(o.score),
        traj15_eod: numOrNull(o.traj15_eod),
        session_return_bps: numOrNull(o.session_return_bps),
        close_price: numOrNull(o.close_price),
        order_flow_supported: Boolean(o.order_flow_supported),
        order_flow_label: (o.order_flow_label as string) ?? null,
        order_flow_imbalance: numOrNull(o.order_flow_imbalance),
        thesis: (o.thesis as string) ?? null,
        recommended_at_utc:
          (o.recommended_at_utc as string) ??
          (next.recommended_at_utc as string) ??
          (next.emitted_utc as string) ??
          null,
        levels: asRecord(o.levels),
        level_status: (o.level_status as string) ?? null,
        entry_low: numOrNull(o.entry_low),
        entry_high: numOrNull(o.entry_high),
        stop_price: numOrNull(o.stop_price),
        target_price: numOrNull(o.target_price),
        risk_reward_display: (o.risk_reward_display as string) ?? null,
        not_executed_entry: o.not_executed_entry !== false,
        not_paper_trade_entry: o.not_paper_trade_entry !== false,
      };
    }),
    total_evaluated: numOrNull(evals.total_evaluated),
    n_qualifying: numOrNull(evals.n_qualifying),
    n_high_target: numOrNull(evals.n_high_target),
    rejection_breakdown: (evals.rejection_breakdown as Record<string, number>) ?? null,
    momentum_distribution: mom
      ? {
          bullish_ge_005: numOrNull(mom.bullish_ge_005) ?? undefined,
          neutral: numOrNull(mom.neutral) ?? undefined,
          bearish_le_005: numOrNull(mom.bearish_le_005) ?? undefined,
        }
      : null,
    daemon_pid: (raw.daemon_pid as number | string) ?? null,
    services: (raw.services as Record<string, string>) ?? null,
    scrips: Array.isArray(raw.scrips) ? (raw.scrips as Array<Record<string, unknown>>) : [],
  };
}

export function mapSystem(raw: Record<string, unknown>): SystemHealth {
  const svc = asRecord(raw.service_status);
  const gates = asRecord(raw.telemetry_gates);
  const ck = asRecord(raw.checkpoint_integrity);
  const acct = asRecord(raw.accounting);
  return {
    available: true,
    live_order_authority: numOrNull(raw.live_order_authority) ?? 0,
    server_time_utc: (raw.server_time_utc as string) ?? null,
    server_time_ist: (raw.server_time_ist as string) ?? null,
    service_status: svc
      ? {
          active: Boolean(svc.active),
          status_text: (svc.status_text as string) ?? undefined,
        }
      : null,
    window_active:
      raw.window_active === undefined || raw.window_active === null
        ? null
        : Boolean(raw.window_active),
    window_elapsed_hours: numOrNull(raw.window_elapsed_hours),
    telemetry_gates: gates
      ? {
          quote_coverage_pct: numOrNull(gates.quote_coverage_pct),
          max_drift_ms: numOrNull(gates.max_drift_ms),
          consecutive_missing_bars: numOrNull(gates.consecutive_missing_bars),
          total_missing_bars: numOrNull(gates.total_missing_bars),
          reconnection_count_24h: numOrNull(gates.reconnection_count_24h),
          window_tainted:
            gates.window_tainted === undefined || gates.window_tainted === null
              ? null
              : Boolean(gates.window_tainted),
        }
      : null,
    checkpoint_integrity: ck
      ? {
          recorded: (ck.recorded as string) ?? null,
          status: (ck.status as string) ?? null,
          last_durable_bar_idx: numOrNull(ck.last_durable_bar_idx),
          last_heartbeat_utc: (ck.last_heartbeat_utc as string) ?? null,
        }
      : null,
    accounting: acct
      ? {
          total_cards: numOrNull(acct.total_cards),
          prospective_count: numOrNull(acct.prospective_count),
          prewindow_excluded: numOrNull(acct.prewindow_excluded),
        }
      : null,
  };
}

export function mapOperatorState(
  raw: Record<string, unknown> | null,
  meta: { fetch_status: 'ok' | 'error' | 'unavailable'; fetch_error?: string | null; fetched_at_ms: number }
): OperatorState {
  if (!raw || meta.fetch_status !== 'ok') {
    return {
      fetch_status: meta.fetch_status,
      fetch_error: meta.fetch_error ?? null,
      fetched_at_ms: meta.fetched_at_ms,
      raw_available: false,
      live_order_authority: 0,
      crypto: { available: false },
      nse: { available: false },
      system: { available: false, live_order_authority: 0 },
    };
  }

  const nseOps = asRecord(raw.nse_operations);
  const rp611 = asRecord(raw.rp611);
  const rp611Nse = asRecord(rp611?.nse);
  const rp611Cry = asRecord(rp611?.crypto);
  // Gate5 is observation-only: authority is always presented as 0 unless Pi sends a number.
  const authority = numOrNull(raw.live_order_authority) ?? numOrNull(nseOps?.order_authority) ?? 0;
  return {
    fetch_status: 'ok',
    fetch_error: null,
    fetched_at_ms: meta.fetched_at_ms,
    raw_available: true,
    live_order_authority: authority,
    crypto: mapCrypto(raw),
    nse: mapNse(nseOps),
    system: { ...mapSystem(raw), live_order_authority: authority },
    rp611: rp611
      ? {
          nse: rp611Nse
            ? {
                available: Boolean(rp611Nse.available),
                short_id: (rp611Nse.short_id as string) ?? null,
                disposition: (rp611Nse.disposition as string) ?? null,
                experiment_clock_started: Boolean(rp611Nse.experiment_clock_started),
                experiment_clock_started_utc:
                  (rp611Nse.experiment_clock_started_utc as string) ?? null,
                observer_deployed: Boolean(rp611Nse.observer_deployed),
                phase: (rp611Nse.phase as string) ?? null,
                phase1_deployed_utc: (rp611Nse.phase1_deployed_utc as string) ?? null,
                pre_deployment_capture_gap_sec: numOrNull(
                  rp611Nse.pre_deployment_capture_gap_sec
                ),
                journal_counts: (rp611Nse.journal_counts as Record<string, number>) ?? {},
              }
            : undefined,
          crypto: rp611Cry
            ? {
                available: Boolean(rp611Cry.available),
                short_id: (rp611Cry.short_id as string) ?? null,
                disposition: (rp611Cry.disposition as string) ?? null,
                experiment_clock_started: Boolean(rp611Cry.experiment_clock_started),
                experiment_clock_started_utc:
                  (rp611Cry.experiment_clock_started_utc as string) ?? null,
                observer_deployed: Boolean(rp611Cry.observer_deployed),
                phase: (rp611Cry.phase as string) ?? null,
                phase1_deployed_utc: (rp611Cry.phase1_deployed_utc as string) ?? null,
                pre_deployment_capture_gap_sec: numOrNull(
                  rp611Cry.pre_deployment_capture_gap_sec
                ),
                journal_counts: (rp611Cry.journal_counts as Record<string, number>) ?? {},
                volume_source: (rp611Cry.volume_source as string) ?? null,
              }
            : undefined,
        }
      : null,
  };
}
