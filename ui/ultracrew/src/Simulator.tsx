import { useState } from 'react';
import type { Nurse } from './NurseRoster';
import type { SimulationState } from './App';

interface SimulatorProps {
  nurses: Nurse[];
  schedule: Record<string, string[]>;
  onMarkSick: (nurseId: string, sickDays: Set<number>) => void;
  simulationState?: SimulationState | null;
  todayIndex?: number;
  onJumpToRecovery?: () => void;
}

export const Simulator = ({ 
  nurses, 
  schedule, 
  onMarkSick, 
  simulationState, 
  todayIndex = 14, 
  onJumpToRecovery
}: SimulatorProps) => {
  const [selectedNurse, setSelectedNurse] = useState<string>('HN_0');
  
  // Default select the first 3 days starting from today
  const [sickDays, setSickDays] = useState<Set<number>>(new Set([todayIndex, todayIndex + 1, todayIndex + 2]));
  const [result, setResult] = useState<SimulationResult | null>(null);
  const [loading, setLoading] = useState(false);

  const daysOfWeek = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];

  const handleSimulate = () => {
    setLoading(true);
    // Simulate network delay for UX
    setTimeout(() => {
      onMarkSick(selectedNurse, sickDays);
      setLoading(false);
    }, 400);
  };

  return (
    <div className="card" style={{ display: 'flex', flexDirection: 'column', gap: '2rem' }}>
      {simulationState && (
        <div style={{ backgroundColor: 'rgba(34, 197, 94, 0.1)', border: '1px solid var(--success-color)', borderRadius: '8px', padding: '1.5rem', display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
          <div style={{ color: 'var(--success-color)', fontWeight: 600, display: 'flex', alignItems: 'center', gap: '0.5rem', fontSize: '1.1rem' }}>
            ✓ Recovery Plan Generated
          </div>
          <div style={{ fontSize: '1.05rem', color: 'var(--text-main)', lineHeight: 1.5 }}>
            <span style={{ fontWeight: 600 }}>{simulationState.affected_nurse}</span> missed {simulationState.missed_shifts} shifts due to sickness.<br/><br/>
            Coverage remained intact.<br/><br/>
            Workload balance will be fully restored within <span style={{ fontWeight: 600 }}>{simulationState.recovery_eta} weeks</span> without manager intervention.
          </div>
        </div>
      )}

      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <h2>Sick Leave Simulator</h2>
          <div style={{ color: 'var(--text-muted)' }}>
            A nurse becomes unavailable. What happens next?
          </div>
        </div>
        <div style={{ display: 'flex', gap: '1rem', alignItems: 'center' }}>
          <select 
            value={selectedNurse}
            onChange={(e) => {
              setSelectedNurse(e.target.value);
            }}
            style={{
              padding: '0.75rem',
              borderRadius: '6px',
              backgroundColor: 'var(--bg-color)',
              color: 'var(--text-main)',
              border: '1px solid var(--border-color)',
              fontSize: '1rem'
            }}
          >
            {nurses.map(n => (
              <option key={n.id} value={n.id}>{n.id}</option>
            ))}
          </select>

          <div style={{ display: 'flex', gap: '0.25rem', backgroundColor: 'var(--bg-color)', padding: '0.25rem', borderRadius: '6px', border: '1px solid var(--border-color)' }}>
            {daysOfWeek.map((day, idx) => {
              const actualDayIndex = todayIndex + idx;
              return (
                <button
                  key={day}
                  onClick={() => {
                    const newSet = new Set(sickDays);
                    if (newSet.has(actualDayIndex)) newSet.delete(actualDayIndex);
                    else newSet.add(actualDayIndex);
                    setSickDays(newSet);
                  }}
                  style={{
                    padding: '0.4rem 0.6rem',
                    borderRadius: '4px',
                    border: 'none',
                    backgroundColor: sickDays.has(actualDayIndex) ? 'var(--danger-color)' : 'transparent',
                    color: sickDays.has(actualDayIndex) ? 'white' : 'var(--text-muted)',
                    cursor: 'pointer',
                    fontSize: '0.85rem',
                    fontWeight: sickDays.has(actualDayIndex) ? 600 : 400,
                    transition: 'all 0.2s'
                  }}
                >
                  {day}
                </button>
              );
            })}
          </div>

          <button 
            onClick={handleSimulate} 
            disabled={loading || sickDays.size === 0}
            style={{
              backgroundColor: 'var(--primary-color)',
              color: 'white',
              border: 'none',
              padding: '0.75rem 1.5rem',
              borderRadius: '6px',
              fontWeight: 600,
              cursor: loading ? 'wait' : 'pointer',
              fontSize: '1rem',
            }}
          >
            {loading ? 'Simulating...' : 'Simulate Sick Leave'}
          </button>
        </div>
      </div>

      {simulationState && (
        <div style={{ display: 'flex', gap: '2rem' }}>
          {/* Traditional Scheduler */}
          <div style={{ flex: 1, backgroundColor: 'var(--bg-color)', padding: '1.5rem', borderRadius: '8px', border: '1px solid var(--border-color)' }}>
            <h3 style={{ color: 'var(--text-muted)', fontSize: '0.85rem', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '1.5rem' }}>
              Traditional Scheduler
            </h3>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', borderBottom: '1px solid var(--border-color)', paddingBottom: '0.5rem' }}>
                <span>Missed</span>
                <span style={{ fontWeight: 600, color: 'var(--danger-color)' }}>{simulationState.missed_shifts} shifts</span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between', borderBottom: '1px solid var(--border-color)', paddingBottom: '0.5rem' }}>
                <span>Recovered</span>
                <span style={{ fontWeight: 600 }}>0 shifts</span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between', borderBottom: '1px solid var(--border-color)', paddingBottom: '0.5rem' }}>
                <span>Recovery completed</span>
                <span style={{ fontWeight: 600 }}>Not Scheduled</span>
              </div>
            </div>
          </div>

          {/* Workload Recovery Summary */}
          <div style={{ flex: 1, padding: '1.5rem', backgroundColor: 'var(--bg-color)', borderRadius: '8px', border: '1px solid var(--border-color)' }}>
            <h3 style={{ marginTop: 0, marginBottom: '1.5rem', display: 'flex', alignItems: 'center', gap: '0.5rem', color: 'var(--text-main)', fontSize: '1rem', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
              Workload Recovery Summary
            </h3>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', borderBottom: '1px solid rgba(37, 99, 235, 0.2)', paddingBottom: '0.5rem' }}>
                <span>Missed</span>
                <span style={{ fontWeight: 600, color: 'var(--danger-color)' }}>{simulationState.missed_shifts} shifts</span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between', borderBottom: '1px solid rgba(37, 99, 235, 0.2)', paddingBottom: '0.5rem' }}>
                <span>Recovered</span>
                <span style={{ fontWeight: 600, color: 'var(--success-color)' }}>{simulationState.recovered_shifts} / {simulationState.missed_shifts} shifts</span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between', borderBottom: '1px solid rgba(37, 99, 235, 0.2)', paddingBottom: '0.5rem' }}>
                <span>Recovery time</span>
                <span style={{ fontWeight: 600, color: 'var(--success-color)' }}>{simulationState.recovery_eta > 0 ? `${simulationState.recovery_eta} weeks` : 'Pending'}</span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between', borderBottom: '1px solid rgba(37, 99, 235, 0.2)', paddingBottom: '0.5rem' }}>
                <span>Coverage impact</span>
                <span style={{ fontWeight: 600, color: 'var(--success-color)' }}>None</span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between', borderBottom: '1px solid rgba(37, 99, 235, 0.2)', paddingBottom: '0.5rem' }}>
                <span>Manager intervention required</span>
                <span style={{ fontWeight: 600, color: 'var(--success-color)' }}>0</span>
              </div>
              
              {simulationState.recovery_eta > 0 && onJumpToRecovery && (
                <button 
                  onClick={onJumpToRecovery}
                  style={{ 
                    marginTop: '0.5rem', 
                    padding: '0.5rem', 
                    backgroundColor: 'var(--accent-color)', 
                    color: 'white', 
                    border: 'none', 
                    borderRadius: '4px', 
                    cursor: 'pointer',
                    fontWeight: 600
                  }}
                >
                  Jump to Recovery →
                </button>
              )}
            </div>
            
            <div style={{ marginTop: '2rem', padding: '1rem', backgroundColor: 'rgba(37, 99, 235, 0.05)', borderRadius: '8px', border: '1px solid rgba(37, 99, 235, 0.2)' }}>
              <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
                <div style={{ display: 'flex', gap: '1rem', alignItems: 'center' }}>
                  <div style={{ width: '8px', height: '8px', borderRadius: '50%', backgroundColor: 'var(--danger-color)' }} />
                  <span style={{ width: '60px', fontSize: '0.8rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Week 1</span>
                  <span style={{ fontWeight: 600 }}>Sick Leave</span>
                </div>
                <div style={{ display: 'flex', gap: '1rem', alignItems: 'center' }}>
                  <div style={{ width: '8px', height: '8px', borderRadius: '50%', backgroundColor: 'var(--accent-color)' }} />
                  <span style={{ width: '60px', fontSize: '0.8rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Week 2</span>
                  <span style={{ fontWeight: 600 }}>Recovery Started</span>
                </div>
                <div style={{ display: 'flex', gap: '1rem', alignItems: 'center' }}>
                  <div style={{ width: '8px', height: '8px', borderRadius: '50%', backgroundColor: 'var(--success-color)' }} />
                  <span style={{ width: '60px', fontSize: '0.8rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Week {simulationState.recovery_eta + 1}</span>
                  <span style={{ fontWeight: 600 }}>Recovery Complete</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
