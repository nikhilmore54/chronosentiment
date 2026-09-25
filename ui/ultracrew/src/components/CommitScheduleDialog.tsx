// CommitScheduleDialog.tsx
import React from 'react';

interface Decision {
  caseId: string;
  recommendationId: string;
  selectedAction: string;
  decisionMaker: string;
  timestamp: string;
  confidence?: number;
  expectedImpact?: string;
  scheduleVersion?: string;
}

interface CommitScheduleDialogProps {
  open?: boolean;
  onClose?: () => void;
  decisions?: Decision[];
  onCommit?: (decisions: Decision[]) => void;
}

export const CommitScheduleDialog: React.FC<CommitScheduleDialogProps> = ({
  open = false,
  onClose,
  decisions = [],
  onCommit,
}) => {
  if (!open) return null;

  const handleConfirm = () => {
    if (onCommit) {
      onCommit(decisions);
    }
    if (onClose) onClose();
  };

  return (
    <div
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: 'rgba(0,0,0,0.5)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 1000,
      }}
    >
      <div style={{ backgroundColor: 'white', padding: '2rem', borderRadius: '8px', maxWidth: '400px', width: '90%' }}>
        <h3>Commit Schedule</h3>
        <p>
          You are about to commit {decisions.length} decision{decisions.length !== 1 ? 's' : ''}.
        </p>
        <div style={{ display: 'flex', gap: '1rem', justifyContent: 'flex-end' }}>
          <button onClick={onClose} style={{ padding: '0.4rem 0.8rem' }}>
            Cancel
          </button>
          <button
            onClick={handleConfirm}
            style={{ backgroundColor: 'var(--primary-color)', color: 'white', border: 'none', padding: '0.4rem 0.8rem', borderRadius: '4px', cursor: 'pointer' }}
          >
            Confirm
          </button>
        </div>
      </div>
    </div>
  );
};
