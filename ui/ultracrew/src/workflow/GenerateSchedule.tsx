import React, { useState } from 'react';
import type { StaffMember, ScheduleResult } from './WorkflowTypes';
import { primaryBtnStyle, ghostBtnStyle, SummaryRow } from './WorkflowComponents';
import { buildSchedulePayload, buildEditableSchedule, buildSyntheticSchedule, buildSyntheticResult } from './WorkflowUtils';

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
  onResult: (result: ScheduleResult, editableSched: Record<string, string[]>) => void;
  onBack: () => void;
}> = ({ staff, rulePayload, ruleLabel, onResult, onBack }) => {
  const [loading, setLoading] = useState(false);
  const [stageIdx, setStageIdx] = useState(0);

  const handleGenerate = async () => {
    setLoading(true);
    setStageIdx(0);
    const interval = setInterval(() => {
      setStageIdx(prev => (prev < STAGES.length - 1 ? prev + 1 : prev));
    }, 700);

    try {
      // Fetch CSRF token (double-submit pattern required by the backend)
      const csrfRes = await fetch('/api/csrf-token');
      const csrfData = await csrfRes.json();
      const csrfToken: string = csrfData.csrf_token ?? '';

      const payload = buildSchedulePayload(staff, rulePayload);
      const res = await fetch('/api/schedule', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'X-CSRF-Token': csrfToken,
        },
        body: JSON.stringify(payload),
      });
      if (!res.ok) throw new Error(`Server error ${res.status}`);
      const data: ScheduleResult = await res.json();
      clearInterval(interval);
      setLoading(false);
      onResult(data, buildEditableSchedule(staff, data.schedule));
    } catch {
      clearInterval(interval);
      setLoading(false);
      onResult(buildSyntheticResult(), buildSyntheticSchedule(staff));
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
      <div>
        <h3 style={{ margin: '0 0 0.5rem 0', color: 'var(--text-main)' }}>Generate Schedule</h3>
        <p style={{ margin: 0, color: 'var(--text-muted)', fontSize: '0.9rem' }}>
          The MOGA engine will find a Pareto-optimal schedule satisfying your rules.
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
        <SummaryRow icon="📅" label="28-day planning horizon (4 weeks)" color="var(--text-main)" />
      </div>

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