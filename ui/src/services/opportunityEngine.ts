/**
 * Opportunity Engine — Intelligence Contract v1 (TypeScript port)
 * ================================================================
 * Direct port of the Rust `intraday_classification` module.
 * Uses ONLY information available at each checkpoint (information-safe).
 *
 * Entry-time: direction + oqs + h60_classification (NO momentum_persistence)
 * H120:       h120_ret + mfe_h120 (LONG WAIT-MID only)
 *
 * Thresholds (frozen):
 *   OQS_LONG_HIGH  = 65
 *   OQS_LONG_MID   = 40
 *   OQS_SHORT_HIGH = 50
 *   FAV_ADV_THRESHOLD = 0.002
 *   MFE_FLOOR         = 0.001
 */

// ── Frozen constants ──────────────────────────────────────────────────────────

export const OQS_LONG_HIGH  = 65;
export const OQS_LONG_MID   = 40;
export const OQS_SHORT_HIGH = 50;
export const FAV_ADV_THRESHOLD = 0.002;
export const MFE_FLOOR         = 0.001;

// ── State vocabulary ──────────────────────────────────────────────────────────

export type OpportunityState =
  | 'ENTER'
  | 'WAIT-HIGH'
  | 'WAIT-MID'
  | 'WAIT-LOW'
  | 'AVOID'
  | 'ENTER-LATE'
  | 'WAIT-LATE'
  | 'AVOID-LATE'
  | 'UNKNOWN';

export type Action = 'ACT' | 'MONITOR' | 'AVOID' | 'UPGRADE' | 'CONTINUE';
export type Confidence = 'HIGH' | 'MODERATE' | 'LOW';
export type Checkpoint = 'ENTRY' | 'H60' | 'H120' | 'H180';
export type ReturnBucket = 'FAV' | 'FLAT' | 'ADV';

// ── Historical performance reference (frozen from v0.9) ───────────────────────

interface HistRef { win: number; pf: number | null; med: number; }

const HIST: Record<string, HistRef> = {
  'SHORT:ENTER':      { win: 0.717, pf: 13.35, med:  0.0076 },
  'SHORT:WAIT-HIGH':  { win: 0.775, pf: 99.0,  med:  0.0056 },
  'SHORT:WAIT-LOW':   { win: 0.044, pf: 0.04,  med: -0.0030 },
  'SHORT:AVOID':      { win: 0.261, pf: 0.23,  med: -0.0032 },
  'LONG:ENTER':       { win: 0.750, pf: 19.46, med:  0.0061 },
  'LONG:WAIT-HIGH':   { win: 0.824, pf: 72.36, med:  0.0066 },
  'LONG:WAIT-MID':    { win: 0.595, pf: 10.06, med:  0.0026 },
  'LONG:WAIT-LOW':    { win: 0.283, pf: 0.67,  med: -0.0023 },
  'LONG:AVOID':       { win: 0.116, pf: 0.08,  med: -0.0065 },
  'LONG:ENTER-LATE':  { win: 0.696, pf: 11.67, med:  0.0046 },
  'LONG:WAIT-LATE':   { win: 0.500, pf: 6.47,  med:  0.0018 },
  'LONG:AVOID-LATE':  { win: 0.000, pf: null,  med:  0.0002 },
};

export function getHistRef(direction: string, state: OpportunityState): HistRef | null {
  return HIST[`${direction}:${state}`] ?? null;
}

// ── Bucket helper ─────────────────────────────────────────────────────────────

export function bucket(ret: number): ReturnBucket {
  if (ret > FAV_ADV_THRESHOLD)  return 'FAV';
  if (ret < -FAV_ADV_THRESHOLD) return 'ADV';
  return 'FLAT';
}

// ── Entry-time classifier (mirrors Rust classify_at_entry) ────────────────────

export function classifyAtEntry(
  direction: string,
  oqs: number,
  h60Classification: string,
): OpportunityState {
  if (h60Classification === 'ENTER') return 'ENTER';
  if (h60Classification === 'AVOID') return 'AVOID';
  // WAIT branch
  if (direction === 'SHORT') {
    return oqs >= OQS_SHORT_HIGH ? 'WAIT-HIGH' : 'WAIT-LOW';
  }
  // LONG
  if (oqs >= OQS_LONG_HIGH) return 'WAIT-HIGH';
  if (oqs >= OQS_LONG_MID)  return 'WAIT-MID';
  return 'WAIT-LOW';
}

// ── H120 reassessment (mirrors Rust reassess_at_h120) ────────────────────────

export function reassessAtH120(
  direction: string,
  entryState: OpportunityState,
  h120Ret: number | null | undefined,
  mfeH120: number | null | undefined,
): OpportunityState {
  if (direction !== 'LONG' || entryState !== 'WAIT-MID') return entryState;
  if (h120Ret == null) return 'WAIT-LATE';
  if (mfeH120 != null && mfeH120 < MFE_FLOOR) return 'AVOID-LATE';
  const b = bucket(h120Ret);
  if (b === 'FAV') return 'ENTER-LATE';
  if (b === 'ADV') return 'AVOID-LATE';
  return 'WAIT-LATE';
}

