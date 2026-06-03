import React from 'react';
import type { ExecutionDeltaLayer } from '../../types/tradeInspector';

interface Props {
  delta: ExecutionDeltaLayer;
}

export const OutcomePanel: React.FC<Props> = ({ delta }) => {
  return (
    <div className="glass-card animate-fade-in" style={{ padding: '1.5rem', animationDelay: '0.2s' }}>
      <h3 style={{ color: 'var(--text-secondary)', marginBottom: '1rem', fontSize: '0.875rem', textTransform: 'uppercase', letterSpacing: '0.05em' }}>Outcome Layer</h3>
      
      <div className="flex-col gap-md">
        <div className="flex-row justify-between" style={{ alignItems: 'flex-start' }}>
          <div className="flex-col">
            <span style={{ color: 'var(--text-primary)', fontWeight: 500 }}>Structural Divergence</span>
            <span style={{ color: 'var(--text-muted)', fontSize: '0.875rem' }}>Did this trade separate the timelines?</span>
          </div>
          {delta.diverged ? (
            <span className="badge badge-critical">Diverged</span>
          ) : (
            <span className="badge badge-success">Intact</span>
          )}
        </div>

        <div className="flex-row justify-between" style={{ alignItems: 'flex-start' }}>
          <div className="flex-col">
            <span style={{ color: 'var(--text-primary)', fontWeight: 500 }}>Fill Status</span>
            <span style={{ color: 'var(--text-muted)', fontSize: '0.875rem' }}>Queue progression outcome</span>
          </div>
          {delta.missed_fill ? (
            <span className="badge badge-warning">Missed</span>
          ) : (
            <span className="badge badge-success">Filled</span>
          )}
        </div>
      </div>
    </div>
  );
};
