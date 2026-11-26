import type { CSSProperties } from 'react';
import type { OperatorState, OfCandleMark, DecisionCardView, TradePlanOutcome } from './types';
import { displayOrUnavailable } from './freshness';
import {
  chip,
  labelStyle,
  op,
  panelStyle,
  pnlColor,
  tdStyle,
  thStyle,
  valueStyle,
} from './operatorStyles';

type Props = {
  state: OperatorState | null;
};

function candleLine(label: string, c: OfCandleMark | null | undefined) {
  if (!c) return `${label}: UNAVAILABLE`;
  const bar = c.bar_idx != null ? `bar ${c.bar_idx}` : 'bar —';
  const t = c.candle_label || c.candle_close_utc || c.mark_time_utc || '—';
  return `${label}: ${bar} · ${t}`;
}

function markHitsCard(
  c: DecisionCardView,
  m: OfCandleMark | null | undefined
): boolean {
  if (!m) return false;
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

function shortIst(s: string | null | undefined): string {
  if (!s) return '—';
  // "2026-11-21 09:55:22 IST" → "09:55:22"
  const m = s.match(/(\d{2}:\d{2}:\d{2})/);
  return m ? m[1] : s;
}

function reasonShort(r: string | null | undefined): string {
  if (!r) return '—';
  if (r.includes('GIVEBACK')) return 'GIVEBACK';
  if (r.includes('PROFIT_TARGET') || r === 'TAKE_PROFIT') return 'TP';
  if (r.includes('STOP_LOSS') || r === 'STOP_LOSS') return 'SL';
  if (r.includes('HORIZON') || r.includes('TIME')) return 'TIME';
  if (r.includes('SCRATCH')) return 'SCRATCH';
  return r.length > 22 ? `${r.slice(0, 20)}…` : r;
}

function patternShort(p: string | null | undefined): string {
  if (!p) return '—';
  if (p === 'ALLOW') return 'ALLOW';
  if (p.includes('STANDASIDE')) return 'STANDASIDE';
  if (p.includes('ABSTAIN')) return 'ABSTAIN';
  return p.length > 14 ? `${p.slice(0, 12)}…` : p;
}

function patternChipColor(p: string | null | undefined): { bg: string; fg: string } {
  if (p === 'ALLOW') return { bg: 'rgba(143,191,159,0.14)', fg: op.signal };
  if (p && p.includes('STANDASIDE')) return { bg: 'rgba(212,136,138,0.16)', fg: op.loss };
  return { bg: 'rgba(196,165,116,0.14)', fg: op.warn };
}

export default function CryptoWorkspace({ state }: Props) {
  const crypto = state?.crypto;
  const plan = crypto?.trade_plan;
  const book = crypto?.trade_plan_pnl;
  const ofRec = crypto?.order_flow;
  const ofExit = crypto?.order_flow_last_exit;
  const ofShadow = crypto?.of_shadow_trade;
  const marks = crypto?.of_candle_markers;
  const entryMark =
    marks?.entry ?? ofRec?.entry_candle ?? ofExit?.entry_candle ?? null;
  const exitMark = marks?.exit ?? ofExit?.exit_candle ?? null;
  const sum = book?.summary;
  const uPnL = plan?.valuation?.unrealized_net_pnl_usdt ?? null;

  return (
    <div style={{ fontFamily: op.sans, color: op.text }}>
      {!crypto?.available ? (
        <div style={{ ...panelStyle, color: op.loss, fontFamily: op.mono }}>UNAVAILABLE</div>
      ) : (
        <div style={{ display: 'grid', gap: 16 }}>
          {/* Market + book scoreboard */}
          <section style={{ ...panelStyle, padding: '18px 20px' }}>
            <div
              style={{
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'flex-start',
                gap: 16,
                flexWrap: 'wrap',
              }}
            >
              <div>
                <div style={{ ...labelStyle, marginBottom: 4 }}>BTCUSDT · CRYPTO_PLAN</div>
                <div
                  style={{
                    fontFamily: op.display,
                    fontSize: 36,
                    fontWeight: 600,
                    letterSpacing: '-0.03em',
                    lineHeight: 1,
                  }}
                >
                  {crypto.reference_price != null
                    ? crypto.reference_price.toLocaleString(undefined, {
                        minimumFractionDigits: 2,
                        maximumFractionDigits: 2,
                      })
                    : '—'}
                </div>
                <div style={{ marginTop: 10, display: 'flex', gap: 8, flexWrap: 'wrap' }}>
                  <span style={chip('rgba(143,191,159,0.12)', op.signal)}>
                    B {displayOrUnavailable(crypto.policy_b)}
                  </span>
                  <span style={chip('rgba(196,165,116,0.14)', op.warn)}>
                    C {displayOrUnavailable(crypto.policy_c)}
                  </span>
                  <span style={{ fontFamily: op.mono, fontSize: 12, color: op.dim }}>
                    imb{' '}
                    {crypto.imbalance != null ? crypto.imbalance.toFixed(3) : '—'} · age{' '}
                    {crypto.quote_age_ms != null
                      ? `${Math.round(crypto.quote_age_ms)}ms`
                      : '—'}
                  </span>
                </div>
              </div>

              <div style={scoreboardStyle}>
                <Score
                  label="W/L"
                  value={
                    sum
                      ? `${sum.n_wins ?? 0}/${sum.n_losses ?? 0}`
                      : '—'
                  }
                />
                <Score
                  label="Cum net"
                  value={
                    sum?.cumulative_net_pnl_usdt != null
                      ? `${sum.cumulative_net_pnl_usdt >= 0 ? '+' : ''}${sum.cumulative_net_pnl_usdt.toFixed(2)}`
                      : '—'
                  }
                  color={pnlColor(sum?.cumulative_net_pnl_usdt)}
                />
                <Score
                  label="Cum bps"
                  value={
                    sum?.cumulative_net_return_bps != null
                      ? `${sum.cumulative_net_return_bps >= 0 ? '+' : ''}${sum.cumulative_net_return_bps.toFixed(0)}`
                      : '—'
                  }
                  color={pnlColor(sum?.cumulative_net_return_bps)}
                />
                <Score label="n" value={sum?.n_trades != null ? String(sum.n_trades) : '—'} />
                <Score
                  label="GB"
                  value={sum?.giveback_exits != null ? String(sum.giveback_exits) : '—'}
                />
              </div>
            </div>
          </section>

          {/* Active plan */}
          <section style={panelStyle}>
            <div style={labelStyle}>Open plan</div>
            {!plan || !plan.is_active ? (
              <div style={{ display: 'grid', gap: 8 }}>
                <div style={{ color: op.dim, fontFamily: op.mono, fontSize: 13 }}>FLAT</div>
                {crypto.gate2e_flat_suppress?.prediction ? (
                  <div
                    style={{
                      display: 'flex',
                      gap: 8,
                      flexWrap: 'wrap',
                      alignItems: 'center',
                    }}
                  >
                    <span
                      style={chip(
                        patternChipColor(crypto.gate2e_flat_suppress.prediction).bg,
                        patternChipColor(crypto.gate2e_flat_suppress.prediction).fg
                      )}
                    >
                      2E suppress {patternShort(crypto.gate2e_flat_suppress.prediction)}
                      {crypto.gate2e_flat_suppress.gate_mode
                        ? ` · ${crypto.gate2e_flat_suppress.gate_mode}`
                        : ''}
                    </span>
                    <span style={{ fontFamily: op.mono, fontSize: 11, color: op.dim }}>
                      {crypto.gate2e_flat_suppress.direction ?? '—'}
                      {crypto.gate2e_pattern_suppressions != null
                        ? ` · n_suppress ${crypto.gate2e_pattern_suppressions}`
                        : ''}
                    </span>
                  </div>
                ) : null}
              </div>
            ) : (
              <div
                style={{
                  display: 'grid',
                  gridTemplateColumns: 'repeat(auto-fit, minmax(120px, 1fr))',
                  gap: 14,
                }}
              >
                <Metric
                  label="Side"
                  value={displayOrUnavailable(plan.direction)}
                />
                <Metric label="Entry" value={fmtPx(plan.entry_price)} />
                <Metric label="SL" value={fmtPx(plan.stop_loss_price)} />
                <Metric label="TP" value={fmtPx(plan.profit_target_price)} />
                <Metric
                  label="uPnL"
                  value={
                    uPnL != null ? `${uPnL >= 0 ? '+' : ''}${uPnL.toFixed(2)}` : '—'
                  }
                  color={pnlColor(uPnL)}
                />
                <Metric
                  label="Rem"
                  value={
                    plan.valuation?.time_remaining_seconds != null
                      ? `${Math.round(plan.valuation.time_remaining_seconds)}s`
                      : '—'
                  }
                />
                <div style={{ gridColumn: '1 / -1', fontFamily: op.mono, fontSize: 11, color: op.dim }}>
                  {displayOrUnavailable(plan.plan_id)}
                  {plan.plan_lifecycle_authority
                    ? ` · ${plan.plan_lifecycle_authority}`
                    : ''}
                </div>
                {plan.trade_pattern_match?.prediction ? (
                  <div
                    style={{
                      gridColumn: '1 / -1',
                      display: 'flex',
                      gap: 8,
                      flexWrap: 'wrap',
                      alignItems: 'center',
                    }}
                  >
                    <span
                      style={chip(
                        patternChipColor(plan.trade_pattern_match.prediction).bg,
                        patternChipColor(plan.trade_pattern_match.prediction).fg
                      )}
                    >
                      2E {patternShort(plan.trade_pattern_match.prediction)}
                      {plan.trade_pattern_match.gate_mode
                        ? ` · ${plan.trade_pattern_match.gate_mode}`
                        : plan.trade_pattern_match.gate_enabled
                          ? ''
                          : ' · obs'}
                    </span>
                    <span style={{ fontFamily: op.mono, fontSize: 11, color: op.dim }}>
                      n={plan.trade_pattern_match.cohort_n ?? '—'}
                      {plan.trade_pattern_match.hist_wr != null
                        ? ` · hist WR ${(plan.trade_pattern_match.hist_wr * 100).toFixed(0)}%`
                        : ''}
                      {plan.trade_pattern_match.blocks_entry ? ' · BLOCKS' : ''}
                    </span>
                  </div>
                ) : null}
              </div>
            )}
          </section>

          {/* Primary ledger */}
          <section style={panelStyle}>
            <div
              style={{
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'baseline',
                marginBottom: 10,
                gap: 12,
                flexWrap: 'wrap',
              }}
            >
              <div style={{ ...labelStyle, marginBottom: 0 }}>Recent trades</div>
              <div style={{ fontSize: 12, color: op.dim }}>
                W/L-first · never pooled with OF · PAPER 0%
              </div>
            </div>
            {!book?.summary ? (
              <div style={{ color: op.dim, fontFamily: op.mono, fontSize: 13 }}>UNAVAILABLE</div>
            ) : (
              <TradeTable rows={(book.recent ?? []).slice(0, 24)} />
            )}
          </section>

          {/* Secondary: OF + cards in two columns */}
          <div
            style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))',
              gap: 16,
            }}
          >
            <section style={panelStyle}>
              <div style={labelStyle}>OF paper (independent)</div>
              {ofShadow?.open ? (
                <div style={{ fontFamily: op.mono, fontSize: 12, lineHeight: 1.6 }}>
                  <div>
                    <span style={chip('rgba(143,191,159,0.15)', op.signal)}>OPEN</span>{' '}
                    {ofShadow.direction} @{fmtPx(ofShadow.entry_price)}
                  </div>
                  <div style={{ color: op.muted, marginTop: 8 }}>
                    mark {fmtPx(ofShadow.mark_price)} · uPnL{' '}
                    <span style={{ color: pnlColor(ofShadow.unrealized_net_pnl_usdt) }}>
                      {ofShadow.unrealized_net_pnl_usdt != null
                        ? ofShadow.unrealized_net_pnl_usdt.toFixed(2)
                        : '—'}
                    </span>
                  </div>
                </div>
              ) : (
                <div style={{ color: op.dim, fontFamily: op.mono, fontSize: 13 }}>No open OF</div>
              )}
              <div style={{ marginTop: 12, fontFamily: op.mono, fontSize: 11, color: op.dim }}>
                {ofRec?.active
                  ? `Latch ACTIVE ${ofRec.direction ?? ''}`
                  : ofExit
                    ? `Last exit ${ofExit.exit_reason ?? '—'}`
                    : 'No OF latch'}
              </div>
              <div style={{ marginTop: 8, fontFamily: op.mono, fontSize: 11, color: op.dim }}>
                <div>{candleLine('IN', entryMark)}</div>
                <div>{candleLine('OUT', exitMark)}</div>
              </div>
              {crypto.of_trade_pnl?.summary && (
                <div style={{ marginTop: 12, fontSize: 12, color: op.muted, fontFamily: op.mono }}>
                  OF ledger W/L {crypto.of_trade_pnl.summary.n_wins ?? 0}/
                  {crypto.of_trade_pnl.summary.n_losses ?? 0} · cum{' '}
                  {crypto.of_trade_pnl.summary.cumulative_net_pnl_usdt?.toFixed(2) ?? '—'} USDT
                </div>
              )}
            </section>

            <section style={panelStyle}>
              <div style={labelStyle}>Recent cards</div>
              <div style={{ overflowX: 'auto', maxHeight: 280 }}>
                <table style={{ width: '100%', borderCollapse: 'collapse', fontFamily: op.mono, fontSize: 11 }}>
                  <thead>
                    <tr>
                      <th style={thStyle}>bar</th>
                      <th style={thStyle}>B</th>
                      <th style={thStyle}>C</th>
                      <th style={thStyle}>mid</th>
                      <th style={thStyle}>mk</th>
                    </tr>
                  </thead>
                  <tbody>
                    {(crypto.recent_cards ?? []).slice(0, 16).map((c, i) => {
                      const isEntry = markHitsCard(c, entryMark);
                      const isExit = markHitsCard(c, exitMark);
                      return (
                        <tr
                          key={`${c.bar_idx}-${c.decision_event_idx}-${i}`}
                          style={{
                            background: isEntry
                              ? 'rgba(143,191,159,0.08)'
                              : isExit
                                ? 'rgba(212,136,138,0.08)'
                                : undefined,
                          }}
                        >
                          <td style={tdStyle}>{c.bar_idx ?? '—'}</td>
                          <td style={tdStyle}>{c.policy_b_action ?? '—'}</td>
                          <td style={tdStyle}>{c.policy_c_action ?? '—'}</td>
                          <td style={tdStyle}>
                            {c.reference_price != null ? c.reference_price.toFixed(1) : '—'}
                          </td>
                          <td style={tdStyle}>
                            {isEntry && isExit ? 'IO' : isEntry ? 'IN' : isExit ? 'OUT' : ''}
                          </td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              </div>
            </section>
          </div>
        </div>
      )}
    </div>
  );
}

function fmtPx(v: number | null | undefined): string {
  return v != null ? v.toFixed(2) : '—';
}

function Score({
  label,
  value,
  color,
}: {
  label: string;
  value: string;
  color?: string;
}) {
  return (
    <div style={{ textAlign: 'right', minWidth: 72 }}>
      <div style={{ fontSize: 10, letterSpacing: '0.1em', color: op.dim, textTransform: 'uppercase' }}>
        {label}
      </div>
      <div
        style={{
          fontFamily: op.mono,
          fontSize: 18,
          fontWeight: 600,
          color: color ?? op.text,
          marginTop: 2,
        }}
      >
        {value}
      </div>
    </div>
  );
}

function Metric({
  label,
  value,
  color,
}: {
  label: string;
  value: string;
  color?: string;
}) {
  return (
    <div>
      <div style={{ fontSize: 10, color: op.dim, letterSpacing: '0.08em', textTransform: 'uppercase' }}>
        {label}
      </div>
      <div style={{ ...valueStyle, fontSize: 16, color: color ?? op.text, marginTop: 2 }}>
        {value}
      </div>
    </div>
  );
}

function TradeTable({ rows }: { rows: TradePlanOutcome[] }) {
  if (!rows.length) {
    return (
      <div style={{ color: op.dim, fontFamily: op.mono, fontSize: 12 }}>
        No closed crypto trade-plan outcomes yet.
      </div>
    );
  }
  return (
    <div style={{ overflowX: 'auto' }}>
      <table style={{ width: '100%', borderCollapse: 'collapse', fontFamily: op.mono, fontSize: 12 }}>
        <thead>
          <tr>
            <th style={thStyle}>open</th>
            <th style={thStyle}>close</th>
            <th style={thStyle}>dir</th>
            <th style={{ ...thStyle, textAlign: 'right' }}>entry</th>
            <th style={{ ...thStyle, textAlign: 'right' }}>exit</th>
            <th style={{ ...thStyle, textAlign: 'right' }}>net $</th>
            <th style={{ ...thStyle, textAlign: 'right' }}>bps</th>
            <th style={thStyle}>exit</th>
            <th style={thStyle}>2E</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((t, i) => {
            const net = t.net_pnl_usdt;
            const pred = t.trade_pattern_match?.prediction;
            return (
              <tr key={`${t.plan_id ?? i}`}>
                <td style={tdStyle}>{shortIst(t.open_time_ist)}</td>
                <td style={tdStyle}>{shortIst(t.close_time_ist)}</td>
                <td style={tdStyle}>{t.direction ?? '—'}</td>
                <td style={{ ...tdStyle, textAlign: 'right' }}>{fmtPx(t.entry_price)}</td>
                <td style={{ ...tdStyle, textAlign: 'right' }}>{fmtPx(t.exit_price)}</td>
                <td style={{ ...tdStyle, textAlign: 'right', color: pnlColor(net), fontWeight: 600 }}>
                  {net != null ? `${net >= 0 ? '+' : ''}${net.toFixed(2)}` : '—'}
                </td>
                <td style={{ ...tdStyle, textAlign: 'right', color: pnlColor(t.net_return_bps) }}>
                  {t.net_return_bps != null
                    ? `${t.net_return_bps >= 0 ? '+' : ''}${t.net_return_bps.toFixed(1)}`
                    : '—'}
                </td>
                <td style={tdStyle}>{reasonShort(t.exit_reason)}</td>
                <td
                  style={{
                    ...tdStyle,
                    color: patternChipColor(pred).fg,
                  }}
                >
                  {patternShort(pred)}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

const scoreboardStyle: CSSProperties = {
  display: 'flex',
  gap: 20,
  flexWrap: 'wrap',
  alignItems: 'flex-end',
  padding: '4px 0',
};
