/**
 * Decision Cockpit v0.4 — REST API edition
 * ==========================================
 * Consumes /api/v1/intraday/decisions from the Rust IC v1 engine.
 * No TypeScript intelligence layer. No raw dataset in the browser.
 *
 * Architecture:
 *   Golden Dataset → Historical Adapter → Rust IC v1 → DecisionBrief DTO
 *   → GET /api/v1/intraday/decisions → CockpitView (render only)
 *
 * Features:
 *   - Ticker search + direction / entry state / H120 state / action filters
 *   - Decision list with sortable columns
 *   - Decision detail: full temporal timeline (T0 → H60 → H120 → H180 → H300)
 *     with ACTION / STATE / CONFIDENCE / HORIZON / WHY / RISK at each checkpoint
 *   - Historical performance reference per state (from Rust DTO)
 */

import React, { useState, useEffect, useMemo, useCallback, useRef } from 'react';
import type { DecisionBrief, DecisionPortfolioContext, DeferredLivePerformance, DeferredLivePosition, DeferredLiveResponse, ExecutionFacts, LifecycleCard, ObservationFeedSnapshot, PaperBlotter, PaperPosition, ReassessExperimentReport } from '../../types/decisionBrief';
import { fetchAllDecisions, fetchPortfolioContext, fetchPositionLifecycle, fetchPaperTrades, fetchDeferredLive } from '../../api/decisions';
import { buildNowActionRows, buildSessionSummary, buildTodayRecommendations, compareAttention, compareAwaitingFill, compareNearestStop, conditionCaption, conditionFromMark, decisionRequiredKind, decisionRequiredReason, isAwaitingFillRow, isLiveBookRow, isOpenBookRow, isSessionRecordRow, type DecisionRequiredKind, type NowActionRow, type NowRecommendation, type SessionSummary, type TodayRecommendations } from './nowAction';
import { DecisionBoard } from './DecisionBoard';
import { classifyLiveFreshness, formatObsAge, freshnessColor, type LiveFreshness } from './liveFreshness';

// ── Colour helpers (pure display — no intelligence) ───────────────────────────

function stateColor(state: string): string {
  switch (state) {
    case 'ENTER':      return '#22c55e';
    case 'WAIT-HIGH':  return '#3b82f6';
    case 'WAIT-MID':   return '#8b5cf6';
    case 'WAIT-LOW':   return '#6b7280';
    case 'AVOID':      return '#ef4444';
    case 'ENTER-LATE': return '#10b981';
    case 'WAIT-LATE':  return '#f59e0b';
    case 'AVOID-LATE': return '#dc2626';
    default:           return '#6b7280';
  }
}

function actionColor(action: string): string {
  switch (action) {
    case 'ACT':      return '#22c55e';
    case 'MONITOR':  return '#8b5cf6';
    case 'AVOID':    return '#ef4444';
    case 'UPGRADE':  return '#10b981';
    case 'CONTINUE': return '#f59e0b';
    case 'WIN':      return '#22c55e';
    case 'LOSS':     return '#ef4444';
    case 'IC':       return '#0369a1';
    default:         return '#6b7280';
  }
}

function confidenceColor(confidence: string): string {
  switch (confidence) {
    case 'HIGH':     return '#22c55e';
    case 'MODERATE': return '#f59e0b';
    case 'LOW':      return '#6b7280';
    default:         return '#6b7280';
  }
}

// ── API hook ──────────────────────────────────────────────────────────────────

function useDecisions() {
  const [decisions, setDecisions] = useState<DecisionBrief[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchAllDecisions()
      .then(resp => {
        setDecisions(resp.decisions);
        setLoading(false);
      })
      .catch(err => {
        setError(
          `Could not reach the Rust API: ${err.message}. ` +
          `Start the server: cargo run -p chronosentiment_server (port 8080).`
        );
        setLoading(false);
      });
  }, []);

  return { decisions, loading, error };
}

// ── Portfolio context hook ────────────────────────────────────────────────────
// Fetches once at mount; builds a lookup map id → DecisionPortfolioContext.
// Non-blocking: if the endpoint fails, the Cockpit still works without context.

function usePortfolioContext(): Map<string, DecisionPortfolioContext> {
  const [ctxMap, setCtxMap] = useState<Map<string, DecisionPortfolioContext>>(new Map());

  useEffect(() => {
    fetchPortfolioContext()
      .then(resp => {
        const m = new Map<string, DecisionPortfolioContext>();
        for (const d of resp.decisions) m.set(d.id, d);
        setCtxMap(m);
      })
      .catch(() => {
        // Portfolio context is supplementary — silently ignore failures.
      });
  }, []);

  return ctxMap;
}

// ── Lifecycle hook ────────────────────────────────────────────────────────────
// Fetches position lifecycle feed once at mount; silently ignores failures.

function useLifecycleFeed(): LifecycleCard[] {
  const [cards, setCards] = useState<LifecycleCard[]>([]);

  useEffect(() => {
    fetchPositionLifecycle()
      .then(resp => setCards(resp.cards))
      .catch(() => {
        // Decision Feed is supplementary — silently ignore failures.
      });
  }, []);

  return cards;
}

function usePaperBlotter(): PaperBlotter | null {
  const [blotter, setBlotter] = useState<PaperBlotter | null>(null);

  useEffect(() => {
    fetchPaperTrades()
      .then(setBlotter)
      .catch(() => {
        // Paper blotter is supplementary — silently ignore failures.
      });
  }, []);

  return blotter;
}

function useDeferredLive(): {
  live: DeferredLiveResponse | null;
  error: string | null;
  updatedAt: number | null;
  lastIngestSeenAt: number | null;
} {
  const [live, setLive] = useState<DeferredLiveResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [updatedAt, setUpdatedAt] = useState<number | null>(null);
  const [lastIngestSeenAt, setLastIngestSeenAt] = useState<number | null>(null);
  const prevIngestUnix = useRef<number | null>(null);

  useEffect(() => {
    let cancelled = false;
    const load = () => {
      fetchDeferredLive()
        .then(data => {
          if (cancelled) return;
          const unix = data.observation_feed?.last_ingest_unix ?? null;
          setLive(data);
          setError(null);
          setUpdatedAt(Date.now());
          if (unix != null && unix !== prevIngestUnix.current) {
            prevIngestUnix.current = unix;
            setLastIngestSeenAt(Date.now());
          }
        })
        .catch(e => {
          if (cancelled) return;
          setError(e instanceof Error ? e.message : 'deferred-live fetch failed');
        });
    };
    load();
    const id = window.setInterval(load, 4000);
    return () => { cancelled = true; window.clearInterval(id); };
  }, []);

  return { live, error, updatedAt, lastIngestSeenAt };
}

// ── Status helpers ────────────────────────────────────────────────────────────

function statusLabel(status: LifecycleCard['status']): string {
  switch (status) {
    case 'REASSESS_EXIT': return 'EXIT';
    case 'CLOSED':        return 'CLOSED';
    default:              return 'NEW';
  }
}

function statusColor(status: LifecycleCard['status']): string {
  switch (status) {
    case 'REASSESS_EXIT': return '#f59e0b';   // amber — reassessment triggered
    case 'CLOSED':        return '#6b7280';   // grey — horizon/stop/target
    default:              return '#3b82f6';   // blue — new recommendation
  }
}

// ── Evidence panel ────────────────────────────────────────────────────────────

function EvidencePanel({ ev, status, realizedRet, reassessTimeIst, h300Cf }: {
  ev: import('../../types/decisionBrief').DecisionEvidence;
  status: LifecycleCard['status'];
  realizedRet: number | null;
  reassessTimeIst: string | null;
  h300Cf: number | null;
}) {
  const fmt = (v: number | null) =>
    v == null ? '—' : `${v >= 0 ? '+' : ''}${(v * 100).toFixed(2)}%`;

  const rows: [string, string][] = [
    ['Historical win rate',   ev.hist_win  != null ? `${(ev.hist_win * 100).toFixed(0)}%` : '—'],
    ['Historical PF',         ev.hist_pf   != null ? `${ev.hist_pf >= 99 ? '>99' : ev.hist_pf.toFixed(1)}×` : '—'],
    ['Historical median ret', fmt(ev.hist_med)],
    ['reference_price',       fmtPrice(ev.reference_price)],
    ['entry_price',           fmtPrice(ev.entry_price)],
    ['adaptive_target',       fmtPrice(ev.execution?.adaptive_target)],
    ['target_distance_pct',   fmtPct(ev.execution?.target_distance_pct)],
    ['adaptive_risk',         fmtPrice(ev.execution?.adaptive_risk)],
    ['risk_distance_pct',     fmtPct(ev.execution?.risk_distance_pct)],
    ['expected_move_pct',     fmtPct(ev.execution?.expected_move_pct)],
    ['freshness',             ev.execution?.freshness ?? '—'],
    ['snap_unix',             fmtIst(ev.execution?.snap_unix)],
    ['OQS',                   String(ev.oqs)],
    ['Rank',                  `#${ev.oqs_rank} / ${ev.candidates_on_date}`],
    ['Entry state',           ev.entry_state],
    ['Entry horizon',         ev.entry_horizon],
    ['Entry risk',            ev.entry_risk],
    ['Why (entry)',           ev.entry_why],
  ];

  if (ev.exit_price != null)  rows.push(['exit_price',    fmtPrice(ev.exit_price)]);

  if (ev.h60_ret  != null) rows.push(['H60 return',  fmt(ev.h60_ret)]);
  if (ev.h120_ret != null) rows.push(['H120 return', fmt(ev.h120_ret)]);
  if (ev.mfe_h60  != null) rows.push(['MFE H60',     fmt(ev.mfe_h60)]);
  if (ev.h300_ret != null) rows.push(['H300 return', fmt(ev.h300_ret)]);

  if (status === 'REASSESS_EXIT') {
    rows.push(['H120 state',      ev.h120_state]);
    rows.push(['Why (reassess)',  ev.h120_why]);
    if (realizedRet != null) rows.push(['Realized return',      fmt(realizedRet)]);
    if (reassessTimeIst)     rows.push(['Exit time (IST)',       reassessTimeIst]);
    if (h300Cf != null)      rows.push(['H300 counterfactual',  fmt(h300Cf)]);
  }

  return (
    <div style={{
      marginTop: '0.5rem', padding: '0.625rem 0.75rem',
      backgroundColor: 'rgba(0,0,0,0.18)', borderRadius: '6px',
      border: '1px solid var(--border-color)',
    }}>
      <div style={{ fontSize: '0.6rem', color: 'var(--text-secondary)', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '0.4rem' }}>
        Decision Evidence
      </div>
      {rows.map(([label, value]) => (
        <div key={label} style={{ display: 'flex', justifyContent: 'space-between', gap: '0.5rem', marginBottom: '0.2rem' }}>
          <span style={{ fontSize: '0.65rem', color: 'var(--text-secondary)' }}>{label}</span>
          <span style={{ fontSize: '0.65rem', color: 'var(--text-primary)', fontWeight: 600, textAlign: 'right', maxWidth: '55%', wordBreak: 'break-word' }}>{value}</span>
        </div>
      ))}
    </div>
  );
}

// ── Decision Feed ─────────────────────────────────────────────────────────────
// Shows top-5 ACT decisions per date with lifecycle state.
// NEW = blue, REASSESS_EXIT = amber, CLOSED = grey.
// Compact card + collapsible "Why?" evidence panel.

function DecisionFeed({
  cards,
  onViewDecision,
}: {
  cards: LifecycleCard[];
  onViewDecision: (id: string) => void;
}) {
  const [expandedId, setExpandedId] = useState<string | null>(null);

  if (cards.length === 0) return null;

  // Group by date, preserve descending date order
  const byDate = new Map<string, LifecycleCard[]>();
  for (const c of cards) {
    if (!byDate.has(c.date)) byDate.set(c.date, []);
    byDate.get(c.date)!.push(c);
  }

  return (
    <div style={{ marginBottom: '1.5rem' }}>
      <div style={{
        fontSize: '0.65rem', color: 'var(--text-secondary)',
        textTransform: 'uppercase', letterSpacing: '0.05em',
        marginBottom: '0.75rem',
      }}>
        Decision Feed · historical IC v1 recommendations · not the live paper book
      </div>

      {[...byDate.entries()].map(([date, dayCards]) => (
        <div key={date} style={{ marginBottom: '1.25rem' }}>
          {/* Date header */}
          <div style={{
            fontSize: '0.62rem', color: 'var(--text-secondary)',
            textTransform: 'uppercase', letterSpacing: '0.06em',
            marginBottom: '0.5rem', paddingLeft: '0.25rem',
          }}>
            {date}
          </div>

          {/* Cards row */}
          <div style={{ display: 'flex', gap: '0.625rem', flexWrap: 'wrap' }}>
            {dayCards.map(card => {
              const dirColor  = card.direction === 'LONG' ? '#22c55e' : '#ef4444';
              const stColor   = statusColor(card.status);
              const stLabel   = statusLabel(card.status);
              const winPct    = card.hist_win != null ? `${(card.hist_win * 100).toFixed(0)}% win` : null;
              const isExit    = card.status === 'REASSESS_EXIT';
              const isClosed  = card.status === 'CLOSED';
              const historical = (card.execution?.freshness ?? 'STALE') !== 'LIVE';
              const showStatus = !(historical && card.status === 'NEW');
              const retVal    = card.realized_ret ?? card.h300_ret;
              const retColor  = retVal != null && retVal > 0.002 ? '#22c55e' : retVal != null && retVal < -0.002 ? '#ef4444' : 'var(--text-secondary)';

              return (
                <div
                  key={card.id}
                  style={{
                    flex: '0 1 260px', minWidth: '240px',
                    backgroundColor: 'var(--bg-card)',
                    border: `1px solid ${isExit ? '#f59e0b44' : dirColor + '33'}`,
                    borderTop: `3px solid ${isExit ? '#f59e0b' : isClosed ? '#6b7280' : dirColor}`,
                    borderRadius: '8px',
                    padding: '0.75rem 0.875rem',
                    display: 'flex', flexDirection: 'column', gap: '0.4rem',
                    opacity: isClosed && !isExit ? 0.75 : 1,
                  }}
                >
                  {/* Status badge + direction + ticker */}
                  <div style={{ display: 'flex', alignItems: 'center', gap: '0.4rem', flexWrap: 'wrap' }}>
                    {showStatus && (
                    <span style={{
                      fontSize: '0.58rem', fontWeight: 700, padding: '0.08rem 0.3rem',
                      borderRadius: '3px',
                      backgroundColor: stColor + '22', color: stColor,
                      border: `1px solid ${stColor}44`,
                      letterSpacing: '0.04em',
                    }}>
                      {stLabel}
                    </span>
                    )}
                    <span style={{
                      fontSize: '0.65rem', fontWeight: 700, padding: '0.08rem 0.3rem',
                      borderRadius: '3px',
                      backgroundColor: dirColor + '22', color: dirColor,
                      border: `1px solid ${dirColor}44`,
                    }}>
                      {card.direction}
                    </span>
                    <span style={{ fontSize: '0.88rem', fontWeight: 700, color: 'var(--text-primary)' }}>
                      {card.ticker.replace('_NS', '')}
                    </span>
                    {historical && (
                      <span style={{
                        fontSize: '0.55rem', fontWeight: 700, padding: '0.08rem 0.3rem',
                        borderRadius: '3px', backgroundColor: '#f59e0b22', color: '#f59e0b',
                        border: '1px solid #f59e0b44', letterSpacing: '0.04em',
                      }}>
                        HISTORICAL
                      </span>
                    )}
                  </div>

                  {/* Rank + OQS */}
                  <div style={{ fontSize: '0.67rem', color: 'var(--text-secondary)' }}>
                    Rank #{card.oqs_rank} · OQS {card.oqs}
                  </div>

                  {/* Historical win rate */}
                  {winPct && (
                    <div style={{ fontSize: '0.67rem', color: '#22c55e' }}>
                      {winPct}
                      {card.hist_pf != null ? ` · PF ${card.hist_pf >= 99 ? '>99' : card.hist_pf.toFixed(1)}x` : ''}
                    </div>
                  )}

                  {/* Lifecycle state */}
                  {isExit && card.realized_ret != null && (
                    <div style={{ fontSize: '0.67rem', color: '#f59e0b', fontWeight: 600 }}>
                      Reassessment exit
                      {` · ${card.realized_ret >= 0 ? '+' : ''}${(card.realized_ret * 100).toFixed(2)}%`}
                      {card.reassess_time_ist ? ` @ ${card.reassess_time_ist}` : ''}
                    </div>
                  )}
                  {isExit && card.h300_counterfactual_ret != null && (
                    <div style={{ fontSize: '0.62rem', color: 'var(--text-secondary)' }}>
                      H300 cf: {card.h300_counterfactual_ret >= 0 ? '+' : ''}{(card.h300_counterfactual_ret * 100).toFixed(2)}%
                    </div>
                  )}
                  {isClosed && !isExit && retVal != null && (
                    <div style={{ fontSize: '0.67rem', color: retColor, fontWeight: 600 }}>
                      {card.exit_reason ?? 'CLOSED'}
                      {` · ${retVal >= 0 ? '+' : ''}${(retVal * 100).toFixed(2)}%`}
                    </div>
                  )}

                  <ExecutionBlock
                    execution={card.execution}
                    direction={card.direction}
                    action={card.entry_action}
                    status={card.status}
                    entryPrice={card.entry_price}
                    entryHorizon={card.evidence?.entry_horizon}
                  />

                  {/* Action buttons */}
                  <div style={{ display: 'flex', gap: '0.375rem', marginTop: '0.25rem' }}>
                    <button
                      onClick={() => setExpandedId(expandedId === card.id ? null : card.id)}
                      style={{
                        flex: '0 0 auto',
                        padding: '0.3rem 0.55rem', borderRadius: '5px', cursor: 'pointer',
                        background: expandedId === card.id ? 'rgba(255,255,255,0.1)' : 'transparent',
                        border: '1px solid var(--border-color)',
                        color: 'var(--text-secondary)', fontSize: '0.67rem', fontWeight: 600,
                      }}
                    >
                      {expandedId === card.id ? 'Hide' : 'Why?'}
                    </button>
                    <button
                      onClick={() => onViewDecision(card.id)}
                      style={{
                        flex: 1,
                        padding: '0.3rem 0.6rem', borderRadius: '5px', cursor: 'pointer',
                        background: dirColor + '18',
                        border: `1px solid ${dirColor}44`,
                        color: dirColor, fontSize: '0.67rem', fontWeight: 600,
                        textAlign: 'center',
                      }}
                    >
                      View Decision →
                    </button>
                  </div>

                  {/* Evidence panel (collapsible) */}
                  {expandedId === card.id && (
                    <EvidencePanel
                      ev={card.evidence}
                      status={card.status}
                      realizedRet={card.realized_ret}
                      reassessTimeIst={card.reassess_time_ist}
                      h300Cf={card.h300_counterfactual_ret}
                    />
                  )}
                </div>
              );
            })}
          </div>
        </div>
      ))}
    </div>
  );
}

