import React from 'react';

const NarrativeBlock = ({
  block,
  blockIndex,
  narratedExecutionTraceLength,
  activeChain,
  getGroupColorClass,
  isBlockDivergent,
  blockDivergenceMessage,
  setSelectedSeqId,
}) => {
  const isActive = activeChain.has(block.id);
  const isDivergent = isBlockDivergent(block.id);
  const isDimmed = activeChain.size > 0 && !isActive;
  // getGroupColorClass returns a cs- modifier: 'intent' | 'queue' | 'execution' | 'other'
  const groupMod = getGroupColorClass(block.group);

  const blockClass = [
    'cs-trace-block',
    groupMod,
    isActive    ? 'active'    : '',
    isDivergent ? 'divergent' : '',
    isDimmed    ? 'dimmed'    : '',
  ].filter(Boolean).join(' ');

  return (
    <React.Fragment>
      <div className={blockClass} onClick={() => setSelectedSeqId(block.id)}>
        <div className="cs-trace-group">
          {block.group}
          {block.isKeyEvent && (
            <span style={{ color: 'var(--pur)', marginLeft: '6px' }}>{block.keyEventMarker}</span>
          )}
          <span style={{ color: 'var(--tm)', marginLeft: '6px', fontWeight: 400 }}>
            seq:{block.id}
          </span>
        </div>
        <div className="cs-trace-narrative">{block.narrative}</div>
        {block.parentId !== undefined && block.parentId !== null && (
          <div className="cs-trace-key-marker">
            ↳ derived from seq:{block.parentId}
          </div>
        )}
        {isDivergent && blockDivergenceMessage && (
          <div className="cs-trace-key-marker" style={{ color: 'var(--red)' }}>
            ⚑ {blockDivergenceMessage}
          </div>
        )}
      </div>
      {blockIndex < narratedExecutionTraceLength - 1 && (
        <div className="cs-causal-arrow">↓</div>
      )}
    </React.Fragment>
  );
};

export default NarrativeBlock;