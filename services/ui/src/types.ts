export interface OrderState {
  order_id: string;
  status: 'NEW' | 'ACTIVE' | 'PARTIAL' | 'FILLED' | 'REJECTED';
  quantity_total: number;
  quantity_filled: number;
  quantity_remaining: number;
  queue_ahead: number;
  price: number;
  side: 'Buy' | 'Sell';
}

export interface PortfolioState {
  pnl: number;
  position: number;
}

export interface SystemState {
  orders: { [order_id: string]: OrderState };
  portfolio: PortfolioState;
  last_sequence_id: number;
}

export interface SimEvent {
  sequence_id: number;
  timestamp: number;
  type: string; // e.g., "MarketEvent", "OrderIntent", "PartialFill"
  parent_sequence_id: number | null;
  payload: any; // The actual event data, can be any specific event type
}

export interface TimelineEvent {
  timestamp: number;
  sequence_id: number;
  description: string;
}

export interface DecisionLayer {
  order_id: string;
  side: 'Buy' | 'Sell';
  price: number;
  quantity: number;
  timestamp: number;
}

export type ExecutionStep =
  | { type: 'OrderEnteredQueue'; queue_ahead: number; sequence_id: number; timestamp: number }
  | { type: 'QueueProgression'; queue_ahead: number; sequence_id: number; timestamp: number }
  | { type: 'PartialFillExecution'; filled_qty: number; price: number; sequence_id: number; timestamp: number }
  | { type: 'OrderFilledExecution'; sequence_id: number; timestamp: number }
  | { type: 'MarketEventExecution'; event_type: string; price: number; quantity: number; side: 'Buy' | 'Sell' | null; sequence_id: number; timestamp: number };

export interface OutcomeLayer {
  status: string; // NEW, ACTIVE, PARTIAL, FILLED
  filled_qty: number;
  remaining_qty: number;
  avg_price: number;
}

export interface TradeInspectorResponse {
  order_id: string;
  decision: DecisionLayer;
  execution: ExecutionStep[];
  outcome: OutcomeLayer;
  causal_chain?: SimEvent[] | null;
}
