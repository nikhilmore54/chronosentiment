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
  const [ws, setWs] = useState<OperatorWorkspace>('overview');
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
          <p style={taglineStyle}>Gate5 operator · Pi telemetry · 0% authority</p>
        </div>
        <div style={{ display: 'flex', gap: 8, alignItems: 'center', flexWrap: 'wrap' }}>
          <span style={chip(authority === 0 ? 'rgba(52,211,153,0.25)' : 'rgba(251,113,133,0.35)', op.text)}>
            AUTHORITY {Number(authority).toFixed(0)}%
          </span>
          <span
            style={chip(
              fetchStale === 'FRESH' ? 'rgba(45,212,191,0.22)' : 'rgba(251,191,36,0.3)',
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
            style={{ ...chip('rgba(148,163,184,0.18)', op.text), textDecoration: 'none' }}
          >
            Pi Command Center ↗
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
  padding: '28px 28px 40px',
  boxSizing: 'border-box',
  fontFamily: op.sans,
  color: op.text,
  background: `
    radial-gradient(ellipse 80% 50% at 10% -10%, rgba(45,212,191,0.12), transparent 55%),
    radial-gradient(ellipse 60% 40% at 90% 0%, rgba(251,191,36,0.06), transparent 50%),
    ${op.bg}
  `,
};

const headerStyle: CSSProperties = {
  display: 'flex',
  justifyContent: 'space-between',
  alignItems: 'flex-start',
  gap: 16,
  flexWrap: 'wrap',
  marginBottom: 20,
  paddingBottom: 18,
  borderBottom: `1px solid ${op.border}`,
};

const brandStyle: CSSProperties = {
  margin: 0,
  fontSize: 28,
  fontWeight: 700,
  letterSpacing: '-0.03em',
  color: op.cyan,
  fontFamily: op.display,
};

const taglineStyle: CSSProperties = {
  margin: '6px 0 0',
  color: op.muted,
  fontSize: 14,
  maxWidth: 420,
};

const navStyle: CSSProperties = {
  display: 'flex',
  gap: 6,
  marginBottom: 22,
  flexWrap: 'wrap',
};

const navBtn = (active: boolean): CSSProperties => ({
  padding: '8px 14px',
  borderRadius: 6,
  border: `1px solid ${active ? op.cyan : op.border}`,
  background: active ? 'rgba(45,212,191,0.14)' : 'transparent',
  color: active ? op.cyan : op.muted,
  fontFamily: op.sans,
  fontWeight: 600,
  fontSize: 13,
  cursor: 'pointer',
});

const mainStyle: CSSProperties = {
  maxWidth: 1200,
};
