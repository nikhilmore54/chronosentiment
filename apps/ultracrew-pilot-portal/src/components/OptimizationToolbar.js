import { S } from "../styles";

import React from 'react';
import { GERAD_INSTANCES } from '../adapters/schedule';


export default function OptimizationToolbar({
  scenario,
  setScenario,
  generationLimit,
  setGenerationLimit,
  seed,
  setSeed,
  onBack,
  onRun,
  running,
  csrfToken,
  runError
}) {
  return (
    <div style={S.card}>
      <div style={S.cardTitle}>Step 2 — Run Optimizer</div>
      <div style={S.cardSub}>Select a scenario and run the UltraCrew optimizer.</div>
      
      <label style={S.label}>Scenario</label>
      <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', marginBottom: '16px' }}>
        {Object.keys(GERAD_INSTANCES).map(scId => {
          const sc = GERAD_INSTANCES[scId];
          return (
            <label key={scId} style={{ display: 'flex', alignItems: 'flex-start', gap: '10px', padding: '10px 12px', borderRadius: '6px', border: `1px solid ${scenario === scId ? '#3b82f6' : '#e2e8f0'}`, background: scenario === scId ? '#eff6ff' : '#fff', cursor: 'pointer' }}>
              <input type="radio" name="scenario" value={scId} checked={scenario === scId} onChange={() => setScenario(scId)} style={{ marginTop: '3px', accentColor: '#3b82f6' }} />
              <div>
                <div style={{ fontWeight: 600, fontSize: '13px', color: '#1e293b' }}>{sc.meta.source.split(' (')[0]}</div>
                <div style={{ fontSize: '12px', color: '#64748b', marginTop: '2px' }}>{sc.meta.total_crew} crew · {sc.meta.total_duties} duties · {sc.meta.horizon_hours}h horizon</div>
              </div>
            </label>
          );
        })}
      </div>
      
      <label style={S.label}>Generation limit</label>
      <input style={S.input} type="number" value={generationLimit} onChange={e => setGenerationLimit(parseInt(e.target.value) || 500)} min={10} max={2000} />
      
      <label style={S.label}>Random seed</label>
      <input style={S.input} type="number" value={seed} onChange={e => setSeed(parseInt(e.target.value) || 42)} />
      
      {runError && <div style={S.alert('error')}>{runError}</div>}
      
      <div style={{ display: 'flex', gap: '8px' }}>
        <button style={S.btn('secondary')} onClick={onBack}>← Back</button>
        <button 
          style={running || !csrfToken ? S.btnDisabled : S.btn('primary')} 
          disabled={running || !csrfToken} 
          onClick={onRun}
        >
          {running ? '⏳ Running optimizer…' : '▶ Run Optimizer'}
        </button>
      </div>
    </div>
  );
}