// ── Action resolver ───────────────────────────────────────────────────────────

export interface CheckpointResult {
  checkpoint: Checkpoint;
  action: Action;
  state: OpportunityState;
  confidence: Confidence;
  horizon: string;
  why: string;
  risk: string;
  hist: HistRef | null;
}

export function resolveEntryAction(
  direction: string,
  state: OpportunityState,
  oqs: number,
): CheckpointResult {
  const hist = getHistRef(direction, state);
  switch (state) {
    case 'ENTER':
      return {
        checkpoint: 'ENTRY', action: 'ACT', state, confidence: 'HIGH',
        horizon: direction === 'SHORT' ? 'Act within 5-hour session' : 'Act now; opportunity peaks around H60',
        why: `H15 and H60 both favourable — strong intraday ${direction} signal.`,
        risk: direction === 'SHORT'
          ? 'Avg loss −0.89% if wrong; monitor for early adverse reversal.'
          : 'Return decays after H60; do not hold expecting daily continuation.',
        hist,
      };
    case 'WAIT-HIGH':
      return {
        checkpoint: 'ENTRY', action: 'ACT', state, confidence: 'HIGH',
        horizon: direction === 'SHORT' ? 'Act within 5-hour session' : 'Act now; hold through session',
        why: direction === 'SHORT'
          ? 'Momentum persistence — price moving favourably for majority of session.'
          : `OQS ${oqs} — high-quality LONG with sustained path.`,
        risk: direction === 'SHORT'
          ? 'Avg loss only −0.20% if wrong; PF >99x historically.'
          : 'Daily return stronger than H300; allow full session to develop.',
        hist,
      };
    case 'WAIT-MID':
      return {
        checkpoint: 'ENTRY', action: 'MONITOR', state, confidence: 'MODERATE',
        horizon: 'Reassess at H120 (2 hours)',
        why: `OQS ${oqs} — developing LONG opportunity, insufficient early signal.`,
        risk: 'Winners and losers indistinguishable before H120; do not act at entry.',
        hist,
      };
    case 'WAIT-LOW':
      return {
        checkpoint: 'ENTRY', action: 'AVOID', state, confidence: 'LOW',
        horizon: 'No action required',
        why: 'Momentum persistence low — price not moving favourably during session.',
        risk: 'PF 0.04–0.67x historically; acting here destroys value.',
        hist,
      };
    case 'AVOID':
      return {
        checkpoint: 'ENTRY', action: 'AVOID', state, confidence: 'HIGH',
        horizon: 'No action required',
        why: 'H15 and H60 both adverse — confirmed negative intraday path.',
        risk: 'PF 0.08–0.23x historically; strong avoidance signal.',
        hist,
      };
    default:
      return {
        checkpoint: 'ENTRY', action: 'AVOID', state: 'UNKNOWN', confidence: 'LOW',
        horizon: 'Unknown', why: 'Unclassified state.', risk: 'No historical reference.',
        hist: null,
      };
  }
}

export function resolveH120Action(
  direction: string,
  entryState: OpportunityState,
  h120State: OpportunityState,
  h120Ret: number | null | undefined,
): CheckpointResult {
  const hist = getHistRef(direction, h120State);
  const retStr = h120Ret != null ? `${(h120Ret * 100).toFixed(2)}%` : '—';

  if (entryState === 'WAIT-MID') {
    switch (h120State) {
      case 'ENTER-LATE':
        return {
          checkpoint: 'H120', action: 'UPGRADE', state: h120State, confidence: 'HIGH',
          horizon: 'Act now; H120 confirms opportunity',
          why: `H120 return ${retStr} — FAV path at 2-hour mark upgrades WAIT-MID to ENTER-LATE.`,
          risk: 'PF 11.67x, 70% win historically; avg loss small.',
          hist,
        };
      case 'WAIT-LATE':
        return {
          checkpoint: 'H120', action: 'CONTINUE', state: h120State, confidence: 'MODERATE',
          horizon: 'Continue monitoring; reassess at H180',
          why: `H120 return ${retStr} — flat path, opportunity still developing.`,
          risk: 'WAIT-LATE still positive (PF 6.47x, 50% win); do not exit prematurely.',
          hist,
        };
      case 'AVOID-LATE':
        return {
          checkpoint: 'H120', action: 'AVOID', state: h120State, confidence: 'MODERATE',
          horizon: 'Consider exiting or reducing',
          why: `H120 return ${retStr} — adverse path at 2-hour mark.`,
          risk: 'N=1 historically; avoidance rule not yet fully evidenced.',
          hist,
        };
    }
  }

  // Non-WAIT-MID pass-through
  if (h120State === 'ENTER' || h120State === 'WAIT-HIGH') {
    return {
      checkpoint: 'H120', action: 'ACT', state: h120State, confidence: 'HIGH',
      horizon: 'Continue; H120 confirms',
      why: `H120 return ${retStr} — path continuing favourably.`,
      risk: 'On track with entry assessment.',
      hist,
    };
  }
  return {
    checkpoint: 'H120', action: 'AVOID', state: h120State, confidence: 'HIGH',
    horizon: 'No action required',
    why: `H120 return ${retStr} — path confirmed adverse.`,
    risk: 'No change from entry assessment.',
    hist,
  };
}