// ── Micro-components ──────────────────────────────────────────────────────────

function StateBadge({ state }: { state: string }) {
  const c = stateColor(state);
  return (
    <span style={{
      display: 'inline-block', padding: '0.15rem 0.45rem', borderRadius: '4px',
      fontSize: '0.7rem', fontWeight: 700, letterSpacing: '0.04em',
      backgroundColor: c + '22', color: c, border: `1px solid ${c}44`,
    }}>
      {state}
    </span>
  );
}

function ActionBadge({ action }: { action: string }) {
  const c = actionColor(action);
  return (
    <span style={{
      display: 'inline-block', padding: '0.15rem 0.45rem', borderRadius: '4px',
      fontSize: '0.7rem', fontWeight: 700, letterSpacing: '0.04em',
      backgroundColor: c + '22', color: c, border: `1px solid ${c}44`,
    }}>
      {action}
    </span>
  );
}

function ConfidenceDot({ confidence }: { confidence: string }) {
  const c = confidenceColor(confidence);
  return (
    <span style={{ display: 'inline-flex', alignItems: 'center', gap: '0.3rem', fontSize: '0.7rem', color: c, fontWeight: 600 }}>
      <span style={{ width: '6px', height: '6px', borderRadius: '50%', backgroundColor: c, display: 'inline-block' }} />
      {confidence}
    </span>
  );
}

function fmtRet(v: number | null | undefined): string {
  if (v == null) return '—';
  const pct = (v * 100).toFixed(2);
  return v >= 0 ? `+${pct}%` : `${pct}%`;
}

