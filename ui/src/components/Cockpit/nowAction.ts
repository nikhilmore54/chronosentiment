/**
 * Decision-to-Action join for the live Cockpit.
 *
 * Presentation only. Does not decide ACT/NO TRADE, does not fill, and does
 * not mark. It projects IC v1 DecisionBrief + Deferred Live ledger fields.
 * Evidence fields are copied from the brief / portfolio context — no new
 * scoring and no reassessment.
 */

import type { DecisionBrief, DecisionPortfolioContext, DeferredLivePosition, DeferredLiveResponse } from '../../types/decisionBrief';

export type NowActionKind = 'ACT' | 'NO TRADE';
export type NowPaperState = 'WAITING FILL' | 'OPEN' | 'TARGET' | 'STOP' | 'HORIZON' | 'NOT IN BOOK' | 'NONE';
export type NowMarkKind = 'MARK' | 'REALIZED';
/** Ledger projection for the Recommendation column. Not a new trading rule. */
export type NowRecommendation = 'HOLD' | 'NOT RECOMMENDED' | 'TARGET' | 'STOP' | 'HORIZON' | '—';
/** Market condition from current mark-to-fill. Not a recommendation and not a lifecycle event. */
export type NowCondition = 'ADVERSE' | 'FAVOURABLE' | 'NEUTRAL' | '—';

export interface NowActionRow {
  decisionId: string;
  ticker: string;
  direction: string;
  action: NowActionKind;
  entryState: string;
  paperState: NowPaperState;
  recommendation: NowRecommendation;
  paperEntry: number | null;
  mark: number | null;
  markKind: NowMarkKind | null;
  target: number | null;
  risk: number | null;
  /** Remaining room to frozen TARGET as a fraction of fill. OPEN only. */
  toTargetPct: number | null;
  /** Remaining room to frozen STOP as a fraction of fill. OPEN only. */
  toStopPct: number | null;
  snapUnix: number | null;
  lastTickAt: number | null;
  currentPrice: number | null;
  asofDecisionId: string | null;
  oqs: number | null;
  oqsRank: number | null;
  candidatesOnDate: number | null;
  histWin: number | null;
  histPf: number | null;
  histMed: number | null;
  entryWhy: string | null;
  entryHorizon: string | null;
  entryRisk: string | null;
  riskDistancePct: number | null;
  enteredAt: number | null;
  exitAt: number | null;
  exitPrice: number | null;
}

function isAct(action: string | null | undefined): boolean {
  return (action ?? '').toUpperCase() === 'ACT';
}

function paperStateFromPosition(pos: DeferredLivePosition | undefined): NowPaperState | null {
  if (!pos) return null;
  const reason = (pos.walk?.exit?.reason ?? pos.paper?.exit_reason ?? '').toUpperCase();
  if (reason === 'TARGET') return 'TARGET';
  if (reason === 'STOP') return 'STOP';
  if (reason === 'HORIZON') return 'HORIZON';
  return 'OPEN';
}

function signedMark(
  direction: string | undefined,
  entry: number | null | undefined,
  last: number | null | undefined,
): number | null {
  if (entry == null || entry === 0 || last == null) return null;
  const raw = (last - entry) / entry;
  return (direction ?? '').toUpperCase() === 'SHORT' ? -raw : raw;
}

function oneBriefPerTicker(briefs: DecisionBrief[]): DecisionBrief[] {
  const best = new Map<string, DecisionBrief>();
  for (const d of briefs) {
    const prev = best.get(d.ticker);
    if (!prev || d.oqs > prev.oqs || (d.oqs === prev.oqs && d.id < prev.id)) {
      best.set(d.ticker, d);
    }
  }
  return [...best.values()];
}

function emptyEvidence(): Pick<NowActionRow,
  'oqs' | 'oqsRank' | 'candidatesOnDate' | 'histWin' | 'histPf' | 'histMed' |
  'entryWhy' | 'entryHorizon' | 'entryRisk' | 'riskDistancePct' |
  'enteredAt' | 'exitAt' | 'exitPrice' | 'toTargetPct' | 'toStopPct'
> {
  return {
    oqs: null,
    oqsRank: null,
    candidatesOnDate: null,
    histWin: null,
    histPf: null,
    histMed: null,
    entryWhy: null,
    entryHorizon: null,
    entryRisk: null,
    riskDistancePct: null,
    enteredAt: null,
    exitAt: null,
    exitPrice: null,
    toTargetPct: null,
    toStopPct: null,
  };
}

/** Remaining room to a frozen barrier as a fraction of fill. Presentation only. */
export function remainingToBarrierPct(
  direction: string | undefined,
  fill: number | null | undefined,
  current: number | null | undefined,
  barrier: number | null | undefined,
  kind: 'stop' | 'target',
): number | null {
  if (fill == null || fill === 0 || current == null || barrier == null) return null;
  const short = (direction ?? '').toUpperCase() === 'SHORT';
  const remaining = kind === 'stop'
    ? (short ? barrier - current : current - barrier)
    : (short ? current - barrier : barrier - current);
  return remaining / fill;
}

export function recommendationFromPaper(
  action: NowActionKind,
  paperState: NowPaperState,
): NowRecommendation {
  if (action === 'NO TRADE' || paperState === 'NONE') return 'NOT RECOMMENDED';
  if (paperState === 'OPEN') return 'HOLD';
  if (paperState === 'TARGET' || paperState === 'STOP') return paperState;
  // HORIZON is a lifecycle outcome, not a recommendation
  return '—';
}

/** Signed mark-to-fill only. Does not run path-shape, INVERT, or CS-P-001-C. */
export function conditionFromMark(mark: number | null | undefined): NowCondition {
  if (mark == null) return '—';
  if (mark < 0) return 'ADVERSE';
  if (mark > 0) return 'FAVOURABLE';
  return 'NEUTRAL';
}

export function conditionCaption(condition: NowCondition): string {
  switch (condition) {
    case 'ADVERSE':
      return 'CAUTION — adverse conditions';
    case 'FAVOURABLE':
      return 'FAVOURABLE — position moving with thesis';
    case 'NEUTRAL':
      return 'NEUTRAL — mark-to-fill flat';
    default:
      return '—';
  }
}

