import React, { useState } from 'react';
import type { SimulationState } from './App';

export interface NurseBalance {
  nurse_id: string;
  balance: number;
  explanation: string[];
}

interface TeamBalanceProps {
  balances: NurseBalance[];
  simulationState?: SimulationState | null;
  balanceScore?: number;
}

export const TeamBalance: React.FC<TeamBalanceProps> = ({ balances, simulationState, balanceScore = 92 }) => {
  const [selectedNurse, setSelectedNurse] = useState<NurseBalance | null>(null);
  const [showFull, setShowFull] = useState(false);

  const activeBalances = balances;
  const surplusCount = activeBalances.filter(b => b.balance > 0).length;
  const deficitCount = activeBalances.filter(b => b.balance < 0).length;

  const anomalies = balances.filter(b => {
    if (simulationState) {
      return b.nurse_id === simulationState.affected_nurse || Object.keys(simulationState.creditors).includes(b.nurse_id);
    }
    return b.balance !== 0;
  });

  const displayList = showFull ? balances : anomalies;

  return (
    <div className="card" style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
      <div>
        <h2>Team Balance</h2>
        <div style={{ display: 'flex', gap: '1rem', marginBottom: '1rem' }}>
          <div style={{ padding: '1rem', backgroundColor: 'var(--bg-color)', borderRadius: '8px', flex: 1 }}>
            <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>Staff Behind Schedule</div>
            <div style={{ fontSize: '1.5rem', fontWeight: 600, color: 'var(--danger-color)' }}>{deficitCount}</div>
          </div>
          <div style={{ padding: '1rem', backgroundColor: 'var(--bg-color)', borderRadius: '8px', flex: 1 }}>
            <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>Staff Ahead of Schedule</div>
            <div style={{ fontSize: '1.5rem', fontWeight: 600, color: 'var(--success-color)' }}>{surplusCount}</div>
          </div>
          <div style={{ padding: '1rem', backgroundColor: 'var(--bg-color)', borderRadius: '8px', flex: 1 }}>
            <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>Balance Score</div>
            <div style={{ fontSize: '1.5rem', fontWeight: 600, color: 'var(--accent-color)' }}>{balanceScore}</div>
          </div>
        </div>
      </div>

      <div style={{ display: 'flex', gap: '2rem' }}>
        <div style={{ flex: 1, backgroundColor: 'var(--bg-color)', borderRadius: '8px', border: '1px solid var(--border-color)', padding: '1rem' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
            <h3 style={{ margin: 0, fontSize: '1rem', color: 'var(--text-main)' }}>
              {showFull ? 'Full Team Balance' : (simulationState ? 'Recovery Participants' : 'Needs Attention')}
            </h3>
            <button 
              onClick={() => setShowFull(!showFull)}
              style={{ background: 'none', border: 'none', color: 'var(--accent-color)', cursor: 'pointer', fontSize: '0.85rem', fontWeight: 600 }}
            >
              {showFull ? 'Show Anomalies Only' : 'View Full Team Balance'}
            </button>
          </div>
          <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left' }}>
            <thead>
              <tr style={{ borderBottom: '1px solid var(--border-color)', color: 'var(--text-muted)', fontSize: '0.85rem' }}>
                <th style={{ paddingBottom: '0.5rem' }}>Employee</th>
                <th style={{ paddingBottom: '0.5rem' }}>{simulationState ? 'Status' : 'Current Balance'}</th>
                {simulationState && <th style={{ paddingBottom: '0.5rem' }}>Today</th>}
                {simulationState && <th style={{ paddingBottom: '0.5rem' }}>Expected</th>}
              </tr>
            </thead>
            <tbody>
              {displayList.map(b => {
                let status = 'Balanced';
                let todayVal = b.balance;
                let expectedVal: string | number = b.balance;
                
                if (simulationState) {
                  if (b.nurse_id === simulationState.affected_nurse) {
                    status = 'Recovering';
                    todayVal = b.balance - simulationState.missed_shifts;
                    expectedVal = 'Balanced';
                  } else if (Object.keys(simulationState.creditors).includes(b.nurse_id)) {
                    status = 'Returning Time';
                    todayVal = b.balance + simulationState.creditors[b.nurse_id];
                    expectedVal = 'Balanced';
                  }
                }
                
                return (
                  <tr 
                    key={b.nurse_id} 
                    style={{ 
                      borderBottom: '1px solid var(--border-color)', 
                      cursor: 'pointer',
                      backgroundColor: selectedNurse?.nurse_id === b.nurse_id ? 'rgba(37, 99, 235, 0.05)' : 'transparent'
                    }}
                    onClick={() => setSelectedNurse(b)}
                  >
                    <td style={{ padding: '0.75rem 0', fontWeight: 500 }}>{b.nurse_id}</td>
                    
                    {!simulationState ? (
                      <td style={{ 
                        padding: '0.75rem 0', 
                        fontWeight: 600, 
                        color: b.balance < 0 ? 'var(--danger-color)' : (b.balance > 0 ? 'var(--success-color)' : 'inherit')
                      }}>
                        {b.balance > 0 ? '+' : ''}{b.balance} shifts
                      </td>
                    ) : (
                      <td style={{ 
                        padding: '0.75rem 0', 
                        fontWeight: 600,
                        color: status === 'Recovering' ? 'var(--danger-color)' : (status === 'Returning Time' ? 'var(--accent-color)' : 'var(--success-color)')
                      }}>
                        {status}
                      </td>
                    )}
                    
                    {simulationState && (
                      <td style={{ padding: '0.75rem 0', fontWeight: 600 }}>
                        {todayVal > 0 ? '+' : ''}{todayVal}
                      </td>
                    )}
                    {simulationState && (
                      <td style={{ padding: '0.75rem 0', fontWeight: 600, color: expectedVal === 'Balanced' ? 'var(--success-color)' : 'inherit' }}>
                        {expectedVal}
                      </td>
                    )}
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>

        {selectedNurse && (
          <div style={{ 
            position: 'fixed', 
            bottom: '2rem', 
            right: '2rem', 
            width: '400px',
            backgroundColor: 'var(--bg-color)', 
            padding: '1.5rem', 
            borderRadius: '12px', 
            border: '1px solid var(--border-color)', 
            boxShadow: '0 20px 25px -5px rgba(0, 0, 0, 0.5), 0 10px 10px -5px rgba(0, 0, 0, 0.3)',
            zIndex: 1000
          }}>
            <button 
              onClick={() => setSelectedNurse(null)}
              style={{ position: 'absolute', top: '1rem', right: '1rem', background: 'none', border: 'none', color: 'var(--text-muted)', cursor: 'pointer', fontSize: '1.2rem', padding: '0.25rem' }}
              title="Close"
            >
              ×
            </button>
            <h3 style={{ marginTop: 0, color: 'var(--text-main)', fontSize: '1.1rem' }}>{selectedNurse.nurse_id}</h3>
            <div style={{ 
              fontSize: '1.25rem', 
              fontWeight: 600, 
              marginBottom: '1.5rem',
              color: selectedNurse.balance < 0 ? 'var(--danger-color)' : (selectedNurse.balance > 0 ? 'var(--success-color)' : 'inherit')
            }}>
              Active Balance: {selectedNurse.balance > 0 ? '+' : ''}{selectedNurse.balance} shifts
            </div>
            
            <h4 style={{ color: 'var(--text-muted)', fontSize: '0.85rem', textTransform: 'uppercase', letterSpacing: '0.05em' }}>Reason for Prioritization:</h4>
            <ul style={{ paddingLeft: '1.2rem', margin: '0.5rem 0 0 0', color: 'var(--text-main)', display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
              {selectedNurse.explanation.map((exp, idx) => (
                <li key={idx}>{exp}</li>
              ))}
            </ul>
          </div>
        )}
      </div>
    </div>
  );
};
