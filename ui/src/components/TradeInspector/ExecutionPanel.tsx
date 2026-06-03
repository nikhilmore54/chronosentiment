import React from 'react';
import type { TradeLeg, ExecutionDeltaLayer } from '../../types/tradeInspector';

interface Props {
  baseline: TradeLeg | null;
  perturbed: TradeLeg | null;
  delta: ExecutionDeltaLayer;
}

export const ExecutionPanel: React.FC<Props> = ({ baseline, perturbed, delta }) => {
  return (
    <div className="glass-card animate-fade-in" style={{ padding: '0', overflow: 'hidden', animationDelay: '0.1s' }}>
      <div style={{ padding: '1rem 1.5rem', borderBottom: '1px solid var(--border-color)', background: 'var(--bg-hover)' }}>
        <h3 style={{ color: 'var(--text-secondary)', fontSize: '0.875rem', textTransform: 'uppercase', letterSpacing: '0.05em', margin: 0 }}>Execution Comparison</h3>
      </div>
      
      {/* Highlight Deltas Top Level */}
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', padding: '1.5rem', borderBottom: '1px solid var(--border-color)', background: 'rgba(0,0,0,0.1)' }}>
        <div className="flex-col">
          <span style={{ color: 'var(--text-muted)', fontSize: '0.85rem', textTransform: 'uppercase', letterSpacing: '0.05em' }}>Fill Delay</span>
          <span style={{ fontSize: '2rem', fontWeight: 600, color: delta.delay_ms > 0 ? 'var(--status-warning)' : 'var(--status-success)' }}>
            +{delta.delay_ticks} ticks
          </span>
          <span style={{ color: 'var(--text-secondary)', fontSize: '0.85rem' }}>+{delta.delay_ms}ms injected latency</span>
        </div>
        <div className="flex-col">
          <span style={{ color: 'var(--text-muted)', fontSize: '0.85rem', textTransform: 'uppercase', letterSpacing: '0.05em' }}>Slippage</span>
          <span style={{ fontSize: '2rem', fontWeight: 600, color: delta.slippage_bps > 0 ? 'var(--status-critical)' : 'var(--status-success)' }}>
            {delta.slippage_bps > 0 ? '+' : ''}{delta.slippage_bps.toFixed(2)} bps
          </span>
        </div>
      </div>

      {/* Raw Values */}
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr' }}>
        
        {/* Baseline Column */}
        <div style={{ padding: '1.5rem', borderRight: '1px solid var(--border-color)' }}>
          <span style={{ color: 'var(--text-muted)', fontSize: '0.75rem', textTransform: 'uppercase', marginBottom: '0.5rem', display: 'block' }}>BASELINE RAW</span>
          {baseline ? (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '0.25rem' }}>
              <span style={{ fontFamily: 'monospace', fontSize: '1.1rem', color: 'var(--text-primary)' }}>Tick {baseline.fill_time}</span>
              <span style={{ fontSize: '1rem', color: 'var(--text-secondary)' }}>{baseline.fill_price.toFixed(2)}</span>
            </div>
          ) : (
            <span style={{ fontSize: '1.1rem', color: 'var(--text-muted)' }}>Missed</span>
          )}
        </div>
        
        {/* Perturbed Column */}
        <div style={{ padding: '1.5rem' }}>
          <span style={{ color: 'var(--text-muted)', fontSize: '0.75rem', textTransform: 'uppercase', marginBottom: '0.5rem', display: 'block' }}>PERTURBED RAW</span>
          {perturbed ? (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '0.25rem' }}>
              <span style={{ fontFamily: 'monospace', fontSize: '1.1rem', color: delta.delay_ticks > 0 ? 'var(--status-warning)' : 'var(--text-primary)' }}>Tick {perturbed.fill_time}</span>
              <span style={{ fontSize: '1rem', color: delta.slippage_bps > 0 ? 'var(--status-critical)' : 'var(--text-secondary)' }}>{perturbed.fill_price.toFixed(2)}</span>
            </div>
          ) : (
            <span style={{ fontSize: '1.1rem', color: 'var(--status-critical)' }}>Missed</span>
          )}
        </div>
      </div>
    </div>
  );
};
