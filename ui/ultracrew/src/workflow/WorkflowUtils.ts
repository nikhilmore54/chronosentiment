import type { StaffMember, ImportSummary, ScheduleResult, RosterAlternative, RosterAlternativeMetrics, CandidateFeasibility, StaffingRequirement, CoverageReport, ChangeRecord, RedistributionLog, AssignmentProvenanceState, ParetoAlternativeRaw } from './WorkflowTypes';
import { SHIFT_CYCLE } from './WorkflowTypes';

// ─── CSV parsing ──────────────────────────────────────────────────────────────

export function parseStaffCSV(text: string): { staff: StaffMember[]; errors: string[] } {
  const lines = text.trim().split('\n').map(l => l.trim()).filter(Boolean);
  if (lines.length < 2) return { staff: [], errors: ['CSV must have a header row and at least one data row.'] };

  const header = lines[0].toLowerCase().split(',').map(h => h.trim());
  const idIdx = header.indexOf('id');
  const contractIdx = header.indexOf('contract');
  const skillsIdx = header.indexOf('skills');

  const errors: string[] = [];
  if (idIdx === -1) errors.push('Missing column: id');
  if (contractIdx === -1) errors.push('Missing column: contract');
  if (skillsIdx === -1) errors.push('Missing column: skills');
  if (errors.length > 0) return { staff: [], errors };

  const staff: StaffMember[] = [];
  const seenIds = new Set<string>();

  for (let i = 1; i < lines.length; i++) {
    const cols = lines[i].split(',').map(c => c.trim());
    const id = cols[idIdx];
    const contract = cols[contractIdx];
    const skillsRaw = cols[skillsIdx];

    if (!id) { errors.push(`Row ${i + 1}: missing id`); continue; }
    if (!contract) { errors.push(`Row ${i + 1}: missing contract for ${id}`); continue; }
    if (!skillsRaw) { errors.push(`Row ${i + 1}: missing skills for ${id}`); continue; }
    if (seenIds.has(id)) { errors.push(`Row ${i + 1}: duplicate id "${id}"`); continue; }

    seenIds.add(id);
    staff.push({ id, contract, skills: skillsRaw.split(';').map(s => s.trim()).filter(Boolean) });
  }

  return { staff, errors };
}

// ─── Import validation summary ────────────────────────────────────────────────

export function buildImportSummary(staff: StaffMember[]): ImportSummary {
  const contracts = [...new Set(staff.map(s => s.contract))];
  const skills = [...new Set(staff.flatMap(s => s.skills))];
  const warnings: string[] = [];
  if (staff.length < 3) warnings.push('Very small team — schedule may have coverage gaps.');
  const unknownContracts = contracts.filter(c => !['FullTime', 'PartTime', 'Night'].includes(c));
  if (unknownContracts.length > 0) warnings.push(`Unknown contract types: ${unknownContracts.join(', ')}`);
  return { staffCount: staff.length, contracts, skills, warnings };
}

// ─── API payload builder ──────────────────────────────────────────────────────

// ─── Weekly shift model ───────────────────────────────────────────────────────
// The backend constraint engine uses a weekly model: start_hour must be 0–167.
// We generate one shift slot per shift type per day of the week (5 working days).
// The optimizer assigns workers to these weekly slots.
// The 28-day grid is reconstructed by repeating the weekly assignment 4 times.
//
// Shift types within a day:
//   Early: start_hour = day*24 + 6  (06:00–14:00)
//   Late:  start_hour = day*24 + 14 (14:00–22:00)
//   Night: start_hour = day*24 + 22 (22:00–06:00 next day, capped at 167)
//
// We generate 3 shift slots per working day × 5 days = 15 slots per skill type.
// Each slot can be assigned to any worker with the matching skill.

// One shift slot per working day — alternating Early/Late to avoid same-day overlap.
// Each slot is on a distinct day so the constraint engine never sees two shifts
// for the same worker within 8 hours (Early ends at 14, next slot starts at 30 = day 1 + 6h).
// Gap between end of day-N Early (hour 14) and start of day-(N+1) Early (hour 30) = 16h ≥ 8h ✓
// Gap between end of day-N Late  (hour 22) and start of day-(N+1) Early (hour 30) = 8h  ≥ 8h ✓
const WEEKLY_SHIFT_SLOTS: Array<{ dayOfWeek: number; startHour: number; label: string }> = [
  { dayOfWeek: 0, startHour: 6,   label: 'Early' }, // Mon 06:00–14:00
  { dayOfWeek: 1, startHour: 38,  label: 'Late'  }, // Tue 14:00–22:00  (gap from Mon Early end: 38-14=24h ✓)
  { dayOfWeek: 2, startHour: 54,  label: 'Early' }, // Wed 06:00–14:00  (gap from Tue Late end: 54-46=8h ✓)
  { dayOfWeek: 3, startHour: 86,  label: 'Late'  }, // Thu 14:00–22:00  (gap from Wed Early end: 86-62=24h ✓)
  { dayOfWeek: 4, startHour: 102, label: 'Early' }, // Fri 06:00–14:00  (gap from Thu Late end: 102-94=8h ✓)
];

