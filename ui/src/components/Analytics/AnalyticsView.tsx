import React from 'react';
import type { CertifiedArtifact } from '../../types/artifact';

interface Props {
  artifact: CertifiedArtifact;
}

export const AnalyticsView: React.FC<Props> = ({ artifact }) => {
  const { analytics, divergence, artifact_summary, timeline } = artifact;

  // Semantic Causal Stages
  let causalityChain = ['PERFECT EXECUTION'];
  if (divergence.structural_divergence > 0) {
    if (artifact_summary.primary_cause === 'ENTRY_DRIFT') {
      causalityChain = ['Latency Injection', 'Fill Delay', 'Entry Drift', 'Exposure Offset', 'Portfolio Divergence'];
    } else {
      causalityChain = ['Anomaly Triggered', artifact_summary.primary_cause, 'State Degradation', 'Structural Divergence'];
    }
  }
  
  const affectedTrades = timeline.cascade?.filter(c => c.event === 'TRADE_ADJUSTMENT' || c.event === 'EXPOSURE_OFFSET_BEGIN')?.length || 0;
  
  const statusColor = divergence.structural_divergence > 0.1 ? 'var(--status-critical)' : 
                      divergence.structural_divergence > 0 ? 'var(--status-warning)' : 'var(--status-success)';

  const cascadeEventsCount = timeline.cascade?.length || 0;
  // Generate dynamic explanation narrative based on metrics
  const generatedNarrative = divergence.structural_divergence > 0 ? (
    `${artifact_summary.primary_cause} observed.\n\nInjected latency delayed fills by an average of ${analytics.average_delay_ticks.toFixed(1)} ticks.\n\nThis created temporary exposure offsets which propagated through ${cascadeEventsCount} downstream state transitions.\n\nReplay fidelity remained high (${(divergence.sequence_fidelity * 100).toFixed(1)}%) but localized structural divergence was observed (${(divergence.structural_divergence * 100).toFixed(1)}%).`
  ) : 'The execution trajectory remained completely faithful to the baseline strategy.';

  return (
    <div style={{ maxWidth: '1200px', margin: '0 auto', padding: '2rem', display: 'flex', flexDirection: 'column', gap: '2.5rem' }}>
      
      {/* Replay Verdict Banner (Visual Hero) */}
      <div className="glass-card" style={{ padding: '2.5rem', background: `linear-gradient(135deg, rgba(0,0,0,0.4) 0%, rgba(0,0,0,0.1) 100%)`, borderLeft: `6px solid ${statusColor}`, display: 'flex', flexDirection: 'column', gap: '1.5rem', boxShadow: 'var(--shadow-glow)' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
          <div>
            <span style={{ fontSize: '1rem', color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.1em', fontWeight: 600 }}>Certified Replay Status</span>
            <div style={{ display: 'flex', alignItems: 'center', gap: '1rem', marginTop: '0.5rem' }}>
              <h2 style={{ fontSize: '3rem', color: statusColor, textTransform: 'uppercase', margin: 0, lineHeight: 1 }}>{artifact_summary.severity}</h2>
              <span style={{ fontSize: '2rem', color: 'var(--text-primary)', fontWeight: 300 }}>|</span>
              <h2 style={{ fontSize: '2.5rem', color: 'var(--text-primary)', margin: 0, lineHeight: 1 }}>{artifact_summary.primary_cause || 'NO DIVERGENCE'}</h2>
            </div>
          </div>
          <div style={{ textAlign: 'right' }}>
             <div style={{ fontSize: '2rem', color: 'var(--text-primary)', fontWeight: 600 }}>{(divergence.sequence_fidelity * 100).toFixed(1)}%</div>
             <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Replay Fidelity</div>
          </div>
        </div>

        <div style={{ display: 'flex', gap: '3rem', marginTop: '1rem' }}>
          <div>
            <div style={{ fontSize: '1.5rem', color: divergence.structural_divergence > 0 ? 'var(--status-critical)' : 'var(--text-primary)', fontWeight: 600 }}>{(divergence.structural_divergence * 100).toFixed(1)}%</div>
            <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Structural Divergence</div>
          </div>
          <div>
            <div style={{ fontSize: '1.5rem', color: 'var(--text-primary)', fontFamily: 'monospace' }}>
               {artifact_summary.where_divergence_started !== null ? `Tick ${artifact_summary.where_divergence_started}` : 'N/A'}
            </div>
            <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>First Observed</div>
          </div>
        </div>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr', gap: '2.5rem' }}>
        
        {/* Explanation Narrative */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
          <div style={{ paddingLeft: '1.5rem', borderLeft: `4px solid ${statusColor}`, display: 'flex', flexDirection: 'column', justifyContent: 'center' }}>
            <h3 style={{ fontSize: '0.875rem', color: 'var(--text-secondary)', textTransform: 'uppercase', marginBottom: '1rem', letterSpacing: '0.05em' }}>Explanation</h3>
            <p style={{ fontSize: '1.25rem', color: 'var(--text-primary)', lineHeight: 1.6, whiteSpace: 'pre-wrap', fontWeight: 300 }}>
              {generatedNarrative}
            </p>
          </div>
        </div>

        {/* Right Column: Execution Realism (Bloomberg Style) */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
          <h3 style={{ fontSize: '0.875rem', color: 'var(--text-secondary)', textTransform: 'uppercase', borderBottom: '1px solid var(--border-color)', paddingBottom: '0.5rem' }}>Execution Realism</h3>
          
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '2rem' }}>
            <div>
              <div style={{ fontSize: '2rem', fontWeight: 300, color: 'var(--text-primary)' }}>{(analytics.fill_rate * 100).toFixed(1)}<span style={{ fontSize: '1rem' }}>%</span></div>
              <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Fill Rate</div>
            </div>
            <div>
              <div style={{ fontSize: '2rem', fontWeight: 300, color: 'var(--text-primary)' }}>{analytics.trades}</div>
              <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Trades Submitted</div>
            </div>
            <div>
              <div style={{ fontSize: '2rem', fontWeight: 300, color: analytics.missed_fills > 0 ? 'var(--status-critical)' : 'var(--text-primary)' }}>{analytics.missed_fills}</div>
              <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Missed Fills</div>
            </div>
            <div>
              <div style={{ fontSize: '2rem', fontWeight: 300, color: 'var(--status-warning)' }}>{analytics.average_delay_ticks.toFixed(1)}</div>
              <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Avg Delay (Ticks)</div>
            </div>
            <div>
              <div style={{ fontSize: '2rem', fontWeight: 300, color: 'var(--status-critical)' }}>{analytics.average_slippage_bps.toFixed(2)}</div>
              <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Avg Slippage (bps)</div>
            </div>
          </div>
        </div>

      </div>

      <div style={{ marginTop: '1rem', paddingTop: '2rem', borderTop: '1px solid var(--border-color)' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-end', marginBottom: '1.5rem' }}>
          <h3 style={{ fontSize: '0.875rem', color: 'var(--text-secondary)', textTransform: 'uppercase', margin: 0 }}>Propagation Graph (Causal Chain)</h3>
          <div style={{ display: 'flex', gap: '1.5rem', fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>
            <span><strong style={{ color: 'var(--text-primary)' }}>{causalityChain.length}</strong> Causal Stages</span>
            <span><strong style={{ color: 'var(--text-primary)' }}>{cascadeEventsCount}</strong> Manifestations</span>
            <span><strong style={{ color: 'var(--text-primary)' }}>{affectedTrades}</strong> Affected Trades</span>
          </div>
        </div>
        
        <div style={{ display: 'flex', alignItems: 'center', gap: '1rem', flexWrap: 'wrap' }}>
          {causalityChain.map((eventStr, index) => (
            <React.Fragment key={index}>
              <div style={{ 
                padding: '0.75rem 1rem', 
                background: index === 0 && causalityChain.length > 1 ? 'rgba(217, 119, 6, 0.1)' : 
                            index === causalityChain.length - 1 && causalityChain.length > 1 ? 'rgba(220, 38, 38, 0.1)' : 'var(--bg-panel)',
                border: '1px solid',
                borderColor: index === 0 && causalityChain.length > 1 ? 'var(--status-warning)' : 
                             index === causalityChain.length - 1 && causalityChain.length > 1 ? 'var(--status-critical)' : 'var(--border-color)',
                borderRadius: 'var(--radius-md)',
                color: index === causalityChain.length - 1 && causalityChain.length > 1 ? 'var(--status-critical)' : 'var(--text-primary)',
                fontWeight: 600,
                fontSize: '0.875rem'
              }}>
                {eventStr}
              </div>
              
              {index < causalityChain.length - 1 && (
                <div style={{ color: 'var(--text-muted)', display: 'flex', alignItems: 'center' }}>
                  <span style={{ fontSize: '1.5rem' }}>➔</span>
                </div>
              )}
            </React.Fragment>
          ))}
        </div>
      </div>

    </div>
  );
};
