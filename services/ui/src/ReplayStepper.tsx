import React from 'react';

interface ReplayStepperProps {
  currentSequenceId: number;
  totalEvents: number;
  onNext: () => void;
  onPrevious: () => void;
  onReset: () => void;
}

const ReplayStepper: React.FC<ReplayStepperProps> = ({
  currentSequenceId,
  totalEvents,
  onNext,
  onPrevious,
  onReset,
}) => {
  return (
    <div style={{ border: '1px solid #ccc', padding: '15px', borderRadius: '5px', marginBottom: '20px' }}>
      <h3>Replay Controls</h3>
      <div style={{ marginBottom: '10px' }}>
        Current Sequence: {currentSequenceId} / {totalEvents > 0 ? totalEvents - 1 : 0}
      </div>
      <div style={{ display: 'flex', gap: '10px' }}>
        <button onClick={onReset} disabled={currentSequenceId === 0}>Reset</button>
        <button onClick={onPrevious} disabled={currentSequenceId === 0}>← Previous Event</button>
        <button onClick={onNext} disabled={currentSequenceId >= totalEvents - 1}>Next Event →</button>
      </div>
    </div>
  );
};

export default ReplayStepper;