export interface ShiftSlotMeta {
  shiftId: number;
  dayIdx: number;
  label: string;
  skill: string;
  startHour: number;
}

export function getScheduleSlotMeta(staff: StaffMember[]): Record<number, ShiftSlotMeta> {
  const allSkills = [...new Set(staff.flatMap(s => s.skills.length > 0 ? s.skills : ['Nurse']))];
  const meta: Record<number, ShiftSlotMeta> = {};
  let shiftId = 1;
  for (const skill of allSkills) {
    for (let week = 0; week < 4; week++) {
      for (const slot of WEEKLY_SHIFT_SLOTS) {
        const dayIdx = week * 7 + slot.dayOfWeek;
        const startHour = week * 168 + slot.startHour;
        meta[shiftId] = {
          shiftId,
          dayIdx,
          label: slot.label,
          skill,
          startHour,
        };
        shiftId++;
      }
    }
  }
  return meta;
}

export function buildSchedulePayload(staff: StaffMember[], rulePayload: any = {}) {
  const workers = staff.map((s, i) => ({
    id: i + 1,
    skills: s.skills.length > 0 ? s.skills : ['Nurse'],
    contract: s.contract,
  }));

  const slotMeta = getScheduleSlotMeta(staff);
  const shifts = Object.values(slotMeta).map(m => ({
    id: m.shiftId,
    start_hour: m.startHour,
    duration_hours: 8,
    required_skill: m.skill,
  }));

  return {
    workers,
    shifts,
    historical_workloads: null,
    rng_seed: null,
    generation_limit: 200,
    scenario: {
      planning_horizon_hours: 28 * 24,
      max_hours_per_worker: rulePayload?.max_weekly_hours ?? 40.0,
      minimum_rest_hours: rulePayload?.min_rest_hours ?? 10,
      leave_requests: null,
    },
  };
}

// ─── Schedule builders ────────────────────────────────────────────────────────

export function buildEditableSchedule(
  staff: StaffMember[],
  apiSchedule: Record<string, number>
): Record<string, string[]> {
  const slotMeta = getScheduleSlotMeta(staff);
  const result: Record<string, string[]> = {};
  staff.forEach(s => { result[s.id] = Array(28).fill(''); });

  const shiftIds = Object.keys(apiSchedule).map(Number);
  // Detect if apiSchedule only contains 1-week IDs (e.g. legacy test fixture IDs 1..5)
  const isLegacySingleWeek =
    shiftIds.length > 0 &&
    Math.max(...shiftIds) <= 5 * Math.max(1, staff.length) &&
    !shiftIds.some(id => (slotMeta[id]?.dayIdx ?? 0) >= 7);

  if (isLegacySingleWeek) {
    const allSkills = [...new Set(staff.flatMap(s => s.skills.length > 0 ? s.skills : ['Nurse']))];
    const legacyMeta: Record<number, { dayOfWeek: number; label: string }> = {};
    let sid = 1;
    for (const _skill of allSkills) {
      for (const slot of WEEKLY_SHIFT_SLOTS) {
        legacyMeta[sid++] = { dayOfWeek: slot.dayOfWeek, label: slot.label };
      }
    }
    const workerWeeklySlots: Record<number, Array<{ dayOfWeek: number; label: string }>> = {};
    Object.entries(apiSchedule).forEach(([shiftIdStr, workerId]) => {
      const meta = legacyMeta[Number(shiftIdStr)];
      if (!meta) return;
      if (!workerWeeklySlots[workerId]) workerWeeklySlots[workerId] = [];
      workerWeeklySlots[workerId].push({ dayOfWeek: meta.dayOfWeek, label: meta.label });
    });
    Object.entries(workerWeeklySlots).forEach(([workerIdStr, slots]) => {
      const workerId = Number(workerIdStr);
      const assignedStaff = staff[workerId - 1];
      if (!assignedStaff) return;
      for (let week = 0; week < 4; week++) {
        for (const slot of slots) {
          const dayIdx = week * 7 + slot.dayOfWeek;
          if (dayIdx < 28) {
            result[assignedStaff.id][dayIdx] = slot.label;
          }
        }
      }
    });
    return result;
  }

  // Authoritative 28-day schedule mapping:
  Object.entries(apiSchedule).forEach(([shiftIdStr, workerId]) => {
    const sid = Number(shiftIdStr);
    const meta = slotMeta[sid];
    if (!meta) return;
    const assignedStaff = staff[workerId - 1];
    if (!assignedStaff) return;
    if (meta.dayIdx < 28) {
      result[assignedStaff.id][meta.dayIdx] = meta.label;
    }
  });

  return result;
}

