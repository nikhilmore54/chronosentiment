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
  const isBlockDivergentHere = isBlockDivergent(block.id);

  return (
    <React.Fragment>
      <div
        className={`
          border-l-2 pl-3 pt-1 pb-1 cursor-pointer
          ${activeChain.has(block.id)
            ? `${getGroupColorClass(block.group)} border-l-4 border-blue-600 shadow-md`
            : `${getGroupColorClass(block.group)} ${activeChain.size > 0 ? 'opacity-50 border-gray-300' : 'border-gray-300'}`}
          ${isBlockDivergentHere ? 'bg-red-100 border-red-500' : ''}
          hover:bg-gray-200
          transition-all duration-200
        `}
        onClick={() => setSelectedSeqId(block.id)}
      >
        {block.isKeyEvent && (
          <span className="text-purple-600 font-bold mr-2">{block.keyEventMarker}</span>
        )}
        <p className={`font-semibold mb-1 ${activeChain.has(block.id) ? 'text-blue-800' : getGroupColorClass(block.group).split(' ')[1]}`}>
          {block.group} (Seq: {block.id})
        </p>
        <p className="text-sm text-gray-800">{block.narrative}</p>
        {block.parentId !== undefined && block.parentId !== null && (
          <p className={`text-xs text-gray-500 mt-1 ${activeChain.has(block.id) ? 'font-medium' : ''}`}>
            Derived from Seq {block.parentId}
          </p>
        )}
        {isBlockDivergentHere && blockDivergenceMessage && (
          <p className="text-xs text-red-700 mt-1">⚠ {blockDivergenceMessage}</p>
        )}
      </div>
      {blockIndex < narratedExecutionTraceLength - 1 && (
        <div className="text-gray-400 text-sm ml-2">↓</div>
      )}
    </React.Fragment>
  );
};

export default NarrativeBlock;