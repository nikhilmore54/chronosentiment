import React, { useEffect, useRef, useState } from 'react';
import type { TimelineContract } from '../../types/timeline';

interface Props {
  timeline: TimelineContract;
  onJumpToTrade: (tradeId: string) => void;
}

export const TimelineView: React.FC<Props> = ({ timeline, onJumpToTrade }) => {
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const [zoomLevel, setZoomLevel] = useState<number>(1); // 0.5, 1, 2, 4, 8

  const zoomOptions = [0.5, 1, 2, 4, 8];
  const baseTickWidth = 60;
  const currentTickWidth = baseTickWidth * zoomLevel;

  const anchorTick = timeline.divergence_anchor_tick;
  
  // Extract semantic causal stages for the navigator
  let causalityChain = ['PERFECT EXECUTION'];
  if (timeline.divergence_anchor_tick !== null && timeline.divergence_reason) {
    if (timeline.divergence_reason === 'ENTRY_DRIFT') {
      causalityChain = ['Latency Injection', 'Fill Delay', 'Entry Drift', 'Exposure Offset', 'Portfolio Divergence'];
    } else {
      causalityChain = ['Anomaly Triggered', timeline.divergence_reason, 'State Degradation', 'Structural Divergence'];
    }
  }

  // Label frequency based on zoom
  const getLabelFrequency = (z: number) => {
    if (z <= 0.5) return 300; // 5 min
    if (z === 1) return 60;   // 1 min
    if (z === 2) return 30;   // 30 sec
    if (z === 4) return 15;   // 15 sec
    return 1; // 1 sec
  };
  const labelFreq = getLabelFrequency(zoomLevel);
  const needsAggregation = zoomLevel <= 1;

  const scrollToAnchor = () => {
    if (scrollContainerRef.current && anchorTick !== null) {
      const index = timeline.lanes.findIndex(l => l.tick === anchorTick);
      if (index >= 0) {
        // Scroll so the anchor is roughly in the middle
        const containerWidth = scrollContainerRef.current.clientWidth;
        const targetScrollLeft = (index * currentTickWidth) - (containerWidth / 2) + (currentTickWidth / 2);
        scrollContainerRef.current.scrollTo({ left: Math.max(0, targetScrollLeft), behavior: 'smooth' });
      }
    }
  };

  useEffect(() => {
    // Initial jump to anchor
    setTimeout(scrollToAnchor, 100);
  }, [timeline, currentTickWidth]);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', padding: '2rem' }}>
      
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '1.5rem' }}>
        <div>
          <h1 style={{ fontSize: '1.5rem', color: 'var(--text-primary)', marginBottom: '0.25rem' }}>Timeline Synchronizer</h1>
          
          {anchorTick !== null ? (
            <div style={{ display: 'flex', alignItems: 'center', gap: '1rem', marginTop: '0.5rem' }}>
              <div style={{ color: 'var(--status-critical)', fontSize: '0.875rem', fontWeight: 600 }}>
                Anchor Divergence: Tick {anchorTick}
              </div>
              <button 
                onClick={scrollToAnchor}
                style={{
                  padding: '0.25rem 0.75rem',
                  background: 'rgba(239, 68, 68, 0.1)',
                  color: 'var(--status-critical)',
                  border: '1px solid rgba(239, 68, 68, 0.3)',
                  borderRadius: 'var(--radius-md)',
                  cursor: 'pointer',
                  fontSize: '0.75rem',
                  fontWeight: 600
                }}
              >
                Jump To First Divergence
              </button>
            </div>
          ) : (
            <p style={{ color: 'var(--text-secondary)', fontSize: '0.875rem' }}>
              No structural divergence detected in this timeline.
            </p>
          )}
        </div>

        {/* Divergence Navigator (P2) */}
        {anchorTick !== null && (
          <div className="glass-card" style={{ padding: '0.5rem 1rem', display: 'flex', alignItems: 'center', gap: '1rem', border: '1px solid var(--status-critical)', background: 'rgba(239, 68, 68, 0.05)' }}>
            <button style={{ background: 'transparent', border: 'none', color: 'var(--text-muted)', cursor: 'not-allowed', fontSize: '0.75rem' }}>◀ Prev</button>
            <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
               <span style={{ color: 'var(--status-critical)', fontWeight: 600, fontSize: '0.875rem', textTransform: 'uppercase' }}>#1 {timeline.divergence_reason || 'DIVERGENCE'}</span>
               <span style={{ color: 'var(--text-muted)', fontSize: '0.75rem', fontFamily: 'monospace' }}>Tick {anchorTick}</span>
            </div>
            <button style={{ background: 'transparent', border: 'none', color: 'var(--text-muted)', cursor: 'not-allowed', fontSize: '0.75rem' }}>Next ▶</button>
          </div>
        )}

        <div className="glass-card" style={{ padding: '0.5rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <span style={{ fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Zoom</span>
          {zoomOptions.map(z => (
            <button
              key={z}
              onClick={() => setZoomLevel(z)}
              style={{
                padding: '0.25rem 0.5rem',
                fontSize: '0.75rem',
                background: zoomLevel === z ? 'var(--accent-blue)' : 'transparent',
                color: zoomLevel === z ? '#fff' : 'var(--text-primary)',
                border: '1px solid',
                borderColor: zoomLevel === z ? 'var(--accent-blue)' : 'var(--border-color)',
                borderRadius: 'var(--radius-md)',
                cursor: 'pointer'
              }}
            >
              {z}x
            </button>
          ))}
        </div>
      </div>

      {/* Causality Explorer */}
      {anchorTick !== null && (
        <div style={{ marginBottom: '2rem', display: 'flex', flexDirection: 'column', alignItems: 'center', padding: '1rem', background: 'rgba(0,0,0,0.1)', border: '1px dashed var(--border-color)', borderRadius: 'var(--radius-lg)' }}>
          <h3 style={{ color: 'var(--text-secondary)', marginBottom: '1rem', fontSize: '0.75rem', textTransform: 'uppercase', letterSpacing: '0.1em' }}>
            Causality Explorer
          </h3>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            {causalityChain.map((stageStr, i) => (
              <React.Fragment key={i}>
                <button 
                  onClick={() => timeline.divergence_anchor_trade_id && onJumpToTrade(timeline.divergence_anchor_trade_id)}
                  style={{
                    display: 'flex',
                    flexDirection: 'column',
                    alignItems: 'center',
                    background: 'var(--bg-panel)',
                    border: '1px solid',
                    borderColor: i === causalityChain.length - 1 ? 'var(--status-critical)' : 'var(--border-color)',
                    padding: '0.5rem 1rem',
                    borderRadius: 'var(--radius-md)',
                    cursor: timeline.divergence_anchor_trade_id ? 'pointer' : 'default',
                    transition: 'all 0.2s',
                    boxShadow: 'var(--shadow-subtle)'
                  }}
                >
                  <span style={{ fontSize: '0.85rem', fontWeight: 600, color: 'var(--text-primary)' }}>{stageStr}</span>
                </button>
                {i < causalityChain.length - 1 && (
                  <span style={{ color: 'var(--border-color)' }}>──▶</span>
                )}
              </React.Fragment>
            ))}
          </div>
          {timeline.divergence_anchor_trade_id && (
             <div style={{ marginTop: '1rem', fontSize: '0.75rem', color: 'var(--text-muted)' }}>
               Click any node to inspect Trade {timeline.divergence_anchor_trade_id}
             </div>
          )}
        </div>
      )}

      {/* Horizontal Scrollable Timeline Area */}
      <div 
        className="glass-panel"
        style={{ 
          flex: 1, 
          display: 'flex', 
          overflow: 'hidden',
          position: 'relative' 
        }}
      >
        {/* Sticky Lane Labels (Y-Axis) */}
        <div style={{ 
          width: '120px', 
          flexShrink: 0, 
          borderRight: '1px solid var(--border-color)',
          background: 'var(--bg-panel)',
          zIndex: 10,
          display: 'flex',
          flexDirection: 'column'
        }}>
          <div style={{ height: '40px', borderBottom: '1px solid var(--border-color)' }}></div>
          <div style={{ flex: 1, display: 'flex', flexDirection: 'column', justifyContent: 'space-around', padding: '1rem 0.5rem', color: 'var(--text-muted)', fontSize: '0.75rem', textTransform: 'uppercase', fontWeight: 600 }}>
            <div style={{ flex: 1, display: 'flex', alignItems: 'center' }}>Market</div>
            <div style={{ flex: 1, display: 'flex', alignItems: 'center' }}>Signal</div>
            <div style={{ flex: 1, display: 'flex', alignItems: 'center' }}>Execution</div>
            <div style={{ flex: 1, display: 'flex', alignItems: 'center' }}>Portfolio</div>
          </div>
        </div>

        {/* Scrollable X-Axis Content */}
        <div 
          ref={scrollContainerRef}
          style={{ 
            flex: 1, 
            overflowX: 'auto', 
            overflowY: 'hidden',
            display: 'flex',
            flexDirection: 'column'
          }}
        >
          {/* Ticks Header */}
          <div style={{ display: 'flex', height: '40px', borderBottom: '1px solid var(--border-color)', minWidth: 'min-content' }}>
            {timeline.lanes.map((lane, index) => {
              const isAnchor = lane.tick === anchorTick;
              // X-Axis Compression based on precise frequency
              const showLabel = isAnchor || (index % labelFreq === 0);

              return (
                <div 
                  key={lane.tick}
                  style={{ 
                    width: `${currentTickWidth}px`, 
                    flexShrink: 0,
                    display: 'flex',
                    flexDirection: 'column',
                    alignItems: 'center',
                    justifyContent: 'center',
                    borderRight: '1px dashed var(--glass-border)',
                    fontSize: '0.75rem',
                    fontFamily: 'monospace',
                    color: isAnchor ? 'var(--status-critical)' : 'var(--text-muted)',
                    background: isAnchor ? 'rgba(239, 68, 68, 0.1)' : 'transparent',
                    fontWeight: isAnchor ? 600 : 400
                  }}
                >
                  {isAnchor && <span style={{ fontSize: '0.55rem', background: 'var(--status-critical)', color: '#fff', padding: '0 4px', borderRadius: '2px', marginBottom: '2px' }}>DIVERGENCE</span>}
                  {showLabel ? new Date(new Date('2025-01-17T09:30:00Z').getTime() + lane.tick * 1000).toISOString().substring(11, 19) : ''}
                </div>
              );
            })}
          </div>

          {/* Data Lanes Container */}
          <div style={{ flex: 1, display: 'flex', minWidth: 'min-content', padding: '1.5rem 0' }}>
            {timeline.lanes.map((lane) => {
              const isAnchor = lane.tick === anchorTick;
              const isPostAnchor = anchorTick !== null && lane.tick >= anchorTick;

              return (
                <div 
                  key={lane.tick}
                  style={{ 
                    width: `${currentTickWidth}px`, 
                    flexShrink: 0,
                    display: 'flex',
                    flexDirection: 'column',
                    justifyContent: 'space-around',
                    borderRight: '1px dashed var(--glass-border)',
                    background: isAnchor ? 'rgba(239, 68, 68, 0.05)' : 'transparent',
                    opacity: isPostAnchor || anchorTick === null ? 1 : 0.4,
                    alignItems: 'center'
                  }}
                >
                  {/* Market Lane */}
                  <div style={{ flex: 1, position: 'relative', display: 'flex', alignItems: 'center', justifyContent: 'center', width: '100%' }}>
                    <div style={{ position: 'absolute', top: '50%', left: 0, right: 0, height: '2px', background: 'var(--border-color)', zIndex: 1, opacity: 0.5 }}></div>
                    <div style={{ position: 'relative', zIndex: 2, background: 'var(--bg-main)', padding: '0 4px' }}>
                      {needsAggregation && !isAnchor ? (
                         <div style={{ width: '4px', height: '4px', borderRadius: '50%', background: 'var(--text-secondary)' }} />
                      ) : (
                        <span style={{ fontSize: '0.75rem', color: 'var(--text-primary)', fontWeight: 500 }}>{lane.market.price.toFixed(2)}</span>
                      )}
                    </div>
                  </div>

                  {/* Signal Lane */}
                  <div style={{ flex: 1, position: 'relative', display: 'flex', alignItems: 'center', justifyContent: 'center', width: '100%' }}>
                    <div style={{ position: 'absolute', top: '50%', left: 0, right: 0, height: '2px', background: 'var(--border-color)', zIndex: 1, opacity: 0.3 }}></div>
                    {lane.signal.intent ? (
                      <div style={{ position: 'relative', zIndex: 2, background: 'var(--bg-main)', padding: '0 4px' }}>
                        {needsAggregation && !isAnchor ? (
                          <div style={{ width: '6px', height: '6px', borderRadius: '50%', background: 'var(--accent-purple)' }} />
                        ) : (
                          <span className="badge badge-info" style={{ fontSize: '0.65rem', padding: '0.15rem 0.4rem', background: 'var(--bg-panel)' }}>{lane.signal.intent}</span>
                        )}
                      </div>
                    ) : null}
                  </div>

                  {/* Execution Lane */}
                  <div style={{ flex: 1, position: 'relative', display: 'flex', alignItems: 'center', justifyContent: 'center', width: '100%' }}>
                    <div style={{ position: 'absolute', top: '50%', left: 0, right: 0, height: '2px', background: 'var(--border-color)', zIndex: 1, opacity: 0.3 }}></div>
                    {(lane.execution.baseline_fill || lane.execution.perturbed_fill || lane.execution.missed_fill) ? (
                      <div style={{ position: 'relative', zIndex: 2, display: 'flex', flexDirection: 'column', gap: '2px', background: 'var(--bg-main)', padding: '4px' }}>
                        {lane.execution.baseline_fill && (
                          needsAggregation && !isAnchor ? (
                            <div style={{ width: '6px', height: '6px', background: 'var(--status-success)', margin: 'auto' }} />
                          ) : (
                            <span className="badge badge-success" style={{ fontSize: '0.55rem', padding: '0.1rem 0.3rem' }}>BL: FILLED</span>
                          )
                        )}
                        {lane.execution.perturbed_fill && (
                          needsAggregation && !isAnchor ? (
                            <div style={{ width: '6px', height: '6px', background: 'var(--status-warning)', margin: 'auto' }} />
                          ) : (
                            <span className="badge badge-warning" style={{ fontSize: '0.55rem', padding: '0.1rem 0.3rem' }}>PT: FILLED</span>
                          )
                        )}
                        {lane.execution.missed_fill && (
                          needsAggregation && !isAnchor ? (
                            <div style={{ width: '6px', height: '6px', background: 'var(--status-critical)', margin: 'auto' }} />
                          ) : (
                            <span className="badge badge-critical" style={{ fontSize: '0.55rem', padding: '0.1rem 0.3rem' }}>PT: MISSED</span>
                          )
                        )}
                      </div>
                    ) : null}
                  </div>

                  {/* Portfolio Lane */}
                  <div style={{ flex: 1, position: 'relative', display: 'flex', alignItems: 'center', justifyContent: 'center', width: '100%' }}>
                    <div style={{ position: 'absolute', top: '50%', left: 0, right: 0, height: '2px', background: 'var(--border-color)', zIndex: 1, opacity: 0.3 }}></div>
                    <div style={{ position: 'relative', zIndex: 2, display: 'flex', flexDirection: 'column', alignItems: 'center', background: 'var(--bg-main)', padding: '0 4px' }}>
                      {lane.portfolio.baseline_position !== lane.portfolio.perturbed_position ? (
                        needsAggregation && !isAnchor ? (
                          <div style={{ width: '6px', height: '12px', background: 'var(--status-critical)' }} />
                        ) : (
                          <>
                            <span style={{ color: 'var(--status-success)', fontSize: '0.65rem' }}>BL: {lane.portfolio.baseline_position}</span>
                            <span style={{ color: 'var(--status-critical)', fontSize: '0.65rem', fontWeight: 600 }}>PT: {lane.portfolio.perturbed_position}</span>
                          </>
                        )
                      ) : (
                        needsAggregation && !isAnchor ? (
                          <div style={{ width: '6px', height: '6px', borderRadius: '50%', background: 'var(--text-muted)' }} />
                        ) : (
                          <span style={{ color: 'var(--text-muted)', fontSize: '0.75rem' }}>{lane.portfolio.baseline_position}</span>
                        )
                      )}
                    </div>
                  </div>

                </div>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
};
