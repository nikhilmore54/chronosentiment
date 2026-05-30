import React, { useState, useEffect } from 'react';
import RunGA from './components/RunGA';
import StrategyInspector from './components/StrategyInspector';
import CompareStrategies from './components/CompareStrategies';
import GlobalRanking from './components/GlobalRanking';
import { apiUrl } from './services/api';

const TABS = [
  { id: 'run-ga',             label: 'Run GA',            icon: '⚡' },
  { id: 'inspect-strategy',   label: 'Inspect Strategy',  icon: '🔬' },
  { id: 'compare-strategies', label: 'Compare Strategies',icon: '⚖' },
  { id: 'global-ranking',     label: 'Global Ranking',    icon: '📊' },
];

function useClock() {
  const [ts, setTs] = useState('');
  useEffect(() => {
    const fmt = () => {
      const p = {};
      new Intl.DateTimeFormat('en-IN', {
        timeZone: 'Asia/Kolkata', hour12: false,
        year: 'numeric', month: '2-digit', day: '2-digit',
        hour: '2-digit', minute: '2-digit', second: '2-digit',
      }).formatToParts(new Date()).forEach(x => { p[x.type] = x.value; });
      setTs(`${p.year}-${p.month}-${p.day} ${p.hour}:${p.minute}:${p.second} IST`);
    };
    fmt();
    const id = setInterval(fmt, 1000);
    return () => clearInterval(id);
  }, []);
  return ts;
}

// useSystemStatus — maps canonical observatory_state schema fields.
// Field authority: observatory_state.schema.json (chrono:schema:observatory_state:v1).
// Fallback defaults are used only when the backend is unreachable; they are not
// authoritative values and are clearly marked as offline placeholders.
function useSystemStatus() {
  const [status, setStatus] = useState({
    // system_phase: INITIALIZING | LIVE | REPLAYING | THROTTLED | DEGRADED | HALTED | MAINTENANCE
    system_phase:        null,
    // governor_state fields
    throttle_state:      null,   // OPEN | THROTTLED | CLOSED
    cohort_id:           null,
    active_cohort_size:  null,
    governor_version:    null,
    // kernel_state fields
    queue_depth:         null,
    fill_latency_ns:     null,
    sync_ratio:          null,
    events_per_second:   null,
    kernel_version:      null,
    // snapshot anchor
    snapshot_sequence_id: null,
    // connectivity
    online: false,
  });

  useEffect(() => {
    // Primary: attempt GET /observatory for a canonical ObservatoryState snapshot.
    // Fallback: GET /health for legacy field mapping.
    // Both fail gracefully — UI renders null fields as '—'.
    const tryObservatory = () =>
      fetch(apiUrl('/observatory'))
        .then(r => r.ok ? r.json() : Promise.reject())
        .then(d => {
          setStatus({
            system_phase:        d.system_phase        ?? null,
            throttle_state:      d.governor_state?.throttle_state     ?? null,
            cohort_id:           d.governor_state?.cohort_id          ?? null,
            active_cohort_size:  d.governor_state?.active_cohort_size ?? null,
            governor_version:    d.governor_state?.governor_version   ?? null,
            queue_depth:         d.kernel_state?.queue_depth          ?? null,
            fill_latency_ns:     d.kernel_state?.fill_latency_ns      ?? null,
            sync_ratio:          d.kernel_state?.sync_ratio           ?? null,
            events_per_second:   d.kernel_state?.events_per_second    ?? null,
            kernel_version:      d.kernel_state?.kernel_version       ?? null,
            snapshot_sequence_id: d.snapshot_sequence_id             ?? null,
            online: true,
          });
        });

    const tryHealth = () =>
      fetch(apiUrl('/health'))
        .then(r => r.ok ? r.json() : Promise.reject())
        .then(d => {
          // Legacy /health field mapping — best-effort projection only.
          setStatus(prev => ({
            ...prev,
            system_phase:   d.system_phase ?? d.state ?? null,
            throttle_state: d.throttle_state ?? d.governor ?? null,
            cohort_id:      d.cohort_id ?? d.cohort ?? null,
            online: true,
          }));
        });

    tryObservatory().catch(() => tryHealth().catch(() => {
      // Both endpoints offline — keep null defaults; online: false.
      setStatus(prev => ({ ...prev, online: false }));
    }));
  }, []);

  return status;
}

