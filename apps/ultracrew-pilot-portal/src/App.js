import React, { useState, useEffect, useCallback } from 'react';
import { S } from './styles';
import ScenarioPanel from './components/ScenarioPanel';
import OptimizationToolbar from './components/OptimizationToolbar';
import ResultsWorkspace from './components/ResultsWorkspace';
import ErrorBoundary from './components/ErrorBoundary';
import ScheduleSummary from './components/ScheduleSummary';
import { buildGeradBenchmarkScenario } from './adapters/schedule';

import { fetchCsrfToken, runOptimization, fetchPairingsAndDuties, submitPilotSession } from './services/ultracrewApi';

const REJECTION_REASONS = ['Crew unavailable in reality', 'Operational preference', 'Qualification issue', 'Rest concern', 'Airport logistics', 'Local knowledge', 'Other'];
const MANUAL_EDIT_REASONS = ['Local knowledge', 'Customer request', 'Crew preference', 'Weather / delay', 'Qualification not in model', 'Other'];
const ADOPTION_OPTIONS = [{ value: 'yes', label: 'Yes' }, { value: 'probably', label: 'Probably' }, { value: 'probably_not', label: 'Probably not' }, { value: 'no', label: 'No' }];
const WILLING_TO_PILOT_OPTIONS = [{ value: 'yes', label: 'Yes — ready to proceed' }, { value: 'maybe', label: 'Maybe — need more information' }, { value: 'no', label: 'No — not at this time' }];
const NEXT_STEPS_OPTIONS = ['Schedule a follow-up call', 'Provide a formal proposal', 'Run a paid pilot', 'Share with procurement / IT', 'No next step agreed', 'Other'];

