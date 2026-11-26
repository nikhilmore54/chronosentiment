/**
 * Prepares a recovery request payload for the backend.
 * This is a pure function that takes the current state and the disruption details
 * and returns the payload to be sent to the API.
 */
export function prepareRecoveryRequest({
  disruptionShiftId,
  disruptionType,
  shiftsMap,
  effectiveSchedule,
  scenarioData
}) {
  const sid = parseInt(disruptionShiftId);
  const shiftMeta = shiftsMap[sid];
  if (!shiftMeta) throw new Error("Shift not found");
  
  const disruptedWorkerId = effectiveSchedule[sid];
  
  const lockedShiftIds = [];
  const existingAssignments = { ...effectiveSchedule };
  
  // Lock all shifts that start before the disrupted one
  for (const [idStr, wid] of Object.entries(effectiveSchedule)) {
     const id = parseInt(idStr);
     const sMeta = shiftsMap[id];
     if (sMeta && sMeta.start_hour < shiftMeta.start_hour) {
        lockedShiftIds.push(id);
     }
  }
  
  // Remove the disrupted assignment so the optimizer can refill it
  delete existingAssignments[sid];
  
  // Create leave request for the unavailable worker
  const leaveRequests = disruptionType === 'sick_call' ? [{
     crew_id: disruptedWorkerId,
     start_hour: shiftMeta.start_hour,
     end_hour: shiftMeta.start_hour + shiftMeta.duration_hours + 24
  }] : [];
  
  return {
    request: {
       workers: scenarioData.workers,
       shifts: scenarioData.shifts,
       scenario: {
          ...scenarioData.scenario,
          leave_requests: leaveRequests.length > 0 ? leaveRequests : undefined
       }
    },
    existing_assignments: existingAssignments,
    locked_shift_ids: lockedShiftIds
  };
}
