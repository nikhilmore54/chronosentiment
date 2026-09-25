import React from 'react';
import './ScheduleGrid.css';
import type { Nurse } from './NurseRoster';

interface ScheduleGridProps {
  nurses: Nurse[];
  schedule: Record<string, string[]>;
  dates?: Date[];
  todayIndex?: number;
  activeHighlightCell?: { nurseId: string; dayIndex: number } | null;
  changedAssignments?: Record<string, boolean[]>;
}

export const ScheduleGrid: React.FC<ScheduleGridProps> = ({ 
  nurses, 
  schedule, 
  dates = [], 
  todayIndex = 14,
  activeHighlightCell = null,
  changedAssignments = {}
}) => {
  const [selectedShiftInfo, setSelectedShiftInfo] = React.useState<{nurse: string, type: string, action: string} | null>(null);
  const wrapperRef = React.useRef<HTMLDivElement>(null);

  // Scroll horizontally to focus on the current week column on mount
  React.useEffect(() => {
    if (wrapperRef.current) {
      const colWidth = 80; // width defined in CSS gridTemplateColumns
      const offset = 100 + todayIndex * colWidth; // 100px for the fixed nurse column
      // Center the current week column in view if possible
      const scrollPos = Math.max(0, offset - wrapperRef.current.clientWidth / 2 + colWidth / 2);
      wrapperRef.current.scrollLeft = scrollPos;
    }
  }, [todayIndex, dates.length]);

  return (
    <div className="card" style={{ position: 'relative' }}>
      <div className="schedule-grid-outer" style={{ overflowY: 'auto', maxHeight: '400px' }}>
        <div className="schedule-grid-wrapper" ref={wrapperRef} style={{ overflowX: 'auto' }}>
          <div className="schedule-grid" style={{ 
            marginTop: '1rem',
            gridTemplateColumns: `100px repeat(${dates.length}, 80px)`
          }}>
        <div className="grid-cell grid-header sticky-nurse" style={{ minWidth: '100px' }}>Nurse</div>
        {dates.map((date, idx) => {
          const isPast = idx < todayIndex;
          const isToday = idx === todayIndex;
          return (
            <div key={idx} className={`grid-cell grid-header ${isToday ? 'today-highlight' : ''}`} style={{ 
              minWidth: '80px', 
              backgroundColor: isPast ? 'var(--panel-bg)' : 'inherit',
              opacity: isPast ? 0.7 : 1
            }}>
              <div style={{ fontSize: '0.75rem', color: isToday ? 'var(--accent-color)' : 'var(--text-muted)', fontWeight: isToday ? 'bold' : 'normal' }}>
                 {date.toLocaleDateString('en-US', { weekday: 'short' }).toUpperCase()}
              </div>
              <div style={{ fontSize: '0.8rem', marginTop: '2px', fontWeight: isToday ? 'bold' : 'normal', color: isToday ? 'var(--accent-color)' : 'inherit' }}>
                 {date.getDate()} {date.toLocaleDateString('en-US', { month: 'short' })}
              </div>
            </div>
          );
        })}
        
        {nurses.map(nurse => {
          const assignments = schedule[nurse.id] || Array(dates.length || 7).fill('');
          const changedRow = changedAssignments[nurse.id] || [];
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
                const isHighlighted = activeHighlightCell?.nurseId === nurse.id && activeHighlightCell?.dayIndex === idx;
                const isChanged = changedRow[idx] || false;

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

                let specialClass = '';
                if (isSick) specialClass = 'sick-chip';
                else if (isNew) specialClass = 'new-chip';
                else if (isRecovered) specialClass = 'recovered-chip';
                else if (isReturned) specialClass = 'returned-chip';

                return (
                  <div key={idx} className={`grid-cell ${isToday ? 'today-highlight' : ''} ${isHighlighted ? 'cell-flash' : ''}`} style={{ 
                    minWidth: '80px',
                    backgroundColor: isPast ? 'var(--panel-bg)' : 'inherit',
                    borderLeft: isToday ? '2px solid var(--accent-color)' : 'none',
                    borderRight: isToday ? '2px solid var(--accent-color)' : 'none',
                    transition: 'all 0.3s ease-out'
                  }}>
                    {shift && (
                      <span 
                        className={`shift-chip ${shiftClass} ${specialClass} ${isChanged ? 'changed-pulse' : ''}`}
                        title={
                          isSick ? 'Unscheduled due to medical leave.' 
                          : isNew ? 'Shift dynamically reassigned to cover sickness.' 
                          : isRecovered ? 'Assigned to restore workload balance after earlier absence.' 
                          : isReturned ? 'Removed because this employee covered additional work previously.' 
                          : 'Standard assigned shift.'
                        }
                        onClick={() => {
                          if (isRecovered) {
                            setSelectedShiftInfo({ nurse: nurse.id, type: 'Recovered', action: `Assigned ${shiftName || shift} shift to restore workload balance.` });
                          } else if (isReturned) {
                            setSelectedShiftInfo({ nurse: nurse.id, type: 'Time Returned', action: `Removed ${shiftName || shift} shift because this employee covered additional work previously.` });
                          }
                        }}
                        style={{
                          ...(isPast ? { filter: 'grayscale(100%)', opacity: 0.5, cursor: 'not-allowed' } : {})
                        }}
                      >
                        {isSick ? 'SICK' 
                         : isNew ? `NEW: ${shift.substring(0, 1)}` 
                         : isRecovered ? `REC` 
                         : isReturned ? `RET` 
                         : (shiftName || shift)}
                      </span>
                    )}
                  </div>
                );
              })}
            </React.Fragment>
          );
        })}
      </div>
        </div>
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
