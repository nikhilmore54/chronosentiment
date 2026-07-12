import React from 'react';
import type { WorkflowStep } from './WorkflowTypes';
import { WORKFLOW_STEPS } from './WorkflowTypes';

// ─── Shared button styles ─────────────────────────────────────────────────────

export const primaryBtnStyle: React.CSSProperties = {
  backgroundColor: 'var(--primary-color)',
  color: 'white',
  border: 'none',
  padding: '0.65rem 1.4rem',
  borderRadius: '6px',
  cursor: 'pointer',
  fontWeight: 600,
  fontSize: '0.9rem',
  display: 'inline-block',
};

export const ghostBtnStyle: React.CSSProperties = {
  background: 'none',
  border: '1px solid var(--border-color)',
  color: 'var(--text-muted)',
  padding: '0.65rem 1.25rem',
  borderRadius: '6px',
  cursor: 'pointer',
  fontSize: '0.9rem',
};

export const accentGhostBtnStyle: React.CSSProperties = {
  background: 'none',
  border: '1px solid var(--accent-color)',
  color: 'var(--accent-color)',
  padding: '0.65rem 1.25rem',
  borderRadius: '6px',
  cursor: 'pointer',
  fontSize: '0.9rem',
};

// ─── Shared box styles ────────────────────────────────────────────────────────

export const codeBoxStyle: React.CSSProperties = {
  backgroundColor: 'rgba(15, 23, 42, 0.8)',
  border: '1px solid var(--border-color)',
  borderRadius: '8px',
  padding: '1rem',
};

export const textareaStyle: React.CSSProperties = {
  width: '100%',
  height: '80px',
  backgroundColor: '#0f172a',
  color: 'var(--text-main)',
  border: '1px solid var(--border-color)',
  borderRadius: '4px',
  padding: '0.5rem',
  fontSize: '0.8rem',
  fontFamily: 'monospace',
  resize: 'vertical',
  boxSizing: 'border-box',
};

export const errorBoxStyle: React.CSSProperties = {
  backgroundColor: 'rgba(239, 68, 68, 0.08)',
  border: '1px solid var(--danger-color)',
  borderRadius: '6px',
  padding: '0.75rem 1rem',
  color: 'var(--danger-color)',
};

// ─── Reusable display components ──────────────────────────────────────────────

export const SummaryRow: React.FC<{ icon: string; label: string; color: string }> = ({ icon, label, color }) => (
  <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', fontSize: '0.9rem' }}>
    <span style={{ color }}>{icon}</span>
    <span style={{ color }}>{label}</span>
  </div>
);

export const SkillBadge: React.FC<{ skill: string }> = ({ skill }) => (
  <span style={{
    backgroundColor: 'rgba(56, 189, 248, 0.1)',
    color: 'var(--accent-color)',
    padding: '0.15rem 0.4rem',
    borderRadius: '4px',
    fontSize: '0.78rem',
    marginRight: '0.3rem',
    display: 'inline-block',
  }}>{skill}</span>
);

// ─── Stepper header ───────────────────────────────────────────────────────────

export const Stepper: React.FC<{
  currentStep: number;
  maxReached: number;
  onJump: (s: number) => void;
  steps?: WorkflowStep[];
}> = ({ currentStep, maxReached, onJump, steps = WORKFLOW_STEPS }) => (
  <div style={{ display: 'flex', alignItems: 'center', gap: 0, marginBottom: '2rem', overflowX: 'auto' }}>
    {steps.map((step, idx) => {
      const done = step.num < currentStep;
      const active = step.num === currentStep;
      const reachable = step.num <= maxReached;
      return (
        <React.Fragment key={step.num}>
          <div
            onClick={() => reachable && onJump(step.num)}
            style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '0.4rem', cursor: reachable ? 'pointer' : 'default', minWidth: '80px' }}
          >
            <div style={{
              width: '32px', height: '32px', borderRadius: '50%',
              display: 'flex', alignItems: 'center', justifyContent: 'center',
              fontWeight: 700, fontSize: '0.85rem',
              backgroundColor: done ? 'var(--success-color)' : active ? 'var(--accent-color)' : 'var(--panel-bg)',
              color: done || active ? 'white' : reachable ? 'var(--text-muted)' : '#334155',
              border: active ? '2px solid var(--accent-color)' : done ? '2px solid var(--success-color)' : '2px solid var(--border-color)',
              transition: 'all 0.2s',
            }}>
              {done ? '✓' : step.num}
            </div>
            <span style={{
              fontSize: '0.72rem', fontWeight: active ? 700 : 400, whiteSpace: 'nowrap',
              color: active ? 'var(--accent-color)' : done ? 'var(--success-color)' : reachable ? 'var(--text-muted)' : '#334155',
            }}>
              {step.label}
            </span>
          </div>
          {idx < steps.length - 1 && (
            <div style={{
              flex: 1, height: '2px', minWidth: '24px', marginBottom: '20px',
              backgroundColor: step.num < currentStep ? 'var(--success-color)' : 'var(--border-color)',
            }} />
          )}
        </React.Fragment>
      );
    })}
  </div>
);