function barrierDistances(pos: DeferredLivePosition | undefined, paperState: NowPaperState): {
  toTargetPct: number | null;
  toStopPct: number | null;
} {
  if (!pos || paperState !== 'OPEN') return { toTargetPct: null, toStopPct: null };
  const fill = pos.paper?.paper_entry_price ?? pos.walk?.entry;
  const current = pos.current_price;
  const target = pos.paper?.paper_target ?? pos.walk?.target;
  const stop = pos.walk?.stop ?? pos.paper?.candidate_stop_price ?? pos.paper?.paper_risk;
  const direction = pos.paper?.direction ?? pos.walk?.direction;
  return {
    toTargetPct: remainingToBarrierPct(direction, fill, current, target, 'target'),
    toStopPct: remainingToBarrierPct(direction, fill, current, stop, 'stop'),
  };
}

/** Smaller remaining room first. Nulls last. Does not instruct an exit. */
export function compareNearestStop(a: NowActionRow, b: NowActionRow): number {
  const sa = a.toStopPct;
  const sb = b.toStopPct;
  if (sa == null && sb == null) return a.ticker.localeCompare(b.ticker);
  if (sa == null) return 1;
  if (sb == null) return -1;
  if (sa !== sb) return sa - sb;
  return a.ticker.localeCompare(b.ticker);
}

export function compareNearestTarget(a: NowActionRow, b: NowActionRow): number {
  const ta = a.toTargetPct;
  const tb = b.toTargetPct;
  if (ta == null && tb == null) return a.ticker.localeCompare(b.ticker);
  if (ta == null) return 1;
  if (tb == null) return -1;
  if (ta !== tb) return ta - tb;
  return a.ticker.localeCompare(b.ticker);
}

export interface TodayRecommendations {
  sessionDate: string | null;
  nAct: number;
  nActOpen: number;
  nLong: number;
  nShort: number;
  nNotAct: number;
  nTarget: number;
  nStop: number;
  nHorizon: number;
  nAdverse: number;
  nFavourable: number;
  nNeutral: number;
  meanOpenMark: number | null;
  lastObsUnix: number | null;
  attention: NowActionRow[];
  opportunity: NowActionRow[];
  notAct: NowActionRow[];
}

export function buildTodayRecommendations(
  live: DeferredLiveResponse | null | undefined,
  openRows: NowActionRow[],
  noTradeRows: NowActionRow[] = [],
): TodayRecommendations | null {
  if (!live) return null;
  const summary = buildSessionSummary(live);
  const nLong = openRows.filter(r => (r.direction ?? '').toUpperCase() === 'LONG').length;
  const nShort = openRows.filter(r => (r.direction ?? '').toUpperCase() === 'SHORT').length;
  const nAct = Math.max(openRows.length + (summary?.nTarget ?? 0) + (summary?.nStop ?? 0) + (summary?.nHorizon ?? 0), live.performance?.n_entered ?? openRows.length);
  const nAdverse = openRows.filter(r => conditionFromMark(r.mark) === 'ADVERSE').length;
  const nFavourable = openRows.filter(r => conditionFromMark(r.mark) === 'FAVOURABLE').length;
  const nNeutral = openRows.filter(r => conditionFromMark(r.mark) === 'NEUTRAL').length;
  return {
    sessionDate: summary?.date ?? live.session?.date ?? null,
    nAct,
    nActOpen: openRows.length,
    nLong,
    nShort,
    nNotAct: noTradeRows.length,
    nTarget: summary?.nTarget ?? 0,
    nStop: summary?.nStop ?? 0,
    nHorizon: summary?.nHorizon ?? 0,
    nAdverse,
    nFavourable,
    nNeutral,
    meanOpenMark: live.performance?.mean_open_mark ?? null,
    lastObsUnix: summary?.lastObsUnix ?? null,
    attention: [...openRows].sort(compareNearestStop).slice(0, 3),
    opportunity: [...openRows].sort(compareNearestTarget).slice(0, 3),
    notAct: noTradeRows,
  };
}

function meanOf(xs: number[]): number | null {
  if (xs.length === 0) return null;
  return xs.reduce((a, b) => a + b, 0) / xs.length;
}

/** Ledger-driven paper book. Not a portfolio optimizer and not a new classifier. */
export interface PaperPortfolio {
  nOpen: number;
  nTarget: number;
  nStop: number;
  nHorizon: number;
  nClosed: number;
  nLong: number;
  nShort: number;
  nWinClosed: number;
  meanOpenMark: number | null;
  meanLongMark: number | null;
  meanShortMark: number | null;
  meanClosedReturn: number | null;
}

export function buildPaperPortfolio(
  live: DeferredLiveResponse | null | undefined,
  openRows: NowActionRow[],
): PaperPortfolio | null {
  if (!live) return null;
  const summary = buildSessionSummary(live);
  const perf = live.performance;
  const nOpen = perf?.n_open ?? summary?.nOpen ?? openRows.length;
  const nTarget = perf?.n_target ?? summary?.nTarget ?? 0;
  const nStop = perf?.n_stop ?? summary?.nStop ?? 0;
  const nHorizon = perf?.n_horizon ?? summary?.nHorizon ?? 0;
  const longs = openRows.filter(r => (r.direction ?? '').toUpperCase() === 'LONG');
  const shorts = openRows.filter(r => (r.direction ?? '').toUpperCase() === 'SHORT');
  const nClosed = nTarget + nStop + nHorizon;
  return {
    nOpen,
    nTarget,
    nStop,
    nHorizon,
    nClosed,
    nLong: longs.length,
    nShort: shorts.length,
    nWinClosed: nClosed === 0 ? 0 : (perf?.n_win_closed ?? 0),
    meanOpenMark: perf?.mean_open_mark ?? meanOf(openRows.map(r => r.mark).filter((v): v is number => v != null)),
    meanLongMark: meanOf(longs.map(r => r.mark).filter((v): v is number => v != null)),
    meanShortMark: meanOf(shorts.map(r => r.mark).filter((v): v is number => v != null)),
    meanClosedReturn: nClosed === 0 ? null : (perf?.mean_closed_return ?? null),
  };
}

export type LifecycleStoryKind = 'PAPER_ENTER' | 'TARGET' | 'STOP' | 'HORIZON';

/** Chronological ledger story. PAPER_ENTER / TARGET / STOP / HORIZON only. */
export interface LifecycleStoryItem {
  unix: number;
  kind: LifecycleStoryKind;
  ticker: string | null;
  headline: string;
  detail: string;
  selectId: string | null;
  realizedReturn: number | null;
}

/**
 * Ledger events only. Does not use marks, CAUTION, path-shape, or remaining-to-barrier.
 * Collapses same-unix PAPER_ENTER so the 54-name open is one story row.
 */