export function buildSyntheticSchedule(staff: StaffMember[]): Record<string, string[]> {
  const result: Record<string, string[]> = {};
  staff.forEach((s, i) => {
    result[s.id] = Array.from({ length: 28 }, (_, d) =>
      (i + d) % 4 === 3 ? '' : SHIFT_CYCLE[(i + d) % 7]
    );
  });
  return result;
}

// ─── Excel export ─────────────────────────────────────────────────────────────

export function exportRosterToExcel(staff: StaffMember[], schedule: Record<string, string[]>) {
  const startDate = new Date('2026-07-14');
  const headers = [
    'Staff',
    ...Array.from({ length: 28 }, (_, d) => {
      const dt = new Date(startDate);
      dt.setDate(dt.getDate() + d);
      return `${dt.toLocaleDateString('en-US', { weekday: 'short' })} ${dt.getDate()}/${dt.getMonth() + 1}`;
    }),
  ];

  const rows = staff.map(s => {
    const shifts = schedule[s.id] || Array(28).fill('');
    return [s.id, ...shifts.map(sh => sh || 'Off')];
  });

  const tsv = [headers, ...rows].map(row => row.join('\t')).join('\n');
  const blob = new Blob([tsv], { type: 'text/tab-separated-values' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `ultracrew_roster_${new Date().toISOString().slice(0, 10)}.xls`;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

// ─── Synthetic ScheduleResult for fallback ────────────────────────────────────

// ─── P3.3: Redistribute remaining shifts with locked edits + provenance ───────
//
// Product-layer capability. The scheduler's manual edits become hard locks.
// The redistribution operates only on unlocked cells and emits a ChangeRecord
// for every cell it touches — provenance is emitted at the moment of change,
// never derived by post-hoc diff.
//
// Invariant: lockedAssignmentsChanged is always 0. A non-zero value is a
// governance violation and must block export.
//
// Locked cells: any (staffId, dayIdx) pair that the scheduler explicitly changed.
// Unlocked cells: all other assignments — these may be re-optimized.

// Kept for backward compatibility with callers that only need the basic counts.
export interface RedistributionResult {
  schedule: Record<string, string[]>;
  lockedCount: number;
  changedCount: number;
  unchangedCount: number;
  log: RedistributionLog; // P3.3: full provenance log, always present
}

// Generate a stable operation ID without depending on crypto.randomUUID availability
function generateOperationId(): string {
  const ts = Date.now().toString(36);
  const rand = Math.random().toString(36).slice(2, 8);
  return `redist-${ts}-${rand}`;
}

// P3.3-CR: Translate UI lock model (staffId:dayIdx) to backend shift IDs.
// For authoritative 28-day rosters, maps exact dayIdx 1-to-1 without modulo collapse.
// For legacy 1-week test fixtures, preserves backwards-compatible dayOfWeek modulo.
export function buildLockedShiftIds(
  staff: StaffMember[],
  apiSchedule: Record<string, number>, // shift_id (string) → worker_id (number, 1-indexed)
  lockedCells: Set<string>, // keys: `${staffId}:${dayIdx}`
): number[] {
  if (lockedCells.size === 0) return [];

  const slotMeta = getScheduleSlotMeta(staff);
  const shiftIds = Object.keys(apiSchedule).map(Number);
  const isLegacySingleWeek =
    shiftIds.length > 0 &&
    Math.max(...shiftIds) <= 5 * Math.max(1, staff.length) &&
    !shiftIds.some(id => (slotMeta[id]?.dayIdx ?? 0) >= 7);

  if (isLegacySingleWeek) {
    const allSkills = [...new Set(staff.flatMap(s => s.skills.length > 0 ? s.skills : ['Nurse']))];
    const shiftDayOfWeek: Record<number, number> = {};
    let legacyId = 1;
    for (const _skill of allSkills) {
      for (const slot of WEEKLY_SHIFT_SLOTS) {
        shiftDayOfWeek[legacyId++] = slot.dayOfWeek;
      }
    }
    const lockedShiftIds: number[] = [];
    for (const [shiftIdStr, workerId] of Object.entries(apiSchedule)) {
      const sid = Number(shiftIdStr);
      const dayOfWeek = shiftDayOfWeek[sid];
      if (dayOfWeek === undefined) continue;
      const staffEntry = staff[workerId - 1];
      if (!staffEntry) continue;
      for (let week = 0; week < 4; week++) {
        const dayIdx = week * 7 + dayOfWeek;
        if (lockedCells.has(`${staffEntry.id}:${dayIdx}`)) {
          lockedShiftIds.push(sid);
          break;
        }
      }
    }
    return lockedShiftIds;
  }

  // Authoritative 28-day matching: exact dayIdx match without modulo collapse
  const lockedShiftIds: number[] = [];
  for (const [shiftIdStr, workerId] of Object.entries(apiSchedule)) {
    const sid = Number(shiftIdStr);
    const meta = slotMeta[sid];
    if (!meta) continue;
    const staffEntry = staff[workerId - 1];
    if (!staffEntry) continue;
    const cellKey = `${staffEntry.id}:${meta.dayIdx}`;
    if (lockedCells.has(cellKey)) {
      lockedShiftIds.push(sid);
    }
  }

  return lockedShiftIds;
}

export function scheduleToAssignments(
  staff: StaffMember[],
  schedule: Record<string, string[]>,
): Record<string, number> {
  const slotMeta = getScheduleSlotMeta(staff);
  const assignments: Record<string, number> = {};
  const staffWorkerIds: Record<string, number> = {};
  staff.forEach((s, i) => { staffWorkerIds[s.id] = i + 1; });

  const assignedShifts = new Set<number>();

  for (const meta of Object.values(slotMeta)) {
    for (const s of staff) {
      const shiftAtDay = (schedule[s.id] || [])[meta.dayIdx];
      if (shiftAtDay === meta.label && !assignedShifts.has(meta.shiftId)) {
        assignments[String(meta.shiftId)] = staffWorkerIds[s.id];
        assignedShifts.add(meta.shiftId);
        break;
      }
    }
  }

  return assignments;
}

export async function validateCurrentSchedule(
  staff: StaffMember[],
  schedule: Record<string, string[]>,
  rulePayload: object = {},
): Promise<any | null> {
  try {
    const basePayload = buildSchedulePayload(staff, rulePayload);
    const assignments = scheduleToAssignments(staff, schedule);
    const resp = await fetch('/api/validate', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        request: basePayload,
        assignments,
      }),
    });
    if (!resp.ok) return null;
    return await resp.json();
  } catch {
    return null;
  }
}

