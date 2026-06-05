import React from 'react';
import type { Nurse } from './NurseRoster';

interface ScheduleGridProps {
  nurses: Nurse[];
  schedule: Record<string, string[]>;
  dates?: Date[];
  todayIndex?: number;
}

export const ScheduleGrid: React.FC<ScheduleGridProps> = ({ nurses, schedule, dates = [], todayIndex = 14 }) => {
  const [selectedShiftInfo, setSelectedShiftInfo] = React.useState<{nurse: string, type: string, action: string} | null>(null);

  return (
    <div className="card" style={{ position: 'relative' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <h2>Weekly Schedule</h2>
        <div style={{ display: 'flex', gap: '2rem', fontSize: '0.85rem', fontWeight: 600, color: 'var(--text-muted)' }}>
          <span>← Immutable Past</span>
          <span style={{ color: 'var(--accent-color)' }}>Today</span>
          <span>Future Recovery Window →</span>
        </div>
      </div>
      <div className="schedule-grid" style={{ marginTop: '1rem' }}>
        <div className="grid-cell grid-header sticky-nurse" style={{ minWidth: '100px' }}>Nurse</div>
        {dates.map((date, idx) => {
          const isPast = idx < todayIndex;
          const isToday = idx === todayIndex;
          return (
            <div key={idx} className="grid-cell grid-header" style={{ 
              minWidth: '80px', 
              backgroundColor: isPast ? 'var(--panel-bg)' : 'inherit',
              borderLeft: isToday ? '3px solid var(--accent-color)' : 'none',
              opacity: isPast ? 0.7 : 1
            }}>
              <div style={{ fontSize: '0.75rem', color: isToday ? 'var(--accent-color)' : 'var(--text-muted)', fontWeight: isToday ? 'bold' : 'normal' }}>
                 {isToday ? 'TODAY' : isPast ? 'PAST' : date.toLocaleDateString('en-US', { weekday: 'short' })}
              </div>
              <div>
                 {date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' })}
              </div>
            </div>
          );
        })}
        
        {nurses.map(nurse => {
          const assignments = schedule[nurse.id] || Array(dates.length || 7).fill('');
          return (
            <React.Fragment key={nurse.id}>
              <div className="grid-cell nurse-row-header sticky-nurse" style={{ minWidth: '100px' }}>{nurse.id}</div>
              {assignments.map((rawShift, idx) => {
                let isSick = false;
                let isNew = false;
                let isRecovered = false;
                let isReturned = false;
                let shift = rawShift;
                if (shift.startsWith('SICK-')) {
                  isSick = true;
                  shift = shift.replace('SICK-', '');
                } else if (shift.startsWith('NEW-')) {
                  isNew = true;
                  shift = shift.replace('NEW-', '');
                } else if (shift.startsWith('RECOVERED-')) {
                  isRecovered = true;
                  shift = shift.replace('RECOVERED-', '');
                } else if (shift.startsWith('RETURNED-')) {
                  isReturned = true;
                  shift = shift.replace('RETURNED-', '');
                }

                const isPast = idx < todayIndex;
                const isToday = idx === todayIndex;

                let shiftClass = '';
                let shiftName = '';
                if (shift.startsWith('E')) {
                  shiftClass = 'early';
                  shiftName = 'Early';
                } else if (shift.startsWith('L')) {
                  shiftClass = 'late';
                  shiftName = 'Late';
                } else if (shift.startsWith('N')) {
                  shiftClass = 'night';
                  shiftName = 'Night';
                }

                return (
                  <div key={idx} className="grid-cell" style={{ 
                    minWidth: '80px',
                    backgroundColor: isPast ? 'var(--panel-bg)' : 'inherit',
                    borderLeft: isToday ? '3px solid var(--accent-color)' : 'none'
                  }}>
                    {shift && (
                      <span 
                        className={`shift-chip ${shiftClass}`}
                        style={{
                          ...(isPast ? { filter: 'grayscale(100%)', opacity: 0.5, cursor: 'not-allowed' } : {}),
                          ...(isSick ? { 
                            opacity: 0.5, 
                            textDecoration: 'line-through', 
                            backgroundColor: 'var(--panel-bg)', 
                            color: 'var(--text-muted)',
                            border: '1px dashed var(--text-muted)' 
                          } : isNew ? {
                            boxShadow: '0 0 10px rgba(52, 211, 153, 0.6)',
                            border: '2px solid var(--success-color)'
                          } : isRecovered ? {
                            backgroundColor: 'rgba(37, 99, 235, 0.15)',
                            border: '2px solid var(--accent-color)',
                            color: 'var(--text-main)',
                            fontWeight: 600
                          } : isReturned ? {
                            opacity: 0.6,
                            backgroundColor: 'transparent',
                            border: '1px dashed var(--success-color)',
                            color: 'var(--success-color)'
                          } : {})
                        }}
                        title={isSick ? 'Shift missed due to sick leave' : isNew ? 'Shift dynamically reassigned for coverage' : isRecovered ? 'Assigned to restore workload balance after earlier absence.' : isReturned ? 'Removed because this employee covered additional work previously.' : ''}
                        onClick={() => {
                          if (isRecovered) {
                            setSelectedShiftInfo({ nurse: nurse.id, type: 'Recovered', action: `Assigned ${shiftName || shift} shift to restore workload balance.` });
                          } else if (isReturned) {
                            setSelectedShiftInfo({ nurse: nurse.id, type: 'Time Returned', action: `Removed ${shiftName || shift} shift because this employee covered additional work previously.` });
                          }
                        }}
                        style={{
                          ...(isPast ? { filter: 'grayscale(100%)', opacity: 0.5, cursor: 'not-allowed' } : {}),
                          ...(isSick ? { 
                            opacity: 0.5, 
                            textDecoration: 'line-through', 
                            backgroundColor: 'var(--panel-bg)', 
                            color: 'var(--text-muted)',
                            border: '1px dashed var(--text-muted)' 
                          } : isNew ? {
                            boxShadow: '0 0 10px rgba(52, 211, 153, 0.6)',
                            border: '2px solid var(--success-color)'
                          } : isRecovered ? {
                            backgroundColor: 'rgba(37, 99, 235, 0.15)',
                            border: '2px solid var(--accent-color)',
                            color: 'var(--text-main)',
                            fontWeight: 600,
                            cursor: 'pointer'
                          } : isReturned ? {
                            opacity: 0.6,
                            backgroundColor: 'transparent',
                            border: '1px dashed var(--success-color)',
                            color: 'var(--success-color)',
                            cursor: 'pointer'
                          } : {})
                        }}
                      >
                        {isRecovered ? 'Recovered' : isReturned ? 'Time Returned' : (shiftName || shift)}
                      </span>
                    )}
                  </div>
                );
              })}
            </React.Fragment>
          );
        })}
      </div>

      {selectedShiftInfo && (
        <div style={{
          position: 'fixed',
          top: '50%',
          left: '50%',
          transform: 'translate(-50%, -50%)',
          backgroundColor: 'var(--bg-color)',
          padding: '2rem',
          borderRadius: '12px',
          boxShadow: '0 25px 50px -12px rgba(0, 0, 0, 0.5)',
          border: '1px solid var(--border-color)',
          zIndex: 1100,
          width: '400px'
        }}>
          <button 
            onClick={() => setSelectedShiftInfo(null)}
            style={{ position: 'absolute', top: '1rem', right: '1rem', background: 'none', border: 'none', color: 'var(--text-muted)', cursor: 'pointer', fontSize: '1.2rem' }}
          >
            ×
          </button>
          <h3 style={{ marginTop: 0, color: 'var(--text-main)' }}>{selectedShiftInfo.type} Shift</h3>
          
          <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem', marginTop: '1.5rem' }}>
            <div>
              <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Employee</div>
              <div style={{ fontWeight: 600, color: 'var(--text-main)' }}>{selectedShiftInfo.nurse}</div>
            </div>
            
            <div>
              <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Reason</div>
              <div style={{ color: 'var(--text-main)' }}>
                {selectedShiftInfo.type === 'Recovered' 
                  ? 'Missed shift earlier due to sickness.' 
                  : 'Covered additional work earlier.'}
              </div>
            </div>
            
            <div>
              <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Action</div>
              <div style={{ color: 'var(--text-main)' }}>{selectedShiftInfo.action}</div>
            </div>
            
            <div>
              <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Coverage Impact</div>
              <div style={{ color: 'var(--success-color)', fontWeight: 600 }}>None. Coverage remained fully staffed.</div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