export function buildLifecycleStory(live: DeferredLiveResponse | null | undefined): LifecycleStoryItem[] {
  if (!live) return [];
  const items: LifecycleStoryItem[] = [];
  const enterByUnix = new Map<number, DeferredLivePosition[]>();
  const emittedExit = new Set<string>();

  for (const pos of live.ledger?.positions ?? []) {
    const paper = pos.paper;
    const ticker = paper?.ticker ?? '';
    const selectId = paper?.decision_id || ticker;
    const reason = (pos.walk?.exit?.reason ?? paper?.exit_reason ?? '').toUpperCase();
    for (const ev of pos.events ?? []) {
      const kind = (ev.kind ?? '').toUpperCase();
      if (kind === 'PAPER_ENTER') {
        const list = enterByUnix.get(ev.unix) ?? [];
        list.push(pos);
        enterByUnix.set(ev.unix, list);
        continue;
      }
      if (kind !== 'PAPER_EXIT' && kind !== 'HORIZON') continue;
      const exitKind: LifecycleStoryKind =
        reason === 'TARGET' ? 'TARGET' : reason === 'STOP' ? 'STOP' : 'HORIZON';
      const ret = paper?.realized_return ?? null;
      items.push({
        unix: ev.unix,
        kind: exitKind,
        ticker,
        headline: (ticker.replace('_NS', '') || '—'),
        detail: ret == null ? exitKind : `${exitKind} · ${(ret >= 0 ? '+' : '')}${(ret * 100).toFixed(2)}% realized`,
        selectId,
        realizedReturn: ret,
      });
      if (ticker) emittedExit.add(ticker);
    }
    const walkExit = pos.walk?.exit;
    if (walkExit && ticker && !emittedExit.has(ticker)) {
      const exitKind: LifecycleStoryKind =
        reason === 'TARGET' ? 'TARGET' : reason === 'STOP' ? 'STOP' : 'HORIZON';
      const ret = paper?.realized_return ?? null;
      items.push({
        unix: walkExit.unix,
        kind: exitKind,
        ticker,
        headline: ticker.replace('_NS', '') || '—',
        detail: ret == null ? exitKind : `${exitKind} · ${(ret >= 0 ? '+' : '')}${(ret * 100).toFixed(2)}% realized`,
        selectId,
        realizedReturn: ret,
      });
    }
  }

  for (const [u, list] of enterByUnix) {
    if (list.length === 1) {
      const pos = list[0];
      const ticker = pos.paper?.ticker ?? '';
      items.push({
        unix: u,
        kind: 'PAPER_ENTER',
        ticker,
        headline: ticker.replace('_NS', '') || '—',
        detail: `PAPER_ENTER · ${pos.paper?.direction ?? ''}`.trim(),
        selectId: pos.paper?.decision_id || ticker,
        realizedReturn: null,
      });
    } else {
      items.push({
        unix: u,
        kind: 'PAPER_ENTER',
        ticker: null,
        headline: `${list.length} paper positions opened`,
        detail: 'PAPER_ENTER',
        selectId: null,
        realizedReturn: null,
      });
    }
  }

  items.sort((a, b) => a.unix - b.unix || a.kind.localeCompare(b.kind));
  return items;
}

export type LiveActivityKind =
  | 'BOOK_MARKED'
  | 'ASOF_COHORT'
  | 'ASOF_NAME'
  | 'PAPER_ENTER'
  | 'TARGET'
  | 'STOP'
  | 'HORIZON'
  | 'NEAREST_STOP'
  | 'NEAREST_TARGET';

/** Chronological cockpit feed. Copied from existing contracts — not a new classifier. */
export interface LiveActivityItem {
  unix: number;
  kind: LiveActivityKind;
  ticker: string | null;
  direction: string | null;
  headline: string;
  detail: string;
  selectId: string | null;
}

