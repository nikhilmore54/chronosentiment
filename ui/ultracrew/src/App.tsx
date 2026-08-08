import React, { useEffect, useState } from 'react';


import { ScheduleGrid } from './ScheduleGrid';
import { TeamBalance } from './TeamBalance';
import type { NurseBalance } from './TeamBalance';
import { Dashboard } from './Dashboard';
import { Simulator } from './Simulator';
import { Landing } from './Landing';
import { Constraints } from './Constraints';
import { PlannerWorkflow } from './workflow/PlannerWorkflow';
import type { Nurse } from './NurseRoster';
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
  const [currentTab, setCurrentTab] = useState<'dashboard' | 'constraints' | 'scheduler'>('dashboard');
  const [nurses, setNurses] = useState<Nurse[]>([]);
  const [schedule, setSchedule] = useState<Record<string, string[]>>({});
  const [authenticSchedule, setAuthenticSchedule] = useState<Record<string, string[]>>({});
  const [balances, setBalances] = useState<NurseBalance[]>([]);
  const [baselineBalances, setBaselineBalances] = useState<NurseBalance[]>([]);
  const [loading, setLoading] = useState(true);
  const [dates, setDates] = useState<Date[]>([]);
  
  const [simulationState, setSimulationState] = useState<SimulationState | null>(null);
  const [dashboardData, setDashboardData] = useState<any>(null);
  const [baselineDashboardData, setBaselineDashboardData] = useState<any>(null);
  const [demoStep, setDemoStep] = useState<'baseline' | 'optimization' | 'recovery'>('baseline');
  const [loadingProgress, setLoadingProgress] = useState(false);
  const [activeHighlightCell, setActiveHighlightCell] = useState<{ nurseId: string; dayIndex: number } | null>(null);
  const [changedAssignments, setChangedAssignments] = useState<Record<string, boolean[]>>({});

  // Backend connectivity state
  const [backendOnline, setBackendOnline] = useState<boolean | null>(null);
  
  const hasAutoScrolledRef = React.useRef(false);

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

  useEffect(() => {
    if (!loading && scheduleContainerRef.current && !hasAutoScrolledRef.current) {
      const container = scheduleContainerRef.current;
      const columnWidth = 80; // per day column width
      const todayPos = todayIndex * columnWidth + columnWidth / 2;
      const scrollPos = Math.max(0, todayPos - container.clientWidth / 2);
      container.scrollTo({ left: scrollPos, behavior: 'smooth' });
      hasAutoScrolledRef.current = true;
    }
  }, [loading, dates]);

  const scheduleContainerRef = React.useRef<HTMLDivElement>(null);
  
  const todayIndex = 14; // Mock Today (Wednesday Week 3)

  const computeChangedAssignments = (oldSched: Record<string, string[]>, newSched: Record<string, string[]>) => {
    const result: Record<string, boolean[]> = {};
    Object.keys(newSched).forEach(nurseId => {
      const oldRow = oldSched[nurseId] || [];
      const newRow = newSched[nurseId] || [];
      result[nurseId] = newRow.map((shift, idx) => shift !== (oldRow[idx] || ''));
    });
    return result;
  };

  useEffect(() => {
    fetch(`/api/nurses`)
      .then(res => res.json())
      .then(data => {
        const parsedNurses = data.nurses.map((n: any) => ({
          id: n.id,
          contract: n.contract,
          skills: n.skills,
        }));
        setNurses(parsedNurses);
        return fetch(`/api/state`);
      })
      .then(res => res.json())
      .then(stateData => {
        setSchedule(stateData.schedule);
        setAuthenticSchedule(stateData.schedule);
        setBalances(stateData.balances);
        setBaselineBalances(stateData.balances);
        setDashboardData(stateData.dashboard);
        setBaselineDashboardData(stateData.dashboard);
        
        const sched = stateData.schedule as Record<string, string[]>;
        const scheduleLength = Object.values(sched)[0]?.length || 28;
        const startDate = new Date('2026-06-01T00:00:00'); // Mock Monday
        const scheduleDates = Array.from({ length: scheduleLength }).map((_, i) => {
          const d = new Date(startDate);
          d.setDate(d.getDate() + i);
          return d;
        });
        setDates(scheduleDates);
        setLoading(false);
      })
      .catch(err => {
        console.error("Failed to load scenario state:", err);
        setLoading(false);
      });
  }, [showDemo]);

  const handleAlertClick = (employeeId: string, dayIndex: number) => {
    if (scheduleContainerRef.current) {
      scheduleContainerRef.current.scrollTo({
        left: Math.max(0, (dayIndex - 2) * 80),
        behavior: 'smooth'
      });
    }
    const gridEl = document.querySelector('.schedule-grid');
    if (gridEl) {
      gridEl.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }
    setActiveHighlightCell({ nurseId: employeeId, dayIndex });
    setTimeout(() => {
      setActiveHighlightCell(null);
    }, 2000);
  };

  const handleLoadInstance = (instance: string) => {
    setLoadingProgress(true);
    fetch(`/api/load-scenario`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ instance })
    })
      .then(res => {
        if (!res.ok) throw new Error('Failed to load instance');
        return fetch(`/api/nurses`);
      })
      .then(res => res.json())
      .then(data => {
        const parsedNurses = data.nurses.map((n: any) => ({
          id: n.id,
          contract: n.contract,
          skills: n.skills,
        }));
        setNurses(parsedNurses);
        return fetch(`/api/state`);
      })
      .then(res => res.json())
      .then(stateData => {
        setSchedule(stateData.schedule);
        setAuthenticSchedule(stateData.schedule);
        setBalances(stateData.balances);
        setBaselineBalances(stateData.balances);
        setDashboardData(stateData.dashboard);
        setBaselineDashboardData(stateData.dashboard);
        
        const sched = stateData.schedule as Record<string, string[]>;
        const scheduleLength = Object.values(sched)[0]?.length || 28;
        const startDate = new Date('2026-06-01T00:00:00');
        const scheduleDates = Array.from({ length: scheduleLength }).map((_, i) => {
          const d = new Date(startDate);
          d.setDate(d.getDate() + i);
          return d;
        });
        setDates(scheduleDates);
        
        setSimulationState(null);
        setChangedAssignments({});
        setDemoStep('baseline');
        setActiveHighlightCell(null);
        setLoadingProgress(false);
      })
      .catch(err => {
        console.error("Failed to load instance:", err);
        setLoadingProgress(false);
      });
  };

  const handleResetDemo = () => {
    fetch(`/api/simulations/reset`, { method: 'POST' })
      .then(res => res.json())
      .then(stateData => {
        setSchedule(stateData.schedule);
        setAuthenticSchedule(stateData.schedule);
        setBalances(stateData.balances);
        setBaselineBalances(stateData.balances);
        setDashboardData(stateData.dashboard);
        setBaselineDashboardData(stateData.dashboard);
        setSimulationState(null);
        setChangedAssignments({});
        setDemoStep('baseline');
        setActiveHighlightCell(null);
      })
      .catch(err => {
        console.error("Failed to reset server state:", err);
        setSchedule(authenticSchedule);
        setBalances(baselineBalances);
        setDashboardData(baselineDashboardData);
        setSimulationState(null);
        setChangedAssignments({});
        setDemoStep('baseline');
        setActiveHighlightCell(null);
      });
  };

  const handleExportCSV = () => {
    let csvContent = "data:text/csv;charset=utf-8,";
    const dayHeaders = Array.from({ length: dates.length }).map((_, idx) => `Day ${idx + 1}`).join(",");
    csvContent += `Nurse,${dayHeaders}\n`;
    
    Object.entries(schedule).forEach(([nurseId, assignments]) => {
      const row = [nurseId, ...assignments].join(",");
      csvContent += `${row}\n`;
    });
    
    const encodedUri = encodeURI(csvContent);
    const link = document.createElement("a");
    link.setAttribute("href", encodedUri);
    link.setAttribute("download", `ultracrew_roster_export.csv`);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  };

  if (!showDemo) {
    return <Landing onStartDemo={() => setShowDemo(true)} />;
  }

  // Backend status pill colours
  const statusColor = backendOnline === null ? '#94a3b8' : backendOnline ? '#22c55e' : '#ef4444';
  const statusLabel = backendOnline === null ? 'Checking...' : backendOnline ? 'Backend Connected' : 'Backend Offline';
  const statusDot = backendOnline === null ? '●' : backendOnline ? '●' : '⚠';

  return (
    <div className="app-container">
      <header className="header" style={{ display: 'flex', gap: '2rem', alignItems: 'center' }}>
        <h1 style={{ cursor: 'pointer' }} onClick={() => setCurrentTab('dashboard')}>UltraCrew</h1>
        <nav style={{ display: 'flex', gap: '1rem', alignItems: 'center', flex: 1 }}>
          <button 
            onClick={() => setCurrentTab('dashboard')}
            style={{ 
              background: 'none', 
              border: 'none', 
              color: currentTab === 'dashboard' ? 'var(--accent-color)' : 'var(--text-muted)', 
              fontWeight: currentTab === 'dashboard' ? 600 : 400,
              fontSize: '1rem',
              cursor: 'pointer',
              padding: '0.5rem'
            }}
          >
            Dashboard
          </button>
          <button 
            onClick={() => setCurrentTab('constraints')}
            style={{ 
              background: 'none', 
              border: 'none', 
              color: currentTab === 'constraints' ? 'var(--accent-color)' : 'var(--text-muted)', 
              fontWeight: currentTab === 'constraints' ? 600 : 400,
              fontSize: '1rem',
              cursor: 'pointer',
              padding: '0.5rem'
            }}
          >
            Constraints
          </button>
          <button 
            onClick={() => setCurrentTab('scheduler')}
            style={{ 
              background: 'none', 
              border: 'none', 
              color: currentTab === 'scheduler' ? 'var(--accent-color)' : 'var(--text-muted)', 
              fontWeight: currentTab === 'scheduler' ? 600 : 400,
              fontSize: '1rem',
              cursor: 'pointer',
              padding: '0.5rem'
            }}
          >
            Import &amp; Schedule
          </button>
        </nav>

        {/* Backend Status Pill */}
        <div style={{
          display: 'flex',
          alignItems: 'center',
          gap: '0.4rem',
          padding: '0.3rem 0.75rem',
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
          {statusLabel}
          {backendOnline && <span style={{ color: 'var(--text-muted)', fontWeight: 400 }}>&nbsp;:3001</span>}
        </div>
      </header>
      
      <div className="content">
  <main className="main-view">
    {currentTab === 'scheduler' ? (
      <PlannerWorkflow />
    ) : currentTab === 'constraints' ? (
      <Constraints />
    ) : (
      <>
        {/* Dashboard empty/offline state */}
        {!loading && !dashboardData && (
          <div style={{
            margin: '1.5rem',
            padding: '2rem',
            backgroundColor: 'var(--panel-bg)',
            border: '1px solid var(--border-color)',
            borderRadius: '12px',
            textAlign: 'center',
            color: 'var(--text-muted)',
          }}>
            {backendOnline === false ? (
              <>
                <div style={{ fontSize: '1.5rem', marginBottom: '0.75rem' }}>⚠</div>
                <div style={{ fontWeight: 600, color: 'var(--text-main)', marginBottom: '0.5rem' }}>Backend Offline</div>
                <div style={{ fontSize: '0.9rem' }}>
                  Start the UltraCrew server: <code style={{ color: 'var(--accent-color)' }}>cargo run -p ultracrew_server</code>
                </div>
              </>
            ) : (
              <>
                <div style={{ fontSize: '1.5rem', marginBottom: '0.75rem' }}>📋</div>
                <div style={{ fontWeight: 600, color: 'var(--text-main)', marginBottom: '0.5rem' }}>No workforce loaded</div>
                <div style={{ fontSize: '0.9rem' }}>Import staff to see team balance and generate a schedule.</div>
              </>
            )}
          </div>
        )}

        {/* Executive Dashboard */}
        {dashboardData && (
          <Dashboard
            data={dashboardData}
            baselineData={baselineDashboardData}
            simulationState={simulationState}
            demoStep={demoStep}
            onAlertClick={handleAlertClick}
          />
        )}
        {/* Decision Workspace */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: '2rem' }}>
          <TeamBalance
            balances={balances}
            simulationState={simulationState}
            balanceScore={dashboardData?.roster_health?.balance_score ?? 100}
          />
          <Simulator
            nurses={nurses}
            schedule={authenticSchedule}
            simulationState={simulationState}
            todayIndex={todayIndex}
            onJumpToRecovery={() => {
              if (scheduleContainerRef.current) {
                scheduleContainerRef.current.scrollIntoView({ behavior: 'smooth', block: 'center' });
                setTimeout(() => {
                  if (scheduleContainerRef.current) {
                    scheduleContainerRef.current.scrollTo({
                      left: (todayIndex + 7) * 80,
                      behavior: 'smooth'
                    });
                  }
                }, 300);
              }
            }}
            onMarkSick={(nurseId, sickDays) => {
              setLoadingProgress(true);
              setDemoStep('optimization');

              const fetchPromise = fetch(`/api/simulations/sick-leave`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                  employee_id: nurseId,
                  sick_days: Array.from(sickDays)
                })
              }).then(res => {
                if (!res.ok) throw new Error('Simulation API request failed.');
                return res.json();
              });

              const minDelayPromise = new Promise(resolve => setTimeout(resolve, 3000));

              Promise.all([fetchPromise, minDelayPromise])
                .then(([data]) => {
                  const diffs = computeChangedAssignments(authenticSchedule, data.schedule);
                  setSchedule(data.schedule);
                  setBalances(data.balances);
                  setDashboardData(data.dashboard);
                  setSimulationState(data.recovery_plan);
                  setChangedAssignments(diffs);
                  setLoadingProgress(false);
                  setDemoStep('recovery');
                })
                .catch(err => {
                  console.error('Simulation error:', err);
                  setLoadingProgress(false);
                  setDemoStep('baseline');
                });
            }}
          />
        </div>
        {/* Active Workforce Roster */}
        <div
          className="roster-section"
          ref={scheduleContainerRef}
          style={{ overflowX: 'auto', overflowY: 'hidden' }}
        >
          <div style={{
            position: 'relative',
            border: simulationState ? '2px dashed var(--accent-color)' : '1px solid var(--border-color)',
            borderRadius: '12px',
            overflow: 'hidden'
          }}>
            <div style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
              backgroundColor: 'var(--panel-bg)',
              padding: '0.75rem 1.5rem',
              borderBottom: simulationState ? '2px dashed var(--accent-color)' : '1px solid var(--border-color)'
            }}>
              <span style={{ fontWeight: 600, letterSpacing: '0.05em', color: simulationState ? 'var(--accent-color)' : 'var(--text-muted)', fontSize: '0.9rem' }}>
                {simulationState ? '⚠️ SIMULATION ACTIVE: RECOVERY DETECTED' : '📋 ACTIVE WORKFORCE ROSTER'}
              </span>
              <div style={{ display: 'flex', gap: '1rem', alignItems: 'center' }}>
                <select
                  onChange={(e) => handleLoadInstance(e.target.value)}
                  defaultValue="n030w4"
                  style={{
                    backgroundColor: 'var(--panel-bg)',
                    color: 'var(--text-main)',
                    border: '1px solid var(--border-color)',
                    padding: '0.4rem',
                    borderRadius: '6px',
                    fontSize: '0.85rem'
                  }}
                >
                  <option value="n030w4">INRC n030w4 (30 Nurses)</option>
                  <option value="n040w4">INRC n040w4 (40 Nurses)</option>
                  <option value="n060w4">INRC n060w4 (60 Nurses)</option>
                  <option value="n080w4">INRC n080w4 (80 Nurses)</option>
                  <option value="n120w4">INRC n120w4 (120 Nurses)</option>
                </select>
                <button
                  onClick={handleExportCSV}
                  style={{
                    backgroundColor: 'transparent',
                    color: 'var(--accent-color)',
                    border: '1px solid var(--accent-color)',
                    padding: '0.4rem 1rem',
                    borderRadius: '6px',
                    cursor: 'pointer',
                    fontSize: '0.85rem',
                    fontWeight: 600,
                    transition: 'all 0.2s'
                  }}
                  onMouseOver={e => { e.currentTarget.style.backgroundColor = 'rgba(56, 189, 248, 0.1)'; }}
                  onMouseOut={e => { e.currentTarget.style.backgroundColor = 'transparent'; }}
                >
                  Export CSV
                </button>
                <button
                  onClick={handleResetDemo}
                  style={{
                    backgroundColor: simulationState ? 'var(--accent-color)' : 'transparent',
                    color: simulationState ? 'white' : 'var(--text-muted)',
                    border: simulationState ? 'none' : '1px solid var(--border-color)',
                    padding: '0.4rem 1rem',
                    borderRadius: '6px',
                    cursor: 'pointer',
                    fontSize: '0.85rem',
                    fontWeight: 600,
                    transition: 'all 0.2s'
                  }}
                  onMouseOver={e => { if (!simulationState) e.currentTarget.style.backgroundColor = 'rgba(255, 255, 255, 0.05)'; }}
                  onMouseOut={e => { if (!simulationState) e.currentTarget.style.backgroundColor = 'transparent'; }}
                >
                  Reset Demo
                </button>
              </div>
            </div>
            <ScheduleGrid
              nurses={nurses}
              schedule={schedule}
              dates={dates}
              todayIndex={todayIndex}
              activeHighlightCell={activeHighlightCell}
              changedAssignments={changedAssignments}
            />
          </div>
        </div>
      </>
    )}
  </main>