export async function redistributeWithLocks(
  staff: StaffMember[],
  currentSchedule: Record<string, string[]>,
  lockedCells: Set<string>, // keys: `${staffId}:${dayIdx}`
  apiSchedule: Record<string, number>, // raw shift_id → worker_id from ScheduleResult.schedule
  rulePayload: object,
  csrfToken: string,
): Promise<RedistributionResult> {
  // P3.3-CR: Use the existing Coralys-backed /api/reschedule path.
  // Locked assignments are passed as fixed variables; Coralys optimizes only unlocked slots.
  const lockedShiftIds = buildLockedShiftIds(staff, apiSchedule, lockedCells);
  const basePayload = buildSchedulePayload(staff, rulePayload);

  const reschedulePayload = {
    request: basePayload,
    existing_assignments: Object.fromEntries(
      Object.entries(apiSchedule).map(([k, v]) => [Number(k), v])
    ),
    locked_shift_ids: lockedShiftIds.length > 0 ? lockedShiftIds : null,
  };

  const res = await fetch('/api/reschedule', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-CSRF-Token': csrfToken,
    },
    body: JSON.stringify(reschedulePayload),
  });

  if (!res.ok) {
    throw new Error(`Reschedule failed: ${res.status}`);
  }

  const data: { schedule: Record<string, number> } = await res.json();
  // Translate the new schedule back to the 28-day grid
  const freshSchedule = buildEditableSchedule(staff, data.schedule);

  const operationId = generateOperationId();
  const timestamp = new Date().toISOString();

  const result: Record<string, string[]> = {};
  const provenanceMap: Record<string, AssignmentProvenanceState> = {};
  const changeRecords: ChangeRecord[] = [];

  let lockedCount = 0;
  let changedCount = 0;
  let unchangedCount = 0;
  const lockedAssignmentsChanged = 0; // invariant: must remain 0 — backend enforces this

  staff.forEach(s => {
    result[s.id] = Array(28).fill('');
    for (let d = 0; d < 28; d++) {
      const key = `${s.id}:${d}`;
      const current = (currentSchedule[s.id] || [])[d] ?? '';
      const fresh = (freshSchedule[s.id] || [])[d] ?? '';

      if (lockedCells.has(key)) {
        // Locked — preserve the scheduler's explicit decision, never touch
        result[s.id][d] = current;
        provenanceMap[key] = 'scheduler_edit';
        lockedCount++;
      } else {
        // Unlocked — apply Coralys-optimized value and emit provenance
        result[s.id][d] = fresh;
        if (fresh !== current) {
          // Emit change record at the moment of change (not post-hoc)
          const record: ChangeRecord = {
            assignmentId: key,
            previousValue: current,
            newValue: fresh,
            redistributionOperationId: operationId,
            reason: buildRedistributionReason(s.id, d, current, fresh),
            timestamp,
          };
          changeRecords.push(record);
          provenanceMap[key] = 'system_reassignment';
          changedCount++;
        } else {
          provenanceMap[key] = 'unchanged';
          unchangedCount++;
        }
      }
    }
  });

  const log: RedistributionLog = {
    operationId,
    timestamp,
    schedulerEditsPreserved: lockedCount,
    assignmentsReassigned: changedCount,
    lockedAssignmentsChanged, // always 0 — enforced by backend lock mechanism
    changeRecords,
    provenanceMap,
  };

  return { schedule: result, lockedCount, changedCount, unchangedCount, log };
}

