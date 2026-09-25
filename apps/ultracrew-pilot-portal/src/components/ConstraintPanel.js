import React from 'react';
import { S } from '../styles';

export default function ConstraintPanel({ pairings, workersMap }) {
  if (!pairings || pairings.length === 0) return null;
  const hardViolations = pairings.filter(p => !p.rest_compliant || !p.fdp_compliant);
  
  if (hardViolations.length === 0) {
    return (
      <div style={S.alert('success')}>
        ✓ All {pairings.length} pairings satisfy rest and FDP requirements — schedule is legally dispatchable.
      </div>
    );
  }
  
  return (
    <div style={S.alert('error')}>
      ⚠ <strong>{hardViolations.length} of {pairings.length} pairings</strong> violate crew rest or FDP limits.
      Affected flights <strong>cannot depart</strong> until violations are resolved.
      {' '}Violations: {hardViolations.map((p, i) => {
        const fw = workersMap && workersMap[p.worker_id] ? `Worker W${p.worker_id}` : `Worker W${p.worker_id}`;
        const prob = !p.rest_compliant ? 'Insufficient rest' : 'Max duty exceeded';
        return <span key={i}><br/>• {fw}: {prob}</span>;
      })}
    </div>
  );
}
