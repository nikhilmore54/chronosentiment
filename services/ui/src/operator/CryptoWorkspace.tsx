import type { OperatorState, OfCandleMark, DecisionCardView } from './types';
import { displayOrUnavailable } from './freshness';
import { chip, labelStyle, op, panelStyle, valueStyle } from './operatorStyles';

type Props = {
  state: OperatorState | null;
  onBack: () => void;
};

function candleLine(label: string, c: OfCandleMark | null | undefined) {
  if (!c) return `${label}: UNAVAILABLE`;
  const bar = c.bar_idx != null ? `bar ${c.bar_idx}` : 'bar —';
  const t = c.candle_label || c.candle_close_utc || c.mark_time_utc || '—';
  return `${label}: ${bar} · ${t}`;
}

function markHitsCard(
  c: DecisionCardView,
  m: OfCandleMark | null | undefined,
): boolean {
  if (!m) return false;
  // Event-precise only — never whole-bar paint (OF EXIT #— / all #665 rows)
  if (m.decision_event_idx == null && m.bar_idx == null) return false;
  if (
    m.decision_event_idx != null &&
    c.decision_event_idx != null &&
    Number(m.decision_event_idx) === Number(c.decision_event_idx)
  ) {
    return true;
  }
  if (
    m.decision_event_idx != null &&
    m.mark_time_ms != null &&
    c.close_time_ms != null &&
    Math.abs(Number(m.mark_time_ms) - Number(c.close_time_ms)) <= 5000
  ) {
    return true;
  }
  return false;
}