function fmtPrice(v: number | null | undefined): string {
  if (v == null) return '—';
  return `₹${v.toLocaleString('en-IN', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

function fmtPct(v: number | null | undefined): string {
  if (v == null) return '—';
  return `${v >= 0 ? '+' : ''}${(v * 100).toFixed(2)}%`;
}

function fmtHistWin(v: number | null | undefined): string | null {
  if (v == null) return null;
  return `${(v * 100).toFixed(0)}%`;
}

function fmtHistPf(v: number | null | undefined): string | null {
  if (v == null) return null;
  return `${v >= 99 ? '>99' : v.toFixed(1)}×`;
}

function joinDot(parts: Array<string | null | undefined>): string {
  return parts.filter((p): p is string => !!p && p !== '—').join(' · ');
}

function fmtIst(unix: number | null | undefined): string {
  if (unix == null) return '—';
  return `${new Intl.DateTimeFormat('en-GB', {
    timeZone: 'Asia/Kolkata',
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  }).format(new Date(unix * 1000))} IST`;
}

function fmtIstTime(unix: number | null | undefined): string {
  if (unix == null) return '—';
  return `${new Intl.DateTimeFormat('en-GB', {
    timeZone: 'Asia/Kolkata',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  }).format(new Date(unix * 1000))} IST`;
}

function fmtIstHm(unix: number | null | undefined): string {
  if (unix == null) return '—';
  return `${new Intl.DateTimeFormat('en-GB', {
    timeZone: 'Asia/Kolkata',
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  }).format(new Date(unix * 1000))} IST`;
}

function snapClockLabel(unix: number | null | undefined, clockSim: boolean): string {
  if (unix == null) return '—';
  return clockSim ? fmtIstHm(unix) : fmtAge(unix);
}

function fmtAge(unix: number | null | undefined): string {
  if (unix == null) return '—';
  const sec = Math.max(0, Math.floor((Date.now() - unix * 1000) / 1000));
  if (sec < 60) return `${sec}s ago`;
  const min = Math.floor(sec / 60);
  if (min < 60) return `${min}m ago`;
  const hr = Math.floor(min / 60);
  if (hr < 48) return `${hr}h ago`;
  return `${Math.floor(hr / 24)}d ago`;
}

function currentDecisionLabel(status: string, action: string | undefined, direction: string): string {
  if (status === 'REASSESS_EXIT') return `EXIT ${direction}`;
  if (status === 'CLOSED') return `CLOSED ${direction}`;
  return `${action ?? '—'} ${direction}`;
}

function ExecutionBlock({
  execution,
  direction,
  action,
  status,
  entryPrice,
  entryHorizon,
}: {
  execution: ExecutionFacts | null | undefined;
  direction: string;
  action?: string;
  status: string;
  entryPrice: number | null | undefined;
  entryHorizon?: string | null;
}) {
  const ex = execution;
  const stale = (ex?.freshness ?? 'STALE') !== 'LIVE';
  const label = currentDecisionLabel(status, action, direction);
  const lastTick = fmtIst(ex?.last_tick_unix);
  const lastTickAge = ex?.last_tick_unix != null ? fmtAge(ex.last_tick_unix) : null;
  const horizonLine = ex?.horizon_elapsed === true
    ? 'ELAPSED'
    : (entryHorizon ?? '—');

  return (
    <div style={{
      marginTop: '0.5rem',
      padding: '0.625rem 0.7rem',
      backgroundColor: stale ? '#f59e0b0d' : '#22c55e0d',
      border: `1px solid ${stale ? '#f59e0b44' : '#22c55e33'}`,
      borderRadius: '6px',
    }}>
      <div style={{ fontSize: '0.55rem', color: 'var(--text-secondary)', textTransform: 'uppercase', letterSpacing: '0.06em', marginBottom: '0.35rem' }}>
        {stale ? 'Historical decision · not live' : 'Current decision'}
      </div>
      <div style={{ fontSize: '0.95rem', fontWeight: 800, color: stale ? '#f59e0b' : actionColor(action ?? ''), marginBottom: '0.15rem' }}>
        {label}
      </div>
      {stale ? (
        <div style={{ fontSize: '0.68rem', color: '#f59e0b', fontWeight: 700, marginBottom: '0.45rem' }}>
          No live last tick · current_price not available
        </div>
      ) : (
        <>
          <div style={{ fontSize: '1.05rem', fontWeight: 700, color: 'var(--text-primary)', fontVariantNumeric: 'tabular-nums' }}>
            {fmtPrice(ex?.current_price)}
          </div>
          <div style={{ fontSize: '0.62rem', color: 'var(--text-secondary)', marginBottom: '0.45rem' }}>
            Last tick {lastTick} · {lastTickAge}
          </div>
        </>
      )}
      {[
        ['TARGET', fmtPrice(ex?.adaptive_target), fmtPct(ex?.target_distance_pct)],
        ['RISK', fmtPrice(ex?.adaptive_risk), fmtPct(ex?.risk_distance_pct)],
        ['entry_price', fmtPrice(entryPrice), ''],
        ['expected_move', fmtPct(ex?.expected_move_pct), ''],
        ['HORIZON', horizonLine, ex?.adaptive_horizon_sessions != null ? `${ex.adaptive_horizon_sessions} sess` : ''],
      ].map(([k, v, extra]) => (
        <div key={k} style={{ display: 'flex', justifyContent: 'space-between', gap: '0.4rem' }}>
          <span style={{ fontSize: '0.62rem', color: 'var(--text-secondary)' }}>{k}</span>
          <span style={{ fontSize: '0.62rem', color: 'var(--text-primary)', fontWeight: 600, fontVariantNumeric: 'tabular-nums', textAlign: 'right' }}>
            {v}{extra ? `  ${extra}` : ''}
          </span>
        </div>
      ))}
      <div style={{
        marginTop: '0.45rem',
        fontSize: '0.58rem',
        fontWeight: 700,
        letterSpacing: '0.04em',
        color: stale ? '#f59e0b' : '#22c55e',
      }}>
        {stale ? 'HISTORICAL REPLAY · not live' : '● LIVE'}
        {ex?.snap_unix != null ? ` · decision ${fmtIst(ex.snap_unix)}` : ''}
      </div>
    </div>
  );
}

function ModeStrip() {
  return (
    <div style={{
      display: 'grid',
      gridTemplateColumns: '1fr 1fr',
      gap: '0.75rem',
      marginBottom: '1.25rem',
    }}>
      <div style={{
        padding: '0.7rem 0.85rem',
        borderRadius: '8px',
        backgroundColor: '#f59e0b14',
        border: '1px solid #f59e0b66',
      }}>
        <div style={{ fontSize: '0.72rem', fontWeight: 800, letterSpacing: '0.04em', color: '#b45309', textTransform: 'uppercase' }}>
          Historical Replay
        </div>
        <div style={{ fontSize: '0.68rem', color: 'var(--text-secondary)', marginTop: '0.2rem' }}>
          Frozen historical paper trading
        </div>
      </div>
      <div style={{
        padding: '0.7rem 0.85rem',
        borderRadius: '8px',
        backgroundColor: '#0ea5e914',
        border: '1px solid #0ea5e966',
      }}>
        <div style={{ fontSize: '0.72rem', fontWeight: 800, letterSpacing: '0.04em', color: '#0369a1', textTransform: 'uppercase' }}>
          Deferred Live
        </div>
        <div style={{ fontSize: '0.68rem', color: 'var(--text-secondary)', marginTop: '0.2rem' }}>
          Live paper book · observation producer · no Yahoo · no broker
        </div>
      </div>
    </div>
  );
}

function paperStateColor(state: string): string {
  switch (state) {
    case 'OPEN': return '#0ea5e9';
    case 'WAITING FILL': return '#0369a1';
    case 'TARGET': return '#16a34a';
    case 'STOP': return '#dc2626';
    case 'HORIZON': return '#6b7280';
    case 'NO TRADE':
    case 'NONE': return '#6b7280';
    default: return 'var(--text-secondary)';
  }
}

function recommendationColor(rec: NowRecommendation | string): string {
  switch (rec) {
    case 'HOLD': return '#0369a1';
    case 'TARGET': return '#16a34a';
    case 'STOP': return '#dc2626';
    case 'HORIZON': return '#6b7280';
    case 'NOT RECOMMENDED': return '#6b7280';
    default: return 'var(--text-secondary)';
  }
}

function FreshnessBadge({ freshness }: { freshness: LiveFreshness }) {
  const color = freshnessColor(freshness.kind);
  const lastObs = fmtIstTime(freshness.lastObsUnix);
  const age = freshness.kind === 'TAPE EXHAUSTED'
    ? 'tape ended'
    : (freshness.kind === 'WAITING FOR FEED' ? '—' : formatObsAge(freshness.ageMs));
  const clockLine = freshness.clockSimulation
    ? `${freshness.sourceLabel}${freshness.speedLabel ? ` · ${freshness.speedLabel}` : ''} · controlled clock · not a broker feed`
    : (freshness.kind === 'WAITING FOR FEED'
      ? 'no observations yet'
      : `${freshness.sourceLabel} · paper observation · not a broker`);
  return (
    <div style={{
      marginTop: '0.5rem',
      padding: '0.45rem 0.55rem',
      borderRadius: '6px',
      border: `1px solid ${color}55`,
      backgroundColor: `${color}12`,
      display: 'flex',
      flexWrap: 'wrap',
      gap: '0.35rem 1rem',
      alignItems: 'baseline',
    }}>
      <div style={{ fontSize: '0.72rem', fontWeight: 800, letterSpacing: '0.04em', color, textTransform: 'uppercase' }}>
        ● {freshness.kind}
      </div>
      <div style={{ fontSize: '0.65rem', color: 'var(--text-secondary)' }}>
        Last observation {lastObs}
        {' · '}Age {age}
      </div>
      <div style={{ fontSize: '0.62rem', color: freshness.clockSimulation ? '#b45309' : 'var(--text-secondary)', marginLeft: 'auto' }}>
        {clockLine}
      </div>
    </div>
  );
}

function NowActionStrip({
  live,
  decisions,
  ctxById,
  lastIngestSeenAt,
  onViewDecision,
}: {
  live: DeferredLiveResponse | null;
  decisions: DecisionBrief[];
  ctxById?: Map<string, DecisionPortfolioContext>;
  lastIngestSeenAt: number | null;
  onViewDecision: (id: string) => void;
}) {
  const [showNoTrade, setShowNoTrade] = useState(false);
  const [nowMs, setNowMs] = useState(() => Date.now());
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const rows = buildNowActionRows(live, decisions, ctxById);
  const actRows = rows.filter(r => r.action === 'ACT');
  const noTradeRows = rows.filter(r => r.action === 'NO TRADE');
  const openRows = rows.filter(isOpenBookRow).sort(compareNearestStop);
  const awaitingRows = rows.filter(isAwaitingFillRow).sort(compareAwaitingFill);
  const recordRows = rows.filter(isSessionRecordRow).sort(compareAttention);
  const sessionSummary = buildSessionSummary(live);
  const todayRecs = buildTodayRecommendations(live, openRows, noTradeRows);
  const sessionDate = sessionSummary?.date ?? live?.session?.date;
  const freshness = classifyLiveFreshness(live?.observation_feed, lastIngestSeenAt, nowMs);
  const waiting = freshness.kind === 'WAITING FOR FEED';

  useEffect(() => {
    if (freshness.kind !== 'LIVE' && freshness.kind !== 'STALE') return;
    const id = window.setInterval(() => setNowMs(Date.now()), 250);
    return () => window.clearInterval(id);
  }, [freshness.kind]);

  const toggleRow = (id: string) => setExpandedId(prev => prev === id ? null : id);

  return (
    <div style={{
      margin: '0.7rem 0 0.85rem',
      padding: '0.7rem 0.8rem',
      borderRadius: '6px',
      backgroundColor: 'var(--bg-card)',
      border: '1px solid var(--border-color)',
    }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', gap: '0.75rem', flexWrap: 'wrap', alignItems: 'baseline' }}>
        <div style={{ fontSize: '0.62rem', fontWeight: 800, letterSpacing: '0.05em', textTransform: 'uppercase', color: 'var(--text-primary)' }}>
          Right now{sessionDate ? ' · session' : ''} · live book
        </div>
        <div style={{ fontSize: '0.62rem', color: 'var(--text-secondary)' }}>
          {sessionDate ? `session ${sessionDate}` : (waiting ? 'no live session' : 'live book')}
          {' · '}{awaitingRows.length} awaiting fill
          {' · '}{openRows.length} open
          {' · '}{recordRows.length} closed
          {' · '}{noTradeRows.length} NOT_ACT
          {' · '}ledger projection · no new score
        </div>
      </div>
      <FreshnessBadge freshness={freshness} />
      <TodaysRecommendationsBoard
        recs={todayRecs}
        sessionDate={sessionDate ?? null}
        onSelect={toggleRow}
      />
      <SessionSummaryBoard summary={sessionSummary} />
      {actRows.length === 0 && openRows.length === 0 && awaitingRows.length === 0 && (
        <div style={{ fontSize: '0.72rem', color: 'var(--text-secondary)', marginTop: '0.45rem', lineHeight: 1.45 }}>
          {waiting
            ? 'No live paper action. Observation producer is idle (WAITING FOR FEED). MONITOR/AVOID stay NO TRADE. Arm a session or supply observations to carry ACT names.'
            : (noTradeRows.length > 0
              ? (sessionDate
                ? 'No ACT names to carry. The current IC decisions on this session are NO TRADE.'
                : 'No ACT names to carry. The current IC decisions on this live book are NO TRADE.')
              : 'No ACT paper position and nothing armed. This is the current deferred-live state.')}
        </div>
      )}
      <AwaitingFillBoard
        rows={awaitingRows}
        clockSim={freshness.clockSimulation}
        expandedId={expandedId}
        onSelect={toggleRow}
        onViewDecision={onViewDecision}
      />
      <NowBookSection
        title="Today's recommended book"
        note="ACT · OPEN · HOLD · ranked by remaining room to frozen STOP · attention only"
        rows={openRows}
        empty="No OPEN paper. ACT names still waiting for an observation are in Awaiting fill."
        expandedId={expandedId}
        onSelect={toggleRow}
        onViewDecision={onViewDecision}
      />
      <NowBookSection
        title={sessionDate ? 'Session record' : 'Closed record'}
        note={sessionDate
          ? 'everything that happened this session · TARGET / STOP / HORIZON'
          : 'TARGET / STOP / HORIZON on this live book'}
        rows={recordRows}
        empty="No TARGET / STOP / HORIZON yet. Open names stay on the live book until they exit."
        expandedId={expandedId}
        onSelect={toggleRow}
        onViewDecision={onViewDecision}
      />
      <SessionHandoffBoard
        date={sessionDate ?? null}
        tape={sessionSummary?.tape ?? '—'}
        lastObsUnix={sessionSummary?.lastObsUnix ?? null}
        clockSim={freshness.clockSimulation}
        openRows={openRows}
        recordRows={recordRows}
        awaitingRows={awaitingRows}
        onSelect={toggleRow}
      />
      <DecisionRequiredBoard
        live={live}
        rows={rows}
        openRows={openRows}
        onViewDecision={onViewDecision}
      />
      {showNoTrade && noTradeRows.length > 0 && (
        <div style={{ marginTop: '0.55rem' }}>
          <div style={{ fontSize: '0.62rem', fontWeight: 800, letterSpacing: '0.05em', textTransform: 'uppercase', color: 'var(--text-secondary)', marginBottom: '0.35rem' }}>
            Not Recommended / NOT_ACT · {noTradeRows.length} · not in the paper book
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
          {noTradeRows.map(r => (
            <NowActionRowView
              key={r.decisionId || r.ticker}
              row={r}
              expanded={expandedId === (r.decisionId || r.ticker)}
              onToggle={() => toggleRow(r.decisionId || r.ticker)}
              onViewDecision={onViewDecision}
            />
          ))}
          </div>
        </div>
      )}
      {noTradeRows.length > 0 && (
        <button
          type="button"
          onClick={() => setShowNoTrade(v => !v)}
          style={{
            marginTop: '0.5rem',
            background: 'transparent',
            border: 'none',
            padding: 0,
            cursor: 'pointer',
            color: 'var(--text-secondary)',
            fontSize: '0.62rem',
            fontWeight: 600,
          }}
        >
          {showNoTrade ? 'Hide NOT_ACT' : `Show ${noTradeRows.length} Not Recommended / NOT_ACT · MONITOR/AVOID · not in the paper book`}
        </button>
      )}
    </div>
  );
}

function HandoffNone() {
  return <div style={{ fontSize: '0.68rem', color: 'var(--text-secondary)', marginTop: '0.12rem' }}>None</div>;
}

function SessionHandoffBoard({
  date,
  tape,
  lastObsUnix,
  clockSim,
  openRows,
  recordRows,
  awaitingRows,
  onSelect,
}: {
  date: string | null;
  tape: string;
  lastObsUnix: number | null;
  clockSim: boolean;
  openRows: NowActionRow[];
  recordRows: NowActionRow[];
  awaitingRows: NowActionRow[];
  onSelect: (id: string) => void;
}) {
  const label = (row: NowActionRow) => (row.ticker ?? '').replace('_NS', '') || '—';
  return (
    <div style={{
      marginTop: '0.7rem',
      padding: '0.55rem 0.6rem',
      borderRadius: '6px',
      border: '1px solid var(--border-color)',
      backgroundColor: 'rgba(0,0,0,0.12)',
      fontVariantNumeric: 'tabular-nums',
    }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', gap: '0.75rem', flexWrap: 'wrap', alignItems: 'baseline' }}>
        <div style={{ fontSize: '0.62rem', fontWeight: 800, letterSpacing: '0.05em', textTransform: 'uppercase', color: 'var(--text-primary)' }}>
          {date ? `Session handoff · ${date}` : 'Ledger handoff'}
        </div>
        <div style={{ fontSize: '0.58rem', color: 'var(--text-secondary)' }}>
          carry-forward from the ledger · not a new book · no new trade
        </div>
      </div>

      <div style={{ fontSize: '0.52rem', fontWeight: 800, letterSpacing: '0.04em', textTransform: 'uppercase', color: 'var(--text-secondary)', marginTop: '0.45rem' }}>
        Open positions
      </div>
      {openRows.length === 0 ? <HandoffNone /> : openRows.map(r => {
        const markC = (r.mark ?? 0) > 0 ? '#16a34a' : (r.mark ?? 0) < 0 ? '#dc2626' : 'var(--text-secondary)';
        return (
          <button
            key={r.decisionId || r.ticker}
            type="button"
            onClick={() => onSelect(r.decisionId || r.ticker)}
            style={{
              display: 'block', width: '100%', textAlign: 'left', background: 'transparent',
              border: 'none', padding: '0.12rem 0', cursor: 'pointer', color: 'inherit',
              fontSize: '0.7rem', lineHeight: 1.45,
            }}
          >
            <span style={{ fontWeight: 800 }}>{label(r)}</span>
            {r.direction ? ` · ${r.direction}` : ''}
            {' · '}
            <span style={{ fontWeight: 700, color: markC }}>
              {r.mark == null ? '—' : `${fmtPct(r.mark)} ${r.markKind ?? 'MARK'}`}
            </span>
            {r.lastTickAt != null ? ` · last ${fmtIstHm(r.lastTickAt)}` : ''}
          </button>
        );
      })}

      <div style={{ fontSize: '0.52rem', fontWeight: 800, letterSpacing: '0.04em', textTransform: 'uppercase', color: 'var(--text-secondary)', marginTop: '0.4rem' }}>
        Realized
      </div>
      {recordRows.length === 0 ? <HandoffNone /> : recordRows.map(r => {
        const markC = (r.mark ?? 0) > 0 ? '#16a34a' : (r.mark ?? 0) < 0 ? '#dc2626' : 'var(--text-secondary)';
        return (
          <button
            key={r.decisionId || r.ticker}
            type="button"
            onClick={() => onSelect(r.decisionId || r.ticker)}
            style={{
              display: 'block', width: '100%', textAlign: 'left', background: 'transparent',
              border: 'none', padding: '0.12rem 0', cursor: 'pointer', color: 'inherit',
              fontSize: '0.7rem', lineHeight: 1.45,
            }}
          >
            <span style={{ fontWeight: 800 }}>{label(r)}</span>
            {r.direction ? ` · ${r.direction}` : ''}
            {' · '}
            <span style={{ fontWeight: 700, color: paperStateColor(r.paperState) }}>{r.paperState}</span>
            {r.mark != null && (
              <span style={{ fontWeight: 700, color: markC }}>{` · ${fmtPct(r.mark)} REALIZED`}</span>
            )}
          </button>
        );
      })}

      <div style={{ fontSize: '0.52rem', fontWeight: 800, letterSpacing: '0.04em', textTransform: 'uppercase', color: 'var(--text-secondary)', marginTop: '0.4rem' }}>
        Awaiting fill
      </div>
      {awaitingRows.length === 0 ? <HandoffNone /> : awaitingRows.map(r => (
        <button
          key={r.decisionId || r.ticker}
          type="button"
          onClick={() => onSelect(r.decisionId || r.ticker)}
          style={{
            display: 'block', width: '100%', textAlign: 'left', background: 'transparent',
            border: 'none', padding: '0.12rem 0', cursor: 'pointer', color: 'inherit',
            fontSize: '0.7rem', lineHeight: 1.45,
          }}
        >
          <span style={{ fontWeight: 800 }}>{label(r)}</span>
          {r.direction ? ` · ${r.direction}` : ''}
          {' · WAITING FILL'}
          {r.snapUnix != null ? ` · ${snapClockLabel(r.snapUnix, clockSim)}` : ''}
        </button>
      ))}

      <div style={{ fontSize: '0.52rem', fontWeight: 800, letterSpacing: '0.04em', textTransform: 'uppercase', color: 'var(--text-secondary)', marginTop: '0.4rem' }}>
        {date ? 'Session status' : 'Tape status'}
      </div>
      <div style={{ fontSize: '0.7rem', color: 'var(--text-primary)', marginTop: '0.12rem', fontWeight: 700 }}>
        TAPE {tape}
        {' · '}Last observation {fmtIstHm(lastObsUnix)}
      </div>
    </div>
  );
}

function DecisionRequiredBoard({
  live,
  rows,
  openRows,
  onViewDecision,
}: {
  live: DeferredLiveResponse | null;
  rows: NowActionRow[];
  openRows: NowActionRow[];
  onViewDecision: (id: string) => void;
}) {
  const required = rows
    .map(row => ({ row, kind: decisionRequiredKind(row, live) }))
    .filter((x): x is { row: NowActionRow; kind: DecisionRequiredKind } => x.kind != null);

  return (
    <div style={{
      marginTop: '0.7rem',
      padding: '0.55rem 0.6rem',
      borderRadius: '6px',
      border: `1px solid ${required.length > 0 ? '#b4530955' : 'var(--border-color)'}`,
      backgroundColor: required.length > 0 ? '#f59e0b12' : 'rgba(0,0,0,0.12)',
      fontVariantNumeric: 'tabular-nums',
    }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', gap: '0.75rem', flexWrap: 'wrap', alignItems: 'baseline' }}>
        <div style={{ fontSize: '0.62rem', fontWeight: 800, letterSpacing: '0.05em', textTransform: 'uppercase', color: 'var(--text-primary)' }}>
          Decision required{required.length > 0 ? ` · ${required.length}` : ''}
        </div>
        <div style={{ fontSize: '0.58rem', color: 'var(--text-secondary)' }}>
          user is the next actor · no new trading rule · no reassessment
        </div>
      </div>

      {required.length === 0 ? (
        <>
          <div style={{ fontSize: '0.78rem', fontWeight: 800, color: 'var(--text-primary)', marginTop: '0.35rem' }}>
            None
          </div>
          {openRows.map(r => {
            const markC = (r.mark ?? 0) > 0 ? '#16a34a' : (r.mark ?? 0) < 0 ? '#dc2626' : 'var(--text-secondary)';
            return (
              <div key={r.decisionId || r.ticker} style={{ fontSize: '0.7rem', marginTop: '0.2rem', lineHeight: 1.45 }}>
                <span style={{ fontWeight: 800 }}>{(r.ticker ?? '').replace('_NS', '')}</span>
                {r.direction ? ` · ${r.direction}` : ''}
                {' · '}
                <span style={{ fontWeight: 700, color: markC }}>
                  {r.mark == null ? '—' : `${fmtPct(r.mark)} ${r.markKind ?? 'MARK'}`}
                </span>
              </div>
            );
          })}
          <div style={{ fontSize: '0.65rem', color: 'var(--text-secondary)', marginTop: '0.25rem', lineHeight: 1.45 }}>
            No decision required from current ledger state.
          </div>
        </>
      ) : required.map(({ row, kind }) => {
        const markC = (row.mark ?? 0) > 0 ? '#16a34a' : (row.mark ?? 0) < 0 ? '#dc2626' : 'var(--text-secondary)';
        return (
          <div key={row.decisionId || row.ticker} style={{ marginTop: '0.4rem' }}>
            <div style={{ fontSize: '0.72rem', fontWeight: 800, lineHeight: 1.45 }}>
              {(row.ticker ?? '').replace('_NS', '')}
              {row.direction ? ` · ${row.direction}` : ''}
              {row.mark != null && (
                <span style={{ color: markC }}>{` · ${fmtPct(row.mark)} ${row.markKind ?? 'MARK'}`}</span>
              )}
              {row.mark == null ? ` · ${row.paperState}` : ''}
            </div>
            <div style={{ fontSize: '0.52rem', fontWeight: 800, letterSpacing: '0.04em', textTransform: 'uppercase', color: 'var(--text-secondary)', marginTop: '0.25rem' }}>
              Reason
            </div>
            <div style={{ fontSize: '0.68rem', color: 'var(--text-primary)', marginTop: '0.08rem', lineHeight: 1.45 }}>
              {decisionRequiredReason(kind)}
            </div>
            {row.decisionId ? (
              <button
                type="button"
                onClick={() => onViewDecision(row.decisionId)}
                style={{
                  marginTop: '0.4rem',
                  background: 'transparent',
                  border: '1px solid #b4530966',
                  borderRadius: '5px',
                  padding: '0.25rem 0.5rem',
                  cursor: 'pointer',
                  color: '#b45309',
                  fontSize: '0.62rem',
                  fontWeight: 700,
                  letterSpacing: '0.04em',
                  textTransform: 'uppercase',
                }}
              >
                View position
              </button>
            ) : null}
          </div>
        );
      })}
    </div>
  );
}

function TodaysRecommendationsBoard({
  recs,
  sessionDate,
  onSelect,
}: {
  recs: TodayRecommendations | null;
  sessionDate: string | null;
  onSelect: (id: string) => void;
}) {
  if (!recs) return null;
  const hold = recs.nActOpen > 0 && recs.nTarget === 0 && recs.nStop === 0 && recs.nHorizon === 0;
  return (
    <div style={{
      marginTop: '0.7rem',
      padding: '0.7rem 0.75rem',
      borderRadius: '8px',
      border: '1px solid #0369a166',
      backgroundColor: '#0369a114',
      fontVariantNumeric: 'tabular-nums',
    }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', gap: '0.75rem', flexWrap: 'wrap', alignItems: 'baseline' }}>
        <div style={{ fontSize: '0.72rem', fontWeight: 800, letterSpacing: '0.05em', textTransform: 'uppercase', color: '#0369a1' }}>
          Today&apos;s Recommendations
        </div>
        <div style={{ fontSize: '0.58rem', color: 'var(--text-secondary)' }}>
          {sessionDate ? `session ${sessionDate}` : 'live paper book'}
          {' · '}ledger projection · not a new classifier
        </div>
      </div>
      <div style={{ fontSize: '0.82rem', fontWeight: 800, color: 'var(--text-primary)', marginTop: '0.35rem' }}>
        {recs.nAct} ACT
        {' · '}{recs.nActOpen} OPEN
        {' · '}{recs.nLong} LONG
        {' · '}{recs.nShort} SHORT
      </div>
      <div style={{
        marginTop: '0.55rem',
        padding: '0.55rem 0.65rem',
        borderRadius: '6px',
        border: '1px solid #0369a155',
        backgroundColor: 'var(--bg-card)',
      }}>
        <div style={{ fontSize: '1.05rem', fontWeight: 800, letterSpacing: '0.04em', color: '#0369a1' }}>
          {hold ? 'HOLD' : 'SEE LIFECYCLE'}
        </div>
        <div style={{ fontSize: '0.72rem', color: 'var(--text-primary)', marginTop: '0.15rem', fontWeight: 600 }}>
          {recs.nActOpen} OPEN · recommendation is not a health claim
        </div>
        <div style={{ fontSize: '0.65rem', color: 'var(--text-secondary)', marginTop: '0.12rem' }}>
          Ledger {recs.nActOpen} OPEN · {recs.nTarget} TARGET · {recs.nStop} STOP · {recs.nHorizon} HORIZON
          {' · '}open mark {fmtPct(recs.meanOpenMark)} · realized {(recs.nTarget + recs.nStop + recs.nHorizon) === 0 ? '—' : 'see closed'}
        </div>
      </div>
      <div style={{
        marginTop: '0.55rem',
        fontSize: '0.62rem',
        fontWeight: 700,
        letterSpacing: '0.03em',
        color: '#b45309',
        lineHeight: 1.45,
      }}>
        HOLD — frozen paper remains active. Adverse mark-to-fill is attention, not an exit. Frozen LIVE-005 barriers remain authoritative.
      </div>
      <div style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))',
        gap: '0.65rem',
        marginTop: '0.55rem',
      }}>
        <WatchOrderPanel
          title="Nearest STOP"
          kind="stop"
          rows={recs.attention}
          onSelect={onSelect}
        />
        <WatchOrderPanel
          title="Nearest TARGET"
          kind="target"
          rows={recs.opportunity}
          onSelect={onSelect}
        />
      </div>
    </div>
  );
}

function WatchOrderPanel({
  title,
  kind,
  rows,
  onSelect,
}: {
  title: string;
  kind: 'stop' | 'target';
  rows: NowActionRow[];
  onSelect: (id: string) => void;
}) {
  return (
    <div style={{
      padding: '0.5rem 0.55rem',
      borderRadius: '6px',
      border: '1px solid var(--border-color)',
      backgroundColor: 'var(--bg-card)',
    }}>
      <div style={{ fontSize: '0.58rem', fontWeight: 800, letterSpacing: '0.04em', textTransform: 'uppercase', color: 'var(--text-secondary)' }}>
        {title}
      </div>
      {rows.length === 0 ? (
        <div style={{ fontSize: '0.68rem', color: 'var(--text-secondary)', marginTop: '0.3rem' }}>None</div>
      ) : rows.map(r => {
        const dist = kind === 'stop' ? r.toStopPct : r.toTargetPct;
        const dirColor = r.direction === 'LONG' ? '#16a34a' : r.direction === 'SHORT' ? '#dc2626' : 'var(--text-secondary)';
        return (
          <button
            key={r.decisionId || r.ticker}
            type="button"
            onClick={() => onSelect(r.decisionId || r.ticker)}
            style={{
              display: 'block', width: '100%', textAlign: 'left', background: 'transparent',
              border: 'none', padding: '0.28rem 0 0', cursor: 'pointer', color: 'inherit',
            }}
          >
            <div style={{ fontSize: '0.78rem', fontWeight: 800 }}>
              {(r.ticker ?? '').replace('_NS', '') || '—'}
              {' '}
              <span style={{ color: dirColor, fontSize: '0.62rem' }}>{r.direction}</span>
            </div>
            <div style={{ fontSize: '0.68rem', color: 'var(--text-primary)', fontWeight: 700 }}>
              {fmtPct(dist)} remaining
              {' · '}
              <span style={{ color: recommendationColor(r.recommendation) }}>{r.recommendation}</span>
            </div>
            <div style={{ fontSize: '0.58rem', color: 'var(--text-secondary)', marginTop: '0.06rem' }}>
              {conditionCaption(conditionFromMark(r.mark))}
            </div>
          </button>
        );
      })}
    </div>
  );
}

function SessionSummaryBoard({ summary }: { summary: SessionSummary | null }) {
  if (!summary) return null;
  return (
    <div style={{
      marginTop: '0.55rem',
      padding: '0.55rem 0.6rem',
      borderRadius: '6px',
      border: '1px solid var(--border-color)',
      backgroundColor: 'rgba(0,0,0,0.12)',
      fontVariantNumeric: 'tabular-nums',
    }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', gap: '0.75rem', flexWrap: 'wrap', alignItems: 'baseline' }}>
        <div style={{ fontSize: '0.62rem', fontWeight: 800, letterSpacing: '0.05em', textTransform: 'uppercase', color: 'var(--text-primary)' }}>
          {summary.date ? `Session · ${summary.date}` : 'Live book'}
        </div>
        <div style={{ fontSize: '0.58rem', color: 'var(--text-secondary)' }}>
          Stage C ledger counts · not a portfolio score
        </div>
      </div>
      <div style={{ fontSize: '0.72rem', color: 'var(--text-primary)', marginTop: '0.28rem', lineHeight: 1.5, fontWeight: 600 }}>
        {summary.nEntered} ACT entered
        {' · '}{summary.nOpen} OPEN
        {' · '}{summary.nStop} STOP
        {' · '}{summary.nTarget} TARGET
        {' · '}{summary.nHorizon} HORIZON
      </div>
      <div style={{ fontSize: '0.65rem', color: 'var(--text-secondary)', marginTop: '0.2rem' }}>
        Last observation · {fmtIstHm(summary.lastObsUnix)}
        {' · '}Tape · {summary.tape}
      </div>
    </div>
  );
}

function AwaitingFillBoard({
  rows,
  clockSim,
  expandedId,
  onSelect,
  onViewDecision,
}: {
  rows: NowActionRow[];
  clockSim: boolean;
  expandedId: string | null;
  onSelect: (id: string) => void;
  onViewDecision: (id: string) => void;
}) {
  return (
    <div style={{
      marginTop: '0.7rem',
      padding: '0.5rem 0.55rem',
      borderRadius: '6px',
      border: '1px solid #0369a133',
      backgroundColor: '#0369a10a',
    }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', gap: '0.75rem', flexWrap: 'wrap', alignItems: 'baseline' }}>
        <div style={{ fontSize: '0.62rem', fontWeight: 800, letterSpacing: '0.05em', textTransform: 'uppercase', color: 'var(--text-primary)' }}>
          Awaiting fill · {rows.length}
        </div>
        <div style={{ fontSize: '0.58rem', color: 'var(--text-secondary)' }}>
          ACT · WAITING FILL · armed, no observed enter · not OPEN
        </div>
      </div>
      {rows.length === 0 ? (
        <div style={{ fontSize: '0.68rem', color: 'var(--text-secondary)', marginTop: '0.3rem', lineHeight: 1.45 }}>
          No ACT names waiting for an observation. Filled names are on the live book.
        </div>
      ) : (
        <div style={{ marginTop: '0.35rem', fontVariantNumeric: 'tabular-nums' }}>
          <div style={{
            display: 'grid',
            gridTemplateColumns: 'minmax(6rem, 1.2fr) minmax(4rem, 0.8fr) minmax(5rem, 0.9fr)',
            gap: '0.35rem 0.6rem',
            padding: '0 0.25rem 0.12rem',
            color: 'var(--text-secondary)',
            fontSize: '0.52rem',
            fontWeight: 700,
            letterSpacing: '0.04em',
            textTransform: 'uppercase',
          }}>
            <span>Ticker</span>
            <span>Action</span>
            <span>Snap</span>
          </div>
          {rows.map(r => {
            const id = r.decisionId || r.ticker;
            const ticker = (r.ticker ?? '').replace('_NS', '') || '—';
            const dirColor = r.direction === 'LONG' ? '#16a34a' : r.direction === 'SHORT' ? '#dc2626' : 'var(--text-secondary)';
            const selected = expandedId === id;
            return (
              <div key={id}>
                <button
                  type="button"
                  onClick={() => onSelect(id)}
                  style={{
                    display: 'grid',
                    gridTemplateColumns: 'minmax(6rem, 1.2fr) minmax(4rem, 0.8fr) minmax(5rem, 0.9fr)',
                    gap: '0.35rem 0.6rem',
                    width: '100%',
                    alignItems: 'baseline',
                    textAlign: 'left',
                    background: selected ? '#0369a118' : 'transparent',
                    border: 'none',
                    borderRadius: '4px',
                    padding: '0.2rem 0.25rem',
                    cursor: 'pointer',
                    color: 'inherit',
                  }}
                >
                  <span style={{ fontSize: '0.72rem', fontWeight: 800 }}>{ticker}</span>
                  <span style={{ fontSize: '0.68rem', fontWeight: 700, color: dirColor }}>{r.direction || 'ACT'}</span>
                  <span style={{ fontSize: '0.68rem', color: 'var(--text-secondary)' }}>{snapClockLabel(r.snapUnix, clockSim)}</span>
                </button>
                {selected && (
                  <NowActionRowView
                    row={r}
                    expanded
                    onToggle={() => onSelect(id)}
                    onViewDecision={onViewDecision}
                  />
                )}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}

function NowBookSection({
  title,
  note,
  rows,
  empty,
  expandedId,
  onSelect,
  onViewDecision,
}: {
  title: string;
  note: string;
  rows: NowActionRow[];
  empty: string;
  expandedId: string | null;
  onSelect: (id: string) => void;
  onViewDecision: (id: string) => void;
}) {
  return (
    <div style={{ marginTop: '0.7rem' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', gap: '0.75rem', flexWrap: 'wrap', alignItems: 'baseline' }}>
        <div style={{ fontSize: '0.62rem', fontWeight: 800, letterSpacing: '0.05em', textTransform: 'uppercase', color: 'var(--text-primary)' }}>
          {title}
        </div>
        <div style={{ fontSize: '0.58rem', color: 'var(--text-secondary)' }}>{note}</div>
      </div>
      {rows.length === 0 ? (
        <div style={{ fontSize: '0.68rem', color: 'var(--text-secondary)', marginTop: '0.3rem', lineHeight: 1.45 }}>{empty}</div>
      ) : (
        <>
          <AttentionBoard rows={rows} expandedId={expandedId} onSelect={onSelect} />
          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', marginTop: '0.45rem' }}>
            {rows.map(r => (
              <NowActionRowView
                key={r.decisionId || r.ticker}
                row={r}
                expanded={expandedId === (r.decisionId || r.ticker)}
                onToggle={() => onSelect(r.decisionId || r.ticker)}
                onViewDecision={onViewDecision}
              />
            ))}
          </div>
        </>
      )}
    </div>
  );
}

function AttentionBoard({
  rows,
  expandedId,
  onSelect,
}: {
  rows: NowActionRow[];
  expandedId: string | null;
  onSelect: (id: string) => void;
}) {
  if (rows.length === 0) return null;
  return (
    <div style={{
      marginTop: '0.35rem',
      padding: '0.4rem 0.45rem',
      borderRadius: '6px',
      border: '1px solid #0369a133',
      backgroundColor: '#0369a10a',
    }}>
      <div style={{ fontVariantNumeric: 'tabular-nums', overflowX: 'auto' }}>
        <div style={{
          display: 'grid',
          gridTemplateColumns: 'minmax(6rem, 1.15fr) 3.6rem 3.8rem minmax(4.4rem, 0.8fr) minmax(4.6rem, 0.75fr) minmax(4.6rem, 0.75fr) 4.2rem 3.6rem',
          gap: '0.3rem 0.45rem',
          padding: '0 0.25rem 0.12rem',
          color: 'var(--text-secondary)',
          fontSize: '0.52rem',
          fontWeight: 700,
          letterSpacing: '0.04em',
          textTransform: 'uppercase',
        }}>
          <span>Ticker</span>
          <span>Side</span>
          <span>Rec</span>
          <span>P&amp;L</span>
          <span>To TARGET</span>
          <span>To STOP</span>
          <span>Last</span>
          <span>Status</span>
        </div>
        {rows.map(r => {
          const id = r.decisionId || r.ticker;
          const ticker = (r.ticker ?? '').replace('_NS', '') || '—';
          const markC = (r.mark ?? 0) > 0 ? '#16a34a' : (r.mark ?? 0) < 0 ? '#dc2626' : 'var(--text-secondary)';
          const selected = expandedId === id;
          const dirColor = r.direction === 'LONG' ? '#16a34a' : r.direction === 'SHORT' ? '#dc2626' : 'var(--text-secondary)';
          return (
            <button
              key={id}
              type="button"
              onClick={() => onSelect(id)}
              style={{
                display: 'grid',
                gridTemplateColumns: 'minmax(6rem, 1.15fr) 3.6rem 3.8rem minmax(4.4rem, 0.8fr) minmax(4.6rem, 0.75fr) minmax(4.6rem, 0.75fr) 4.2rem 3.6rem',
                gap: '0.3rem 0.45rem',
                width: '100%',
                alignItems: 'baseline',
                textAlign: 'left',
                background: selected ? '#0369a118' : 'transparent',
                border: 'none',
                borderRadius: '4px',
                padding: '0.18rem 0.25rem',
                cursor: 'pointer',
                color: 'inherit',
              }}
            >
              <span style={{ fontSize: '0.72rem', fontWeight: 800 }}>{ticker}</span>
              <span style={{ fontSize: '0.62rem', fontWeight: 700, color: dirColor }}>{r.direction || '—'}</span>
              <span style={{ fontSize: '0.62rem', fontWeight: 800, color: recommendationColor(r.recommendation) }}>
                {r.recommendation}
              </span>
              <span style={{ fontSize: '0.72rem', fontWeight: 700, color: markC }}>
                {r.mark == null ? '—' : fmtPct(r.mark)}
              </span>
              <span style={{ fontSize: '0.65rem', fontWeight: 600 }}>
                {fmtPct(r.toTargetPct)}
              </span>
              <span style={{ fontSize: '0.65rem', fontWeight: 600 }}>
                {fmtPct(r.toStopPct)}
              </span>
              <span style={{ fontSize: '0.62rem', color: 'var(--text-secondary)' }}>
                {fmtIstHm(r.lastTickAt)}
              </span>
              <span style={{ fontSize: '0.62rem', fontWeight: 700, color: paperStateColor(r.paperState) }}>
                {r.paperState}
              </span>
            </button>
          );
        })}
      </div>
    </div>
  );
}

function LifecycleLines({ row }: { row: NowActionRow }) {
  if (row.paperState === 'WAITING FILL' && row.enteredAt == null) {
    return (
      <div style={{
        marginTop: '0.35rem', padding: '0.35rem 0.4rem', borderRadius: '4px',
        backgroundColor: 'rgba(0,0,0,0.14)', border: '1px solid var(--border-color)',
        fontSize: '0.65rem', color: 'var(--text-secondary)', lineHeight: 1.45,
      }}>
        <div style={{ fontSize: '0.52rem', fontWeight: 700, letterSpacing: '0.04em', textTransform: 'uppercase', marginBottom: '0.12rem' }}>
          Lifecycle · ledger
        </div>
        WAITING FILL · no observed enter yet
      </div>
    );
  }
  if (!isLiveBookRow(row) || (row.enteredAt == null && row.paperEntry == null)) return null;

  const verb = row.paperState === 'STOP' ? 'STOPPED' : row.paperState;
  const closed = row.paperState === 'STOP' || row.paperState === 'TARGET' || row.paperState === 'HORIZON';
  const markC = (row.mark ?? 0) > 0 ? '#16a34a' : (row.mark ?? 0) < 0 ? '#dc2626' : 'var(--text-secondary)';

  return (
    <div style={{
      marginTop: '0.35rem', padding: '0.35rem 0.4rem', borderRadius: '4px',
      backgroundColor: 'rgba(0,0,0,0.14)', border: '1px solid var(--border-color)',
      fontSize: '0.65rem', color: 'var(--text-secondary)', lineHeight: 1.45, fontVariantNumeric: 'tabular-nums',
    }}>
      <div style={{ fontSize: '0.52rem', fontWeight: 700, letterSpacing: '0.04em', textTransform: 'uppercase', marginBottom: '0.12rem' }}>
        Lifecycle · existing ledger events · not a new path
      </div>
      {(row.enteredAt != null || row.paperEntry != null) && (
        <div>ENTERED {fmtIstHm(row.enteredAt)} @ {fmtPrice(row.paperEntry)}</div>
      )}
      {row.paperState === 'OPEN' && (
        <>
          <div style={{ fontWeight: 700, color: paperStateColor(row.paperState) }}>
            OPEN → MARK <span style={{ color: markC }}>{row.mark == null ? '—' : fmtPct(row.mark)}</span>
          </div>
          {row.lastTickAt != null && <div>Last observation {fmtIstHm(row.lastTickAt)}</div>}
        </>
      )}
      {closed && (
        <>
          <div style={{ fontWeight: 700, color: paperStateColor(row.paperState) }}>
            {verb} {fmtIstHm(row.exitAt)} @ {fmtPrice(row.exitPrice)}
          </div>
          {row.mark != null && (
            <div style={{ fontWeight: 700, color: markC }}>{fmtPct(row.mark)} REALIZED</div>
          )}
        </>
      )}
    </div>
  );
}

function NowActionRowView({
  row,
  expanded,
  onToggle,
  onViewDecision,
}: {
  row: NowActionRow;
  expanded: boolean;
  onToggle: () => void;
  onViewDecision: (id: string) => void;
}) {
  const open = expanded;
  const act = row.action === 'ACT';
  const dirColor = row.direction === 'LONG' ? '#16a34a' : row.direction === 'SHORT' ? '#dc2626' : 'var(--text-secondary)';
  const actColor = act ? '#0369a1' : '#6b7280';
  const markC = (row.mark ?? 0) > 0 ? '#16a34a' : (row.mark ?? 0) < 0 ? '#dc2626' : 'var(--text-secondary)';
  const ticker = (row.ticker ?? '').replace('_NS', '') || '—';
  const win = fmtHistWin(row.histWin);
  const pf = fmtHistPf(row.histPf);
  const rankLine = row.oqsRank != null ? `Rank #${row.oqsRank}` : null;
  const oqsLine = row.oqs != null ? `OQS ${row.oqs}` : null;
  const histLine = joinDot([
    win != null ? `Hist win ${win}` : null,
    pf != null ? `PF ${pf}` : null,
  ]);
  const riskLine = joinDot([
    row.entryHorizon || null,
    row.riskDistancePct != null ? `Risk ${fmtPct(row.riskDistancePct)}` : null,
  ]);
  const evidenceRows: [string, string][] = [
    ['OQS / rank', joinDot([
      row.oqs != null ? String(row.oqs) : null,
      row.oqsRank != null
        ? (row.candidatesOnDate != null ? `#${row.oqsRank} / ${row.candidatesOnDate}` : `#${row.oqsRank}`)
        : null,
    ]) || '—'],
    ['Historical win', win ?? '—'],
    ['Historical PF', pf ?? '—'],
    ['Historical median', row.histMed == null ? '—' : fmtPct(row.histMed)],
    ['Entry state', row.entryState || '—'],
    ['Entry reason', row.entryWhy || '—'],
    ['Risk', joinDot([row.entryRisk, row.riskDistancePct != null ? fmtPct(row.riskDistancePct) : null]) || '—'],
    ['Horizon', row.entryHorizon || '—'],
    ['As of', fmtIst(row.snapUnix)],
  ];

  return (
    <div style={{
      borderRadius: '6px',
      border: `1px solid ${act ? '#0369a144' : 'var(--border-color)'}`,
      backgroundColor: act ? '#0369a10d' : 'rgba(0,0,0,0.12)',
      fontVariantNumeric: 'tabular-nums',
    }}>
      <button
        type="button"
        onClick={onToggle}
        style={{
          display: 'block',
          width: '100%',
          textAlign: 'left',
          background: 'transparent',
          border: 'none',
          padding: '0.55rem 0.65rem',
          cursor: 'pointer',
          color: 'inherit',
        }}
      >
        <div style={{ display: 'flex', justifyContent: 'space-between', gap: '0.5rem', alignItems: 'baseline' }}>
          <div style={{ fontSize: '0.78rem', fontWeight: 800, letterSpacing: '0.01em' }}>
            {ticker}
            {' · '}
            <span style={{ color: recommendationColor(row.recommendation) }}>{row.recommendation}</span>
            {' · '}
            <span style={{ color: actColor }}>{row.action}</span>
            {row.direction ? (
              <>
                {' · '}
                <span style={{ color: dirColor }}>{row.direction}</span>
              </>
            ) : null}
          </div>
          <div style={{ fontSize: '0.58rem', fontWeight: 700, color: 'var(--text-secondary)', letterSpacing: '0.04em', textTransform: 'uppercase' }}>
            {open ? 'Hide evidence' : 'Evidence'}
          </div>
        </div>
        <div style={{ fontSize: '0.68rem', color: 'var(--text-secondary)', marginTop: '0.22rem', lineHeight: 1.45 }}>
          {joinDot([oqsLine, rankLine]) || '—'}
        </div>
        {histLine && (
          <div style={{ fontSize: '0.68rem', color: 'var(--text-primary)', marginTop: '0.08rem', lineHeight: 1.45 }}>
            {histLine}
          </div>
        )}
        {riskLine && (
          <div style={{ fontSize: '0.68rem', color: 'var(--text-secondary)', marginTop: '0.08rem', lineHeight: 1.45 }}>
            {riskLine}
          </div>
        )}
        <div style={{ fontSize: '0.68rem', color: 'var(--text-secondary)', marginTop: '0.08rem', lineHeight: 1.45 }}>
          Observed fill {fmtPrice(row.paperEntry)}
        </div>
        <div style={{ fontSize: '0.68rem', marginTop: '0.08rem', lineHeight: 1.45, fontWeight: 700 }}>
          <span style={{ color: markC }}>{row.mark == null ? 'Mark —' : `Mark ${fmtPct(row.mark)}${row.markKind ? ` ${row.markKind}` : ''}`}</span>
          {row.paperState !== 'NONE' && (
            <span style={{ color: paperStateColor(row.paperState) }}>{` · ${row.paperState}`}</span>
          )}
          {row.toStopPct != null ? ` · to STOP ${fmtPct(row.toStopPct)}` : ''}
          {row.toTargetPct != null ? ` · to TARGET ${fmtPct(row.toTargetPct)}` : ''}
        </div>
        <LifecycleLines row={row} />
      </button>
      {open && (
        <div style={{
          margin: '0 0.55rem 0.55rem',
          padding: '0.5rem 0.6rem',
          borderRadius: '5px',
          backgroundColor: 'rgba(0,0,0,0.16)',
          border: '1px solid var(--border-color)',
        }}>
          <div style={{
            fontSize: '0.58rem', color: 'var(--text-secondary)',
            textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '0.35rem',
          }}>
            Decision Evidence · existing IC v1 facts · not a new score
          </div>
          {evidenceRows.map(([label, value]) => (
            <div key={label} style={{ display: 'flex', justifyContent: 'space-between', gap: '0.5rem', marginBottom: '0.18rem' }}>
              <span style={{ fontSize: '0.65rem', color: 'var(--text-secondary)' }}>{label}</span>
              <span style={{
                fontSize: '0.65rem', color: 'var(--text-primary)', fontWeight: 600,
                textAlign: 'right', maxWidth: '62%', wordBreak: 'break-word',
              }}>{value}</span>
            </div>
          ))}
          {row.decisionId ? (
            <button
              type="button"
              onClick={() => onViewDecision(row.decisionId)}
              style={{
                marginTop: '0.4rem',
                background: 'transparent',
                border: 'none',
                padding: 0,
                cursor: 'pointer',
                color: '#0369a1',
                fontSize: '0.62rem',
                fontWeight: 700,
              }}
            >
              Open DecisionBrief
            </button>
          ) : null}
        </div>
      )}
    </div>
  );
}