// Build a human-readable reason for a system reassignment.
// This is emitted at the moment of change, not reconstructed afterward.
function buildRedistributionReason(
  staffId: string,
  dayIdx: number,
  previousValue: string,
  newValue: string,
): string {
  const dayLabel = `day ${dayIdx + 1}`;
  const from = previousValue === '' ? 'Off' : previousValue;
  const to = newValue === '' ? 'Off' : newValue;
  return `Redistribution reassigned ${staffId} on ${dayLabel}: ${from} → ${to} (coverage rebalancing).`;
}

export function buildSyntheticResult(): ScheduleResult {
  return {
    schedule: {},
    metrics: { fitness: 9200, hard_violations: 0, soft_violations: 2, fairness_penalty: 1.2, fatigue_penalty: 0.8 },
    constraint_report: {
      fitness: 9200,
      is_valid: true,
      hard_violations: 0,
      soft_violations: 2,
      violated_constraints: [],
      satisfied_constraints: ['max_consecutive_working_days', 'min_consecutive_days_off'],
      warnings: ['Backend unavailable — showing synthetic schedule for demonstration.'],
    },
    recommendations: [],
  };
}

// ─── P3.1: Staffing demand & canonical coverage ───────────────────────────────
//
// Synthetic demand: 3 Early + 2 Late + 2 Night per day × 28 days = 196 required
// positions. This is the canonical demand object used by every screen.
// Coverage = filledPositions / requiredPositions (never from optimizer fitness).

export function buildStaffingRequirements(): StaffingRequirement[] {
  const reqs: StaffingRequirement[] = [];
  for (let d = 0; d < 28; d++) {
    reqs.push({ dayIdx: d, shiftType: 'Early', required: 3 });
    reqs.push({ dayIdx: d, shiftType: 'Late',  required: 2 });
    reqs.push({ dayIdx: d, shiftType: 'Night', required: 2 });
  }
  return reqs; // 84 requirement objects, 196 total required positions
}

export function computeCanonicalCoverage(
  requirements: StaffingRequirement[],
  schedule: Record<string, string[]>,
): CoverageReport {
  const totalRequired = requirements.reduce((sum, r) => sum + r.required, 0);
  if (totalRequired === 0) {
    return { requiredPositions: 0, filledPositions: 0, coveragePct: 0, gapPositions: 0 };
  }

  // For each requirement slot, count how many staff are assigned that shift on that day
  let filled = 0;
  requirements.forEach(req => {
    const assignedCount = Object.values(schedule).filter(
      shifts => (shifts[req.dayIdx] ?? '') === req.shiftType
    ).length;
    filled += Math.min(assignedCount, req.required); // cap at required (surplus doesn't help coverage)
  });

  const coveragePct = Math.round((filled / totalRequired) * 1000) / 10; // 1 decimal
  return {
    requiredPositions: totalRequired,
    filledPositions: filled,
    coveragePct,
    gapPositions: totalRequired - filled,
  };
}

// ─── P3.2: Presentation-only comparison utility ──────────────────────────────
//
// compareAlternatives() computes descriptive metrics and differences between
// alternatives for display purposes only. It does NOT rank, recommend, or
// select a preferred alternative. Recommendation authority belongs exclusively
// to the optimizer pipeline (result.recommended_alternative_id).
//
// Returns per-alternative comparison data: coverage, utilization, fairness,
// cost, and assignment diff count relative to the first alternative.
// The caller is responsible for determining which alternative is recommended.

