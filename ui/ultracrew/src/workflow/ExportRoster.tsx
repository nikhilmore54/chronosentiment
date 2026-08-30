import React, { useState } from 'react';
import type { StaffMember, ScheduleResult, SchedulerDecision } from './WorkflowTypes';
import { primaryBtnStyle, ghostBtnStyle, accentGhostBtnStyle } from './WorkflowComponents';
import { exportRosterToExcel } from './WorkflowUtils';

const StatBlock: React.FC<{ label: string; value: string; color: string }> = ({ label, value, color }) => (
  <div style={{ display: 'flex', flexDirection: 'column', gap: '0.2rem' }}>
    <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.04em' }}>{label}</div>
    <div style={{ fontSize: '1.5rem', fontWeight: 700, color }}>{value}</div>
  </div>
);

export const ExportRoster: React.FC<{
  staff: StaffMember[];
  schedule: Record<string, string[]>;
  result: ScheduleResult;
  manualEditCount: number;
  editDistribution: Record<string, number>;
  originalAssignmentCount: number;
  schedulerDecision?: SchedulerDecision | null;
  onBack: () => void;
  onStartOver: () => void;
}> = ({ staff, schedule, result, manualEditCount, editDistribution, originalAssignmentCount, schedulerDecision, onBack, onStartOver }) => {
  const [exported, setExported] = useState(false);

  const handleExcel = () => {
    exportRosterToExcel(staff, schedule);
    setExported(true);
  };

  const cr = result.constraint_report;
  const totalShifts = Object.values(schedule).flat().filter(s => s !== '').length;
  const totalSlots = staff.length * 28;
  const coveragePct = totalSlots > 0 ? Math.round((totalShifts / totalSlots) * 100) : 0;

  // Planner Acceptance Score: fraction of optimizer-generated assignments accepted without change.
  // Denominator is originalAssignmentCount (set at generation time), not totalShifts (which
  // changes as the planner edits). This prevents a sparse schedule from inflating PAS.
  const pasDenominator = originalAssignmentCount > 0 ? originalAssignmentCount : totalShifts;
  const pas = pasDenominator > 0
    ? Math.round(((pasDenominator - manualEditCount) / pasDenominator) * 1000) / 10
    : 100;
  const pasColor = pas >= 95 ? '#22c55e' : pas >= 80 ? '#f59e0b' : '#ef4444';

  const shiftCounts: Record<string, number> = { Early: 0, Late: 0, Night: 0 };
  Object.values(schedule).flat().forEach(s => {
    if (s in shiftCounts) shiftCounts[s]++;
  });

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
      <div>
        <h3 style={{ margin: '0 0 0.5rem 0', color: 'var(--text-main)' }}>Export Roster</h3>
        <p style={{ margin: 0, color: 'var(--text-muted)', fontSize: '0.9rem' }}>
          Your schedule is ready. Export to Excel or print for distribution.
        </p>
      </div>

      {/* ── Planner Acceptance Score ─────────────────────────────────────── */}
      <div style={{
        backgroundColor: `${pasColor}10`,
        border: `1px solid ${pasColor}50`,
        borderRadius: '10px',
        padding: '1.25rem 1.5rem',
        display: 'flex',
        alignItems: 'center',
        gap: '2rem',
        flexWrap: 'wrap',
      }}>
        <div>
          <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.04em', marginBottom: '0.2rem' }}>
            Planner Acceptance Score
          </div>
          <div style={{ fontSize: '2.5rem', fontWeight: 800, color: pasColor, lineHeight: 1 }}>
            {pas.toFixed(1)}%
          </div>
        </div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '0.3rem', fontSize: '0.85rem', color: 'var(--text-muted)' }}>
          <span>Generated assignments: <strong style={{ color: 'var(--text-main)' }}>{pasDenominator}</strong></span>
          <span>Manual edits: <strong style={{ color: manualEditCount > 0 ? '#f59e0b' : 'var(--success-color)' }}>{manualEditCount}</strong></span>
          <span>Accepted without change: <strong style={{ color: pasColor }}>{pasDenominator - manualEditCount}</strong></span>
        </div>
        <div style={{ marginLeft: 'auto', fontSize: '0.8rem', color: 'var(--text-muted)', maxWidth: '200px', textAlign: 'right' }}>
          {pas >= 95 ? '✓ Planner-quality threshold met (≥ 95%)' : pas >= 80 ? '⚠ Below target — review flagged assignments' : '✗ High edit rate — schedule quality needs improvement'}
        </div>
      </div>

      {/* ── P3: Decision Provenance ──────────────────────────────────────── */}
      {schedulerDecision && (
        <div style={{
          background: schedulerDecision.overrode_recommendation
            ? 'rgba(245,158,11,0.06)'
            : 'rgba(52,211,153,0.06)',
          border: `1px solid ${schedulerDecision.overrode_recommendation ? 'rgba(245,158,11,0.3)' : 'rgba(52,211,153,0.3)'}`,
          borderRadius: '8px',
          padding: '0.75rem 1rem',
          fontSize: '0.85rem',
        }}>
          <div style={{ fontSize: '0.75rem', fontWeight: 700, color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '0.4rem' }}>
            Decision Provenance
          </div>
          <div style={{ display: 'flex', gap: '1.5rem', flexWrap: 'wrap', fontSize: '0.83rem' }}>
            <span style={{ color: 'var(--text-muted)' }}>
              Recommended: <strong style={{ color: '#34d399' }}>{schedulerDecision.recommended_id}</strong>
            </span>
            <span style={{ color: 'var(--text-muted)' }}>
              Selected: <strong style={{ color: 'var(--accent-color)' }}>{schedulerDecision.selected_id}</strong>
            </span>
            <span style={{ color: schedulerDecision.overrode_recommendation ? '#f59e0b' : '#34d399', fontWeight: 600 }}>
              {schedulerDecision.overrode_recommendation ? '⚡ Override' : '✓ Accepted recommendation'}
            </span>
          </div>
          <div style={{ marginTop: '0.3rem', fontSize: '0.75rem', color: 'var(--text-muted)' }}>
            Recorded: {new Date(schedulerDecision.created_at_iso).toLocaleString()} · ID: {schedulerDecision.decision_id}
          </div>
        </div>
      )}

      {/* ── Edit Distribution ────────────────────────────────────────────── */}
      {manualEditCount > 0 && (
        <div style={{ backgroundColor: 'var(--panel-bg)', border: '1px solid var(--border-color)', borderRadius: '8px', padding: '1rem 1.25rem' }}>
          <div style={{ fontSize: '0.8rem', fontWeight: 700, color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '0.75rem' }}>
            Edit Distribution — why the planner changed assignments
          </div>
          {/* Primary categories — these are mutually exclusive and sum to manualEditCount */}
          <div style={{ display: 'flex', gap: '1.5rem', flexWrap: 'wrap' }}>
            {[
              { key: 'shift_swap',   label: 'Shift swaps',    hint: 'Changed from one shift type to another' },
              { key: 'coverage_fix', label: 'Coverage fixes', hint: 'Filled an empty slot' },
              { key: 'removal',      label: 'Removals',       hint: 'Removed an assignment' },
            ].map(({ key, label, hint }) => {
              const count = editDistribution[key] ?? 0;
              const pct = manualEditCount > 0 ? Math.round((count / manualEditCount) * 100) : 0;
              return (
                <div key={key} title={hint} style={{ display: 'flex', flexDirection: 'column', gap: '0.15rem', minWidth: '110px' }}>
                  <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>{label}</div>
                  <div style={{ fontSize: '1.25rem', fontWeight: 700, color: count > 0 ? '#f59e0b' : 'var(--text-muted)' }}>{count}</div>
                  <div style={{ fontSize: '0.72rem', color: 'var(--text-muted)' }}>{pct}% of edits</div>
                </div>
              );
            })}
          </div>
          {/* Weekend dimension — overlaps with primary categories, shown separately */}
          {(editDistribution['weekend_change'] ?? 0) > 0 && (
            <div style={{ marginTop: '0.65rem', fontSize: '0.78rem', color: 'var(--text-muted)' }}>
              <span style={{ color: '#818cf8', fontWeight: 600 }}>
                {editDistribution['weekend_change']} of these edit{editDistribution['weekend_change'] !== 1 ? 's' : ''} occurred on a weekend
              </span>
              {' '}(overlaps with the categories above — not an additional edit).
            </div>
          )}
          <div style={{ marginTop: '0.6rem', fontSize: '0.78rem', color: 'var(--text-muted)', fontStyle: 'italic' }}>
            Dominant edit type guides optimizer priority.
          </div>
        </div>
      )}

      {/* ── Schedule stats ───────────────────────────────────────────────── */}
      <div style={{
        backgroundColor: cr?.hard_violations === 0 ? 'rgba(34,197,94,0.06)' : 'rgba(245,158,11,0.06)',
        border: `1px solid ${cr?.hard_violations === 0 ? 'rgba(34,197,94,0.3)' : 'rgba(245,158,11,0.3)'}`,
        borderRadius: '10px',
        padding: '1.25rem 1.5rem',
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(140px, 1fr))',
        gap: '1rem',
      }}>
        <StatBlock label="Staff" value={String(staff.length)} color="var(--text-main)" />
        <StatBlock label="Planning Days" value="28" color="var(--text-main)" />
        <StatBlock label="Total Shifts" value={String(totalShifts)} color="var(--accent-color)" />
        <StatBlock label="Coverage" value={`${coveragePct}%`} color={coveragePct >= 70 ? 'var(--success-color)' : '#f59e0b'} />
        <StatBlock label="Hard Violations" value={String(cr?.hard_violations ?? '—')} color={cr?.hard_violations === 0 ? 'var(--success-color)' : 'var(--danger-color)'} />
        <StatBlock label="Soft Violations" value={String(cr?.soft_violations ?? '—')} color="#f59e0b" />
      </div>

      <div style={{ backgroundColor: 'var(--panel-bg)', border: '1px solid var(--border-color)', borderRadius: '8px', padding: '1rem 1.25rem' }}>
        <div style={{ fontSize: '0.8rem', fontWeight: 700, color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '0.75rem' }}>
          Shift Distribution
        </div>
        <div style={{ display: 'flex', gap: '1.5rem', flexWrap: 'wrap' }}>
          {Object.entries(shiftCounts).map(([type, count]) => (
            <div key={type} style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
              <div style={{ width: '10px', height: '10px', borderRadius: '2px', backgroundColor: type === 'Early' ? '#38bdf8' : type === 'Late' ? '#f59e0b' : '#818cf8' }} />
              <span style={{ color: 'var(--text-main)', fontWeight: 600 }}>{count}</span>
              <span style={{ color: 'var(--text-muted)', fontSize: '0.85rem' }}>{type}</span>
            </div>
          ))}
        </div>
      </div>

      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
        <div style={{ fontSize: '0.8rem', fontWeight: 700, color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
          Export Options
        </div>
        <div style={{ display: 'flex', gap: '0.75rem', flexWrap: 'wrap' }}>
          <button onClick={handleExcel} style={{ ...primaryBtnStyle, display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <span>📊</span>
            <span>Download Excel (.xls)</span>
          </button>
          <button onClick={() => window.print()} style={{ ...accentGhostBtnStyle, display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <span>🖨</span>
            <span>Print / Save PDF</span>
          </button>
        </div>
        {exported && (
          <div style={{ backgroundColor: 'rgba(34,197,94,0.08)', border: '1px solid rgba(34,197,94,0.3)', borderRadius: '6px', padding: '0.6rem 1rem', fontSize: '0.85rem', color: 'var(--success-color)', fontWeight: 600 }}>
            ✓ Roster exported — open the downloaded .xls file in Excel or Google Sheets.
          </div>
        )}
      </div>

      {result.recommendations.length > 0 && (
        <div style={{ backgroundColor: 'var(--panel-bg)', border: '1px solid var(--border-color)', borderRadius: '8px', padding: '1rem 1.25rem' }}>
          <div style={{ fontSize: '0.8rem', fontWeight: 700, color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '0.75rem' }}>Recommendations</div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.6rem' }}>
            {result.recommendations.slice(0, 5).map((rec, i) => (
              <div key={i} style={{ borderLeft: `3px solid ${rec.severity === 'Hard' ? 'var(--danger-color)' : '#f59e0b'}`, paddingLeft: '0.75rem', fontSize: '0.85rem' }}>
                <div style={{ fontWeight: 600, color: 'var(--text-main)', marginBottom: '0.2rem' }}>{rec.constraint_id}</div>
                <div style={{ color: 'var(--text-muted)' }}>{rec.explanation}</div>
                <div style={{ color: 'var(--accent-color)', fontStyle: 'italic', marginTop: '0.2rem' }}>→ {rec.recommended_action}</div>
              </div>
            ))}
          </div>
        </div>
      )}

      <div style={{ display: 'flex', justifyContent: 'space-between', paddingTop: '0.5rem' }}>
        <button onClick={onBack} style={ghostBtnStyle}>← Back to Edit</button>
        <button onClick={onStartOver} style={{ ...ghostBtnStyle, borderColor: 'var(--accent-color)', color: 'var(--accent-color)' }}>
          Start New Schedule
        </button>
      </div>
    </div>
  );
};