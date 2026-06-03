import { useState, useEffect } from 'react';
import { TradeInspector } from './components/TradeInspector/TradeInspector';
import { TimelineView } from './components/Timeline/TimelineView';
import { AnalyticsView } from './components/Analytics/AnalyticsView';
import type { CertifiedArtifact } from './types/artifact';
import type { TradeInspectorViewModel, ExplanationRule } from './types/tradeInspector';

import artifactTWAP50 from '../../docs/certification/certified_artifacts/certified_artifact_01_twap_50ms.json';
import artifactTWAP5 from '../../docs/certification/certified_artifacts/certified_artifact_02_twap_5ms.json';
import artifactMomentum5 from '../../docs/certification/certified_artifacts/certified_artifact_10_momentum_5ms.json';

const availableArtifacts: Record<string, CertifiedArtifact> = {
  'TWAP (50ms Latency)': artifactTWAP50 as CertifiedArtifact,
  'TWAP (5ms Latency)': artifactTWAP5 as CertifiedArtifact,
  'Momentum (5ms Latency)': artifactMomentum5 as CertifiedArtifact,
};

function App() {
  const [theme, setTheme] = useState<'light' | 'dark'>(() => {
    return (localStorage.getItem('chronosentiment_theme') as 'light' | 'dark') || 'light';
  });
  
  const [activeArtifactName, setActiveArtifactName] = useState<string>('TWAP (50ms Latency)');
  const artifact = availableArtifacts[activeArtifactName];

  const [selectedTradeId, setSelectedTradeId] = useState<string>(artifact.trade_deltas[0]?.trade_id || '');
  const [activeTab, setActiveTab] = useState<'analytics' | 'timeline' | 'inspector'>('analytics');

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
    localStorage.setItem('chronosentiment_theme', theme);
  }, [theme]);

  useEffect(() => {
    setSelectedTradeId(artifact.trade_deltas[0]?.trade_id || '');
  }, [artifact]);

  const rulesMap: Record<string, ExplanationRule> = {};
  artifact.rules.forEach(r => {
    rulesMap[r.id] = {
      id: r.id,
      type: r.type,
      severity: r.severity as "info" | "warning" | "critical",
      message: r.message
    };
  });

  const selectedTrade = artifact.trade_deltas.find(t => t.trade_id === selectedTradeId);

  const viewModel: TradeInspectorViewModel | null = selectedTrade ? {
    trade_delta: selectedTrade,
    rules_map: rulesMap
  } : null;

  // Derive Session Header details
  const stratName = artifact.strategy?.archetype?.toUpperCase() || 'UNKNOWN';
  const latency = artifact.environment?.latency_injected_ms || 0;
  const structDiv = ((artifact.divergence?.structural_divergence || 0) * 100).toFixed(1);

  // Jump handler for the Causality Graph
  const handleJumpToTrade = (tradeId: string) => {
    setSelectedTradeId(tradeId);
    setActiveTab('inspector');
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', minHeight: '100vh', backgroundColor: 'var(--bg-main)' }}>
      
      {/* Top Navigation Bar */}
      <div style={{ background: 'var(--bg-panel)', borderBottom: '1px solid var(--border-color)', display: 'flex', flexDirection: 'column', padding: '1rem 2rem', zIndex: 10, boxShadow: 'var(--shadow-subtle)' }}>
        
        {/* Row 1: Brand & Utilities */}
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '2rem' }}>
            <h1 style={{ fontSize: '1.25rem', color: 'var(--text-primary)', margin: 0, letterSpacing: '0.02em' }}>ChronoSentiment</h1>
            
            {/* Session Context Header */}
            <div style={{ display: 'flex', gap: '0.75rem', alignItems: 'center', color: 'var(--text-secondary)', fontSize: '0.875rem' }}>
              <span style={{ fontWeight: 600 }}>{stratName} Strategy</span>
              <span style={{ color: 'var(--border-color)' }}>|</span>
              <span>NIFTY Futures</span>
              <span style={{ color: 'var(--border-color)' }}>|</span>
              <span style={{ color: latency > 0 ? 'var(--status-warning)' : 'inherit' }}>{latency}ms Latency</span>
              <span style={{ color: 'var(--border-color)' }}>|</span>
              <span style={{ color: structDiv !== '0.0' ? 'var(--status-critical)' : 'var(--status-success)', fontWeight: 500 }}>{structDiv}% Structural Divergence</span>
            </div>
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
            <select 
              value={activeArtifactName}
              onChange={(e) => setActiveArtifactName(e.target.value)}
              style={{
                padding: '0.35rem 0.75rem',
                borderRadius: 'var(--radius-md)',
                background: 'var(--bg-card)',
                color: 'var(--text-primary)',
                border: '1px solid var(--border-color)',
                outline: 'none',
                cursor: 'pointer',
                fontSize: '0.85rem'
              }}
            >
              {Object.keys(availableArtifacts).map(name => (
                <option key={name} value={name}>{name}</option>
              ))}
            </select>

            <button 
              onClick={() => setTheme(theme === 'light' ? 'dark' : 'light')}
              style={{
                background: 'var(--bg-card)',
                border: '1px solid var(--border-color)',
                borderRadius: 'var(--radius-md)',
                padding: '0.35rem 0.75rem',
                cursor: 'pointer',
                color: 'var(--text-primary)',
                fontSize: '0.85rem'
              }}
            >
              {theme === 'light' ? '🌙' : '☀️'}
            </button>
          </div>
        </div>

        {/* Row 2: Tabs */}
        <div style={{ display: 'flex', gap: '1.5rem', marginTop: '0.5rem' }}>
          {[
            { id: 'analytics', label: 'Analytics' },
            { id: 'timeline', label: 'Timeline' },
            { id: 'inspector', label: 'Inspector' }
          ].map(tab => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id as 'analytics' | 'timeline' | 'inspector')}
              style={{
                background: 'transparent',
                border: 'none',
                borderBottom: activeTab === tab.id ? '2px solid var(--accent-blue)' : '2px solid transparent',
                color: activeTab === tab.id ? 'var(--accent-blue)' : 'var(--text-secondary)',
                padding: '0.5rem 0.25rem',
                fontSize: '0.875rem',
                fontWeight: activeTab === tab.id ? 600 : 500,
                cursor: 'pointer',
                textTransform: 'uppercase',
                letterSpacing: '0.05em',
                transition: 'all 0.2s'
              }}
            >
              {tab.label}
            </button>
          ))}
        </div>
      </div>

      {/* Main Content Area */}
      <div style={{ flex: 1, overflowY: 'auto', position: 'relative' }}>
        {activeTab === 'analytics' && (
          <AnalyticsView artifact={artifact as CertifiedArtifact} />
        )}

        {activeTab === 'timeline' && (
          <TimelineView timeline={artifact.timeline} onJumpToTrade={handleJumpToTrade} />
        )}
        
        {activeTab === 'inspector' && (
          <div style={{ padding: '2rem', maxWidth: '1000px', margin: '0 auto' }}>
            <div style={{ marginBottom: '2rem', display: 'flex', alignItems: 'center', gap: '1rem' }}>
              <span style={{ fontSize: '0.875rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Select Trade</span>
              <select 
                value={selectedTradeId}
                onChange={(e) => setSelectedTradeId(e.target.value)}
                style={{
                  padding: '0.5rem 1rem',
                  borderRadius: 'var(--radius-md)',
                  background: 'var(--bg-panel)',
                  color: 'var(--text-primary)',
                  border: '1px solid var(--border-color)',
                  outline: 'none',
                  cursor: 'pointer',
                  minWidth: '200px'
                }}
              >
                {artifact.trade_deltas.map(t => (
                  <option key={t.trade_id} value={t.trade_id}>Trade {t.trade_id} {t.delta.diverged ? ' (Diverged)' : ''}</option>
                ))}
              </select>
            </div>

            {viewModel ? (
              <TradeInspector model={viewModel} />
            ) : (
              <div style={{ padding: '2rem', color: 'var(--text-muted)' }}>Select a trade to inspect</div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

export default App;