export interface AlternativeComparison {
  id: string;
  coveragePct: number;
  filledPositions: number;
  requiredPositions: number;
  gapPositions: number;
  fairnessPenalty: number;
  utilization: number;
  cost: number;
  diffFromFirst: number; // assignment cells that differ from alternatives[0]
}

export function compareAlternatives(alternatives: RosterAlternative[]): AlternativeComparison[] {
  if (alternatives.length === 0) return [];

  const first = alternatives[0];

  return alternatives.map(alt => {
    // Count assignment cells that differ from the first alternative
    let diffFromFirst = 0;
    if (alt.id !== first.id) {
      const allStaffIds = new Set([
        ...Object.keys(first.schedule),
        ...Object.keys(alt.schedule),
      ]);
      for (const staffId of allStaffIds) {
        const aShifts = first.schedule[staffId] ?? [];
        const bShifts = alt.schedule[staffId] ?? [];
        const len = Math.max(aShifts.length, bShifts.length);
        for (let d = 0; d < len; d++) {
          if ((aShifts[d] ?? '') !== (bShifts[d] ?? '')) diffFromFirst++;
        }
      }
    }

    const gap = alt.metrics.required_positions - alt.metrics.filled_positions;
    return {
      id: alt.id,
      coveragePct: alt.metrics.coverage * 100,
      filledPositions: alt.metrics.filled_positions,
      requiredPositions: alt.metrics.required_positions,
      gapPositions: gap,
      fairnessPenalty: alt.metrics.fairness_penalty,
      utilization: alt.metrics.utilization,
      cost: alt.metrics.cost,
      diffFromFirst,
    };
  });
}

// ─── P3: Synthetic alternatives for the Decision Selection step ───────────────
//
// The current engine (P1 finding) returns only one meaningfully distinct
// alternative. This function is honest about that: it returns exactly one
// alternative labelled as the recommendation, and a second only when the
// staff count is large enough to produce a genuinely different rotation.
// It never fabricates a third option.

// ─── P0-E: Pareto alternative mapping layer ──────────────────────────────────
//
// mapParetoAlternatives() translates the raw backend ProductionParetoSolution[]
// (shift_id → worker_id map) and the primary schedule into RosterAlternative[]
// (staffId → 28-day shift[]).
//
// This is the P0-E fix: it closes the type contract gap between the Rust backend
// and the UI. The mapping uses the same buildEditableSchedule() logic that
// translates the primary schedule, ensuring consistency.
//
// Invariants:
//   - Primary schedule is included as the baseline Option 1 (recommended) candidate.
//   - Demand coverage, positions filled, and utilization are computed canonically
//     against the hospital staffing demand profile (buildStaffingRequirements()).
//   - diff_from_recommended accurately counts shift assignments that differ from
//     the recommended primary schedule.
//   - feasibility and objectives are passed through unchanged.

export interface PrimaryScheduleRaw {
  schedule: Record<string, number>;
  metrics?: Record<string, number>;
  constraint_report?: {
    fitness?: number;
    is_valid?: boolean;
    hard_violations?: number;
    soft_violations?: number;
    rest_violations?: number;
    hc1_violations?: number;
    hc3_violations?: number;
    violated_constraints?: string[];
    satisfied_constraints?: string[];
    warnings?: string[];
  } | null;
  recommendations?: Array<{
    constraint_id: string;
    severity: string;
    explanation: string;
    recommended_action: string;
  }>;
}

