/**
 * DecisionBrief — clean DTO produced by the Rust IC v1 engine.
 *
 * Consumed by CockpitView. The UI never needs to know how the intelligence
 * is calculated — it only renders what the Rust service produces.
 *
 * Mirrors adapters/chronosentiment/src/product/intraday_decision.rs :: DecisionBrief
 */

/**
 * Adapter-owned execution facts from DecisionBrief.execution.
 * Distances are computed in the adapter. The UI never recalculates them.
 */
export interface ExecutionFacts {
  snap_unix: number | null;
  adaptive_target: number | null;
  adaptive_risk: number | null;
  adaptive_horizon_sessions: number | null;
  target_distance_abs: number | null;
  target_distance_pct: number | null;
  risk_distance_abs: number | null;
  risk_distance_pct: number | null;
  expected_move_pct: number | null;
  current_price: number | null;
  last_tick_unix: number | null;
  freshness: 'LIVE' | 'STALE' | string;
  horizon_elapsed: boolean | null;
}

export interface DecisionBrief {
  id: string;
  ticker: string;
  date: string;
  direction: 'LONG' | 'SHORT';
  oqs: number;
  h60_class: string;

  /** Copied from the adapter DecisionBrief. Never recalculated. */
  reference_price: number | null;
  /** Copied from the adapter DecisionBrief. Never recalculated. */
  entry_price: number | null;
  execution: ExecutionFacts;

  // Entry-time classification (IC v1, T0 information only)
  entry_state: string;
  entry_action: string;
  entry_confidence: string;
  entry_horizon: string;
  entry_why: string;
  entry_risk: string;

  // H120 reassessment (LONG WAIT-MID only; all others pass through)
  h120_state: string;
  h120_action: string;
  h120_confidence: string;
  h120_horizon: string;
  h120_why: string;
  h120_risk: string;

  // Path returns (direction-adjusted)
  h15_ret: number | null;
  h30_ret: number | null;
  h60_ret: number | null;
  h120_ret: number | null;
  h180_ret: number | null;
  h300_ret: number | null;
  mfe_h60: number | null;
  mfe_h120: number | null;

  // Outcome
  outcome: string | null;
  pnl: number | null;

  // Historical reference (frozen from v0.9)
  hist_win: number | null;
  hist_pf: number | null;
  hist_med: number | null;
}

export interface DecisionListResponse {
  total: number;
  decisions: DecisionBrief[];
}

/**
 * Portfolio context for a single IC v1 ACT decision.
 * Produced by GET /api/v1/intraday/portfolio-context.
 * Mirrors services/chronosentiment_server/src/intraday_api.rs :: DecisionPortfolioContext
 */
export interface DecisionPortfolioContext {
  id: string;
  ticker: string;
  date: string;
  direction: 'LONG' | 'SHORT';
  oqs: number;
  entry_state: string;
  entry_action: string;
  /** 1-based rank within the day's ACT candidates (sorted by OQS desc) */
  oqs_rank: number;
  /** Total ACT candidates on this date */
  candidates_on_date: number;
  /** Whether this decision is in the top MAX_POSITIONS for the day */
  selected: boolean;
  /** Position size as a percentage of portfolio capital (0 if not selected) */
  position_size_pct: number;
}

export interface PortfolioContextResponse {
  total_candidates: number;
  total_selected: number;
  max_positions: number;
  position_fraction: number;
  decisions: DecisionPortfolioContext[];
}

/**
 * A single recommendation card for the Decision Feed.
 * Produced by GET /api/v1/intraday/recommendations.
 * Mirrors services/chronosentiment_server/src/intraday_api.rs :: RecommendationCard
 */
export interface RecommendationCard {
  id: string;
  ticker: string;
  date: string;
  direction: 'LONG' | 'SHORT';
  oqs: number;
  oqs_rank: number;
  candidates_on_date: number;
  entry_state: string;
  entry_action: string;
  hist_win: number | null;
  hist_pf: number | null;
  hist_med: number | null;
  h300_ret: number | null;
  outcome: string | null;
  /** Copied from the adapter DecisionBrief. Never recalculated. */
  reference_price: number | null;
  /** Copied from the adapter DecisionBrief. Never recalculated. */
  entry_price: number | null;
  execution: ExecutionFacts;
  /** v0.1: always "NEW" — position management is a future capability */
  status: 'NEW' | 'ACTIVE' | 'REASSESS_EXIT' | 'CLOSED';
}

export interface RecommendationFeedResponse {
  total: number;
  selected_per_day: number;
  recommendations: RecommendationCard[];
}

/**
 * Authoritative facts supporting a recommendation.
 * The frontend renders these facts; it never manufactures explanations.
 * Mirrors services/chronosentiment_server/src/intraday_api.rs :: DecisionEvidence
 */