// ── Dataset record type ───────────────────────────────────────────────────────

export interface OpportunityRecord {
  decision_id: string;
  date: string;
  instrument: string;
  direction: string;
  opportunity_dimensions: { opportunity_quality_score: number };
  path_5m: {
    h60_classification: string;
    momentum_persistence?: number;
    h15_ret?: number | null;
    h30_ret?: number | null;
    h60_ret?: number | null;
    h120_ret?: number | null;
    h180_ret?: number | null;
    h300_ret?: number | null;
    mfe_h60?: number | null;
    mfe_h120?: number | null;
  };
  outcome?: string;
  pnl?: number | null;
}

// ── Enriched decision (dataset record + computed states) ─────────────────────

export interface EnrichedDecision {
  record: OpportunityRecord;
  entryState: OpportunityState;
  h120State: OpportunityState;
  entryAction: CheckpointResult;
  h120Action: CheckpointResult;
  // Convenience fields for search/filter
  ticker: string;
  date: string;
  direction: string;
  oqs: number;
  h60Class: string;
  outcome: string;
  pnl: number | null;
  h300Ret: number | null;
}

// ── Enrich a single record ────────────────────────────────────────────────────

export function enrichRecord(record: OpportunityRecord): EnrichedDecision {
  const direction = record.direction;
  const oqs = record.opportunity_dimensions.opportunity_quality_score;
  const h60Class = record.path_5m.h60_classification;
  const h120Ret = record.path_5m.h120_ret ?? null;
  const mfeH120 = record.path_5m.mfe_h120 ?? null;

  const entryState = classifyAtEntry(direction, oqs, h60Class);
  const h120State  = reassessAtH120(direction, entryState, h120Ret, mfeH120);

  const entryAction = resolveEntryAction(direction, entryState, oqs);
  const h120Action  = resolveH120Action(direction, entryState, h120State, h120Ret);

  return {
    record,
    entryState,
    h120State,
    entryAction,
    h120Action,
    ticker:    record.instrument ?? record.decision_id,
    date:      record.date,
    direction: record.direction,
    oqs,
    h60Class,
    outcome:   record.outcome ?? '—',
    pnl:       record.pnl ?? null,
    h300Ret:   record.path_5m.h300_ret ?? null,
  };
}

// ── Search / filter ───────────────────────────────────────────────────────────

export interface SearchFilters {
  ticker?: string;
  direction?: string;
  entryState?: string;
  h120State?: string;
  action?: string;
  date?: string;
}

export function searchDecisions(
  decisions: EnrichedDecision[],
  filters: SearchFilters,
): EnrichedDecision[] {
  return decisions.filter(d => {
    if (filters.ticker) {
      const q = filters.ticker.toUpperCase();
      if (!d.ticker.toUpperCase().includes(q)) return false;
    }
    if (filters.direction && d.direction !== filters.direction) return false;
    if (filters.entryState && d.entryState !== filters.entryState) return false;
    if (filters.h120State && d.h120State !== filters.h120State) return false;
    if (filters.action && d.entryAction.action !== filters.action) return false;
    if (filters.date && d.date !== filters.date) return false;
    return true;
  });
}

// ── State colour coding ───────────────────────────────────────────────────────

export function stateColor(state: OpportunityState): string {
  switch (state) {
    case 'ENTER':      return '#22c55e';  // green
    case 'WAIT-HIGH':  return '#3b82f6';  // blue
    case 'WAIT-MID':   return '#f59e0b';  // amber
    case 'ENTER-LATE': return '#10b981';  // emerald
    case 'WAIT-LATE':  return '#6366f1';  // indigo
    case 'WAIT-LOW':   return '#94a3b8';  // slate
    case 'AVOID':      return '#ef4444';  // red
    case 'AVOID-LATE': return '#f97316';  // orange
    default:           return '#64748b';  // muted
  }
}

export function actionColor(action: Action): string {
  switch (action) {
    case 'ACT':      return '#22c55e';
    case 'UPGRADE':  return '#10b981';
    case 'MONITOR':  return '#f59e0b';
    case 'CONTINUE': return '#6366f1';
    case 'AVOID':    return '#ef4444';
    default:         return '#64748b';
  }
}

export function confidenceColor(confidence: Confidence): string {
  switch (confidence) {
    case 'HIGH':     return '#22c55e';
    case 'MODERATE': return '#f59e0b';
    case 'LOW':      return '#94a3b8';
    default:         return '#64748b';
  }
}