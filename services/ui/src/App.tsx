import { useState, type CSSProperties } from 'react';
import Gate5PaperDashboard from './Gate5PaperDashboard';
import DecisionSurfacePanel from './components/DecisionSurfacePanel';
import ResearchForensicsWorkspace from './components/ResearchForensicsWorkspace';
import MultiHorizonBooksPanel from './components/MultiHorizonBooksPanel';
import { OperatorShell } from './operator';

type AppTab = 'operator' | 'forensics' | 'surface' | 'gate5' | 'horizons';

const tabBtn = (active: boolean, accent: string): CSSProperties => ({
  padding: '10px 16px',
  backgroundColor: active ? accent : '#1f2937',
  color: 'white',
  border: 'none',
  borderRadius: '6px',
  cursor: 'pointer',
  fontWeight: 700,
  boxShadow: active ? `0 4px 12px ${accent}55` : 'none',
});

function App() {
  const [activeTab, setActiveTab] = useState<AppTab>('operator');

  return (
    <div
      style={{
        padding: '20px',
        fontFamily: 'IBM Plex Sans, Segoe UI, sans-serif',
        backgroundColor: '#0a0a0a',
        color: '#e5e5e5',
        minHeight: '100vh',
      }}
    >
      <header
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          marginBottom: '30px',
          borderBottom: '1px solid #333',
          paddingBottom: '20px',
          flexWrap: 'wrap',
          gap: '15px',
        }}
      >
        <div>
          <h1
            style={{
              margin: 0,
              fontSize: '24px',
              fontWeight: 700,
              letterSpacing: '-0.02em',
              color: activeTab === 'operator' ? '#2dd4bf' : '#38bdf8',
            }}
          >
            {activeTab === 'operator' ? 'Gate5 Operator' : 'ChronoSentiment'}
          </h1>
          <p style={{ margin: '5px 0 0 0', color: '#888', fontSize: '14px' }}>
            {activeTab === 'operator'
              ? 'Pi telemetry · 0% authority · Crypto · NSE · Coralys input gate'
              : 'Presentation surfaces only · no order authority'}
          </p>
        </div>
        <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
          <button
            id="tab-operator"
            type="button"
            style={tabBtn(activeTab === 'operator', '#0d9488')}
            onClick={() => setActiveTab('operator')}
          >
            Operator
          </button>
          <button
            id="tab-forensics"
            type="button"
            style={tabBtn(activeTab === 'forensics', '#0ea5e9')}
            onClick={() => setActiveTab('forensics')}
          >
            Research & Forensics
          </button>
          <button
            id="tab-surface"
            type="button"
            style={tabBtn(activeTab === 'surface', '#0ea5e9')}
            onClick={() => setActiveTab('surface')}
          >
            Decision Surface
          </button>
          <button
            id="tab-gate5"
            type="button"
            style={tabBtn(activeTab === 'gate5', '#0284c7')}
            onClick={() => setActiveTab('gate5')}
          >
            Paper Trading
          </button>
          <button
            id="tab-horizons"
            type="button"
            style={tabBtn(activeTab === 'horizons', '#7c3aed')}
            onClick={() => setActiveTab('horizons')}
          >
            Multi-Horizon Books
          </button>
        </div>
      </header>

      <main
        style={{
          backgroundColor: '#111',
          padding: '20px',
          borderRadius: '8px',
          border: '1px solid #222',
        }}
      >
        {activeTab === 'operator' && (
          <OperatorShell onOpenHorizons={() => setActiveTab('horizons')} />
        )}
        {activeTab === 'forensics' && <ResearchForensicsWorkspace />}
        {activeTab === 'surface' && <DecisionSurfacePanel />}
        {activeTab === 'gate5' && <Gate5PaperDashboard />}
        {activeTab === 'horizons' && <MultiHorizonBooksPanel />}
      </main>
    </div>
  );
}

export default App;