export function buildLiveActivityFeed(
  live: DeferredLiveResponse | null | undefined,
  recs: TodayRecommendations | null,
): LiveActivityItem[] {
  if (!live) return [];
  const items: LiveActivityItem[] = [];

  const asof = live.asof_events ?? [];
  if (asof.length > 0) {
    const byUnix = new Map<number, typeof asof>();
    for (const ev of asof) {
      const u = ev.as_of_unix;
      if (u == null) continue;
      const list = byUnix.get(u) ?? [];
      list.push(ev);
      byUnix.set(u, list);
    }
    let cohortUnix = 0;
    let cohort: typeof asof = [];
    for (const [u, list] of byUnix) {
      if (list.length > cohort.length) {
        cohortUnix = u;
        cohort = list;
      }
    }
    if (asof.length >= 2) {
      const nAct = asof.filter(e => (e.entry_action ?? '').toUpperCase() === 'ACT').length;
      items.push({
        unix: cohortUnix || asof[0].as_of_unix,
        kind: 'ASOF_COHORT',
        ticker: null,
        direction: null,
        headline: `${asof.length} As-of recommendations generated`,
        detail: `${nAct} ACT · ${asof.length - nAct} NOT_ACT`,
        selectId: null,
      });
    }
    for (const ev of asof) {
      if (cohort.length >= 2 && ev.as_of_unix === cohortUnix) continue;
      const action = (ev.entry_action ?? '').toUpperCase();
      const act = action === 'ACT';
      items.push({
        unix: ev.as_of_unix,
        kind: 'ASOF_NAME',
        ticker: ev.ticker,
        direction: null,
        headline: (ev.ticker ?? '').replace('_NS', '') || '—',
        detail: act ? 'ACT' : `NOT_ACT · ${ev.entry_action || ev.offer || 'AVOID'}`,
        selectId: ev.decision_id || ev.ticker,
      });
    }
  }

  const enterByUnix = new Map<number, DeferredLivePosition[]>();
  const emittedExit = new Set<string>();
  for (const pos of live.ledger?.positions ?? []) {
    const paper = pos.paper;
    const ticker = paper?.ticker ?? '';
    const direction = paper?.direction ?? pos.walk?.direction ?? null;
    const selectId = paper?.decision_id || ticker;
    for (const ev of pos.events ?? []) {
      const kind = (ev.kind ?? '').toUpperCase();
      if (kind === 'PAPER_ENTER') {
        const list = enterByUnix.get(ev.unix) ?? [];
        list.push(pos);
        enterByUnix.set(ev.unix, list);
        continue;
      }
      if (kind !== 'PAPER_EXIT' && kind !== 'HORIZON') continue;
      items.push(exitActivityItem(ev.unix, pos, ticker, direction, selectId));
      if (ticker) emittedExit.add(ticker);
    }
    const walkExit = pos.walk?.exit;
    if (walkExit && ticker && !emittedExit.has(ticker)) {
      items.push(exitActivityItem(walkExit.unix, pos, ticker, direction, selectId));
      emittedExit.add(ticker);
    }
  }
  const hasAsofCohort = items.some(i => i.kind === 'ASOF_COHORT');
  for (const [u, list] of enterByUnix) {
    if (list.length > 1 && hasAsofCohort) continue;
    if (list.length === 1) {
      const pos = list[0];
      const ticker = pos.paper?.ticker ?? '';
      items.push({
        unix: u,
        kind: 'PAPER_ENTER',
        ticker,
        direction: pos.paper?.direction ?? null,
        headline: ticker.replace('_NS', '') || '—',
        detail: ['PAPER_ENTER', pos.paper?.direction].filter(Boolean).join(' · '),
        selectId: pos.paper?.decision_id || ticker,
      });
    } else {
      items.push({
        unix: u,
        kind: 'PAPER_ENTER',
        ticker: null,
        direction: null,
        headline: `${list.length} positions entered`,
        detail: 'PAPER_ENTER',
        selectId: null,
      });
    }
  }

  const ingest = live.observation_feed?.last_ingest_unix
    ?? live.last_surface?.last_ingest_unix
    ?? live.last_surface?.as_of_unix
    ?? recs?.lastObsUnix
    ?? null;
  const nOpen = recs?.nActOpen ?? live.last_surface?.open ?? live.performance?.n_open ?? 0;
  const meanOpen = recs?.meanOpenMark ?? live.performance?.mean_open_mark ?? null;
  if (ingest != null) {
    items.push({
      unix: ingest,
      kind: 'BOOK_MARKED',
      ticker: null,
      direction: null,
      headline: 'Book marked',
      detail: `${nOpen} OPEN · mean ${fmtActivityPct(meanOpen)}`,
      selectId: null,
    });
  }

  const stop = recs?.attention[0];
  if (stop?.lastTickAt != null) {
    items.push({
      unix: stop.lastTickAt,
      kind: 'NEAREST_STOP',
      ticker: stop.ticker,
      direction: stop.direction,
      headline: stop.ticker.replace('_NS', '') || '—',
      detail: liveNameDetail(stop.direction, stop.mark, stop.toStopPct, 'STOP'),
      selectId: stop.decisionId || stop.ticker,
    });
  }
  const target = recs?.opportunity[0];
  if (target?.lastTickAt != null && target.ticker !== stop?.ticker) {
    items.push({
      unix: target.lastTickAt,
      kind: 'NEAREST_TARGET',
      ticker: target.ticker,
      direction: target.direction,
      headline: target.ticker.replace('_NS', '') || '—',
      detail: liveNameDetail(target.direction, target.mark, target.toTargetPct, 'TARGET'),
      selectId: target.decisionId || target.ticker,
    });
  }

  items.sort((a, b) => {
    if (b.unix !== a.unix) return b.unix - a.unix;
    return activityRank(a.kind) - activityRank(b.kind);
  });
  return items;
}

function exitActivityItem(
  unix: number,
  pos: DeferredLivePosition,
  ticker: string,
  direction: string | null,
  selectId: string,
): LiveActivityItem {
  const reason = (pos.walk?.exit?.reason ?? pos.paper?.exit_reason ?? '').toUpperCase();
  const exitKind: LiveActivityKind =
    reason === 'TARGET' ? 'TARGET' : reason === 'STOP' ? 'STOP' : 'HORIZON';
  const ret = pos.paper?.realized_return ?? null;
  return {
    unix,
    kind: exitKind,
    ticker,
    direction,
    headline: ticker.replace('_NS', '') || '—',
    detail: ret == null ? exitKind : `${exitKind} · ${fmtActivityPct(ret)}`,
    selectId,
  };
}

function liveNameDetail(
  direction: string | null | undefined,
  mark: number | null | undefined,
  remaining: number | null | undefined,
  barrier: 'STOP' | 'TARGET',
): string {
  return [
    direction || null,
    fmtActivityPct(mark),
    `${fmtRemainingPct(remaining)} remaining to ${barrier}`,
  ].filter(Boolean).join(' · ');
}

function activityRank(kind: LiveActivityKind): number {
  switch (kind) {
    case 'BOOK_MARKED': return 0;
    case 'NEAREST_STOP': return 1;
    case 'NEAREST_TARGET': return 2;
    case 'TARGET':
    case 'STOP':
    case 'HORIZON': return 3;
    case 'PAPER_ENTER': return 4;
    case 'ASOF_NAME': return 5;
    case 'ASOF_COHORT': return 6;
    default: return 9;
  }
}

function fmtActivityPct(v: number | null | undefined): string {
  if (v == null) return '—';
  return `${v >= 0 ? '+' : ''}${(v * 100).toFixed(2)}%`;
}

function fmtRemainingPct(v: number | null | undefined): string {
  if (v == null) return '—';
  return `${(Math.abs(v) * 100).toFixed(2)}%`;
}

function eventByKind(pos: DeferredLivePosition, kind: string): { unix: number; price: number } | undefined {
  const want = kind.toUpperCase();
  return (pos.events ?? []).find(e => (e.kind ?? '').toUpperCase() === want);
}

/** Ledger facts only. Does not decide TARGET/STOP/HORIZON — copies walk.exit / PAPER_ENTER. */
function lifecycleFromPosition(pos: DeferredLivePosition | undefined): Pick<NowActionRow, 'enteredAt' | 'exitAt' | 'exitPrice'> {
  if (!pos) return { enteredAt: null, exitAt: null, exitPrice: null };
  const enter = eventByKind(pos, 'PAPER_ENTER');
  const exitEv = eventByKind(pos, 'PAPER_EXIT') ?? eventByKind(pos, 'HORIZON');
  return {
    enteredAt: enter?.unix ?? pos.opened_at ?? null,
    exitAt: pos.walk?.exit?.unix ?? exitEv?.unix ?? null,
    exitPrice: pos.walk?.exit?.price ?? pos.paper?.paper_exit_price ?? exitEv?.price ?? null,
  };
}

/** Same-day ACT OQS rank. Uses portfolio context when present; otherwise the same sort as the API. */
function actRank(
  brief: DecisionBrief,
  decisions: DecisionBrief[],
  ctx: DecisionPortfolioContext | undefined,
): { rank: number | null; n: number | null } {
  if (ctx?.oqs_rank != null) {
    return { rank: ctx.oqs_rank, n: ctx.candidates_on_date ?? null };
  }
  const same = decisions
    .filter(d => d.date === brief.date && isAct(d.entry_action))
    .sort((a, b) => b.oqs - a.oqs || a.id.localeCompare(b.id));
  const i = same.findIndex(d => d.id === brief.id);
  return { rank: i >= 0 ? i + 1 : null, n: same.length > 0 ? same.length : null };
}