export interface DecisionEvidence {
  // Historical performance facts (frozen from v0.9)
  hist_win: number | null;
  hist_pf: number | null;
  hist_med: number | null;
  /** Copied from the adapter DecisionBrief. Never recalculated. */
  reference_price: number | null;
  /** Copied from the adapter DecisionBrief. Never recalculated. */
  entry_price: number | null;
  execution: ExecutionFacts;
  /** Exit price from reassessment v0.2 CSV (null if NEW). */
  exit_price: number | null;
  // Opportunity facts
  oqs: number;
  oqs_rank: number;
  candidates_on_date: number;
  entry_state: string;
  entry_why: string;
  entry_risk: string;
  entry_horizon: string;
  // Observed path facts (null for live/future decisions)
  h60_ret: number | null;
  h120_ret: number | null;
  h300_ret: number | null;
  mfe_h60: number | null;
  // Reassessment facts
  h120_state: string;
  h120_why: string;
}

/**
 * Lifecycle-enriched recommendation card.
 * Produced by GET /api/v1/intraday/position-lifecycle.
 * Extends RecommendationCard with position state from reassessment v0.2 replay.
 * Mirrors services/chronosentiment_server/src/intraday_api.rs :: LifecycleCard
 */
export interface LifecycleCard {
  id: string;
  ticker: string;
  date: string;
  direction: 'LONG' | 'SHORT';
  oqs: number;
  oqs_rank: number;
  candidates_on_date: number;
  entry_state: string;
  entry_action: string;
  /** Copied from the adapter DecisionBrief. Never recalculated. */
  reference_price: number | null;
  /** Copied from the adapter DecisionBrief. Never recalculated. */
  entry_price: number | null;
  execution: ExecutionFacts;
  /** Exit price from reassessment v0.2 CSV (null if NEW). */
  exit_price: number | null;
  hist_win: number | null;
  hist_pf: number | null;
  hist_med: number | null;
  h300_ret: number | null;
  outcome: string | null;
  /** NEW | REASSESS_EXIT | CLOSED */
  status: 'NEW' | 'REASSESS_EXIT' | 'CLOSED';
  /** Realized return at exit (null if NEW) */
  realized_ret: number | null;
  /** STOP | TARGET | HORIZON | REASSESS_EXIT (null if NEW) */
  exit_reason: string | null;
  /** IST time of reassessment exit (null if not reassessed) */
  reassess_time_ist: string | null;
  /** H300 counterfactual return — what would have happened without early exit */
  h300_counterfactual_ret: number | null;
  /** Authoritative facts supporting the recommendation — rendered by the UI, never manufactured */
  evidence: DecisionEvidence;
}

export interface LifecycleFeedResponse {
  total: number;
  selected_per_day: number;
  cards: LifecycleCard[];
}

/**
 * Frozen Paper Trader v0.2 position. paper_* prices are the replay fill,
 * not DecisionBrief.entry_price / reference_price.
 */
export interface PaperPosition {
  ticker: string;
  direction: 'LONG' | 'SHORT' | string;
  date: string;
  paper_entry_price: number;
  paper_target: number;
  paper_risk: number;
  candidate_stop_pct: number | null;
  candidate_stop_price: number | null;
  stop_confidence: string;
  coverage_quality: string;
  n_obs: number;
  path_context: string;
  final_tier: string;
  paper_exit_price: number | null;
  exit_time_ist: string | null;
  exit_reason: string;
  realized_return: number | null;
  max_adverse_excursion: number | null;
  max_favourable_excursion: number | null;
  bars_held: number;
  h300_counterfactual_ret: number | null;
  stop_consequence_pp: number | null;
  decision_id: string | null;
  entry_action: string | null;
  oqs: number | null;
}

export interface PaperBlotter {
  total: number;
  n_stop: number;
  n_target: number;
  n_horizon: number;
  n_win: number;
  mean_realized_return: number | null;
  total_realized_return: number | null;
  positions: PaperPosition[];
}

/** GET /api/v1/intraday/deferred-live — live paper ledger, not CSV replay. */
export interface DeferredLiveEvent {
  unix: number;
  kind: string;
  price: number;
  note: string | null;
}

export interface DeferredLiveWalkExit {
  reason: string;
  price: number;
  unix: number;
}

export interface DeferredLiveWalk {
  direction: string;
  entry: number;
  target: number;
  stop: number | null;
  bars_held: number;
  mae: number;
  mfe: number;
  exit: DeferredLiveWalkExit | null;
}

export type DeferredLiveHorizon =
  | { Unix: { horizon_unix: number } }
  | { SessionBarIndex: { limit: number } };

