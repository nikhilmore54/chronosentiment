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
    if (strategyNum === 1) {
      return divergenceStatements.some(s => (s.block1 && s.block1.id === id));
    } else {
      return divergenceStatements.some(s => (s.block2 && s.block2.id === id));
    }
  };

  const getBlockDivergenceMessage = (blockId) => {
    if (strategyNum === 1) {
      return divergenceStatements.find(s => s.block1 && s.block1.id === blockId)?.message;
    } else {
      return divergenceStatements.find(s => s.block2 && s.block2.id === blockId)?.message;
    }
  };

  return (
    <div>
      <h3 className="text-xl font-bold text-gray-800 mb-4">Strategy {strategyNum} ({strategyId || 'N/A'})</h3>
      <div className="border p-4 rounded-lg shadow-sm mb-6">
        <h3 className="text-xl font-semibold mb-2">Decision Context</h3>
        <p><span className="font-medium">Strategy ID:</span> {inspectionResult?.strategy_id}</p>
        <p><span className="font-medium">Seed:</span> {seed}</p>
        <div className="mt-2">
          <p className="font-medium">Metrics:</p>
          <pre className="bg-gray-50 p-2 rounded-md text-sm text-gray-800">{JSON.stringify(inspectionResult?.metrics, null, 2)}</pre>
        </div>
      </div>

      <div className="border p-4 rounded-lg shadow-sm mb-6">
        <h3 className="text-xl font-semibold mb-2">Execution Narrative</h3>

        {setSelectedSeqId && (
          <button
            className="text-sm text-gray-500 hover:text-gray-700 mb-2 underline"
            onClick={() => setSelectedSeqId(null)}
          >
            Clear Causal Chain Selection
          </button>
        )}

        <div className="bg-gray-50 p-4 rounded-md max-h-60 overflow-y-auto space-y-4">
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
            <p>No execution narrative available for Strategy {strategyNum}.</p>
          )}
        </div>

        {selectedSeqId && inspectionResult && (
          <div className="mt-6 border p-4 rounded-lg shadow-sm bg-gray-50">
            <h3 className="text-xl font-semibold mb-3">Causal Chain for Seq {selectedSeqId}</h3>
            {setSelectedSeqId && (
              <button
                className="text-sm text-gray-500 hover:text-gray-700 mb-2 underline"
                onClick={() => setSelectedSeqId(null)}
              >
                Clear Causal Chain Selection
              </button>
            )}
            <div className="space-y-2 mt-2">
              {Array.from(activeChain)
                .sort((a, b) => b - a)
                .map((seqIdInChain, idx) => {
                  const block = eventMap[seqIdInChain];
                  if (!block) return null;

                  return (
                    <div key={block.id} className={`border-l-2 pl-3 ${getGroupColorClass(block.group)} pt-1 pb-1`}>
                      <p className={`font-semibold text-blue-700 mb-1`}>{block.group} (Seq: {block.id})</p>
                      <p className="text-sm text-gray-800">{block.narrative}</p>
                      {idx < activeChain.size - 1 && (
                        <div className="text-gray-400 text-sm ml-2 mt-1">↑</div>
                      )}
                    </div>
                  );
                })}
            </div>
          </div>
        )}

        {showRawEvents && inspectionResult?.execution_trace?.length > 0 && (
          <div className="mt-4 p-4 bg-gray-700 text-white rounded-md">
            <h4 className="text-lg font-semibold mb-2">Raw Execution Trace JSON (Strategy {strategyNum})</h4>
            <div className="text-sm overflow-x-auto space-y-2">
              {inspectionResult.execution_trace.map((event, index) => (
                <pre
                  key={event.sequence_id || index}
                  ref={el => { if (el) rawEventRefs.current[event.sequence_id] = el; }}
                  className={`
                    p-2 rounded-md transition-colors duration-200
                    ${activeChain.has(event.sequence_id) ? 'bg-blue-600 text-white' : 'bg-gray-600 text-gray-100'}
                  `}
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