/** Live paper book only — not session ACT names that were never filled. */
export function isLiveBookRow(row: NowActionRow): boolean {
  return row.paperState === 'OPEN'
    || row.paperState === 'WAITING FILL'
    || row.paperState === 'TARGET'
    || row.paperState === 'STOP'
    || row.paperState === 'HORIZON';
}

function attentionBucket(row: NowActionRow): number {
  switch (row.paperState) {
    case 'OPEN': return 0;
    case 'WAITING FILL': return 1;
    case 'TARGET':
    case 'STOP':
    case 'HORIZON': return 2;
    default: return row.action === 'ACT' ? 3 : 4;
  }
}

/**
 * Presentation order for “what deserves attention now”.
 * Worst current mark first among the same paper state.
 * Does not use OQS / ACT rank — that stays a separate decision field.
 */
export function compareAttention(a: NowActionRow, b: NowActionRow): number {
  const db = attentionBucket(a) - attentionBucket(b);
  if (db !== 0) return db;
  const ma = a.mark;
  const mb = b.mark;
  if (ma == null && mb == null) return a.ticker.localeCompare(b.ticker);
  if (ma == null) return 1;
  if (mb == null) return -1;
  if (ma !== mb) return ma - mb;
  return a.ticker.localeCompare(b.ticker);
}

export function attentionRows(rows: NowActionRow[]): NowActionRow[] {
  return rows.filter(isLiveBookRow).sort(compareAttention);
}

/** Currently open paper — LIVE BOOK. WAITING FILL is a separate workflow state. */
export function isOpenBookRow(row: NowActionRow): boolean {
  return row.paperState === 'OPEN';
}

/** ACT armed, no observed fill yet. Not OPEN and not a new trading rule. */
export function isAwaitingFillRow(row: NowActionRow): boolean {
  return row.paperState === 'WAITING FILL';
}

export function compareAwaitingFill(a: NowActionRow, b: NowActionRow): number {
  const sa = a.snapUnix;
  const sb = b.snapUnix;
  if (sa == null && sb == null) return a.ticker.localeCompare(b.ticker);
  if (sa == null) return 1;
  if (sb == null) return -1;
  if (sa !== sb) return sa - sb;
  return a.ticker.localeCompare(b.ticker);
}

export function isSessionRecordRow(row: NowActionRow): boolean {
  return row.paperState === 'TARGET' || row.paperState === 'STOP' || row.paperState === 'HORIZON';
}

/**
 * Human is the next actor. Not a trading recommendation.
 * OPEN / TARGET / STOP / HORIZON never qualify — the frozen path already
 * owns those. Marks, freshness, and tape-end do not create a decision.
 */
export type DecisionRequiredKind = 'ARM_REQUIRED' | 'OBSERVATION_REQUIRED';

export function decisionRequiredKind(
  row: NowActionRow,
  live: DeferredLiveResponse | null | undefined,
): DecisionRequiredKind | null {
  if (row.action !== 'ACT') return null;
  if (row.paperState === 'OPEN') return null;
  if (row.paperState === 'TARGET' || row.paperState === 'STOP' || row.paperState === 'HORIZON') return null;
  const status = String(live?.observation_feed?.status ?? '').toUpperCase().replace(/\s+/g, '_');
  if (row.paperState === 'WAITING FILL' && (status === 'WAITING_FOR_FEED' || status === 'WAITINGFORFEED')) {
    return 'OBSERVATION_REQUIRED';
  }
  if (row.paperState === 'NOT IN BOOK' && live?.session?.auto_arm !== true) {
    return 'ARM_REQUIRED';
  }
  return null;
}

export function decisionRequiredReason(kind: DecisionRequiredKind): string {
  switch (kind) {
    case 'ARM_REQUIRED':
      return 'POST /arm required · ACT is not in the live book';
    case 'OBSERVATION_REQUIRED':
      return 'Observation required · WAITING FILL · producer idle (WAITING FOR FEED)';
  }
}

export interface SessionSummary {
  date: string | null;
  nEntered: number;
  nOpen: number;
  nTarget: number;
  nStop: number;
  nHorizon: number;
  lastObsUnix: number | null;
  tape: string;
}

function tapeFromFeed(status: string | null | undefined): string {
  const s = String(status ?? '').toUpperCase().replace(/\s+/g, '_');
  if (s === 'TAPE_EXHAUSTED' || s === 'TAPEEXHAUSTED') return 'EXHAUSTED';
  if (s === 'WAITING_FOR_FEED' || s === 'WAITINGFORFEED') return 'WAITING FOR FEED';
  if (s === 'WAITING_FOR_INBOX' || s === 'WAITINGFORINBOX') return 'WAITING FOR INBOX';
  if (!s) return '—';
  return s.replace(/_/g, ' ');
}

function countsFromLedger(positions: DeferredLivePosition[]): Omit<SessionSummary, 'date' | 'lastObsUnix' | 'tape'> {
  let nOpen = 0;
  let nTarget = 0;
  let nStop = 0;
  let nHorizon = 0;
  for (const pos of positions) {
    const st = paperStateFromPosition(pos);
    if (st === 'OPEN') nOpen += 1;
    else if (st === 'TARGET') nTarget += 1;
    else if (st === 'STOP') nStop += 1;
    else if (st === 'HORIZON') nHorizon += 1;
  }
  return { nEntered: positions.length, nOpen, nTarget, nStop, nHorizon };
}

/**
 * Today's book as a whole. Stage C counts when present; otherwise ledger
 * paper states. Session date is only present when auto-arm ran. Not a new
 * score and not a portfolio optimizer.
 */
export function buildSessionSummary(live: DeferredLiveResponse | null | undefined): SessionSummary | null {
  if (!live) return null;
  const positions = live.ledger?.positions ?? [];
  const perf = live.performance;
  const feed = live.observation_feed;
  const fromLedger = countsFromLedger(positions);
  const counts = perf != null
    ? {
      nEntered: perf.n_entered ?? fromLedger.nEntered,
      nOpen: perf.n_open ?? fromLedger.nOpen,
      nTarget: perf.n_target ?? fromLedger.nTarget,
      nStop: perf.n_stop ?? fromLedger.nStop,
      nHorizon: perf.n_horizon ?? fromLedger.nHorizon,
    }
    : fromLedger;
  const lastFromPos = positions.reduce((m, p) => Math.max(m, p.last_tick_at ?? 0), 0);
  return {
    // Session date is only a session when auto-arm ran. Do not promote
    // paper.date from a ticker-mode fill into a session label.
    date: live.session?.date ?? null,
    ...counts,
    lastObsUnix: feed?.last_ingest_unix ?? (lastFromPos > 0 ? lastFromPos : null),
    tape: tapeFromFeed(feed?.status),
  };
}

