#!/usr/bin/env python3
"""
Replaces the monolithic runOptimizer() in App.js with:
  - buildSunairScenario()
  - buildGeradFixtureScenario()
  - runOptimizer(scenarioId, generationLimit, seed)
"""
import sys

path = 'apps/ultracrew-pilot-portal/src/App.js'

with open(path, 'r', encoding='utf-8') as f:
    content = f.read()

start_marker = 'async function runOptimizer(generationLimit, seed) {'
end_marker = '  return { data, shifts, workers, layoverMarkers: LAYOVER_MARKERS };\n}'

start_idx = content.find(start_marker)
end_idx = content.find(end_marker)

if start_idx == -1:
    print('ERROR: start marker not found', file=sys.stderr)
    sys.exit(1)
if end_idx == -1:
    print('ERROR: end marker not found', file=sys.stderr)
    sys.exit(1)

end_idx += len(end_marker)
print(f'Found block: chars {start_idx}..{end_idx} '
      f'(lines {content[:start_idx].count(chr(10))+1} to {content[:end_idx].count(chr(10))+1})')

ARR = '\u2192'  # →

replacement = f'''// \u2500\u2500 Scenario builders \u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500
//
// Three scenarios are available in the portal:
//
//   sunair          \u2014 Product demo. 20 workers, 42 shifts, 7-day horizon.
//                     Synthetic Indian airline schedule (BOM/DEL/BLR/HYD/CCU/MAA).
//                     Use for dispatcher evidence collection sessions.
//
//   gerad-fixture   \u2014 Deterministic dev/test. 8 crew, 10 legs, 5 duties, 5-day horizon.
//                     Derived from adapters/gerad/tests/fixtures/ (22/22 tests pass).
//                     Block times are real (not synthetic 8h slots).
//                     HONEST LABEL: this is the adapter test fixture, NOT the research dataset.
//
//   gerad-benchmark \u2014 Optimisation validation. Requires G1422-DataSets.zip from
//                     GERAD Technical Report G-2014-22 (Kasirzadeh, Saddoune, Soumis).
//                     See benchmarks/gerad-g2014-22/README.md for acquisition instructions.
//                     GATED \u2014 not available until the dataset is downloaded and imported.

function buildSunairScenario() {{
  const SLOT_OFFSETS = [6, 14, 22];
  const WORKER_PROFILES = [
    {{ role: 'Captain',       type_rating: 'A320',      skills: ['A320-CPT'] }},
    {{ role: 'First Officer', type_rating: 'A320',      skills: ['A320-FO'] }},
    {{ role: 'Captain',       type_rating: 'A320',      skills: ['A320-CPT'] }},
    {{ role: 'First Officer', type_rating: 'A320',      skills: ['A320-FO'] }},
    {{ role: 'Captain',       type_rating: 'A320',      skills: ['A320-CPT'] }},
    {{ role: 'First Officer', type_rating: 'A320',      skills: ['A320-FO'] }},
    {{ role: 'Captain',       type_rating: 'A320',      skills: ['A320-CPT'] }},
    {{ role: 'First Officer', type_rating: 'A320',      skills: ['A320-FO'] }},
    {{ role: 'Captain',       type_rating: 'A320',      skills: ['A320-CPT'] }},
    {{ role: 'First Officer', type_rating: 'A320',      skills: ['A320-FO'] }},
    {{ role: 'Captain',       type_rating: 'B737',      skills: ['B737-CPT'] }},
    {{ role: 'First Officer', type_rating: 'B737',      skills: ['B737-FO'] }},
    {{ role: 'Captain',       type_rating: 'B737',      skills: ['B737-CPT'] }},
    {{ role: 'First Officer', type_rating: 'B737',      skills: ['B737-FO'] }},
    {{ role: 'Captain',       type_rating: 'ATR72',     skills: ['ATR72-CPT'] }},
    {{ role: 'First Officer', type_rating: 'ATR72',     skills: ['ATR72-FO'] }},
    {{ role: 'Cabin Crew',    type_rating: 'All fleet', skills: ['CabinCrew'] }},
    {{ role: 'Cabin Crew',    type_rating: 'All fleet', skills: ['CabinCrew'] }},
    {{ role: 'Cabin Crew',    type_rating: 'All fleet', skills: ['CabinCrew'] }},
    {{ role: 'Cabin Crew',    type_rating: 'All fleet', skills: ['CabinCrew'] }},
  ];
  const workers = WORKER_PROFILES.map((p, i) => ({{ id: i + 1, skills: p.skills, role: p.role, type_rating: p.type_rating }}));
  const flights = [
    {{ id: 'SA101', aircraft: 'A320',  day: 0,  slot: 0, duration: 8, route: 'BOM{ARR}DEL' }},
    {{ id: 'SA102', aircraft: 'A320',  day: 2,  slot: 1, duration: 6, route: 'DEL{ARR}BLR' }},
    {{ id: 'SA103', aircraft: 'A320',  day: 4,  slot: 0, duration: 8, route: 'BLR{ARR}HYD' }},
    {{ id: 'SA104', aircraft: 'A320',  day: 6,  slot: 2, duration: 7, route: 'HYD{ARR}CCU' }},
    {{ id: 'SA105', aircraft: 'A320',  day: 8,  slot: 1, duration: 6, route: 'CCU{ARR}BOM' }},
    {{ id: 'SA201', aircraft: 'B737',  day: 1,  slot: 0, duration: 9, route: 'BOM{ARR}MAA' }},
    {{ id: 'SA202', aircraft: 'B737',  day: 3,  slot: 2, duration: 8, route: 'MAA{ARR}AMD' }},
    {{ id: 'SA203', aircraft: 'B737',  day: 5,  slot: 1, duration: 7, route: 'AMD{ARR}PNQ' }},
    {{ id: 'SA204', aircraft: 'B737',  day: 9,  slot: 0, duration: 8, route: 'PNQ{ARR}BOM' }},
    {{ id: 'SA301', aircraft: 'ATR72', day: 7,  slot: 0, duration: 4, route: 'GOI{ARR}BOM' }},
    {{ id: 'SA302', aircraft: 'ATR72', day: 11, slot: 1, duration: 4, route: 'BOM{ARR}GOI' }},
  ];
  const CC_REQUIRED = {{ A320: 2, B737: 2, ATR72: 1 }};
  const shifts = [];
  let shiftId = 1;
  for (const flt of flights) {{
    const startHour = flt.day * 24 + SLOT_OFFSETS[flt.slot];
    const base = {{ start_hour: startHour, duration_hours: flt.duration, aircraft_type: flt.aircraft, flight_id: flt.id, route: flt.route }};
    shifts.push({{ id: shiftId++, ...base, required_skill: `${{flt.aircraft}}-CPT`, crew_role: 'Captain' }});
    shifts.push({{ id: shiftId++, ...base, required_skill: `${{flt.aircraft}}-FO`,  crew_role: 'First Officer' }});
    const ccCount = CC_REQUIRED[flt.aircraft] || 1;
    for (let c = 0; c < ccCount; c++) shifts.push({{ id: shiftId++, ...base, required_skill: 'CabinCrew', crew_role: 'Cabin Crew' }});
  }}
  const layoverMarkers = [
    {{ start_hour: 14,  duration_hours: 16, type: 'layover',  label: 'Layover DEL', flight_id: 'SA101' }},
    {{ start_hour: 39,  duration_hours: 15, type: 'layover',  label: 'Layover MAA', flight_id: 'SA201' }},
    {{ start_hour: 166, duration_hours: 4,  type: 'deadhead', label: 'DH BOM{ARR}GOI',  flight_id: 'SA301' }},
    {{ start_hour: 94,  duration_hours: 8,  type: 'layover',  label: 'Layover AMD', flight_id: 'SA202' }},
  ];
  return {{ workers, shifts, layoverMarkers, horizonHours: 336, maxHoursPerWorker: 48 }};
}}

// GERAD Fixture \u2014 adapter test dataset (honest label: NOT the research benchmark).
// Source: adapters/gerad/tests/fixtures/ (5 CSV files, 22/22 tests pass).
// 8 crew, 10 legs, 5 duties, 5-day horizon (2014-01-06 to 2014-01-10).
// start_hour = hours since 2014-01-06T00:00 UTC.
// Block times derived from fixture scheduled_departure / scheduled_arrival.
// All crew based at ORD. Each duty = 2 legs (outbound + return same day).
function buildGeradFixtureScenario() {{
  const workers = [
    {{ id: 1, skills: ['B738-CPT'],            role: 'Captain',           type_rating: 'B738',      name: 'Alice Brennan',    gerad_id: 'C0001' }},
    {{ id: 2, skills: ['B738-FO'],             role: 'First Officer',     type_rating: 'B738',      name: 'Robert Okafor',    gerad_id: 'C0002' }},
    {{ id: 3, skills: ['A320-CPT'],            role: 'Captain',           type_rating: 'A320',      name: 'Priya Nair',       gerad_id: 'C0003' }},
    {{ id: 4, skills: ['A320-FO'],             role: 'First Officer',     type_rating: 'A320',      name: 'James Whitfield',  gerad_id: 'C0004' }},
    {{ id: 5, skills: ['CabinCrew'],           role: 'Cabin Crew Senior', type_rating: 'B738',      name: 'Maria Santos',     gerad_id: 'C0005' }},
    {{ id: 6, skills: ['CabinCrew'],           role: 'Cabin Crew',        type_rating: 'B738',      name: 'David Kim',        gerad_id: 'C0006' }},
    {{ id: 7, skills: ['CabinCrew'],           role: 'Cabin Crew',        type_rating: 'A320',      name: 'Fatima Al-Hassan', gerad_id: 'C0007' }},
    {{ id: 8, skills: ['B738-CPT','A320-CPT'], role: 'Captain',           type_rating: 'B738/A320', name: 'Thomas Eriksson',  gerad_id: 'C0008' }},
  ];
  const GERAD_FLIGHTS = [
    {{ id: 'FL0001', duty_id: 'D0001', aircraft: 'B738', start_hour: 8,    duration_hours: 3.5,  route: 'ORD{ARR}LAX' }},
    {{ id: 'FL0002', duty_id: 'D0001', aircraft: 'B738', start_hour: 14,   duration_hours: 5.5,  route: 'LAX{ARR}ORD' }},
    {{ id: 'FL0003', duty_id: 'D0002', aircraft: 'B738', start_hour: 31,   duration_hours: 2.25, route: 'ORD{ARR}DFW' }},
    {{ id: 'FL0004', duty_id: 'D0002', aircraft: 'B738', start_hour: 35,   duration_hours: 2.33, route: 'DFW{ARR}ORD' }},
    {{ id: 'FL0005', duty_id: 'D0003', aircraft: 'A320', start_hour: 54.5, duration_hours: 4.25, route: 'ORD{ARR}MIA' }},
    {{ id: 'FL0006', duty_id: 'D0003', aircraft: 'A320', start_hour: 61,   duration_hours: 4.25, route: 'MIA{ARR}ORD' }},
    {{ id: 'FL0007', duty_id: 'D0004', aircraft: 'B738', start_hour: 80,   duration_hours: 3,    route: 'ORD{ARR}JFK' }},
    {{ id: 'FL0008', duty_id: 'D0004', aircraft: 'B738', start_hour: 85.5, duration_hours: 3,    route: 'JFK{ARR}ORD' }},
    {{ id: 'FL0009', duty_id: 'D0005', aircraft: 'A320', start_hour: 103,  duration_hours: 2.5,  route: 'ORD{ARR}SEA' }},
    {{ id: 'FL0010', duty_id: 'D0005', aircraft: 'A320', start_hour: 107,  duration_hours: 2.5,  route: 'SEA{ARR}ORD' }},
  ];
  const shifts = [];
  let shiftId = 1;
  for (const flt of GERAD_FLIGHTS) {{
    const base = {{ start_hour: flt.start_hour, duration_hours: flt.duration_hours, aircraft_type: flt.aircraft, flight_id: flt.id, duty_id: flt.duty_id, route: flt.route }};
    shifts.push({{ id: shiftId++, ...base, required_skill: `${{flt.aircraft}}-CPT`, crew_role: 'Captain' }});
    shifts.push({{ id: shiftId++, ...base, required_skill: `${{flt.aircraft}}-FO`,  crew_role: 'First Officer' }});
    shifts.push({{ id: shiftId++, ...base, required_skill: 'CabinCrew',           crew_role: 'Cabin Crew' }});
  }}
  const layoverMarkers = [
    {{ start_hour: 11.5,  duration_hours: 2.5,  type: 'layover', label: 'Turnaround LAX', flight_id: 'FL0001' }},
    {{ start_hour: 33.25, duration_hours: 1.75, type: 'layover', label: 'Turnaround DFW', flight_id: 'FL0003' }},
    {{ start_hour: 58.75, duration_hours: 2.25, type: 'layover', label: 'Turnaround MIA', flight_id: 'FL0005' }},
    {{ start_hour: 83,    duration_hours: 2.5,  type: 'layover', label: 'Turnaround JFK', flight_id: 'FL0007' }},
    {{ start_hour: 105.5, duration_hours: 1.5,  type: 'layover', label: 'Turnaround SEA', flight_id: 'FL0009' }},
  ];
  return {{ workers, shifts, layoverMarkers, horizonHours: 120, maxHoursPerWorker: 20 }};
}}

async function runOptimizer(scenarioId, generationLimit, seed) {{
  const csrfToken = await fetchCsrfToken();
  const scenarioData = scenarioId === 'gerad-fixture' ? buildGeradFixtureScenario() : buildSunairScenario();
  const {{ workers, shifts, layoverMarkers, horizonHours, maxHoursPerWorker }} = scenarioData;
  const res = await fetch('/api/schedule', {{
    method: 'POST',
    headers: {{ 'Content-Type': 'application/json', 'X-CSRF-Token': csrfToken }},
    body: JSON.stringify({{ workers, shifts, rng_seed: seed, generation_limit: generationLimit, scenario: {{ planning_horizon_hours: horizonHours, max_hours_per_worker: maxHoursPerWorker }} }}),
  }});
  if (!res.ok) throw new Error(await res.text());
  const data = await res.json();
  return {{ data, shifts, workers, layoverMarkers }};
}}'''

new_content = content[:start_idx] + replacement + content[end_idx:]
with open(path, 'w', encoding='utf-8') as f:
    f.write(new_content)
print(f'OK: replaced {end_idx - start_idx} chars with {len(replacement)} chars')
print(f'New line count: {new_content.count(chr(10))}')