export interface DeferredLivePosition {
  paper: PaperPosition;
  walk: DeferredLiveWalk;
  current_price: number;
  opened_at: number;
  last_tick_at: number;
  horizon: DeferredLiveHorizon;
  events: DeferredLiveEvent[];
}

export interface DeferredLiveLedger {
  mode: string;
  positions: DeferredLivePosition[];
}

export interface DeferredLiveArmed {
  decision_id: string;
  ticker: string;
}

/** As-of IC offer copied from GET /deferred-live. Presentation only. */
export interface AsOfSessionEvent {
  ticker: string;
  as_of_unix: number;
  offer: string;
  decision_id: string;
  entry_action: string;
}

export interface ObservationFeedSnapshot {
  background: 'NONE' | 'EXTERNAL_LIVE' | 'CACHED_1M' | 'YAHOO_1M' | 'YAHOO_UNAUTHORIZED' | string;
  observation_source?: 'NONE' | 'EXTERNAL_LIVE' | 'CACHED_1M' | 'YAHOO_1M' | 'YAHOO_UNAUTHORIZED' | string;
  status: string;
  yahoo_authorized: boolean;
  clock_simulation: boolean;
  speed?: number | null;
  inbox_path: string | null;
  last_ingest_unix: number | null;
  last_source_kind: 'NONE' | 'EXTERNAL_LIVE' | 'CACHED_1M' | 'YAHOO_1M' | 'CLOCK_SIMULATION' | 'YAHOO_UNAUTHORIZED' | string | null;
  note: string;
}

export interface DeferredLiveSession {
  date: string;
  auto_arm: boolean;
  on_date: number;
  eligible: number;
  armed_at_open: number;
  skipped_not_act: number;
  skipped_missing_geometry: number;
  note: string;
}

export interface DeferredLivePerformanceRow {
  decision_id: string;
  ticker: string;
  direction: string;
  oqs?: number | null;
  decision_entry_price?: number | null;
  paper_entry_price: number;
  fill_vs_decision?: number | null;
  last_or_exit_price: number;
  outcome: string;
  ret_kind: string;
  ret: number;
  bars_held: number;
}

export interface DeferredLivePerformance {
  date?: string | null;
  n_entered: number;
  n_open: number;
  n_target: number;
  n_stop: number;
  n_horizon: number;
  n_win_closed: number;
  mean_closed_return?: number | null;
  mean_open_mark?: number | null;
  mean_fill_vs_decision?: number | null;
  rows: DeferredLivePerformanceRow[];
  note: string;
}

export interface ReassessVariantStats {
  n_triggered: number;
  n_helped: number;
  n_hurt: number;
  mean_delta_vs_frozen?: number | null;
  sum_delta_vs_frozen: number;
}

export interface ReassessExperimentRow {
  decision_id: string;
  ticker: string;
  direction: string;
  paper_entry_price: number;
  frozen_outcome: string;
  frozen_ret: number;
  frozen_ret_kind: string;
  v02_triggered: boolean;
  v02_price?: number | null;
  v02_unix?: number | null;
  v02_ret?: number | null;
  v02_delta_vs_frozen?: number | null;
  reassess_triggered: boolean;
  reassess_price?: number | null;
  reassess_unix?: number | null;
  experiment_ret?: number | null;
  delta_vs_frozen?: number | null;
  suppressed_by_mark_gate: boolean;
  note: string;
}

export interface ReassessExperimentReport {
  enabled: boolean;
  invert_enabled: boolean;
  rule?: string;
  n_open_watched: number;
  baseline?: ReassessVariantStats | null;
  n_triggered: number;
  n_helped: number;
  n_hurt: number;
  mean_delta_vs_frozen?: number | null;
  sum_delta_vs_frozen?: number | null;
  n_suppressed_by_mark_gate?: number;
  rows: ReassessExperimentRow[];
  note: string;
}

export interface DeferredLiveResponse {
  mode: string;
  observation_feed?: ObservationFeedSnapshot;
  session?: DeferredLiveSession | null;
  performance?: DeferredLivePerformance | null;
  reassess_experiment?: ReassessExperimentReport | null;
  armed: DeferredLiveArmed[];
  ledger: DeferredLiveLedger;
  /** As-of IC population for the session. Optional for older payloads. */
  asof_events?: AsOfSessionEvent[];
  /** Latest DecisionSurface snapshot. Presentation only. */
  last_surface?: DecisionSurfaceSnapshot | null;
}

export interface DecisionSurfaceSnapshot {
  as_of_unix?: number;
  last_ingest_unix?: number | null;
  open?: number;
  exited?: number;
  horizon?: number;
  last_tick_at?: number | null;
}