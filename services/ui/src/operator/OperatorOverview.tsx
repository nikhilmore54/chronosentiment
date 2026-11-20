import type { CSSProperties, ReactNode } from 'react';
import type { OperatorState } from './types';
import { displayOrUnavailable } from './freshness';
import { chip, labelStyle, op, panelStyle, valueStyle } from './operatorStyles';

export type OperatorWorkspace = 'overview' | 'crypto' | 'nse';

type Props = {
  state: OperatorState | null;
  loading: boolean;
  onNavigate: (ws: OperatorWorkspace) => void;
};

function Strip({
  title,
  children,
  accent,
}: {
  title: string;
  children: ReactNode;
  accent: string;
}) {
  return (
    <section style={{ ...panelStyle, borderTop: `3px solid ${accent}`, minHeight: 148 }}>
      <div style={{ ...labelStyle, color: accent }}>{title}</div>
      {children}
    </section>
  );
}

function Kv({ k, v }: { k: string; v: string }) {
  return (
    <div style={{ marginBottom: 8 }}>
      <div style={{ fontSize: 11, color: op.dim }}>{k}</div>
      <div style={{ ...valueStyle, fontSize: 14 }}>{v}</div>
    </div>
  );
}

export default function OperatorOverview({ state, onNavigate }: Props) {
  const crypto = state?.crypto;
  const nse = state?.nse;
  const sys = state?.system;

  const ofActive = crypto?.order_flow?.active;
  const ofLabel = ofActive
    ? `ACTIVE ${crypto?.order_flow?.direction ?? ''}`.trim()
    : crypto?.order_flow_last_exit
      ? 'EXITED (last)'
      : 'NONE';

  const planState = crypto?.trade_plan?.state ?? (crypto?.trade_plan?.is_active ? 'OPEN' : null);

  const boardEmptyReason =
    nse?.available && (nse.total_evaluated == null || nse.total_evaluated === 0)
      ? nse.next_session_note || 'Session board empty / weekend or outside eval window'
      : null;

  return (
    <div>
      <p style={{ margin: '0 0 16px', color: op.muted, fontSize: 14, maxWidth: 560 }}>
        Live books at a glance. Fail closed on missing Pi fields — no invented healthy defaults.
      </p>

      {state?.fetch_status !== 'ok' && state && (
        <div
          style={{
            ...panelStyle,
            marginBottom: 14,
            borderColor: op.rose,
            color: op.rose,
            fontFamily: op.mono,
            fontSize: 13,
          }}
        >
          UNAVAILABLE — {displayOrUnavailable(state.fetch_error, 'Pi /api/state unreachable')}.
        </div>
      )}

      <div
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(260px, 1fr))',
          gap: 14,
          marginBottom: 18,
        }}
      >
        <Strip title="Crypto" accent={op.cyan}>
          {!crypto?.available ? (
            <div style={{ color: op.rose, fontFamily: op.mono, fontSize: 13 }}>UNAVAILABLE</div>
          ) : (
            <>
              <div style={{ display: 'flex', gap: 8, marginBottom: 10, flexWrap: 'wrap' }}>
                <span style={chip('rgba(45,212,191,0.2)', op.cyan)}>
                  B {displayOrUnavailable(crypto.policy_b)}
                </span>
                <span style={chip('rgba(251,191,36,0.22)', op.amber)}>
                  C {displayOrUnavailable(crypto.policy_c)}
                </span>
              </div>
              <Kv k="Trade plan" v={displayOrUnavailable(planState)} />
              <Kv k="Order flow" v={ofLabel} />
              <Kv
                k="Mid / quote age"
                v={`${
                  crypto.reference_price != null ? crypto.reference_price.toFixed(2) : 'UNAVAILABLE'
                } · ${
                  crypto.quote_age_ms != null
                    ? `${Math.round(crypto.quote_age_ms)}ms`
                    : 'UNAVAILABLE'
                }`}
              />
            </>
          )}
        </Strip>

        <Strip title="NSE" accent={op.amber}>
          {!nse?.available ? (
            <div style={{ color: op.rose, fontFamily: op.mono, fontSize: 13 }}>UNAVAILABLE</div>
          ) : (
            <>
              <Kv
                k="Session / Coralys phase"
                v={`${displayOrUnavailable(nse.status)} · ${displayOrUnavailable(
                  nse.session_phase ?? nse.coralys_input_gate?.session_phase
                )}`}
              />
              <Kv
                k="Input gate"
                v={
                  nse.coralys_input_gate
                    ? `${nse.coralys_input_gate.gate_enabled === false ? 'OFF' : 'ON'} · prior ${
                        nse.coralys_input_gate.prior_status ?? '—'
                      } · preopen ${nse.coralys_input_gate.preopen_status ?? '—'} · standaside ${
                        nse.coralys_input_gate.n_standaside ?? 0
                      }`
                    : 'UNAVAILABLE'
                }
              />
              <Kv
                k="Next session"
                v={`${displayOrUnavailable(nse.next_session_date)} · n=${
                  nse.n_recommendations ?? '—'
                } · OF=${nse.n_order_flow_supported ?? '—'}`}
              />
              <Kv
                k="Universe"
                v={`${nse.classified_count ?? '—'} / ${nse.census_total ?? '—'} classified · Q ${
                  nse.quarantined_count ?? '—'
                }`}
              />
              {boardEmptyReason && (
                <div style={{ fontSize: 12, color: op.amber, marginTop: 4 }}>{boardEmptyReason}</div>
              )}
            </>
          )}
        </Strip>

        <Strip title="System" accent={op.emerald}>
          {!sys?.available ? (
            <div style={{ color: op.rose, fontFamily: op.mono, fontSize: 13 }}>UNAVAILABLE</div>
          ) : (
            <>
              <div style={{ display: 'flex', gap: 6, flexWrap: 'wrap', marginBottom: 10 }}>
                <span
                  style={chip(
                    sys.service_status?.active ? 'rgba(52,211,153,0.25)' : 'rgba(251,113,133,0.3)',
                    op.text
                  )}
                >
                  {sys.service_status?.status_text ??
                    (sys.service_status?.active ? 'SERVICE ACTIVE' : 'SERVICE UNKNOWN')}
                </span>
                {sys.telemetry_gates?.window_tainted != null && (
                  <span
                    style={chip(
                      sys.telemetry_gates.window_tainted
                        ? 'rgba(251,113,133,0.35)'
                        : 'rgba(52,211,153,0.2)',
                      op.text
                    )}
                  >
                    {sys.telemetry_gates.window_tainted ? 'WINDOW TAINTED' : 'GATES OK'}
                  </span>
                )}
              </div>
              <Kv
                k="Quote coverage / drift"
                v={`${
                  sys.telemetry_gates?.quote_coverage_pct != null
                    ? `${sys.telemetry_gates.quote_coverage_pct}%`
                    : 'UNAVAILABLE'
                } · drift ${
                  sys.telemetry_gates?.max_drift_ms != null
                    ? `${sys.telemetry_gates.max_drift_ms}ms`
                    : 'UNAVAILABLE'
                }`}
              />
              <Kv
                k="Checkpoint"
                v={`${displayOrUnavailable(sys.checkpoint_integrity?.status)} · ${displayOrUnavailable(
                  sys.checkpoint_integrity?.last_heartbeat_utc
                )}`}
              />
              <Kv k="Server IST" v={displayOrUnavailable(sys.server_time_ist)} />
            </>
          )}
        </Strip>
      </div>

      <div style={{ display: 'flex', gap: 10, flexWrap: 'wrap' }}>
        <button type="button" style={ctaStyle} onClick={() => onNavigate('crypto')}>
          Open Crypto →
        </button>
        <button type="button" style={ctaStyle} onClick={() => onNavigate('nse')}>
          Open NSE →
        </button>
      </div>
    </div>
  );
}

const ctaStyle: CSSProperties = {
  padding: '10px 16px',
  background: 'rgba(45,212,191,0.1)',
  border: `1px solid ${op.cyan}`,
  color: op.cyan,
  borderRadius: 6,
  cursor: 'pointer',
  fontWeight: 700,
  fontFamily: op.sans,
  fontSize: 13,
};