export default function CryptoWorkspace({ state, onBack }: Props) {
  const crypto = state?.crypto;
  const plan = crypto?.trade_plan;
  const ofRec = crypto?.order_flow;
  const ofExit = crypto?.order_flow_last_exit;
  const ofShadow = crypto?.of_shadow_trade;
  const marks = crypto?.of_candle_markers;
  const entryMark =
    marks?.entry ?? ofRec?.entry_candle ?? ofExit?.entry_candle ?? null;
  const exitMark = marks?.exit ?? ofExit?.exit_candle ?? null;

  return (
    <div style={{ fontFamily: op.sans, color: op.text }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 16, gap: 12, flexWrap: 'wrap' }}>
        <div>
          <button type="button" onClick={onBack} style={backBtn}>
            ← Overview
          </button>
          <h2 style={{ margin: '10px 0 4px', fontSize: 20 }}>Crypto workspace</h2>
          <p style={{ margin: 0, color: op.muted, fontSize: 13 }}>
            Policy B/C · 60m trade plan · OF latch/exit (independent) · candle marks — Pi fields only.
          </p>
        </div>
        <span style={chip(op.emerald, '#052e16')}>AUTHORITY 0%</span>
      </div>

      {!crypto?.available ? (
        <div style={{ ...panelStyle, color: op.rose, fontFamily: op.mono }}>UNAVAILABLE</div>
      ) : (
        <div style={{ display: 'grid', gap: 14 }}>
          <section style={panelStyle}>
            <div style={labelStyle}>Policy B / C</div>
            <div style={{ display: 'flex', gap: 10, flexWrap: 'wrap', marginBottom: 8 }}>
              <span style={chip('rgba(34,211,238,0.22)', op.cyan)}>
                B {displayOrUnavailable(crypto.policy_b)}
              </span>
              <span style={chip('rgba(167,139,250,0.25)', op.violet)}>
                C {displayOrUnavailable(crypto.policy_c)}
              </span>
            </div>
            <div style={{ ...valueStyle, fontSize: 13, color: op.muted }}>
              mid {crypto.reference_price != null ? crypto.reference_price.toFixed(2) : 'UNAVAILABLE'} ·
              imb{' '}
              {crypto.imbalance != null ? crypto.imbalance.toFixed(3) : 'UNAVAILABLE'} · quote_age{' '}
              {crypto.quote_age_ms != null ? `${Math.round(crypto.quote_age_ms)}ms` : 'UNAVAILABLE'} ·
              ATR {crypto.atr_14 != null ? crypto.atr_14.toFixed(4) : 'UNAVAILABLE'}
            </div>
          </section>

          <section style={panelStyle}>
            <div style={labelStyle}>Trade plan (60m lifecycle)</div>
            {!plan ? (
              <div style={{ color: op.dim, fontFamily: op.mono, fontSize: 13 }}>No active/display plan</div>
            ) : (
              <>
                <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap', marginBottom: 8 }}>
                  <span style={chip(plan.is_active ? 'rgba(52,211,153,0.3)' : 'rgba(148,163,184,0.25)', op.text)}>
                    {displayOrUnavailable(plan.state)}
                  </span>
                  <span style={chip('rgba(34,211,238,0.2)', op.cyan)}>
                    {displayOrUnavailable(plan.direction)}
                  </span>
                  <span style={{ fontFamily: op.mono, fontSize: 12, color: op.dim }}>
                    {displayOrUnavailable(plan.plan_id)}
                  </span>
                </div>
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit,minmax(140px,1fr))', gap: 8, fontFamily: op.mono, fontSize: 12 }}>
                  <div>entry {plan.entry_price ?? '—'}</div>
                  <div>SL {plan.stop_loss_price ?? '—'}</div>
                  <div>TP {plan.profit_target_price ?? '—'}</div>
                  <div>inv {plan.invalidation_price ?? '—'}</div>
                  <div>
                    uPNL{' '}
                    {plan.valuation?.unrealized_net_pnl_usdt != null
                      ? plan.valuation.unrealized_net_pnl_usdt.toFixed(4)
                      : '—'}
                  </div>
                  <div>
                    rem_s{' '}
                    {plan.valuation?.time_remaining_seconds != null
                      ? Math.round(plan.valuation.time_remaining_seconds)
                      : '—'}
                  </div>
                </div>
                <p style={{ margin: '10px 0 0', fontSize: 12, color: op.dim }}>
                  Plan TTL is independent of order-flow latch/exit.
                </p>
              </>
            )}
          </section>

          <section style={panelStyle}>
            <div style={labelStyle}>OF paper trade (open · PAPER · 0% authority)</div>
            {ofShadow?.open ? (
              <>
                <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap', marginBottom: 8 }}>
                  <span style={chip('rgba(45,212,191,0.3)', '#134e4a')}>OPEN PAPER</span>
                  <span style={chip('rgba(34,211,238,0.2)', op.cyan)}>
                    {displayOrUnavailable(ofShadow.direction)}
                  </span>
                  <span style={chip('rgba(148,163,184,0.2)', op.text)}>
                    {displayOrUnavailable(ofShadow.of_trade_id)}
                  </span>
                </div>
                <div
                  style={{
                    display: 'grid',
                    gridTemplateColumns: 'repeat(auto-fit,minmax(140px,1fr))',
                    gap: 8,
                    fontFamily: op.mono,
                    fontSize: 12,
                  }}
                >
                  <div>entry {ofShadow.entry_price ?? '—'}</div>
                  <div>mark {ofShadow.mark_price ?? '—'}</div>
                  <div>qty {ofShadow.simulated_quantity ?? '—'}</div>
                  <div>notional {ofShadow.simulated_notional_usdt ?? '—'}</div>
                  <div>
                    part{' '}
                    {ofShadow.participation_pct != null
                      ? `${(ofShadow.participation_pct * 100).toFixed(0)}%`
                      : '—'}
                  </div>
                  <div>sigQty {ofShadow.entry_signal_qty ?? '—'}</div>
                  <div>
                    uPNL{' '}
                    {ofShadow.unrealized_net_pnl_usdt != null
                      ? `${ofShadow.unrealized_net_pnl_usdt.toFixed(4)} USDT`
                      : '—'}
                  </div>
                  <div>
                    uBPS{' '}
                    {ofShadow.unrealized_net_return_bps != null
                      ? ofShadow.unrealized_net_return_bps.toFixed(2)
                      : '—'}
                  </div>
                  <div>
                    hold{' '}
                    {ofShadow.holding_duration_seconds != null
                      ? `${ofShadow.holding_duration_seconds.toFixed(1)}s`
                      : '—'}
                  </div>
                  <div>opened {displayOrUnavailable(ofShadow.paper_open_timestamp)}</div>
                  <div>spread {ofShadow.spread_cost_usdt ?? '—'}</div>
                  <div>fees {ofShadow.fees_usdt ?? '—'}</div>
                </div>
                <p style={{ margin: '10px 0 0', fontSize: 12, color: op.dim }}>
                  Exit = sustained reverse OF after min hold · not 60m plan TTL
                </p>
              </>
            ) : (
              <div style={{ color: op.dim, fontFamily: op.mono, fontSize: 13 }}>
                No open OF paper trade
              </div>
            )}
          </section>

          <section style={panelStyle}>
            <div style={labelStyle}>OF P&L ledger (PAPER · independent of 60m plan)</div>
            {crypto.of_trade_pnl?.summary ? (
              <>
                <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap', marginBottom: 8 }}>
                  <span style={chip('rgba(34,211,238,0.2)', op.cyan)}>
                    n {crypto.of_trade_pnl.summary.n_trades ?? 0}
                  </span>
                  <span style={chip('rgba(52,211,153,0.25)', op.emerald)}>
                    W {crypto.of_trade_pnl.summary.n_wins ?? 0}
                  </span>
                  <span style={chip('rgba(251,113,133,0.3)', op.rose)}>
                    L {crypto.of_trade_pnl.summary.n_losses ?? 0}
                  </span>
                  <span style={chip('rgba(148,163,184,0.2)', op.text)}>
                    cum net{' '}
                    {crypto.of_trade_pnl.summary.cumulative_net_pnl_usdt != null
                      ? `${crypto.of_trade_pnl.summary.cumulative_net_pnl_usdt.toFixed(2)} USDT`
                      : 'UNAVAILABLE'}
                  </span>
                  <span style={chip('rgba(45,212,191,0.25)', op.emerald)}>
                    cum{' '}
                    {crypto.of_trade_pnl.summary.cumulative_net_return_bps != null
                      ? `${crypto.of_trade_pnl.summary.cumulative_net_return_bps.toFixed(2)} bps`
                      : '—'}
                  </span>
                  <span style={chip('rgba(245,158,11,0.2)', op.amber)}>PAPER · 0%</span>
                </div>
                <div style={{ overflowX: 'auto' }}>
                  <table style={{ width: '100%', borderCollapse: 'collapse', fontFamily: op.mono, fontSize: 11 }}>
                    <thead>
                      <tr style={{ color: op.dim, textAlign: 'left' }}>
                        <th style={th}>open_utc</th>
                        <th style={th}>close_utc</th>
                        <th style={th}>hold</th>
                        <th style={th}>dir</th>
                        <th style={th}>entry</th>
                        <th style={th}>exit</th>
                        <th style={th}>net USDT</th>
                        <th style={th}>net bps</th>
                        <th style={th}>fees</th>
                        <th style={th}>reason</th>
                      </tr>
                    </thead>
                    <tbody>
                      {(crypto.of_trade_pnl.recent ?? []).slice(0, 24).map((t, i) => (
                        <tr key={`${t.of_trade_id ?? i}`}>
                          <td style={td}>{t.paper_open_timestamp ?? t.activated_ms ?? '—'}</td>
                          <td style={td}>
                            {t.paper_close_timestamp ?? t.signal_exit_timestamp ?? t.exited_ms ?? '—'}
                          </td>
                          <td style={td}>
                            {t.holding_duration_seconds != null
                              ? `${t.holding_duration_seconds.toFixed(1)}s`
                              : '—'}
                          </td>
                          <td style={td}>{t.direction ?? '—'}</td>
                          <td style={td}>{t.entry_price ?? '—'}</td>
                          <td style={td}>{t.exit_price ?? '—'}</td>
                          <td style={td}>
                            {t.net_pnl_usdt != null ? t.net_pnl_usdt.toFixed(4) : '—'}
                          </td>
                          <td style={td}>
                            {t.net_return_bps != null ? t.net_return_bps.toFixed(2) : '—'}
                          </td>
                          <td style={td}>
                            {t.fees_usdt != null ? t.fees_usdt.toFixed(4) : '—'}
                          </td>
                          <td style={td}>{t.exit_reason ?? '—'}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                  {(crypto.of_trade_pnl.recent?.length ?? 0) === 0 && (
                    <div style={{ color: op.dim, marginTop: 8, fontFamily: op.mono, fontSize: 12 }}>
                      No closed OF paper trades yet (ledger updates on reverse-OF exit).
                    </div>
                  )}
                </div>
              </>
            ) : (
              <div style={{ color: op.dim, fontFamily: op.mono, fontSize: 13 }}>UNAVAILABLE</div>
            )}
          </section>

          <section style={panelStyle}>
            <div style={labelStyle}>Order-flow latch / exit (not 60m)</div>
            {ofRec?.active ? (
              <>
                <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap', marginBottom: 8 }}>
                  <span style={chip('rgba(52,211,153,0.35)', '#052e16')}>OF ACTIVE</span>
                  <span style={chip('rgba(34,211,238,0.2)', op.cyan)}>
                    {displayOrUnavailable(ofRec.direction)}
                  </span>
                  <span style={chip('rgba(251,191,36,0.25)', op.amber)}>
                    {displayOrUnavailable(ofRec.order_flow_label)}
                  </span>
                </div>
                <div style={{ fontFamily: op.mono, fontSize: 12, color: op.muted }}>
                  imb {ofRec.order_flow_imbalance != null ? ofRec.order_flow_imbalance.toFixed(3) : '—'} ·
                  entry {ofRec.entry_price ?? '—'} · {displayOrUnavailable(ofRec.plan_id)}
                </div>
              </>
            ) : ofExit ? (
              <>
                <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap', marginBottom: 8 }}>
                  <span style={chip('rgba(251,113,133,0.35)', '#450a0a')}>OF EXITED</span>
                  <span style={chip('rgba(148,163,184,0.2)', op.text)}>
                    {displayOrUnavailable(ofExit.direction)}
                  </span>
                  <span style={{ fontFamily: op.mono, fontSize: 12, color: op.dim }}>
                    {displayOrUnavailable(ofExit.exit_reason)}
                  </span>
                </div>
                <div style={{ fontFamily: op.mono, fontSize: 12, color: op.muted }}>
                  exit imb {ofExit.exit_imbalance != null ? ofExit.exit_imbalance.toFixed(3) : '—'} ·
                  opposite {displayOrUnavailable(ofExit.exit_opposite_side)}
                </div>
              </>
            ) : (
              <div style={{ color: op.dim, fontFamily: op.mono, fontSize: 13 }}>No OF latch</div>
            )}
            <div style={{ marginTop: 10, fontFamily: op.mono, fontSize: 12, color: op.muted }}>
              <div>{candleLine('ENTRY candle', marks?.entry ?? ofRec?.entry_candle ?? ofExit?.entry_candle)}</div>
              <div>{candleLine('EXIT candle', marks?.exit ?? ofExit?.exit_candle)}</div>
            </div>
          </section>

          <section style={panelStyle}>
            <div style={labelStyle}>Recent cards (ENTRY / EXIT marks)</div>
            <div style={{ overflowX: 'auto' }}>
              <table style={{ width: '100%', borderCollapse: 'collapse', fontFamily: op.mono, fontSize: 11 }}>
                <thead>
                  <tr style={{ color: op.dim, textAlign: 'left' }}>
                    <th style={th}>bar</th>
                    <th style={th}>time</th>
                    <th style={th}>B</th>
                    <th style={th}>C</th>
                    <th style={th}>mid</th>
                    <th style={th}>OF</th>
                    <th style={th}>mark</th>
                  </tr>
                </thead>
                <tbody>
                  {(crypto.recent_cards ?? []).slice(0, 24).map((c, i) => {
                    const isEntry = markHitsCard(c, entryMark);
                    const isExit = markHitsCard(c, exitMark);
                    const bg = isEntry
                      ? 'rgba(52,211,153,0.12)'
                      : isExit
                        ? 'rgba(251,113,133,0.12)'
                        : 'transparent';
                    return (
                      <tr key={`${c.bar_idx}-${c.decision_event_idx}-${i}`} style={{ background: bg }}>
                        <td style={td}>{c.bar_idx ?? '—'}</td>
                        <td style={td}>{c.timestamp_utc ?? '—'}</td>
                        <td style={td}>{c.policy_b_action ?? '—'}</td>
                        <td style={td}>{c.policy_c_action ?? '—'}</td>
                        <td style={td}>
                          {c.reference_price != null ? c.reference_price.toFixed(2) : '—'}
                        </td>
                        <td style={td}>
                          {c.order_flow_supported
                            ? c.order_flow_label || 'OF'
                            : '—'}
                        </td>
                        <td style={td}>
                          {isEntry && isExit ? 'IN/OUT' : isEntry ? 'ENTRY' : isExit ? 'EXIT' : ''}
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
              {(crypto.recent_cards?.length ?? 0) === 0 && (
                <div style={{ color: op.dim, marginTop: 8 }}>No recent cards</div>
              )}
            </div>
          </section>
        </div>
      )}
    </div>
  );
}

const backBtn = {
  background: 'transparent',
  border: `1px solid ${op.border}`,
  color: op.muted,
  borderRadius: 6,
  padding: '6px 10px',
  cursor: 'pointer',
  fontSize: 12,
};

const th = { padding: '6px 8px', fontWeight: 600, borderBottom: `1px solid ${op.border}` };
const td = { padding: '6px 8px', borderBottom: `1px solid rgba(148,163,184,0.08)`, color: op.text };
