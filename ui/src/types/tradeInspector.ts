export interface DecisionLayer {
  signal_timestamp: number;
  signal_type: string;
}

export interface TradeLeg {
  signal_time: number;
  fill_time: number;
  fill_price: number;
}

export interface ExecutionDeltaLayer {
  delay_ms: number;
  delay_ticks: number;
  slippage_bps: number;
  missed_fill: boolean;
  diverged: boolean;
}

export interface ExplanationRule {
  id: string; // The trace compiler outputs this as `id`
  type: string; // The trace compiler outputs this as `type`
  severity: "info" | "warning" | "critical";
  message: string;
}

export interface TimelineReferences {
  market_tick_index: number;
  signal_index: number;
  execution_index: number;
  portfolio_index: number;
}

export interface TradeDelta {
  trade_id: string;
  strategy: string;
  signal: DecisionLayer;
  baseline: TradeLeg | null;
  perturbed: TradeLeg | null;
  delta: ExecutionDeltaLayer;
  explanations: string[]; // List of Rule IDs
  timeline_refs: TimelineReferences;
}

export interface TradeInspectorViewModel {
  trade_delta: TradeDelta;
  rules_map: Record<string, ExplanationRule>;
}
