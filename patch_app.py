import re

with open("apps/ultracrew-pilot-portal/src/App.js", "r") as f:
    content = f.read()

# 1. Add runRecoveryOptimization function
func = """
  const [isRecovering, setIsRecovering] = useState(false);
  const [recoveryError, setRecoveryError] = useState(null);

  const runRecoveryOptimization = async () => {
    if (!disruptionShiftId || !disruptionType) return;
    setIsRecovering(true);
    setRecoveryError(null);
    try {
      const sid = parseInt(disruptionShiftId);
      const shiftMeta = shiftsMap[sid];
      if (!shiftMeta) throw new Error("Shift not found");
      
      const disruptedWorkerId = effectiveSchedule[sid];
      
      // Lock all shifts that start before this one
      const lockedShiftIds = [];
      const existingAssignments = { ...effectiveSchedule };
      
      for (const [idStr, wid] of Object.entries(effectiveSchedule)) {
         const id = parseInt(idStr);
         const sMeta = shiftsMap[id];
         if (sMeta && sMeta.start_hour < shiftMeta.start_hour) {
            lockedShiftIds.push(id);
         }
      }
      
      // Remove the disrupted assignment
      delete existingAssignments[sid];
      
      // Create leave request for the sick worker
      const leaveRequests = disruptionType === 'sick_call' ? [{
         crew_id: disruptedWorkerId,
         start_hour: shiftMeta.start_hour,
         end_hour: shiftMeta.start_hour + shiftMeta.duration_hours + 24 // Give them 24 hours off
      }] : [];
      
      const payload = {
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
      
      const res = await fetch('http://127.0.0.1:3000/api/reschedule', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload)
      });
      if (!res.ok) throw new Error("Recovery optimization failed");
      const data = await res.json();
      
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
"""

content = content.replace("const applyReassignment = () => {", func + "\n  const applyReassignment = () => {")

# 2. Update UI
ui_replacement = """
                      {disruptionShiftId && (
                        <div>
                          <div style={{ fontSize: '11px', color: '#94a3b8', marginBottom: '4px' }}>
                            OEN Recovery Optimization
                          </div>
                          <button onClick={runRecoveryOptimization} disabled={isRecovering} style={{ ...S.btn('primary'), background: '#10b981' }}>
                            {isRecovering ? 'Optimizing...' : 'Run Recovery Engine'}
                          </button>
                        </div>
                      )}
"""

content = re.sub(r'\{disruptionShiftId && \(\n\s*<div>\n\s*<div style=\{\{ fontSize: \'11px\'.*?\{disruptionShiftId && \(\n\s*<div style=\{\{ fontSize: \'12px\'', ui_replacement + "\n                    {disruptionApplied.length > 0 && (\n                      <div style={{ fontSize: '12px'", content, flags=re.DOTALL)

with open("apps/ultracrew-pilot-portal/src/App.js", "w") as f:
    f.write(content)
