import React, { useState } from 'react';
import type { StaffMember, ScheduleResult } from './WorkflowTypes';
import { SHIFT_COLORS } from './WorkflowTypes';
import { primaryBtnStyle, ghostBtnStyle } from './WorkflowComponents';

const SHIFT_OPTIONS = ['Early', 'Late', 'Night', ''];

// ─── Shift Picker Popover ─────────────────────────────────────────────────────

const ShiftPicker: React.FC<{
  current: string;
  onPick: (shift: string) => void;
  onClose: () => void;
}> = ({ current, onPick, onClose }) => (
  <div
    style={{
      position: 'absolute',
      zIndex: 300,
      top: '100%',
      left: '50%',
      transform: 'translateX(-50%)',
      backgroundColor: 'var(--bg-color)',
      border: '1px solid var(--border-color)',
      borderRadius: '8px',
      padding: '0.4rem',
      boxShadow: '0 8px 24px rgba(0,0,0,0.5)',
      display: 'flex',
      flexDirection: 'column',
      gap: '0.2rem',
      minWidth: '120px',
    }}
    onClick={e => e.stopPropagation()}
  >
    {SHIFT_OPTIONS.map(opt => (
      <button
        key={opt || 'off'}
        onClick={() => { onPick(opt); onClose(); }}
        style={{
          background: current === opt ? 'rgba(56,189,248,0.12)' : 'none',
          border: `1px solid ${current === opt ? 'var(--accent-color)' : 'transparent'}`,
          color: opt ? SHIFT_COLORS[opt] : 'var(--text-muted)',
          padding: '0.35rem 0.75rem',
          borderRadius: '4px',
          cursor: 'pointer',
          fontWeight: current === opt ? 700 : 400,
          fontSize: '0.82rem',
          textAlign: 'left',
        }}
      >
        {opt || 'Off (rest)'}
      </button>
    ))}
  </div>
);

// ─── Explain Modal ────────────────────────────────────────────────────────────

