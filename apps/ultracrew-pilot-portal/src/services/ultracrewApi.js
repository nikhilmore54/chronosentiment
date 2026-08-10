import { buildOptimizationRequest } from './dto/OptimizationRequest';
import { parseOptimizationResponse } from './dto/OptimizationResponse';

const API_BASE = process.env.REACT_APP_API_URL || '';

export async function fetchCsrfToken() {
  const res = await fetch(`${API_BASE}/api/csrf-token`);
  if (!res.ok) throw new Error('Failed to fetch CSRF token');
  return (await res.json()).csrf_token;
}

export async function runOptimization(scenarioData, generationLimit, seed) {
  const csrfToken = await fetchCsrfToken();
  const { workers, shifts, layoverMarkers, horizonHours } = scenarioData;
  const payload = buildOptimizationRequest(scenarioData, generationLimit, seed);
  
  const res = await fetch(`${API_BASE}/api/schedule`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', 'X-CSRF-Token': csrfToken },
    body: JSON.stringify(payload),
  });
  if (!res.ok) throw new Error(await res.text());
  
  const rawData = await res.json();
  const data = parseOptimizationResponse(rawData);
  return { data, shifts, workers, layoverMarkers, horizonHours };
}

export async function runReschedule(payload) {
  // We use the same backend endpoint but potentially with existing_assignments
  const res = await fetch(`${API_BASE}/api/reschedule`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload)
  });
  if (!res.ok) throw new Error("Recovery optimization failed");
  return await res.json();
}

export async function submitPilotSession(payload) {
  const csrfToken = await fetchCsrfToken();
  const res = await fetch(`${API_BASE}/api/pilot/session`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', 'X-CSRF-Token': csrfToken },
    body: JSON.stringify(payload),
  });
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}

export async function fetchPairingsAndDuties(data, shifts, workers, lm) {
  const API_BASE = process.env.REACT_APP_API_URL || '';
  const analysisBody = JSON.stringify({ schedule: data.schedule, shifts, workers });
  const [pairRes, dutyRes] = await Promise.all([
    fetch(`${API_BASE}/api/pairings`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: analysisBody }),
    fetch(`${API_BASE}/api/duties`,   { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: analysisBody }),
  ]);
  
  let ganttMarkers = lm || [];
  let pairings = [];
  
  if (dutyRes.ok) {
    const dutyData = await dutyRes.json();
    const byWorker = {};
    (dutyData.duties || []).forEach(d => { (byWorker[d.worker_id] = byWorker[d.worker_id] || []).push(d); });
    const restMarkers = [];
    Object.entries(byWorker).forEach(([wid, duties]) => {
      duties.sort((a, b) => a.report_hour - b.report_hour);
      for (let i = 0; i < duties.length - 1; i++) {
        const restStart = duties[i].release_hour;
        const restEnd   = duties[i + 1].report_hour;
        const restHrs   = restEnd - restStart;
        if (restHrs > 0 && restHrs < 72) {
          restMarkers.push({
            start_hour: restStart,
            duration_hours: restHrs,
            type: restHrs >= 10 ? 'layover' : 'deadhead',
            label: restHrs >= 10 ? `Rest ${restHrs}h` : `Short rest ${restHrs}h (FDP risk)`,
            worker_id: parseInt(wid),
            fdp_violation: !duties[i].rest_compliant,
          });
        }
      }
    });
    if (restMarkers.length > 0) ganttMarkers = restMarkers;
  }
  
  if (pairRes.ok) {
    const pairData = await pairRes.json();
    pairings = pairData.pairings || [];
  }
  
  return { ganttMarkers, pairings };
}
