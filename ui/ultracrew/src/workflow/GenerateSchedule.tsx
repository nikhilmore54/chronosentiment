import React, { useState, useMemo } from 'react';
import type { StaffMember, ScheduleResult, ScheduleRawResponse } from './WorkflowTypes';
import { primaryBtnStyle, ghostBtnStyle, SummaryRow } from './WorkflowComponents';
import {
  buildSchedulePayload,
  buildEditableSchedule,
  mapParetoAlternatives,
  computeDemandCapacityDiagnostic,
  getScheduleSlotMeta,
  type CustomShiftInput,
  type ShiftSlotMeta,
} from './WorkflowUtils';

const STAGES = [
  'Validating staff and rules...',
  'Initialising population...',
  'Running MOGA optimiser...',
  'Evaluating constraint satisfaction...',
  'Selecting Pareto-optimal schedule...',
];

export const GenerateSchedule: React.FC<{
  staff: StaffMember[];
  rulePayload: object;
  ruleLabel: string;
  customShifts?: CustomShiftInput[];
  customSlotMeta?: Record<number, ShiftSlotMeta>;
  onResult: (result: ScheduleResult, editableSched: Record<string, string[]>) => void;
  onBack: () => void;
}> = ({ staff, rulePayload, ruleLabel, customShifts, customSlotMeta, onResult, onBack }) => {
  const [loading, setLoading] = useState(false);
  const [stageIdx, setStageIdx] = useState(0);
  const [error, setError] = useState<string | null>(null);

  const slotMeta = useMemo(() => customSlotMeta ?? getScheduleSlotMeta(staff, customShifts), [staff, customShifts, customSlotMeta]);
  const diagnostic = useMemo(() => computeDemandCapacityDiagnostic(staff, customShifts), [staff, customShifts]);

  const handleGenerate = async () => {
    setLoading(true);
    setError(null);
    setStageIdx(0);
    const interval = setInterval(() => {
      setStageIdx(prev => (prev < STAGES.length - 1 ? prev + 1 : prev));
    }, 700);

    try {
      // Fetch CSRF token (double-submit pattern required by the backend)
      const csrfRes = await fetch('/api/csrf-token');
      if (!csrfRes.ok) throw new Error(`CSRF token request failed (${csrfRes.status})`);
      const csrfData = await csrfRes.json();
      const csrfToken: string = csrfData.csrf_token ?? '';

      const payload = buildSchedulePayload(staff, rulePayload, customShifts);
      const res = await fetch('/api/schedule', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'X-CSRF-Token': csrfToken,
        },
        body: JSON.stringify(payload),
      });
      if (!res.ok) throw new Error(`Schedule generation failed (${res.status})`);
      // P0-E: Parse as ScheduleRawResponse (actual backend shape), then map
      // alternatives from ProductionParetoSolution[] → RosterAlternative[].
      // This closes the type contract gap between the Rust backend and the UI,
      // including the authoritative primary schedule as Option 1 (recommended).
      const raw: ScheduleRawResponse = await res.json();
      const mappedAlternatives = mapParetoAlternatives(
        raw.alternatives ?? [],
        staff,
        raw.schedule ? {
          schedule: raw.schedule,
          metrics: raw.metrics,
          constraint_report: raw.constraint_report,
          recommendations: raw.recommendations,
        } : undefined,
        slotMeta,
      );
      const data: ScheduleResult = {
        schedule: raw.schedule,
        metrics: raw.metrics,
        constraint_report: raw.constraint_report,
        recommendations: raw.recommendations,
        alternatives: mappedAlternatives,
        recommended_alternative_id: raw.recommended_alternative_id ?? mappedAlternatives[0]?.id,
      };
      clearInterval(interval);
      setLoading(false);
      onResult(data, buildEditableSchedule(staff, data.schedule, slotMeta));
    } catch (err) {
      clearInterval(interval);
      setLoading(false);
      const message = err instanceof Error ? err.message : 'Unknown error';
      setError(`Unable to generate schedule: ${message}. Please check the backend is running and try again.`);
      // Do NOT fall back to synthetic data — the scheduler must see a real optimizer result.
    }
  };

  const statusColors = {
    balanced: { bg: 'rgba(34, 197, 94, 0.08)', border: 'rgba(34, 197, 94, 0.3)', text: '#22c55e', label: '✓ Balanced Capacity' },
    tight: { bg: 'rgba(245, 158, 11, 0.08)', border: 'rgba(245, 158, 11, 0.3)', text: '#f59e0b', label: '⚠ Tight Capacity' },
    under_capacity: { bg: 'rgba(239, 68, 68, 0.08)', border: 'rgba(239, 68, 68, 0.3)', text: '#ef4444', label: '✕ Under-Capacity Deficit' },
    excess_capacity: { bg: 'rgba(56, 189, 248, 0.08)', border: 'rgba(56, 189, 248, 0.3)', text: '#38bdf8', label: 'ℹ Surplus Capacity' },
  }[diagnostic.status];

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
      <div>
        <h3 style={{ margin: '0 0 0.5rem 0', color: 'var(--text-main)' }}>Generate Schedule</h3>
        <p style={{ margin: 0, color: 'var(--text-muted)', fontSize: '0.9rem' }}>
          The MOGA engine will find a Pareto-optimal schedule satisfying your rules and hospital demand.
        </p>
      </div>

      <div style={{
        backgroundColor: 'var(--panel-bg)',
        border: '1px solid var(--border-color)',
        borderRadius: '8px',
        padding: '1.25rem',
        display: 'flex',
        flexDirection: 'column',
        gap: '0.6rem',
      }}>
        <SummaryRow icon="👥" label={`${staff.length} staff members`} color="var(--text-main)" />
        <SummaryRow icon="📋" label={`Rule set: ${ruleLabel}`} color="var(--text-main)" />
        <SummaryRow
          icon="📅"
          label={`${diagnostic.horizonDays}-day planning horizon (${diagnostic.shiftCount} shift positions required)`}
          color="var(--text-main)"
        />
        {customShifts && customShifts.length > 0 && (
          <SummaryRow icon="⚡" label="Custom hospital demand loaded" color="var(--accent-color)" />
        )}
      </div>

      {/* Pre-Optimization Demand-Capacity Diagnostic */}
      <div style={{
        backgroundColor: statusColors.bg,
        border: `1px solid ${statusColors.border}`,
        borderRadius: '8px',
        padding: '1.1rem 1.25rem',
        display: 'flex',
        flexDirection: 'column',
        gap: '0.65rem',
      }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <div style={{ fontSize: '0.8rem', fontWeight: 700, color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
            Pre-Optimization Demand–Capacity Diagnostic
          </div>
          <span style={{
            fontSize: '0.78rem',
            fontWeight: 700,
            color: statusColors.text,
            backgroundColor: `${statusColors.text}18`,
            padding: '0.2rem 0.6rem',
            borderRadius: '4px',
          }}>
            {statusColors.label}
          </span>
        </div>

        <div style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(130px, 1fr))',
          gap: '1rem',
          paddingTop: '0.25rem',
        }}>
          <div>
            <div style={{ fontSize: '0.72rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Demand Hours</div>
            <div style={{ fontSize: '1.25rem', fontWeight: 700, color: 'var(--text-main)' }}>{diagnostic.demandHours}h</div>
            <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>{diagnostic.shiftCount} shifts</div>
          </div>
          <div>
            <div style={{ fontSize: '0.72rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Workforce Capacity</div>
            <div style={{ fontSize: '1.25rem', fontWeight: 700, color: 'var(--text-main)' }}>{diagnostic.workforceCapacityHours}h</div>
            <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>{staff.length} contracted staff</div>
          </div>
          <div>
            <div style={{ fontSize: '0.72rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Capacity Utilization</div>
            <div style={{ fontSize: '1.25rem', fontWeight: 700, color: statusColors.text }}>
              {Math.round(diagnostic.utilizationRatio * 100)}%
            </div>
            <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>demand / max capacity</div>
          </div>
        </div>

        <div style={{ fontSize: '0.82rem', color: 'var(--text-muted)', borderTop: `1px solid ${statusColors.border}`, paddingTop: '0.5rem' }}>
          {diagnostic.message}
        </div>
      </div>

      {error && (
        <div style={{
          backgroundColor: 'rgba(239, 68, 68, 0.1)',
          border: '1px solid rgba(239, 68, 68, 0.4)',
          borderRadius: '8px',
          padding: '1rem 1.25rem',
          color: '#fca5a5',
          fontSize: '0.9rem',
        }}>
          <strong>Generation failed</strong><br />
          {error}
        </div>
      )}

      {loading && (
        <div style={{ textAlign: 'center', padding: '2rem' }}>
          <div style={{
            width: '40px',
            height: '40px',
            border: '4px solid rgba(56, 189, 248, 0.15)',
            borderTopColor: 'var(--accent-color)',
            borderRadius: '50%',
            animation: 'spin 1s linear infinite',
            margin: '0 auto 1.25rem',
          }} />
          <div style={{ color: 'var(--accent-color)', fontWeight: 600 }}>{STAGES[stageIdx]}</div>
          <style>{`@keyframes spin{0%{transform:rotate(0deg)}100%{transform:rotate(360deg)}}`}</style>
        </div>
      )}

      <div style={{ display: 'flex', justifyContent: 'space-between' }}>
        <button onClick={onBack} style={ghostBtnStyle} disabled={loading}>← Back</button>
        <button
          onClick={handleGenerate}
          disabled={loading}
          style={{ ...primaryBtnStyle, opacity: loading ? 0.5 : 1 }}
        >
          {loading ? 'Generating...' : '⚡ Generate Schedule'}
        </button>
      </div>
    </div>
  );
};