import React from 'react';
import type { ExplanationRule } from '../../types/tradeInspector';

interface Props {
  explanations: ExplanationRule[];
}

export const ExplanationPanel: React.FC<Props> = ({ explanations }) => {
  if (explanations.length === 0) {
    return null;
  }

  return (
    <div className="glass-card animate-fade-in" style={{ padding: '1.5rem', animationDelay: '0.3s', borderColor: 'rgba(139, 92, 246, 0.3)' }}>
      <h3 style={{ color: 'var(--accent-purple)', marginBottom: '1rem', fontSize: '0.875rem', textTransform: 'uppercase', letterSpacing: '0.05em' }}>Explanation Layer</h3>
      
      <div className="flex-col gap-sm">
        {explanations.map((rule, idx) => (
          <div key={idx} className="glass-panel flex-row justify-between" style={{ padding: '1rem', background: 'rgba(139, 92, 246, 0.05)' }}>
            <div className="flex-col">
              <span style={{ color: 'var(--text-primary)', fontWeight: 500, marginBottom: '0.25rem' }}>{rule.type}</span>
              <span style={{ color: 'var(--text-secondary)', fontSize: '0.875rem' }}>{rule.message}</span>
            </div>
            <span className={`badge badge-${rule.severity === 'critical' ? 'critical' : rule.severity === 'warning' ? 'warning' : 'info'}`}>
              {rule.severity}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
};
