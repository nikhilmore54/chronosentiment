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
    <div className="card">
      <h2>Nurse Roster</h2>
      <div className="nurse-list">
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
