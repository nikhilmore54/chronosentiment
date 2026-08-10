import React, { useState } from 'react';
import { S } from '../styles';
import OptimizationMetricsPanel from './OptimizationMetricsPanel';
import GanttChart from './GanttChart';
import ConstraintPanel from './ConstraintPanel';
import DisruptionPanel from './DisruptionPanel';
import DecisionWorkspace from './DecisionWorkspace';

export default function ResultsWorkspace({
  result,
  runtimeSecs,
  horizonHours,
  shiftsMap,
  workersMap,
  layoverMarkers,
  pairings,
  scenarioData,
  
  // States from App that we're delegating downwards for now
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

  recDecisions, updateRec, toggleRejectionReason, REJECTION_REASONS,
  MANUAL_EDIT_REASONS, manualEdits, addManualEdit, updateManualEdit, removeManualEdit,
  ADOPTION_OPTIONS, WILLING_TO_PILOT_OPTIONS, NEXT_STEPS_OPTIONS,
  overallRating, setOverallRating, adoptionSignal, setAdoptionSignal,
  adoptionBarrier, setAdoptionBarrier, dispatcherComments, setDispatcherComments,
  orgName, setOrgName, baselineSchedulingMins, setBaselineSchedulingMins,
  baselineDisruptionMins, setBaselineDisruptionMins, productGaps, setProductGaps,
  willingToPilot, setWillingToPilot, nextSteps, setNextSteps,
  submitting, submitError, handleSubmit
}) {
  const [ganttFilter, setGanttFilter] = useState(null);

  if (!result) return null;



  return (
    <div style={S.card}>
      <div style={S.cardTitle}>Optimization Results</div>
      <div style={S.cardSub}>Review the proposed schedule and simulate disruptions.</div>

      <OptimizationMetricsPanel result={result} runtimeSecs={runtimeSecs} />

      <ConstraintPanel pairings={pairings} workersMap={workersMap} />

      {result.schedule && Object.keys(result.schedule).length > 0 && (
        <GanttChart
          result={result}
          horizonHours={horizonHours}
          shiftsMap={shiftsMap}
          workersMap={workersMap}
          ganttFilter={ganttFilter}
          setGanttFilter={setGanttFilter}
          layoverMarkers={layoverMarkers}
        />
      )}

      <div style={S.divider} />

      <DisruptionPanel
        result={result}
        shiftsMap={shiftsMap}
        scheduleOverride={scheduleOverride}
        setScheduleOverride={setScheduleOverride}
        disruptionShiftId={disruptionShiftId}
        setDisruptionShiftId={setDisruptionShiftId}
        disruptionType={disruptionType}
        setDisruptionType={setDisruptionType}
        disruptionReassignTo={disruptionReassignTo}
        setDisruptionReassignTo={setDisruptionReassignTo}
        disruptionApplied={disruptionApplied}
        setDisruptionApplied={setDisruptionApplied}
        disruptionRecoverySecs={disruptionRecoverySecs}
        setDisruptionRecoverySecs={setDisruptionRecoverySecs}
        disruptionNotes={disruptionNotes}
        setDisruptionNotes={setDisruptionNotes}
        scenarioData={scenarioData}
      />

      <div style={S.divider} />

      <DecisionWorkspace
        recDecisions={recDecisions} updateRec={updateRec} toggleRejectionReason={toggleRejectionReason}
        REJECTION_REASONS={REJECTION_REASONS} MANUAL_EDIT_REASONS={MANUAL_EDIT_REASONS}
        manualEdits={manualEdits} addManualEdit={addManualEdit} updateManualEdit={updateManualEdit} removeManualEdit={removeManualEdit}
        ADOPTION_OPTIONS={ADOPTION_OPTIONS} WILLING_TO_PILOT_OPTIONS={WILLING_TO_PILOT_OPTIONS} NEXT_STEPS_OPTIONS={NEXT_STEPS_OPTIONS}
        overallRating={overallRating} setOverallRating={setOverallRating} adoptionSignal={adoptionSignal} setAdoptionSignal={setAdoptionSignal}
        adoptionBarrier={adoptionBarrier} setAdoptionBarrier={setAdoptionBarrier} dispatcherComments={dispatcherComments} setDispatcherComments={setDispatcherComments}
        orgName={orgName} setOrgName={setOrgName} baselineSchedulingMins={baselineSchedulingMins} setBaselineSchedulingMins={setBaselineSchedulingMins}
        baselineDisruptionMins={baselineDisruptionMins} setBaselineDisruptionMins={setBaselineDisruptionMins} productGaps={productGaps} setProductGaps={setProductGaps}
        willingToPilot={willingToPilot} setWillingToPilot={setWillingToPilot} nextSteps={nextSteps} setNextSteps={setNextSteps}
        submitting={submitting} submitError={submitError} handleSubmit={handleSubmit}
      />
    </div>
  );
}
