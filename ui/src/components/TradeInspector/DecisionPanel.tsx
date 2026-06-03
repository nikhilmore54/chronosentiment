import React from 'react';
import type { DecisionLayer } from '../../types/tradeInspector';

interface Props {
  data: DecisionLayer;
}

export const DecisionPanel: React.FC<Props> = ({ data }) => {
  return (
    <div className="glass-card animate-fade-in" style={{ padding: '1.5rem' }}>
      <h3 style={{ color: 'var(--text-secondary)', marginBottom: '1rem', fontSize: '0.875rem', textTransform: 'uppercase', letterSpacing: '0.05em' }}>Decision Layer</h3>
      <div className="flex-col gap-sm">
        <div className="flex-row justify-between">
          <span style={{ color: 'var(--text-muted)' }}>Signal Timestamp</span>
          <span style={{ fontFamily: 'monospace', color: 'var(--text-primary)' }}>Tick {data.signal_timestamp}</span>
        </div>
        <div className="flex-row justify-between">
          <span style={{ color: 'var(--text-muted)' }}>Signal Type</span>
          <span className="badge badge-info">{data.signal_type}</span>
        </div>
      </div>
    </div>
  );
};
