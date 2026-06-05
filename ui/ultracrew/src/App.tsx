import React, { useEffect, useState, useRef } from 'react';
import { NurseRoster } from './NurseRoster';
import type { Nurse } from './NurseRoster';
import { ScheduleGrid } from './ScheduleGrid';
import { TeamBalance } from './TeamBalance';
import type { NurseBalance } from './TeamBalance';
import { Dashboard } from './Dashboard';
import { Simulator } from './Simulator';
import { Landing } from './Landing';
import { Constraints } from './Constraints';

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
  const [currentTab, setCurrentTab] = useState<'dashboard' | 'constraints'>('dashboard');
  const [nurses, setNurses] = useState<Nurse[]>([]);
  const [schedule, setSchedule] = useState<Record<string, string[]>>({});
  const [authenticSchedule, setAuthenticSchedule] = useState<Record<string, string[]>>({});
  const [balances, setBalances] = useState<NurseBalance[]>([]);
  const [loading, setLoading] = useState(true);
  const [dates, setDates] = useState<Date[]>([]);
  
  const [simulationState, setSimulationState] = useState<SimulationState | null>(null);
  
  const scheduleContainerRef = React.useRef<HTMLDivElement>(null);
  
  const todayIndex = 14; // Mock Today (Wednesday Week 3)

  useEffect(() => {
    fetch('http://127.0.0.1:3000/api/scenario')
      .then(res => res.json())
      .then(data => {
        // Parse nurses from INRC Scenario
        const parsedNurses: Nurse[] = data.nurses.map((n: any) => ({
          id: n.id,
          contract: n.contract,
          skills: n.skills,
        }));
        setNurses(parsedNurses);
        
        const nurses = data.nurses.slice(0, 15);
        setNurses(nurses);

        // Generate 56 days (8 weeks) starting from next Monday
        const startDate = new Date('2026-06-01T00:00:00'); // Mock Monday
        const scheduleDates = Array.from({ length: 56 }).map((_, i) => {
          const d = new Date(startDate);
          d.setDate(d.getDate() + i);
          return d;
        });
        setDates(scheduleDates);

        const mockSchedule: Record<string, string[]> = {};
        nurses.forEach((nurse: Nurse, index: number) => {
          const shiftTypes = ['E', 'L', 'N', ''];
          const pattern = [];
          for (let i = 0; i < 56; i++) {
            if (nurse.id === 'HN_0') {
              const base = ['E', 'E', 'E', '', 'L', '', ''];
              pattern.push(base[i % 7]);
            } else {
              if (i % 7 === (index % 7) || i % 7 === ((index + 1) % 7)) {
                pattern.push('');
              } else {
                pattern.push(shiftTypes[(index + i) % 3]);
              }
            }
          }
          mockSchedule[nurse.id] = pattern;
        });
        setSchedule(mockSchedule);
        setAuthenticSchedule(mockSchedule);
        
        return fetch('http://127.0.0.1:3000/api/balance');
      })
      .then(res => res.json())
      .then(data => {
        setBalances(data);
        setLoading(false);
      })
      .catch(err => {
        console.error("Failed to load scenario:", err);
        setLoading(false);
      });
  }, [showDemo]);

  if (!showDemo) {
    return <Landing onStartDemo={() => setShowDemo(true)} />;
  }

  return (
    <div className="app-container">
      <header className="header" style={{ display: 'flex', gap: '2rem' }}>
        <h1 style={{ cursor: 'pointer' }} onClick={() => setCurrentTab('dashboard')}>UltraCrew</h1>
        <nav style={{ display: 'flex', gap: '1rem', alignItems: 'center' }}>
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
        </nav>
      </header>
      
      <div className="content">
        <aside className="sidebar">
          {loading ? <p>Loading Roster...</p> : <NurseRoster nurses={nurses} />}
        </aside>
        
        <main className="main-view">
          {currentTab === 'constraints' ? (
            <Constraints />
          ) : (
            loading ? <p>Loading Schedule...</p> : (
              <div style={{ display: 'flex', flexDirection: 'column', gap: '2rem' }}>
                <Dashboard simulationState={simulationState} />
              
              <div style={{ 
                position: 'relative', 
                border: simulationState ? '2px dashed var(--accent-color)' : 'none', 
                borderRadius: '12px',
                padding: simulationState ? '2px' : '0'
              }}>
                {simulationState && (
                  <div style={{ 
                    display: 'flex', 
                    justifyContent: 'space-between', 
                    alignItems: 'center',
                    backgroundColor: 'rgba(56, 189, 248, 0.1)',
                    padding: '0.75rem 1.5rem',
                    borderTopLeftRadius: '10px',
                    borderTopRightRadius: '10px',
                    borderBottom: '1px solid var(--accent-color)',
                    color: 'var(--accent-color)'
                  }}>
                    <span style={{ fontWeight: 600, letterSpacing: '0.05em' }}>⚠️ SIMULATION MODE</span>
                    <button 
                      onClick={() => {
                        setSchedule(authenticSchedule);
                        setSimulationState(null);
                      }}
                      style={{
                        backgroundColor: 'transparent',
                        color: 'var(--accent-color)',
                        border: '1px solid var(--accent-color)',
                        padding: '0.25rem 0.75rem',
                        borderRadius: '4px',
                        cursor: 'pointer',
                        fontSize: '0.85rem'
                      }}
                    >
                      Reset to Authentic
                    </button>
                  </div>
                )}
                <div ref={scheduleContainerRef} style={{ overflowX: 'auto', paddingBottom: '1rem', scrollBehavior: 'smooth' }}>
                  <ScheduleGrid nurses={nurses} schedule={schedule} dates={dates} todayIndex={todayIndex} />
                </div>
              </div>

              <TeamBalance 
                balances={balances} 
                simulationState={simulationState} 
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
                          left: (todayIndex + 7) * 80, // Scroll past Week 1
                          behavior: 'smooth'
                        });
                      }
                    }, 300);
                  }
                }}
                onMarkSick={(nurseId, sickDays) => {
                  const newSchedule = JSON.parse(JSON.stringify(authenticSchedule));
                  const sickShifts = newSchedule[nurseId];
                  const sickNurse = nurses.find(n => n.id === nurseId);
                  
                  let newCountTotal = 0;
                  let missedTotal = 0;
                  
                  // Track who covered how many shifts
                  const creditors: Record<string, number> = {};
                  
                  sickShifts.forEach((shift: string, dayIndex: number) => {
                    // Rule 1 & 2: Immutable past. Sickness can only apply >= todayIndex
                    if (shift && sickNurse && sickDays.has(dayIndex) && dayIndex >= todayIndex) {
                      newSchedule[nurseId][dayIndex] = `SICK-${shift}`;
                      missedTotal++;
                      // 1. Priority: Exact designation (Primary Skill)
                      let pool = nurses.filter(n => 
                        n.id !== nurseId && 
                        !newSchedule[n.id][dayIndex] &&
                        n.skills[0] === sickNurse.skills[0]
                      );

                      // 2. Fallback: Any overlapping skill
                      if (pool.length === 0) {
                        pool = nurses.filter(n => 
                          n.id !== nurseId && 
                          !newSchedule[n.id][dayIndex] &&
                          n.skills.some(skill => sickNurse.skills.includes(skill))
                        );
                      }

                      if (pool.length > 0) {
                        const replacement = pool[Math.floor(Math.random() * pool.length)];
                        newSchedule[replacement.id][dayIndex] = `NEW-${shift}`;
                        newCountTotal++;
                        creditors[replacement.id] = (creditors[replacement.id] || 0) + 1;
                      }
                    }
                  });

                  // --- WORKLOAD RECOVERY LOGIC (Weeks 2-8) ---
                  let maxRecoveryDay = todayIndex;
                  
                  // For each shift owed to a creditor, try to swap a future shift
                  Object.entries(creditors).forEach(([creditorId, shiftsOwed]) => {
                    let remainingToShed = shiftsOwed;
                    // Rule 3: Recovery can only use future available shifts (after sickness week)
                    const recoveryStart = todayIndex + 7;
                    for (let dayIndex = recoveryStart; dayIndex < 56 && remainingToShed > 0; dayIndex++) {
                      // If the creditor is working, and the sick nurse is off
                      const creditorShift = newSchedule[creditorId][dayIndex];
                      const sickNurseShift = newSchedule[nurseId][dayIndex];
                      
                      // Check if it's a regular shift (not already swapped/empty)
                      if (creditorShift && !creditorShift.includes('-') && !sickNurseShift) {
                        // Swap them!
                        newSchedule[nurseId][dayIndex] = `RECOVERED-${creditorShift}`;
                        newSchedule[creditorId][dayIndex] = `RETURNED-${creditorShift}`;
                        remainingToShed--;
                        if (dayIndex > maxRecoveryDay) {
                          maxRecoveryDay = dayIndex;
                        }
                      }
                    }
                  });

                  // Calculate metrics
                  const weeksToRecover = maxRecoveryDay > todayIndex ? Math.ceil((maxRecoveryDay - (todayIndex + 7)) / 7) + 1 : 0;
                  
                  const balance_changes: Record<string, { previous: number, current: number }> = {};
                  balances.forEach(b => {
                    const shifts = newSchedule[b.nurse_id] || [];
                    const sickCount = shifts.filter((s: string) => s.startsWith('SICK-')).length;
                    const recoveredCount = shifts.filter((s: string) => s.startsWith('RECOVERED-')).length;
                    const newCount = shifts.filter((s: string) => s.startsWith('NEW-')).length;
                    const returnedCount = shifts.filter((s: string) => s.startsWith('RETURNED-')).length;
                    
                    const delta = -sickCount + recoveredCount + newCount - returnedCount;
                    if (delta !== 0) {
                      balance_changes[b.nurse_id] = {
                        previous: b.balance,
                        current: b.balance + delta
                      };
                    }
                  });

                  setSchedule(newSchedule);
                  setSimulationState({
                    affected_nurse: nurseId,
                    missed_shifts: missedTotal,
                    recovered_shifts: newCountTotal,
                    recovery_eta: weeksToRecover,
                    creditors,
                    balance_changes,
                    coverage_impact: 'None'
                  });
                }} 
              />
            </div>
            )
          )}
        </main>
      </div>
    </div>
  );
}

export default App;