export default function App() {
  const [csrfToken, setCsrfToken] = useState('');
  const [csrfError, setCsrfError] = useState('');
  const [dispatcherId, setDispatcherId] = useState('');
  const [dispatcherRole, setDispatcherRole] = useState('');
  
  const [scenario, setScenario] = useState('gerad-instance1');
  const [generationLimit, setGenerationLimit] = useState(500);
  const [seed, setSeed] = useState(42);
  const [running, setRunning] = useState(false);
  const [runError, setRunError] = useState('');
  
  const [result, setResult] = useState(null);
  const [scenarioData, setScenarioData] = useState(null);
  const [horizonHours, setHorizonHours] = useState(336);
  const [shiftsMap, setShiftsMap] = useState({});
  const [workersMap, setWorkersMap] = useState({});
  const [layoverMarkers, setLayoverMarkers] = useState([]);
  const [pairings, setPairings] = useState([]);
  const [runtimeSecs, setRuntimeSecs] = useState(0);
  
  const [manualEdits, setManualEdits] = useState([]);
  const [recDecisions, setRecDecisions] = useState([]);
  const [disruptionRecoverySecs, setDisruptionRecoverySecs] = useState('');
  const [disruptionNotes, setDisruptionNotes] = useState('');
  const [overallRating, setOverallRating] = useState(0);
  const [adoptionSignal, setAdoptionSignal] = useState('');
  const [adoptionBarrier, setAdoptionBarrier] = useState('');
  const [dispatcherComments, setDispatcherComments] = useState('');
  
  const [orgName, setOrgName] = useState('');
  const [baselineSchedulingMins, setBaselineSchedulingMins] = useState('');
  const [baselineDisruptionMins, setBaselineDisruptionMins] = useState('');
  const [productGaps, setProductGaps] = useState('');
  const [nextSteps, setNextSteps] = useState('');
  const [willingToPilot, setWillingToPilot] = useState('');
  
  const [disruptionType, setDisruptionType] = useState('');
  const [disruptionShiftId, setDisruptionShiftId] = useState('');
  const [disruptionReassignTo, setDisruptionReassignTo] = useState('');
  const [disruptionApplied, setDisruptionApplied] = useState([]);
  const [scheduleOverride, setScheduleOverride] = useState({});
  
  const [submitting, setSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState('');
  const [submitted, setSubmitted] = useState(null);

  useEffect(() => { fetchCsrfToken().then(setCsrfToken).catch(e => setCsrfError(e.message)); }, []);

  const handleRunOptimizer = useCallback(async () => {
    setRunning(true); setRunError('');
    const t0 = Date.now();
    try {
      const scenarioDataLocal = buildGeradBenchmarkScenario(scenario);
      const { data, shifts, workers, layoverMarkers: lm, horizonHours: hz, scenario: optimizedScenario } = await runOptimization(
        scenarioDataLocal,
        generationLimit, 
        seed
      );
      setRuntimeSecs((Date.now() - t0) / 1000);
      setResult(data);
      setHorizonHours(hz || 336);
      
      const sm = {}; shifts.forEach(s => { sm[s.id] = s; }); setShiftsMap(sm);
      const wm = {}; workers.forEach(w => { wm[w.id] = w; }); setWorkersMap(wm);

      try {
        const { ganttMarkers, pairings: p } = await fetchPairingsAndDuties(data, shifts, workers, lm, optimizedScenario);
        setLayoverMarkers(ganttMarkers);
        setPairings(p);
      } catch (analysisErr) {
        console.warn('Backend analysis unavailable, using static markers:', analysisErr.message);
        setLayoverMarkers(lm || []);
      }

      if (data.recommendations && data.recommendations.length > 0) {
        setRecDecisions(data.recommendations.map(r => ({
          recommendation_text: typeof r === 'string' ? r : (r.text || r.recommendation || JSON.stringify(r)),
          action: 'pending', rejection_reasons: [], rejection_comment: '', explanation_rating: 0,
        })));
      }
      setScenarioData({ workers, shifts, scenario: { planning_horizon_hours: hz, minimum_rest_hours: 10 } });
    }
    catch (e) { setRunError(e.message); }
    finally { setRunning(false); }
  }, [scenario, generationLimit, seed]);

  const updateRec = (idx, patch) => setRecDecisions(prev => prev.map((d, i) => i === idx ? { ...d, ...patch } : d));
  const toggleRejectionReason = (idx, reason, checked) => setRecDecisions(prev => prev.map((d, i) => {
    if (i !== idx) return d;
    const reasons = checked ? [...d.rejection_reasons, reason] : d.rejection_reasons.filter(r => r !== reason);
    return { ...d, rejection_reasons: reasons };
  }));
  const addManualEdit = () => setManualEdits(prev => [...prev, { reason: '', comment: '' }]);
  const updateManualEdit = (idx, patch) => setManualEdits(prev => prev.map((e, i) => i === idx ? { ...e, ...patch } : e));
  const removeManualEdit = (idx) => setManualEdits(prev => prev.filter((_, i) => i !== idx));

  const handleSubmit = async () => {
    setSubmitting(true); setSubmitError('');
    const cr = result && result.constraint_report ? result.constraint_report : {};
    const metrics = result && result.metrics ? result.metrics : {};
    const accepted = recDecisions.filter(d => d.action === 'accepted').length;
    const rejected = recDecisions.filter(d => d.action === 'rejected').length;
    const modified = recDecisions.filter(d => d.action === 'modified').length;
    const rated = recDecisions.filter(d => d.explanation_rating > 0);
    const avgExplanationRating = rated.length > 0 ? rated.reduce((s, d) => s + d.explanation_rating, 0) / rated.length : 0;
    
    const payload = {
      dispatcher_id: dispatcherId, dispatcher_role: dispatcherRole, scenario_id: scenario,
      coverage_pct: metrics.coverage_pct || 100.0, hard_violations: cr.hard_violations || 0,
      rest_violations: cr.rest_violations || 0, fitness: cr.fitness || 0.0, runtime_secs: runtimeSecs,
      disruption_recovery_secs: disruptionRecoverySecs ? parseFloat(disruptionRecoverySecs) : null,
      manual_edits: manualEdits.length, recommendations_presented: recDecisions.length,
      recommendations_accepted: accepted, recommendations_rejected: rejected,
      recommendation_decisions: recDecisions.map(d => ({
        recommendation_text: d.recommendation_text,
        action: d.action === 'pending' ? 'skipped' : d.action,
        rejection_reason: d.rejection_reasons.length > 0 ? d.rejection_reasons.join('; ') : (d.rejection_comment || null),
      })),
      explanation_usefulness: Math.round(avgExplanationRating),
      dispatcher_comments: [dispatcherComments, disruptionNotes ? `Disruption notes: ${disruptionNotes}` : '', adoptionBarrier ? `Adoption barrier: ${adoptionBarrier}` : ''].filter(Boolean).join('\n\n'),
      session_complete: true, overall_satisfaction: overallRating, adoption_signal: adoptionSignal,
      manual_edit_reasons: manualEdits.map(e => e.reason).filter(Boolean),
      avg_explanation_rating: avgExplanationRating, recommendations_modified: modified,
      override_rate: recDecisions.length > 0 ? ((rejected + modified) / recDecisions.length * 100).toFixed(1) : '0.0',
      org_name: orgName || null,
      baseline_scheduling_mins: baselineSchedulingMins ? parseFloat(baselineSchedulingMins) : null,
      baseline_disruption_mins: baselineDisruptionMins ? parseFloat(baselineDisruptionMins) : null,
      product_gaps: productGaps || null,
      next_steps: nextSteps || null,
      willing_to_pilot: willingToPilot || null,
    };
    try { const record = await submitPilotSession(payload); setSubmitted(record); }
    catch (e) { setSubmitError(e.message); }
    finally { setSubmitting(false); }
  };

  if (submitted) {
    const presented = submitted.recommendations_presented;
    const overrideRate = presented > 0 ? ((submitted.recommendations_rejected || 0) / presented * 100).toFixed(1) : '0.0';
    return (
      <div style={S.app}>
        <div style={S.header}><div><div style={S.headerTitle}>SunAir Scheduling Review</div><div style={S.headerSub}>Dispatcher Feedback Session</div></div><span style={S.badge}>v0.1</span></div>
        <div style={S.main}>
          <div style={S.alert('success')}>✅ Evidence record <strong>{submitted.id}</strong> written to <code>pilot_sessions/{submitted.id}.json</code></div>
          <ScheduleSummary submitted={submitted} />
          <div style={S.alert('success')}>Your reference number is <strong>{submitted.id}</strong>. The session facilitator may ask you a few brief follow-up questions.</div>
        </div>
      </div>
    );
  }

  return (
    <div style={S.app}>
      <div style={S.header}><div><div style={S.headerTitle}>SunAir Scheduling Review</div><div style={S.headerSub}>Dispatcher Feedback Session</div></div><span style={S.badge}>v0.1</span></div>
      <div style={S.main}>
        {csrfError && <div style={S.alert('error')}>⚠ Cannot connect to UltraCrew server: {csrfError}. Ensure the server is running on port 3001.</div>}

        {!dispatcherId || !dispatcherRole ? (
          <ScenarioPanel 
            dispatcherId={dispatcherId} 
            setDispatcherId={setDispatcherId} 
            dispatcherRole={dispatcherRole} 
            setDispatcherRole={setDispatcherRole} 
            onContinue={() => {}} // Not using `step` anymore for simple view gating, just show the next component
          />
        ) : !result ? (
          <OptimizationToolbar 
            scenario={scenario} setScenario={setScenario}
            generationLimit={generationLimit} setGenerationLimit={setGenerationLimit}
            seed={seed} setSeed={setSeed}
            onBack={() => { setDispatcherId(''); setDispatcherRole(''); }}
            onRun={handleRunOptimizer}
            running={running}
            csrfToken={csrfToken}
            runError={runError}
          />
        ) : (
          <ErrorBoundary>
            <ResultsWorkspace 
              result={result} runtimeSecs={runtimeSecs} horizonHours={horizonHours}
              shiftsMap={shiftsMap} workersMap={workersMap} layoverMarkers={layoverMarkers}
              pairings={pairings} scenarioData={scenarioData}
              scheduleOverride={scheduleOverride} setScheduleOverride={setScheduleOverride}
              disruptionShiftId={disruptionShiftId} setDisruptionShiftId={setDisruptionShiftId}
              disruptionType={disruptionType} setDisruptionType={setDisruptionType}
              disruptionReassignTo={disruptionReassignTo} setDisruptionReassignTo={setDisruptionReassignTo}
              disruptionApplied={disruptionApplied} setDisruptionApplied={setDisruptionApplied}
              disruptionRecoverySecs={disruptionRecoverySecs} setDisruptionRecoverySecs={setDisruptionRecoverySecs}
              disruptionNotes={disruptionNotes} setDisruptionNotes={setDisruptionNotes}
              recDecisions={recDecisions} updateRec={updateRec} toggleRejectionReason={toggleRejectionReason} REJECTION_REASONS={REJECTION_REASONS}
              MANUAL_EDIT_REASONS={MANUAL_EDIT_REASONS} manualEdits={manualEdits} addManualEdit={addManualEdit} updateManualEdit={updateManualEdit} removeManualEdit={removeManualEdit}
              ADOPTION_OPTIONS={ADOPTION_OPTIONS} WILLING_TO_PILOT_OPTIONS={WILLING_TO_PILOT_OPTIONS} NEXT_STEPS_OPTIONS={NEXT_STEPS_OPTIONS}
              overallRating={overallRating} setOverallRating={setOverallRating} adoptionSignal={adoptionSignal} setAdoptionSignal={setAdoptionSignal}
              adoptionBarrier={adoptionBarrier} setAdoptionBarrier={setAdoptionBarrier} dispatcherComments={dispatcherComments} setDispatcherComments={setDispatcherComments}
              orgName={orgName} setOrgName={setOrgName} baselineSchedulingMins={baselineSchedulingMins} setBaselineSchedulingMins={setBaselineSchedulingMins}
              baselineDisruptionMins={baselineDisruptionMins} setBaselineDisruptionMins={setBaselineDisruptionMins} productGaps={productGaps} setProductGaps={setProductGaps}
              willingToPilot={willingToPilot} setWillingToPilot={setWillingToPilot} nextSteps={nextSteps} setNextSteps={setNextSteps}
              submitting={submitting} submitError={submitError} handleSubmit={handleSubmit}
            />
          </ErrorBoundary>
        )}
      </div>
    </div>
  );
}