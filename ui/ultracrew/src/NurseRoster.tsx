import React from 'react';

export interface Nurse {
  id: string;
  contract: string;
  skills: string[];
}

interface NurseRosterProps {
  nurses: Nurse[];
}

export const NurseRoster: React.FC<NurseRosterProps> = ({ nurses }) => {
  return (
    <div className="card" style={{ overflow: 'hidden' }}>
      <h2 style={{ background: 'var(--bg-panel)', zIndex: 10, margin: 0, padding: '0.5rem 0' }}>Nurse Roster</h2>
      <div className="nurse-list" style={{ maxHeight: '300px', overflowY: 'auto' }}>
        {nurses.map(nurse => (
          <div key={nurse.id} className="nurse-item">
            <div className="nurse-info">
              <span className="nurse-name">{nurse.id}</span>
              <span className="nurse-skills">{nurse.skills.join(', ')}</span>
            </div>
            <span className="nurse-contract">{nurse.contract}</span>
          </div>
        ))}
      </div>
    </div>
  );
};