function rowFromBrief(
  brief: DecisionBrief,
  pos: DeferredLivePosition | undefined,
  armed: boolean,
  perfRet: number | undefined,
  decisions: DecisionBrief[],
  ctx: DecisionPortfolioContext | undefined,
): NowActionRow {
  const fromPos = paperStateFromPosition(pos);
  const filled = fromPos === 'OPEN' || fromPos === 'TARGET' || fromPos === 'STOP' || fromPos === 'HORIZON';
  // Ledger / as-of fill is today's recommendation. T0 Watch fixtures may be AVOID.
  const act = filled || isAct(pos?.paper?.entry_action) || isAct(brief.entry_action);
  let paperState: NowPaperState;
  if (fromPos) paperState = fromPos;
  else if (act && armed) paperState = 'WAITING FILL';
  else if (act) paperState = 'NOT IN BOOK';
  else paperState = 'NONE';

  const paperEntry = pos?.paper?.paper_entry_price ?? null;
  const last = pos == null
    ? null
    : (fromPos === 'OPEN' ? pos.current_price : (pos.paper?.paper_exit_price ?? pos.current_price));
  const carried = fromPos === 'OPEN' || fromPos === 'TARGET' || fromPos === 'STOP' || fromPos === 'HORIZON';
  const mark = carried ? (perfRet ?? signedMark(brief.direction, paperEntry, last)) : null;
  const markKind: NowMarkKind | null = fromPos === 'OPEN'
    ? 'MARK'
    : (fromPos === 'TARGET' || fromPos === 'STOP' || fromPos === 'HORIZON' ? 'REALIZED' : null);
  const { rank: oqsRank, n: candidatesOnDate } = actRank(brief, decisions, ctx);
  const action: NowActionKind = act ? 'ACT' : 'NO TRADE';
  const { toTargetPct, toStopPct } = barrierDistances(pos, paperState);

  return {
    decisionId: brief.id,
    ticker: brief.ticker,
    direction: brief.direction,
    action,
    entryState: filled ? (pos?.paper?.entry_action ?? 'ACT') : brief.entry_state,
    paperState,
    recommendation: recommendationFromPaper(action, paperState),
    paperEntry,
    mark,
    markKind,
    target: pos?.paper?.paper_target ?? pos?.walk?.target ?? brief.execution?.adaptive_target ?? null,
    risk: pos?.paper?.paper_risk ?? pos?.walk?.stop ?? brief.execution?.adaptive_risk ?? null,
    toTargetPct,
    toStopPct,
    snapUnix: brief.execution?.snap_unix ?? pos?.opened_at ?? null,
    lastTickAt: pos?.last_tick_at ?? null,
    currentPrice: pos == null ? null : (fromPos === 'OPEN' ? pos.current_price : (pos.paper?.paper_exit_price ?? pos.current_price)),
    asofDecisionId: pos?.paper?.decision_id ?? null,
    oqs: filled ? (pos?.paper?.oqs ?? brief.oqs ?? null) : (brief.oqs ?? null),
    oqsRank,
    candidatesOnDate,
    histWin: brief.hist_win ?? null,
    histPf: brief.hist_pf ?? null,
    histMed: brief.hist_med ?? null,
    entryWhy: brief.entry_why ?? null,
    entryHorizon: brief.entry_horizon ?? null,
    entryRisk: brief.entry_risk ?? null,
    riskDistancePct: brief.execution?.risk_distance_pct ?? null,
    ...lifecycleFromPosition(pos),
  };
}

/**
 * Rows the user should be able to read in seconds.
 * Session date → that day's IC briefs (one per ticker).
 * No session → only names already on the live book or armed.
 * Evidence is a projection of existing DecisionBrief / portfolio-context facts.
 */
export function buildNowActionRows(
  live: DeferredLiveResponse | null | undefined,
  decisions: DecisionBrief[],
  ctxById?: Map<string, DecisionPortfolioContext>,
): NowActionRow[] {
  const positions = live?.ledger?.positions ?? [];
  const armed = live?.armed ?? [];
  const perfRows = live?.performance?.rows ?? [];
  const sessionDate = live?.session?.date;

  const posById = new Map<string, DeferredLivePosition>();
  const posByTicker = new Map<string, DeferredLivePosition>();
  for (const p of positions) {
    const id = p.paper?.decision_id;
    if (id) posById.set(id, p);
    if (p.paper?.ticker) posByTicker.set(p.paper.ticker, p);
  }
  const armedByTicker = new Set(armed.map(a => a.ticker));
  const perfById = new Map(perfRows.filter(r => r.decision_id).map(r => [r.decision_id, r.ret]));

  let briefs: DecisionBrief[];
  if (sessionDate) {
    briefs = oneBriefPerTicker(decisions.filter(d => d.date === sessionDate));
  } else {
    const liveIds = new Set([
      ...armed.map(a => a.decision_id).filter(Boolean),
      ...positions.map(p => p.paper?.decision_id).filter((id): id is string => !!id),
    ]);
    briefs = decisions.filter(d => liveIds.has(d.id));
    const haveTickers = new Set(briefs.map(d => d.ticker));
    const unmatchedTickers = [
      ...armed.map(a => a.ticker),
      ...positions.map(p => p.paper?.ticker).filter((t): t is string => !!t),
    ].filter(t => !haveTickers.has(t));
    if (unmatchedTickers.length > 0) {
      const extra = oneBriefPerTicker(
        decisions.filter(d => unmatchedTickers.includes(d.ticker)),
      );
      briefs = [...briefs, ...extra];
    }
  }

  const rows = briefs.map(brief => {
    const pos = posById.get(brief.id) ?? posByTicker.get(brief.ticker);
    return rowFromBrief(
      brief,
      pos,
      armedByTicker.has(brief.ticker),
      pos?.paper?.decision_id ? perfById.get(pos.paper.decision_id) : perfById.get(brief.id),
      decisions,
      ctxById?.get(brief.id),
    );
  });

  if (rows.length === 0 && !sessionDate) {
    for (const pos of positions) {
      const paper = pos.paper;
      const fromPos = paperStateFromPosition(pos) ?? 'OPEN';
      const paperEntry = paper?.paper_entry_price ?? null;
      const last = fromPos === 'OPEN' ? pos.current_price : (paper?.paper_exit_price ?? pos.current_price);
      const id = paper?.decision_id ?? paper?.ticker ?? '';
      const { toTargetPct, toStopPct } = barrierDistances(pos, fromPos);
      rows.push({
        decisionId: id,
        ticker: paper?.ticker ?? '',
        direction: paper?.direction ?? '',
        action: 'ACT',
        entryState: paper?.entry_action ?? 'ACT',
        paperState: fromPos,
        recommendation: recommendationFromPaper('ACT', fromPos),
        paperEntry,
        mark: perfById.get(id) ?? signedMark(paper?.direction, paperEntry, last),
        markKind: fromPos === 'OPEN' ? 'MARK' : 'REALIZED',
        target: paper?.paper_target ?? pos.walk?.target ?? null,
        risk: paper?.paper_risk ?? pos.walk?.stop ?? null,
        snapUnix: pos.opened_at ?? null,
        lastTickAt: pos.last_tick_at ?? null,
        currentPrice: fromPos === 'OPEN' ? pos.current_price : (paper?.paper_exit_price ?? pos.current_price),
        asofDecisionId: paper?.decision_id ?? null,
        ...emptyEvidence(),
        oqs: paper?.oqs ?? null,
        ...lifecycleFromPosition(pos),
        toTargetPct,
        toStopPct,
      });
    }
    for (const a of armed) {
      if (rows.some(r => r.ticker === a.ticker)) continue;
      rows.push({
        decisionId: a.decision_id,
        ticker: a.ticker,
        direction: '',
        action: 'ACT',
        entryState: 'ACT',
        paperState: 'WAITING FILL',
        recommendation: recommendationFromPaper('ACT', 'WAITING FILL'),
        paperEntry: null,
        mark: null,
        markKind: null,
        target: null,
        risk: null,
        snapUnix: null,
        lastTickAt: null,
        currentPrice: null,
        asofDecisionId: a.decision_id,
        ...emptyEvidence(),
      });
    }
  }

  appendAsofNotAct(rows, live);
  rows.sort(compareAttention);
  return rows;
}

