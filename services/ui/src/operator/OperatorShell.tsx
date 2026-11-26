import { useState, type CSSProperties } from 'react';
import OperatorOverview, { type OperatorWorkspace } from './OperatorOverview';
import CryptoWorkspace from './CryptoWorkspace';
import NseWorkspace from './NseWorkspace';
import { useOperatorState } from './useOperatorState';
import { chip, op } from './operatorStyles';
import { ageFromMs, displayOrUnavailable, formatAgeSec, staleLabel } from './freshness';

const NAV: { id: OperatorWorkspace; label: string }[] = [
  { id: 'overview', label: 'Overview' },
  { id: 'crypto', label: 'Crypto' },
  { id: 'nse', label: 'NSE' },
];

export default function OperatorShell() {
  const [ws, setWs] = useState<OperatorWorkspace>('crypto');
  const { state, loading } = useOperatorState(true);
  const now = Date.now();
  const fetchAge = state ? ageFromMs(state.fetched_at_ms, now) : null;
  const fetchStale = staleLabel(fetchAge, 15, 60);
  const authority = state?.live_order_authority ?? 0;

  return (
    <div style={shellStyle}>
      <header style={headerStyle}>
        <div>
          <div style={brandStyle}>ChronoSentiment</div>
          <p style={taglineStyle}>Gate5 operator · paper books · Pi telemetry</p>
        </div>
        <div style={{ display: 'flex', gap: 8, alignItems: 'center', flexWrap: 'wrap' }}>
          <span
            style={chip(
              authority === 0 ? 'rgba(143,191,159,0.14)' : 'rgba(212,136,138,0.2)',
              authority === 0 ? op.signal : op.loss
            )}
          >
            {Number(authority).toFixed(0)}% AUTH
          </span>
          <span
            style={chip(
              fetchStale === 'FRESH' ? 'rgba(143,191,159,0.12)' : 'rgba(196,165,116,0.18)',
              fetchStale === 'FRESH' ? op.signal : op.warn
            )}
          >
            {loading && !state
              ? 'LOADING'
              : state?.fetch_status !== 'ok'
                ? `TELEMETRY ${displayOrUnavailable(state?.fetch_status)}`
                : `${fetchStale} · ${formatAgeSec(fetchAge)}`}
          </span>
          <a
            href="http://192.168.0.9:8765/"
            target="_blank"
            rel="noreferrer"
            style={{
              ...chip('transparent', op.muted),
              textDecoration: 'none',
            }}
          >
            Pi CC ↗
          </a>
        </div>
      </header>

      <nav style={navStyle} aria-label="Operator workspaces">
        {NAV.map((item) => {
          const active = ws === item.id;
          return (
            <button
              key={item.id}
              type="button"
              onClick={() => setWs(item.id)}
              style={navBtn(active)}
            >
              {item.label}
            </button>
          );
        })}
      </nav>

      <main style={mainStyle}>
        {ws === 'overview' && (
          <OperatorOverview state={state} loading={loading} onNavigate={setWs} />
        )}
        {ws === 'crypto' && <CryptoWorkspace state={state} />}
        {ws === 'nse' && <NseWorkspace state={state} />}
      </main>
    </div>
  );
}

const shellStyle: CSSProperties = {
  minHeight: '100vh',
  margin: 0,
  padding: '24px 28px 48px',
  boxSizing: 'border-box',
  fontFamily: op.sans,
  color: op.text,
  background: `
    linear-gradient(180deg, #0e1218 0%, ${op.bg} 42%),
    ${op.bg}
  `,
};

const headerStyle: CSSProperties = {
  display: 'flex',
  justifyContent: 'space-between',
  alignItems: 'flex-end',
  gap: 16,
  flexWrap: 'wrap',
  marginBottom: 18,
  paddingBottom: 16,
  borderBottom: `1px solid ${op.border}`,
};

const brandStyle: CSSProperties = {
  margin: 0,
  fontSize: 32,
  fontWeight: 600,
  letterSpacing: '-0.04em',
  color: op.text,
  fontFamily: op.display,
  lineHeight: 1.1,
};

const taglineStyle: CSSProperties = {
  margin: '8px 0 0',
  color: op.muted,
  fontSize: 13,
  maxWidth: 420,
  letterSpacing: '0.01em',
};

const navStyle: CSSProperties = {
  display: 'flex',
  gap: 0,
  marginBottom: 24,
  borderBottom: `1px solid ${op.border}`,
};

const navBtn = (active: boolean): CSSProperties => ({
  padding: '10px 16px 12px',
  borderRadius: 0,
  border: 'none',
  borderBottom: active ? `2px solid ${op.signal}` : '2px solid transparent',
  background: 'transparent',
  color: active ? op.text : op.muted,
  fontFamily: op.sans,
  fontWeight: active ? 600 : 500,
  fontSize: 13,
  cursor: 'pointer',
  marginBottom: -1,
});

const mainStyle: CSSProperties = {
  maxWidth: 1280,
};
