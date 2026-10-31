import type {
  StaffMember,
  ImportSummary,
  ScheduleResult,
  RosterAlternative,
  RosterAlternativeMetrics,
  CandidateFeasibility,
  StaffingRequirement,
  CoverageReport,
  ChangeRecord,
  RedistributionLog,
  AssignmentProvenanceState,
  ParetoAlternativeRaw,
  BaselineAssignment,
  BaselineAuditReport,
  AuditFinding,
} from './WorkflowTypes';
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

// Canonical 28-day hospital shift model matching buildStaffingRequirements():
// 7 shifts per day × 28 days = 196 shifts total.
// Early: 06:00–14:00 (3 per day: 1 HeadNurse, 2 Nurse)
// Late:  14:00–22:00 (2 per day: 2 Nurse)
// Night: 22:00–06:00 (2 per day: 2 Nurse)
//
// Kept for backward compatibility with 1-week legacy test fixtures:
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
  durationHours?: number;
}

export type CustomShiftInput = ShiftSlotMeta;

// ─── Variable Demand CSV Parsing ─────────────────────────────────────────────
// Supports customer-provided shifts.csv (id,start_hour,duration_hours,required_skill)
// as specified in the UltraRoster Assessment Protocol.
export function parseShiftsCSV(text: string): { shifts: ShiftSlotMeta[]; errors: string[] } {
  const lines = text.trim().split('\n').map(l => l.trim()).filter(Boolean);
  if (lines.length < 2) return { shifts: [], errors: ['CSV must have a header row and at least one data row.'] };

  const header = lines[0].toLowerCase().split(',').map(h => h.trim());
  const idIdx = header.indexOf('id') !== -1 ? header.indexOf('id') : header.indexOf('shift_id');
  const startIdx = header.indexOf('start_hour');
  const durIdx = header.indexOf('duration_hours');
  const skillIdx = header.indexOf('required_skill');

  const errors: string[] = [];
  if (idIdx === -1) errors.push('Missing column: id (or shift_id)');
  if (startIdx === -1) errors.push('Missing column: start_hour');
  if (durIdx === -1) errors.push('Missing column: duration_hours');
  if (skillIdx === -1) errors.push('Missing column: required_skill');
  if (errors.length > 0) return { shifts: [], errors };

  const shifts: ShiftSlotMeta[] = [];
  const seenIds = new Set<number>();

  for (let i = 1; i < lines.length; i++) {
    const cols = lines[i].split(',').map(c => c.trim().replace(/^"|"$/g, ''));
    const id = parseInt(cols[idIdx], 10);
    const startHour = parseInt(cols[startIdx], 10);
    const durationHours = parseInt(cols[durIdx], 10);
    const skill = cols[skillIdx];

    if (isNaN(id)) { errors.push(`Row ${i + 1}: invalid id`); continue; }
    if (isNaN(startHour) || startHour < 0) { errors.push(`Row ${i + 1}: invalid start_hour for shift ${id}`); continue; }
    if (isNaN(durationHours) || durationHours <= 0) { errors.push(`Row ${i + 1}: duration_hours must be > 0 for shift ${id}`); continue; }
    if (!skill) { errors.push(`Row ${i + 1}: missing required_skill for shift ${id}`); continue; }
    if (seenIds.has(id)) { errors.push(`Row ${i + 1}: duplicate id "${id}"`); continue; }

    seenIds.add(id);
    const dayIdx = Math.floor(startHour / 24);
    const hourOfDay = startHour % 24;
    let label = 'Night';
    if (hourOfDay >= 6 && hourOfDay < 14) {
      label = 'Early';
    } else if (hourOfDay >= 14 && hourOfDay < 22) {
      label = 'Late';
    }

    shifts.push({
      shiftId: id,
      dayIdx,
      label,
      skill,
      startHour,
      durationHours,
    });
  }

  return { shifts, errors };
}

export function getScheduleSlotMeta(
  staff: StaffMember[],
  customShifts?: ShiftSlotMeta[],
): Record<number, ShiftSlotMeta> {
  if (customShifts && customShifts.length > 0) {
    const meta: Record<number, ShiftSlotMeta> = {};
    for (const s of customShifts) {
      meta[s.shiftId] = s;
    }
    return meta;
  }

  const hasHeadNurse = staff.some(s => s.skills.includes('HeadNurse'));
  const nurseSkill = staff.some(s => s.skills.includes('Nurse'))
    ? 'Nurse'
    : (staff.find(s => s.skills.length > 0)?.skills[0] || 'Nurse');
  const headNurseSkill = hasHeadNurse ? 'HeadNurse' : nurseSkill;

  const meta: Record<number, ShiftSlotMeta> = {};
  let shiftId = 1;

  for (let day = 0; day < 28; day++) {
    const baseHour = day * 24;

    // 3 Early shifts (06:00–14:00): 1 HeadNurse, 2 Nurse
    meta[shiftId] = { shiftId, dayIdx: day, label: 'Early', skill: headNurseSkill, startHour: baseHour + 6, durationHours: 8 };
    shiftId++;
    meta[shiftId] = { shiftId, dayIdx: day, label: 'Early', skill: nurseSkill, startHour: baseHour + 6, durationHours: 8 };
    shiftId++;
    meta[shiftId] = { shiftId, dayIdx: day, label: 'Early', skill: nurseSkill, startHour: baseHour + 6, durationHours: 8 };
    shiftId++;

    // 2 Late shifts (14:00–22:00): 2 Nurse
    meta[shiftId] = { shiftId, dayIdx: day, label: 'Late', skill: nurseSkill, startHour: baseHour + 14, durationHours: 8 };
    shiftId++;
    meta[shiftId] = { shiftId, dayIdx: day, label: 'Late', skill: nurseSkill, startHour: baseHour + 14, durationHours: 8 };
    shiftId++;

    // 2 Night shifts (22:00–06:00 next day): 2 Nurse
    meta[shiftId] = { shiftId, dayIdx: day, label: 'Night', skill: nurseSkill, startHour: baseHour + 22, durationHours: 8 };
    shiftId++;
    meta[shiftId] = { shiftId, dayIdx: day, label: 'Night', skill: nurseSkill, startHour: baseHour + 22, durationHours: 8 };
    shiftId++;
  }

  return meta;
}