function appendAsofNotAct(rows: NowActionRow[], live: DeferredLiveResponse | null | undefined): void {
  const seen = new Set(rows.map(r => r.ticker));
  for (const ev of live?.asof_events ?? []) {
    const action = (ev.entry_action ?? '').toUpperCase();
    if (action === 'ACT') continue;
    if (!ev.ticker || seen.has(ev.ticker)) continue;
    seen.add(ev.ticker);
    rows.push({
      decisionId: ev.decision_id || ev.ticker,
      ticker: ev.ticker,
      direction: '',
      action: 'NO TRADE',
      entryState: ev.entry_action || 'NOT_ACT',
      paperState: 'NONE',
      recommendation: 'NOT RECOMMENDED',
      paperEntry: null,
      mark: null,
      markKind: null,
      target: null,
      risk: null,
      snapUnix: ev.as_of_unix ?? null,
      lastTickAt: null,
      currentPrice: null,
      asofDecisionId: ev.decision_id || null,
      ...emptyEvidence(),
    });
  }
}

/** Paper lifecycle copied onto a journal entry. Not inferred from marks. */
export interface JournalLifecycleEvent {
  unix: number;
  kind: string;
  price: number | null;
  note: string | null;
}

/**
 * Observational decision record. Copies as-of + ledger + LIVE-005 watch id.
 * Does not re-score, reassess, or invent TARGET / STOP / HORIZON.
 */
export interface DecisionJournalEntry {
  ticker: string;
  direction: string;
  decisionKind: 'ACT' | 'NOT_ACT';
  entryAction: string;
  offer: string | null;
  decisionId: string;
  watchId: string | null;
  decisionUnix: number | null;
  oqs: number | null;
  fill: number | null;
  stop: number | null;
  target: number | null;
  lifecycleEvents: JournalLifecycleEvent[];
  paperState: NowPaperState;
  realizedReturn: number | null;
  selectId: string;
}

function offerText(offer: unknown): string | null {
  if (offer == null) return null;
  if (typeof offer === 'string' && offer.trim()) return offer;
  return null;
}

function isActDecision(entryAction: string, offer: string | null, paperState: NowPaperState): boolean {
  if ((entryAction ?? '').toUpperCase() === 'ACT') return true;
  if ((offer ?? '').toUpperCase() === 'ARMED') return true;
  return paperState === 'OPEN' || paperState === 'TARGET' || paperState === 'STOP' || paperState === 'HORIZON';
}

/**
 * Ledger events only. PAPER_EXIT is labelled TARGET / STOP / HORIZON
 * when the ledger says so — never from mark, CAUTION, or elapsed time.
 */
function journalEventsFromPosition(pos: DeferredLivePosition | undefined): JournalLifecycleEvent[] {
  if (!pos) return [];
  const reason = (pos.walk?.exit?.reason ?? pos.paper?.exit_reason ?? '').toUpperCase();
  const mappedExit =
    reason === 'TARGET' || reason === 'STOP' || reason === 'HORIZON' ? reason : null;
  const out: JournalLifecycleEvent[] = [];
  const seen = new Set<string>();
  for (const ev of pos.events ?? []) {
    const kind = (ev.kind ?? '').toUpperCase();
    if (kind === 'PAPER_ENTER') {
      out.push({ unix: ev.unix, kind: 'PAPER_ENTER', price: ev.price ?? null, note: ev.note ?? null });
      seen.add(`PAPER_ENTER:${ev.unix}`);
      continue;
    }
    if (kind === 'PAPER_EXIT' || kind === 'TARGET' || kind === 'STOP' || kind === 'HORIZON') {
      const label = kind === 'PAPER_EXIT' ? (mappedExit ?? 'PAPER_EXIT') : kind;
      const key = `${label}:${ev.unix}`;
      if (seen.has(key)) continue;
      seen.add(key);
      out.push({ unix: ev.unix, kind: label, price: ev.price ?? null, note: ev.note ?? null });
    }
  }
  const walkExit = pos.walk?.exit;
  if (walkExit && mappedExit && !out.some(e => e.kind === mappedExit)) {
    out.push({
      unix: walkExit.unix,
      kind: mappedExit,
      price: walkExit.price ?? null,
      note: null,
    });
  }
  out.sort((a, b) => a.unix - b.unix || a.kind.localeCompare(b.kind));
  return out;
}

