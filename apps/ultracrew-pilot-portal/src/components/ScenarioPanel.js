import { S } from "../styles";

import React from 'react';


export default function ScenarioPanel({
  dispatcherId,
  setDispatcherId,
  dispatcherRole,
  setDispatcherRole,
  onContinue
}) {
  return (
    <div style={S.card}>
      <div style={S.cardTitle}>Step 1 — Dispatcher Identity</div>
      <div style={S.cardSub}>Your responses are kept confidential and used only to improve the scheduling tool.</div>
      
      <label style={S.label}>Dispatcher ID (anonymised, e.g. D-01)</label>
      <input style={S.input} value={dispatcherId} onChange={e => setDispatcherId(e.target.value)} placeholder="D-01" />
      
      <label style={S.label}>Role and experience</label>
      <select style={S.select} value={dispatcherRole} onChange={e => setDispatcherRole(e.target.value)}>
        <option value="">Select role…</option>
        <option value="Senior Dispatcher (5+ years)">Senior Dispatcher (5+ years)</option>
        <option value="Dispatcher (2–5 years)">Dispatcher (2–5 years)</option>
        <option value="Junior Dispatcher (<2 years)">Junior Dispatcher (&lt;2 years)</option>
        <option value="Crew Planning Manager">Crew Planning Manager</option>
        <option value="Observer / Evaluator">Observer / Evaluator</option>
      </select>
      
      <button 
        style={dispatcherId && dispatcherRole ? S.btn('primary') : S.btnDisabled} 
        disabled={!dispatcherId || !dispatcherRole} 
        onClick={onContinue}
      >
        Continue →
      </button>
    </div>
  );
}
