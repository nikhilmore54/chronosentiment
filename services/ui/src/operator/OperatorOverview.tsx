import type { CSSProperties, ReactNode } from 'react';
import type { OperatorState } from './types';
import { displayOrUnavailable, formatAgeSec, ageFromMs, staleLabel } from './freshness';
import { chip, labelStyle, op, panelStyle, valueStyle } from './operatorStyles';

export type OperatorWorkspace = 'overview' | 'crypto' | 'nse' | 'horizons';

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
    <section style={{ ...panelStyle, borderTop: `3px solid ${accent}`, minHeight: 160 }}>
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

export default function OperatorOverview({ state, loading, onNavigate }: Props) {
  const now = Date.now();
  const fetchAge = state ? ageFromMs(state.fetched_at_ms, now) : null;
  const fetchStale = staleLabel(fetchAge, 15, 60);

  const crypto = state?.crypto;
  const nse = state?.nse;
  const sys = state?.system;
  const authority = state?.live_order_authority ?? 0;

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
    <div style={{ fontFamily: op.sans, color: op.text }}>
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'flex-start',
          gap: 16,
          marginBottom: 18,
          flexWrap: 'wrap',
        }}
      >
        <div>
          <h2 style={{ margin: 0, fontSize: 22, fontWeight: 700, letterSpacing: '-0.02em' }}>
            Operator Overview
          </h2>
          <p style={{ margin: '6px 0 0', color: op.muted, fontSize: 13, maxWidth: 560 }}>
            Presentation-only surface over Pi telemetry. Same input → same mapped status. No order
            authority.
          </p>
        </div>
        <div style={{ display: 'flex', gap: 8, alignItems: 'center', flexWrap: 'wrap' }}>
          <span style={chip(authority === 0 ? op.emerald : op.rose, '#052e16')}>
            AUTHORITY {Number(authority).toFixed(0)}%
          </span>
          <span
            style={chip(
              fetchStale === 'FRESH' ? 'rgba(34,211,238,0.25)' : 'rgba(251,191,36,0.35)',
              op.text
            )}
          >
            {loading && !state
              ? 'LOADING'
              : state?.fetch_status !== 'ok'
                ? `TELEMETRY ${displayOrUnavailable(state?.fetch_status)}`
                : `FETCH ${fetchStale} · ${formatAgeSec(fetchAge)}`}
          </span>
          <a
            href="http://192.168.0.9:8765/"
            target="_blank"
            rel="noreferrer"
            style={{ ...chip('rgba(148,163,184,0.2)', op.text), textDecoration: 'none' }}
          >
            Pi Command Center ↗
          </a>
        </div>
      </div>

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
          UNAVAILABLE — {displayOrUnavailable(state.fetch_error, 'Pi /api/state unreachable')}. Fail
          closed; no invented healthy defaults.
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
        <Strip title="Crypto status" accent={op.cyan}>
          {!crypto?.available ? (
            <div style={{ color: op.rose, fontFamily: op.mono, fontSize: 13 }}>UNAVAILABLE</div>
          ) : (
            <>
              <div style={{ display: 'flex', gap: 8, marginBottom: 10, flexWrap: 'wrap' }}>
                <span style={chip('rgba(34,211,238,0.2)', op.cyan)}>
                  B {displayOrUnavailable(crypto.policy_b)}
                </span>
                <span style={chip('rgba(167,139,250,0.25)', op.violet)}>
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

        <Strip title="NSE status" accent={op.violet}>
          {!nse?.available ? (
            <div style={{ color: op.rose, fontFamily: op.mono, fontSize: 13 }}>UNAVAILABLE</div>
          ) : (
            <>
              <Kv k="Session / phase" v={displayOrUnavailable(nse.status)} />
              <Kv
                k="Next session"
                v={`${displayOrUnavailable(nse.next_session_date)} · n=${
                  nse.n_recommendations ?? '—'
                } · OF=${nse.n_order_flow_supported ?? '—'}${
                  nse.recommended_at_utc || nse.emitted_utc
                    ? ` · rec ${nse.recommended_at_utc || nse.emitted_utc}`
                    : ''
                }`}
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

        <Strip title="System health" accent={op.emerald}>
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

      <section style={{ ...panelStyle, marginBottom: 18, borderTop: `3px solid ${op.emerald}` }}>
        <div style={{ ...labelStyle, color: op.emerald }}>RP-611 Volume Absorption · Phase-1</div>
        {!state?.rp611 ? (
          <div style={{ color: op.dim, fontFamily: op.mono, fontSize: 12 }}>UNAVAILABLE</div>
        ) : (
          <div
            style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fit,minmax(240px,1fr))',
              gap: 12,
              fontFamily: op.mono,
              fontSize: 12,
            }}
          >
            <div>
              <div style={{ color: op.dim, marginBottom: 4 }}>611N NSE</div>
              <div>{displayOrUnavailable(state.rp611.nse?.disposition)}</div>
              <div style={{ color: op.muted }}>
                clock {state.rp611.nse?.experiment_clock_started ? 'ON' : 'OFF'} · T0{' '}
                {displayOrUnavailable(state.rp611.nse?.experiment_clock_started_utc)} · observer{' '}
                {state.rp611.nse?.observer_deployed ? 'DEPLOYED' : 'NO'} · cand{' '}
                {state.rp611.nse?.journal_counts?.candidates ?? '—'}
              </div>
            </div>
            <div>
              <div style={{ color: op.dim, marginBottom: 4 }}>611C Crypto spot</div>
              <div>{displayOrUnavailable(state.rp611.crypto?.disposition)}</div>
              <div style={{ color: op.muted }}>
                {displayOrUnavailable(state.rp611.crypto?.volume_source)} · L1 rows{' '}
                {state.rp611.crypto?.journal_counts?.l1_telemetry_rows ?? '—'} · cand{' '}
                {state.rp611.crypto?.journal_counts?.candidates ?? '—'}
              </div>
            </div>
          </div>
        )}
      </section>

      <div style={{ display: 'flex', gap: 10, flexWrap: 'wrap' }}>
        <button type="button" style={ctaStyle} onClick={() => onNavigate('crypto')}>
          Crypto workspace →
        </button>
        <button type="button" style={ctaStyle} onClick={() => onNavigate('nse')}>
          NSE workspace →
        </button>
        <button type="button" style={ctaStyle} onClick={() => onNavigate('horizons')}>
          Multi-Horizon →
        </button>
      </div>
    </div>
  );
}

const ctaStyle: CSSProperties = {
  padding: '10px 16px',
  background: 'rgba(34,211,238,0.12)',
  border: `1px solid ${op.cyan}`,
  color: op.cyan,
  borderRadius: 8,
  cursor: 'pointer',
  fontWeight: 700,
  fontFamily: op.sans,
  fontSize: 13,
};
