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
                narratedExecutionTraceLength={narratedExecutionTrace.length}
                activeChain={activeChain}
                getGroupColorClass={getGroupColorClass}
                isBlockDivergent={isBlockDivergent}
                blockDivergenceMessage={getBlockDivergenceMessage(block.id)}
                setSelectedSeqId={setSelectedSeqId}
              />
            ))
          ) : (
            <p style={{ fontSize: 12, color: 'var(--tm)', padding: '12px 0' }}>
              No execution narrative available for Strategy {strategyNum}.
            </p>
          )}
        </div>

        {/* Causal chain panel */}
        {selectedSeqId && inspectionResult && (
          <div style={{ marginTop: 16 }}>
            <div className="cs-card-title">Causal Chain — Seq {selectedSeqId}</div>
            <div className="cs-causal-chain">
              {Array.from(activeChain)
                .sort((a, b) => b - a)
                .map((seqIdInChain, idx) => {
                  const block = eventMap[seqIdInChain];
                  if (!block) return null;
                  return (
                    <React.Fragment key={block.id}>
                      <div className="cs-causal-step">
                        <div className="cs-causal-step-group">{block.group} · Seq {block.id}</div>
                        <div className="cs-causal-step-text">{block.narrative}</div>
                      </div>
                      {idx < activeChain.size - 1 && (
                        <div className="cs-causal-arrow">↑</div>
                      )}
                    </React.Fragment>
                  );
                })}
            </div>
          </div>
        )}

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