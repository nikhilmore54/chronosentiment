import { useMemo, useState } from 'react';
import type { OperatorState } from './types';
import { displayOrUnavailable } from './freshness';
import { chip, labelStyle, op, panelStyle } from './operatorStyles';

type Props = {
  state: OperatorState | null;
  onBack: () => void;
};

const PAGE = 25;

function formatIst(iso: string | null | undefined): string {
  if (!iso) return '—';
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  try {
    return (
      d.toLocaleString('en-IN', {
        timeZone: 'Asia/Kolkata',
        year: 'numeric',
        month: 'short',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
        hour12: false,
      }) + ' IST'
    );
  } catch {
    return iso;
  }
}

function formatIstClock(iso: string | null | undefined): string {
  if (!iso) return '—';
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return '—';
  try {
    return (
      d.toLocaleTimeString('en-IN', {
        timeZone: 'Asia/Kolkata',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
        hour12: false,
      }) + ' IST'
    );
  } catch {
    return '—';
  }
}

export default function NseWorkspace({ state, onBack }: Props) {
  const nse = state?.nse;
  const [showExplorer, setShowExplorer] = useState(false);
  const [filter, setFilter] = useState('');
  const [page, setPage] = useState(0);
  const recommendedAt = nse?.recommended_at_utc ?? nse?.emitted_utc ?? null;

  const filtered = useMemo(() => {
    const rows = nse?.scrips ?? [];
    const q = filter.trim().toUpperCase();
    if (!q) return rows;
    return rows.filter((r) => {
      const sym = String(r.tradingsymbol ?? r.symbol ?? '').toUpperCase();
      return sym.includes(q);
    });
  }, [nse?.scrips, filter]);

  const pageRows = filtered.slice(page * PAGE, page * PAGE + PAGE);
  const pages = Math.max(1, Math.ceil(filtered.length / PAGE));

  const rejections = nse?.rejection_breakdown
    ? Object.entries(nse.rejection_breakdown).sort((a, b) => b[1] - a[1]).slice(0, 8)
    : [];

  return (
    <div style={{ fontFamily: op.sans, color: op.text }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 16, gap: 12, flexWrap: 'wrap' }}>
        <div>
          <button type="button" onClick={onBack} style={backBtn}>
            ← Overview
          </button>
          <h2 style={{ margin: '10px 0 4px', fontSize: 20 }}>NSE workspace</h2>
          <p style={{ margin: 0, color: op.muted, fontSize: 13 }}>
            Census / classified / quarantine · next-session + OF · session board. Explorer secondary.
          </p>
        </div>
        <span style={chip(op.emerald, '#052e16')}>AUTHORITY 0%</span>
      </div>

      {!nse?.available ? (
        <div style={{ ...panelStyle, color: op.rose, fontFamily: op.mono }}>UNAVAILABLE</div>
      ) : (
        <div style={{ display: 'grid', gap: 14 }}>
          <section style={panelStyle}>
            <div style={labelStyle}>Universe strip</div>
            <div style={{ display: 'flex', gap: 10, flexWrap: 'wrap' }}>
              <span style={chip('rgba(34,211,238,0.2)', op.cyan)}>
                CENSUS {nse.census_total ?? 'UNAVAILABLE'}
              </span>
              <span style={chip('rgba(52,211,153,0.25)', op.emerald)}>
                CLASSIFIED {nse.classified_count ?? 'UNAVAILABLE'}
              </span>
              <span style={chip('rgba(148,163,184,0.25)', op.text)}>
                QUARANTINED {nse.quarantined_count ?? 'UNAVAILABLE'}
              </span>
              {nse.universe_label && (
                <span style={{ fontFamily: op.mono, fontSize: 12, color: op.dim }}>
                  {nse.universe_label}
                </span>
              )}
            </div>
          </section>

          <section style={panelStyle}>
            <div style={{ display: 'flex', justifyContent: 'space-between', gap: 10, flexWrap: 'wrap', marginBottom: 8 }}>
              <div style={labelStyle}>Next-session watchlist (ORDER FLOW badges)</div>
              <span style={chip('rgba(103,232,249,0.35)', '#083344')}>
                LIVE · poll {nse.live_poll_hint_sec ?? 5}s
              </span>
            </div>
            <div style={{ fontFamily: op.mono, fontSize: 12, color: op.muted, marginBottom: 6 }}>
              from {displayOrUnavailable(nse.from_session_date)} → next{' '}
              {displayOrUnavailable(nse.next_session_date)} · n=
              {nse.n_recommendations ?? '—'} · OF=
              {nse.n_order_flow_supported ?? '—'}
              {nse.next_session_note ? ` · ${nse.next_session_note}` : ''}
            </div>
            <div style={{ fontFamily: op.mono, fontSize: 12, color: op.cyan, marginBottom: 4 }}>
              Recommended at: {formatIst(recommendedAt)}
            </div>
            <div style={{ fontFamily: op.mono, fontSize: 11, color: op.dim, marginBottom: 10 }}>
              UI refresh: {formatIst(new Date().toISOString())}
              {nse.stamp_mtime_utc ? ` · stamp mtime ${formatIst(nse.stamp_mtime_utc)}` : ''}
            </div>
            {(nse.recommendations?.length ?? 0) === 0 ? (
              <div style={{ color: op.amber, fontSize: 13 }}>
                No next-session recommendations (weekend / empty stamp / unavailable).
              </div>
            ) : (
              <div style={{ overflowX: 'auto' }}>
                <table style={{ width: '100%', borderCollapse: 'collapse', fontFamily: op.mono, fontSize: 11 }}>
                  <thead>
                    <tr style={{ color: op.dim, textAlign: 'left' }}>
                      <th style={th}>#</th>
                      <th style={th}>symbol</th>
                      <th style={th}>status</th>
                      <th style={th}>entry range</th>
                      <th style={th}>target</th>
                      <th style={th}>stop</th>
                      <th style={th}>R:R</th>
                      <th style={th}>OF</th>
                      <th style={th}>rec time</th>
                    </tr>
                  </thead>
                  <tbody>
                    {(nse.recommendations ?? []).map((r, i) => {
                      const lv = r.levels;
                      const eLo = r.entry_low ?? lv?.entry_low ?? null;
                      const eHi = r.entry_high ?? lv?.entry_high ?? null;
                      const stop = r.stop_price ?? lv?.stop_price ?? null;
                      const tgt = r.target_price ?? lv?.target_price ?? null;
                      const rr = r.risk_reward_display ?? lv?.risk_reward_display ?? '—';
                      const lst = r.level_status ?? lv?.level_status ?? '—';
                      return (
                      <tr key={`${r.symbol}-${i}`}>
                        <td style={td}>{i + 1}</td>
                        <td style={td}>{r.tradingsymbol || r.symbol || '—'}</td>
                        <td style={td}>
                          <span style={chip('rgba(103,232,249,0.25)', op.cyan)}>LIVE</span>{' '}
                          {lst}
                        </td>
                        <td style={td}>
                          {eLo != null && eHi != null
                            ? `₹${Number(eLo).toFixed(2)}–${Number(eHi).toFixed(2)}`
                            : '—'}
                        </td>
                        <td style={td}>{tgt != null ? `₹${Number(tgt).toFixed(2)}` : '—'}</td>
                        <td style={td}>{stop != null ? `₹${Number(stop).toFixed(2)}` : '—'}</td>
                        <td style={td}>{rr}</td>
                        <td style={td}>
                          {r.order_flow_supported ? (
                            <span style={chip('rgba(251,191,36,0.3)', op.amber)}>
                              {r.order_flow_label || 'ORDER FLOW'}
                            </span>
                          ) : (
                            '—'
                          )}
                        </td>
                        <td style={td} title={formatIst(r.recommended_at_utc ?? recommendedAt)}>
                          {formatIstClock(r.recommended_at_utc ?? recommendedAt)}
                        </td>
                      </tr>
                      );
                    })}
                  </tbody>
                </table>
                <div style={{ marginTop: 8, fontSize: 11, color: op.dim, fontFamily: op.mono }}>
                  Indicative levels only (RP608_DYNAMIC_OHLCV_STRUCTURE_V1) — not executed / not
                  paper-trade entry · authority 0%
                </div>
              </div>
            )}
          </section>

          <section style={panelStyle}>
            <div style={labelStyle}>Session board / eval summary</div>
            <div style={{ display: 'flex', gap: 10, flexWrap: 'wrap', marginBottom: 10 }}>
              <span style={chip('rgba(34,211,238,0.2)', op.cyan)}>
                EVAL {nse.total_evaluated ?? 'UNAVAILABLE'}
              </span>
              <span style={chip('rgba(52,211,153,0.25)', op.emerald)}>
                QUAL {nse.n_qualifying ?? 'UNAVAILABLE'}
              </span>
              <span style={chip('rgba(167,139,250,0.25)', op.violet)}>
                HIGH-T {nse.n_high_target ?? 'UNAVAILABLE'}
              </span>
              <span style={chip('rgba(251,191,36,0.3)', op.amber)}>
                LIVE OF {nse.n_live_order_flow_supported ?? '—'}
                {nse.n_live_order_flow_long != null || nse.n_live_order_flow_short != null
                  ? ` (L${nse.n_live_order_flow_long ?? '—'} / S${nse.n_live_order_flow_short ?? '—'})`
                  : ''}
              </span>
            </div>
            {nse.momentum_distribution && (
              <div style={{ fontFamily: op.mono, fontSize: 12, color: op.muted, marginBottom: 8 }}>
                mom ↑{nse.momentum_distribution.bullish_ge_005 ?? '—'} · ≈
                {nse.momentum_distribution.neutral ?? '—'} · ↓
                {nse.momentum_distribution.bearish_le_005 ?? '—'}
              </div>
            )}
            {rejections.length > 0 && (
              <div style={{ fontFamily: op.mono, fontSize: 11, color: op.dim }}>
                Top rejections:{' '}
                {rejections.map(([k, v]) => `${k}=${v}`).join(' · ')}
              </div>
            )}
            {(nse.total_evaluated == null || nse.total_evaluated === 0) && (
              <div style={{ color: op.amber, fontSize: 13, marginTop: 8 }}>
                Board empty — likely weekend / outside session eval window.
              </div>
            )}
          </section>

          <section style={panelStyle}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', gap: 8, flexWrap: 'wrap' }}>
              <div style={labelStyle}>Symbol explorer (secondary)</div>
              <button
                type="button"
                style={backBtn}
                onClick={() => {
                  setShowExplorer((v) => !v);
                  setPage(0);
                }}
              >
                {showExplorer ? 'Hide explorer' : `Show explorer (${nse.scrips?.length ?? 0})`}
              </button>
            </div>
            {showExplorer && (
              <>
                <input
                  value={filter}
                  onChange={(e) => {
                    setFilter(e.target.value);
                    setPage(0);
                  }}
                  placeholder="Filter symbol…"
                  style={{
                    marginTop: 8,
                    width: '100%',
                    maxWidth: 280,
                    padding: '8px 10px',
                    borderRadius: 6,
                    border: `1px solid ${op.border}`,
                    background: '#0f172a',
                    color: op.text,
                    fontFamily: op.mono,
                    fontSize: 12,
                  }}
                />
                <div style={{ overflowX: 'auto', marginTop: 10 }}>
                  <table style={{ width: '100%', borderCollapse: 'collapse', fontFamily: op.mono, fontSize: 11 }}>
                    <thead>
                      <tr style={{ color: op.dim, textAlign: 'left' }}>
                        <th style={th}>symbol</th>
                        <th style={th}>status</th>
                        <th style={th}>dec</th>
                        <th style={th}>OF</th>
                        <th style={th}>imb</th>
                        <th style={th}>ltp</th>
                        <th style={th}>hist</th>
                      </tr>
                    </thead>
                    <tbody>
                      {pageRows.map((r, i) => (
                        <tr key={`${String(r.tradingsymbol ?? r.symbol)}-${i}`}>
                          <td style={td}>{String(r.tradingsymbol ?? r.symbol ?? '—')}</td>
                          <td style={td}>
                            {String(
                              r.quarantine_reason
                                ? 'QUARANTINED'
                                : r.classified === false
                                  ? '—'
                                  : r.status ?? r.comp_status ?? '—'
                            )}
                          </td>
                          <td style={td}>{String(r.decision ?? r.decision_code ?? '—')}</td>
                          <td style={td}>
                            {r.order_flow_supported ? (
                              <span style={chip('rgba(251,191,36,0.3)', op.amber)}>
                                {String(r.order_flow_label || 'ORDER FLOW')}
                              </span>
                            ) : (
                              '—'
                            )}
                          </td>
                          <td style={td}>
                            {typeof r.order_flow_imbalance === 'number'
                              ? r.order_flow_imbalance.toFixed(3)
                              : '—'}
                          </td>
                          <td style={td}>{String(r.ltp ?? r.last_price ?? '—')}</td>
                          <td style={td}>{String(r.history_length ?? '—')}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
                <div style={{ display: 'flex', gap: 8, marginTop: 10, alignItems: 'center' }}>
                  <button
                    type="button"
                    style={backBtn}
                    disabled={page <= 0}
                    onClick={() => setPage((p) => Math.max(0, p - 1))}
                  >
                    Prev
                  </button>
                  <span style={{ fontFamily: op.mono, fontSize: 11, color: op.dim }}>
                    {page + 1} / {pages} · {filtered.length} rows
                  </span>
                  <button
                    type="button"
                    style={backBtn}
                    disabled={page >= pages - 1}
                    onClick={() => setPage((p) => Math.min(pages - 1, p + 1))}
                  >
                    Next
                  </button>
                </div>
              </>
            )}
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
