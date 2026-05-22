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
    <div className="cs-gap-16">
      {/* Column header */}
      <div>
        <div className="cs-section-sub">Strategy {strategyNum}</div>
        <div className="cs-section-title" style={{ fontSize: 13 }}>
          {strategyId || 'N/A'}
        </div>
      </div>

      {/* Decision context */}
      <div className="cs-card">
        <div className="cs-card-title">Decision Context</div>
        <div className="cs-gap-4">
          <div className="cs-row">
            <span className="cs-row-key">Strategy ID</span>
            <span className="cs-row-val" style={{ fontSize: 11, maxWidth: 200, overflow: 'hidden', textOverflow: 'ellipsis' }}>
              {inspectionResult?.strategy_id ?? '—'}
            </span>
          </div>
          <div className="cs-row">
            <span className="cs-row-key">Seed</span>
            <span className="cs-row-val">{seed}</span>
          </div>
        </div>
        {inspectionResult?.metrics && (
          <div style={{ marginTop: 12 }}>
            <div className="cs-label" style={{ marginBottom: 6 }}>Metrics</div>
            <pre className="cs-pre" style={{ maxHeight: 160 }}>
              {JSON.stringify(inspectionResult.metrics, null, 2)}
            </pre>
          </div>
        )}
      </div>

      {/* Execution narrative */}
      <div className="cs-card">
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