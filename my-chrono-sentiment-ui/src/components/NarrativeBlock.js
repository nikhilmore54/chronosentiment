import React from 'react';

// divergenceTypeLabel — display-only label for divergence type.
// Types are emitted by compareNarrativeBlocks (ARTIFACT-010); labels are projection-only.
const DIVERGENCE_TYPE_LABELS = {
  group_type_divergence:        { label: 'GROUP DIVERGENCE',    color: 'var(--red)' },
  narrative_content_divergence: { label: 'NARRATIVE DIVERGENCE', color: 'var(--amb)' },
  sequence_id_timing_divergence:{ label: 'TIMING DIVERGENCE',   color: 'var(--amb)' },
  causal_parent_divergence:     { label: 'CAUSAL DIVERGENCE',   color: 'var(--red)' },
  missing_s1:                   { label: 'MISSING IN S1',       color: 'var(--red)' },
  missing_s2:                   { label: 'MISSING IN S2',       color: 'var(--red)' },
};

const NarrativeBlock = ({
  block,
  blockIndex,
  narratedExecutionTrace,
  narratedExecutionTraceLength,
  activeChain,
  getGroupColorClass,
  isBlockDivergent,
  blockDivergenceMessage,
  blockDivergenceType,
  setSelectedSeqId,
}) => {
  // Next block in the trace — used for transition-aware arrow rendering
  const nextBlock = narratedExecutionTrace ? narratedExecutionTrace[blockIndex + 1] : null;
  const isActive = activeChain.has(block.id);
  const isDivergent = isBlockDivergent(block.id);
  const isDimmed = activeChain.size > 0 && !isActive;
  // getGroupColorClass returns a cs- modifier: 'intent' | 'queue' | 'execution' | 'other'
  const groupMod = getGroupColorClass(block.group);

  const divTypeInfo = blockDivergenceType ? (DIVERGENCE_TYPE_LABELS[blockDivergenceType] ?? { label: 'DIVERGENCE', color: 'var(--red)' }) : null;

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
        {/* Block header row: group label + key event badge + seq id + timestamp */}
        <div className="cs-trace-group" style={{ display: 'flex', alignItems: 'center', gap: '6px', flexWrap: 'wrap' }}>
          <span>{block.group}</span>
          {block.isKeyEvent && (
            <span style={{
              display: 'inline-flex', alignItems: 'center', gap: '3px',
              background: 'var(--pur)', color: '#fff',
              fontSize: '9px', fontWeight: 700, letterSpacing: '0.06em',
              padding: '1px 5px', borderRadius: 'var(--r4)',
            }}>
              ★ {block.keyEventMarker || 'KEY'}
            </span>
          )}
          <span style={{ color: 'var(--tm)', fontWeight: 400, fontFamily: 'var(--mono)', fontSize: '10px' }}>
            seq:{block.id}
          </span>
          {block.timestamp_ns != null && (
            <span style={{ color: 'var(--tm)', fontWeight: 400, fontFamily: 'var(--mono)', fontSize: '10px', marginLeft: 'auto' }}>
              t:{block.timestamp_ns}
            </span>
          )}
        </div>

        {/* Narrative text */}
        <div className="cs-trace-narrative">{block.narrative}</div>

        {/* Causal parent reference */}
        {block.parentId !== undefined && block.parentId !== null && (
          <div className="cs-trace-key-marker">
            ↳ derived from seq:{block.parentId}
          </div>
        )}

        {/* Divergence indicator with type label */}
        {isDivergent && blockDivergenceMessage && (
          <div style={{ marginTop: '6px', display: 'flex', alignItems: 'flex-start', gap: '6px' }}>
            {divTypeInfo && (
              <span style={{
                flexShrink: 0, fontSize: '9px', fontWeight: 700,
                color: divTypeInfo.color, fontFamily: 'var(--mono)',
                letterSpacing: '0.05em', padding: '1px 5px',
                background: 'rgba(0,0,0,0.12)', borderRadius: 'var(--r4)',
                border: `1px solid ${divTypeInfo.color}`,
              }}>
                {divTypeInfo.label}
              </span>
            )}
            <span className="cs-trace-key-marker" style={{ color: divTypeInfo?.color ?? 'var(--red)', margin: 0 }}>
              {blockDivergenceMessage}
            </span>
          </div>
        )}
      </div>
      {blockIndex < narratedExecutionTraceLength - 1 && (() => {
        const isGroupTransition = nextBlock && nextBlock.group !== block.group;
        const isNextDivergent   = nextBlock && isBlockDivergent(nextBlock.id);
        const arrowColor = isNextDivergent ? 'var(--red)'
                         : isGroupTransition ? 'var(--blu)'
                         : 'var(--tm)';
        const arrowLabel = isGroupTransition ? `→ ${nextBlock.group}` : null;
        return (
          <div className="cs-causal-arrow" style={{ color: arrowColor, display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '4px' }}>
            <span>↓</span>
            {arrowLabel && (
              <span style={{ fontSize: '9px', fontFamily: 'var(--mono)', letterSpacing: '0.04em', opacity: 0.8 }}>{arrowLabel}</span>
            )}
            {isNextDivergent && (
              <span style={{ fontSize: '9px', fontFamily: 'var(--mono)', fontWeight: 700, letterSpacing: '0.04em' }}>⚑</span>
            )}
          </div>
        );
      })()}
    </React.Fragment>
  );
};

export default NarrativeBlock;