export interface DivergenceCascadeEvent {
  tick: number;
  event: string;
}

export interface TimelineLaneEvent {
  tick: number;
  market: {
    price: number;
  };
  signal: {
    intent: string | null;
  };
  execution: {
    baseline_fill: boolean;
    perturbed_fill: boolean;
    missed_fill: boolean;
  };
  portfolio: {
    baseline_position: number;
    perturbed_position: number;
  };
}

export interface TimelineContract {
  divergence_anchor_tick: number | null;
  divergence_anchor_trade_id: string | null;
  divergence_reason: string | null;
  cascade: DivergenceCascadeEvent[];
  lanes: TimelineLaneEvent[];
}
