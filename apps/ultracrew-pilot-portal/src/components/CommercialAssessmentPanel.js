import React from 'react';
import { S } from '../styles';

export default function CommercialAssessmentPanel({
  orgName, setOrgName,
  baselineSchedulingMins, setBaselineSchedulingMins,
  baselineDisruptionMins, setBaselineDisruptionMins,
  productGaps, setProductGaps,
  willingToPilot, setWillingToPilot, WILLING_TO_PILOT_OPTIONS,
  nextSteps, setNextSteps, NEXT_STEPS_OPTIONS
}) {
  return (
    <>
      <div style={{ fontSize: '14px', fontWeight: '600', color: '#f1f5f9', marginBottom: '4px' }}>Commercial Evidence</div>
      <div style={{ fontSize: '13px', color: '#64748b', marginBottom: '16px' }}>These fields help us build the business case. All optional.</div>

      <label style={S.label}>Organisation name</label>
      <input style={S.input} value={orgName} onChange={e => setOrgName(e.target.value)} placeholder="e.g. SunAir, IndiGo, Air India Express" />

      <label style={S.label}>How long does manual scheduling take today? (minutes)</label>
      <input style={S.input} type="number" min={0} value={baselineSchedulingMins} onChange={e => setBaselineSchedulingMins(e.target.value)} placeholder="e.g. 120" />

      <label style={S.label}>How long does disruption recovery take today? (minutes)</label>
      <input style={S.input} type="number" min={0} value={baselineDisruptionMins} onChange={e => setBaselineDisruptionMins(e.target.value)} placeholder="e.g. 45" />

      <label style={S.label}>What was missing or would need to change before you could use this?</label>
      <textarea style={S.textarea} value={productGaps} onChange={e => setProductGaps(e.target.value)} placeholder="Missing features, integration requirements, data needs, regulatory concerns…" />

      <label style={S.label}>Would you be willing to run a paid pilot?</label>
      <div style={{ display: 'flex', gap: '10px', flexWrap: 'wrap', marginBottom: '16px' }}>
        {WILLING_TO_PILOT_OPTIONS.map(o => (
          <button key={o.value} style={S.radioBtn(willingToPilot === o.value)} onClick={() => setWillingToPilot(o.value)}>{o.label}</button>
        ))}
      </div>

      <label style={S.label}>Agreed next step</label>
      <select style={S.select} value={nextSteps} onChange={e => setNextSteps(e.target.value)}>
        <option value="">— select —</option>
        {NEXT_STEPS_OPTIONS.map(o => <option key={o} value={o}>{o}</option>)}
      </select>
    </>
  );
}
