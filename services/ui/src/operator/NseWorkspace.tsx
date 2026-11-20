import type { OperatorState } from './types';
import { displayOrUnavailable } from './freshness';
import { chip, labelStyle, op, panelStyle } from './operatorStyles';

type Props = {
  state: OperatorState | null;
};

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

export default function NseWorkspace({ state }: Props) {
  const nse = state?.nse;
  const recommendedAt = nse?.recommended_at_utc ?? nse?.emitted_utc ?? null;

  const rejections = nse?.rejection_breakdown
    ? Object.entries(nse.rejection_breakdown).sort((a, b) => b[1] - a[1]).slice(0, 8)
    : [];

  return (
    <div style={{ fontFamily: op.sans, color: op.text }}>
      <div style={{ marginBottom: 16 }}>
        <h2 style={{ margin: '0 0 4px', fontSize: 20, fontWeight: 700 }}>NSE</h2>
        <p style={{ margin: 0, color: op.muted, fontSize: 13 }}>
          Universe · Coralys input gate · next-session · session board — Pi fields only.
        </p>
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
              <span style={chip('rgba(251,191,36,0.25)', op.amber)}>
                PHASE {nse.session_phase ?? nse.coralys_input_gate?.session_phase ?? '—'}
              </span>
              {nse.universe_label && (
                <span style={{ fontFamily: op.mono, fontSize: 12, color: op.dim }}>
                  {nse.universe_label}
                </span>
              )}
            </div>
          </section>

          <section style={{ ...panelStyle, borderColor: 'rgba(251,191,36,0.35)' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', gap: 10, flexWrap: 'wrap', marginBottom: 8 }}>
              <div style={labelStyle}>Coralys input gate · prior + pre-open + phase</div>
              <span
                style={chip(
                  nse.coralys_input_gate?.gate_enabled === false
                    ? 'rgba(148,163,184,0.25)'
                    : 'rgba(251,191,36,0.3)',
                  op.amber
                )}
              >
                {nse.coralys_input_gate
                  ? nse.coralys_input_gate.gate_enabled === false
                    ? 'GATE OFF'
                    : 'GATE ON'
                  : 'UNAVAILABLE'}
              </span>
            </div>
            {!nse.coralys_input_gate ? (
              <div style={{ color: op.amber, fontSize: 13 }}>
                No coralys_input_gate on /api/state — Pi dashboard may need refresh.
              </div>
            ) : (
              <>
                <div style={{ fontFamily: op.mono, fontSize: 12, color: op.muted, marginBottom: 8 }}>
                  {nse.coralys_input_gate.protocol_id ?? 'RP-608-NSE-CORALYS-INPUT-GATE-001'} ·
                  crates frozen · OF ±0.30 untouched
                </div>
                <div style={{ display: 'flex', gap: 10, flexWrap: 'wrap' }}>
                  <span style={chip('rgba(251,191,36,0.2)', op.amber)}>
                    PHASE {nse.coralys_input_gate.session_phase ?? '—'}
                  </span>
                  <span style={chip('rgba(34,211,238,0.2)', op.cyan)}>
                    PRIOR {nse.coralys_input_gate.prior_status ?? '—'}
                    {nse.coralys_input_gate.prior_next_session_date
                      ? ` · ${nse.coralys_input_gate.prior_next_session_date}`
                      : ''}
                  </span>
                  <span style={chip('rgba(251,191,36,0.22)', op.amber)}>
                    PREOPEN {nse.coralys_input_gate.preopen_status ?? '—'}
                    {nse.coralys_input_gate.preopen_n_records != null
                      ? ` · n=${nse.coralys_input_gate.preopen_n_records}`
                      : ''}
                  </span>
                  <span style={chip('rgba(251,113,133,0.25)', op.rose)}>
                    STANDASIDE {nse.coralys_input_gate.n_standaside ?? 0}
                  </span>
                  <span style={chip('rgba(52,211,153,0.25)', op.emerald)}>
                    QUAL {nse.coralys_input_gate.n_qualifying ?? nse.n_qualifying ?? '—'}
                  </span>
                </div>
              </>
            )}
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
              <span style={chip('rgba(251,113,133,0.25)', op.rose)}>
                STANDASIDE {nse.rejection_breakdown?.STANDASIDE_CORALYS_INPUT_GATE ?? nse.coralys_input_gate?.n_standaside ?? 0}
              </span>
              <span style={chip('rgba(148,163,184,0.22)', op.muted)}>
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

        </div>
      )}
    </div>
  );
}