function paperReasonColor(reason: string): string {
  switch (reason) {
    case 'STOP': return '#ef4444';
    case 'TARGET': return '#22c55e';
    case 'HORIZON': return '#6b7280';
    default: return 'var(--text-secondary)';
  }
}

function horizonUnix(h: DeferredLivePosition['horizon'] | undefined): number | null {
  if (!h) return null;
  if ('Unix' in h && h.Unix?.horizon_unix != null) return h.Unix.horizon_unix;
  return null;
}

function DeferredLivePerformancePanel({ performance, accent, sessionDate }: {
  performance: DeferredLivePerformance;
  accent: string;
  sessionDate?: string | null;
}) {
  const rows = performance.rows ?? [];
  return (
    <div style={{ margin: '0 0 0.85rem' }}>
      <div style={{ fontSize: '0.62rem', color: accent, fontWeight: 700, letterSpacing: '0.04em', textTransform: 'uppercase', marginBottom: '0.35rem' }}>
        {sessionDate ? `Session marks · ${sessionDate}` : 'Live marks'} · not a predictive-value claim
      </div>
      <div style={{ fontSize: '0.65rem', color: 'var(--text-secondary)', marginBottom: '0.45rem' }}>
        entered {performance.n_entered ?? 0}
        {' · '}OPEN {performance.n_open ?? 0}
        {' · '}TARGET {performance.n_target ?? 0}
        {' · '}STOP {performance.n_stop ?? 0}
        {' · '}HORIZON {performance.n_horizon ?? 0}
        {' · '}closed wins {performance.n_win_closed ?? 0}
        {' · '}mean closed {fmtPct(performance.mean_closed_return)}
        {' · '}mean open mark {fmtPct(performance.mean_open_mark)}
        {' · '}mean fill vs brief {fmtPct(performance.mean_fill_vs_decision)}
      </div>
      <div style={{ fontSize: '0.62rem', color: 'var(--text-secondary)', marginBottom: '0.45rem', lineHeight: 1.4 }}>
        {performance.note}
      </div>
      {rows.length > 0 && (
        <div style={{ overflowX: 'auto' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '0.65rem' }}>
            <thead>
              <tr style={{ textAlign: 'left', color: 'var(--text-secondary)' }}>
                {['Ticker', 'Dir', 'Outcome', 'Brief', 'Paper', 'Last', 'Kind', 'Ret'].map(h => (
                  <th key={h} style={{ padding: '0.2rem 0.35rem', fontWeight: 600 }}>{h}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {rows.map(r => (
                <tr key={r.decision_id || r.ticker}>
                  <td style={{ padding: '0.2rem 0.35rem' }}>{(r.ticker ?? '').replace('_NS', '')}</td>
                  <td style={{ padding: '0.2rem 0.35rem' }}>{r.direction}</td>
                  <td style={{ padding: '0.2rem 0.35rem' }}>{r.outcome}</td>
                  <td style={{ padding: '0.2rem 0.35rem' }}>{r.decision_entry_price != null ? r.decision_entry_price.toFixed(2) : '—'}</td>
                  <td style={{ padding: '0.2rem 0.35rem' }}>{r.paper_entry_price?.toFixed(2) ?? '—'}</td>
                  <td style={{ padding: '0.2rem 0.35rem' }}>{r.last_or_exit_price?.toFixed(2) ?? '—'}</td>
                  <td style={{ padding: '0.2rem 0.35rem' }}>{r.ret_kind}</td>
                  <td style={{ padding: '0.2rem 0.35rem', color: (r.ret ?? 0) > 0 ? '#16a34a' : (r.ret ?? 0) < 0 ? '#dc2626' : 'inherit' }}>
                    {fmtPct(r.ret)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

function ReassessExperimentPanel({ experiment, accent }: {
  experiment: ReassessExperimentReport;
  accent: string;
}) {
  const rows = experiment.rows ?? [];
  return (
    <div style={{ margin: '0 0 0.85rem' }}>
      <div style={{ fontSize: '0.62rem', color: accent, fontWeight: 700, letterSpacing: '0.04em', textTransform: 'uppercase', marginBottom: '0.35rem' }}>
        Live-paper reassess experiment · not Stage C
      </div>
      <div style={{ fontSize: '0.65rem', color: 'var(--text-secondary)', marginBottom: '0.45rem' }}>
        rule {experiment.rule ?? '—'}
        {' · '}INVERT {experiment.invert_enabled ? 'on' : 'off'}
      </div>
      <div style={{ fontSize: '0.65rem', color: 'var(--text-secondary)', marginBottom: '0.25rem' }}>
        baseline v0.2 · triggered {experiment.baseline?.n_triggered ?? 0}
        {' · '}helped {experiment.baseline?.n_helped ?? 0}
        {' · '}hurt {experiment.baseline?.n_hurt ?? 0}
        {' · '}mean Δ {fmtPct(experiment.baseline?.mean_delta_vs_frozen)}
      </div>
      <div style={{ fontSize: '0.65rem', color: 'var(--text-secondary)', marginBottom: '0.45rem' }}>
        gated mark-adverse · triggered {experiment.n_triggered ?? 0}
        {' · '}helped {experiment.n_helped ?? 0}
        {' · '}hurt {experiment.n_hurt ?? 0}
        {' · '}suppressed {experiment.n_suppressed_by_mark_gate ?? 0}
        {' · '}mean Δ {fmtPct(experiment.mean_delta_vs_frozen)}
      </div>
      <div style={{ fontSize: '0.62rem', color: 'var(--text-secondary)', marginBottom: '0.45rem', lineHeight: 1.4 }}>
        {experiment.note}
      </div>
      {rows.length > 0 && (
        <div style={{ overflowX: 'auto' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '0.65rem' }}>
            <thead>
              <tr style={{ textAlign: 'left', color: 'var(--text-secondary)' }}>
                {['Ticker', 'Frozen', 'Frozen ret', 'v0.2', 'v0.2 Δ', 'Gated', 'Exp ret', 'Δ vs frozen', 'Gate'].map(h => (
                  <th key={h} style={{ padding: '0.2rem 0.35rem', fontWeight: 600 }}>{h}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {rows.map(r => (
                <tr key={r.decision_id || r.ticker}>
                  <td style={{ padding: '0.2rem 0.35rem' }}>{(r.ticker ?? '').replace('_NS', '')}</td>
                  <td style={{ padding: '0.2rem 0.35rem' }}>{r.frozen_outcome}</td>
                  <td style={{ padding: '0.2rem 0.35rem', color: (r.frozen_ret ?? 0) > 0 ? '#16a34a' : (r.frozen_ret ?? 0) < 0 ? '#dc2626' : 'inherit' }}>
                    {fmtPct(r.frozen_ret)}
                  </td>
                  <td style={{ padding: '0.2rem 0.35rem' }}>{r.v02_triggered ? 'YES' : 'no'}</td>
                  <td style={{ padding: '0.2rem 0.35rem', color: (r.v02_delta_vs_frozen ?? 0) > 0 ? '#16a34a' : (r.v02_delta_vs_frozen ?? 0) < 0 ? '#dc2626' : 'inherit' }}>
                    {fmtPct(r.v02_delta_vs_frozen)}
                  </td>
                  <td style={{ padding: '0.2rem 0.35rem' }}>{r.reassess_triggered ? 'YES' : 'no'}</td>
                  <td style={{ padding: '0.2rem 0.35rem' }}>{fmtPct(r.experiment_ret)}</td>
                  <td style={{ padding: '0.2rem 0.35rem', color: (r.delta_vs_frozen ?? 0) > 0 ? '#16a34a' : (r.delta_vs_frozen ?? 0) < 0 ? '#dc2626' : 'inherit' }}>
                    {fmtPct(r.delta_vs_frozen)}
                  </td>
                  <td style={{ padding: '0.2rem 0.35rem' }}>
                    {r.suppressed_by_mark_gate ? 'held' : r.reassess_triggered ? 'exit' : '—'}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

function liveStatus(pos: DeferredLivePosition): string {
  const reason = pos.walk?.exit?.reason ?? pos.paper?.exit_reason;
  if (!reason) return 'OPEN';
  const r = String(reason).toUpperCase();
  if (r === 'HORIZON') return 'HORIZON';
  if (r === 'TARGET' || r === 'STOP') return 'EXITED';
  return 'OPEN';
}

function liveAction(pos: DeferredLivePosition): string {
  const st = liveStatus(pos);
  if (st === 'OPEN') return 'OPEN';
  if (st === 'HORIZON') return 'HORIZON';
  return pos.walk?.exit?.reason?.toUpperCase() ?? pos.paper?.exit_reason ?? 'EXIT';
}

function feedIsClockSim(feed: ObservationFeedSnapshot | undefined): boolean {
  if (!feed) return false;
  const src = feed.observation_source ?? feed.background ?? feed.last_source_kind;
  return feed.clock_simulation === true
    || src === 'CACHED_1M'
    || src === 'YAHOO_1M'
    || src === 'CLOCK_SIMULATION';
}

function feedIsLiveProducer(feed: ObservationFeedSnapshot | undefined): boolean {
  if (!feed || feedIsClockSim(feed)) return false;
  return feed.background === 'EXTERNAL_LIVE'
    || feed.last_source_kind === 'EXTERNAL_LIVE'
    || feed.status === 'POLLING'
    || feed.status === 'WAITING_FOR_INBOX';
}

function DeferredLivePanel({ live, error, updatedAt, lastIngestSeenAt, decisions, ctxById, onViewDecision }: {
  live: DeferredLiveResponse | null;
  error: string | null;
  updatedAt: number | null;
  lastIngestSeenAt: number | null;
  decisions: DecisionBrief[];
  ctxById?: Map<string, DecisionPortfolioContext>;
  onViewDecision: (id: string) => void;
}) {
  const positions = live?.ledger?.positions ?? [];
  const armed = live?.armed ?? [];
  const feed = live?.observation_feed;
  const session = live?.session;
  const clockSim = feedIsClockSim(feed);
  const liveProducer = feedIsLiveProducer(feed);
  const watching = updatedAt != null
    ? `watching GET /deferred-live · ${new Date(updatedAt).toLocaleTimeString('en-GB', { hour12: false })}`
    : 'connecting to GET /api/v1/intraday/deferred-live';
  const border = clockSim ? '#f59e0b88' : '#0ea5e988';
  const bg = clockSim ? '#f59e0b12' : '#0ea5e912';
  const accent = clockSim ? '#b45309' : '#0369a1';

  return (
    <div style={{
      marginBottom: '1.5rem',
      padding: '0.9rem 1rem',
      borderRadius: '8px',
      border: `1px solid ${border}`,
      backgroundColor: bg,
    }}>
      <div style={{ display: 'flex', alignItems: 'baseline', justifyContent: 'space-between', gap: '0.75rem', flexWrap: 'wrap' }}>
        <div>
          <div style={{
            fontSize: '0.72rem', color: accent,
            textTransform: 'uppercase', letterSpacing: '0.05em', fontWeight: 800,
          }}>
            ● Deferred Live paper book
          </div>
          <div style={{ fontSize: '0.68rem', color: 'var(--text-secondary)', marginTop: '0.2rem' }}>
            {clockSim
              ? `Controlled clock · ${feed?.observation_source ?? feed?.background ?? 'CACHED_1M'} · not a broker feed`
              : 'Paper execution · no broker · Cockpit does not trade'}
            {session?.auto_arm
              ? ` · session ${session.date} · auto-arm ACT · no POST /arm`
              : ''}
          </div>
        </div>
        <div style={{ fontSize: '0.62rem', color: accent, fontWeight: 600 }}>
          {watching}
        </div>
      </div>
      <NowActionStrip live={live} decisions={decisions} ctxById={ctxById} lastIngestSeenAt={lastIngestSeenAt} onViewDecision={onViewDecision} />
      <div style={{ fontSize: '0.62rem', color: 'var(--text-secondary)', margin: '0.15rem 0 0.35rem' }}>
        Source: <code>/api/v1/intraday/deferred-live</code>
        {' · '}mode {live?.mode ?? '—'}
        {' · '}source {feed?.observation_source ?? feed?.background ?? '—'}
        {' · '}{feed?.status ?? '—'}
        {' · '}{armed.length} armed
        {' · '}{positions.length} paper positions.{' '}
        {session?.auto_arm
          ? `Session ${session.date}: ${session.eligible ?? 0} ACT eligible · ${session.skipped_not_act ?? 0} NO TRADE. `
          : ''}
        Fills are observed <code>paper_entry_price</code> — not DecisionBrief.entry_price, not CSV replay.
      </div>
      {feed?.note && (
        <div style={{
          fontSize: '0.68rem',
          color: clockSim ? '#b45309' : 'var(--text-secondary)',
          margin: '0 0 0.75rem',
          lineHeight: 1.45,
        }}>
          {feed.note}
          {feed.inbox_path ? ` · ${feed.inbox_path}` : ''}
        </div>
      )}
      {error && (
        <div style={{ fontSize: '0.68rem', color: '#ef4444', marginBottom: '0.65rem' }}>{error}</div>
      )}
      {live?.performance && (live.performance.n_entered ?? 0) > 0 && (
        <DeferredLivePerformancePanel
          performance={live.performance}
          accent={accent}
          sessionDate={session?.date ?? null}
        />
      )}
      {live?.reassess_experiment?.enabled && (
        <ReassessExperimentPanel experiment={live.reassess_experiment} accent={accent} />
      )}
      {positions.length === 0 ? (
        <div style={{
          padding: '0.85rem 0.9rem',
          borderRadius: '6px',
          border: `1px dashed ${border}`,
          backgroundColor: 'var(--bg-card)',
          fontSize: '0.72rem',
          color: 'var(--text-secondary)',
          lineHeight: 1.5,
        }}>
          {clockSim
            ? (session?.auto_arm
              ? 'Clock simulation is attached. If the book is empty, no ACT DecisionBrief has filled yet — MONITOR/AVOID stay NO TRADE.'
              : 'Clock simulation is attached but the live paper book is empty. Cached 1m bars are not the current session.')
            : liveProducer
              ? 'No positions currently open. The observation producer is attached — this is not a missing historical blotter.'
              : 'No positions currently open. That is the current deferred-live state — not a missing historical blotter. Yahoo is not authorized. Cached 1m bars are not a live feed. Arm a DecisionBrief and supply live MarketObservation events (JSONL inbox or POST).'}
        </div>
      ) : (
        <DeferredLiveBooks
          positions={positions}
          clockSim={clockSim}
          session={!!session?.auto_arm}
          onViewDecision={onViewDecision}
        />
      )}
    </div>
  );
}

function isClosedLivePosition(pos: DeferredLivePosition): boolean {
  const reason = (pos.walk?.exit?.reason ?? pos.paper?.exit_reason ?? '').toUpperCase();
  return reason === 'TARGET' || reason === 'STOP' || reason === 'HORIZON';
}

function DeferredLiveBooks({
  positions,
  clockSim,
  session,
  onViewDecision,
}: {
  positions: DeferredLivePosition[];
  clockSim?: boolean;
  session?: boolean;
  onViewDecision: (id: string) => void;
}) {
  const open = positions.filter(p => !isClosedLivePosition(p));
  const closed = positions.filter(isClosedLivePosition);
  const render = (list: DeferredLivePosition[]) => (
    <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.75rem' }}>
      {list.map((pos, i) => (
        <LivePaperCard
          key={`${pos.paper?.decision_id ?? pos.paper?.ticker ?? i}-${pos.opened_at}`}
          pos={pos}
          clockSim={clockSim}
          onViewDecision={onViewDecision}
        />
      ))}
    </div>
  );
  return (
    <div>
      <div style={{ fontSize: '0.62rem', fontWeight: 800, letterSpacing: '0.05em', textTransform: 'uppercase', color: 'var(--text-secondary)', margin: '0.35rem 0 0.4rem' }}>
        Live book · open paper
      </div>
      {open.length === 0
        ? <div style={{ fontSize: '0.68rem', color: 'var(--text-secondary)', marginBottom: '0.75rem' }}>No open paper. Closed names are in the record below.</div>
        : <div style={{ marginBottom: '0.85rem' }}>{render(open)}</div>}
      <div style={{ fontSize: '0.62rem', fontWeight: 800, letterSpacing: '0.05em', textTransform: 'uppercase', color: 'var(--text-secondary)', margin: '0.15rem 0 0.4rem' }}>
        {session ? 'Session record · TARGET / STOP / HORIZON' : 'Closed record · TARGET / STOP / HORIZON'}
      </div>
      {closed.length === 0
        ? <div style={{ fontSize: '0.68rem', color: 'var(--text-secondary)' }}>{session ? 'No closed paper this session.' : 'No closed paper on this live book.'}</div>
        : render(closed)}
    </div>
  );
}

function LivePaperCard({ pos, clockSim, onViewDecision }: {
  pos: DeferredLivePosition;
  clockSim?: boolean;
  onViewDecision: (id: string) => void;
}) {
  const paper = pos.paper;
  const hz = horizonUnix(pos.horizon);
  const remainMin = hz != null && pos.last_tick_at != null
    ? Math.max(0, Math.round((hz - pos.last_tick_at) / 60))
    : null;
  const entry = paper?.paper_entry_price;
  const current = pos.current_price;
  const pnl = entry && entry !== 0 && current != null
    ? ((paper.direction === 'SHORT' ? (entry - current) : (current - entry)) / entry)
    : paper?.realized_return ?? null;
  const st = liveStatus(pos);
  const action = liveAction(pos);
  const dirC = paper?.direction === 'LONG' ? '#22c55e' : '#ef4444';
  const id = paper?.decision_id;
  const pnlC = pnl != null && pnl > 0 ? '#22c55e' : pnl != null && pnl < 0 ? '#ef4444' : 'var(--text-primary)';

  return (
    <div style={{
      flex: '1 1 300px',
      minWidth: '280px',
      backgroundColor: 'var(--bg-card)',
      border: '1px solid #0ea5e944',
      borderTop: `3px solid ${st === 'OPEN' ? '#0ea5e9' : dirC}`,
      borderRadius: '8px',
      padding: '0.8rem 0.9rem',
    }}>
      <div style={{ fontSize: '0.55rem', fontWeight: 800, letterSpacing: '0.06em', color: clockSim ? '#b45309' : '#0369a1', textTransform: 'uppercase', marginBottom: '0.35rem' }}>
        {clockSim ? 'Controlled clock · not a broker order' : 'Paper position · not a broker order'}
      </div>
      <div style={{ display: 'flex', alignItems: 'center', gap: '0.4rem', flexWrap: 'wrap' }}>
        <span style={{ fontSize: '1.05rem', fontWeight: 800, color: 'var(--text-primary)' }}>
          {(paper?.ticker ?? '—').replace('_NS', '')}
        </span>
        <span style={{ fontSize: '0.68rem', fontWeight: 800, color: dirC }}>{paper?.direction}</span>
        {paper?.oqs != null && (
          <span style={{ fontSize: '0.62rem', color: 'var(--text-secondary)' }}>OQS {paper.oqs}</span>
        )}
        <span style={{
          marginLeft: 'auto', fontSize: '0.58rem', fontWeight: 800, letterSpacing: '0.04em',
          color: st === 'OPEN' ? '#0369a1' : '#6b7280',
        }}>{st === 'OPEN' ? 'HOLD · OPEN' : `${st} · ${action}`}</span>
      </div>
      <div style={{ marginTop: '0.55rem', fontSize: '1.35rem', fontWeight: 800, fontVariantNumeric: 'tabular-nums', color: 'var(--text-primary)' }}>
        {fmtPrice(current)}
      </div>
      <div style={{ fontSize: '0.62rem', color: 'var(--text-secondary)', marginBottom: '0.5rem' }}>
        current_price · last observation {fmtIst(pos.last_tick_at)}
        {clockSim ? ' · controlled clock' : ` · ${fmtAge(pos.last_tick_at)}`}
      </div>
      <div style={{ fontSize: '0.68rem', fontVariantNumeric: 'tabular-nums' }}>
        {[
          ['Observed fill', fmtPrice(entry)],
          [st === 'OPEN' ? 'Mark' : 'Realized', fmtPct(pnl)],
          ['Target', fmtPrice(paper?.paper_target ?? pos.walk?.target)],
          ['Risk', fmtPrice(paper?.paper_risk ?? pos.walk?.stop)],
          ['Horizon', hz != null ? `H300 · ${remainMin} min remaining` : '—'],
          ['Opened', fmtIst(pos.opened_at)],
        ].map(([k, v]) => (
          <div key={k} style={{ display: 'flex', justifyContent: 'space-between', gap: '0.5rem' }}>
            <span style={{ color: 'var(--text-secondary)' }}>{k}</span>
            <span style={{ fontWeight: 600, color: k === 'Mark' || k === 'Realized' ? pnlC : 'var(--text-primary)', textAlign: 'right' }}>{v}</span>
          </div>
        ))}
      </div>
      {(pos.events ?? []).length > 0 && (
        <div style={{ marginTop: '0.6rem', paddingTop: '0.45rem', borderTop: '1px solid var(--border-color)', fontSize: '0.62rem', color: 'var(--text-secondary)' }}>
          {(pos.events ?? []).map((ev, j) => (
            <div key={`${ev.unix}-${ev.kind}-${j}`}>
              {fmtIst(ev.unix)} · {ev.kind}{ev.note ? ` ${ev.note}` : ''} · {fmtPrice(ev.price)}
            </div>
          ))}
        </div>
      )}
      {id && (
        <button
          onClick={() => onViewDecision(id)}
          style={{
            marginTop: '0.65rem', width: '100%',
            padding: '0.3rem 0.55rem', borderRadius: '5px', cursor: 'pointer',
            background: 'transparent', border: '1px solid #0ea5e944',
            color: '#0369a1', fontSize: '0.65rem', fontWeight: 600,
          }}
        >
          Open DecisionBrief
        </button>
      )}
    </div>
  );
}

function PaperBlotterPanel({
  blotter,
  decisions,
  onViewDecision,
}: {
  blotter: PaperBlotter;
  decisions: DecisionBrief[];
  onViewDecision: (id: string) => void;
}) {
  const briefById = useMemo(() => {
    const m = new Map<string, DecisionBrief>();
    for (const d of decisions) m.set(d.id, d);
    return m;
  }, [decisions]);

  const byDate = new Map<string, PaperPosition[]>();
  for (const p of blotter.positions ?? []) {
    if (!byDate.has(p.date)) byDate.set(p.date, []);
    byDate.get(p.date)!.push(p);
  }
  const winPct = blotter.total > 0 ? `${((blotter.n_win / blotter.total) * 100).toFixed(0)}%` : '—';
  const nJoined = (blotter.positions ?? []).filter(p => p.decision_id && briefById.has(p.decision_id)).length;
  const nDates = byDate.size;
  const th: React.CSSProperties = {
    textAlign: 'left', fontSize: '0.52rem', fontWeight: 600,
    color: 'var(--text-secondary)', textTransform: 'uppercase',
    letterSpacing: '0.04em', padding: '0.35rem 0.45rem',
    borderBottom: '1px solid var(--border-color)', whiteSpace: 'nowrap',
  };
  const td: React.CSSProperties = {
    padding: '0.35rem 0.45rem', fontSize: '0.66rem', whiteSpace: 'nowrap',
  };

  return (
    <div style={{ marginBottom: '1.5rem' }}>
      <div style={{
        fontSize: '0.65rem', color: 'var(--text-secondary)',
        textTransform: 'uppercase', letterSpacing: '0.05em',
        marginBottom: '0.35rem',
      }}>
        Paper blotter · Historical Replay · frozen historical paper trading
      </div>
      <div style={{
        display: 'flex', flexWrap: 'wrap', gap: '0.75rem',
        marginBottom: '0.35rem',
        fontSize: '0.7rem', color: 'var(--text-primary)',
      }}>
        <span>{blotter.total} positions / {nDates} dates</span>
        <span>{nJoined} joined to DecisionBrief</span>
        <span>STOP {blotter.n_stop}</span>
        <span>TARGET {blotter.n_target}</span>
        <span>HORIZON {blotter.n_horizon}</span>
        <span>win {winPct}</span>
        <span>mean {fmtPct(blotter.mean_realized_return)}</span>
        <span>sum {fmtPct(blotter.total_realized_return)}</span>
      </div>
      <div style={{ fontSize: '0.62rem', color: 'var(--text-secondary)', marginBottom: '0.75rem', lineHeight: 1.45 }}>
        paper_entry_price is the actual replay fill. DecisionBrief.entry_price is decision/reference data.
        The fill must never overwrite the adapter price. Sum is aggregate trade-return percentage points, not a portfolio return.
        Joined rows open the matching DecisionBrief.
      </div>
      {[...byDate.entries()].map(([date, rows]) => (
        <div key={date} style={{ marginBottom: '0.85rem' }}>
          <div style={{
            fontSize: '0.62rem', color: 'var(--text-secondary)',
            textTransform: 'uppercase', letterSpacing: '0.06em',
            marginBottom: '0.35rem',
          }}>
            {date} · {rows.length}
          </div>
          <div style={{ overflowX: 'auto', border: '1px solid var(--border-color)', borderRadius: '6px' }}>
            <table style={{ width: '100%', borderCollapse: 'collapse', fontVariantNumeric: 'tabular-nums' }}>
              <thead>
                <tr>
                  {['ticker', 'dir', 'action', 'paper_fill', 'entry_price', 'reference', 'exit', 'reason', 'ret', 'lifecycle'].map(h => (
                    <th key={h} style={th}>{h}</th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {rows.map((p, i) => {
                  const brief = p.decision_id ? briefById.get(p.decision_id) : undefined;
                  const joined = Boolean(p.decision_id && brief);
                  const retC = p.realized_return != null && p.realized_return > 0 ? '#22c55e'
                    : p.realized_return != null && p.realized_return < 0 ? '#ef4444'
                    : 'var(--text-secondary)';
                  return (
                    <tr
                      key={`${p.ticker}-${p.date}-${i}`}
                      onClick={() => { if (joined && p.decision_id) onViewDecision(p.decision_id); }}
                      title={joined ? `Open ${p.decision_id}` : 'No DecisionBrief join'}
                      style={{
                        cursor: joined ? 'pointer' : 'default',
                        opacity: joined ? 1 : 0.62,
                        borderBottom: '1px solid var(--border-color)',
                      }}
                    >
                      <td style={{ ...td, fontSize: '0.72rem', fontWeight: 600, color: 'var(--text-primary)' }}>
                        {p.ticker.replace('_NS', '')}
                      </td>
                      <td style={{ ...td, color: p.direction === 'LONG' ? '#22c55e' : '#ef4444', fontWeight: 700 }}>
                        {p.direction}
                      </td>
                      <td style={{ ...td, color: actionColor(p.entry_action ?? ''), fontWeight: 700 }}>
                        {p.entry_action ?? '—'}
                      </td>
                      <td style={{ ...td, fontWeight: 700 }}>{fmtPrice(p.paper_entry_price)}</td>
                      <td style={td}>{joined ? fmtPrice(brief?.entry_price) : '—'}</td>
                      <td style={td}>{joined ? fmtPrice(brief?.reference_price) : '—'}</td>
                      <td style={td}>
                        {fmtPrice(p.paper_exit_price)}
                        {p.exit_time_ist ? ` · ${p.exit_time_ist}` : ''}
                      </td>
                      <td style={{ ...td, fontWeight: 700, color: paperReasonColor(p.exit_reason) }}>
                        {p.exit_reason}
                      </td>
                      <td style={{ ...td, fontWeight: 600, color: retC }}>
                        {fmtPct(p.realized_return)}
                      </td>
                      <td style={{
                        ...td,
                        fontSize: '0.6rem',
                        fontWeight: 700,
                        color: joined ? '#3b82f6' : 'var(--text-secondary)',
                      }}>
                        {joined ? 'Open decision' : 'no DecisionBrief'}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </div>
      ))}
    </div>
  );
}

function ReturnCell({ value }: { value: number | null | undefined }) {
  if (value == null) return <span style={{ color: 'var(--text-secondary)' }}>—</span>;
  const c = value > 0.002 ? '#22c55e' : value < -0.002 ? '#ef4444' : 'var(--text-secondary)';
  return <span style={{ color: c, fontWeight: 600, fontVariantNumeric: 'tabular-nums' }}>{fmtRet(value)}</span>;
}

// ── Checkpoint row ────────────────────────────────────────────────────────────

function CheckpointRow({
  label, time, state, action, confidence, horizon, why, risk, retValue,
  histWin, histPf, histMed, isLast,
}: {
  label: string; time: string;
  state: string; action: string; confidence: string;
  horizon: string; why: string; risk: string;
  retValue?: number | null;
  histWin?: number | null; histPf?: number | null; histMed?: number | null;
  isLast?: boolean;
}) {
  const [open, setOpen] = useState(true);
  const ac = actionColor(action);
  const hasHist = histWin != null;

  return (
    <div style={{ display: 'flex', gap: '0.875rem' }}>
      {/* Spine */}
      <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', width: '30px', flexShrink: 0 }}>
        <div style={{
          width: '30px', height: '30px', borderRadius: '50%', flexShrink: 0,
          backgroundColor: ac + '22', border: `2px solid ${ac}`,
          display: 'flex', alignItems: 'center', justifyContent: 'center',
          fontSize: '0.6rem', fontWeight: 700, color: ac,
        }}>
          {label}
        </div>
        {!isLast && <div style={{ width: '2px', flex: 1, backgroundColor: 'var(--border-color)', minHeight: '20px' }} />}
      </div>

      {/* Card */}
      <div style={{
        flex: 1, backgroundColor: 'var(--bg-card)', border: '1px solid var(--border-color)',
        borderRadius: '8px', marginBottom: isLast ? 0 : '0.625rem', overflow: 'hidden',
      }}>
        <div
          style={{
            display: 'flex', alignItems: 'center', gap: '0.625rem', padding: '0.5rem 0.75rem',
            cursor: 'pointer', borderBottom: open ? '1px solid var(--border-color)' : 'none',
          }}
          onClick={() => setOpen(o => !o)}
        >
          <span style={{ fontSize: '0.65rem', color: 'var(--text-secondary)', width: '3rem', flexShrink: 0 }}>{time}</span>
          <StateBadge state={state} />
          <ActionBadge action={action} />
          <ConfidenceDot confidence={confidence} />
          {retValue != null && (
            <span style={{ marginLeft: 'auto', fontSize: '0.8rem' }}>
              <ReturnCell value={retValue} />
            </span>
          )}
          <span style={{ marginLeft: retValue != null ? '0' : 'auto', color: 'var(--text-secondary)', fontSize: '0.7rem' }}>
            {open ? '▲' : '▼'}
          </span>
        </div>

        {open && (
          <div style={{ padding: '0.625rem 0.75rem', display: 'flex', flexDirection: 'column', gap: '0.4rem' }}>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '0.5rem' }}>
              <div>
                <div style={{ fontSize: '0.6rem', color: 'var(--text-secondary)', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '0.15rem' }}>Horizon</div>
                <div style={{ fontSize: '0.78rem', color: 'var(--text-primary)' }}>{horizon}</div>
              </div>
              {hasHist && (
                <div>
                  <div style={{ fontSize: '0.6rem', color: 'var(--text-secondary)', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '0.15rem' }}>Historical</div>
                  <div style={{ fontSize: '0.78rem', color: 'var(--text-primary)' }}>
                    {((histWin ?? 0) * 100).toFixed(0)}% win
                    {histPf != null ? ` · PF ${histPf >= 99 ? '>99' : histPf.toFixed(2)}x` : ''}
                    {histMed != null ? ` · med ${fmtRet(histMed)}` : ''}
                  </div>
                </div>
              )}
            </div>
            <div>
              <div style={{ fontSize: '0.6rem', color: 'var(--text-secondary)', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '0.15rem' }}>Why</div>
              <div style={{ fontSize: '0.78rem', color: 'var(--text-primary)' }}>{why}</div>
            </div>
            <div>
              <div style={{ fontSize: '0.6rem', color: '#f59e0b', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '0.15rem' }}>Risk</div>
              <div style={{ fontSize: '0.78rem', color: 'var(--text-secondary)' }}>{risk}</div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

// ── Decision detail ───────────────────────────────────────────────────────────

function PaperReplayOverlay({ d, paper }: { d: DecisionBrief; paper: PaperPosition }) {
  const retC = paper.realized_return != null && paper.realized_return > 0 ? '#22c55e'
    : paper.realized_return != null && paper.realized_return < 0 ? '#ef4444'
    : 'var(--text-secondary)';
  const rows: [string, string][] = [
    ['paper_entry_price (replay fill)', fmtPrice(paper.paper_entry_price)],
    ['DecisionBrief.entry_price', fmtPrice(d.entry_price)],
    ['DecisionBrief.reference_price', fmtPrice(d.reference_price)],
    ['paper_exit_price', `${fmtPrice(paper.paper_exit_price)}${paper.exit_time_ist ? ` · ${paper.exit_time_ist}` : ''}`],
    ['exit_reason', paper.exit_reason],
    ['realized_return', fmtPct(paper.realized_return)],
    ['bars_held', String(paper.bars_held)],
    ['entry_action', paper.entry_action ?? d.entry_action],
    ['outcome', d.outcome ?? '—'],
  ];
  return (
    <div style={{
      margin: '0.75rem 0 1.25rem',
      padding: '0.7rem 0.85rem',
      borderRadius: '8px',
      backgroundColor: '#f59e0b12',
      border: '1px solid #f59e0b66',
    }}>
      <div style={{
        fontSize: '0.72rem', fontWeight: 800, letterSpacing: '0.04em',
        color: '#f59e0b', textTransform: 'uppercase', marginBottom: '0.2rem',
      }}>
        Historical paper replay · not live market execution
      </div>
      <div style={{ fontSize: '0.62rem', color: 'var(--text-secondary)', marginBottom: '0.5rem' }}>
        Replay fill is paper_entry_price. It does not overwrite DecisionBrief.entry_price.
      </div>
      {rows.map(([k, v]) => (
        <div key={k} style={{ display: 'flex', justifyContent: 'space-between', gap: '0.5rem' }}>
          <span style={{ fontSize: '0.62rem', color: 'var(--text-secondary)' }}>{k}</span>
          <span style={{
            fontSize: '0.62rem', fontWeight: 700, fontVariantNumeric: 'tabular-nums',
            color: k === 'realized_return' ? retC : 'var(--text-primary)',
          }}>{v}</span>
        </div>
      ))}
    </div>
  );
}

function liveMarkPct(pos: DeferredLivePosition): number | null {
  const entry = pos.paper?.paper_entry_price;
  const current = pos.current_price;
  if (entry != null && entry !== 0 && current != null) {
    const raw = (current - entry) / entry;
    return pos.paper?.direction === 'SHORT' ? -raw : raw;
  }
  return pos.paper?.realized_return ?? null;
}

function livePositionForBrief(
  live: DeferredLiveResponse | null | undefined,
  decisionId: string,
): DeferredLivePosition | null {
  return (live?.ledger?.positions ?? []).find(p => p.paper?.decision_id === decisionId) ?? null;
}

function LivePaperBriefBanner({ pos }: { pos: DeferredLivePosition }) {
  const st = liveStatus(pos);
  const mark = liveMarkPct(pos);
  const markKind = st === 'OPEN' ? 'MARK' : 'REALIZED';
  const markC = mark != null && mark > 0 ? '#22c55e' : mark != null && mark < 0 ? '#ef4444' : 'var(--text-primary)';
  const rows: [string, string, string | undefined][] = [
    ['Ledger', st, undefined],
    ['Observed fill', fmtPrice(pos.paper?.paper_entry_price), undefined],
    ['Last observation', fmtPrice(pos.current_price), undefined],
    [markKind, fmtPct(mark), markC],
  ];
  return (
    <div style={{
      margin: '0 0 1.25rem',
      padding: '0.7rem 0.85rem',
      borderRadius: '8px',
      backgroundColor: '#0ea5e912',
      border: '1px solid #0ea5e966',
    }}>
      <div style={{
        fontSize: '0.72rem', fontWeight: 800, letterSpacing: '0.04em',
        color: '#0369a1', textTransform: 'uppercase', marginBottom: '0.2rem',
      }}>
        Deferred Live paper · {st} · not IC H300 / outcome
      </div>
      <div style={{ fontSize: '0.62rem', color: 'var(--text-secondary)', marginBottom: '0.5rem', lineHeight: 1.45 }}>
        This name is on the live book. Fill and mark are the paper walk.
        H300, Outcome, and path returns below are the IC v1 DecisionBrief — they do not close or replace this position.
      </div>
      {rows.map(([k, v, c]) => (
        <div key={k} style={{ display: 'flex', justifyContent: 'space-between', gap: '0.5rem' }}>
          <span style={{ fontSize: '0.62rem', color: 'var(--text-secondary)' }}>{k}</span>
          <span style={{
            fontSize: '0.62rem', fontWeight: 700, fontVariantNumeric: 'tabular-nums',
            color: c ?? 'var(--text-primary)',
          }}>{v}</span>
        </div>
      ))}
    </div>
  );
}

function DecisionDetail({ d, onBack, portfolioCtx, paper, livePos }: {
  d: DecisionBrief;
  onBack: () => void;
  portfolioCtx?: DecisionPortfolioContext | null;
  paper?: PaperPosition | null;
  livePos?: DeferredLivePosition | null;
}) {
  const outcomeColor = d.outcome === 'WIN' ? '#22c55e' : d.outcome === 'LOSS' ? '#ef4444' : 'var(--text-secondary)';
  const icOnLiveBook = livePos != null;

  // H60 informational checkpoint — path observation, no reassessment
  const h60Why = `H60 return ${fmtRet(d.h60_ret)}${d.mfe_h60 != null ? ` · MFE ${fmtRet(d.mfe_h60)}` : ''}`;
  const h60Risk = icOnLiveBook
    ? 'IC H60 path observation · not a live paper action.'
    : (d.entry_state === 'WAIT-MID'
      ? 'H60 does not separate winners from losers for WAIT-MID; wait for H120.'
      : 'Watch for reversal after H60 peak (LONG) or continuation (SHORT).');

  // H180 informational checkpoint
  const h180Why = `H180 return ${fmtRet(d.h180_ret)} — ${
    d.h180_ret != null && d.h180_ret > 0.002 ? 'path sustained through 3-hour mark.' :
    d.h180_ret != null && d.h180_ret < -0.002 ? 'path reversed after strong early development.' :
    'opportunity developing slowly.'
  }`;

  // H300 checkpoint: IC outcome is a DecisionBrief field. Do not invent ACT/AVOID
  // from h300_ret — that reads as a live instruction next to an OPEN paper walk.
  const h300Action = icOnLiveBook
    ? (d.outcome ?? 'IC')
    : (d.h300_ret != null && d.h300_ret > 0.002 ? 'ACT' : 'AVOID');
  const h300Conf = icOnLiveBook ? 'IC' : (d.h300_ret != null ? 'HIGH' : 'LOW');
  const h300Why = `H300 return ${fmtRet(d.h300_ret)} — ${d.outcome ? `Outcome: ${d.outcome}` : (icOnLiveBook ? 'IC H300 path observation.' : 'session closed.')}`;
  const h300Risk = d.pnl != null
    ? `${icOnLiveBook ? 'IC daily return' : 'Daily return'}: ${fmtRet(d.pnl)}`
    : 'No daily return recorded.';

  return (
    <div>
      {/* Back + header */}
      <div style={{ display: 'flex', alignItems: 'flex-start', gap: '1rem', marginBottom: '1.25rem' }}>
        <button
          onClick={onBack}
          style={{
            background: 'var(--bg-card)', border: '1px solid var(--border-color)',
            borderRadius: '6px', padding: '0.3rem 0.7rem', cursor: 'pointer',
            color: 'var(--text-primary)', fontSize: '0.78rem', flexShrink: 0,
          }}
        >
          ← Back
        </button>
        <div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.625rem', flexWrap: 'wrap' }}>
            <span style={{ fontSize: '1.2rem', fontWeight: 700, color: 'var(--text-primary)' }}>{d.ticker}</span>
            <span style={{
              fontSize: '0.7rem', fontWeight: 700, padding: '0.15rem 0.45rem', borderRadius: '4px',
              backgroundColor: d.direction === 'LONG' ? '#22c55e22' : '#ef444422',
              color: d.direction === 'LONG' ? '#22c55e' : '#ef4444',
              border: `1px solid ${d.direction === 'LONG' ? '#22c55e44' : '#ef444444'}`,
            }}>
              {d.direction}
            </span>
            <span style={{ fontSize: '0.78rem', color: 'var(--text-secondary)' }}>{d.date}</span>
          </div>
          <div style={{ fontSize: '0.7rem', color: 'var(--text-secondary)', marginTop: '0.2rem' }}>
            {d.id} · OQS {d.oqs} · H60 class: {d.h60_class}
          </div>
        </div>
      </div>

      {livePos && <LivePaperBriefBanner pos={livePos} />}

      <ExecutionBlock
        execution={d.execution}
        direction={d.direction}
        action={d.entry_action}
        status="NEW"
        entryPrice={d.entry_price}
        entryHorizon={d.entry_horizon}
      />

      {paper && <PaperReplayOverlay d={d} paper={paper} />}

      {/* Summary strip */}
      <div style={{
        display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(110px, 1fr))',
        gap: '0.625rem', marginBottom: '1.25rem',
      }}>
        {([
          ['Entry State', <StateBadge state={d.entry_state} />],
          ['H120 State', <StateBadge state={d.h120_state} />],
          ['Action', <ActionBadge action={d.entry_action} />],
          ['reference_price', <span style={{ fontSize: '0.78rem', fontWeight: 600, color: 'var(--text-primary)', fontVariantNumeric: 'tabular-nums' }}>{fmtPrice(d.reference_price)}</span>],
          ['entry_price', <span style={{ fontSize: '0.78rem', fontWeight: 600, color: 'var(--text-primary)', fontVariantNumeric: 'tabular-nums' }}>{fmtPrice(d.entry_price)}</span>],
          [icOnLiveBook ? 'IC H300' : 'H300 Return', <ReturnCell value={d.h300_ret} />],
          ['MFE H60', <ReturnCell value={d.mfe_h60} />],
          ['MFE H120', <ReturnCell value={d.mfe_h120} />],
          [icOnLiveBook ? 'IC Outcome' : 'Outcome', <span style={{ fontSize: '0.78rem', color: outcomeColor, fontWeight: 600 }}>{d.outcome ?? '—'}</span>],
          [icOnLiveBook ? 'IC Daily Ret' : 'Daily Ret', <ReturnCell value={d.pnl} />],
        ] as [string, React.ReactNode][]).map(([label, val]) => (
          <div key={label} style={{
            backgroundColor: 'var(--bg-card)', border: '1px solid var(--border-color)',
            borderRadius: '8px', padding: '0.5rem 0.625rem',
          }}>
            <div style={{ fontSize: '0.6rem', color: 'var(--text-secondary)', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '0.25rem' }}>{label}</div>
            {val}
          </div>
        ))}
      </div>

      {/* Portfolio context panel — shown only when API data is available */}
      {portfolioCtx && (
        <div style={{
          backgroundColor: portfolioCtx.selected ? '#22c55e0d' : 'var(--bg-card)',
          border: `1px solid ${portfolioCtx.selected ? '#22c55e33' : 'var(--border-color)'}`,
          borderRadius: '8px', padding: '0.625rem 0.875rem', marginBottom: '1.25rem',
          display: 'flex', alignItems: 'center', gap: '1.5rem', flexWrap: 'wrap',
        }}>
          <div>
            <div style={{ fontSize: '0.6rem', color: 'var(--text-secondary)', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '0.2rem' }}>Portfolio</div>
            <span style={{
              fontSize: '0.78rem', fontWeight: 700,
              color: portfolioCtx.selected ? '#22c55e' : 'var(--text-secondary)',
            }}>
              {portfolioCtx.selected ? '✓ Selected' : '— Not selected'}
            </span>
          </div>
          <div>
            <div style={{ fontSize: '0.6rem', color: 'var(--text-secondary)', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '0.2rem' }}>OQS Rank</div>
            <span style={{ fontSize: '0.78rem', fontWeight: 600, color: 'var(--text-primary)' }}>
              #{portfolioCtx.oqs_rank} / {portfolioCtx.candidates_on_date}
            </span>
          </div>
          <div>
            <div style={{ fontSize: '0.6rem', color: 'var(--text-secondary)', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '0.2rem' }}>Position</div>
            <span style={{ fontSize: '0.78rem', fontWeight: 600, color: 'var(--text-primary)' }}>
              {portfolioCtx.selected ? `${portfolioCtx.position_size_pct.toFixed(0)}% capital` : '—'}
            </span>
          </div>
          <div style={{ marginLeft: 'auto', fontSize: '0.65rem', color: 'var(--text-secondary)' }}>
            Top {5} by OQS · {portfolioCtx.candidates_on_date} ACT candidates on {portfolioCtx.date}
          </div>
        </div>
      )}

      {/* Path returns */}
      <div style={{
        backgroundColor: 'var(--bg-card)', border: '1px solid var(--border-color)',
        borderRadius: '8px', padding: '0.625rem 0.875rem', marginBottom: '1.25rem',
      }}>
        <div style={{ fontSize: '0.6rem', color: 'var(--text-secondary)', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '0.4rem' }}>
          {icOnLiveBook ? 'IC v1 path returns · not live paper marks' : 'Intraday Path Returns'}
        </div>
        <div style={{ display: 'flex', gap: '1.25rem', flexWrap: 'wrap' }}>
          {([
            ['H15', d.h15_ret], ['H30', d.h30_ret], ['H60', d.h60_ret],
            ['H120', d.h120_ret], ['H180', d.h180_ret], ['H300', d.h300_ret],
          ] as [string, number | null][]).map(([lbl, val]) => (
            <div key={lbl} style={{ textAlign: 'center' }}>
              <div style={{ fontSize: '0.6rem', color: 'var(--text-secondary)', marginBottom: '0.15rem' }}>{lbl}</div>
              <ReturnCell value={val} />
            </div>
          ))}
        </div>
      </div>

      {/* Timeline */}
      <div style={{ fontSize: '0.65rem', color: 'var(--text-secondary)', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '0.875rem' }}>
        {icOnLiveBook ? 'IC v1 decision timeline · not live paper lifecycle' : 'Decision Timeline'}
      </div>

      <CheckpointRow
        label="T0" time="Entry"
        state={d.entry_state} action={d.entry_action} confidence={d.entry_confidence}
        horizon={d.entry_horizon} why={d.entry_why} risk={d.entry_risk}
        histWin={d.hist_win} histPf={d.hist_pf} histMed={d.hist_med}
      />
      <CheckpointRow
        label="H60" time="+60m"
        state={d.entry_state}
        action={icOnLiveBook ? 'IC' : 'MONITOR'}
        confidence={icOnLiveBook ? 'IC' : 'MODERATE'}
        horizon={icOnLiveBook ? 'IC H60 path · not a live instruction' : 'Continue monitoring; reassess at H120'}
        why={h60Why} risk={h60Risk}
        retValue={d.h60_ret}
        histWin={d.hist_win} histPf={d.hist_pf} histMed={d.hist_med}
      />
      <CheckpointRow
        label="H120" time="+120m"
        state={d.h120_state} action={d.h120_action} confidence={d.h120_confidence}
        horizon={d.h120_horizon} why={d.h120_why} risk={d.h120_risk}
        retValue={d.h120_ret}
        histWin={d.hist_win} histPf={d.hist_pf} histMed={d.hist_med}
      />
      <CheckpointRow
        label="H180" time="+180m"
        state={d.h120_state}
        action={icOnLiveBook ? 'IC' : 'CONTINUE'}
        confidence={icOnLiveBook ? 'IC' : 'MODERATE'}
        horizon={icOnLiveBook ? 'IC H180 path · not a live instruction' : 'Final hour; allow to H300'}
        why={h180Why}
        risk={icOnLiveBook
          ? 'IC H180 path observation · not a live paper action.'
          : 'Late reversal is a known failure mode; watch for adverse path after H180.'}
        retValue={d.h180_ret}
        histWin={d.hist_win} histPf={d.hist_pf} histMed={d.hist_med}
      />
      <CheckpointRow
        label="H300" time="+300m"
        state={d.h120_state} action={h300Action} confidence={h300Conf}
        horizon={icOnLiveBook ? 'IC H300 path · not a live instruction' : 'Session complete'}
        why={h300Why} risk={h300Risk}
        retValue={d.h300_ret}
        isLast
      />
    </div>
  );
}

// ── Decision list row ─────────────────────────────────────────────────────────

function DecisionRow({ d, onClick }: { d: DecisionBrief; onClick: () => void }) {
  return (
    <tr
      onClick={onClick}
      style={{ cursor: 'pointer', borderBottom: '1px solid var(--border-color)', transition: 'background 0.1s' }}
      onMouseEnter={e => (e.currentTarget.style.backgroundColor = 'rgba(255,255,255,0.04)')}
      onMouseLeave={e => (e.currentTarget.style.backgroundColor = 'transparent')}
    >
      <td style={{ padding: '0.5rem 0.625rem', fontSize: '0.8rem', fontWeight: 600, color: 'var(--text-primary)' }}>{d.ticker}</td>
      <td style={{ padding: '0.5rem 0.625rem', fontSize: '0.72rem', color: 'var(--text-secondary)' }}>{d.date}</td>
      <td style={{ padding: '0.5rem 0.625rem' }}>
        <span style={{
          fontSize: '0.68rem', fontWeight: 700, padding: '0.1rem 0.35rem', borderRadius: '3px',
          backgroundColor: d.direction === 'LONG' ? '#22c55e22' : '#ef444422',
          color: d.direction === 'LONG' ? '#22c55e' : '#ef4444',
        }}>{d.direction}</span>
      </td>
      <td style={{ padding: '0.5rem 0.625rem' }}><StateBadge state={d.entry_state} /></td>
      <td style={{ padding: '0.5rem 0.625rem' }}><StateBadge state={d.h120_state} /></td>
      <td style={{ padding: '0.5rem 0.625rem' }}><ActionBadge action={d.entry_action} /></td>
      <td style={{ padding: '0.5rem 0.625rem', fontSize: '0.78rem', color: 'var(--text-secondary)', textAlign: 'right' }}>{d.oqs}</td>
      <td style={{ padding: '0.5rem 0.625rem', textAlign: 'right' }}><ReturnCell value={d.h300_ret} /></td>
      <td style={{ padding: '0.5rem 0.625rem', fontSize: '0.72rem', color: d.outcome === 'WIN' ? '#22c55e' : d.outcome === 'LOSS' ? '#ef4444' : 'var(--text-secondary)' }}>{d.outcome ?? '—'}</td>
      <td style={{ padding: '0.5rem 0.625rem', textAlign: 'right' }}>
        <ReturnCell value={d.pnl} />
      </td>
    </tr>
  );
}

// ── Filter select ─────────────────────────────────────────────────────────────

function FilterSelect({
  label, value, onChange, options,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
  options: { value: string; label: string }[];
}) {
  return (
    <div style={{ flex: '0 1 130px' }}>
      <label style={{ fontSize: '0.6rem', color: 'var(--text-secondary)', textTransform: 'uppercase', letterSpacing: '0.05em', display: 'block', marginBottom: '0.25rem' }}>
        {label}
      </label>
      <select
        value={value}
        onChange={e => onChange(e.target.value)}
        style={{
          width: '100%', padding: '0.35rem 0.5rem', borderRadius: '6px',
          background: 'var(--bg-main)', color: 'var(--text-primary)',
          border: '1px solid var(--border-color)', outline: 'none', fontSize: '0.8rem',
          cursor: 'pointer',
        }}
      >
        {options.map(o => <option key={o.value} value={o.value}>{o.label}</option>)}
      </select>
    </div>
  );
}

// ── Filter state ──────────────────────────────────────────────────────────────

interface Filters {
  ticker?: string;
  direction?: string;
  entryState?: string;
  h120State?: string;
  action?: string;
}

// ── Main CockpitView ──────────────────────────────────────────────────────────

export function CockpitView() {
  const { decisions, loading, error } = useDecisions();
  const portfolioCtxMap = usePortfolioContext();
  const recommendationCards = useLifecycleFeed();
  const paperBlotter = usePaperBlotter();
  const { live: deferredLive, error: deferredLiveError, updatedAt: deferredLiveAt, lastIngestSeenAt } = useDeferredLive();
  const [showResearchArchive, setShowResearchArchive] = useState(false);
  const [showHistoricalReplay, setShowHistoricalReplay] = useState(false);
  const [filters, setFilters] = useState<Filters>({});
  const [selected, setSelected] = useState<DecisionBrief | null>(null);
  const [sortField, setSortField] = useState<'date' | 'ticker' | 'oqs' | 'h300_ret' | 'pnl'>('date');
  const [sortDir, setSortDir] = useState<'asc' | 'desc'>('desc');

  const paperByDecisionId = useMemo(() => {
    const m = new Map<string, PaperPosition>();
    for (const p of paperBlotter?.positions ?? []) {
      if (p.decision_id) m.set(p.decision_id, p);
    }
    return m;
  }, [paperBlotter]);

  const tickers = useMemo(() => [...new Set(decisions.map(d => d.ticker))].sort(), [decisions]);

  const filtered = useMemo(() => {
    let results = decisions;
    if (filters.ticker) {
      const t = filters.ticker.toUpperCase();
      results = results.filter(d => d.ticker.toUpperCase().includes(t));
    }
    if (filters.direction) results = results.filter(d => d.direction === filters.direction);
    if (filters.entryState) results = results.filter(d => d.entry_state === filters.entryState);
    if (filters.h120State) results = results.filter(d => d.h120_state === filters.h120State);
    if (filters.action) results = results.filter(d => d.entry_action === filters.action);

    return [...results].sort((a, b) => {
      let av: string | number, bv: string | number;
      switch (sortField) {
        case 'ticker':   av = a.ticker;       bv = b.ticker;       break;
        case 'oqs':      av = a.oqs;          bv = b.oqs;          break;
        case 'h300_ret': av = a.h300_ret ?? -999; bv = b.h300_ret ?? -999; break;
        case 'pnl':      av = a.pnl ?? -999999; bv = b.pnl ?? -999999; break;
        default:         av = a.date;         bv = b.date;
      }
      if (av < bv) return sortDir === 'asc' ? -1 : 1;
      if (av > bv) return sortDir === 'asc' ? 1 : -1;
      return 0;
    });
  }, [decisions, filters, sortField, sortDir]);

  const toggleSort = useCallback((field: typeof sortField) => {
    if (sortField === field) setSortDir(d => d === 'asc' ? 'desc' : 'asc');
    else { setSortField(field); setSortDir('desc'); }
  }, [sortField]);

  const SortTh = ({ field, label, align }: { field: typeof sortField; label: string; align?: string }) => (
    <th
      onClick={() => toggleSort(field)}
      style={{
        padding: '0.45rem 0.625rem', fontSize: '0.6rem', textTransform: 'uppercase',
        letterSpacing: '0.05em', color: 'var(--text-secondary)', cursor: 'pointer',
        userSelect: 'none', whiteSpace: 'nowrap', textAlign: (align as React.CSSProperties['textAlign']) ?? 'left',
        borderBottom: '1px solid var(--border-color)',
        backgroundColor: sortField === field ? 'rgba(255,255,255,0.04)' : 'transparent',
      }}
    >
      {label}{sortField === field ? (sortDir === 'asc' ? ' ↑' : ' ↓') : ''}
    </th>
  );

  if (loading && !deferredLive) {
    return (
      <div style={{ padding: '3rem', textAlign: 'center', color: 'var(--text-secondary)', fontSize: '0.875rem' }}>
        Loading today&apos;s paper session…
      </div>
    );
  }

  if (error && !deferredLive) {
    return (
      <div style={{ padding: '2rem' }}>
        <div style={{
          backgroundColor: '#ef444411', border: '1px solid #ef444433',
          borderRadius: '8px', padding: '1rem', color: '#ef4444', fontSize: '0.85rem',
        }}>
          <strong>API error</strong><br />{error}
        </div>
      </div>
    );
  }

  if (selected) {
    return (
      <div style={{ padding: '1.5rem 2rem', maxWidth: '860px', margin: '0 auto' }}>
        <ModeStrip />
        <DecisionDetail
            d={selected}
            onBack={() => setSelected(null)}
            portfolioCtx={portfolioCtxMap.get(selected.id)}
            paper={paperByDecisionId.get(selected.id) ?? null}
            livePos={livePositionForBrief(deferredLive, selected.id)}
          />
      </div>
    );
  }

  return (
    <div style={{ padding: '1.25rem 1.5rem 2rem' }}>
      <DecisionBoard
        live={deferredLive}
        decisions={decisions}
        ctxById={portfolioCtxMap}
        lastIngestSeenAt={lastIngestSeenAt}
        error={deferredLiveError}
      >
      <div style={{ maxWidth: '720px', margin: '0 auto' }}>
        <button
          type="button"
          onClick={() => setShowResearchArchive(v => !v)}
          style={{
            background: 'transparent',
            border: '1px solid var(--border-color)',
            color: 'var(--text-secondary)',
            borderRadius: '6px',
            padding: '0.4rem 0.7rem',
            cursor: 'pointer',
            fontSize: '0.68rem',
            fontWeight: 700,
            letterSpacing: '0.03em',
            textTransform: 'uppercase',
          }}
        >
          {showResearchArchive ? 'Hide' : 'Show'} research archive · IC table · live ledger
        </button>
      </div>

      {showResearchArchive && (
        <div style={{ marginTop: '1.25rem' }}>
      <ModeStrip />

      <DeferredLivePanel
        live={deferredLive}
        error={deferredLiveError}
        updatedAt={deferredLiveAt}
        lastIngestSeenAt={lastIngestSeenAt}
        decisions={decisions}
        ctxById={portfolioCtxMap}
        onViewDecision={(id) => {
          const d = decisions.find(dec => dec.id === id);
          if (d) setSelected(d);
        }}
      />

      {paperBlotter && paperBlotter.total > 0 && (
        <div style={{ marginBottom: '1.5rem' }}>
          <button
            onClick={() => setShowHistoricalReplay(v => !v)}
            style={{
              background: 'transparent',
              border: '1px solid #f59e0b66',
              color: '#b45309',
              borderRadius: '6px',
              padding: '0.4rem 0.7rem',
              cursor: 'pointer',
              fontSize: '0.68rem',
              fontWeight: 700,
              letterSpacing: '0.03em',
              textTransform: 'uppercase',
            }}
          >
            {showHistoricalReplay ? 'Hide' : 'Show'} Historical Replay archive · frozen paper_trader_v2 · {paperBlotter.total} fills
          </button>
          {showHistoricalReplay && (
            <div style={{ marginTop: '0.75rem' }}>
              <PaperBlotterPanel
                blotter={paperBlotter}
                decisions={decisions}
                onViewDecision={(id) => {
                  const d = decisions.find(dec => dec.id === id);
                  if (d) setSelected(d);
                }}
              />
            </div>
          )}
        </div>
      )}

      {/* Decision Feed */}
      {recommendationCards.length > 0 && (
        <DecisionFeed
          cards={recommendationCards}
          onViewDecision={(id) => {
            const d = decisions.find(dec => dec.id === id);
            if (d) setSelected(d);
          }}
        />
      )}

      {/* Search + filters */}
      <div style={{
        display: 'flex', gap: '0.625rem', flexWrap: 'wrap', marginBottom: '0.875rem',
        backgroundColor: 'var(--bg-card)', border: '1px solid var(--border-color)',
        borderRadius: '8px', padding: '0.75rem 0.875rem', alignItems: 'flex-end',
      }}>
        {/* Ticker */}
        <div style={{ flex: '1 1 160px' }}>
          <label style={{ fontSize: '0.6rem', color: 'var(--text-secondary)', textTransform: 'uppercase', letterSpacing: '0.05em', display: 'block', marginBottom: '0.25rem' }}>
            Ticker
          </label>
          <input
            type="text"
            placeholder={`e.g. ${tickers.slice(0, 3).join(', ')}`}
            value={filters.ticker ?? ''}
            onChange={e => setFilters(f => ({ ...f, ticker: e.target.value || undefined }))}
            list="cockpit-ticker-list"
            style={{
              width: '100%', padding: '0.35rem 0.5rem', borderRadius: '6px',
              background: 'var(--bg-main)', color: 'var(--text-primary)',
              border: '1px solid var(--border-color)', outline: 'none', fontSize: '0.8rem',
              boxSizing: 'border-box',
            }}
          />
          <datalist id="cockpit-ticker-list">
            {tickers.map(t => <option key={t} value={t} />)}
          </datalist>
        </div>

        <FilterSelect
          label="Direction"
          value={filters.direction ?? ''}
          onChange={v => setFilters(f => ({ ...f, direction: v || undefined }))}
          options={[
            { value: '', label: 'All' },
            { value: 'LONG', label: 'LONG' },
            { value: 'SHORT', label: 'SHORT' },
          ]}
        />

        <FilterSelect
          label="Entry State"
          value={filters.entryState ?? ''}
          onChange={v => setFilters(f => ({ ...f, entryState: v || undefined }))}
          options={[
            { value: '', label: 'All' },
            { value: 'ENTER', label: 'ENTER' },
            { value: 'WAIT-HIGH', label: 'WAIT-HIGH' },
            { value: 'WAIT-MID', label: 'WAIT-MID' },
            { value: 'WAIT-LOW', label: 'WAIT-LOW' },
            { value: 'AVOID', label: 'AVOID' },
          ]}
        />

        <FilterSelect
          label="H120 State"
          value={filters.h120State ?? ''}
          onChange={v => setFilters(f => ({ ...f, h120State: v || undefined }))}
          options={[
            { value: '', label: 'All' },
            { value: 'ENTER', label: 'ENTER' },
            { value: 'WAIT-HIGH', label: 'WAIT-HIGH' },
            { value: 'WAIT-MID', label: 'WAIT-MID' },
            { value: 'WAIT-LOW', label: 'WAIT-LOW' },
            { value: 'AVOID', label: 'AVOID' },
            { value: 'ENTER-LATE', label: 'ENTER-LATE' },
            { value: 'WAIT-LATE', label: 'WAIT-LATE' },
            { value: 'AVOID-LATE', label: 'AVOID-LATE' },
          ]}
        />

        <FilterSelect
          label="Action"
          value={filters.action ?? ''}
          onChange={v => setFilters(f => ({ ...f, action: v || undefined }))}
          options={[
            { value: '', label: 'All' },
            { value: 'ACT', label: 'ACT' },
            { value: 'MONITOR', label: 'MONITOR' },
            { value: 'AVOID', label: 'AVOID' },
            { value: 'UPGRADE', label: 'UPGRADE' },
            { value: 'CONTINUE', label: 'CONTINUE' },
          ]}
        />

        {/* Clear */}
        {Object.values(filters).some(Boolean) && (
          <div style={{ flex: '0 0 auto', alignSelf: 'flex-end' }}>
            <button
              onClick={() => setFilters({})}
              style={{
                padding: '0.35rem 0.7rem', borderRadius: '6px', cursor: 'pointer',
                background: 'transparent', border: '1px solid var(--border-color)',
                color: 'var(--text-secondary)', fontSize: '0.78rem',
              }}
            >
              Clear
            </button>
          </div>
        )}
      </div>

      {/* Result count */}
      <div style={{ fontSize: '0.72rem', color: 'var(--text-secondary)', marginBottom: '0.5rem' }}>
        {filtered.length} of {decisions.length} IC v1 DecisionBrief records · not the live paper book
        {filters.ticker ? ` · ticker: ${filters.ticker.toUpperCase()}` : ''}
      </div>

      {/* Table */}
      <div style={{ overflowX: 'auto', borderRadius: '8px', border: '1px solid var(--border-color)' }}>
        <table style={{ width: '100%', borderCollapse: 'collapse', backgroundColor: 'var(--bg-card)' }}>
          <thead>
            <tr style={{ backgroundColor: 'var(--bg-main)' }}>
              <SortTh field="ticker" label="Ticker" />
              <SortTh field="date" label="Date" />
              <th style={{ padding: '0.45rem 0.625rem', fontSize: '0.6rem', textTransform: 'uppercase', letterSpacing: '0.05em', color: 'var(--text-secondary)', borderBottom: '1px solid var(--border-color)' }}>Dir</th>
              <th style={{ padding: '0.45rem 0.625rem', fontSize: '0.6rem', textTransform: 'uppercase', letterSpacing: '0.05em', color: 'var(--text-secondary)', borderBottom: '1px solid var(--border-color)' }}>Entry State</th>
              <th style={{ padding: '0.45rem 0.625rem', fontSize: '0.6rem', textTransform: 'uppercase', letterSpacing: '0.05em', color: 'var(--text-secondary)', borderBottom: '1px solid var(--border-color)' }}>H120 State</th>
              <th style={{ padding: '0.45rem 0.625rem', fontSize: '0.6rem', textTransform: 'uppercase', letterSpacing: '0.05em', color: 'var(--text-secondary)', borderBottom: '1px solid var(--border-color)' }}>Action</th>
              <SortTh field="oqs" label="OQS" align="right" />
              <SortTh field="h300_ret" label="IC H300" align="right" />
              <th style={{ padding: '0.45rem 0.625rem', fontSize: '0.6rem', textTransform: 'uppercase', letterSpacing: '0.05em', color: 'var(--text-secondary)', borderBottom: '1px solid var(--border-color)' }}>IC Outcome</th>
              <SortTh field="pnl" label="IC Daily Ret" align="right" />
            </tr>
          </thead>
          <tbody>
            {filtered.length === 0 ? (
              <tr>
                <td colSpan={10} style={{ padding: '2rem', textAlign: 'center', color: 'var(--text-secondary)', fontSize: '0.85rem' }}>
                  No decisions match the current filters.
                </td>
              </tr>
            ) : (
              filtered.map(d => (
                <DecisionRow key={d.id} d={d} onClick={() => setSelected(d)} />
              ))
            )}
          </tbody>
        </table>
      </div>
        </div>
      )}
      </DecisionBoard>
    </div>
  );
}