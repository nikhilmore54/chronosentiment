import React from 'react';
import NarrativeBlock from './NarrativeBlock';

const StrategyColumn = ({
  strategyNum,
  strategyId,
  seed,
  inspectionResult,
  narratedExecutionTrace,
  rawEventRefs,
  activeChain,
  eventMap,
  showRawEvents,
  getGroupColorClass,
  divergenceStatements,
  setSelectedSeqId,
  selectedSeqId,
}) => {
  const isBlockDivergent = (id) => {
    if (strategyNum === 1) return divergenceStatements.some(s => s.block1 && s.block1.id === id);
    return divergenceStatements.some(s => s.block2 && s.block2.id === id);
  };

  const getBlockDivergenceMessage = (blockId) => {
    if (strategyNum === 1) return divergenceStatements.find(s => s.block1 && s.block1.id === blockId)?.message;
    return divergenceStatements.find(s => s.block2 && s.block2.id === blockId)?.message;
  };

  // Returns the divergence type string for a given block id, used by NarrativeBlock
  // to render a type-specific label (GROUP DIVERGENCE, TIMING DIVERGENCE, etc.)
  const getBlockDivergenceType = (blockId) => {
    if (strategyNum === 1) return divergenceStatements.find(s => s.block1 && s.block1.id === blockId)?.type;
    return divergenceStatements.find(s => s.block2 && s.block2.id === blockId)?.type;
  };

  // Forward propagation: compute direct children of a given block id from eventMap.
  // Pure derivation from backend-certified parentId fields — no synthesis.
  const getForwardChildren = (blockId) =>
    Object.values(eventMap).filter(b => b.parentId === blockId);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '40px' }}>
      {/* Column header & Context */}
      <div>
        <div style={{ fontSize: '12px', fontWeight: 500, color: 'var(--t2)', marginBottom: '8px' }}>Strategy {strategyNum} context</div>
        <div style={{ fontSize: '18px', fontWeight: 600, color: 'var(--t1)', marginBottom: '16px' }}>
          {strategyId || 'N/A'}
        </div>
        
        <div style={{ display: 'flex', gap: '32px', borderBottom: '1px solid var(--b)', paddingBottom: '16px', fontSize: '13px', color: 'var(--t1)', fontFamily: 'var(--mono)' }}>
          <div><span style={{ color: 'var(--tm)', fontSize: '11px', display: 'block', marginBottom: '4px', fontFamily: 'var(--sans)' }}>Strategy ID</span>{inspectionResult?.strategy_id ?? '—'}</div>
          <div><span style={{ color: 'var(--tm)', fontSize: '11px', display: 'block', marginBottom: '4px', fontFamily: 'var(--sans)' }}>Seed</span>{seed}</div>
          {inspectionResult?.metrics?.total_trades !== undefined && (
            <div><span style={{ color: 'var(--tm)', fontSize: '11px', display: 'block', marginBottom: '4px', fontFamily: 'var(--sans)' }}>Total trades</span>{inspectionResult.metrics.total_trades}</div>
          )}
        </div>
      </div>

      {/* Execution narrative stream */}
      <div>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 12 }}>
          <div className="cs-card-title" style={{ marginBottom: 0 }}>Execution Narrative</div>
          {setSelectedSeqId && selectedSeqId && (
            <button
              className="cs-btn"
              style={{ padding: '4px 10px', fontSize: 10 }}
              onClick={() => setSelectedSeqId(null)}
            >
              Clear selection
            </button>
          )}
        </div>

        <div className="cs-trace-list">
          {narratedExecutionTrace?.length > 0 ? (
            narratedExecutionTrace.map((block, blockIndex) => (
              <NarrativeBlock
                key={block.id || blockIndex}
                block={block}
                blockIndex={blockIndex}
                narratedExecutionTrace={narratedExecutionTrace}
                narratedExecutionTraceLength={narratedExecutionTrace.length}
                activeChain={activeChain}
                getGroupColorClass={getGroupColorClass}
                isBlockDivergent={isBlockDivergent}
                blockDivergenceMessage={getBlockDivergenceMessage(block.id)}
                blockDivergenceType={getBlockDivergenceType(block.id)}
                setSelectedSeqId={setSelectedSeqId}
              />
            ))
          ) : (
            <div className="cs-empty" style={{ padding: '32px 16px' }}>
              <div className="cs-empty-icon">∅</div>
              <div className="cs-empty-title">No narrative events</div>
              <div style={{ fontSize: '11px', color: 'var(--tm)', marginTop: '4px' }}>
                Strategy {strategyNum} produced no certified narrative blocks at this replay position.
              </div>
            </div>
          )}
        </div>

        {/* Causal chain panel */}
        {selectedSeqId && inspectionResult && (() => {
          const forwardChildren = getForwardChildren(selectedSeqId);
          return (
            <div style={{ marginTop: 20, padding: '16px', background: 'var(--card2)', border: '1px solid var(--b)', borderRadius: 'var(--r8)' }}>
              {/* Panel header with depth counter */}
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px' }}>
                <div className="cs-card-title" style={{ marginBottom: 0 }}>Causal ancestry</div>
                <div style={{ display: 'flex', gap: '12px', alignItems: 'center' }}>
                  <span style={{ fontSize: '10px', color: 'var(--tm)', fontFamily: 'var(--mono)' }}>
                    depth {activeChain.size}
                  </span>
                  <span style={{ fontSize: '10px', color: 'var(--blu)', fontFamily: 'var(--mono)', padding: '2px 6px', background: 'var(--bdim)', borderRadius: 'var(--r4)' }}>
                    Seq {selectedSeqId}
                  </span>
                </div>
              </div>
              {/* Ancestry path label */}
              <div style={{ fontSize: '10px', color: 'var(--tm)', fontFamily: 'var(--mono)', marginBottom: '12px', letterSpacing: '0.03em' }}>
                {Array.from(activeChain).sort((a, b) => a - b).join(' → ')}
              </div>
              <div className="cs-causal-chain">
                {Array.from(activeChain)
                  .sort((a, b) => b - a)
                  .map((seqIdInChain, idx) => {
                    const b = eventMap[seqIdInChain];
                    if (!b) return null;
                    return (
                      <React.Fragment key={b.id}>
                        <div className="cs-causal-step">
                          <div className="cs-causal-step-group">{b.group} · Seq {b.id}</div>
                          <div className="cs-causal-step-text">{b.narrative}</div>
                        </div>
                        {idx < activeChain.size - 1 && (
                          <div className="cs-causal-arrow">↑</div>
                        )}
                      </React.Fragment>
                    );
                  })}
              </div>

              {/* Forward propagation: children of selected block */}
              {forwardChildren.length > 0 && (
                <div style={{ marginTop: '16px', paddingTop: '12px', borderTop: '1px solid var(--b)' }}>
                  <div style={{ fontSize: '10px', fontWeight: 600, color: 'var(--tm)', marginBottom: '8px', textTransform: 'uppercase', letterSpacing: '0.06em' }}>
                    Propagates to ({forwardChildren.length})
                  </div>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
                    {forwardChildren.map(child => (
                      <div
                        key={child.id}
                        style={{ display: 'flex', alignItems: 'center', gap: '8px', cursor: 'pointer', padding: '6px 8px', background: 'var(--card)', border: '1px solid var(--b)', borderRadius: 'var(--r4)' }}
                        onClick={() => setSelectedSeqId(child.id)}
                      >
                        <span style={{ fontSize: '10px', color: 'var(--tm)', fontFamily: 'var(--mono)' }}>↓</span>
                        <span style={{ fontSize: '11px', color: 'var(--t2)', fontWeight: 500 }}>{child.group}</span>
                        <span style={{ fontSize: '10px', color: 'var(--tm)', fontFamily: 'var(--mono)' }}>Seq {child.id}</span>
                        {isBlockDivergent(child.id) && (
                          <span style={{ fontSize: '9px', color: 'var(--red)', fontFamily: 'var(--mono)', fontWeight: 700, marginLeft: 'auto' }}>⚑ DIVERGENT</span>
                        )}
                      </div>
                    ))}
                  </div>
                </div>
              )}
              {forwardChildren.length === 0 && (
                <div style={{ marginTop: '12px', paddingTop: '10px', borderTop: '1px solid var(--b)', fontSize: '10px', color: 'var(--tm)', fontFamily: 'var(--mono)' }}>
                  No downstream events — terminal node
                </div>
              )}
            </div>
          );
        })()}

        {/* Raw events */}
        {showRawEvents && inspectionResult?.execution_trace?.length > 0 && (
          <div style={{ marginTop: 16 }}>
            <div className="cs-card-title">Raw Execution Trace — Strategy {strategyNum}</div>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 4, maxHeight: 320, overflowY: 'auto' }}>
              {inspectionResult.execution_trace.map((event, index) => (
                <pre
                  key={event.sequence_id || index}
                  ref={el => { if (el) rawEventRefs.current[event.sequence_id] = el; }}
                  className="cs-pre"
                  style={{
                    background: activeChain.has(event.sequence_id) ? 'var(--bdim)' : 'var(--card2)',
                    borderColor: activeChain.has(event.sequence_id) ? 'rgba(59,130,246,.4)' : 'var(--b)',
                    maxHeight: 'none',
                    padding: '8px 12px',
                  }}
                >
                  {JSON.stringify(event, null, 2)}
                </pre>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default StrategyColumn;