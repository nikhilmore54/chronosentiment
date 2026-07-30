import React, { useState, useEffect, useCallback } from 'react';
import { GERAD_INSTANCE1_WORKERS, GERAD_INSTANCE1_SHIFTS, GERAD_INSTANCE1_META } from './geradInstance1';

const S = {
  app: { minHeight: '100vh', background: '#0f172a', color: '#e2e8f0', fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif" },
  header: { background: '#1e293b', borderBottom: '1px solid #334155', padding: '16px 32px', display: 'flex', alignItems: 'center', gap: '16px' },
  headerTitle: { fontSize: '18px', fontWeight: '600', color: '#f1f5f9' },
  headerSub: { fontSize: '13px', color: '#64748b' },
  badge: { background: '#0ea5e9', color: '#fff', fontSize: '11px', fontWeight: '700', padding: '2px 8px', borderRadius: '4px', letterSpacing: '0.05em' },
  main: { maxWidth: '860px', margin: '0 auto', padding: '32px 24px' },
  stepBar: { display: 'flex', gap: '8px', marginBottom: '32px' },
  stepDot: (active, done) => ({ flex: 1, height: '4px', borderRadius: '2px', background: done ? '#0ea5e9' : active ? '#38bdf8' : '#1e293b', transition: 'background 0.2s' }),
  card: { background: '#1e293b', border: '1px solid #334155', borderRadius: '12px', padding: '28px', marginBottom: '20px' },
  cardTitle: { fontSize: '16px', fontWeight: '600', color: '#f1f5f9', marginBottom: '4px' },
  cardSub: { fontSize: '13px', color: '#64748b', marginBottom: '20px', lineHeight: '1.5' },
  label: { display: 'block', fontSize: '13px', color: '#94a3b8', marginBottom: '6px', fontWeight: '500' },
  input: { width: '100%', background: '#0f172a', border: '1px solid #334155', borderRadius: '8px', color: '#e2e8f0', fontSize: '14px', padding: '10px 14px', marginBottom: '16px', outline: 'none' },
  select: { width: '100%', background: '#0f172a', border: '1px solid #334155', borderRadius: '8px', color: '#e2e8f0', fontSize: '14px', padding: '10px 14px', marginBottom: '16px', outline: 'none' },
  textarea: { width: '100%', background: '#0f172a', border: '1px solid #334155', borderRadius: '8px', color: '#e2e8f0', fontSize: '14px', padding: '10px 14px', marginBottom: '16px', outline: 'none', resize: 'vertical', minHeight: '80px', fontFamily: 'inherit' },
  btn: (v) => ({ padding: '10px 20px', borderRadius: '8px', border: 'none', cursor: 'pointer', fontSize: '14px', fontWeight: '600', marginRight: '8px', color: '#fff', background: v === 'primary' ? '#0ea5e9' : v === 'success' ? '#10b981' : v === 'danger' ? '#ef4444' : '#334155' }),
  btnDisabled: { padding: '10px 20px', borderRadius: '8px', border: 'none', fontSize: '14px', fontWeight: '600', background: '#1e293b', color: '#475569', marginRight: '8px', cursor: 'not-allowed' },
  kpiGrid: { display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(130px, 1fr))', gap: '12px', marginBottom: '20px' },
  kpiCard: { background: '#0f172a', border: '1px solid #334155', borderRadius: '8px', padding: '14px', textAlign: 'center' },
  kpiValue: { fontSize: '22px', fontWeight: '700', color: '#38bdf8' },
  kpiLabel: { fontSize: '11px', color: '#64748b', marginTop: '4px', textTransform: 'uppercase', letterSpacing: '0.05em' },
  recCard: (a) => ({ background: a === 'accepted' ? '#052e16' : a === 'rejected' ? '#450a0a' : a === 'modified' ? '#1c1917' : '#0f172a', border: `1px solid ${a === 'accepted' ? '#14532d' : a === 'rejected' ? '#7f1d1d' : a === 'modified' ? '#44403c' : '#334155'}`, borderRadius: '8px', padding: '16px', marginBottom: '12px' }),
  recText: { fontSize: '14px', color: '#cbd5e1', marginBottom: '12px', lineHeight: '1.5' },
  recActions: { display: 'flex', gap: '8px', flexWrap: 'wrap', marginBottom: '10px' },
  recSubCard: { background: '#1e293b', border: '1px solid #334155', borderRadius: '6px', padding: '12px', marginTop: '10px' },
  recSubLabel: { fontSize: '12px', color: '#64748b', marginBottom: '8px', fontWeight: '500' },
  checkRow: { display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '6px', cursor: 'pointer' },
  checkLabel: { fontSize: '13px', color: '#94a3b8' },
  starRow: { display: 'flex', gap: '6px', marginBottom: '8px', alignItems: 'center' },
  star: (f) => ({ fontSize: '24px', cursor: 'pointer', color: f ? '#f59e0b' : '#334155', transition: 'color 0.1s', userSelect: 'none' }),
  alert: (t) => ({ padding: '12px 16px', borderRadius: '8px', marginBottom: '16px', fontSize: '13px', lineHeight: '1.5', background: t === 'error' ? '#450a0a' : t === 'success' ? '#052e16' : '#0c1a2e', border: `1px solid ${t === 'error' ? '#7f1d1d' : t === 'success' ? '#14532d' : '#1e3a5f'}`, color: t === 'error' ? '#fca5a5' : t === 'success' ? '#86efac' : '#93c5fd' }),
  divider: { borderTop: '1px solid #334155', margin: '20px 0' },
  summaryTable: { width: '100%', borderCollapse: 'collapse', fontSize: '13px' },
  summaryTd: { padding: '8px 12px', borderBottom: '1px solid #1e293b', color: '#94a3b8' },
  summaryTdVal: { padding: '8px 12px', borderBottom: '1px solid #1e293b', color: '#e2e8f0', fontWeight: '600', textAlign: 'right' },
  radioBtn: (s) => ({ padding: '8px 16px', borderRadius: '6px', border: `1px solid ${s ? '#0ea5e9' : '#334155'}`, background: s ? '#0c2d48' : '#0f172a', color: s ? '#38bdf8' : '#94a3b8', cursor: 'pointer', fontSize: '13px', fontWeight: s ? '600' : '400' }),
  editCard: { background: '#0f172a', border: '1px solid #334155', borderRadius: '8px', padding: '14px', marginBottom: '10px' },
};

const STEPS = ['Identity', 'Run Optimizer', 'Review Schedule', 'Recommendations', 'Disruption Recovery', 'Session Debrief'];
const REJECTION_REASONS = ['Crew unavailable in reality', 'Operational preference', 'Qualification issue', 'Rest concern', 'Airport logistics', 'Local knowledge', 'Other'];
const MANUAL_EDIT_REASONS = ['Local knowledge', 'Customer request', 'Crew preference', 'Weather / delay', 'Qualification not in model', 'Other'];
const ADOPTION_OPTIONS = [{ value: 'yes', label: 'Yes' }, { value: 'probably', label: 'Probably' }, { value: 'probably_not', label: 'Probably not' }, { value: 'no', label: 'No' }];
const WILLING_TO_PILOT_OPTIONS = [{ value: 'yes', label: 'Yes — ready to proceed' }, { value: 'maybe', label: 'Maybe — need more information' }, { value: 'no', label: 'No — not at this time' }];
const NEXT_STEPS_OPTIONS = ['Schedule a follow-up call', 'Provide a formal proposal', 'Run a paid pilot', 'Share with procurement / IT', 'No next step agreed', 'Other'];
const EXPLANATION_LABELS = ['', 'Not useful', 'Slightly useful', 'Moderately useful', 'Very useful', 'Extremely useful'];

const API_BASE = process.env.REACT_APP_API_URL || '';

async function fetchCsrfToken() {
  const res = await fetch(`${API_BASE}/api/csrf-token`);
  if (!res.ok) throw new Error('Failed to fetch CSRF token');
  return (await res.json()).csrf_token;
}

// ── Scenario builders ─────────────────────────────────────────────────────
//
// Three scenarios are available in the portal:
//
//   sunair          — Product demo. 20 workers, 42 shifts, 7-day horizon.
//                     Synthetic Indian airline schedule (BOM/DEL/BLR/HYD/CCU/MAA).
//                     Use for dispatcher evidence collection sessions.
//
//   gerad-fixture   — Deterministic dev/test. 8 crew, 10 legs, 5 duties, 5-day horizon.
//                     Derived from adapters/gerad/tests/fixtures/ (22/22 tests pass).
//                     Block times are real (not synthetic 8h slots).
//                     HONEST LABEL: this is the adapter test fixture, NOT the research dataset.
//
//   gerad-benchmark — Optimisation validation. Requires G1422-DataSets.zip from
//                     GERAD Technical Report G-2014-22 (Kasirzadeh, Saddoune, Soumis).
//                     See benchmarks/gerad-g2014-22/README.md for acquisition instructions.
//                     GATED — not available until the dataset is downloaded and imported.

function buildSunairScenario() {
  const SLOT_OFFSETS = [6, 14, 22];
  const WORKER_PROFILES = [
    { role: 'Captain',       type_rating: 'A320',      skills: ['A320-CPT'] },
    { role: 'First Officer', type_rating: 'A320',      skills: ['A320-FO'] },
    { role: 'Captain',       type_rating: 'A320',      skills: ['A320-CPT'] },
    { role: 'First Officer', type_rating: 'A320',      skills: ['A320-FO'] },
    { role: 'Captain',       type_rating: 'A320',      skills: ['A320-CPT'] },
    { role: 'First Officer', type_rating: 'A320',      skills: ['A320-FO'] },
    { role: 'Captain',       type_rating: 'A320',      skills: ['A320-CPT'] },
    { role: 'First Officer', type_rating: 'A320',      skills: ['A320-FO'] },
    { role: 'Captain',       type_rating: 'A320',      skills: ['A320-CPT'] },
    { role: 'First Officer', type_rating: 'A320',      skills: ['A320-FO'] },
    { role: 'Captain',       type_rating: 'B737',      skills: ['B737-CPT'] },
    { role: 'First Officer', type_rating: 'B737',      skills: ['B737-FO'] },
    { role: 'Captain',       type_rating: 'B737',      skills: ['B737-CPT'] },
    { role: 'First Officer', type_rating: 'B737',      skills: ['B737-FO'] },
    { role: 'Captain',       type_rating: 'ATR72',     skills: ['ATR72-CPT'] },
    { role: 'First Officer', type_rating: 'ATR72',     skills: ['ATR72-FO'] },
    { role: 'Cabin Crew',    type_rating: 'All fleet', skills: ['CabinCrew'] },
    { role: 'Cabin Crew',    type_rating: 'All fleet', skills: ['CabinCrew'] },
    { role: 'Cabin Crew',    type_rating: 'All fleet', skills: ['CabinCrew'] },
    { role: 'Cabin Crew',    type_rating: 'All fleet', skills: ['CabinCrew'] },
  ];
  const workers = WORKER_PROFILES.map((p, i) => ({ id: i + 1, skills: p.skills, role: p.role, type_rating: p.type_rating }));
  const flights = [
    { id: 'SA101', aircraft: 'A320',  day: 0,  slot: 0, duration: 8, route: 'BOM→DEL' },
    { id: 'SA102', aircraft: 'A320',  day: 2,  slot: 1, duration: 6, route: 'DEL→BLR' },
    { id: 'SA103', aircraft: 'A320',  day: 4,  slot: 0, duration: 8, route: 'BLR→HYD' },
    { id: 'SA104', aircraft: 'A320',  day: 6,  slot: 2, duration: 7, route: 'HYD→CCU' },
    { id: 'SA105', aircraft: 'A320',  day: 8,  slot: 1, duration: 6, route: 'CCU→BOM' },
    { id: 'SA201', aircraft: 'B737',  day: 1,  slot: 0, duration: 9, route: 'BOM→MAA' },
    { id: 'SA202', aircraft: 'B737',  day: 3,  slot: 2, duration: 8, route: 'MAA→AMD' },
    { id: 'SA203', aircraft: 'B737',  day: 5,  slot: 1, duration: 7, route: 'AMD→PNQ' },
    { id: 'SA204', aircraft: 'B737',  day: 9,  slot: 0, duration: 8, route: 'PNQ→BOM' },
    { id: 'SA301', aircraft: 'ATR72', day: 7,  slot: 0, duration: 4, route: 'GOI→BOM' },
    { id: 'SA302', aircraft: 'ATR72', day: 11, slot: 1, duration: 4, route: 'BOM→GOI' },
  ];
  const CC_REQUIRED = { A320: 2, B737: 2, ATR72: 1 };
  const shifts = [];
  let shiftId = 1;
  for (const flt of flights) {
    const startHour = flt.day * 24 + SLOT_OFFSETS[flt.slot];
    const base = { start_hour: startHour, duration_hours: flt.duration, aircraft_type: flt.aircraft, flight_id: flt.id, route: flt.route };
    shifts.push({ id: shiftId++, ...base, required_skill: `${flt.aircraft}-CPT`, crew_role: 'Captain' });
    shifts.push({ id: shiftId++, ...base, required_skill: `${flt.aircraft}-FO`,  crew_role: 'First Officer' });
    const ccCount = CC_REQUIRED[flt.aircraft] || 1;
    for (let c = 0; c < ccCount; c++) shifts.push({ id: shiftId++, ...base, required_skill: 'CabinCrew', crew_role: 'Cabin Crew' });
  }
  const layoverMarkers = [
    { start_hour: 14,  duration_hours: 16, type: 'layover',  label: 'Layover DEL', flight_id: 'SA101' },
    { start_hour: 39,  duration_hours: 15, type: 'layover',  label: 'Layover MAA', flight_id: 'SA201' },
    { start_hour: 166, duration_hours: 4,  type: 'deadhead', label: 'DH BOM→GOI',  flight_id: 'SA301' },
    { start_hour: 94,  duration_hours: 8,  type: 'layover',  label: 'Layover AMD', flight_id: 'SA202' },
  ];
  return { workers, shifts, layoverMarkers, horizonHours: 336, maxHoursPerWorker: 48 };
}

// GERAD Fixture — adapter test dataset (honest label: NOT the research benchmark).
// Source: adapters/gerad/tests/fixtures/ (5 CSV files, 22/22 tests pass).
// 8 crew, 10 legs, 5 duties, 5-day horizon (2014-01-06 to 2014-01-10).
// start_hour = hours since 2014-01-06T00:00 UTC.
// Block times derived from fixture scheduled_departure / scheduled_arrival.
// All crew based at ORD. Each duty = 2 legs (outbound + return same day).
function buildGeradFixtureScenario() {
  const workers = [
    { id: 1, skills: ['B738-CPT'],            role: 'Captain',           type_rating: 'B738',      name: 'Alice Brennan',    gerad_id: 'C0001' },
    { id: 2, skills: ['B738-FO'],             role: 'First Officer',     type_rating: 'B738',      name: 'Robert Okafor',    gerad_id: 'C0002' },
    { id: 3, skills: ['A320-CPT'],            role: 'Captain',           type_rating: 'A320',      name: 'Priya Nair',       gerad_id: 'C0003' },
    { id: 4, skills: ['A320-FO'],             role: 'First Officer',     type_rating: 'A320',      name: 'James Whitfield',  gerad_id: 'C0004' },
    { id: 5, skills: ['CabinCrew'],           role: 'Cabin Crew Senior', type_rating: 'B738',      name: 'Maria Santos',     gerad_id: 'C0005' },
    { id: 6, skills: ['CabinCrew'],           role: 'Cabin Crew',        type_rating: 'B738',      name: 'David Kim',        gerad_id: 'C0006' },
    { id: 7, skills: ['CabinCrew'],           role: 'Cabin Crew',        type_rating: 'A320',      name: 'Fatima Al-Hassan', gerad_id: 'C0007' },
    { id: 8, skills: ['B738-CPT','A320-CPT'], role: 'Captain',           type_rating: 'B738/A320', name: 'Thomas Eriksson',  gerad_id: 'C0008' },
  ];
  const GERAD_FLIGHTS = [
    { id: 'FL0001', duty_id: 'D0001', aircraft: 'B738', start_hour: 8,    duration_hours: 3.5,  route: 'ORD→LAX' },
    { id: 'FL0002', duty_id: 'D0001', aircraft: 'B738', start_hour: 14,   duration_hours: 5.5,  route: 'LAX→ORD' },
    { id: 'FL0003', duty_id: 'D0002', aircraft: 'B738', start_hour: 31,   duration_hours: 2.25, route: 'ORD→DFW' },
    { id: 'FL0004', duty_id: 'D0002', aircraft: 'B738', start_hour: 35,   duration_hours: 2.33, route: 'DFW→ORD' },
    { id: 'FL0005', duty_id: 'D0003', aircraft: 'A320', start_hour: 54.5, duration_hours: 4.25, route: 'ORD→MIA' },
    { id: 'FL0006', duty_id: 'D0003', aircraft: 'A320', start_hour: 61,   duration_hours: 4.25, route: 'MIA→ORD' },
    { id: 'FL0007', duty_id: 'D0004', aircraft: 'B738', start_hour: 80,   duration_hours: 3,    route: 'ORD→JFK' },
    { id: 'FL0008', duty_id: 'D0004', aircraft: 'B738', start_hour: 85.5, duration_hours: 3,    route: 'JFK→ORD' },
    { id: 'FL0009', duty_id: 'D0005', aircraft: 'A320', start_hour: 103,  duration_hours: 2.5,  route: 'ORD→SEA' },
    { id: 'FL0010', duty_id: 'D0005', aircraft: 'A320', start_hour: 107,  duration_hours: 2.5,  route: 'SEA→ORD' },
  ];
  const shifts = [];
  let shiftId = 1;
  for (const flt of GERAD_FLIGHTS) {
    const base = { start_hour: flt.start_hour, duration_hours: flt.duration_hours, aircraft_type: flt.aircraft, flight_id: flt.id, duty_id: flt.duty_id, route: flt.route };
    shifts.push({ id: shiftId++, ...base, required_skill: `${flt.aircraft}-CPT`, crew_role: 'Captain' });
    shifts.push({ id: shiftId++, ...base, required_skill: `${flt.aircraft}-FO`,  crew_role: 'First Officer' });
    shifts.push({ id: shiftId++, ...base, required_skill: 'CabinCrew',           crew_role: 'Cabin Crew' });
  }
  const layoverMarkers = [
    { start_hour: 11.5,  duration_hours: 2.5,  type: 'layover', label: 'Turnaround LAX', flight_id: 'FL0001' },
    { start_hour: 33.25, duration_hours: 1.75, type: 'layover', label: 'Turnaround DFW', flight_id: 'FL0003' },
    { start_hour: 58.75, duration_hours: 2.25, type: 'layover', label: 'Turnaround MIA', flight_id: 'FL0005' },
    { start_hour: 83,    duration_hours: 2.5,  type: 'layover', label: 'Turnaround JFK', flight_id: 'FL0007' },
    { start_hour: 105.5, duration_hours: 1.5,  type: 'layover', label: 'Turnaround SEA', flight_id: 'FL0009' },
  ];
  return { workers, shifts, layoverMarkers, horizonHours: 120, maxHoursPerWorker: 20 };
}

// GERAD Benchmark — extended fixture dataset (honest label: fixture data scaled to benchmark size).
// 16 crew, 20 legs, 10 duties, 10-day horizon (2014-01-06 to 2014-01-15).
// Doubles the fixture crew pool and adds 5 additional duty days to stress-test the optimizer.
// This is NOT the G1422-DataSets.zip research benchmark — it is the fixture data extended for
// internal validation. The real research benchmark requires the GERAD Technical Report dataset.
// GERAD Benchmark — real G-2014-22 Instance 1 data (Kasirzadeh, Saddoune & Soumis 2014).
// 33 crew · 60 duties (portal demo slice of 172 pairings) · 706h horizon.
// Workers and shifts are projected from the real instance1 CSVs via gen_gerad_js.py.
// start_hour and duration_hours are integers (u64-compatible) with hours normalized
// so the first duty starts at hour 0 (relative timing preserved).
function buildGeradBenchmarkScenario() {
  // Project GERAD_INSTANCE1_WORKERS: keep only id and skills (backend contract).
  // Extra fields (name, base, gerad_id, contract_type) are passed through for UI display.
  const workers = GERAD_INSTANCE1_WORKERS.map(w => ({
    id: w.id,
    skills: w.skills,
    // UI-only metadata (ignored by backend, used for Gantt/table display)
    name: w.name,
    base: w.base,
    gerad_id: w.gerad_id,
    contract_type: w.contract_type,
  }));

  // Project GERAD_INSTANCE1_SHIFTS: keep backend-required fields as integers.
  // Extra fields (gerad_duty_id, gerad_crew_id, flight_ids) are passed through for UI.
  const shifts = GERAD_INSTANCE1_SHIFTS.map(s => ({
    id: s.id,
    start_hour: Math.round(s.start_hour),       // u64: integer hours
    duration_hours: Math.max(1, Math.round(s.duration_hours)), // u64: min 1h
    required_skill: s.required_skill,
    // UI-only metadata
    gerad_duty_id: s.gerad_duty_id,
    gerad_crew_id: s.gerad_crew_id,
    flight_ids: s.flight_ids,
  }));

  // No layover markers for the real instance (duties don't have turnaround data in the CSVs).
  const layoverMarkers = [];

  return {
    workers,
    shifts,
    layoverMarkers,
    horizonHours: GERAD_INSTANCE1_META.horizon_hours,
    maxHoursPerWorker: GERAD_INSTANCE1_META.max_hours_per_worker,
  };
}

async function runOptimizer(scenarioId, generationLimit, seed) {
  const csrfToken = await fetchCsrfToken();
  const scenarioData = scenarioId === 'gerad-fixture' ? buildGeradFixtureScenario()
    : scenarioId === 'gerad-benchmark' ? buildGeradBenchmarkScenario()
    : buildSunairScenario();
  const { workers, shifts, layoverMarkers, horizonHours, maxHoursPerWorker } = scenarioData;
  const res = await fetch(`${API_BASE}/api/schedule`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', 'X-CSRF-Token': csrfToken },
    body: JSON.stringify({ workers, shifts, rng_seed: seed, generation_limit: generationLimit, scenario: { planning_horizon_hours: horizonHours, max_hours_per_worker: maxHoursPerWorker } }),
  });
  if (!res.ok) throw new Error(await res.text());
  const data = await res.json();
  return { data, shifts, workers, layoverMarkers, horizonHours };
}

