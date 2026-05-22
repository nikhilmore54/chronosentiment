import React, { useState, useEffect } from 'react';
import RunGA from './components/RunGA';
import StrategyInspector from './components/StrategyInspector';
import CompareStrategies from './components/CompareStrategies';
import GlobalRanking from './components/GlobalRanking';

const TABS = [
  { id: 'run-ga',            label: 'Run GA' },
  { id: 'inspect-strategy',  label: 'Inspect Strategy' },
  { id: 'compare-strategies',label: 'Compare Strategies' },
  { id: 'global-ranking',    label: 'Global Ranking' },
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

export default function App() {
  const [activeTab, setActiveTab] = useState('run-ga');
  const [selectedStrategyId, setSelectedStrategyId] = useState(null);
  const [selectedSeed, setSelectedSeed] = useState(null);
  const [selectedStrategyId2, setSelectedStrategyId2] = useState(null);
  const [selectedSeed2, setSelectedSeed2] = useState(null);
  const clock = useClock();

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

      {/* Operational Awareness Strip */}
      <div style={{ display: 'flex', alignItems: 'center', gap: '32px', padding: '12px 32px', background: 'var(--card2)', borderBottom: '1px solid var(--b)', fontSize: '12px', fontFamily: 'var(--sans)' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <span style={{ color: 'var(--tm)', fontWeight: 500 }}>System State:</span>
          <span style={{ display: 'flex', alignItems: 'center', gap: '6px', color: 'var(--grn)', fontWeight: 600 }}>
            <span style={{ width: '6px', height: '6px', borderRadius: '50%', background: 'var(--grn)', display: 'inline-block' }}></span>
            Nominal
          </span>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <span style={{ color: 'var(--tm)', fontWeight: 500 }}>Chronology Engine:</span>
          <span style={{ color: 'var(--t1)', fontFamily: 'var(--mono)' }}>Synchronized (1.00x)</span>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <span style={{ color: 'var(--tm)', fontWeight: 500 }}>Governor:</span>
          <span style={{ color: 'var(--t1)', fontFamily: 'var(--mono)' }}>Active</span>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <span style={{ color: 'var(--tm)', fontWeight: 500 }}>Cohort:</span>
          <span style={{ color: 'var(--t1)', fontFamily: 'var(--mono)' }}>NSE_ALPHA_01</span>
        </div>
      </div>

      {/* Content Workspace */}
      <div style={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
        {/* Persistent Left Rail Navigation */}
        <aside style={{ width: '220px', flexShrink: 0, borderRight: '1px solid var(--b)', background: 'var(--card)', display: 'flex', flexDirection: 'column' }}>
          <div style={{ padding: '24px 20px', fontSize: '11px', fontWeight: 600, color: 'var(--tm)', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
            Workspaces
          </div>
          <nav style={{ display: 'flex', flexDirection: 'column', padding: '0 12px' }}>
            {TABS.map(tab => (
              <button
                key={tab.id}
                style={{
                  display: 'flex', alignItems: 'center', padding: '8px 12px', marginBottom: '4px',
                  borderRadius: '6px', border: 'none', background: activeTab === tab.id ? 'var(--card2)' : 'transparent',
                  color: activeTab === tab.id ? 'var(--t1)' : 'var(--t2)',
                  fontWeight: activeTab === tab.id ? 600 : 500,
                  fontSize: '13px', fontFamily: 'var(--sans)', textAlign: 'left', cursor: 'pointer',
                  transition: 'all 0.15s ease'
                }}
                onClick={() => setActiveTab(tab.id)}
              >
                {tab.label}
              </button>
            ))}
          </nav>
        </aside>

        {/* Main Workspace Area */}
        <main className="cs-main" style={{ flex: 1, overflowY: 'auto' }}>
          {renderContent()}
        </main>
      </div>

      {/* Footer */}
      <footer className="cs-footer">
        <span>ChronoSentiment · NSE Execution Intelligence</span>
        <span>{clock}</span>
      </footer>
    </div>
  );
}