const ExplainModal: React.FC<{
  nurseId: string;
  dayIdx: number;
  shift: string;
  staff: StaffMember[];
  result: ScheduleResult;
  onClose: () => void;
}> = ({ nurseId, dayIdx, shift, staff, result, onClose }) => {
  const member = staff.find(s => s.id === nurseId);
  const startDate = new Date('2026-07-14');
  const dt = new Date(startDate);
  dt.setDate(dt.getDate() + dayIdx);
  const dateStr = dt.toLocaleDateString('en-US', { weekday: 'long', month: 'short', day: 'numeric' });
  const isWeekend = dt.getDay() === 0 || dt.getDay() === 6;

  const reasons: string[] = [];
  if (shift) {
    reasons.push(`${shift} shift assigned to meet coverage requirements for ${dateStr}.`);
    if (member) {
      reasons.push(`${member.id} holds a "${member.contract}" contract — eligible for this shift type.`);
      reasons.push(`Skill match: ${member.skills.join(', ')}.`);
    }
    reasons.push(isWeekend
      ? 'Weekend assignment — counted against the maximum working weekends limit.'
      : 'Weekday assignment — does not consume weekend allocation.');
    const cr = result.constraint_report;
    if (cr) {
      reasons.push(cr.hard_violations === 0
        ? 'Schedule has zero hard constraint violations at time of generation.'
        : `Note: schedule has ${cr.hard_violations} hard violation(s) — consider re-generating.`);
    }
  } else {
    reasons.push(`Rest day on ${dateStr}.`);
    if (member) reasons.push(`Ensures minimum consecutive days off for ${member.id}.`);
    reasons.push('Prevents fatigue accumulation and maintains legal rest requirements.');
    if (isWeekend) reasons.push('Weekend rest — does not consume weekend allocation.');
  }

  const cr = result.constraint_report;

  return (
    <div
      style={{ position: 'fixed', top: 0, left: 0, right: 0, bottom: 0, backgroundColor: 'rgba(15,23,42,0.75)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 1000 }}
      onClick={onClose}
    >
      <div
        style={{ backgroundColor: 'var(--bg-color)', border: '1px solid var(--border-color)', borderRadius: '12px', padding: '2rem', maxWidth: '480px', width: '90%', boxShadow: '0 25px 50px rgba(0,0,0,0.5)' }}
        onClick={e => e.stopPropagation()}
      >
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '1.25rem' }}>
          <div>
            <div style={{ fontWeight: 700, fontSize: '1.1rem', color: shift ? SHIFT_COLORS[shift] : 'var(--text-muted)' }}>
              {shift ? `${shift} Shift` : 'Rest Day'}
            </div>
            <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)', marginTop: '0.2rem' }}>{nurseId} · {dateStr}</div>
          </div>
          <button onClick={onClose} style={{ background: 'none', border: 'none', color: 'var(--text-muted)', cursor: 'pointer', fontSize: '1.4rem', lineHeight: 1, padding: 0 }}>×</button>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem', marginBottom: '1.25rem' }}>
          <div style={{ fontSize: '0.78rem', fontWeight: 700, color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.06em' }}>Why this assignment?</div>
          {reasons.map((r, i) => (
            <div key={i} style={{ display: 'flex', gap: '0.6rem', fontSize: '0.88rem', color: 'var(--text-main)' }}>
              <span style={{ color: 'var(--accent-color)', flexShrink: 0 }}>→</span>
              <span>{r}</span>
            </div>
          ))}
        </div>

        {cr && (
          <div style={{ paddingTop: '1rem', borderTop: '1px solid var(--border-color)' }}>
            <div style={{ fontSize: '0.78rem', fontWeight: 700, color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.06em', marginBottom: '0.6rem' }}>Schedule health</div>
            <div style={{ display: 'flex', gap: '1.5rem', fontSize: '0.85rem' }}>
              <div>
                <span style={{ color: cr.hard_violations === 0 ? 'var(--success-color)' : 'var(--danger-color)', fontWeight: 700 }}>{cr.hard_violations}</span>
                <span style={{ color: 'var(--text-muted)' }}> hard violations</span>
              </div>
              <div>
                <span style={{ color: '#f59e0b', fontWeight: 700 }}>{cr.soft_violations}</span>
                <span style={{ color: 'var(--text-muted)' }}> soft violations</span>
              </div>
              <div>
                <span style={{ color: cr.is_valid ? 'var(--success-color)' : 'var(--danger-color)', fontWeight: 700 }}>{cr.is_valid ? '✓ Valid' : '✗ Invalid'}</span>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

// ─── Main ReviewSchedule component ───────────────────────────────────────────

export const ReviewSchedule: React.FC<{
  staff: StaffMember[];
  schedule: Record<string, string[]>;
  result: ScheduleResult;
  onScheduleChange: (s: Record<string, string[]>) => void;
  onNext: (editCount: number, editDistribution: Record<string, number>) => void;
  onBack: () => void;
}> = ({ staff, schedule, result, onScheduleChange, onNext, onBack }) => {
  const [activeCell, setActiveCell] = useState<{ nurseId: string; dayIdx: number } | null>(null);
  const [explainTarget, setExplainTarget] = useState<{ nurseId: string; dayIdx: number; shift: string } | null>(null);
  const [editCount, setEditCount] = useState(0);
  const [editDist, setEditDist] = useState<Record<string, number>>({
    shift_swap: 0,       // changed from one shift type to another
    coverage_fix: 0,     // changed from empty to a shift (filling a gap)
    removal: 0,          // changed from a shift to empty (removing assignment)
    weekend_change: 0,   // any edit on a weekend day (Sat=5, Sun=6 mod 7)
  });

  const startDate = new Date('2026-07-14');
  const days = Array.from({ length: 28 }, (_, i) => {
    const dt = new Date(startDate);
    dt.setDate(dt.getDate() + i);
    return dt;
  });

  const handleShiftPick = (nurseId: string, dayIdx: number, shift: string) => {
    const prev = (schedule[nurseId] || [])[dayIdx] ?? '';
    const next = { ...schedule, [nurseId]: [...(schedule[nurseId] || [])] };
    next[nurseId][dayIdx] = shift;
    onScheduleChange(next);
    setEditCount(c => c + 1);

    // Classify the edit
    setEditDist(d => {
      const updated = { ...d };
      const dayOfWeek = days[dayIdx]?.getDay() ?? 0; // 0=Sun, 6=Sat
      const isWeekend = dayOfWeek === 0 || dayOfWeek === 6;
      if (isWeekend) updated.weekend_change = (updated.weekend_change ?? 0) + 1;
      if (prev === '' && shift !== '') updated.coverage_fix = (updated.coverage_fix ?? 0) + 1;
      else if (prev !== '' && shift === '') updated.removal = (updated.removal ?? 0) + 1;
      else if (prev !== '' && shift !== '' && prev !== shift) updated.shift_swap = (updated.shift_swap ?? 0) + 1;
      return updated;
    });

    setActiveCell(null);
  };

  const cr = result.constraint_report;

  const stickyTh: React.CSSProperties = {
    padding: '0.4rem 0.5rem',
    textAlign: 'center',
    color: 'var(--text-muted)',
    fontWeight: 600,
    fontSize: '0.72rem',
    whiteSpace: 'nowrap',
    borderBottom: '2px solid var(--border-color)',
    backgroundColor: 'var(--panel-bg)',
    position: 'sticky',
    top: 0,
    zIndex: 10,
  };

  const stickyName: React.CSSProperties = {
    padding: '0.4rem 0.75rem',
    fontWeight: 600,
    fontSize: '0.85rem',
    color: 'var(--text-main)',
    whiteSpace: 'nowrap',
    borderRight: '1px solid var(--border-color)',
    position: 'sticky',
    left: 0,
    backgroundColor: 'var(--panel-bg)',
    zIndex: 5,
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', flexWrap: 'wrap', gap: '1rem' }}>
        <div>
          <h3 style={{ margin: '0 0 0.5rem 0', color: 'var(--text-main)' }}>Review & Edit Schedule</h3>
          <p style={{ margin: 0, color: 'var(--text-muted)', fontSize: '0.9rem' }}>
            Click any cell to change a shift. Click <strong>?</strong> to understand why it was assigned.
          </p>
        </div>
        {editCount > 0 && (
          <span style={{ color: '#f59e0b', fontSize: '0.85rem', fontWeight: 600 }}>
            ✏ {editCount} manual edit{editCount !== 1 ? 's' : ''}
          </span>
        )}
      </div>

      {cr && (
        <div style={{ display: 'flex', gap: '0.75rem', flexWrap: 'wrap' }}>
          <div style={{
            backgroundColor: cr.hard_violations === 0 ? 'rgba(34,197,94,0.06)' : 'rgba(239,68,68,0.06)',
            border: `1px solid ${cr.hard_violations === 0 ? 'rgba(34,197,94,0.3)' : 'var(--danger-color)'}`,
            borderRadius: '6px', padding: '0.5rem 0.9rem', fontSize: '0.85rem',
          }}>
            <span style={{ fontWeight: 700, color: cr.hard_violations === 0 ? 'var(--success-color)' : 'var(--danger-color)' }}>
              {cr.hard_violations === 0 ? '✓ Valid — no hard violations' : `✗ ${cr.hard_violations} hard violation(s)`}
            </span>
          </div>
          <div style={{ backgroundColor: 'rgba(245,158,11,0.06)', border: '1px solid rgba(245,158,11,0.3)', borderRadius: '6px', padding: '0.5rem 0.9rem', fontSize: '0.85rem', color: '#f59e0b' }}>
            {cr.soft_violations} soft violation{cr.soft_violations !== 1 ? 's' : ''}
          </div>
          {cr.warnings.map((w, i) => (
            <div key={i} style={{ backgroundColor: 'rgba(245,158,11,0.06)', border: '1px solid rgba(245,158,11,0.3)', borderRadius: '6px', padding: '0.5rem 0.9rem', fontSize: '0.82rem', color: '#f59e0b' }}>⚠ {w}</div>
          ))}
        </div>
      )}

      <div style={{ display: 'flex', gap: '1rem', flexWrap: 'wrap', fontSize: '0.8rem' }}>
        {[['Early', '#38bdf8'], ['Late', '#f59e0b'], ['Night', '#818cf8'], ['Off', 'var(--text-muted)']].map(([label, color]) => (
          <div key={label} style={{ display: 'flex', alignItems: 'center', gap: '0.35rem' }}>
            <div style={{ width: '10px', height: '10px', borderRadius: '2px', backgroundColor: color }} />
            <span style={{ color: 'var(--text-muted)' }}>{label}</span>
          </div>
        ))}
        <div style={{ color: 'var(--text-muted)', marginLeft: '0.5rem' }}>· Click cell to edit · Click ? to explain</div>
      </div>

      <div
        style={{ overflowX: 'auto', overflowY: 'auto', maxHeight: '380px', border: '1px solid var(--border-color)', borderRadius: '8px' }}
        onClick={() => setActiveCell(null)}
      >
        <table style={{ borderCollapse: 'collapse', fontSize: '0.78rem', minWidth: 'max-content' }}>
          <thead>
            <tr>
              <th style={{ ...stickyTh, textAlign: 'left', minWidth: '90px', left: 0, zIndex: 15 }}>Staff</th>
              {days.map((dt, i) => {
                const isWeekend = dt.getDay() === 0 || dt.getDay() === 6;
                return (
                  <th key={i} style={{ ...stickyTh, minWidth: '52px', backgroundColor: isWeekend ? 'rgba(129,140,248,0.06)' : 'var(--panel-bg)', color: isWeekend ? '#818cf8' : 'var(--text-muted)' }}>
                    <div>{dt.toLocaleDateString('en-US', { weekday: 'short' }).toUpperCase()}</div>
                    <div style={{ fontWeight: 400, fontSize: '0.68rem' }}>{dt.getDate()}/{dt.getMonth() + 1}</div>
                  </th>
                );
              })}
            </tr>
          </thead>
          <tbody>
            {staff.map(s => {
              const shifts = schedule[s.id] || Array(28).fill('');
              return (
                <tr key={s.id} style={{ borderBottom: '1px solid var(--border-color)' }}>
                  <td style={stickyName}>{s.id}</td>
                  {shifts.map((shift, dayIdx) => {
                    const isActive = activeCell?.nurseId === s.id && activeCell?.dayIdx === dayIdx;
                    const isWeekend = days[dayIdx].getDay() === 0 || days[dayIdx].getDay() === 6;
                    const color = SHIFT_COLORS[shift] || 'var(--text-muted)';
                    return (
                      <td
                        key={dayIdx}
                        style={{
                          padding: '0.3rem 0.25rem',
                          textAlign: 'center',
                          position: 'relative',
                          backgroundColor: isActive ? 'rgba(56,189,248,0.12)' : isWeekend ? 'rgba(129,140,248,0.04)' : 'transparent',
                          cursor: 'pointer',
                          transition: 'background-color 0.1s',
                        }}
                        onClick={e => { e.stopPropagation(); setActiveCell(prev => prev?.nurseId === s.id && prev?.dayIdx === dayIdx ? null : { nurseId: s.id, dayIdx }); }}
                      >
                        <div style={{
                          display: 'inline-flex', alignItems: 'center', gap: '2px',
                          backgroundColor: shift ? `${color}18` : 'transparent',
                          border: `1px solid ${shift ? color : 'transparent'}`,
                          borderRadius: '4px', padding: '0.15rem 0.3rem',
                          color, fontWeight: shift ? 700 : 400, fontSize: '0.75rem',
                          minWidth: '28px', justifyContent: 'center',
                        }}>
                          {shift ? shift.charAt(0) : '·'}
                        </div>
                        <button
                          onClick={e => { e.stopPropagation(); setActiveCell(null); setExplainTarget({ nurseId: s.id, dayIdx, shift }); }}
                          style={{ position: 'absolute', top: '1px', right: '1px', background: 'none', border: 'none', color: 'var(--text-muted)', cursor: 'pointer', fontSize: '0.6rem', lineHeight: 1, padding: '1px 2px', opacity: 0.5 }}
                          title="Explain this assignment"
                        >?</button>
                        {isActive && (
                          <ShiftPicker
                            current={shift}
                            onPick={s2 => handleShiftPick(s.id, dayIdx, s2)}
                            onClose={() => setActiveCell(null)}
                          />
                        )}
                      </td>
                    );
                  })}
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      <div style={{ display: 'flex', justifyContent: 'space-between' }}>
        <button onClick={onBack} style={ghostBtnStyle}>← Back</button>
        <button onClick={() => onNext(editCount, editDist)} style={primaryBtnStyle}>Next: Export →</button>
      </div>

      {explainTarget && (
        <ExplainModal
          nurseId={explainTarget.nurseId}
          dayIdx={explainTarget.dayIdx}
          shift={explainTarget.shift}
          staff={staff}
          result={result}
          onClose={() => setExplainTarget(null)}
        />
      )}
    </div>
  );
};