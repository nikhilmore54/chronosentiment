import React from 'react';

interface Props {
  trade_id: string;
  strategy: string;
}

export const TradeHeader: React.FC<Props> = ({ trade_id, strategy }) => {
  return (
    <div style={{ marginBottom: '2rem' }}>
      <div className="flex-row gap-sm" style={{ marginBottom: '0.5rem' }}>
        <span className="badge badge-info">{strategy.toUpperCase()}</span>
      </div>
      <h1 style={{ fontSize: '2rem', color: 'var(--text-primary)' }}>Trade {trade_id}</h1>
      <p style={{ color: 'var(--text-secondary)' }}>Execution Consequence Trace</p>
    </div>
  );
};