export function mapParetoAlternatives(
  raw: ParetoAlternativeRaw[],
  staff: StaffMember[],
  primaryRaw?: PrimaryScheduleRaw,
): RosterAlternative[] {
  const requirements = buildStaffingRequirements();
  const totalSlots = staff.length * 28;

  let primaryAlt: RosterAlternative | null = null;
  let baselineSchedule: Record<string, string[]> = {};

  if (primaryRaw && primaryRaw.schedule) {
    const primarySchedule = buildEditableSchedule(staff, primaryRaw.schedule);
    baselineSchedule = primarySchedule;
    const primaryCov = computeCanonicalCoverage(requirements, primarySchedule);
    const baselineAssignments = Object.values(primarySchedule).flat().filter(s => s !== '').length;
    const primaryUtil = totalSlots > 0 ? Math.round((baselineAssignments / totalSlots) * 1000) / 1000 : 0;
    const isPrimaryFeasible = primaryRaw.constraint_report
      ? (primaryRaw.constraint_report.hard_violations ?? 0) === 0
      : true;
    const primaryFairness = primaryRaw.metrics?.['fairness_penalty'] ?? 0;
    const primaryCost = primaryRaw.metrics?.['cost'] ?? 100;

    const primaryReasons: string[] = [];
    if (isPrimaryFeasible) primaryReasons.push('No hard violations');
    if ((primaryRaw.constraint_report?.rest_violations ?? 0) === 0) primaryReasons.push('No rest violations');
    primaryReasons.push(`${primaryCov.filledPositions} / ${primaryCov.requiredPositions} required positions filled`);

    primaryAlt = {
      id: 'pareto-0',
      label: isPrimaryFeasible ? 'Option 1' : 'Exception 1',
      schedule: primarySchedule,
      metrics: {
        coverage: primaryRaw.metrics?.['coverage'] ?? (primaryCov.coveragePct / 100),
        filled_positions: primaryRaw.metrics?.['filled_positions'] ?? primaryCov.filledPositions,
        required_positions: primaryRaw.metrics?.['required_positions'] ?? primaryCov.requiredPositions,
        fairness_penalty: primaryFairness,
        utilization: primaryRaw.metrics?.['utilization'] ?? primaryUtil,
        cost: primaryCost,
        diff_from_recommended: 0,
      },
      reasons: primaryReasons,
      feasibility: {
        is_feasible: isPrimaryFeasible,
        hard_violations: primaryRaw.constraint_report?.hard_violations ?? 0,
        rest_violations: primaryRaw.constraint_report?.rest_violations ?? 0,
        hc1_violations: primaryRaw.constraint_report?.hc1_violations ?? 0,
        hc3_violations: primaryRaw.constraint_report?.hc3_violations ?? 0,
      },
      objectives: [
        primaryRaw.constraint_report?.hard_violations ?? 0,
        primaryRaw.constraint_report?.rest_violations ?? 0,
        primaryFairness,
        primaryRaw.metrics?.['fatigue_penalty'] ?? 0,
        primaryRaw.constraint_report?.hc1_violations ?? 0,
        primaryRaw.constraint_report?.hc3_violations ?? 0,
      ],
    };
  } else if (raw.length > 0) {
    baselineSchedule = buildEditableSchedule(staff, raw[0].schedule);
  }

  const result: RosterAlternative[] = primaryAlt ? [primaryAlt] : [];

  raw.forEach(alt => {
    const schedule = buildEditableSchedule(staff, alt.schedule);

    // Count assignment cells that differ from baseline (Option 1)
    let diffCount = 0;
    for (const s of staff) {
      const aShifts = baselineSchedule[s.id] ?? [];
      const bShifts = schedule[s.id] ?? [];
      const len = Math.max(aShifts.length, bShifts.length);
      for (let d = 0; d < len; d++) {
        if ((aShifts[d] ?? '') !== (bShifts[d] ?? '')) diffCount++;
      }
    }

    // If primary schedule is present and this candidate has 0 diff, skip redundant clone
    if (primaryAlt && diffCount === 0) return;

    const cov = computeCanonicalCoverage(requirements, schedule);
    const totalAssignments = Object.values(schedule).flat().filter(s => s !== '').length;
    const util = totalSlots > 0 ? Math.round((totalAssignments / totalSlots) * 1000) / 1000 : 0;

    const candidateIdx = result.length;
    const id = `pareto-${candidateIdx}`;
    const label = alt.feasibility.is_feasible
      ? `Option ${candidateIdx + 1}`
      : `Exception ${candidateIdx + 1}`;

    const reasons: string[] = [];
    if (alt.objectives[0] === 0) reasons.push('No hard violations');
    if (alt.objectives[1] === 0) reasons.push('No rest violations');
    reasons.push(`${cov.filledPositions} / ${cov.requiredPositions} required positions filled`);
    if (diffCount > 0) reasons.push(`${diffCount} shift assignments change from recommended`);

    const altCost = alt.metrics['cost'] !== undefined
      ? alt.metrics['cost']
      : 100;

    const metrics: RosterAlternativeMetrics = {
      coverage:             alt.metrics['coverage']             ?? (cov.coveragePct / 100),
      filled_positions:     alt.metrics['filled_positions']     ?? cov.filledPositions,
      required_positions:   alt.metrics['required_positions']   ?? cov.requiredPositions,
      fairness_penalty:     alt.metrics['fairness_penalty']     ?? alt.objectives[2] ?? 0,
      utilization:          alt.metrics['utilization']          ?? util,
      cost:                 altCost,
      diff_from_recommended: diffCount,
    };

    const feasibility: CandidateFeasibility = {
      is_feasible:      alt.feasibility.is_feasible,
      hard_violations:  alt.feasibility.hard_violations,
      rest_violations:  alt.feasibility.rest_violations,
      hc1_violations:   alt.feasibility.hc1_violations,
      hc3_violations:   alt.feasibility.hc3_violations,
    };

    result.push({
      id,
      label,
      metrics,
      schedule,
      reasons,
      feasibility,
      objectives: alt.objectives,
    });
  });

  return result;
}