function watchIdFromBrief(brief: DecisionBrief | undefined): string | null {
  const id = brief?.id ?? '';
  return id.startsWith('LIVE-005') ? id : null;
}

/**
 * One journal row per as-of decision, plus any ledger name the as-of feed omitted.
 * T0 Watch AVOID/MONITOR text is not the ACT rationale — as-of is.
 */
export function buildDecisionJournal(
  live: DeferredLiveResponse | null | undefined,
  rows: NowActionRow[],
  decisions: DecisionBrief[],
): DecisionJournalEntry[] {
  if (!live) return [];
  const sessionDate = live.session?.date ?? null;
  const rowByTicker = new Map(rows.map(r => [r.ticker, r]));
  const posByTicker = new Map<string, DeferredLivePosition>();
  const posById = new Map<string, DeferredLivePosition>();
  for (const p of live.ledger?.positions ?? []) {
    if (p.paper?.ticker) posByTicker.set(p.paper.ticker, p);
    if (p.paper?.decision_id) posById.set(p.paper.decision_id, p);
  }
  const briefByTicker = new Map<string, DecisionBrief>();
  for (const d of decisions) {
    if (sessionDate && d.date !== sessionDate) continue;
    const prev = briefByTicker.get(d.ticker);
    if (!prev || (d.id.startsWith('LIVE-005') && !prev.id.startsWith('LIVE-005'))) {
      briefByTicker.set(d.ticker, d);
    }
  }

  const entries: DecisionJournalEntry[] = [];
  const seen = new Set<string>();

  const push = (
    ticker: string,
    asof?: { as_of_unix?: number; offer?: string; decision_id?: string; entry_action?: string },
  ) => {
    if (!ticker || seen.has(ticker)) return;
    seen.add(ticker);
    const row = rowByTicker.get(ticker);
    const pos = (asof?.decision_id ? posById.get(asof.decision_id) : undefined)
      ?? posByTicker.get(ticker);
    const paper = pos?.paper;
    const paperState = row?.paperState ?? paperStateFromPosition(pos) ?? 'NONE';
    const offer = offerText(asof?.offer) ?? null;
    const entryAction = (asof?.entry_action || paper?.entry_action || row?.entryState || '').toUpperCase();
    const decisionKind: 'ACT' | 'NOT_ACT' = isActDecision(entryAction, offer, paperState) ? 'ACT' : 'NOT_ACT';
    const closed = paperState === 'TARGET' || paperState === 'STOP' || paperState === 'HORIZON';
    entries.push({
      ticker,
      direction: (paper?.direction || row?.direction || '').toUpperCase(),
      decisionKind,
      entryAction: entryAction || (decisionKind === 'ACT' ? 'ACT' : 'NOT_ACT'),
      offer,
      decisionId: asof?.decision_id || paper?.decision_id || row?.asofDecisionId || row?.decisionId || ticker,
      watchId: watchIdFromBrief(briefByTicker.get(ticker)),
      decisionUnix: asof?.as_of_unix ?? row?.snapUnix ?? pos?.opened_at ?? row?.enteredAt ?? null,
      oqs: paper?.oqs ?? (decisionKind === 'ACT' ? (row?.oqs ?? null) : null),
      fill: paper?.paper_entry_price ?? null,
      stop: pos?.walk?.stop ?? paper?.candidate_stop_price ?? paper?.paper_risk ?? null,
      target: paper?.paper_target ?? pos?.walk?.target ?? null,
      lifecycleEvents: journalEventsFromPosition(pos),
      paperState,
      realizedReturn: closed ? (paper?.realized_return ?? null) : null,
      selectId: asof?.decision_id || paper?.decision_id || row?.decisionId || ticker,
    });
  };

  for (const ev of live.asof_events ?? []) {
    push(ev.ticker, ev);
  }
  for (const pos of live.ledger?.positions ?? []) {
    push(pos.paper?.ticker ?? '', {
      as_of_unix: pos.opened_at,
      offer: 'ARMED',
      decision_id: pos.paper?.decision_id,
      entry_action: pos.paper?.entry_action ?? 'ACT',
    });
  }

  entries.sort((a, b) => {
    const ua = a.decisionUnix ?? 0;
    const ub = b.decisionUnix ?? 0;
    if (ua !== ub) return ua - ub;
    return a.ticker.localeCompare(b.ticker);
  });
  return entries;
}

/** Closed journal rows only. Open marks are never outcomes. */
export interface OutcomeClosedRow {
  ticker: string;
  direction: string;
  outcome: 'TARGET' | 'STOP' | 'HORIZON';
  realizedReturn: number | null;
  unix: number | null;
  selectId: string;
}

/**
 * Outcome analytics from realized lifecycle history.
 * Empty until TARGET / STOP / HORIZON exist. Does not use open marks,
 * CAUTION, path-shape, or elapsed time.
 */
export interface OutcomeAnalytics {
  nClosed: number;
  nTarget: number;
  nStop: number;
  nHorizon: number;
  nWithRealized: number;
  meanRealized: number | null;
  closed: OutcomeClosedRow[];
}

export function buildOutcomeAnalytics(journal: DecisionJournalEntry[]): OutcomeAnalytics {
  const closed = journal.filter(e =>
    e.paperState === 'TARGET' || e.paperState === 'STOP' || e.paperState === 'HORIZON',
  );
  const rows: OutcomeClosedRow[] = closed.map(e => {
    const ev = [...e.lifecycleEvents].reverse().find(x =>
      x.kind === 'TARGET' || x.kind === 'STOP' || x.kind === 'HORIZON',
    );
    const outcome = e.paperState as 'TARGET' | 'STOP' | 'HORIZON';
    return {
      ticker: e.ticker,
      direction: e.direction,
      outcome,
      realizedReturn: e.realizedReturn,
      unix: ev?.unix ?? null,
      selectId: e.selectId,
    };
  });
  rows.sort((a, b) => (a.unix ?? 0) - (b.unix ?? 0) || a.ticker.localeCompare(b.ticker));
  const realized = rows.map(r => r.realizedReturn).filter((v): v is number => v != null);
  return {
    nClosed: rows.length,
    nTarget: rows.filter(r => r.outcome === 'TARGET').length,
    nStop: rows.filter(r => r.outcome === 'STOP').length,
    nHorizon: rows.filter(r => r.outcome === 'HORIZON').length,
    nWithRealized: realized.length,
    meanRealized: realized.length === 0 ? null : realized.reduce((a, b) => a + b, 0) / realized.length,
    closed: rows,
  };
}
