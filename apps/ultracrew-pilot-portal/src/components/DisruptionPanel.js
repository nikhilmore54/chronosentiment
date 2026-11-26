import { S } from "../styles";

import React, { useState } from 'react';
import { runReschedule } from '../services/ultracrewApi';
import { prepareRecoveryRequest } from '../adapters/recovery';


export default function DisruptionPanel({
  result,
  shiftsMap,
  scheduleOverride,
  setScheduleOverride,
  disruptionShiftId,
  setDisruptionShiftId,
  disruptionType,
  setDisruptionType,
  disruptionReassignTo,
  setDisruptionReassignTo,
  disruptionApplied,
  setDisruptionApplied,
  disruptionRecoverySecs,
  setDisruptionRecoverySecs,
  disruptionNotes,
  setDisruptionNotes,
  scenarioData
}) {
  const [isRecovering, setIsRecovering] = useState(false);
  const [recoveryError, setRecoveryError] = useState(null);

  if (!result || !result.schedule || Object.keys(result.schedule).length === 0) return null;

  const effectiveSchedule = { ...result.schedule, ...scheduleOverride };
  const shiftIds = Object.keys(effectiveSchedule).map(Number).sort((a, b) => a - b);
  const allWorkers = [...new Set(Object.values(effectiveSchedule).map(Number))].sort((a, b) => a - b);

  const getAvailableWorkers = (targetShiftId) => {
    const meta = shiftsMap[targetShiftId] || {};
    const tStart = meta.start_hour || 0;
    const tEnd = tStart + (meta.duration_hours || 8);
    return allWorkers.filter(wid => {
      if (wid === Number(effectiveSchedule[targetShiftId])) return false;
      return shiftIds.every(sid => {
        if (Number(effectiveSchedule[sid]) !== wid) return true;
        const sm = shiftsMap[sid] || {};
        const sStart = sm.start_hour || 0;
        const sEnd = sStart + (sm.duration_hours || 8);
        return tEnd <= sStart || tStart >= sEnd;
      });
    });
  };

  const runRecoveryOptimization = async () => {
    if (!disruptionShiftId || !disruptionType) return;
    setIsRecovering(true);
    setRecoveryError(null);
    try {
      const payload = prepareRecoveryRequest({
        disruptionShiftId,
        disruptionType,
        shiftsMap,
        effectiveSchedule,
        scenarioData
      });
      
      const data = await runReschedule(payload);
      const sid = parseInt(disruptionShiftId);
      const disruptedWorkerId = effectiveSchedule[sid];
      
      setScheduleOverride(data.schedule);
      setDisruptionApplied(prev => [...prev, {
        shiftId: sid,
        type: disruptionType,
        fromWorker: disruptedWorkerId,
        toWorker: data.schedule[sid] || 'Unassigned',
        note: 'Optimized Recovery'
      }]);
    } catch (e) {
      setRecoveryError(e.message);
    } finally {
      setIsRecovering(false);
    }
  };

  const applyReassignment = () => {
    if (!disruptionShiftId || !disruptionReassignTo) return;
    const sid = parseInt(disruptionShiftId);
    const fromWorker = effectiveSchedule[sid];
    const toWorker = parseInt(disruptionReassignTo);
    setScheduleOverride(prev => ({ ...prev, [sid]: toWorker }));
    setDisruptionApplied(prev => [...prev, {
      shiftId: sid, type: disruptionType || 'reassignment',
      fromWorker, toWorker,
    }]);
    setDisruptionShiftId('');
    setDisruptionReassignTo('');
  };

  return (
    <div style={S.card}>
      <div style={S.cardTitle}>Simulate Disruption (Optional)</div>
      <div style={S.cardSub}>Test how the schedule handles a disruption, and how you would recover it.</div>
      
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '16px' }}>
        <div>
          <label style={S.label}>Disrupted Shift</label>
          <select style={S.select} value={disruptionShiftId} onChange={e => setDisruptionShiftId(e.target.value)}>
            <option value="">Select a shift to disrupt...</option>
            {shiftIds.map(sid => {
              const route = (shiftsMap[sid] || {}).route || '';
              const rDisp = route ? ` · ${route}` : '';
              return (
                <option key={sid} value={sid}>Shift {sid}{rDisp} (Worker {effectiveSchedule[sid]})</option>
              );
            })}
          </select>
        </div>
        <div>
          <label style={S.label}>Disruption Type</label>
          <select style={S.select} value={disruptionType} onChange={e => setDisruptionType(e.target.value)}>
            <option value="">Select type...</option>
            <option value="sick_call">Crew Sick Call</option>
            <option value="no_show">Crew No Show</option>
            <option value="fatigue">Crew Fatigue Report</option>
          </select>
        </div>
      </div>

      {disruptionShiftId && (
        <div style={{ background: '#0f172a', padding: '16px', borderRadius: '8px', border: '1px solid #334155', marginBottom: '16px' }}>
          <div style={{ fontSize: '13px', color: '#e2e8f0', marginBottom: '12px' }}>
            Shift <strong>{disruptionShiftId}</strong> is disrupted. Worker <strong>W{effectiveSchedule[disruptionShiftId]}</strong> is unavailable.
          </div>
          
          <div style={{ display: 'flex', gap: '16px', alignItems: 'flex-start' }}>
            <div style={{ flex: 1, paddingRight: '16px', borderRight: '1px solid #334155' }}>
              <div style={{ fontSize: '12px', fontWeight: '600', color: '#94a3b8', marginBottom: '8px' }}>Manual Recovery</div>
              <label style={S.label}>Reassign to</label>
              <select style={S.select} value={disruptionReassignTo} onChange={e => setDisruptionReassignTo(e.target.value)}>
                <option value="">Select available worker...</option>
                {getAvailableWorkers(disruptionShiftId).map(wid => (
                  <option key={wid} value={wid}>Worker W{wid}</option>
                ))}
              </select>
              <button style={S.btn('primary')} onClick={applyReassignment} disabled={!disruptionReassignTo}>
                Apply Manual Fix
              </button>
            </div>
            
            <div style={{ flex: 1 }}>
              <div style={{ fontSize: '12px', fontWeight: '600', color: '#94a3b8', marginBottom: '8px' }}>Automated Recovery</div>
              <p style={{ fontSize: '12px', color: '#64748b', marginBottom: '12px', lineHeight: '1.4' }}>
                Run the optimizer to find the best recovery plan. Locks all shifts prior to this disruption.
              </p>
              <button 
                style={isRecovering ? S.btnDisabled : S.btn('success')} 
                onClick={runRecoveryOptimization} 
                disabled={isRecovering || !disruptionType}
              >
                {isRecovering ? 'Optimizing Recovery...' : 'Run Auto-Recovery'}
              </button>
              {recoveryError && <div style={{ color: '#ef4444', fontSize: '12px', marginTop: '8px' }}>{recoveryError}</div>}
            </div>
          </div>
        </div>
      )}

      {disruptionApplied.length > 0 && (
        <div style={{ marginBottom: '16px' }}>
          <div style={S.label}>Applied Disruptions & Recoveries</div>
          {disruptionApplied.map((d, i) => (
            <div key={i} style={{ fontSize: '13px', color: '#f59e0b', background: '#451a03', padding: '8px 12px', borderRadius: '6px', marginBottom: '6px', border: '1px solid #78350f' }}>
              ⚠ Shift {d.shiftId} ({d.type}) — Reassigned W{d.fromWorker} → W{d.toWorker} {d.note ? `(${d.note})` : ''}
            </div>
          ))}
        </div>
      )}

      <label style={S.label}>How long did it take to decide on this recovery? (seconds)</label>
      <input type="number" style={S.input} value={disruptionRecoverySecs} onChange={e => setDisruptionRecoverySecs(e.target.value)} placeholder="e.g. 45" />

      <label style={S.label}>Recovery Notes / What was missing from the tool?</label>
      <textarea style={S.textarea} value={disruptionNotes} onChange={e => setDisruptionNotes(e.target.value)} placeholder="e.g. I had to call crew scheduling to verify visa status before making this swap..." />
    </div>
  );
}