export default function App() {
  const [activeTab, setActiveTab] = useState('run-ga');
  const [selectedStrategyId,  setSelectedStrategyId]  = useState(null);
  const [selectedSeed,        setSelectedSeed]        = useState(null);
  const [selectedStrategyId2, setSelectedStrategyId2] = useState(null);
  const [selectedSeed2,       setSelectedSeed2]       = useState(null);
  const clock  = useClock();
  const sysStatus = useSystemStatus();

  const handleSetSelectedStrategyForInspection = (strategyId, seed, isSecondStrategy = false) => {
    if (isSecondStrategy) {
      setSelectedStrategyId2(strategyId);
      setSelectedSeed2(seed);
    } else {
      setSelectedStrategyId(strategyId);
      setSelectedSeed(seed);
      setSelectedStrategyId2(null);
      setSelectedSeed2(null);
    }
    setActiveTab('inspect-strategy');
  };

  const renderContent = () => {
    switch (activeTab) {
      case 'run-ga':
        return <RunGA setSelectedStrategyForInspection={handleSetSelectedStrategyForInspection} />;
      case 'inspect-strategy':
        return (
          <StrategyInspector
            strategyId={selectedStrategyId}
            seed={selectedSeed}
            strategyId2={selectedStrategyId2}
            seed2={selectedSeed2}
            onReset={() => {
              setSelectedStrategyId(null);
              setSelectedSeed(null);
              setSelectedStrategyId2(null);
              setSelectedSeed2(null);
            }}
          />
        );
      case 'compare-strategies':
        return <CompareStrategies setSelectedStrategyForInspection={handleSetSelectedStrategyForInspection} />;
      case 'global-ranking':
        return <GlobalRanking />;
      default:
        return <RunGA setSelectedStrategyForInspection={handleSetSelectedStrategyForInspection} />;
    }
  };

  // Derive phase color from canonical system_phase enum values.
  const phaseColor = sysStatus.system_phase === 'LIVE'        ? 'var(--grn)'
                   : sysStatus.system_phase === 'REPLAYING'   ? 'var(--blu)'
                   : sysStatus.system_phase === 'THROTTLED'   ? 'var(--amb)'
                   : sysStatus.system_phase === 'DEGRADED'    ? 'var(--amb)'
                   : sysStatus.system_phase === 'HALTED'      ? 'var(--red)'
                   : sysStatus.system_phase === 'MAINTENANCE' ? 'var(--amb)'
                   : 'var(--tm)';
  const phaseDot   = sysStatus.system_phase === 'LIVE'        ? 'grn'
                   : sysStatus.system_phase === 'REPLAYING'   ? 'blu'
                   : sysStatus.system_phase === 'THROTTLED'   ? 'amb'
                   : sysStatus.system_phase === 'DEGRADED'    ? 'amb'
                   : sysStatus.system_phase === 'HALTED'      ? 'red'
                   : 'amb';

  // Format fill_latency_ns → human-readable (µs or ms)
  const fmtLatency = (ns) => {
    if (ns === null) return '—';
    if (ns < 1000) return `${ns}ns`;
    if (ns < 1_000_000) return `${(ns / 1000).toFixed(1)}µs`;
    return `${(ns / 1_000_000).toFixed(1)}ms`;
  };

  return (
    <div className="cs-app">
      {/* Header */}
      <header className="cs-header">
        <div className="cs-logo">
          <div className="cs-logo-mark">C</div>
          <div className="cs-logo-text">
            <h1>ChronoSentiment</h1>
            <p>Execution Intelligence Platform</p>
          </div>
        </div>
        <div className="cs-clock">{clock}</div>
      </header>

      {/* Operational Awareness Strip — fields from observatory_state.schema.json */}
      <div style={{
        display: 'flex', alignItems: 'center', gap: '24px',
        padding: '8px 32px', background: 'var(--card2)',
        borderBottom: '1px solid var(--b)', fontSize: '11px',
        fontFamily: 'var(--sans)', overflowX: 'auto', flexShrink: 0,
      }}>
        {/* system_phase */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px', flexShrink: 0 }}>
          <span style={{ color: 'var(--tm)', fontWeight: 500 }}>Phase:</span>
          <span style={{ display: 'flex', alignItems: 'center', gap: '5px', color: phaseColor, fontWeight: 600, fontFamily: 'var(--mono)', fontSize: '10px', letterSpacing: '0.04em' }}>
            {sysStatus.online && <span className={`cs-status-dot ${phaseDot}`}></span>}
            {sysStatus.system_phase ?? (sysStatus.online ? 'UNKNOWN' : 'OFFLINE')}
          </span>
        </div>
        {/* governor throttle_state */}
        {sysStatus.throttle_state && (
          <div style={{ display: 'flex', alignItems: 'center', gap: '6px', flexShrink: 0 }}>
            <span style={{ color: 'var(--tm)', fontWeight: 500 }}>Throttle:</span>
            <span style={{ color: sysStatus.throttle_state === 'OPEN' ? 'var(--grn)' : sysStatus.throttle_state === 'CLOSED' ? 'var(--red)' : 'var(--amb)', fontFamily: 'var(--mono)', fontSize: '10px', fontWeight: 600 }}>
              {sysStatus.throttle_state}
            </span>
          </div>
        )}
        {/* cohort_id + active_cohort_size */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px', flexShrink: 0 }}>
          <span style={{ color: 'var(--tm)', fontWeight: 500 }}>Cohort:</span>
          <span style={{ color: 'var(--t1)', fontFamily: 'var(--mono)' }}>
            {sysStatus.cohort_id ?? '—'}
            {sysStatus.active_cohort_size !== null && (
              <span style={{ color: 'var(--tm)', marginLeft: '4px' }}>({sysStatus.active_cohort_size})</span>
            )}
          </span>
        </div>
        {/* kernel queue_depth */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px', flexShrink: 0 }}>
          <span style={{ color: 'var(--tm)', fontWeight: 500 }}>Queue:</span>
          <span style={{ color: 'var(--t1)', fontFamily: 'var(--mono)' }}>{sysStatus.queue_depth ?? '—'}</span>
        </div>
        {/* kernel fill_latency_ns */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px', flexShrink: 0 }}>
          <span style={{ color: 'var(--tm)', fontWeight: 500 }}>Fill latency:</span>
          <span style={{ color: 'var(--t1)', fontFamily: 'var(--mono)' }}>{fmtLatency(sysStatus.fill_latency_ns)}</span>
        </div>
        {/* kernel sync_ratio */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px', flexShrink: 0 }}>
          <span style={{ color: 'var(--tm)', fontWeight: 500 }}>Sync:</span>
          <span style={{ color: sysStatus.sync_ratio !== null ? (sysStatus.sync_ratio >= 0.95 ? 'var(--grn)' : sysStatus.sync_ratio >= 0.8 ? 'var(--amb)' : 'var(--red)') : 'var(--tm)', fontFamily: 'var(--mono)' }}>
            {sysStatus.sync_ratio !== null ? sysStatus.sync_ratio.toFixed(2) : '—'}
          </span>
        </div>
        {/* kernel events_per_second */}
        {sysStatus.events_per_second !== null && (
          <div style={{ display: 'flex', alignItems: 'center', gap: '6px', flexShrink: 0 }}>
            <span style={{ color: 'var(--tm)', fontWeight: 500 }}>EPS:</span>
            <span style={{ color: 'var(--t1)', fontFamily: 'var(--mono)' }}>{sysStatus.events_per_second.toFixed(1)}</span>
          </div>
        )}
        {/* snapshot_sequence_id — chronology anchor */}
        {sysStatus.snapshot_sequence_id !== null && (
          <div style={{ display: 'flex', alignItems: 'center', gap: '6px', flexShrink: 0 }}>
            <span style={{ color: 'var(--tm)', fontWeight: 500 }}>Seq:</span>
            <span style={{ color: 'var(--t2)', fontFamily: 'var(--mono)' }}>#{sysStatus.snapshot_sequence_id}</span>
          </div>
        )}
        {/* active workspace breadcrumb */}
        <div style={{ marginLeft: 'auto', flexShrink: 0 }}>
          <span style={{ color: 'var(--tm)', fontSize: '10px', fontFamily: 'var(--mono)' }}>
            {activeTab.replace(/-/g, ' ').replace(/\b\w/g, c => c.toUpperCase())}
          </span>
        </div>
      </div>

      {/* Content Workspace */}
      <div style={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
        {/* Persistent Left Rail Navigation */}
        <aside style={{
          width: '216px', flexShrink: 0,
          borderRight: '1px solid var(--b)',
          background: 'var(--card)',
          display: 'flex', flexDirection: 'column',
          overflowY: 'auto',
        }}>
          <div style={{
            padding: '20px 16px 12px',
            fontSize: '10px', fontWeight: 700,
            color: 'var(--tm)', textTransform: 'uppercase',
            letterSpacing: '0.08em',
          }}>
            Workspaces
          </div>
          <nav style={{ display: 'flex', flexDirection: 'column', padding: '0 8px 16px' }}>
            {TABS.map(tab => (
              <button
                key={tab.id}
                className={`cs-nav-item${activeTab === tab.id ? ' active' : ''}`}
                onClick={() => setActiveTab(tab.id)}
              >
                <span className="cs-nav-icon">{tab.icon}</span>
                {tab.label}
              </button>
            ))}
          </nav>

          {/* Rail footer: active workspace context */}
          <div style={{
            marginTop: 'auto', padding: '16px',
            borderTop: '1px solid var(--b)',
            fontSize: '11px', color: 'var(--tm)',
            fontFamily: 'var(--mono)',
          }}>
            <div style={{ marginBottom: '4px', color: 'var(--t2)', fontWeight: 600 }}>Active</div>
            <div style={{ wordBreak: 'break-all' }}>
              {selectedStrategyId || '—'}
            </div>
          </div>
        </aside>

        {/* Main Workspace Area */}
        <main className="cs-main" style={{ flex: 1, overflowY: 'auto' }}>
          {renderContent()}
        </main>
      </div>

      {/* Footer */}
      <footer className="cs-footer">
        <span>ChronoSentiment · NSE Execution Intelligence · v2026</span>
        <span style={{ color: 'var(--t2)' }}>{sysStatus.cohort_id ?? ''}</span>
      </footer>
    </div>
  );
}
