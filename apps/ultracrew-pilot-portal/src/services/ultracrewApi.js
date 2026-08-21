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
    
    if (pairRes.ok && dutyRes.ok) {
      const pairData = await pairRes.json();
      pairings = pairData.pairings || [];
      
      const restMarkers = [];
      pairings.forEach(pairing => {
        const fdps = pairing.fdp_periods || [];
        
        // 1. Rests WITHIN a pairing are layovers
        for (let i = 0; i < fdps.length - 1; i++) {
          const fdp = fdps[i];
          const nextFdp = fdps[i + 1];
          const release = fdp.sectors[fdp.sectors.length - 1].start_hour + fdp.sectors[fdp.sectors.length - 1].duration_hours;
          const report = nextFdp.sectors[0].start_hour;
          const restHrs = report - release;
          
          if (restHrs > 0) {
            restMarkers.push({
              start_hour: release,
              duration_hours: restHrs,
              type: 'layover',
              label: `Layover ${restHrs}h`,
              worker_id: parseInt(pairing.worker_id),
              fdp_violation: !fdp.rest_compliant,
            });
          }
        }
        
        // 2. Rest AFTER a pairing is home base rest (if it's not the last pairing)
        if (pairing.home_base_rest_hours) {
          const lastFdp = fdps[fdps.length - 1];
          const release = lastFdp.sectors[lastFdp.sectors.length - 1].start_hour + lastFdp.sectors[lastFdp.sectors.length - 1].duration_hours;
          const restHrs = pairing.home_base_rest_hours;
          
          if (restHrs > 0 && restHrs < 168) { // Only show if it's within a reasonable timeframe, e.g., a week
            restMarkers.push({
              start_hour: release,
              duration_hours: restHrs,
              type: 'home_base_rest',
              label: `Home Base Rest ${restHrs}h`,
              worker_id: parseInt(pairing.worker_id),
              fdp_violation: false,
            });
          }
        }
      });
      
      if (restMarkers.length > 0) ganttMarkers = restMarkers;
    } else if (dutyRes.ok) {
      // Fallback if pairings fail
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
              type: restHrs >= 36 ? 'home_base_rest' : 'layover', // Fallback heuristic based on 36h rule
              label: restHrs >= 36 ? `Home Base Rest ${restHrs}h` : `Layover ${restHrs}h`,
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