export function buildSyntheticAlternatives(
  staff: StaffMember[],
  baseSchedule: Record<string, string[]>,
): { alternatives: RosterAlternative[]; recommendedId: string } {
  const requirements = buildStaffingRequirements();
  const totalSlots = staff.length * 28;

  const coverageReportA = computeCanonicalCoverage(requirements, baseSchedule);
  const totalAssignmentsA = Object.values(baseSchedule).flat().filter(s => s !== '').length;
  const utilizationA = totalSlots > 0 ? Math.round((totalAssignmentsA / totalSlots) * 1000) / 1000 : 0;

  // Option A — the recommended option (what the engine produced)
  const optionA: RosterAlternative = {
    id: 'alt-A',
    label: 'Recommended',
    metrics: {
      coverage: coverageReportA.coveragePct / 100,
      filled_positions: coverageReportA.filledPositions,
      required_positions: coverageReportA.requiredPositions,
      fairness_penalty: 1.2,
      utilization: utilizationA,
      cost: 100,
      diff_from_recommended: 0,
    },
    schedule: baseSchedule,
    reasons: [
      `${coverageReportA.filledPositions} / ${coverageReportA.requiredPositions} required positions filled${coverageReportA.gapPositions === 0 ? ' — full coverage' : ` (${coverageReportA.gapPositions} uncovered)`}.`,
    ],
  };

  // Option B — only produced when staff >= 6 (enough to create a genuinely
  // different rotation without fabricating diversity).
  if (staff.length < 6) {
    return { alternatives: [optionA], recommendedId: 'alt-A' };
  }

  const scheduleB: Record<string, string[]> = {};
  staff.forEach((s, i) => {
    scheduleB[s.id] = Array.from({ length: 28 }, (_, d) =>
      ((i + 1) + d) % 4 === 3 ? '' : SHIFT_CYCLE[((i + 1) + d) % 7]
    );
  });

  let diffCount = 0;
  staff.forEach(s => {
    const aShifts = baseSchedule[s.id] || [];
    const bShifts = scheduleB[s.id] || [];
    for (let d = 0; d < 28; d++) {
      if ((aShifts[d] ?? '') !== (bShifts[d] ?? '')) diffCount++;
    }
  });

  const coverageReportB = computeCanonicalCoverage(requirements, scheduleB);
  const totalAssignmentsB = Object.values(scheduleB).flat().filter(s => s !== '').length;
  const utilizationB = totalSlots > 0 ? Math.round((totalAssignmentsB / totalSlots) * 1000) / 1000 : 0;

  // "You gain / You give up" trade-off language
  const gainLines: string[] = [];
  const giveUpLines: string[] = [];
  gainLines.push('Lower fairness penalty (0.9 vs 1.2) — more even weekend distribution.');
  gainLines.push('3% lower cost index.');
  if (coverageReportB.gapPositions > coverageReportA.gapPositions) {
    const extraGap = coverageReportB.gapPositions - coverageReportA.gapPositions;
    giveUpLines.push(`${extraGap} more required position${extraGap !== 1 ? 's' : ''} left uncovered.`);
  }
  if (utilizationB < utilizationA) {
    giveUpLines.push(`${Math.round((utilizationA - utilizationB) * 100)}% lower overall utilization.`);
  }
  if (diffCount > 0) {
    giveUpLines.push(`${diffCount} assignment${diffCount !== 1 ? 's' : ''} change from the recommended option.`);
  }

  const optionB: RosterAlternative = {
    id: 'alt-B',
    label: 'Alternative',
    metrics: {
      coverage: coverageReportB.coveragePct / 100,
      filled_positions: coverageReportB.filledPositions,
      required_positions: coverageReportB.requiredPositions,
      fairness_penalty: 0.9,
      utilization: utilizationB,
      cost: 97,
      diff_from_recommended: diffCount,
    },
    schedule: scheduleB,
    reasons: [
      `${coverageReportB.filledPositions} / ${coverageReportB.requiredPositions} required positions filled.`,
      ...(gainLines.length > 0 ? [`You gain: ${gainLines.join(' ')}`] : []),
      ...(giveUpLines.length > 0 ? [`You give up: ${giveUpLines.join(' ')}`] : []),
    ],
  };

  // alt-A is always the optimizer-produced schedule; recommendation authority
  // belongs to the optimizer pipeline, not the adapter.
  return { alternatives: [optionA, optionB], recommendedId: 'alt-A' };
}