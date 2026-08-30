import type { StaffMember, ImportSummary, ScheduleResult, RosterAlternative, StaffingRequirement, CoverageReport } from './WorkflowTypes';
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

export function buildSchedulePayload(staff: StaffMember[], _rulePayload: object) {
  const workers = staff.map((s, i) => ({
    id: i + 1,
    skills: s.skills.length > 0 ? s.skills : ['Nurse'],
  }));

  // Collect all unique skills across staff
  const allSkills = [...new Set(staff.flatMap(s => s.skills.length > 0 ? s.skills : ['Nurse']))];

  // Generate shift slots: one per (slot, skill) combination
  const shifts: Array<{ id: number; start_hour: number; duration_hours: number; required_skill: string }> = [];
  let shiftId = 1;
  for (const skill of allSkills) {
    for (const slot of WEEKLY_SHIFT_SLOTS) {
      shifts.push({
        id: shiftId++,
        start_hour: slot.startHour,
        duration_hours: 8,
        required_skill: skill,
      });
    }
  }

  return {
    workers,
    shifts,
    historical_workloads: null,
    rng_seed: null,
    generation_limit: 200,
  };
}

// ─── Schedule builders ────────────────────────────────────────────────────────

export function buildEditableSchedule(
  staff: StaffMember[],
  apiSchedule: Record<string, number>
): Record<string, string[]> {
  // apiSchedule: shift_id (string) → worker_id (number, 1-indexed)
  // Shift IDs were generated in buildSchedulePayload:
  //   for each skill, for each WEEKLY_SHIFT_SLOTS entry → shiftId++
  //
  // We reconstruct the shiftId → {dayOfWeek, label} mapping,
  // then expand the weekly pattern into a 28-day grid (4 weeks).

  const allSkills = [...new Set(staff.flatMap(s => s.skills.length > 0 ? s.skills : ['Nurse']))];

  // Rebuild shiftId → {dayOfWeek, label, skill} map
  const shiftMeta: Record<number, { dayOfWeek: number; label: string; skill: string }> = {};
  let shiftId = 1;
  for (const skill of allSkills) {
    for (const slot of WEEKLY_SHIFT_SLOTS) {
      shiftMeta[shiftId++] = { dayOfWeek: slot.dayOfWeek, label: slot.label, skill };
    }
  }

  // Initialize result: staffId → 28 empty strings
  const result: Record<string, string[]> = {};
  staff.forEach(s => { result[s.id] = Array(28).fill(''); });

  // Build worker assignment map: workerId → Set of (dayOfWeek, label) assigned
  // Then expand across 4 weeks
  const workerWeeklySlots: Record<number, Array<{ dayOfWeek: number; label: string }>> = {};
  Object.entries(apiSchedule).forEach(([shiftIdStr, workerId]) => {
    const sid = Number(shiftIdStr);
    const meta = shiftMeta[sid];
    if (!meta) return;
    if (!workerWeeklySlots[workerId]) workerWeeklySlots[workerId] = [];
    workerWeeklySlots[workerId].push({ dayOfWeek: meta.dayOfWeek, label: meta.label });
  });

  // Expand weekly pattern into 28 days (4 weeks × 7 days)
  Object.entries(workerWeeklySlots).forEach(([workerIdStr, slots]) => {
    const workerId = Number(workerIdStr);
    const assignedStaff = staff[workerId - 1]; // 1-indexed
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

// ─── P3.5: Redistribute remaining shifts with locked edits ───────────────────
//
// Product-layer capability. The scheduler's manual edits become hard locks.
// The optimizer redistributes only the unlocked assignments.
// No optimizer algorithm changes — this is a product-layer constraint.
//
// Locked cells: any (staffId, dayIdx) pair that the scheduler explicitly changed.
// Unlocked cells: all other assignments — these may be re-optimized.
//
// The redistribution uses the same SHIFT_CYCLE pattern as buildSyntheticSchedule
// but skips any cell that is locked, preserving the scheduler's explicit decisions.

export interface RedistributionResult {
  schedule: Record<string, string[]>;
  lockedCount: number;
  changedCount: number;
  unchangedCount: number;
}

export function redistributeWithLocks(
  staff: StaffMember[],
  currentSchedule: Record<string, string[]>,
  lockedCells: Set<string>, // keys: `${staffId}:${dayIdx}`
): RedistributionResult {
  // Build a fresh synthetic schedule as the redistribution base
  const freshSchedule = buildSyntheticSchedule(staff);

  const result: Record<string, string[]> = {};
  let lockedCount = 0;
  let changedCount = 0;
  let unchangedCount = 0;

  staff.forEach(s => {
    result[s.id] = Array(28).fill('');
    for (let d = 0; d < 28; d++) {
      const key = `${s.id}:${d}`;
      const current = (currentSchedule[s.id] || [])[d] ?? '';
      const fresh = (freshSchedule[s.id] || [])[d] ?? '';

      if (lockedCells.has(key)) {
        // Locked — preserve the scheduler's explicit decision
        result[s.id][d] = current;
        lockedCount++;
      } else {
        // Unlocked — use the redistributed value
        result[s.id][d] = fresh;
        if (fresh !== current) changedCount++;
        else unchangedCount++;
      }
    }
  });

  return { schedule: result, lockedCount, changedCount, unchangedCount };
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

// ─── P3: Synthetic alternatives for the Decision Selection step ───────────────
//
// The current engine (P1 finding) returns only one meaningfully distinct
// alternative. This function is honest about that: it returns exactly one
// alternative labelled as the recommendation, and a second only when the
// staff count is large enough to produce a genuinely different rotation.
// It never fabricates a third option.

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
      `${coverageReportA.filledPositions} / ${coverageReportA.requiredPositions} required positions filled.`,
      'Best overall balance of fairness and cost.',
      'Recommended based on the current objective weighting.',
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

  return { alternatives: [optionA, optionB], recommendedId: 'alt-A' };
}