async function submitPilotSession(payload) {
  // Re-fetch CSRF token immediately before the POST to avoid stale-token errors
  const csrfToken = await fetchCsrfToken();
  const res = await fetch(`${API_BASE}/api/pilot/session`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', 'X-CSRF-Token': csrfToken },
    body: JSON.stringify(payload),
  });
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}

function StarRating({ value, onChange, label }) {
  const [hover, setHover] = useState(0);
  return (
    <div>
      {label && <div style={S.recSubLabel}>{label}</div>}
      <div style={S.starRow}>
        {[1, 2, 3, 4, 5].map(n => (
          <span key={n} style={S.star(n <= (hover || value))}
            onClick={() => onChange(n)} onMouseEnter={() => setHover(n)} onMouseLeave={() => setHover(0)}>★</span>
        ))}
        {(hover || value) > 0 && <span style={{ fontSize: '12px', color: '#64748b', marginLeft: '6px' }}>{EXPLANATION_LABELS[hover || value]}</span>}
      </div>
    </div>
  );
}

export default function App() {
  const [step, setStep] = useState(0);
  const [csrfToken, setCsrfToken] = useState('');
  const [csrfError, setCsrfError] = useState('');
  const [dispatcherId, setDispatcherId] = useState('');
  const [dispatcherRole, setDispatcherRole] = useState('');
  const [scenario, setScenario] = useState('sunair');
  const [generationLimit, setGenerationLimit] = useState(10);
  const [seed, setSeed] = useState(42);
  const [running, setRunning] = useState(false);
  const [runError, setRunError] = useState('');
  const [result, setResult] = useState(null);
  const [horizonHours, setHorizonHours] = useState(336); // actual scenario horizon for Gantt
  const [shiftsMap, setShiftsMap] = useState({}); // {id → shift metadata}
  const [workersMap, setWorkersMap] = useState({}); // {id → {skills, role, type_rating}}
  const [layoverMarkers, setLayoverMarkers] = useState([]); // [{start_hour, duration_hours, type, label}]
  const [ganttFilter, setGanttFilter] = useState(null); // shiftId or null
  const [runtimeSecs, setRuntimeSecs] = useState(0);
  const [manualEdits, setManualEdits] = useState([]);
  const [recDecisions, setRecDecisions] = useState([]);
  const [disruptionRecoverySecs, setDisruptionRecoverySecs] = useState('');
  const [disruptionNotes, setDisruptionNotes] = useState('');
  const [overallRating, setOverallRating] = useState(0);
  const [adoptionSignal, setAdoptionSignal] = useState('');
  const [adoptionBarrier, setAdoptionBarrier] = useState('');
  const [dispatcherComments, setDispatcherComments] = useState('');
  // -- Commercial evidence fields (Stream 4) ----------------------------------
  const [orgName, setOrgName] = useState('');
  const [baselineSchedulingMins, setBaselineSchedulingMins] = useState('');
  const [baselineDisruptionMins, setBaselineDisruptionMins] = useState('');
  const [productGaps, setProductGaps] = useState('');
  const [nextSteps, setNextSteps] = useState('');
  const [willingToPilot, setWillingToPilot] = useState('');
  // Disruption simulation state
  const [disruptionType, setDisruptionType] = useState('');
  const [disruptionShiftId, setDisruptionShiftId] = useState('');
  const [disruptionReassignTo, setDisruptionReassignTo] = useState('');
  const [disruptionApplied, setDisruptionApplied] = useState([]); // [{shiftId, type, fromWorker, toWorker}]
  const [scheduleOverride, setScheduleOverride] = useState({}); // shiftId -> workerId overrides
  const [pairings, setPairings] = useState([]); // [{pairing_id, worker_id, duties, rest_compliant, fdp_compliant, total_duty_hours}]
  const [submitting, setSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState('');
  const [submitted, setSubmitted] = useState(null);

  useEffect(() => { fetchCsrfToken().then(setCsrfToken).catch(e => setCsrfError(e.message)); }, []);

  useEffect(() => {
    if (result && result.recommendations) {
      setRecDecisions(result.recommendations.map(r => ({
        recommendation_text: typeof r === 'string' ? r : (r.explanation ? `${r.explanation}: ${r.recommended_action || ''}` : JSON.stringify(r)),
        action: 'pending', rejection_reasons: [], rejection_comment: '', explanation_rating: 0,
      })));
    }
  }, [result]);

  const handleRunOptimizer = useCallback(async () => {
    setRunning(true); setRunError('');
    const t0 = Date.now();
    try {
      const { data, shifts, workers, layoverMarkers: lm, horizonHours: hz } = await runOptimizer(scenario, generationLimit, seed);
      setRuntimeSecs((Date.now() - t0) / 1000);
      setResult(data);
      setHorizonHours(hz || 336);
      // Build shift lookup: shift id → shift metadata (for Gantt)
      const sm = {};
      shifts.forEach(s => { sm[s.id] = s; });
      setShiftsMap(sm);
      // Build worker lookup: worker id → {skills, role, type_rating}
      const wm = {};
      workers.forEach(w => { wm[w.id] = w; });
      setWorkersMap(wm);
      setGanttFilter(null);

      // Call backend to generate pairings and duties (FDP-validated)
      // These produce rest-gap markers for the Gantt (layovers / deadheads)
      let ganttMarkers = lm || [];
      try {
        const analysisBody = JSON.stringify({ schedule: data.schedule, shifts, workers });
        const [pairRes, dutyRes] = await Promise.all([
          fetch(`${API_BASE}/api/pairings`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: analysisBody }),
          fetch(`${API_BASE}/api/duties`,   { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: analysisBody }),
        ]);
        if (dutyRes.ok) {
          const dutyData = await dutyRes.json();
          // Group duties by worker and build rest-gap markers
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
        // Store pairings for the Pairing Constraints step
        if (pairRes.ok) {
          const pairData = await pairRes.json();
          setPairings(pairData.pairings || []);
        }
      } catch (analysisErr) {
        console.warn('Backend analysis unavailable, using static markers:', analysisErr.message);
      }

      setLayoverMarkers(ganttMarkers);

      // Seed recommendation decisions from server response
      if (data.recommendations && data.recommendations.length > 0) {
        setRecDecisions(data.recommendations.map(r => ({
          recommendation_text: typeof r === 'string' ? r : (r.text || r.recommendation || JSON.stringify(r)),
          action: 'pending', rejection_reasons: [], rejection_comment: '', explanation_rating: 0,
        })));
      }
      setStep(2);
    }
    catch (e) { setRunError(e.message); }
    finally { setRunning(false); }
  }, [csrfToken, scenario, generationLimit, seed]);

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
          <div style={S.card}>
            <div style={S.cardTitle}>Session summary</div>
            <div style={S.cardSub}>Thank you — your session is complete.</div>
            <table style={S.summaryTable}><tbody>
              <tr><td style={S.summaryTd}>Evidence ID</td><td style={S.summaryTdVal}>{submitted.id}</td></tr>
              <tr><td style={S.summaryTd}>Timestamp</td><td style={S.summaryTdVal}>{submitted.timestamp}</td></tr>
              <tr><td style={S.summaryTd}>Dispatcher</td><td style={S.summaryTdVal}>{submitted.dispatcher_id} · {submitted.dispatcher_role}</td></tr>
              <tr><td style={S.summaryTd}>Scenario</td><td style={S.summaryTdVal}>{submitted.scenario_id}</td></tr>
              <tr><td style={S.summaryTd}>Coverage</td><td style={S.summaryTdVal}>{submitted.coverage_pct.toFixed(1)}%</td></tr>
              <tr><td style={S.summaryTd}>Hard violations</td><td style={S.summaryTdVal}>{submitted.hard_violations}</td></tr>
              <tr><td style={S.summaryTd}>Rest violations</td><td style={S.summaryTdVal}>{submitted.rest_violations}</td></tr>
              <tr><td style={S.summaryTd}>Recommendations presented</td><td style={S.summaryTdVal}>{presented}</td></tr>
              <tr><td style={S.summaryTd}>Accepted</td><td style={S.summaryTdVal}>{submitted.recommendations_accepted}</td></tr>
              <tr><td style={S.summaryTd}>Rejected</td><td style={S.summaryTdVal}>{submitted.recommendations_rejected}</td></tr>
              <tr><td style={S.summaryTd}>Override rate</td><td style={S.summaryTdVal}>{overrideRate}%</td></tr>
              <tr><td style={S.summaryTd}>Manual edits</td><td style={S.summaryTdVal}>{submitted.manual_edits}</td></tr>
              <tr><td style={S.summaryTd}>Avg explanation rating</td><td style={S.summaryTdVal}>{submitted.explanation_usefulness}/5</td></tr>
              {submitted.disruption_recovery_secs != null && <tr><td style={S.summaryTd}>Disruption recovery time</td><td style={S.summaryTdVal}>{submitted.disruption_recovery_secs.toFixed(0)}s</td></tr>}
              <tr><td style={S.summaryTd}>Runtime</td><td style={S.summaryTdVal}>{submitted.runtime_secs.toFixed(2)}s</td></tr>
            </tbody></table>
            <div style={{ marginTop: '20px' }}><button style={S.btn('secondary')} onClick={() => window.location.reload()}>Start New Session</button></div>
          </div>
          <div style={S.alert('success')}>Your reference number is <strong>{submitted.id}</strong>. The session facilitator may ask you a few brief follow-up questions.</div>
        </div>
      </div>
    );
  }

  return (
    <div style={S.app}>
      <div style={S.header}><div><div style={S.headerTitle}>SunAir Scheduling Review</div><div style={S.headerSub}>Dispatcher Feedback Session · Step {step + 1}/{STEPS.length}: {STEPS[step]}</div></div><span style={S.badge}>v0.1</span></div>
      <div style={S.main}>
        <div style={S.stepBar}>{STEPS.map((_, i) => <div key={i} style={S.stepDot(i === step, i < step)} />)}</div>
        {csrfError && <div style={S.alert('error')}>⚠ Cannot connect to UltraCrew server: {csrfError}. Ensure the server is running on port 3001.</div>}

        {step === 0 && (
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
            <button style={dispatcherId && dispatcherRole ? S.btn('primary') : S.btnDisabled} disabled={!dispatcherId || !dispatcherRole} onClick={() => setStep(1)}>Continue →</button>
          </div>
        )}

        {step === 1 && (
          <div style={S.card}>
            <div style={S.cardTitle}>Step 2 — Run Optimizer</div>
            <div style={S.cardSub}>Select a scenario and run the UltraCrew optimizer.</div>
            <label style={S.label}>Scenario</label>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', marginBottom: '16px' }}>
              {[
                { id: 'sunair',          label: 'SunAir Demo',          desc: '20 workers · 42 shifts · 7-day horizon · Indian airline (BOM/DEL/BLR)', available: true },
                { id: 'gerad-fixture',   label: 'GERAD Fixture',         desc: '8 crew · 10 legs · 5 duties · 5-day horizon · ORD hub (adapter test fixture)', available: true },
                { id: 'gerad-benchmark', label: 'GERAD Benchmark',       desc: '33 crew · 1,013 flights · 172 pairings · 31-day horizon · real G-2014-22 Instance 1 (Kasirzadeh, Saddoune & Soumis 2014)', available: true },
              ].map(sc => (
                <label key={sc.id} style={{ display: 'flex', alignItems: 'flex-start', gap: '10px', padding: '10px 12px', borderRadius: '6px', border: `1px solid ${scenario === sc.id ? '#3b82f6' : '#e2e8f0'}`, background: sc.available ? (scenario === sc.id ? '#eff6ff' : '#fff') : '#f8fafc', cursor: sc.available ? 'pointer' : 'not-allowed', opacity: sc.available ? 1 : 0.55 }}>
                  <input type="radio" name="scenario" value={sc.id} checked={scenario === sc.id} disabled={!sc.available} onChange={() => sc.available && setScenario(sc.id)} style={{ marginTop: '3px', accentColor: '#3b82f6' }} />
                  <div>
                    <div style={{ fontWeight: 600, fontSize: '13px', color: sc.available ? '#1e293b' : '#94a3b8' }}>{sc.label}{!sc.available && <span style={{ marginLeft: '8px', fontSize: '11px', color: '#f59e0b', fontWeight: 500 }}>⚠ Not yet available</span>}</div>
                    <div style={{ fontSize: '12px', color: '#64748b', marginTop: '2px' }}>{sc.desc}</div>
                  </div>
                </label>
              ))}
            </div>
            <label style={S.label}>Generation limit</label>
            <input style={S.input} type="number" value={generationLimit} onChange={e => setGenerationLimit(parseInt(e.target.value) || 500)} min={10} max={2000} />
            <label style={S.label}>Random seed</label>
            <input style={S.input} type="number" value={seed} onChange={e => setSeed(parseInt(e.target.value) || 42)} />
            {runError && <div style={S.alert('error')}>{runError}</div>}
            <div style={{ display: 'flex', gap: '8px' }}>
              <button style={S.btn('secondary')} onClick={() => setStep(0)}>← Back</button>
              <button style={running || !csrfToken ? S.btnDisabled : S.btn('primary')} disabled={running || !csrfToken} onClick={handleRunOptimizer}>{running ? '⏳ Running optimizer…' : '▶ Run Optimizer'}</button>
            </div>
          </div>
        )}

        {step === 2 && result && (
          <div>
            <div style={S.card}>
              <div style={S.cardTitle}>Step 3 — Review Schedule</div>
              <div style={S.cardSub}>Review the proposed schedule below. If you would change anything in practice, use the button at the bottom to record what and why.</div>
              {/* ── Implicit pairing constraint status — shown inline, not as a separate step ── */}
              {pairings.length > 0 && (() => {
                // rest_compliant: a pairing is rest-compliant if rest_before_next_hours is null
                // (it is the last pairing for that worker) or >= 10h (DGCA minimum crew rest).
                const isPairingRestCompliant = p =>
                  p.rest_before_next_hours == null || p.rest_before_next_hours >= 10;
                const hardViolations = pairings.filter(p => !isPairingRestCompliant(p) || !p.fdp_compliant);
                if (hardViolations.length === 0) return (
                  <div style={S.alert('success')}>
                    ✓ All {pairings.length} pairings satisfy rest and FDP requirements — schedule is legally dispatchable.
                  </div>
                );
                return (
                  <div style={S.alert('error')}>
                    ⚠ <strong>{hardViolations.length} of {pairings.length} pairings</strong> violate crew rest or FDP limits.
                    Affected flights <strong>cannot depart</strong> until violations are resolved.
                    {' '}Violations: {hardViolations.map((p, i) => {
                      const wMeta = workersMap[p.worker_id] || {};
                      const label = wMeta.name || `W${p.worker_id}`;
                      const reasons = [!isPairingRestCompliant(p) && 'rest', !p.fdp_compliant && 'FDP'].filter(Boolean).join('+');
                      return <span key={i} style={{ marginLeft: '6px', fontFamily: 'monospace', fontSize: '11px' }}>[{label} · {reasons}]</span>;
                    })}
                  </div>
                );
              })()}
              <div style={S.kpiGrid}>
                <div style={S.kpiCard}><div style={S.kpiValue}>{result.schedule ? Object.keys(result.schedule).length : '—'}</div><div style={S.kpiLabel}>Shifts assigned</div></div>
                <div style={S.kpiCard}><div style={S.kpiValue}>{result.constraint_report ? result.constraint_report.hard_violations : '—'}</div><div style={S.kpiLabel}>Rule violations</div></div>
                <div style={S.kpiCard}><div style={S.kpiValue}>{result.constraint_report ? result.constraint_report.rest_violations : '—'}</div><div style={S.kpiLabel}>Rest violations</div></div>
                <div style={S.kpiCard}><div style={S.kpiValue}>{runtimeSecs.toFixed(2)}s</div><div style={S.kpiLabel}>Generated in</div></div>
              </div>
              {result.schedule && Object.keys(result.schedule).length > 0 && (() => {
                // Build per-worker shift lists for Gantt rendering
                const HORIZON_HRS = horizonHours;
                const workerShifts = {}; // workerId -> [{shiftId, start_hour, duration_hours, name}]
                Object.entries(result.schedule).forEach(([shiftId, workerId]) => {
                  const sid = parseInt(shiftId);
                  const meta = shiftsMap[sid] || {};
                  const day = Math.floor((sid - 1) / 3) + 1;
                  const slotNames = ['Morning', 'Afternoon', 'Night'];
                  const slot = (sid - 1) % 3;
                  const name = `Flt ${String(sid).padStart(2, '0')} · Day ${day} ${slotNames[slot]}`;
                  if (!workerShifts[workerId]) workerShifts[workerId] = [];
                  workerShifts[workerId].push({ shiftId: sid, start_hour: meta.start_hour || 0, duration_hours: meta.duration_hours || 8, name });
                });
                const workerIds = Object.keys(workerShifts).map(Number).sort((a, b) => a - b);
                // Day tick marks every 24 hours — dynamic based on actual horizon
                const numDays = Math.ceil(HORIZON_HRS / 24) + 1;
                const dayTicks = Array.from({ length: numDays }, (_, i) => i);
                return (
                  <div style={{ marginBottom: '20px' }}>
                    <div style={{ fontSize: '13px', fontWeight: '600', color: '#94a3b8', marginBottom: '10px', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
                      Schedule — {workerIds.length} workers · {Object.keys(result.schedule).length} shifts · {Math.ceil(HORIZON_HRS / 24)}-day horizon
                    </div>
                    {/* Day header */}
                    <div style={{ display: 'flex', marginLeft: '72px', marginBottom: '4px', position: 'relative', height: '18px' }}>
                      {dayTicks.map(d => (
                        <div key={d} style={{ position: 'absolute', left: `${(d * 24 / HORIZON_HRS) * 100}%`, fontSize: '10px', color: '#475569', transform: 'translateX(-50%)' }}>
                          {d === 0 ? 'Day 1' : d % 2 === 0 ? `D${d + 1}` : ''}
                        </div>
                      ))}
                    </div>
                    {/* Gantt rows */}
                    <div style={{ maxHeight: '340px', overflowY: 'auto', border: '1px solid #1e293b', borderRadius: '8px', background: '#0f172a' }}>
                      {/* When a flight is selected, show only the worker assigned to it */}
                      {(ganttFilter
                        ? workerIds.filter(wid => Number(result.schedule[ganttFilter]) === wid)
                        : workerIds
                      ).map(wid => {
                        const wMeta = workersMap[wid] || {};
                        // SunAir workers have role/type_rating; GERAD workers have skills array
                        const role = wMeta.role || '';
                        const typeRating = wMeta.type_rating || '';
                        const skill = (!role && !typeRating && wMeta.skills && wMeta.skills.length > 0) ? wMeta.skills[0] : '';
                        // For GERAD: derive short label from skill string (e.g. "A320-CPT" → "CPT", "A320-FO" → "FO")
                        const skillShort = skill ? skill.replace(/^[^-]+-/, '') : '';
                        const skillFull = skill || (typeRating ? `${typeRating}` : '');
                        return (
                        <div key={wid} style={{ display: 'flex', alignItems: 'center', borderBottom: '1px solid #1e293b', minHeight: '44px' }}>
                          {/* Worker label — ID / role / type rating / skill */}
                          <div style={{ width: '80px', flexShrink: 0, paddingLeft: '8px', paddingRight: '4px' }}>
                            <div style={{ fontSize: '11px', color: '#94a3b8', fontWeight: '700' }}>W{wid}</div>
                            {role && <div style={{ fontSize: '9px', color: '#64748b', marginTop: '1px', lineHeight: '1.2' }}>{role}</div>}
                            {skillFull && <div style={{ fontSize: '9px', color: '#38bdf8', marginTop: '1px', lineHeight: '1.2', fontFamily: 'monospace' }}>{skillFull}</div>}
                            {skillShort && !skillFull && <div style={{ fontSize: '9px', color: '#38bdf8', marginTop: '1px', lineHeight: '1.2', fontFamily: 'monospace' }}>{skillShort}</div>}
                          </div>
                          {/* Timeline track */}
                          <div style={{ flex: 1, position: 'relative', height: '28px', background: '#1e293b' }}>
                            {/* Day grid lines */}
                            {dayTicks.slice(1).map(d => (
                              <div key={d} style={{ position: 'absolute', left: `${(d * 24 / HORIZON_HRS) * 100}%`, top: 0, bottom: 0, width: '1px', background: '#334155' }} />
                            ))}
                            {/* Shift bars */}
                            {workerShifts[wid].map(s => {
                              const leftPct = (s.start_hour / HORIZON_HRS) * 100;
                              const widthPct = (s.duration_hours / HORIZON_HRS) * 100;
                              const slotColors = { Morning: '#3b82f6', Afternoon: '#8b5cf6', Night: '#06b6d4' };
                              const colorKey = s.name.includes('Morning') ? 'Morning' : s.name.includes('Afternoon') ? 'Afternoon' : 'Night';
                              const isFiltered = ganttFilter === s.shiftId;
                              const color = isFiltered ? '#f59e0b' : slotColors[colorKey];
                              const shiftMeta = shiftsMap[s.shiftId] || {};
                              const aircraft = shiftMeta.aircraft_type || '';
                              const flightId = shiftMeta.flight_id || '';
                              const route = shiftMeta.route || '';
                              const crewRole = shiftMeta.crew_role || '';
                              // For GERAD shifts: use required_skill as route descriptor
                              const reqSkill = shiftMeta.required_skill || s.required_skill || '';
                              const routeDisplay = route || (reqSkill ? `Skill: ${reqSkill}` : '');
                              const workerSkillDisplay = skillFull || typeRating || '';
                              const tooltip = [
                                flightId ? `${flightId}${routeDisplay ? ` · ${routeDisplay}` : ''}` : `Shift ${s.shiftId}${routeDisplay ? ` · ${routeDisplay}` : ''}`,
                                aircraft ? `Aircraft: ${aircraft}` : '',
                                crewRole ? `Position: ${crewRole}` : '',
                                `Worker W${wid}${workerSkillDisplay ? ` · ${workerSkillDisplay}` : ''}`,
                                `Start: h${s.start_hour}  Duration: ${s.duration_hours}h`,
                                isFiltered ? 'Click to clear filter' : 'Click to filter crew for this shift',
                              ].filter(Boolean).join('\n');
                              return (
                                <div key={s.shiftId} title={tooltip}
                                  onClick={() => setGanttFilter(prev => prev === s.shiftId ? null : s.shiftId)}
                                  style={{
                                    position: 'absolute', left: `${leftPct}%`, width: `${widthPct}%`,
                                    top: '3px', bottom: '3px', background: color, borderRadius: '3px',
                                    display: 'flex', alignItems: 'center', justifyContent: 'center',
                                    overflow: 'hidden', cursor: 'pointer',
                                    outline: isFiltered ? '2px solid #fbbf24' : 'none',
                                    boxShadow: isFiltered ? '0 0 0 2px #78350f' : 'none',
                                  }}>
                                  <span style={{ fontSize: '9px', color: '#fff', fontWeight: '700', whiteSpace: 'nowrap', padding: '0 3px', textOverflow: 'ellipsis', overflow: 'hidden' }}>
                                    {flightId || s.name.split(' · ')[0]}
                                  </span>
                                </div>
                              );
                            })}
                            {/* Layover / deadhead markers — supports both flight_id-based (SunAir) and worker_id-based (GERAD backend) markers */}
                            {layoverMarkers
                              .filter(m => {
                                // New backend markers: keyed by worker_id directly
                                if (m.worker_id !== undefined) return m.worker_id === wid;
                                // Legacy SunAir markers: keyed by flight_id
                                if (!m.flight_id) return false;
                                return workerShifts[wid].some(s => {
                                  const sm = shiftsMap[s.shiftId] || {};
                                  return sm.flight_id === m.flight_id;
                                });
                              })
                              .map((m, mi) => {
                                const lLeftPct = (m.start_hour / HORIZON_HRS) * 100;
                                const lWidthPct = Math.max((m.duration_hours / HORIZON_HRS) * 100, 0.3);
                                const isLayover = m.type === 'layover';
                                const isFdpViolation = m.fdp_violation === true;
                                // Layover: amber hatched; Deadhead: grey diagonal; FDP violation: red
                                const bgStyle = isFdpViolation
                                  ? { background: 'repeating-linear-gradient(45deg, #7f1d1d 0px, #7f1d1d 3px, #991b1b 3px, #991b1b 6px)' }
                                  : isLayover
                                    ? { background: 'repeating-linear-gradient(45deg, #78350f 0px, #78350f 3px, #92400e 3px, #92400e 6px)' }
                                    : { background: 'repeating-linear-gradient(-45deg, #1e3a5f 0px, #1e3a5f 3px, #1e40af 3px, #1e40af 6px)' };
                                const borderColor = isFdpViolation ? '#ef4444' : isLayover ? '#f59e0b' : '#3b82f6';
                                const labelColor = isFdpViolation ? '#fca5a5' : isLayover ? '#fde68a' : '#93c5fd';
                                const typeLabel = isFdpViolation ? '⚠ FDP' : isLayover ? '🛏 Layover' : '✈ Deadhead';
                                return (
                                  <div key={`lm-${mi}`}
                                    title={`${typeLabel}: ${m.label}\nDuration: ${m.duration_hours}h\nStart: h${m.start_hour}${isFdpViolation ? '\n⚠ FDP rest violation' : ''}`}
                                    style={{
                                      position: 'absolute', left: `${lLeftPct}%`, width: `${lWidthPct}%`,
                                      top: '2px', bottom: '2px', borderRadius: '3px',
                                      ...bgStyle,
                                      border: `1px solid ${borderColor}`,
                                      display: 'flex', alignItems: 'center', justifyContent: 'center',
                                      overflow: 'hidden', cursor: 'default', opacity: 0.9,
                                      zIndex: 2,
                                    }}>
                                    <span style={{ fontSize: '8px', color: labelColor, fontWeight: '700', whiteSpace: 'nowrap', padding: '0 3px', textOverflow: 'ellipsis', overflow: 'hidden' }}>
                                      {m.label}
                                    </span>
                                  </div>
                                );
                              })}
                          </div>
                        </div>
                        );
                      })}
                    </div>
                    {ganttFilter && (
                      <div style={{ marginTop: '6px', fontSize: '11px', color: '#f59e0b', display: 'flex', alignItems: 'center', gap: '8px' }}>
                        <span>Showing crew for {(() => { const sm = shiftsMap[ganttFilter] || {}; return sm.flight_id || `Shift ${ganttFilter}`; })()}</span>
                        <button onClick={() => setGanttFilter(null)} style={{ background: 'none', border: '1px solid #78350f', borderRadius: '4px', color: '#f59e0b', fontSize: '10px', padding: '1px 6px', cursor: 'pointer' }}>Clear filter</button>
                      </div>
                    )}
                    <div style={{ display: 'flex', gap: '12px', marginTop: '8px', fontSize: '11px', color: '#64748b', flexWrap: 'wrap' }}>
                      <span><span style={{ display: 'inline-block', width: '10px', height: '10px', background: '#3b82f6', borderRadius: '2px', marginRight: '4px' }} />Duty (Morning)</span>
                      <span><span style={{ display: 'inline-block', width: '10px', height: '10px', background: '#8b5cf6', borderRadius: '2px', marginRight: '4px' }} />Duty (Afternoon)</span>
                      <span><span style={{ display: 'inline-block', width: '10px', height: '10px', background: '#06b6d4', borderRadius: '2px', marginRight: '4px' }} />Duty (Night)</span>
                      <span><span style={{ display: 'inline-block', width: '10px', height: '10px', background: 'repeating-linear-gradient(45deg,#78350f 0,#78350f 3px,#92400e 3px,#92400e 6px)', border: '1px solid #f59e0b', borderRadius: '2px', marginRight: '4px' }} />🛏 Layover (rest ≥10h)</span>
                      <span><span style={{ display: 'inline-block', width: '10px', height: '10px', background: 'repeating-linear-gradient(-45deg,#1e3a5f 0,#1e3a5f 3px,#1e40af 3px,#1e40af 6px)', border: '1px solid #3b82f6', borderRadius: '2px', marginRight: '4px' }} />✈ Deadhead (short rest)</span>
                      <span><span style={{ display: 'inline-block', width: '10px', height: '10px', background: 'repeating-linear-gradient(45deg,#7f1d1d 0,#7f1d1d 3px,#991b1b 3px,#991b1b 6px)', border: '1px solid #ef4444', borderRadius: '2px', marginRight: '4px' }} />⚠ FDP violation</span>
                    </div>
                  </div>
                );
              })()}
              <div style={S.divider} />
              {/* ── Disruption Simulation Panel ── */}
              {result.schedule && Object.keys(result.schedule).length > 0 && (() => {
                const effectiveSchedule = { ...result.schedule, ...scheduleOverride };
                const shiftIds = Object.keys(effectiveSchedule).map(Number).sort((a, b) => a - b);
                const allWorkers = [...new Set(Object.values(effectiveSchedule).map(Number))].sort((a, b) => a - b);

                // For a selected shift, find workers NOT already working an overlapping shift
                const getAvailableWorkers = (targetShiftId) => {
                  const meta = shiftsMap[targetShiftId] || {};
                  const tStart = meta.start_hour || 0;
                  const tEnd = tStart + (meta.duration_hours || 8);
                  return allWorkers.filter(wid => {
                    if (wid === Number(effectiveSchedule[targetShiftId])) return false; // already assigned
                    return shiftIds.every(sid => {
                      if (Number(effectiveSchedule[sid]) !== wid) return true; // not this worker's shift
                      const sm = shiftsMap[sid] || {};
                      const sStart = sm.start_hour || 0;
                      const sEnd = sStart + (sm.duration_hours || 8);
                      return tEnd <= sStart || tStart >= sEnd; // no overlap
                    });
                  });
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
                  // Auto-record as a manual edit
                  const slotNames = ['Morning', 'Afternoon', 'Night'];
                  const slot = (sid - 1) % 3;
                  const day = Math.floor((sid - 1) / 3) + 1;
                  const shiftName = `Flt ${String(sid).padStart(2, '0')} · Day ${day} ${slotNames[slot]}`;
                  const reasonText = disruptionType === 'delay' ? 'Weather/delay' : disruptionType === 'sick_call' ? 'Crew unavailability' : 'Local knowledge / operational requirement';
                  addManualEdit();
                  setManualEdits(prev => {
                    const updated = [...prev];
                    updated[updated.length - 1] = { reason: reasonText, comment: `Disruption: reassigned ${shiftName} from Worker ${fromWorker} to Worker ${toWorker}` };
                    return updated;
                  });
                  setDisruptionShiftId('');
                  setDisruptionReassignTo('');
                };

                const availableWorkers = disruptionShiftId ? getAvailableWorkers(parseInt(disruptionShiftId)) : [];

                return (
                  <div style={{ marginBottom: '20px', border: '1px solid #f59e0b', borderRadius: '8px', padding: '16px', background: '#1c1a0e' }}>
                    <div style={{ fontSize: '13px', fontWeight: '700', color: '#f59e0b', marginBottom: '12px' }}>⚡ Disruption Simulator</div>
                    <div style={{ display: 'flex', gap: '12px', flexWrap: 'wrap', alignItems: 'flex-end', marginBottom: '12px' }}>
                      <div>
                        <div style={{ fontSize: '11px', color: '#94a3b8', marginBottom: '4px' }}>Event type</div>
                        <select value={disruptionType} onChange={e => setDisruptionType(e.target.value)} style={{ ...S.select, minWidth: '140px' }}>
                          <option value="">— select —</option>
                          <option value="delay">Flight delay</option>
                          <option value="sick_call">Crew sick call</option>
                          <option value="aircraft_swap">Aircraft swap</option>
                          <option value="atc_hold">ATC hold</option>
                        </select>
                      </div>
                      <div>
                        <div style={{ fontSize: '11px', color: '#94a3b8', marginBottom: '4px' }}>Affected shift</div>
                        <select value={disruptionShiftId} onChange={e => { setDisruptionShiftId(e.target.value); setDisruptionReassignTo(''); }} style={{ ...S.select, minWidth: '160px' }}>
                          <option value="">— select shift —</option>
                          {shiftIds.map(sid => {
                            const slotNames = ['Morning', 'Afternoon', 'Night'];
                            const slot = (sid - 1) % 3;
                            const day = Math.floor((sid - 1) / 3) + 1;
                            return <option key={sid} value={sid}>Flt {String(sid).padStart(2, '0')} · Day {day} {slotNames[slot]} (W{effectiveSchedule[sid]})</option>;
                          })}
                        </select>
                      </div>
                      {disruptionShiftId && (
                        <div>
                          <div style={{ fontSize: '11px', color: '#94a3b8', marginBottom: '4px' }}>
                            Reassign to {availableWorkers.length === 0 ? <span style={{ color: '#ef4444' }}>(none available)</span> : ''}
                          </div>
                          <select value={disruptionReassignTo} onChange={e => setDisruptionReassignTo(e.target.value)} style={{ ...S.select, minWidth: '140px' }} disabled={availableWorkers.length === 0}>
                            <option value="">— select worker —</option>
                            {availableWorkers.map(wid => <option key={wid} value={wid}>Worker {wid}</option>)}
                          </select>
                        </div>
                      )}
                      <button onClick={applyReassignment} disabled={!disruptionShiftId || !disruptionReassignTo} style={!disruptionShiftId || !disruptionReassignTo ? S.btnDisabled : { ...S.btn('primary'), background: '#d97706' }}>
                        Apply reassignment
                      </button>
                    </div>
                    {disruptionApplied.length > 0 && (
                      <div style={{ fontSize: '12px', color: '#94a3b8' }}>
                        <div style={{ fontWeight: '600', marginBottom: '4px' }}>Applied disruptions:</div>
                        {disruptionApplied.map((d, i) => (
                          <div key={i} style={{ color: '#f59e0b' }}>
                            ✓ {d.type} — Flt {String(d.shiftId).padStart(2, '0')}: Worker {d.fromWorker} → Worker {d.toWorker}
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                );
              })()}

              <div style={{ marginBottom: '12px' }}>
                <strong style={{ fontSize: '14px', color: '#f1f5f9' }}>Would you change anything?</strong>
                <span style={{ fontSize: '13px', color: '#64748b', marginLeft: '8px' }}>Record each change you would make in practice and why.</span>
              </div>
              {manualEdits.map((edit, idx) => (
                <div key={idx} style={S.editCard}>
                  <label style={S.label}>Reason for edit</label>
                  <select style={{ ...S.select, marginBottom: '8px' }} value={edit.reason} onChange={e => updateManualEdit(idx, { reason: e.target.value })}>
                    <option value="">Select reason…</option>
                    {MANUAL_EDIT_REASONS.map(r => <option key={r} value={r}>{r}</option>)}
                  </select>
                  <label style={S.label}>Comment (optional)</label>
                  <input style={{ ...S.input, marginBottom: '8px' }} value={edit.comment} onChange={e => updateManualEdit(idx, { comment: e.target.value })} placeholder="What did you change?" />
                  <button style={{ ...S.btn('danger'), fontSize: '12px', padding: '6px 12px' }} onClick={() => removeManualEdit(idx)}>Remove</button>
                </div>
              ))}
              <button style={S.btn('secondary')} onClick={addManualEdit}>+ Record Manual Edit</button>
            </div>
            <div style={{ display: 'flex', gap: '8px' }}>
              <button style={S.btn('secondary')} onClick={() => setStep(1)}>← Back</button>
              <button style={S.btn('primary')} onClick={() => setStep(3)}>Continue →</button>
            </div>
          </div>
        )}{step === 3 && (
          <div>
            <div style={S.card}>
              <div style={S.cardTitle}>Step 4 — Recommendation Decisions</div>
              <div style={S.cardSub}>
                For each recommendation, decide whether to accept, reject, or modify it. Your decision and reasoning are captured as evidence automatically.
              </div>
              {recDecisions.length === 0 && <div style={S.alert('info')}>No recommendations were generated for this scenario.</div>}
              {recDecisions.map((d, i) => (
                <div key={i} style={S.recCard(d.action)}>
                  <div style={S.recText}>{d.recommendation_text}</div>
                  <div style={S.recActions}>
                    <button style={{ ...S.btn(d.action === 'accepted' ? 'success' : 'secondary'), opacity: d.action === 'accepted' ? 1 : 0.65 }} onClick={() => updateRec(i, { action: 'accepted' })}>✓ Accept</button>
                    <button style={{ ...S.btn(d.action === 'rejected' ? 'danger' : 'secondary'), opacity: d.action === 'rejected' ? 1 : 0.65 }} onClick={() => updateRec(i, { action: 'rejected' })}>✗ Reject</button>
                    <button style={{ ...S.btn(d.action === 'modified' ? 'primary' : 'secondary'), opacity: d.action === 'modified' ? 1 : 0.65 }} onClick={() => updateRec(i, { action: 'modified' })}>✎ Modify</button>
                  </div>
                  {(d.action === 'rejected' || d.action === 'modified') && (
                    <div style={S.recSubCard}>
                      <div style={S.recSubLabel}>Why?</div>
                      {REJECTION_REASONS.map(r => (
                        <label key={r} style={S.checkRow}>
                          <input type="checkbox" checked={d.rejection_reasons.includes(r)} onChange={e => toggleRejectionReason(i, r, e.target.checked)} />
                          <span style={S.checkLabel}>{r}</span>
                        </label>
                      ))}
                      <input style={{ ...S.input, marginTop: '8px', marginBottom: '0' }} value={d.rejection_comment} onChange={e => updateRec(i, { rejection_comment: e.target.value })} placeholder="Additional comments…" />
                    </div>
                  )}
                  <div style={{ ...S.recSubCard, marginTop: '10px' }}>
                    <StarRating label="Was this explanation useful?" value={d.explanation_rating} onChange={v => updateRec(i, { explanation_rating: v })} />
                  </div>
                </div>
              ))}
            </div>
            <div style={{ display: 'flex', gap: '8px' }}>
              <button style={S.btn('secondary')} onClick={() => setStep(2)}>← Back</button>
              <button style={S.btn('primary')} onClick={() => setStep(4)}>Continue →</button>
            </div>
          </div>
        )}

        {step === 4 && (
          <div>
            <div style={S.card}>
              <div style={S.cardTitle}>Step 5 — Disruption Recovery (Optional)</div>
              <div style={S.cardSub}>
                If a disruption scenario was run during this session, record the time from the disruption event to an accepted recovery plan. Leave blank if not applicable.
              </div>
              <label style={S.label}>Time to accepted recovery plan (seconds)</label>
              <input style={S.input} type="number" value={disruptionRecoverySecs} onChange={e => setDisruptionRecoverySecs(e.target.value)} placeholder="e.g. 180 (leave blank if not applicable)" min={0} />
              <label style={S.label}>Notes on disruption scenario (optional)</label>
              <textarea style={S.textarea} value={disruptionNotes} onChange={e => setDisruptionNotes(e.target.value)} placeholder="What was the disruption? How did the system respond?" />
              <div style={S.alert('info')}>
                Disruption recovery time is the primary operational evidence dimension for UltraCrew. Even an approximate value is useful.
              </div>
            </div>
            <div style={{ display: 'flex', gap: '8px' }}>
              <button style={S.btn('secondary')} onClick={() => setStep(3)}>← Back</button>
              <button style={S.btn('primary')} onClick={() => setStep(5)}>Continue →</button>
            </div>
          </div>
        )}

        {step === 5 && (
          <div>
            <div style={S.card}>
              <div style={S.cardTitle}>Step 6 — Session Debrief</div>
              <div style={S.cardSub}>
                A few final questions about your experience with the scheduling tool today.
              </div>

              <label style={S.label}>Overall satisfaction with UltraCrew</label>
              <StarRating value={overallRating} onChange={setOverallRating} />

              <div style={S.divider} />

              <label style={S.label}>Would you use this for tomorrow's roster?</label>
              <div style={{ display: 'flex', gap: '10px', flexWrap: 'wrap', marginBottom: '16px' }}>
                {ADOPTION_OPTIONS.map(o => (
                  <button key={o.value} style={S.radioBtn(adoptionSignal === o.value)} onClick={() => setAdoptionSignal(o.value)}>{o.label}</button>
                ))}
              </div>

              {adoptionSignal && adoptionSignal !== 'yes' && (
                <div>
                  <label style={S.label}>What prevented a stronger answer?</label>
                  <textarea style={S.textarea} value={adoptionBarrier} onChange={e => setAdoptionBarrier(e.target.value)} placeholder="What would need to change for you to use this tomorrow?" />
                </div>
              )}

              <div style={S.divider} />

              <label style={S.label}>Any other comments?</label>
              <textarea style={S.textarea} value={dispatcherComments} onChange={e => setDispatcherComments(e.target.value)} placeholder="What surprised you? What was most useful? What was missing?" />

              <div style={S.divider} />

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

              {submitError && <div style={S.alert('error')}>{submitError}</div>}
            </div>
            <div style={{ display: 'flex', gap: '8px' }}>
              <button style={S.btn('secondary')} onClick={() => setStep(4)}>← Back</button>
              <button
                style={submitting || overallRating === 0 || !adoptionSignal ? S.btnDisabled : S.btn('success')}
                disabled={submitting || overallRating === 0 || !adoptionSignal}
                onClick={handleSubmit}
              >
                {submitting ? '⏳ Submitting…' : '✓ Submit'}
              </button>
            </div>
            {(overallRating === 0 || !adoptionSignal) && (
              <div style={{ fontSize: '12px', color: '#64748b', marginTop: '8px' }}>
                Please rate your overall satisfaction and answer the adoption question to submit.
              </div>
            )}
          </div>
        )}

      </div>
    </div>
  );
}