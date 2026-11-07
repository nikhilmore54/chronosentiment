/**
 * Snapshot parity helpers: compare Pi raw `/api/state` vs mapped OperatorState.
 * Presentation equality only — no behavioural recompute.
 */

import { mapOperatorState } from './mapState';
import { numOrNull } from './freshness';
import type { OperatorState } from './types';

export type ParityItem = {
  id: string;
  ok: boolean;
  expected: string;
  actual: string;
};

function asRec(v: unknown): Record<string, unknown> | null {
  return v && typeof v === 'object' && !Array.isArray(v) ? (v as Record<string, unknown>) : null;
}

export function runParityChecklist(
  raw: Record<string, unknown>,
  mapped?: OperatorState
): ParityItem[] {
  const state =
    mapped ??
    mapOperatorState(raw, {
      fetch_status: 'ok',
      fetch_error: null,
      fetched_at_ms: Date.now(),
    });
  const latest = asRec(raw.latest_card);
  const plan = asRec(raw.trade_plan);
  const ofRec = asRec(raw.order_flow_recommendation);
  const ofExit = asRec(raw.order_flow_last_exit);
  const marks = asRec(raw.of_candle_markers);
  const nse = asRec(raw.nse_operations);
  const next = asRec(nse?.next_session_recommendations);
  const rp607 = asRec(nse?.rp607_summary);

  const items: ParityItem[] = [];

  const push = (id: string, expected: unknown, actual: unknown) => {
    const e = expected === undefined || expected === null ? 'null' : String(expected);
    const a = actual === undefined || actual === null ? 'null' : String(actual);
    items.push({ id, ok: e === a, expected: e, actual: a });
  };

  push('authority_zero', 0, state.live_order_authority);
  push('policy_b', latest?.policy_b_action ?? null, state.crypto.policy_b);
  push('policy_c', latest?.policy_c_action ?? null, state.crypto.policy_c);
  push('plan_id', plan?.plan_id ?? null, state.crypto.trade_plan?.plan_id ?? null);
  push('plan_state', plan?.state ?? null, state.crypto.trade_plan?.state ?? null);
  push(
    'of_active',
    Boolean(ofRec?.active),
    Boolean(state.crypto.order_flow?.active)
  );
  push(
    'of_exit_present',
    Boolean(ofExit),
    Boolean(state.crypto.order_flow_last_exit)
  );
  push(
    'of_entry_bar',
    numOrNull(asRec(marks?.entry)?.bar_idx) ??
      numOrNull(asRec(ofRec?.entry_candle)?.bar_idx) ??
      null,
    state.crypto.of_candle_markers?.entry?.bar_idx ??
      state.crypto.order_flow?.entry_candle?.bar_idx ??
      null
  );
  push(
    'next_session_n',
    numOrNull(next?.n_recommendations) ??
      (Array.isArray(next?.recommendations) ? next!.recommendations.length : null),
    state.nse.n_recommendations ?? null
  );
  push(
    'next_session_of_n',
    numOrNull(next?.n_order_flow_supported),
    state.nse.n_order_flow_supported ?? null
  );
  push(
    'census_119',
    numOrNull(rp607?.census_total) ?? 119,
    state.nse.census_total ?? null
  );
  push(
    'classified_102',
    numOrNull(rp607?.classified_count),
    state.nse.classified_count ?? null
  );
  push(
    'quarantined_17',
    numOrNull(rp607?.quarantined_count),
    state.nse.quarantined_count ?? null
  );

  const rawRecs = Array.isArray(next?.recommendations) ? next!.recommendations : [];
  const mappedRecs = state.nse.recommendations ?? [];
  const topN = Math.min(5, rawRecs.length, mappedRecs.length);
  for (let i = 0; i < topN; i++) {
    const rr = asRec(rawRecs[i]);
    const mr = mappedRecs[i];
    push(
      `watch_${i}_symbol`,
      rr?.tradingsymbol ?? rr?.symbol ?? null,
      mr.tradingsymbol ?? mr.symbol ?? null
    );
    push(
      `watch_${i}_of`,
      Boolean(rr?.order_flow_supported),
      Boolean(mr.order_flow_supported)
    );
  }

  return items;
}

export function parityPassed(items: ParityItem[]): boolean {
  return items.every((i) => i.ok);
}
