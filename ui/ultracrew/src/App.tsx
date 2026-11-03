import { useEffect, useState } from 'react';
import { Landing } from './Landing';
import { Constraints } from './Constraints';
import { PlannerWorkflow } from './workflow/PlannerWorkflow';

export interface SimulationState {
  affected_nurse: string;
  missed_shifts: number;
  recovered_shifts: number;
  recovery_eta: number;
  creditors: Record<string, number>;
  balance_changes: Record<string, { previous: number; current: number }>;
  coverage_impact: string;
}

function App() {
  const [showDemo, setShowDemo] = useState(false);
  const [currentTab, setCurrentTab] = useState<'scheduler' | 'constraints'>('scheduler');
  const [backendOnline, setBackendOnline] = useState<boolean | null>(null);
  const [workflowMode, setWorkflowMode] = useState<'blank' | 'sample'>('blank');
  const [workflowTrigger, setWorkflowTrigger] = useState<number>(0);

  // Poll backend health every 5 seconds
  useEffect(() => {
    const check = () => {
      fetch('/api/health')
        .then(res => setBackendOnline(res.ok))
        .catch(() => setBackendOnline(false));
    };
    check();
    const id = setInterval(check, 5000);
    return () => clearInterval(id);
  }, []);

  const loadSampleHospital = () => {
    setShowDemo(true);
    setWorkflowMode('sample');
    setWorkflowTrigger(Date.now());
    setCurrentTab('scheduler');
  };

  if (!showDemo) {
    return (
      <Landing
        onStartDemo={loadSampleHospital}
      />
    );
  }

  // Backend status pill colours
  const statusColor = backendOnline === null ? '#94a3b8' : backendOnline ? '#10b981' : '#ef4444';
  const statusLabel = backendOnline === null ? 'Checking...' : backendOnline ? 'Connected' : 'Offline';
  const statusDot = backendOnline === null ? '●' : backendOnline ? '●' : '⚠';

  return (
    <div className="app-container">
      <header className="header" style={{ display: 'flex', gap: '2rem', alignItems: 'center' }}>
        <h1 
          style={{ cursor: 'pointer', color: 'var(--primary-color)', fontSize: '1.35rem', fontWeight: 800 }} 
          onClick={() => setCurrentTab('scheduler')}
          title="Back to Workforce & Rostering"
        >
          UltraRoster
        </h1>
        <nav style={{ display: 'flex', gap: '1.25rem', alignItems: 'center', flex: 1 }}>
          <button 
            onClick={() => setCurrentTab('scheduler')}
            style={{ 
              background: 'none', 
              border: 'none', 
              color: currentTab === 'scheduler' ? 'var(--primary-color)' : 'var(--text-muted)', 
              fontWeight: currentTab === 'scheduler' ? 700 : 500,
              fontSize: '0.95rem',
              cursor: 'pointer',
              padding: '0.4rem 0',
              borderBottom: currentTab === 'scheduler' ? '2px solid var(--primary-color)' : '2px solid transparent',
              transition: 'all 0.15s',
            }}
          >
            Workforce &amp; Rostering
          </button>
          <button 
            onClick={() => setCurrentTab('constraints')}
            style={{ 
              background: 'none', 
              border: 'none', 
              color: currentTab === 'constraints' ? 'var(--primary-color)' : 'var(--text-muted)', 
              fontWeight: currentTab === 'constraints' ? 700 : 500,
              fontSize: '0.95rem',
              cursor: 'pointer',
              padding: '0.4rem 0',
              borderBottom: currentTab === 'constraints' ? '2px solid var(--primary-color)' : '2px solid transparent',
              transition: 'all 0.15s',
            }}
          >
            Staffing Policies
          </button>
        </nav>

        {/* Backend Status Pill */}
        <div style={{
          display: 'flex',
          alignItems: 'center',
          gap: '0.4rem',
          padding: '0.25rem 0.65rem',
          borderRadius: '999px',
          border: `1px solid ${statusColor}33`,
          backgroundColor: `${statusColor}11`,
          fontSize: '0.78rem',
          fontWeight: 600,
          color: statusColor,
          whiteSpace: 'nowrap',
          flexShrink: 0,
        }}>
          <span style={{ fontSize: '0.6rem' }}>{statusDot}</span>
          <span>{statusLabel}</span>
        </div>
      </header>

      {/* Synthetic Demo Disclaimer Banner */}
      <div className="no-print" style={{
        backgroundColor: 'rgba(15, 118, 110, 0.04)',
        borderBottom: '1px solid var(--border-color)',
        padding: '0.35rem 1.5rem',
        fontSize: '0.78rem',
        color: 'var(--text-muted)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        gap: '0.4rem',
        fontWeight: 500,
      }}>
        <span>ℹ</span>
        <span>Demonstration Environment — Synthetic Hospital Data Only</span>
      </div>
      
      <div className="content">
        <main className="main-view">
          {currentTab === 'scheduler' ? (
            <PlannerWorkflow
              workflowMode={workflowMode}
              workflowTrigger={workflowTrigger}
              onWorkflowModeChange={setWorkflowMode}
            />
          ) : (
            <Constraints />
          )}
        </main>
      </div>
    </div>
  );
}

export default App;
