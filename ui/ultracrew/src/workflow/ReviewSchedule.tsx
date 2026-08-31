import React, { useState } from 'react';
import type { StaffMember, ScheduleResult, AssignmentProvenanceState, ChangeRecord, RedistributionLog } from './WorkflowTypes';
import { SHIFT_COLORS } from './WorkflowTypes';
import { primaryBtnStyle, ghostBtnStyle } from './WorkflowComponents';
import { redistributeWithLocks, buildStaffingRequirements, computeCanonicalCoverage } from './WorkflowUtils';
import type { RedistributionResult } from './WorkflowUtils';

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

// ─── Provenance Inspector Modal ──────────────────────────────────────────────
// Shows why a system_reassignment happened, using the emitted ChangeRecord.

const ProvenanceInspector: React.FC<{
  staffId: string;
  dayIdx: number;
  shift: string;
  provenanceState: AssignmentProvenanceState;
  changeRecord: ChangeRecord | null;
  onClose: () => void;
}> = ({ staffId, dayIdx, shift, provenanceState, changeRecord, onClose }) => {
  const startDate = new Date('2026-07-14');
  const dt = new Date(startDate);
  dt.setDate(dt.getDate() + dayIdx);
  const dateStr = dt.toLocaleDateString('en-US', { weekday: 'long', month: 'short', day: 'numeric' });

  const stateLabel: Record<AssignmentProvenanceState, string> = {
    original: 'Original assignment',
    scheduler_edit: '✎ Scheduler edit (locked)',
    system_reassignment: '↻ UltraRoster reassignment',
    unchanged: 'Unchanged by redistribution',
  };
  const stateColor: Record<AssignmentProvenanceState, string> = {
    original: 'var(--text-muted)',
    scheduler_edit: '#f59e0b',
    system_reassignment: 'var(--accent-color)',
    unchanged: 'var(--text-muted)',
  };

  return (
    <div
      style={{ position: 'fixed', top: 0, left: 0, right: 0, bottom: 0, backgroundColor: 'rgba(15,23,42,0.75)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 1000 }}
      onClick={onClose}
    >
      <div
        style={{ backgroundColor: 'var(--bg-color)', border: '1px solid var(--border-color)', borderRadius: '12px', padding: '2rem', maxWidth: '480px', width: '92%', boxShadow: '0 25px 50px rgba(0,0,0,0.5)', maxHeight: '80vh', overflowY: 'auto' }}
        onClick={e => e.stopPropagation()}
      >
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '1.25rem' }}>
          <div>
            <div style={{ fontWeight: 700, fontSize: '1.05rem', color: shift ? SHIFT_COLORS[shift] : 'var(--text-muted)' }}>
              {shift ? `${shift} Shift` : 'Rest Day'}
            </div>
            <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)', marginTop: '0.2rem' }}>{staffId} · {dateStr}</div>
          </div>
          <button onClick={onClose} style={{ background: 'none', border: 'none', color: 'var(--text-muted)', cursor: 'pointer', fontSize: '1.4rem', lineHeight: 1, padding: 0 }}>×</button>
        </div>

        <div style={{ marginBottom: '1rem' }}>
          <div style={{ fontSize: '0.78rem', fontWeight: 700, color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.06em', marginBottom: '0.4rem' }}>Provenance</div>
          <div style={{ fontSize: '0.92rem', fontWeight: 600, color: stateColor[provenanceState] }}>{stateLabel[provenanceState]}</div>
        </div>

        {provenanceState === 'system_reassignment' && changeRecord && (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', paddingTop: '0.75rem', borderTop: '1px solid var(--border-color)' }}>
            <div style={{ fontSize: '0.78rem', fontWeight: 700, color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.06em', marginBottom: '0.2rem' }}>Change record</div>
            <div style={{ fontSize: '0.85rem', color: 'var(--text-main)' }}>
              <strong>From:</strong> {changeRecord.previousValue || 'Off'} → <strong>To:</strong> {changeRecord.newValue || 'Off'}
            </div>
            <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>
              <strong>Reason:</strong> {changeRecord.reason}
            </div>
            <div style={{ fontSize: '0.78rem', color: 'var(--text-muted)', fontStyle: 'italic' }}>
              Operation: {changeRecord.redistributionOperationId}
            </div>
            <div style={{ fontSize: '0.78rem', color: 'var(--text-muted)', fontStyle: 'italic' }}>
              {new Date(changeRecord.timestamp).toLocaleString()}
            </div>
          </div>
        )}

        {provenanceState === 'scheduler_edit' && (
          <div style={{ paddingTop: '0.75rem', borderTop: '1px solid var(--border-color)', fontSize: '0.85rem', color: '#f59e0b' }}>
            This cell was set by you and is locked. UltraRoster will not change it during redistribution.
          </div>
        )}

        {(provenanceState === 'original' || provenanceState === 'unchanged') && (
          <div style={{ paddingTop: '0.75rem', borderTop: '1px solid var(--border-color)', fontSize: '0.85rem', color: 'var(--text-muted)' }}>
            This cell was not changed by redistribution.
          </div>
        )}
      </div>
    </div>
  );
};

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

  const cr = result.constraint_report;
  const metrics = result.metrics ?? {};
  const fitness = metrics.fitness ?? metrics.score ?? null;
  const sc1 = metrics.fairness_penalty ?? null;
  const sc2 = metrics.fatigue_penalty ?? null;

  // ── Assignment reasoning ──────────────────────────────────────────────────
  const reasons: Array<{ icon: string; text: string; color?: string }> = [];

  if (shift) {
    // Shift type and coverage
    const shiftHours: Record<string, string> = { Early: '06:00–14:00', Late: '14:00–22:00', Night: '22:00–06:00' };
    reasons.push({ icon: '📋', text: `${shift} shift (${shiftHours[shift] ?? '8h'}) assigned on ${dateStr} to satisfy coverage requirements.` });

    // Contract eligibility
    if (member) {
      const contractNote = member.contract === 'Night'
        ? `"${member.contract}" contract — eligible for night rotations.`
        : `"${member.contract}" contract — eligible for day and evening shifts.`;
      reasons.push({ icon: '📄', text: `${member.id}: ${contractNote}` });
      reasons.push({ icon: '🎓', text: `Skill match: ${member.skills.join(', ')}.` });
    }

    // Weekend flag
    reasons.push(isWeekend
      ? { icon: '📅', text: 'Weekend assignment — counts against the maximum working weekends limit.', color: '#f59e0b' }
      : { icon: '📅', text: 'Weekday assignment — does not consume weekend allocation.' });

    // Fatigue signal (SC2)
    if (sc2 !== null) {
      const sc2Level = sc2 < 400 ? 'low' : sc2 < 800 ? 'moderate' : 'elevated';
      const sc2Color = sc2 < 400 ? 'var(--success-color)' : sc2 < 800 ? '#f59e0b' : '#ef4444';
      reasons.push({ icon: '⚡', text: `Team fatigue load is ${sc2Level} (SC2 = ${sc2.toFixed(0)}). ${sc2Level === 'elevated' ? 'High-workload workers are being assigned fewer shifts.' : 'Workload is well-distributed.'}`, color: sc2Color });
    }

    // Fairness signal (SC1)
    if (sc1 !== null) {
      const sc1Good = sc1 < 100;
      reasons.push({ icon: '⚖️', text: `Fairness penalty SC1 = ${sc1.toFixed(1)}. ${sc1Good ? 'Hours are distributed equitably across the team.' : 'Some imbalance in hours distribution — optimizer is working to reduce this.'}`, color: sc1Good ? 'var(--success-color)' : '#f59e0b' });
    }

    // Hard constraint status
    if (cr) {
      reasons.push(cr.hard_violations === 0
        ? { icon: '✓', text: 'Schedule has zero hard constraint violations.', color: 'var(--success-color)' }
        : { icon: '⚠', text: `Schedule has ${cr.hard_violations} hard violation(s) — consider re-generating.`, color: '#ef4444' });
    }
  } else {
    // Rest day reasoning
    reasons.push({ icon: '😴', text: `Rest day on ${dateStr}.` });
    if (member) reasons.push({ icon: '📄', text: `Ensures minimum consecutive days off for ${member.id} per contract rules.` });
    reasons.push({ icon: '⚡', text: 'Prevents fatigue accumulation and maintains legal rest requirements.' });
    if (isWeekend) reasons.push({ icon: '📅', text: 'Weekend rest — does not consume weekend allocation.' });
    if (sc2 !== null && sc2 > 800) {
      reasons.push({ icon: '⚡', text: `Team fatigue load is elevated (SC2 = ${sc2.toFixed(0)}). Rest days are being prioritised for high-workload workers.`, color: '#f59e0b' });
    }
  }

  // ── Satisfied constraints ─────────────────────────────────────────────────
  const satisfied = cr?.satisfied_constraints ?? [];
  const violated = cr?.violated_constraints ?? [];

  return (
    <div
      style={{ position: 'fixed', top: 0, left: 0, right: 0, bottom: 0, backgroundColor: 'rgba(15,23,42,0.75)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 1000 }}
      onClick={onClose}
    >
      <div
        style={{ backgroundColor: 'var(--bg-color)', border: '1px solid var(--border-color)', borderRadius: '12px', padding: '2rem', maxWidth: '520px', width: '92%', boxShadow: '0 25px 50px rgba(0,0,0,0.5)', maxHeight: '85vh', overflowY: 'auto' }}
        onClick={e => e.stopPropagation()}
      >
        {/* Header */}
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '1.25rem' }}>
          <div>
            <div style={{ fontWeight: 700, fontSize: '1.1rem', color: shift ? SHIFT_COLORS[shift] : 'var(--text-muted)' }}>
              {shift ? `${shift} Shift` : 'Rest Day'}
            </div>
            <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)', marginTop: '0.2rem' }}>{nurseId} · {dateStr}</div>
          </div>
          <button onClick={onClose} style={{ background: 'none', border: 'none', color: 'var(--text-muted)', cursor: 'pointer', fontSize: '1.4rem', lineHeight: 1, padding: 0 }}>×</button>
        </div>

        {/* Assignment reasoning */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: '0.6rem', marginBottom: '1.25rem' }}>
          <div style={{ fontSize: '0.78rem', fontWeight: 700, color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.06em' }}>Why this assignment?</div>
          {reasons.map((r, i) => (
            <div key={i} style={{ display: 'flex', gap: '0.6rem', fontSize: '0.87rem', color: r.color ?? 'var(--text-main)', alignItems: 'flex-start' }}>
              <span style={{ flexShrink: 0, width: '1.2rem', textAlign: 'center' }}>{r.icon}</span>
              <span>{r.text}</span>
            </div>
          ))}
        </div>

        {/* Schedule health */}
        {cr && (
          <div style={{ paddingTop: '1rem', borderTop: '1px solid var(--border-color)', marginBottom: '1rem' }}>
            <div style={{ fontSize: '0.78rem', fontWeight: 700, color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.06em', marginBottom: '0.6rem' }}>Schedule health</div>
            <div style={{ display: 'flex', gap: '1.5rem', fontSize: '0.85rem', flexWrap: 'wrap' }}>
              <div>
                <span style={{ color: cr.hard_violations === 0 ? 'var(--success-color)' : 'var(--danger-color)', fontWeight: 700 }}>{cr.hard_violations}</span>
                <span style={{ color: 'var(--text-muted)' }}> hard violations</span>
              </div>
              <div>
                <span style={{ color: '#f59e0b', fontWeight: 700 }}>{cr.soft_violations}</span>
                <span style={{ color: 'var(--text-muted)' }}> soft violations</span>
              </div>
              {fitness !== null && (
                <div>
                  <span style={{ color: 'var(--accent-color)', fontWeight: 700 }}>{fitness.toFixed(1)}</span>
                  <span style={{ color: 'var(--text-muted)' }}> fitness</span>
                </div>
              )}
              <div>
                <span style={{ color: cr.is_valid ? 'var(--success-color)' : 'var(--danger-color)', fontWeight: 700 }}>{cr.is_valid ? '✓ Valid' : '✗ Invalid'}</span>
              </div>
            </div>
          </div>
        )}

        {/* Constraint detail */}
        {(satisfied.length > 0 || violated.length > 0) && (
          <div style={{ paddingTop: '1rem', borderTop: '1px solid var(--border-color)' }}>
            <div style={{ fontSize: '0.78rem', fontWeight: 700, color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.06em', marginBottom: '0.6rem' }}>Constraints</div>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '0.3rem' }}>
              {violated.map((c, i) => (
                <div key={`v${i}`} style={{ fontSize: '0.82rem', color: '#ef4444', display: 'flex', gap: '0.4rem' }}>
                  <span>✗</span><span>{c}</span>
                </div>
              ))}
              {satisfied.slice(0, 4).map((c, i) => (
                <div key={`s${i}`} style={{ fontSize: '0.82rem', color: 'var(--success-color)', display: 'flex', gap: '0.4rem' }}>
                  <span>✓</span><span>{c}</span>
                </div>
              ))}
              {satisfied.length > 4 && (
                <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)', fontStyle: 'italic' }}>+{satisfied.length - 4} more satisfied</div>
              )}
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
  // P3.3: called when redistribution completes — parent persists the log (hard gate 5)
  decision_id?: string;
  onRedistributionComplete?: (log: RedistributionLog) => void;
}> = ({ staff, schedule, result, onScheduleChange, onNext, onBack, onRedistributionComplete }) => {
  const [activeCell, setActiveCell] = useState<{ nurseId: string; dayIdx: number } | null>(null);
  const [explainTarget, setExplainTarget] = useState<{ nurseId: string; dayIdx: number; shift: string } | null>(null);
  const [editCount, setEditCount] = useState(0);
  const [editDist, setEditDist] = useState<Record<string, number>>({
    shift_swap: 0,       // changed from one shift type to another
    coverage_fix: 0,     // changed from empty to a shift (filling a gap)
    removal: 0,          // changed from a shift to empty (removing assignment)
    weekend_change: 0,   // any edit on a weekend day (Sat=5, Sun=6 mod 7)
  });
  // P3.5: locked cells — the scheduler's explicit edits become hard locks
  const [lockedCells, setLockedCells] = useState<Set<string>>(new Set());
  const [redistResult, setRedistResult] = useState<RedistributionResult | null>(null);
  // P3.3: provenance inspector
  const [provenanceTarget, setProvenanceTarget] = useState<{ staffId: string; dayIdx: number } | null>(null);

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
    setRedistResult(null); // clear any previous redistribution result on new edit

    // Track this cell as locked (scheduler's explicit decision)
    setLockedCells(prev => new Set([...prev, `${nurseId}:${dayIdx}`]));

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
                    const cellKey = `${s.id}:${dayIdx}`;
                    // P3.3: determine provenance state for this cell
                    const provenance: AssignmentProvenanceState = redistResult
                      ? (redistResult.log.provenanceMap[cellKey] ?? 'original')
                      : lockedCells.has(cellKey) ? 'scheduler_edit' : 'original';
                    const isSchedulerEdit = provenance === 'scheduler_edit';
                    const isSystemReassignment = provenance === 'system_reassignment';
                    return (
                      <td
                        key={dayIdx}
                        style={{
                          padding: '0.3rem 0.25rem',
                          textAlign: 'center',
                          position: 'relative',
                          backgroundColor: isActive
                            ? 'rgba(56,189,248,0.12)'
                            : isSystemReassignment
                            ? 'rgba(56,189,248,0.07)'
                            : isSchedulerEdit
                            ? 'rgba(245,158,11,0.07)'
                            : isWeekend ? 'rgba(129,140,248,0.04)' : 'transparent',
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
                        {/* P3.3: provenance marker — ✎ for scheduler edit, ↻ for system reassignment */}
                        {(isSchedulerEdit || isSystemReassignment) && (
                          <span
                            style={{
                              position: 'absolute', top: '1px', left: '2px',
                              fontSize: '0.55rem', lineHeight: 1,
                              color: isSchedulerEdit ? '#f59e0b' : 'var(--accent-color)',
                              opacity: 0.85,
                              cursor: 'pointer',
                            }}
                            title={isSchedulerEdit ? 'Scheduler edit (locked)' : 'UltraRoster reassignment — click ? to inspect'}
                            onClick={e => {
                              e.stopPropagation();
                              setActiveCell(null);
                              setProvenanceTarget({ staffId: s.id, dayIdx });
                            }}
                          >
                            {isSchedulerEdit ? '✎' : '↻'}
                          </span>
                        )}
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

      {/* ── P3.3: Redistribute remaining shifts with provenance ───────────── */}
      {editCount > 0 && (
        <div style={{
          backgroundColor: 'var(--panel-bg)',
          border: '1px solid var(--border-color)',
          borderRadius: '8px',
          padding: '1rem 1.25rem',
        }}>
          <div style={{ fontSize: '0.8rem', fontWeight: 700, color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '0.5rem' }}>
            Redistribute Remaining Shifts
          </div>
          <p style={{ margin: '0 0 0.75rem 0', fontSize: '0.87rem', color: 'var(--text-muted)' }}>
            You have made <strong style={{ color: '#f59e0b' }}>{editCount} manual edit{editCount !== 1 ? 's' : ''}</strong>.
            UltraRoster can rebalance the remaining assignments while keeping your changes locked.
          </p>

          {!redistResult ? (
            <button
              onClick={() => {
                const r = redistributeWithLocks(staff, schedule, lockedCells);
                onScheduleChange(r.schedule);
                setRedistResult(r);
                // P3.3 hard gate 5: notify parent to persist the log
                onRedistributionComplete?.(r.log);
              }}
              style={{ ...primaryBtnStyle, fontSize: '0.87rem' }}
            >
              ⚡ Redistribute remaining shifts
            </button>
          ) : (() => {
            const log = redistResult.log;
            const reqs = buildStaffingRequirements();
            const coverage = computeCanonicalCoverage(reqs, redistResult.schedule);
            const lockedViolation = log.lockedAssignmentsChanged > 0;
            return (
              <div style={{ display: 'flex', flexDirection: 'column', gap: '0.6rem' }}>
                <div style={{
                  background: lockedViolation ? 'rgba(239,68,68,0.06)' : 'rgba(52,211,153,0.06)',
                  border: `1px solid ${lockedViolation ? 'var(--danger-color)' : 'rgba(52,211,153,0.3)'}`,
                  borderRadius: '6px',
                  padding: '0.75rem 0.9rem',
                  fontSize: '0.85rem',
                }}>
                  <strong style={{ color: lockedViolation ? 'var(--danger-color)' : '#34d399' }}>
                    {lockedViolation ? '⚠ Redistribution error — locked cell was changed.' : 'Redistribution completed'}
                  </strong>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '0.25rem', marginTop: '0.5rem', fontSize: '0.82rem' }}>
                    <div style={{ color: '#f59e0b' }}>
                      ✎ <strong>{log.schedulerEditsPreserved}</strong> scheduler edit{log.schedulerEditsPreserved !== 1 ? 's' : ''} preserved
                    </div>
                    <div style={{ color: 'var(--accent-color)' }}>
                      ↻ <strong>{log.assignmentsReassigned}</strong> assignment{log.assignmentsReassigned !== 1 ? 's' : ''} reassigned
                    </div>
                    <div style={{ color: lockedViolation ? 'var(--danger-color)' : 'var(--success-color)', fontWeight: 700 }}>
                      🔒 <strong>{log.lockedAssignmentsChanged}</strong> locked assignment{log.lockedAssignmentsChanged !== 1 ? 's' : ''} changed
                      {!lockedViolation && <span style={{ fontWeight: 400, color: 'var(--text-muted)' }}> — invariant satisfied</span>}
                    </div>
                    <div style={{ color: 'var(--text-muted)' }}>
                      Coverage: <strong style={{ color: coverage.gapPositions === 0 ? 'var(--success-color)' : 'var(--text-main)' }}>
                        {coverage.filledPositions} / {coverage.requiredPositions}
                      </strong> required positions
                    </div>
                  </div>
                </div>
                <div style={{ display: 'flex', gap: '1rem', fontSize: '0.78rem', color: 'var(--text-muted)' }}>
                  <span><span style={{ color: '#f59e0b' }}>✎</span> Scheduler edit</span>
                  <span><span style={{ color: 'var(--accent-color)' }}>↻</span> System reassignment — click marker to inspect</span>
                </div>
                <div style={{ fontSize: '0.78rem', color: 'var(--text-muted)', fontStyle: 'italic' }}>
                  Your {editCount} edit{editCount !== 1 ? 's' : ''} are preserved. You can continue editing or proceed to export.
                </div>
              </div>
            );
          })()}
        </div>
      )}

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

      {/* P3.3: Provenance Inspector — shown when scheduler clicks ✎ or ↻ marker */}
      {provenanceTarget && redistResult && (() => {
        const { staffId, dayIdx } = provenanceTarget;
        const cellKey = `${staffId}:${dayIdx}`;
        const provenanceState: AssignmentProvenanceState =
          redistResult.log.provenanceMap[cellKey] ?? 'original';
        const changeRecord: ChangeRecord | null =
          redistResult.log.changeRecords.find(r => r.assignmentId === cellKey) ?? null;
        const shift = (redistResult.schedule[staffId] || [])[dayIdx] ?? '';
        return (
          <ProvenanceInspector
            staffId={staffId}
            dayIdx={dayIdx}
            shift={shift}
            provenanceState={provenanceState}
            changeRecord={changeRecord}
            onClose={() => setProvenanceTarget(null)}
          />
        );
      })()}
    </div>
  );
};