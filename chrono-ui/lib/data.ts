// Data types matching the real JSON ledger structures

export interface DecisionState {
  trend: string;
  momentum: string;
  volatility: string;
  input_schema: string[];
  state_hash: string;
}

export interface Decision {
  decision_id: string;
  instrument: string;
  decision_time: string;
  state: DecisionState;
  action: "LONG" | "SHORT" | "NO_TRADE";
  policy_id: string;
  policy_artifact_sha256: string;
  engine_version: string;
  horizon_days: number;
  sealed_status: string;
  paper_only: boolean;
}

export interface Observation {
  decision_id: string;
  observation_time: string;
  observation_status: string;
  realized_return: number;
  value_long: number;
  value_short: number;
  value_no_trade: number;
}

export interface HistoricalLedger {
  contract_id: string;
  policy_id: string;
  policy_artifact_sha256: string;
  paper_only: boolean;
  path_kind: string;
  search_three_authorized: boolean;
  regime_persistence_experiment_authorized: boolean;
  decisions: Decision[];
  observations: Observation[];
}

export interface ExecutionIntent {
  decision_id: string;
  instrument: string;
  decision_time: string;
  action: "LONG" | "SHORT" | "NO_TRADE";
  entry_price: number;
  target_pct: number;
  target_price: number;
  stop_pct: null;
  stop_price: null;
  max_holding_sessions: number;
  target_source: string;
  execution_contract: string;
  sealed_at_t: boolean;
  intent_hash: string;
}

export interface ExecutionTick {
  instrument: string;
  requested_clock: string;
  decision_time: string;
  decision_id: string;
  direction: "LONG" | "SHORT" | "NO_TRADE";
  entry_price: number;
  target_pct: number;
  target_price: number;
  target_hit: boolean;
  target_hit_session: number | null;
  exit_price: number;
  exit_reason: "TARGET" | "HORIZON";
  holding_sessions: number;
  decision_value: number;
  peeked_returns_at_seal: boolean;
}

export interface ExecutionReport {
  path_kind: string;
  execution_contract: string;
  target_source: string;
  target_pct: number;
  max_holding_sessions: number;
  stop_exit_authorized: boolean;
  target_path_optimization_authorized: boolean;
  n_decisions: number;
  n_exits: number;
  n_target: number;
  n_horizon: number;
  n_no_trade: number;
  peeked_returns_at_seal: boolean;
  prospective_cohort_mutated: boolean;
  statistical_backtest: boolean;
  ticks: ExecutionTick[];
}

export interface PE2Exit {
  decision_id: string;
  target_hit: boolean;
  target_hit_session: number | null;
  exit_price: number;
  exit_reason: "TARGET" | "HORIZON";
  holding_sessions: number;
  exit_time: string;
  decision_value: number;
  trigger_type: string;
  trigger_session: number;
  trigger_timestamp: string;
}

export interface PE2Record {
  instrument: string;
  requested_clock: string;
  certified_t: string;
  decision: Decision;
  intent: ExecutionIntent;
  exit: PE2Exit;
}

export interface PE2Ledger {
  path_kind: string;
  execution_contract: string;
  execution_contract_label: string;
  requested_clock: string;
  certified_t: string;
  target_pct: number;
  max_holding_sessions: number;
  n_decisions: number;
  n_execution_intents: number;
  n_target: number;
  n_horizon: number;
  n_gap_through: number;
  n_high_reached: number;
  n_low_reached: number;
  n_session_close: number;
  determinism_pass: boolean;
  lookahead_clean: boolean;
  poison_test_pass: boolean;
  peeked_returns_at_seal: boolean;
  prospective_cohort_mutated: boolean;
  protected_artifacts_mutated: boolean;
  statistical_backtest: boolean;
  lifecycle_validation: string;
  records: PE2Record[];
}

// ─── P.E.3 Historical Replay — coralys-exec-v0 ───────────────────────────────

export interface PE3Exit {
  decision_id: string;
  target_hit: boolean;
  target_hit_session: number | null;
  exit_price: number | null;
  exit_reason: "TARGET" | "HORIZON" | "STOP" | "AMBIGUOUS" | "NO_TRADE" | "OBSERVING";
  holding_sessions: number | null;
  exit_time: string | null;
  decision_value: number | null;
  trigger_type: string | null;
  trigger_session: number | null;
  trigger_timestamp: string | null;
  trigger_price: number | null;
  execution_price: number | null;
}

export interface PE3Record {
  instrument: string;
  requested_clock: string;
  certified_t: string;
  decision: Decision;
  coralys_model_id: string;
  coralys_model_version: string;
  coralys_artifact_hash: string;
  coralys_intent_hash: string | null;
  atr_14_at_t: number | null;
  coralys_target_pct: number | null;
  coralys_risk_pct: number | null;
  coralys_tmv_state: string | null;
  intent: ExecutionIntent;
  exit: PE3Exit;
  pe3_eligible: boolean;
  exclusion_reason: string | null;
  determinism_pass: boolean;
  lookahead_clean: boolean;
  poison_test_pass: boolean;
  learning_scope: string;
  retrospective_characterization: boolean;
}