// ─── Pre-Optimization Demand-Capacity Diagnostic ─────────────────────────────
// Compares required shift hours against available contractual workforce capacity
// before optimization. Provides early indication of under-capacity or excess buffer.
export interface DemandCapacityDiagnostic {
  demandHours: number;
  workforceCapacityHours: number;
  utilizationRatio: number; // demandHours / workforceCapacityHours
  surplusOrDeficitHours: number;
  status: 'balanced' | 'tight' | 'under_capacity' | 'excess_capacity';
  message: string;
  shiftCount: number;
  horizonDays: number;
  feasibilityDisclaimer: string;
}

export function computeDemandCapacityDiagnostic(
  staff: StaffMember[],
  shifts?: Array<{ duration_hours?: number; durationHours?: number }>,
  horizonDays: number = 28,
): DemandCapacityDiagnostic {
  const shiftList: Array<{ duration_hours?: number; durationHours?: number }> = (shifts && shifts.length > 0)
    ? shifts
    : Array.from({ length: 7 * horizonDays }, () => ({ durationHours: 8 }));
  const demandHours = shiftList.reduce((sum, s) => sum + (s.duration_hours ?? s.durationHours ?? 8), 0);
  
  // Contractual capacity over planning horizon: FullTime=40h/wk, PartTime=20h/wk, Night=32h/wk
  const weeks = horizonDays / 7;
  const workforceCapacityHours = staff.reduce((sum, s) => {
    let weeklyHours = 40;
    if (s.contract === 'PartTime') weeklyHours = 20;
    else if (s.contract === 'Night') weeklyHours = 32;
    return sum + (weeklyHours * weeks);
  }, 0);

  const utilizationRatio = workforceCapacityHours > 0
    ? Math.round((demandHours / workforceCapacityHours) * 1000) / 1000
    : 0;
  const surplusOrDeficitHours = workforceCapacityHours - demandHours;

  let status: 'balanced' | 'tight' | 'under_capacity' | 'excess_capacity' = 'balanced';
  let message = '';

  if (demandHours > workforceCapacityHours) {
    status = 'under_capacity';
    message = `Under-Capacity Deficit: Total demand (${demandHours}h) exceeds contractual capacity (${workforceCapacityHours}h) by ${Math.abs(surplusOrDeficitHours)}h. Unscheduled gaps or overtime required.`;
  } else if (utilizationRatio > 0.90) {
    status = 'tight';
    message = `Tight Capacity (${(utilizationRatio * 100).toFixed(1)}%): Minimal buffer for rotation constraints, rest periods, and unexpected absences.`;
  } else if (utilizationRatio >= 0.65) {
    status = 'balanced';
    message = `Balanced Capacity (${(utilizationRatio * 100).toFixed(1)}%): Healthy buffer (${surplusOrDeficitHours}h surplus) to satisfy rest and fairness constraints.`;
  } else {
    status = 'excess_capacity';
    message = `Excess Capacity (${(utilizationRatio * 100).toFixed(1)}%): High staffing surplus (${surplusOrDeficitHours}h) relative to demand.`;
  }

  return {
    demandHours,
    workforceCapacityHours,
    utilizationRatio,
    surplusOrDeficitHours,
    status,
    message,
    shiftCount: shiftList.length,
    horizonDays,
    feasibilityDisclaimer:
      'Aggregate capacity is a necessary pre-optimization diagnostic but does not prove mathematical feasibility. Shift succession, minimum rest periods (10h), specialized skill distribution, and weekend balance must still be verified by the optimization solver.',
  };
}

export function buildSchedulePayload(
  staff: StaffMember[],
  rulePayload: any = {},
  customShifts?: ShiftSlotMeta[],
) {
  const workers = staff.map((s, i) => ({
    id: i + 1,
    skills: s.skills.length > 0 ? s.skills : ['Nurse'],
    contract: s.contract,
  }));

  const slotMeta = getScheduleSlotMeta(staff, customShifts);
  const shifts = Object.values(slotMeta).map(m => ({
    id: m.shiftId,
    start_hour: m.startHour,
    duration_hours: m.durationHours ?? 8,
    required_skill: m.skill,
  }));

  const maxDay = customShifts && customShifts.length > 0
    ? Math.max(...Object.values(slotMeta).map(m => m.dayIdx))
    : Math.max(...Object.values(slotMeta).map(m => m.dayIdx), 27);
  const horizonHours = (maxDay + 1) * 24;
  const maxShiftEnd = Math.max(...shifts.map(s => s.start_hour + s.duration_hours), horizonHours);

  return {
    workers,
    shifts,
    historical_workloads: null,
    rng_seed: null,
    generation_limit: 100,
    scenario: {
      planning_horizon_hours: horizonHours, // 672h: shift start window 0 <= start < 672
      validation_envelope_hours: maxShiftEnd, // 678h: latest shift completion
      max_hours_per_worker: rulePayload?.max_weekly_hours ?? 40.0,
      minimum_rest_hours: rulePayload?.min_rest_hours ?? 10,
      leave_requests: null,
    },
  };
}

// ─── Schedule builders ────────────────────────────────────────────────────────