</div>
      <OptimizationLoader active={loadingProgress} />
    </div>
  );
}



function OptimizationLoader({ active }: { active: boolean }) {
  const [stageIndex, setStageIndex] = useState(0);
  const stages = [
    "Analyzing schedule and coverage...",
    "Finding qualified replacements...",
    "Balancing workloads...",
    "Validating union contract constraints...",
    "Finalizing recovery schedule..."
  ];

  useEffect(() => {
    if (!active) {
      setStageIndex(0);
      return;
    }
    const interval = setInterval(() => {
      setStageIndex(prev => (prev < stages.length - 1 ? prev + 1 : prev));
    }, 600);
    return () => clearInterval(interval);
  }, [active]);

  if (!active) return null;

  return (
    <div style={{
      position: 'fixed',
      top: 0,
      left: 0,
      right: 0,
      bottom: 0,
      backgroundColor: 'rgba(15, 23, 42, 0.85)',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      zIndex: 9999,
      backdropFilter: 'blur(4px)'
    }}>
      <div style={{
        backgroundColor: 'var(--panel-bg)',
        border: '1px solid var(--border-color)',
        borderRadius: '16px',
        padding: '3rem',
        maxWidth: '500px',
        width: '90%',
        textAlign: 'center',
        boxShadow: '0 25px 50px -12px rgba(0, 0, 0, 0.5)'
      }}>
        <div style={{
          width: '50px',
          height: '50px',
          border: '4px solid rgba(56, 189, 248, 0.2)',
          borderTopColor: 'var(--accent-color)',
          borderRadius: '50%',
          animation: 'spin 1s linear infinite',
          margin: '0 auto 2rem auto'
        }} />
        <h3 style={{ margin: '0 0 1rem 0', color: 'var(--text-main)', fontSize: '1.25rem' }}>Running MOGA Scheduler</h3>
        <p style={{ margin: 0, color: 'var(--accent-color)', fontWeight: 600, minHeight: '24px' }}>
          {stages[stageIndex]}
        </p>
      </div>
      <style>{`
        @keyframes spin {
          0% { transform: rotate(0deg); }
          100% { transform: rotate(360deg); }
        }
      `}</style>
    </div>
  );
}

export default App;