export interface PE3Ledger {
  path_kind: string;
  execution_contract: string;
  execution_contract_label: string;
  coralys_model_id: string;
  coralys_model_version: string;
  coralys_artifact_hash: string;
  requested_clock: string;
  certified_t: string;
  max_holding_sessions: number;
  n_decisions: number;
  n_pe3_eligible: number;
  n_excluded_no_atr: number;
  n_target: number;
  n_risk: number;
  n_horizon: number;
  n_no_trade: number;
  n_ambiguous: number;
  determinism_pass: boolean;
  lookahead_clean: boolean;
  poison_test_pass: boolean;
  peeked_returns_at_seal: boolean;
  statistical_backtest: boolean;
  retrospective_characterization: boolean;
  lifecycle_validation: string;
  records: PE3Record[];
}

// Live execution ledger — price-complete schema
export type LifecycleState = "DECISION_ONLY" | "READY_TO_ENTER" | "ACTIVE" | "EXITED";

export interface DecisionSealRecord {
  decision_timestamp: string;
  certified_session: string;
  policy_id: string;
  policy_artifact_sha256: string;
  market_state: { trend: string; momentum: string; volatility: string };
  decision_reference_price: number | null;
  note?: string;
}

export interface ExecutionSealRecord {
  entry_timestamp: string;
  entry_price: number;
  entry_source: string;
  target_price: number;
  target_pct: number;
  risk_boundary: number | null;
  risk_pct: number | null;
  maximum_hold_sessions: number;
  execution_contract: string;
  coralys_execution_artifact: string | null;
  intent_hash: string;
  sealed_at: string;
}

export interface RiskBoundaryChange {
  session: string;
  previous_boundary: number;
  new_boundary: number;
  reason: string;
}

export interface MonitoringRecord {
  sessions_elapsed: number;
  current_session: string;
  last_certified_state: { trend: string; momentum: string; volatility: string };
  risk_boundary_history: RiskBoundaryChange[];
}

export interface TriggerOHLC {
  open: number;
  high: number;
  low: number;
  close: number;
}

export interface ExitRecord {
  exit_timestamp: string;
  exit_price: number;
  exit_reason: "TARGET" | "RISK" | "HORIZON";
  trigger_type: "HIGH_REACHED" | "LOW_REACHED" | "GAP_THROUGH" | "SESSION_CLOSE" | "RISK_BOUNDARY";
  trigger_session: string;
  trigger_timestamp: string;
  trigger_session_ohlc: TriggerOHLC | null;
  holding_sessions: number;
  realized_return: number;
  decision_value: number;
}

export interface LivePosition {
  position_id: string;
  decision_id: string;
  instrument: string;
  market: string;
  venue: string;
  currency: string;
  direction: "LONG" | "SHORT" | "NO_TRADE";
  lifecycle_state: LifecycleState;
  decision_seal: DecisionSealRecord;
  execution_seal: ExecutionSealRecord | null;
  monitoring: MonitoringRecord | null;
  exit_record: ExitRecord | null;
  next_eligible_session?: string;
  reason_pending?: string;
}

export interface LiveExecutionLedger {
  schema_version: string;
  ledger_kind: string;
  description: string;
  paper_only: boolean;
  execution_contract: string;
  coralys_execution_model: string;
  created_at: string;
  last_updated: string;
  positions: LivePosition[];
}

// Helper functions
export function formatDate(isoString: string): string {
  const d = new Date(isoString);
  return d.toLocaleDateString("en-GB", {
    day: "2-digit",
    month: "short",
    year: "numeric",
  });
}

export function formatDateTime(isoString: string): string {
  const d = new Date(isoString);
  return d.toLocaleDateString("en-GB", {
    day: "2-digit",
    month: "short",
    year: "numeric",
  }) + " " + d.toLocaleTimeString("en-GB", { hour: "2-digit", minute: "2-digit", timeZoneName: "short" });
}

export function shortHash(hash: string): string {
  return hash.slice(0, 8) + "…" + hash.slice(-4);
}

export function formatReturn(r: number): string {
  const pct = (r * 100).toFixed(2);
  return (r >= 0 ? "+" : "") + pct + "%";
}

export function formatPrice(p: number): string {
  return "₹" + p.toLocaleString("en-IN", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

export function stateLabel(state: DecisionState): string {
  return `${state.trend} / ${state.momentum}`;
}

export function actionBadgeClass(action: string): string {
  if (action === "LONG") return "badge badge-long";
  if (action === "SHORT") return "badge badge-short";
  return "badge badge-no-trade";
}

export function exitBadgeClass(reason: string): string {
  if (reason === "TARGET") return "badge badge-target";
  return "badge badge-horizon";
}