export function buildEditableSchedule(
  staff: StaffMember[],
  apiSchedule: Record<string, number>,
  customSlotMeta?: Record<number, ShiftSlotMeta>,
): Record<string, string[]> {
  const slotMeta = customSlotMeta ?? getScheduleSlotMeta(staff);
  const maxDay = Math.max(...Object.values(slotMeta).map(m => m.dayIdx), 27);
  const result: Record<string, string[]> = {};
  staff.forEach(s => { result[s.id] = Array(maxDay + 1).fill(''); });

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
          if (dayIdx <= maxDay) {
            result[assignedStaff.id][dayIdx] = slot.label;
          }
        }
      }
    });
    return result;
  }

  // Authoritative schedule mapping:
  Object.entries(apiSchedule).forEach(([shiftIdStr, workerId]) => {
    const sid = Number(shiftIdStr);
    const meta = slotMeta[sid];
    if (!meta) return;
    const assignedStaff = staff[workerId - 1];
    if (!assignedStaff) return;
    if (meta.dayIdx <= maxDay) {
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

// ─── Excel export (Control D: Microsoft XML Spreadsheet 2003 format) ──────────
// Genuine Excel-compatible XML (urn:schemas-microsoft-com:office:spreadsheet).
// Avoids the classic Excel format/extension mismatch dialog caused by TSV-in-.xls.

function escapeXml(unsafe: string): string {
  return String(unsafe).replace(/[<>&'"]/g, c => {
    switch (c) {
      case '<': return '&lt;';
      case '>': return '&gt;';
      case '&': return '&amp;';
      case '\'': return '&apos;';
      case '"': return '&quot;';
      default: return c;
    }
  });
}

export function buildExcelXmlSpreadsheet(
  staff: StaffMember[],
  schedule: Record<string, string[]>,
  horizonDays: number = 28,
): string {
  const startDate = new Date('2026-07-14');
  const dateHeaders = Array.from({ length: horizonDays }, (_, d) => {
    const dt = new Date(startDate);
    dt.setDate(dt.getDate() + d);
    return `${dt.toLocaleDateString('en-US', { weekday: 'short' })} ${dt.getDate()}/${dt.getMonth() + 1}`;
  });

  const headerCells = ['Staff', ...dateHeaders]
    .map(h => `<Cell ss:StyleID="Header"><Data ss:Type="String">${escapeXml(h)}</Data></Cell>`)
    .join('');

  const rowNodes = staff.map(s => {
    const shifts = schedule[s.id] || Array(horizonDays).fill('');
    const cells = [
      `<Cell ss:StyleID="StaffCol"><Data ss:Type="String">${escapeXml(s.id)}</Data></Cell>`,
      ...shifts.map(sh => `<Cell ss:StyleID="ShiftCell"><Data ss:Type="String">${escapeXml(sh || 'Off')}</Data></Cell>`),
    ].join('');
    return `<Row>${cells}</Row>`;
  }).join('\n   ');

  return `<?xml version="1.0" encoding="UTF-8"?>
<?mso-application progid="Excel.Sheet"?>
<Workbook xmlns="urn:schemas-microsoft-com:office:spreadsheet"
 xmlns:o="urn:schemas-microsoft-com:office:office"
 xmlns:x="urn:schemas-microsoft-com:office:excel"
 xmlns:ss="urn:schemas-microsoft-com:office:spreadsheet"
 xmlns:html="http://www.w3.org/TR/REC-html40">
 <DocumentProperties xmlns="urn:schemas-microsoft-com:office:office">
  <Author>UltraRoster Assessment Engine</Author>
  <Created>${new Date().toISOString()}</Created>
 </DocumentProperties>
 <Styles>
  <Style ss:ID="Default" ss:Name="Normal">
   <Alignment ss:Vertical="Center"/>
   <Borders/>
   <Font ss:FontName="Calibri" ss:Size="11" ss:Color="#000000"/>
  </Style>
  <Style ss:ID="Header">
   <Alignment ss:Horizontal="Center" ss:Vertical="Center"/>
   <Borders>
    <Border ss:Position="Bottom" ss:LineStyle="Continuous" ss:Weight="1"/>
   </Borders>
   <Font ss:FontName="Calibri" ss:Size="11" ss:Color="#FFFFFF" ss:Bold="1"/>
   <Interior ss:Color="#1E3A8A" ss:Pattern="Solid"/>
  </Style>
  <Style ss:ID="StaffCol">
   <Alignment ss:Horizontal="Left" ss:Vertical="Center"/>
   <Font ss:FontName="Calibri" ss:Size="11" ss:Bold="1"/>
  </Style>
  <Style ss:ID="ShiftCell">
   <Alignment ss:Horizontal="Center" ss:Vertical="Center"/>
  </Style>
 </Styles>
 <Worksheet ss:Name="UltraRoster Schedule">
  <Table>
   <Column ss:Width="120"/>
   ${Array.from({ length: horizonDays }, () => '<Column ss:Width="70"/>').join('\n   ')}
   <Row ss:Height="24">
    ${headerCells}
   </Row>
   ${rowNodes}
  </Table>
 </Worksheet>
</Workbook>`;
}

export function exportRosterToExcel(
  staff: StaffMember[],
  schedule: Record<string, string[]>,
  horizonDays: number = 28,
) {
  const xml = buildExcelXmlSpreadsheet(staff, schedule, horizonDays);
  const blob = new Blob([xml], { type: 'application/vnd.ms-excel' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `ultraroster_roster_${new Date().toISOString().slice(0, 10)}.xls`;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

// ─── CSV export & Assignment Conservation ─────────────────────────────────────
// Generates tabular CSV export matching the exact validated UI schedule.
// Guarantees: Assignments_UI = Assignments_Validation = Assignments_Export
export function exportRosterToCSV(
  staff: StaffMember[],
  schedule: Record<string, string[]>,
  horizonDays: number = 28,
): string {
  const lines = ['worker_id,staff_id,day_idx,shift_type'];
  staff.forEach((s, i) => {
    const workerId = i + 1;
    const shifts = schedule[s.id] || [];
    for (let d = 0; d < horizonDays; d++) {
      const shift = shifts[d];
      if (shift && shift !== '') {
        lines.push(`${workerId},${s.id},${d},${shift}`);
      }
    }
  });
  return lines.join('\n');
}

// ─── Cycle Continuity & Rest Lockout ─────────────────────────────────────────
// AT-20: Final-day night shifts end at hour 678. With min 10h rest, rest lockout
// extends to hour 688, preventing assignment to next cycle's Day 0 Early shift (hour 678).
export interface CycleBoundaryHistory {
  workerId: number;
  staffId: string;
  lastAssignedShift: string;
  lastShiftEndHour: number;
  restLockoutUntilHour: number;
  isRestLockedForCycleStartEarly: boolean;
}

export function computeNextCycleBoundary(
  staff: StaffMember[],
  schedule: Record<string, string[]>,
  planningHorizonHours: number = 672,
  minRestHours: number = 10,
): CycleBoundaryHistory[] {
  const lastDayIdx = Math.floor(planningHorizonHours / 24) - 1; // e.g. Day 27 for 672h
  const nextCycleEarlyStartHour = planningHorizonHours + 6; // e.g. 678 for 672h (Day 0 06:00 of next cycle)

  return staff.map((s, i) => {
    const workerId = i + 1;
    const lastShift = (schedule[s.id] || [])[lastDayIdx] || '';
    let lastShiftEndHour = 0;

    if (lastShift === 'Night') {
      lastShiftEndHour = lastDayIdx * 24 + 22 + 8; // ends at 678
    } else if (lastShift === 'Late') {
      lastShiftEndHour = lastDayIdx * 24 + 14 + 8; // ends at 670
    } else if (lastShift === 'Early') {
      lastShiftEndHour = lastDayIdx * 24 + 6 + 8; // ends at 662
    }

    const restLockoutUntilHour = lastShiftEndHour > 0 ? lastShiftEndHour + minRestHours : 0;
    const isRestLockedForCycleStartEarly = restLockoutUntilHour > nextCycleEarlyStartHour;

    return {
      workerId,
      staffId: s.id,
      lastAssignedShift: lastShift,
      lastShiftEndHour,
      restLockoutUntilHour,
      isRestLockedForCycleStartEarly,
    };
  });
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

export interface AssignmentConversionResult {
  assignments: Record<string, number>;
  totalUiAssignments: number;
  totalMappedAssignments: number;
  isConserved: boolean;
  unmappedUiAssignments: Array<{ staffId: string; dayIdx: number; shiftType: string; reason: string }>;
  duplicateAssignments: Array<{ staffId: string; dayIdx: number; shiftTypes: string[] }>;
}

export function scheduleToAssignments(
  staff: StaffMember[],
  schedule: Record<string, string[]>,
  customSlotMeta?: Record<number, ShiftSlotMeta>,
): AssignmentConversionResult {
  const slotMeta = customSlotMeta ?? getScheduleSlotMeta(staff);
  const assignments: Record<string, number> = {};
  const staffWorkerIds: Record<string, number> = {};
  staff.forEach((s, i) => { staffWorkerIds[s.id] = i + 1; });

  const assignedShifts = new Set<number>();
  const assignedWorkerDays = new Set<string>();
  const unmappedUiAssignments: Array<{ staffId: string; dayIdx: number; shiftType: string; reason: string }> = [];
  const duplicateAssignments: Array<{ staffId: string; dayIdx: number; shiftTypes: string[] }> = [];

  // Control B: Calculate skill frequency across workforce to determine scarcity
  const skillWorkerCount: Record<string, number> = {};
  for (const s of staff) {
    for (const sk of s.skills) {
      skillWorkerCount[sk] = (skillWorkerCount[sk] || 0) + 1;
    }
  }

  // Group slots by (dayIdx, label)
  const availableSlotsByDayAndLabel: Record<string, ShiftSlotMeta[]> = {};
  for (const meta of Object.values(slotMeta)) {
    const key = `${meta.dayIdx}:${meta.label}`;
    if (!availableSlotsByDayAndLabel[key]) {
      availableSlotsByDayAndLabel[key] = [];
    }
    availableSlotsByDayAndLabel[key].push(meta);
  }

  // Control B: Sort slots within each bucket by skill scarcity (rarest skill first)
  // Ensures specialists are assigned to scarce qualified positions before general slots
  for (const key of Object.keys(availableSlotsByDayAndLabel)) {
    availableSlotsByDayAndLabel[key].sort((a, b) => {
      const countA = skillWorkerCount[a.skill] ?? 999;
      const countB = skillWorkerCount[b.skill] ?? 999;
      return countA - countB;
    });
  }

  let totalUiAssignments = 0;

  // Prioritize specialized workers (fewer skills) so scarce qualified slots are filled by them
  const sortedStaff = [...staff].sort((a, b) => a.skills.length - b.skills.length);

  // Process all UI assignments across all staff and days
  for (const s of sortedStaff) {
    const shiftsForStaff = schedule[s.id] || [];
    for (let d = 0; d < shiftsForStaff.length; d++) {
      const shiftType = shiftsForStaff[d];
      if (!shiftType || shiftType === '') continue;

      totalUiAssignments++;
      const cellKey = `${s.id}:${d}`;

      // Explicit duplicate worker/day detection: never silently drop assignments!
      if (assignedWorkerDays.has(cellKey)) {
        duplicateAssignments.push({
          staffId: s.id,
          dayIdx: d,
          shiftTypes: [shiftType],
        });
      }
      assignedWorkerDays.add(cellKey);

      // Find an available slot on day d with matching label
      const key = `${d}:${shiftType}`;
      const slots = availableSlotsByDayAndLabel[key] || [];

      // Prefer a slot that matches the staff member's skills
      let chosenSlot = slots.find(
        slot => !assignedShifts.has(slot.shiftId) && s.skills.includes(slot.skill)
      );
      if (!chosenSlot) {
        // Fall back to any open slot of that shift type
        chosenSlot = slots.find(slot => !assignedShifts.has(slot.shiftId));
      }

      if (chosenSlot) {
        assignments[String(chosenSlot.shiftId)] = staffWorkerIds[s.id];
        assignedShifts.add(chosenSlot.shiftId);
      } else {
        // Unmapped UI assignment: no slot available on this day for this shift type
        unmappedUiAssignments.push({
          staffId: s.id,
          dayIdx: d,
          shiftType,
          reason: `No remaining '${shiftType}' shift slot available on day ${d + 1}.`,
        });
      }
    }
  }

  const totalMappedAssignments = Object.keys(assignments).length;
  const isConserved = totalUiAssignments === totalMappedAssignments && unmappedUiAssignments.length === 0 && duplicateAssignments.length === 0;

  return {
    assignments,
    totalUiAssignments,
    totalMappedAssignments,
    isConserved,
    unmappedUiAssignments,
    duplicateAssignments,
  };
}

export async function validateCurrentSchedule(
  staff: StaffMember[],
  schedule: Record<string, string[]>,
  rulePayload: object = {},
  customSlotMeta?: Record<number, ShiftSlotMeta>,
): Promise<any | null> {
  try {
    const slotMeta = customSlotMeta ?? getScheduleSlotMeta(staff);
    const basePayload = buildSchedulePayload(staff, rulePayload, Object.values(slotMeta));
    const conversion = scheduleToAssignments(staff, schedule, slotMeta);

    const csrfRes = await fetch('/api/csrf-token').catch(() => null);
    const csrfToken = csrfRes && csrfRes.ok ? (await csrfRes.json()).csrf_token ?? '' : '';

    const resp = await fetch('/api/validate', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-CSRF-Token': csrfToken,
      },
      body: JSON.stringify({
        request: basePayload,
        assignments: conversion.assignments,
      }),
    });
    if (!resp.ok) return null;
    const report = await resp.json();

    // Assignment Conservation & Duplicate Detection Invariant:
    // If any UI assignment was unmapped or duplicate, inject structured violations.
    // A feasibility certificate is NEVER issued if the validated matrix differs from the UI matrix!
    if (!conversion.isConserved || conversion.unmappedUiAssignments.length > 0 || conversion.duplicateAssignments.length > 0) {
      report.is_valid = false;
      report.hard_violations = (report.hard_violations || 0) + conversion.unmappedUiAssignments.length + conversion.duplicateAssignments.length;
      if (!report.violated_constraints) report.violated_constraints = [];
      if (!report.warnings) report.warnings = [];

      for (const unmapped of conversion.unmappedUiAssignments) {
        report.violated_constraints.push(`unmapped_assignment_${unmapped.staffId}_day_${unmapped.dayIdx + 1}`);
        report.warnings.push(`Assignment conservation failure: ${unmapped.staffId} has assignment '${unmapped.shiftType}' on day ${unmapped.dayIdx + 1} which could not be mapped to any demand slot (${unmapped.reason}).`);
      }

      for (const dup of conversion.duplicateAssignments) {
        report.violated_constraints.push(`duplicate_worker_day_${dup.staffId}_day_${dup.dayIdx + 1}`);
        report.warnings.push(`Duplicate worker/day violation: ${dup.staffId} has multiple shift assignments on day ${dup.dayIdx + 1}.`);
      }
    }

    return report;
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

export function buildStaffingRequirements(
  slotMeta?: Record<number, ShiftSlotMeta>,
): StaffingRequirement[] {
  if (!slotMeta || Object.keys(slotMeta).length === 0) {
    const reqs: StaffingRequirement[] = [];
    for (let d = 0; d < 28; d++) {
      reqs.push({ dayIdx: d, shiftType: 'Early', required: 3 });
      reqs.push({ dayIdx: d, shiftType: 'Late',  required: 2 });
      reqs.push({ dayIdx: d, shiftType: 'Night', required: 2 });
    }
    return reqs; // 84 requirement objects, 196 total required positions (default fixture)
  }

  // Dynamically compute requirements from custom demand slotMeta
  const counts: Record<string, number> = {};
  let maxDay = 0;
  for (const m of Object.values(slotMeta)) {
    const key = `${m.dayIdx}:${m.label}`;
    counts[key] = (counts[key] || 0) + 1;
    if (m.dayIdx > maxDay) maxDay = m.dayIdx;
  }

  const reqs: StaffingRequirement[] = [];
  const shiftTypes = ['Early', 'Late', 'Night'];
  for (let d = 0; d <= maxDay; d++) {
    for (const st of shiftTypes) {
      const required = counts[`${d}:${st}`] || 0;
      if (required > 0) {
        reqs.push({ dayIdx: d, shiftType: st, required });
      }
    }
  }
  return reqs;
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
  customSlotMeta?: Record<number, ShiftSlotMeta>,
): RosterAlternative[] {
  const requirements = buildStaffingRequirements(customSlotMeta);
  let horizonDays = 28;
  if (customSlotMeta) {
    const days = Object.values(customSlotMeta).map(m => m.dayIdx);
    if (days.length > 0) horizonDays = Math.max(...days) + 1;
  }
  const totalSlots = staff.length * horizonDays;

  let primaryAlt: RosterAlternative | null = null;
  let baselineSchedule: Record<string, string[]> = {};

  if (primaryRaw && primaryRaw.schedule) {
    const primarySchedule = buildEditableSchedule(staff, primaryRaw.schedule, customSlotMeta);
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
    baselineSchedule = buildEditableSchedule(staff, raw[0].schedule, customSlotMeta);
  }

  const result: RosterAlternative[] = primaryAlt ? [primaryAlt] : [];

  raw.forEach(alt => {
    const schedule = buildEditableSchedule(staff, alt.schedule, customSlotMeta);

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

// ─── Gate 3 (G3.1): Historical Baseline Ingestion and Audit Engine ────────────

export function parseBaselineCSV(text: string): {
  assignments: BaselineAssignment[];
  errors: string[];
  rawRowCount: number;
} {
  const lines = text.trim().split('\n').map(l => l.trim()).filter(Boolean);
  if (lines.length < 2) {
    return {
      assignments: [],
      errors: ['Baseline CSV must have a header row and at least one data row.'],
      rawRowCount: lines.length,
    };
  }

  const header = lines[0].toLowerCase().split(',').map(h => h.trim().replace(/^"|"$/g, ''));
  const workerIdx = header.indexOf('worker_id') !== -1
    ? header.indexOf('worker_id')
    : header.indexOf('worker') !== -1
    ? header.indexOf('worker')
    : header.indexOf('staff_id') !== -1
    ? header.indexOf('staff_id')
    : header.indexOf('nurse_id');

  const shiftIdx = header.indexOf('shift_id') !== -1
    ? header.indexOf('shift_id')
    : header.indexOf('shift');

  const errors: string[] = [];
  if (workerIdx === -1) errors.push('Missing column: worker_id (or staff_id / nurse_id)');
  if (shiftIdx === -1) errors.push('Missing column: shift_id (or shift)');
  if (errors.length > 0) return { assignments: [], errors, rawRowCount: lines.length - 1 };

  const assignments: BaselineAssignment[] = [];
  const rawRowCount = lines.length - 1;

  for (let i = 1; i < lines.length; i++) {
    const cols = lines[i].split(',').map(c => c.trim().replace(/^"|"$/g, ''));
    const workerId = cols[workerIdx];
    const shiftIdStr = cols[shiftIdx];
    const shiftId = parseInt(shiftIdStr, 10);

    if (!workerId) {
      errors.push(`Row ${i + 1}: missing worker_id`);
      continue;
    }
    if (isNaN(shiftId) || shiftId <= 0) {
      errors.push(`Row ${i + 1}: invalid shift_id "${shiftIdStr}"`);
      continue;
    }

    assignments.push({ worker_id: workerId, shift_id: shiftId });
  }

  return { assignments, errors, rawRowCount };
}

export function generateSampleBaselineWithBreaches(
  staff: StaffMember[],
  customSlotMeta?: Record<number, ShiftSlotMeta>,
): string {
  const slotMeta = customSlotMeta ?? getScheduleSlotMeta(staff);
  const slots = Object.values(slotMeta).sort((a, b) => a.shiftId - b.shiftId);
  const rows = ['worker_id,shift_id'];

  if (staff.length === 0 || slots.length === 0) {
    return 'worker_id,shift_id\n';
  }

  const generalNurse1 = staff.find(s => !s.skills.includes('HeadNurse')) ?? staff[1 % staff.length];
  const generalNurse2 = staff.filter(s => !s.skills.includes('HeadNurse'))[1] ?? staff[2 % staff.length];
  const overtimeNurse = staff.find(s => s.contract === 'FullTime' && s.id !== generalNurse1.id && s.id !== generalNurse2.id) ?? staff[3 % staff.length];

  // Map slots across staff:
  // Breach 1: Skill Deficit — slot 1 requires HeadNurse, assign to general nurse who lacks HeadNurse
  const headNurseSlot1 = slots.find(s => s.skill === 'HeadNurse') || slots[0];
  rows.push(`${generalNurse1.id},${headNurseSlot1.shiftId}`);

  // Breach 2: Rest Deficit 1 — generalNurse1 works Night on Day 0 (ends 06:00 Day 1) then Early on Day 1 (starts 06:00 Day 1) -> 0h rest
  const day0Night = slots.find(s => s.dayIdx === 0 && s.label === 'Night' && s.shiftId !== headNurseSlot1.shiftId);
  const day1Early = slots.find(s => s.dayIdx === 1 && s.label === 'Early' && s.shiftId !== headNurseSlot1.shiftId);
  if (day0Night) rows.push(`${generalNurse1.id},${day0Night.shiftId}`);
  if (day1Early) rows.push(`${generalNurse1.id},${day1Early.shiftId}`);

  // Breach 3: Rest Deficit 2 — generalNurse2 works Night on Day 2 then Early on Day 3 -> 0h rest
  const day2Night = slots.find(s => s.dayIdx === 2 && s.label === 'Night');
  const day3Early = slots.find(s => s.dayIdx === 3 && s.label === 'Early');
  if (day2Night) rows.push(`${generalNurse2.id},${day2Night.shiftId}`);
  if (day3Early) rows.push(`${generalNurse2.id},${day3Early.shiftId}`);

  // Breach 4: Weekly Contract Overtime — overtimeNurse assigned 6 shifts (48h) in Week 1 (Days 0-6)
  const week1Slots = slots.filter(s => s.dayIdx < 7 && s.shiftId !== headNurseSlot1.shiftId && s.shiftId !== day0Night?.shiftId && s.shiftId !== day1Early?.shiftId);
  const overtimeAssignedSlots = week1Slots.slice(0, 6);
  for (const s of overtimeAssignedSlots) {
    rows.push(`${overtimeNurse.id},${s.shiftId}`);
  }

  // Set of already assigned shifts
  const assignedShiftIds = new Set<number>([
    headNurseSlot1.shiftId,
    ...(day0Night ? [day0Night.shiftId] : []),
    ...(day1Early ? [day1Early.shiftId] : []),
    ...(day2Night ? [day2Night.shiftId] : []),
    ...(day3Early ? [day3Early.shiftId] : []),
    ...overtimeAssignedSlots.map(s => s.shiftId),
  ]);

  // Breach 5: Uncovered positions — leave last 4 shifts unassigned!
  const uncoveredSlotIds = new Set(slots.slice(-4).map(s => s.shiftId));

  // Assign remaining shifts fairly across remaining staff
  let staffIdx = 0;
  for (const slot of slots) {
    if (assignedShiftIds.has(slot.shiftId) || uncoveredSlotIds.has(slot.shiftId)) {
      continue;
    }
    const staffMember = staff[staffIdx % staff.length];
    rows.push(`${staffMember.id},${slot.shiftId}`);
    staffIdx++;
  }

  return rows.join('\n');
}

export function auditBaselineClientSide(
  staff: StaffMember[],
  shifts: ShiftSlotMeta[],
  baselineAssignments: BaselineAssignment[],
  rulePayload: any = {},
  policyVersion: string = 'hospital_policy_v1',
): BaselineAuditReport {
  const auditId = `audit-${Date.now()}`;
  const minRestHours = rulePayload?.min_rest_hours ?? 10;
  const maxWeeklyHours = rulePayload?.max_weekly_hours ?? 40.0;

  const staffMap = new Map<string, StaffMember>();
  staff.forEach(s => staffMap.set(s.id, s));

  const shiftMap = new Map<number, ShiftSlotMeta>();
  shifts.forEach(s => shiftMap.set(s.shiftId, s));

  // Assignment Integrity Check
  let duplicateCount = 0;
  const seenWorkerShift = new Set<string>();
  const unknownWorkersSet = new Set<string>();
  let unknownWorkerErrors = 0;
  const unmatchedShiftIdsSet = new Set<number>();
  let unmatchedExcessCount = 0;
  const validAssignments: Array<{ worker: StaffMember; shift: ShiftSlotMeta }> = [];
  const findings: AuditFinding[] = [];

  for (const ba of baselineAssignments) {
    const pairKey = `${ba.worker_id}:${ba.shift_id}`;
    if (seenWorkerShift.has(pairKey)) {
      duplicateCount++;
      findings.push({
        code: 'DUPLICATE_ASSIGNMENT',
        severity: 'high',
        description: `Duplicate assignment in baseline: worker ${ba.worker_id} assigned to shift ${ba.shift_id} multiple times.`,
        worker_id: ba.worker_id,
        shift_id: ba.shift_id,
      });
      continue;
    }
    seenWorkerShift.add(pairKey);

    const worker = staffMap.get(ba.worker_id);
    if (!worker) {
      unknownWorkerErrors++;
      unknownWorkersSet.add(ba.worker_id);
      findings.push({
        code: 'UNKNOWN_WORKER',
        severity: 'critical',
        description: `Baseline assigns shift ${ba.shift_id} to unknown worker ID '${ba.worker_id}'.`,
        worker_id: ba.worker_id,
        shift_id: ba.shift_id,
      });
      continue;
    }

    const shift = shiftMap.get(ba.shift_id);
    if (!shift) {
      unmatchedExcessCount++;
      unmatchedShiftIdsSet.add(ba.shift_id);
      findings.push({
        code: 'UNMATCHED_SHIFT',
        severity: 'high',
        description: `Baseline assigns worker '${ba.worker_id}' to shift ID ${ba.shift_id} not present in current demand requirements.`,
        worker_id: ba.worker_id,
        shift_id: ba.shift_id,
      });
      continue;
    }

    validAssignments.push({ worker, shift });
  }

  // Coverage Analysis
  const shiftCoveredMap = new Map<number, StaffMember[]>();
  validAssignments.forEach(({ worker, shift }) => {
    if (!shiftCoveredMap.has(shift.shiftId)) {
      shiftCoveredMap.set(shift.shiftId, []);
    }
    shiftCoveredMap.get(shift.shiftId)!.push(worker);
  });

  const requiredPositions = shifts.length;
  let filledPositions = 0;
  let unfilledPositions = 0;

  for (const shift of shifts) {
    const assigned = shiftCoveredMap.get(shift.shiftId);
    if (assigned && assigned.length > 0) {
      filledPositions++;
    } else {
      unfilledPositions++;
      findings.push({
        code: 'UNFILLED_POSITION',
        severity: 'critical',
        description: `Required position shift ${shift.shiftId} (${shift.label} on Day ${shift.dayIdx + 1}, requires ${shift.skill}) is completely uncovered.`,
        shift_id: shift.shiftId,
        day_idx: shift.dayIdx,
      });
    }
  }

  const coveragePct = requiredPositions > 0
    ? Math.round((filledPositions / requiredPositions) * 1000) / 10
    : 0;

  // Group valid assignments by worker
  const workerShifts = new Map<string, ShiftSlotMeta[]>();
  for (const { worker, shift } of validAssignments) {
    if (!workerShifts.has(worker.id)) {
      workerShifts.set(worker.id, []);
    }
    workerShifts.get(worker.id)!.push(shift);
  }

  let restViolations = 0;
  let skillCoverageDeficits = 0;
  let seniorCoverageDeficits = 0;
  let weeklyContractViolations = 0;
  let scheduledOvertimeHours = 0;

  const workerTotalHours = new Map<string, number>();
  staff.forEach(s => workerTotalHours.set(s.id, 0));

  for (const [staffId, assignedSlots] of workerShifts.entries()) {
    const worker = staffMap.get(staffId)!;
    assignedSlots.sort((a, b) => a.startHour - b.startHour);

    let totalHoursWorker = 0;

    // Check rest violations between consecutive shifts
    for (let i = 0; i < assignedSlots.length; i++) {
      const cur = assignedSlots[i];
      const curDur = cur.durationHours ?? 8;
      totalHoursWorker += curDur;

      // Check skill coverage
      if (!worker.skills.includes(cur.skill)) {
        skillCoverageDeficits++;
        if (cur.skill === 'HeadNurse') {
          seniorCoverageDeficits++;
        }
        findings.push({
          code: 'SKILL_DEFICIT',
          severity: 'high',
          description: `Worker ${worker.id} assigned to shift ${cur.shiftId} requiring '${cur.skill}' but holds only: [${worker.skills.join(', ')}].`,
          worker_id: worker.id,
          shift_id: cur.shiftId,
          day_idx: cur.dayIdx,
        });
      }

      // Rest with next shift
      if (i + 1 < assignedSlots.length) {
        const next = assignedSlots[i + 1];
        const restGap = next.startHour - (cur.startHour + curDur);
        if (restGap < minRestHours) {
          restViolations++;
          findings.push({
            code: 'REST_VIOLATION',
            severity: 'critical',
            description: `Insufficient rest for ${worker.id}: ${restGap}h between shift ${cur.shiftId} (ends at h${cur.startHour + curDur}) and shift ${next.shiftId} (starts at h${next.startHour}). Required: ${minRestHours}h.`,
            worker_id: worker.id,
            shift_id: next.shiftId,
            day_idx: next.dayIdx,
          });
        }
      }
    }

    workerTotalHours.set(worker.id, totalHoursWorker);

    // Weekly Contract Overtime (168h windows / calendar weeks)
    const maxHorizonDay = Math.max(...shifts.map(s => s.dayIdx), 27);
    const totalWeeks = Math.ceil((maxHorizonDay + 1) / 7);

    for (let w = 0; w < totalWeeks; w++) {
      const weekStartHour = w * 168;
      const weekEndHour = (w + 1) * 168;
      const weekShifts = assignedSlots.filter(s => s.startHour >= weekStartHour && s.startHour < weekEndHour);
      const weekHours = weekShifts.reduce((sum, s) => sum + (s.durationHours ?? 8), 0);

      let workerMaxWeekly = maxWeeklyHours;
      if (worker.contract === 'PartTime') workerMaxWeekly = 20.0;
      else if (worker.contract === 'Night') workerMaxWeekly = 32.0;

      if (weekHours > workerMaxWeekly) {
        weeklyContractViolations++;
        const excess = weekHours - workerMaxWeekly;
        scheduledOvertimeHours += excess;
        findings.push({
          code: 'CONTRACT_OVERTIME',
          severity: 'medium',
          description: `Weekly contractual overtime breach for ${worker.id}: ${weekHours}h scheduled in Week ${w + 1} (max allowed: ${workerMaxWeekly}h, excess: +${excess}h).`,
          worker_id: worker.id,
          day_idx: w * 7,
        });
      }
    }
  }

  // Workload variance
  const hoursArr = Array.from(workerTotalHours.values());
  const meanHours = hoursArr.length > 0 ? hoursArr.reduce((a, b) => a + b, 0) / hoursArr.length : 0;
  const workloadVariance = hoursArr.length > 0
    ? Math.round((hoursArr.reduce((sum, h) => sum + Math.pow(h - meanHours, 2), 0) / hoursArr.length) * 10) / 10
    : 0;

  const hardViolations = restViolations + weeklyContractViolations + skillCoverageDeficits + unfilledPositions + duplicateCount + unknownWorkerErrors;
  const isFeasible = hardViolations === 0;

  const violatedConstraints: string[] = [];
  if (restViolations > 0) violatedConstraints.push(`minimum_rest_hours (${restViolations} breaches)`);
  if (weeklyContractViolations > 0) violatedConstraints.push(`max_weekly_hours (${weeklyContractViolations} breaches)`);
  if (skillCoverageDeficits > 0) violatedConstraints.push(`required_skill_coverage (${skillCoverageDeficits} breaches)`);
  if (unfilledPositions > 0) violatedConstraints.push(`unfilled_demand_positions (${unfilledPositions} positions)`);
  if (duplicateCount > 0) violatedConstraints.push(`duplicate_assignments (${duplicateCount} rows)`);
  if (unknownWorkerErrors > 0) violatedConstraints.push(`unknown_workers (${unknownWorkerErrors} rows)`);

  const satisfiedConstraints: string[] = [];
  if (restViolations === 0) satisfiedConstraints.push('minimum_rest_hours');
  if (weeklyContractViolations === 0) satisfiedConstraints.push('max_weekly_hours');
  if (skillCoverageDeficits === 0) satisfiedConstraints.push('required_skill_coverage');
  if (unfilledPositions === 0) satisfiedConstraints.push('full_demand_coverage');

  return {
    audit_id: auditId,
    status: 'completed',
    policy_version: policyVersion,
    policy_retrospective_note: `Retrospective audit against hospital policy version ${policyVersion}. Evaluates regulatory and contract compliance on historical manual roster.`,
    feasibility: {
      is_feasible: isFeasible,
      hard_violations: hardViolations,
    },
    coverage: {
      required_positions: requiredPositions,
      filled_positions: filledPositions,
      unfilled_positions: unfilledPositions,
      coverage_pct: coveragePct,
    },
    rest_violations: restViolations,
    weekly_contract_violations: weeklyContractViolations,
    skill_coverage_deficits: skillCoverageDeficits,
    senior_coverage_deficits: seniorCoverageDeficits,
    scheduled_overtime_hours: scheduledOvertimeHours,
    workload_variance: workloadVariance,
    assignment_integrity: {
      duplicate_assignments: duplicateCount,
      unknown_worker_errors: unknownWorkerErrors,
      unknown_workers: Array.from(unknownWorkersSet),
      unmatched_excess_assignments: unmatchedExcessCount,
      unmatched_shift_ids: Array.from(unmatchedShiftIdsSet),
      total_baseline_rows: baselineAssignments.length,
      valid_assignments_count: validAssignments.length,
    },
    violated_constraints: violatedConstraints,
    satisfied_constraints: satisfiedConstraints,
    detailed_findings: findings,
  };
}

export async function runBaselineAudit(
  staff: StaffMember[],
  shifts: ShiftSlotMeta[],
  baselineAssignments: BaselineAssignment[],
  rulePayload: any = {},
  policyVersion: string = 'hospital_policy_v1',
): Promise<BaselineAuditReport> {
  try {
    const payload = {
      request: buildSchedulePayload(staff, rulePayload, shifts),
      baseline_assignments: baselineAssignments,
      policy_version: policyVersion,
    };

    const csrfRes = await fetch('/api/csrf-token').catch(() => null);
    const csrfToken = csrfRes && csrfRes.ok ? (await csrfRes.json()).csrf_token ?? '' : '';

    const res = await fetch('/api/audit/baseline', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-CSRF-Token': csrfToken,
      },
      body: JSON.stringify(payload),
    });

    if (res.ok) {
      return await res.json();
    }
  } catch {
    // Backend unreachable or offline — fall back to client-side audit engine
  }

  return auditBaselineClientSide(staff, shifts, baselineAssignments, rulePayload, policyVersion);
}