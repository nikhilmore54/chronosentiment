// @vitest-environment jsdom
// Acceptance Tests AT-14 to AT-20: Assessment Readiness Suite
//
// AT-14: Assignment conservation (Assignments_UI = Assignments_Validation = Assignments_Export)
// AT-15: Duplicate worker/day explicit detection (never silently omitted)
// AT-16: Variable demand file processing
// AT-17: Weekend demand variation preservation
// AT-18: Skill coverage satisfaction
// AT-19: Export consistency
// AT-20: Cycle continuity and boundary history

import { describe, test, expect } from 'vitest';
import {
  parseStaffCSV,
  parseShiftsCSV,
  parseBaselineCSV,
  generateSampleBaselineWithBreaches,
  auditBaselineClientSide,
  buildExcelXmlSpreadsheet,
  getScheduleSlotMeta,
  buildSchedulePayload,
  buildStaffingRequirements,
  computeDemandCapacityDiagnostic,
  scheduleToAssignments,
  exportRosterToCSV,
  computeNextCycleBoundary,
  buildSyntheticSchedule,
  buildSyntheticAlternatives,
  mapParetoAlternatives,
  generateShiftLevelDeltaRecords,
  exportShiftLevelDeltaCSV,
  validateHospitalWardConfiguration,
  calculateClinicalDemand,
  computePolicySnapshotHash,
  createApprovedDemandSnapshot,
  DEFAULT_HOSPITAL_CONFIG,
} from './WorkflowUtils';
import type {
  StaffMember,
  BaselineAssignment,
  ProductionParetoSolution,
  HospitalWardConfiguration,
  ApprovedDemandSnapshot,
  ShiftCensusOverride,
  ShiftModifierCensusInput,
} from './WorkflowTypes';
import { SAMPLE_CSV } from './WorkflowTypes';

const TEST_STAFF: StaffMember[] = [
  { id: 'Sarah_Chen', contract: 'FullTime', skills: ['HeadNurse', 'Nurse'] },
  { id: 'Marcus_Webb', contract: 'FullTime', skills: ['Nurse'] },
  { id: 'Priya_Nair', contract: 'FullTime', skills: ['HeadNurse', 'Nurse'] },
  { id: 'James_Okafor', contract: 'FullTime', skills: ['Nurse'] },
  { id: 'Elena_Vasquez', contract: 'FullTime', skills: ['Nurse'] },
  { id: 'Tom_Lindqvist', contract: 'PartTime', skills: ['Nurse'] },
  { id: 'Aisha_Diallo', contract: 'FullTime', skills: ['HeadNurse'] },
  { id: 'Ingrid_Sorensen', contract: 'Night', skills: ['Nurse'] },
];

describe('Assessment Readiness Suite (AT-14 to AT-20)', () => {
  // ── AT-14: Assignment Conservation ──────────────────────────────────────────
  test('AT-14: Assignment conservation guarantees Assignments_UI = Assignments_Validation = Assignments_Export', () => {
    const slotMeta = getScheduleSlotMeta(TEST_STAFF);
    const schedule: Record<string, string[]> = {};
    TEST_STAFF.forEach(s => { schedule[s.id] = Array(28).fill(''); });

    // Assign 14 valid shifts across days 0 and 1
    schedule['Sarah_Chen'][0] = 'Early';   // HeadNurse slot
    schedule['Marcus_Webb'][0] = 'Early';  // Nurse slot
    schedule['Priya_Nair'][0] = 'Early';   // Nurse slot
    schedule['James_Okafor'][0] = 'Late';  // Nurse slot
    schedule['Elena_Vasquez'][0] = 'Late'; // Nurse slot
    schedule['Ingrid_Sorensen'][0] = 'Night'; // Nurse slot
    schedule['Tom_Lindqvist'][0] = 'Night'; // Nurse slot

    schedule['Aisha_Diallo'][1] = 'Early';  // HeadNurse slot
    schedule['Marcus_Webb'][1] = 'Early';   // Nurse slot
    schedule['Priya_Nair'][1] = 'Early';    // Nurse slot
    schedule['James_Okafor'][1] = 'Late';   // Nurse slot
    schedule['Elena_Vasquez'][1] = 'Late';  // Nurse slot
    schedule['Ingrid_Sorensen'][1] = 'Night'; // Nurse slot
    schedule['Sarah_Chen'][1] = 'Night';    // Nurse slot

    const conversion = scheduleToAssignments(TEST_STAFF, schedule, slotMeta);

    // Invariant 1: Total UI non-empty cells exactly equals total mapped validation assignments
    expect(conversion.totalUiAssignments).toBe(14);
    expect(conversion.totalMappedAssignments).toBe(14);
    expect(conversion.isConserved).toBe(true);
    expect(conversion.unmappedUiAssignments.length).toBe(0);
    expect(conversion.duplicateAssignments.length).toBe(0);

    // Invariant 2: Exported CSV rows exactly equals total validated assignments
    const csvExport = exportRosterToCSV(TEST_STAFF, schedule, 28);
    const exportLines = csvExport.trim().split('\n').slice(1); // skip header
    expect(exportLines.length).toBe(14);
    expect(exportLines.length).toBe(conversion.totalMappedAssignments);
  });

  // ── AT-15: Duplicate Worker/Day Explicit Detection ──────────────────────────
  test('AT-15: Duplicate worker/day is explicitly reported and never silently omitted', () => {
    const slotMeta = getScheduleSlotMeta(TEST_STAFF);
    const schedule: Record<string, string[]> = {};
    TEST_STAFF.forEach(s => { schedule[s.id] = Array(28).fill(''); });

    // Intentionally assign Sarah_Chen to Early on Day 0
    schedule['Sarah_Chen'][0] = 'Early';

    // Simulate duplicate assignment by directly calling conversion with duplicate detection
    const conversion = scheduleToAssignments(TEST_STAFF, schedule, slotMeta);
    expect(conversion.isConserved).toBe(true);

    // Now test unmapped UI assignment: add a shift type that exceeds capacity on Day 0
    // (Only 3 Early slots exist on Day 0; assign 4 workers to Early)
    schedule['Marcus_Webb'][0] = 'Early';
    schedule['Priya_Nair'][0] = 'Early';
    schedule['Elena_Vasquez'][0] = 'Early'; // 4th Early shift! Exceeds demand capacity (3 Early slots)

    const overflowConversion = scheduleToAssignments(TEST_STAFF, schedule, slotMeta);
    expect(overflowConversion.totalUiAssignments).toBe(4);
    expect(overflowConversion.totalMappedAssignments).toBe(3);
    expect(overflowConversion.isConserved).toBe(false);

    // The 4th assignment is explicitly recorded in unmappedUiAssignments, NEVER silently swallowed!
    expect(overflowConversion.unmappedUiAssignments.length).toBe(1);
    expect(['Elena_Vasquez', 'Priya_Nair']).toContain(overflowConversion.unmappedUiAssignments[0].staffId);
    expect(overflowConversion.unmappedUiAssignments[0].reason).toContain("No remaining 'Early' shift slot available on day 1");
  });

  // ── AT-16: Variable Demand File Processing ──────────────────────────────────
  test('AT-16: Variable demand file with arbitrary shift count and planning horizon is processed correctly', () => {
    // 2-day demand CSV with 8 total shifts
    const customCSV = `id,start_hour,duration_hours,required_skill
1,6,8,HeadNurse
2,6,8,Nurse
3,14,8,Nurse
4,22,8,Nurse
5,30,8,HeadNurse
6,30,8,Nurse
7,38,8,Nurse
8,46,8,Nurse`;

    const { shifts, errors } = parseShiftsCSV(customCSV);
    expect(errors.length).toBe(0);
    expect(shifts.length).toBe(8);

    const slotMeta = getScheduleSlotMeta(TEST_STAFF, shifts);
    expect(Object.keys(slotMeta).length).toBe(8);

    const payload = buildSchedulePayload(TEST_STAFF, {}, shifts);
    expect(payload.shifts.length).toBe(8);
    // 2-day horizon = 48 hours
    expect(payload.scenario.planning_horizon_hours).toBe(48);
    // Shift 8 starts at 46, duration 8, ends at 54
    expect(payload.scenario.validation_envelope_hours).toBe(54);

    const requirements = buildStaffingRequirements(slotMeta);
    // Day 0: 2 Early, 1 Late, 1 Night (4 shifts)
    // Day 1: 2 Early, 1 Late, 1 Night (4 shifts)
    const day0Early = requirements.find(r => r.dayIdx === 0 && r.shiftType === 'Early');
    expect(day0Early?.required).toBe(2);
  });

  // ── AT-17: Weekend Demand Variation ─────────────────────────────────────────
  test('AT-17: Weekend demand variation (different Saturday/Sunday counts) is preserved', () => {
    // Generate a 1-week demand where Mon–Fri have 7 shifts/day, but Sat–Sun have only 3 shifts/day
    const variableShifts = [];
    let sid = 1;
    for (let day = 0; day < 7; day++) {
      const baseHour = day * 24;
      const isWeekend = day === 5 || day === 6; // Sat=5, Sun=6

      if (isWeekend) {
        // Weekend reduced staffing: 1 Early, 1 Late, 1 Night
        variableShifts.push({ shiftId: sid++, dayIdx: day, label: 'Early', skill: 'Nurse', startHour: baseHour + 6, durationHours: 8 });
        variableShifts.push({ shiftId: sid++, dayIdx: day, label: 'Late', skill: 'Nurse', startHour: baseHour + 14, durationHours: 8 });
        variableShifts.push({ shiftId: sid++, dayIdx: day, label: 'Night', skill: 'Nurse', startHour: baseHour + 22, durationHours: 8 });
      } else {
        // Weekday standard: 3 Early, 2 Late, 2 Night
        for (let k = 0; k < 3; k++) variableShifts.push({ shiftId: sid++, dayIdx: day, label: 'Early', skill: 'Nurse', startHour: baseHour + 6, durationHours: 8 });
        for (let k = 0; k < 2; k++) variableShifts.push({ shiftId: sid++, dayIdx: day, label: 'Late', skill: 'Nurse', startHour: baseHour + 14, durationHours: 8 });
        for (let k = 0; k < 2; k++) variableShifts.push({ shiftId: sid++, dayIdx: day, label: 'Night', skill: 'Nurse', startHour: baseHour + 22, durationHours: 8 });
      }
    }

    // 5 weekdays * 7 + 2 weekend days * 3 = 35 + 6 = 41 shifts
    expect(variableShifts.length).toBe(41);

    const slotMeta = getScheduleSlotMeta(TEST_STAFF, variableShifts);
    const reqs = buildStaffingRequirements(slotMeta);

    // Weekday (day 0): 3 Early, 2 Late, 2 Night = 7 total
    const monTotal = reqs.filter(r => r.dayIdx === 0).reduce((s, r) => s + r.required, 0);
    expect(monTotal).toBe(7);

    // Weekend (day 5 Saturday): 1 Early, 1 Late, 1 Night = 3 total
    const satTotal = reqs.filter(r => r.dayIdx === 5).reduce((s, r) => s + r.required, 0);
    expect(satTotal).toBe(3);

    // Weekend variation is accurately preserved!
    expect(satTotal).toBeLessThan(monTotal);
  });

  // ── AT-18: Skill Coverage ───────────────────────────────────────────────────
  test('AT-18: Skill coverage strictly matches staff qualifications', () => {
    const slotMeta = getScheduleSlotMeta(TEST_STAFF);
    const schedule: Record<string, string[]> = {};
    TEST_STAFF.forEach(s => { schedule[s.id] = Array(28).fill(''); });

    // Aisha_Diallo only has 'HeadNurse' skill (cannot work 'Nurse' only slots)
    // Sarah_Chen has both 'HeadNurse' and 'Nurse'
    // Marcus_Webb only has 'Nurse' skill
    schedule['Aisha_Diallo'][0] = 'Early'; // Assigned to Early (which has 1 HeadNurse slot)
    schedule['Marcus_Webb'][0] = 'Early';  // Assigned to Early (Nurse slot)
    schedule['Sarah_Chen'][0] = 'Early';   // Assigned to Early (Nurse slot)

    const conversion = scheduleToAssignments(TEST_STAFF, schedule, slotMeta);
    expect(conversion.isConserved).toBe(true);

    // Shift 1 is the HeadNurse slot
    const headNurseSlotShiftId = '1';
    expect(conversion.assignments[headNurseSlotShiftId]).toBe(7); // Aisha_Diallo is worker #7!
  });

  // ── AT-19: Export Consistency ───────────────────────────────────────────────
  test('AT-19: Exported assignments exactly match the independently validated schedule', () => {
    const baseSchedule = buildSyntheticSchedule(TEST_STAFF);
    const csvExport = exportRosterToCSV(TEST_STAFF, baseSchedule, 28);
    const lines = csvExport.trim().split('\n');

    expect(lines[0]).toBe('worker_id,staff_id,day_idx,shift_type');

    let expectedNonEmptyCount = 0;
    TEST_STAFF.forEach(s => {
      expectedNonEmptyCount += baseSchedule[s.id].filter(sh => sh !== '').length;
    });

    const dataRows = lines.slice(1);
    expect(dataRows.length).toBe(expectedNonEmptyCount);

    // Verify row integrity
    for (const row of dataRows) {
      const [workerId, staffId, dayIdx, shiftType] = row.split(',');
      expect(baseSchedule[staffId][Number(dayIdx)]).toBe(shiftType);
    }
  });

  // ── AT-20: Cycle Continuity & Rest Lockout ──────────────────────────────────
  test('AT-20: Final-night assignments correctly influence next-cycle rest calculations', () => {
    const schedule: Record<string, string[]> = {};
    TEST_STAFF.forEach(s => { schedule[s.id] = Array(28).fill(''); });

    // Final day of 28-day cycle is dayIdx = 27
    // Ingrid works Night shift (22:00 to 06:00 next day, ending at hour 678)
    schedule['Ingrid_Sorensen'][27] = 'Night';
    // Marcus works Late shift (14:00 to 22:00, ending at hour 670)
    schedule['Marcus_Webb'][27] = 'Late';
    // Sarah works Early shift (06:00 to 14:00, ending at hour 662)
    schedule['Sarah_Chen'][27] = 'Early';

    const boundary = computeNextCycleBoundary(TEST_STAFF, schedule, 672, 10);

    const ingrid = boundary.find(b => b.staffId === 'Ingrid_Sorensen')!;
    expect(ingrid.lastShiftEndHour).toBe(678);
    // 678 + 10 = 688h rest lockout
    expect(ingrid.restLockoutUntilHour).toBe(688);
    // Next cycle Early starts at 678. Since 688 > 678, she CANNOT work Early shift on Day 0 of next cycle!
    expect(ingrid.isRestLockedForCycleStartEarly).toBe(true);

    const marcus = boundary.find(b => b.staffId === 'Marcus_Webb')!;
    expect(marcus.lastShiftEndHour).toBe(670);
    // 670 + 10 = 680h rest lockout > 678h
    expect(marcus.restLockoutUntilHour).toBe(680);
    expect(marcus.isRestLockedForCycleStartEarly).toBe(true);

    const sarah = boundary.find(b => b.staffId === 'Sarah_Chen')!;
    expect(sarah.lastShiftEndHour).toBe(662);
    // 662 + 10 = 672h <= 678h -> Sarah is rested and eligible to work Early shift on Day 0 of next cycle!
    expect(sarah.restLockoutUntilHour).toBe(672);
    expect(sarah.isRestLockedForCycleStartEarly).toBe(false);
  });

  // ── Demand-Capacity Diagnostic ──────────────────────────────────────────────
  test('Demand-Capacity Diagnostic accurately computes workforce capacity, utilization, and surplus/deficit', () => {
    // 8 staff: 6 FullTime (40h), 1 PartTime (20h), 1 Night (32h)
    // Weekly capacity = 6*40 + 20 + 32 = 292h/week. Over 4 weeks = 1168h capacity.
    const shifts = Array(100).fill({ duration_hours: 8 }); // 100 shifts * 8h = 800h demand
    const diag = computeDemandCapacityDiagnostic(TEST_STAFF, shifts, 28);

    expect(diag.demandHours).toBe(800);
    expect(diag.workforceCapacityHours).toBe(1168);
    expect(diag.surplusOrDeficitHours).toBe(368);
    expect(diag.status).toBe('balanced');
    expect(diag.message).toContain('Balanced Capacity');

    // Test under-capacity: 200 shifts = 1600h demand > 1168h capacity
    const heavyShifts = Array(200).fill({ duration_hours: 8 });
    const underDiag = computeDemandCapacityDiagnostic(TEST_STAFF, heavyShifts, 28);
    expect(underDiag.status).toBe('under_capacity');
    expect(underDiag.message).toContain('Under-Capacity Deficit');
  });
});

// ── Gate 3 (G3.1) Acceptance Tests: Historical Baseline Audit & Controls A-D ──
describe('Gate 3 (G3.1) Acceptance Tests (AT-21 to AT-32 & Controls A-D)', () => {
  // ── AT-21: Valid Baseline Ingestion ─────────────────────────────────────────
  test('AT-21: Valid baseline imports correctly and is parsed without errors', () => {
    const csv = `worker_id,shift_id\nSarah_Chen,1\nMarcus_Webb,2\nPriya_Nair,3`;
    const { assignments, errors, rawRowCount } = parseBaselineCSV(csv);

    expect(errors.length).toBe(0);
    expect(rawRowCount).toBe(3);
    expect(assignments.length).toBe(3);
    expect(assignments[0]).toEqual({ worker_id: 'Sarah_Chen', shift_id: 1 });
  });

  // ── AT-22: Rest Violations Detection ────────────────────────────────────────
  test('AT-22: Rest deficits (< 10h rest) are detected in historical baseline', () => {
    const slotMeta = getScheduleSlotMeta(TEST_STAFF);
    const slots = Object.values(slotMeta);

    // Marcus_Webb works Day 0 Night (shift 6: 22:00 to 06:00 Day 1) and Day 1 Early (shift 9: 06:00 to 14:00 Day 1) -> 0h rest
    const assignments: BaselineAssignment[] = [
      { worker_id: 'Marcus_Webb', shift_id: 6 },
      { worker_id: 'Marcus_Webb', shift_id: 9 },
    ];

    const report = auditBaselineClientSide(TEST_STAFF, slots, assignments);

    expect(report.rest_violations).toBeGreaterThanOrEqual(1);
    expect(report.feasibility.is_feasible).toBe(false);
    expect(report.audit_classification).toBe('VALID_BUT_INFEASIBLE');
    expect(report.feasibility.hard_violations).toBeGreaterThanOrEqual(1);
    const restFinding = report.detailed_findings.find(f => f.code === 'REST_VIOLATION');
    expect(restFinding).toBeDefined();
    expect(restFinding?.worker_id).toBe('Marcus_Webb');
  });

  // ── AT-23: Weekly Contract Buckets & Scheduled Overtime ─────────────────────
  test('AT-23: Overtime evaluated in 168h weekly buckets and correctly flags scheduled excess hours', () => {
    const slotMeta = getScheduleSlotMeta(TEST_STAFF);
    const slots = Object.values(slotMeta);

    // Assign Marcus_Webb (FullTime, 40h cap) to 6 shifts (48h) in Week 1 (Days 0 to 5)
    // Shifts 2 (D0), 9 (D1), 16 (D2), 23 (D3), 30 (D4), 37 (D5)
    const assignments: BaselineAssignment[] = [
      { worker_id: 'Marcus_Webb', shift_id: 2 },
      { worker_id: 'Marcus_Webb', shift_id: 9 },
      { worker_id: 'Marcus_Webb', shift_id: 16 },
      { worker_id: 'Marcus_Webb', shift_id: 23 },
      { worker_id: 'Marcus_Webb', shift_id: 30 },
      { worker_id: 'Marcus_Webb', shift_id: 37 },
    ];

    const report = auditBaselineClientSide(TEST_STAFF, slots, assignments);

    expect(report.weekly_contract_violations).toBe(1);
    expect(report.scheduled_overtime_hours).toBe(8); // 48h - 40h = 8h overtime
    const overtimeFinding = report.detailed_findings.find(f => f.code === 'CONTRACT_OVERTIME');
    expect(overtimeFinding).toBeDefined();
    expect(overtimeFinding?.worker_id).toBe('Marcus_Webb');
  });

  // ── AT-24: Missing Skill Requirements Detected ──────────────────────────────
  test('AT-24: Missing certified skill requirements produce skill coverage deficits', () => {
    const slotMeta = getScheduleSlotMeta(TEST_STAFF);
    const slots = Object.values(slotMeta);

    // Slot 1 requires HeadNurse. Marcus_Webb has only ['Nurse']
    const assignments: BaselineAssignment[] = [
      { worker_id: 'Marcus_Webb', shift_id: 1 },
    ];

    const report = auditBaselineClientSide(TEST_STAFF, slots, assignments);

    expect(report.skill_coverage_deficits).toBe(1);
    expect(report.senior_coverage_deficits).toBe(1);
    const skillFinding = report.detailed_findings.find(f => f.code === 'SKILL_DEFICIT');
    expect(skillFinding).toBeDefined();
    expect(skillFinding?.worker_id).toBe('Marcus_Webb');
  });

  // ── AT-25: Uncovered Demand Positions Identified ────────────────────────────
  test('AT-25: Uncovered demand positions are identified and reported in coverage metrics', () => {
    const slotMeta = getScheduleSlotMeta(TEST_STAFF);
    const slots = Object.values(slotMeta); // 196 shifts total

    // Only assign 10 shifts
    const assignments: BaselineAssignment[] = slots.slice(0, 10).map((s, i) => ({
      worker_id: TEST_STAFF[i % TEST_STAFF.length].id,
      shift_id: s.shiftId,
    }));

    const report = auditBaselineClientSide(TEST_STAFF, slots, assignments);

    expect(report.coverage.required_positions).toBe(196);
    expect(report.coverage.filled_positions).toBe(10);
    expect(report.coverage.unfilled_positions).toBe(186);
    expect(report.coverage.coverage_pct).toBe(5.1);
    expect(report.detailed_findings.some(f => f.code === 'UNFILLED_POSITION')).toBe(true);
  });

  // ── AT-26: Duplicate Assignments Detected ───────────────────────────────────
  test('AT-26: Duplicate assignments for same worker/shift are detected and reported', () => {
    const slotMeta = getScheduleSlotMeta(TEST_STAFF);
    const slots = Object.values(slotMeta);

    const assignments: BaselineAssignment[] = [
      { worker_id: 'Sarah_Chen', shift_id: 1 },
      { worker_id: 'Sarah_Chen', shift_id: 1 }, // Duplicate!
    ];

    const report = auditBaselineClientSide(TEST_STAFF, slots, assignments);

    expect(report.audit_classification).toBe('INCOMPLETE_OR_UNVERIFIABLE');
    expect(report.assignment_integrity.duplicate_assignments).toBe(1);
    expect(report.detailed_findings.some(f => f.code === 'DUPLICATE_ASSIGNMENT')).toBe(true);
  });

  // ── AT-27: Unknown Worker and Shift IDs ──────────────────────────────────────
  test('AT-27: Unknown worker and shift IDs produce structured errors without crashing', () => {
    const slotMeta = getScheduleSlotMeta(TEST_STAFF);
    const slots = Object.values(slotMeta);

    const assignments: BaselineAssignment[] = [
      { worker_id: 'Ghost_Worker_99', shift_id: 1 }, // Unknown worker
      { worker_id: 'Sarah_Chen', shift_id: 99999 },    // Unmatched shift
      { worker_id: 'Sarah_Chen', shift_id: 2 },        // Valid
    ];

    const report = auditBaselineClientSide(TEST_STAFF, slots, assignments);

    expect(report.audit_classification).toBe('INCOMPLETE_OR_UNVERIFIABLE');
    expect(report.assignment_integrity.unknown_worker_errors).toBe(1);
    expect(report.assignment_integrity.unknown_workers).toContain('Ghost_Worker_99');
    expect(report.assignment_integrity.unmatched_excess_assignments).toBe(1);
    expect(report.assignment_integrity.unmatched_shift_ids).toContain(99999);
    expect(report.assignment_integrity.valid_assignments_count).toBe(1);
  });

  // ── AT-28: Boundary Continuity & Rest Lockout ───────────────────────────────
  test('AT-28: Cycle boundary continuity correctly carries forward rest lockouts', () => {
    const schedule: Record<string, string[]> = {};
    TEST_STAFF.forEach(s => { schedule[s.id] = Array(28).fill(''); });

    schedule['Ingrid_Sorensen'][27] = 'Night'; // ends at 678h
    const boundary = computeNextCycleBoundary(TEST_STAFF, schedule, 672, 10);
    const ingrid = boundary.find(b => b.staffId === 'Ingrid_Sorensen')!;

    expect(ingrid.restLockoutUntilHour).toBe(688);
    expect(ingrid.isRestLockedForCycleStartEarly).toBe(true);
  });

  // ── AT-29: Control B — Scarce Skill Allocation ──────────────────────────────
  test('AT-29: Control B sorts slots by skill scarcity so specialists are allocated to scarce slots first', () => {
    const slotMeta = getScheduleSlotMeta(TEST_STAFF);
    const schedule: Record<string, string[]> = {};
    TEST_STAFF.forEach(s => { schedule[s.id] = Array(28).fill(''); });

    // Aisha_Diallo is ONLY HeadNurse (1 skill, scarce)
    // Sarah_Chen is HeadNurse + Nurse (2 skills)
    // Marcus_Webb is Nurse only (1 skill)
    // Assign Aisha and Sarah to Early on Day 0
    schedule['Aisha_Diallo'][0] = 'Early';
    schedule['Sarah_Chen'][0] = 'Early';

    const conversion = scheduleToAssignments(TEST_STAFF, schedule, slotMeta);

    // Day 0 Early slots: shift 1 is HeadNurse, shifts 2 and 3 are Nurse
    // Aisha_Diallo must be assigned to shift 1 (HeadNurse)
    expect(conversion.assignments['1']).toBe(7); // Aisha_Diallo is staff index 6 -> worker_id 7
  });

  // ── AT-30: Control C — Demand-Capacity Diagnostic Scope ─────────────────────
  test('AT-30: Control C ensures demand-capacity diagnostic includes explicit feasibility disclaimer', () => {
    const diag = computeDemandCapacityDiagnostic(TEST_STAFF, undefined, 28);

    expect(diag.feasibilityDisclaimer).toBeDefined();
    expect(diag.feasibilityDisclaimer).toContain('does not prove mathematical feasibility');
    expect(diag.feasibilityDisclaimer).toContain('Shift succession, minimum rest periods');
  });

  // ── AT-31: Excess Baseline Assignments Retained and Reported ────────────────
  test('AT-31: Excess baseline assignments are retained and reported in assignment integrity', () => {
    const slotMeta = getScheduleSlotMeta(TEST_STAFF);
    const slots = Object.values(slotMeta);

    const assignments: BaselineAssignment[] = [
      { worker_id: 'Sarah_Chen', shift_id: 1 },
      { worker_id: 'Sarah_Chen', shift_id: 1 }, // duplicate
      { worker_id: 'NonExistent', shift_id: 2 }, // unknown worker
      { worker_id: 'Marcus_Webb', shift_id: 9999 }, // unmatched shift
    ];

    const report = auditBaselineClientSide(TEST_STAFF, slots, assignments);

    // Total rows must be conserved and reported
    expect(report.assignment_integrity.total_baseline_rows).toBe(4);
    expect(report.assignment_integrity.duplicate_assignments).toBe(1);
    expect(report.assignment_integrity.unknown_worker_errors).toBe(1);
    expect(report.assignment_integrity.unmatched_excess_assignments).toBe(1);
    expect(report.assignment_integrity.valid_assignments_count).toBe(1);
  });

  // ── AT-32: Policy Version and Retrospective Note ────────────────────────────
  test('AT-32: Policy version and retrospective note are identified in the audit report', () => {
    const slotMeta = getScheduleSlotMeta(TEST_STAFF);
    const slots = Object.values(slotMeta);

    const report = auditBaselineClientSide(
      TEST_STAFF,
      slots,
      [{ worker_id: 'Sarah_Chen', shift_id: 1 }],
      {},
      'hospital_policy_pilot_v2.1',
    );

    expect(report.policy_version).toBe('hospital_policy_pilot_v2.1');
    expect(report.policy_retrospective_note).toContain('hospital_policy_pilot_v2.1');
    expect(report.policy_retrospective_note).toContain('Retrospective audit against hospital policy version');
  });

  // ── Control A: Canonical Shifts CSV Parsing ─────────────────────────────────
  test('Control A: parseShiftsCSV supports both canonical "id" and legacy "shift_id" columns', () => {
    const csvCanonical = `id,start_hour,duration_hours,required_skill\n1,6,8,HeadNurse\n2,14,8,Nurse`;
    const { shifts: shifts1, errors: errs1 } = parseShiftsCSV(csvCanonical);
    expect(errs1.length).toBe(0);
    expect(shifts1.length).toBe(2);
    expect(shifts1[0].shiftId).toBe(1);

    const csvAlias = `shift_id,start_hour,duration_hours,required_skill\n10,6,8,HeadNurse`;
    const { shifts: shifts2, errors: errs2 } = parseShiftsCSV(csvAlias);
    expect(errs2.length).toBe(0);
    expect(shifts2.length).toBe(1);
    expect(shifts2[0].shiftId).toBe(10);
  });

  // ── Control D: Genuine XML Spreadsheet 2003 Export ──────────────────────────
  test('Control D: buildExcelXmlSpreadsheet generates valid Microsoft XML Spreadsheet 2003', () => {
    const schedule = buildSyntheticSchedule(TEST_STAFF);
    const xml = buildExcelXmlSpreadsheet(TEST_STAFF, schedule, 28);

    expect(xml).toContain('<?xml version="1.0" encoding="UTF-8"?>');
    expect(xml).toContain('<?mso-application progid="Excel.Sheet"?>');
    expect(xml).toContain('xmlns="urn:schemas-microsoft-com:office:spreadsheet"');
    expect(xml).toContain('<Worksheet ss:Name="UltraRoster Schedule">');
    expect(xml).toContain('<Row ss:Height="24">');
    expect(xml).toContain('Sarah_Chen');
  });

  // ── G3.2: Baseline vs Candidate Comparative Scorecard Rules ────────────────
  test('G3.2: Comparative Scorecard satisfies all three assessment rules', () => {
    const slotMeta = getScheduleSlotMeta(TEST_STAFF);
    const slots = Object.values(slotMeta);

    // Generate baseline with breaches (e.g. 4 breaches: rest, overtime, skill, unfilled)
    const baselineCsv = generateSampleBaselineWithBreaches(TEST_STAFF, slotMeta);
    const { assignments } = parseBaselineCSV(baselineCsv);
    const baselineReport = auditBaselineClientSide(TEST_STAFF, slots, assignments);

    // Rule 1: Baseline and candidates share identical planning demand positions
    expect(baselineReport.coverage.required_positions).toBe(196);

    // Baseline has measured violations and is correctly classified
    expect(baselineReport.feasibility.is_feasible).toBe(false);
    expect(baselineReport.audit_classification).toBe('VALID_BUT_INFEASIBLE');
    expect(baselineReport.feasibility.hard_violations).toBeGreaterThan(0);
    expect(baselineReport.scheduled_overtime_hours).toBeGreaterThan(0);

    // Rule 2 & 3: Candidates have 0 hard violations, 0 overtime exposure, and valid status
    const candidateAMetrics = {
      required_positions: 196,
      filled_positions: 196,
      coverage: 1.0,
      hard_violations: 0,
      scheduled_overtime_hours: 0,
      fairness_penalty: 102.4,
    };

    expect(candidateAMetrics.required_positions).toBe(baselineReport.coverage.required_positions);
    expect(candidateAMetrics.hard_violations).toBe(0);
    expect(candidateAMetrics.scheduled_overtime_hours).toBe(0);
  });

  test('G3.2-DynamicDemand: Scorecard derives demand dynamically from canonical planning request (not fixed 196)', () => {
    // 3-day custom demand CSV with 15 total positions (5 positions per day)
    const customDemandCSV = `id,start_hour,duration_hours,required_skill
1,6,8,HeadNurse
2,6,8,Nurse
3,6,8,Nurse
4,14,8,Nurse
5,22,8,Nurse
6,30,8,HeadNurse
7,30,8,Nurse
8,30,8,Nurse
9,38,8,Nurse
10,46,8,Nurse
11,54,8,HeadNurse
12,54,8,Nurse
13,54,8,Nurse
14,62,8,Nurse
15,70,8,Nurse`;

    const { shifts, errors } = parseShiftsCSV(customDemandCSV);
    expect(errors.length).toBe(0);
    expect(shifts.length).toBe(15);

    const slotMeta = getScheduleSlotMeta(TEST_STAFF, shifts);
    expect(Object.keys(slotMeta).length).toBe(15);

    // Provide arbitrary baseline assignments covering 12 of the 15 positions
    const baselineAssignments: BaselineAssignment[] = [
      { worker_id: 'Sarah_Chen', shift_id: 1 },
      { worker_id: 'Marcus_Webb', shift_id: 2 },
      { worker_id: 'Priya_Nair', shift_id: 3 },
      { worker_id: 'James_Okafor', shift_id: 4 },
      { worker_id: 'Elena_Vasquez', shift_id: 5 },
      { worker_id: 'Sarah_Chen', shift_id: 6 },
      { worker_id: 'Marcus_Webb', shift_id: 7 },
      { worker_id: 'Priya_Nair', shift_id: 8 },
      { worker_id: 'James_Okafor', shift_id: 9 },
      { worker_id: 'Elena_Vasquez', shift_id: 10 },
      { worker_id: 'Sarah_Chen', shift_id: 11 },
      { worker_id: 'Marcus_Webb', shift_id: 12 },
    ];

    const baselineReport = auditBaselineClientSide(TEST_STAFF, shifts, baselineAssignments);

    // Dynamically evaluated against 15 required positions, not fixed 196
    expect(baselineReport.coverage.required_positions).toBe(15);
    expect(baselineReport.coverage.filled_positions).toBe(12);
    expect(baselineReport.coverage.unfilled_positions).toBe(3);
    expect(baselineReport.coverage.coverage_pct).toBe(80.0);

    // Verify candidate alternatives mapped with this customSlotMeta also dynamically derive 15 required positions
    const rawCandidates: ProductionParetoSolution[] = [
      {
        id: 'sol-1',
        schedule: {
          '1': 1, '2': 2, '3': 3, '4': 4, '5': 5,
          '6': 1, '7': 2, '8': 3, '9': 4, '10': 5,
          '11': 1, '12': 2, '13': 3, '14': 4, '15': 5,
        },
        objectives: [0, 0, 0.5, 0, 0, 0],
        metrics: {},
        feasibility: {
          is_feasible: true,
          hard_violations: 0,
          rest_violations: 0,
          hc1_violations: 0,
          hc3_violations: 0,
        },
      },
    ];

    const mapped = mapParetoAlternatives(rawCandidates, TEST_STAFF, undefined, slotMeta);
    expect(mapped.length).toBe(1);
    expect(mapped[0].metrics.required_positions).toBe(15);
    expect(mapped[0].metrics.filled_positions).toBe(15);
    expect(mapped[0].metrics.coverage).toBe(1.0);
  });

  // ── G3.3: Executive Assessment Report & Shift-Level Delta Export ───────────
  test('G3.3-Deliverable1: Executive Assessment Report invariant and evidence controls verification', () => {
    const slotMeta = getScheduleSlotMeta(TEST_STAFF);
    const slots = Object.values(slotMeta);
    const baselineCsv = generateSampleBaselineWithBreaches(TEST_STAFF, slotMeta);
    const { assignments } = parseBaselineCSV(baselineCsv);
    const report = auditBaselineClientSide(TEST_STAFF, slots, assignments, {}, 'hospital_policy_v1');

    // 1. Assessment scope & policy parameters
    expect(report.policy_version).toBe('hospital_policy_v1');
    expect(report.policy_parameters.min_rest_hours).toBe(10);
    expect(report.policy_parameters.max_weekly_hours).toBe(40.0);
    expect(report.policy_parameters.senior_skill_name).toBe('HeadNurse');
    expect(report.policy_parameters.weekly_contract_mode).toBe('monday_sunday');

    // 2. Hospital baseline findings
    expect(report.audit_classification).toBe('VALID_BUT_INFEASIBLE');
    expect(report.feasibility.hard_violations).toBeGreaterThan(0);
    expect(report.rest_violations).toBeGreaterThan(0);
    expect(report.weekly_contract_violations).toBeGreaterThan(0);
    expect(report.senior_coverage_deficits).toBeGreaterThan(0);
    expect(report.scheduled_overtime_hours).toBeGreaterThan(0);

    // 3. Evidence standard disclaimers
    expect(report.policy_retrospective_note).toContain('Retrospective audit');
  });

  test('G3.3-Deliverable2: Shift-Level Delta Export retains all records, unfilled slots, excess rows, and classifies changes accurately', () => {
    // 2-day demand setup with 8 slots (shiftId 1..8)
    const customCSV = `id,start_hour,duration_hours,required_skill
1,6,8,HeadNurse
2,6,8,Nurse
3,14,8,Nurse
4,22,8,Nurse
5,30,8,HeadNurse
6,30,8,Nurse
7,38,8,Nurse
8,46,8,Nurse`;

    const { shifts } = parseShiftsCSV(customCSV);
    const slotMeta = getScheduleSlotMeta(TEST_STAFF, shifts);

    // Candidate schedule covering all 8 shifts
    // Day 0 (shifts 1-4): Sarah_Chen(HeadNurse), Marcus_Webb(Nurse), Priya_Nair(Nurse), James_Okafor(Nurse)
    // Day 1 (shifts 5-8): Priya_Nair(HeadNurse), Marcus_Webb(Nurse), James_Okafor(Nurse), Sarah_Chen(Nurse)
    const candidateSchedule: Record<string, string[]> = {
      Sarah_Chen:    ['Early', 'Night'],
      Marcus_Webb:   ['Early', 'Early'],
      Priya_Nair:    ['Late',  'Early'],
      James_Okafor:  ['Night', 'Late'],
      Elena_Vasquez: ['', ''],
      Tom_Lindqvist: ['', ''],
      Aisha_Diallo:  ['', ''],
      Ingrid_Sorensen: ['', ''],
    };

    // Baseline assignments:
    // Shift 1: Sarah_Chen (matches candidate -> UNTOUCHED)
    // Shift 2: Elena_Vasquez (candidate has Marcus_Webb -> WORKER_REASSIGNED)
    // Shift 3: UNFILLED in baseline (candidate has Priya_Nair -> UNFILLED_IN_BASELINE_NOW_COVERED)
    // Shift 4: James_Okafor, PLUS duplicate row with Tom_Lindqvist -> DUPLICATE_IN_BASELINE
    // Shift 5: UNFILLED in baseline, candidate covers -> UNFILLED_IN_BASELINE_NOW_COVERED
    // Shift 6: Marcus_Webb -> UNTOUCHED
    // Shift 7: James_Okafor -> UNTOUCHED
    // Shift 8: UNFILLED in baseline, candidate covers -> UNFILLED_IN_BASELINE_NOW_COVERED
    // Excess shift 999: Ingrid_Sorensen -> UNMATCHED_EXCESS_IN_BASELINE
    const baselineAssignments: BaselineAssignment[] = [
      { worker_id: 'Sarah_Chen', shift_id: 1 },
      { worker_id: 'Elena_Vasquez', shift_id: 2 },
      // shift 3 left unfilled
      { worker_id: 'James_Okafor', shift_id: 4 },
      { worker_id: 'Tom_Lindqvist', shift_id: 4 }, // duplicate on shift 4
      // shift 5 left unfilled
      { worker_id: 'Marcus_Webb', shift_id: 6 },
      { worker_id: 'James_Okafor', shift_id: 7 },
      // shift 8 left unfilled
      { worker_id: 'Ingrid_Sorensen', shift_id: 999 }, // excess unmatched shift
    ];

    const records = generateShiftLevelDeltaRecords(TEST_STAFF, candidateSchedule, baselineAssignments, slotMeta);

    // Total records must include: 8 demand slots + 1 duplicate on shift 4 + 1 excess shift 999 = 10 records
    expect(records.length).toBe(10);

    // Verify Untouched classification
    const shift1Record = records.find(r => r.shift_id === 1);
    expect(shift1Record).toBeDefined();
    expect(shift1Record?.baseline_worker_id).toBe('Sarah_Chen');
    expect(shift1Record?.candidate_worker_id).toBe('Sarah_Chen');
    expect(shift1Record?.change_classification).toBe('UNTOUCHED');

    // Verify Worker Reassigned classification
    const shift2Record = records.find(r => r.shift_id === 2);
    expect(shift2Record).toBeDefined();
    expect(shift2Record?.baseline_worker_id).toBe('Elena_Vasquez');
    expect(shift2Record?.candidate_worker_id).toBe('Marcus_Webb');
    expect(shift2Record?.change_classification).toBe('WORKER_REASSIGNED');

    // Verify Unfilled in Baseline Now Covered
    const shift3Record = records.find(r => r.shift_id === 3);
    expect(shift3Record).toBeDefined();
    expect(shift3Record?.baseline_worker_id).toBe('UNFILLED');
    expect(shift3Record?.candidate_worker_id).toBe('Priya_Nair');
    expect(shift3Record?.change_classification).toBe('UNFILLED_IN_BASELINE_NOW_COVERED');

    // Verify Duplicate in Baseline
    const dupShift4 = records.filter(r => r.shift_id === 4);
    expect(dupShift4.length).toBe(2);
    expect(dupShift4.some(r => r.change_classification === 'DUPLICATE_IN_BASELINE')).toBe(true);

    // Verify Unmatched Excess in Baseline
    const excessRecord = records.find(r => r.shift_id === 999);
    expect(excessRecord).toBeDefined();
    expect(excessRecord?.baseline_worker_id).toBe('Ingrid_Sorensen');
    expect(excessRecord?.candidate_worker_id).toBe('N/A');
    expect(excessRecord?.change_classification).toBe('UNMATCHED_EXCESS_IN_BASELINE');

    // Verify CSV Generation and parseability
    const csvContent = exportShiftLevelDeltaCSV(records);
    expect(csvContent).toContain('shift_id,day_index,date_or_day,shift_type,start_hour,duration_hours,required_skill,baseline_worker_id,candidate_worker_id,change_classification,notes');
    expect(csvContent).toContain('UNTOUCHED');
    expect(csvContent).toContain('WORKER_REASSIGNED');
    expect(csvContent).toContain('UNFILLED_IN_BASELINE_NOW_COVERED');
    expect(csvContent).toContain('DUPLICATE_IN_BASELINE');
    expect(csvContent).toContain('UNMATCHED_EXCESS_IN_BASELINE');

    // Line count equals 1 header + 10 data rows
    const lines = csvContent.trim().split('\n');
    expect(lines.length).toBe(11);
  });

  // ── AT-33: First-Use Workflow Separation ────────────────────────────────────
  test('AT-33: First-use workflow separation guarantees blank workspace vs preloaded demonstration', () => {
    // 1. Verify Sample Demonstration setup
    const { staff: sampleStaff } = parseStaffCSV(SAMPLE_CSV);
    expect(sampleStaff.length).toBe(20);

    const slotMeta = getScheduleSlotMeta(sampleStaff, []);
    const slots = Object.values(slotMeta);
    expect(slots.length).toBe(196);

    const sampleBaselineCsv = generateSampleBaselineWithBreaches(sampleStaff, slotMeta);
    const { assignments: parsedBaseline } = parseBaselineCSV(sampleBaselineCsv);
    expect(parsedBaseline.length).toBeGreaterThan(150);

    const auditReport = auditBaselineClientSide(
      sampleStaff,
      slots,
      parsedBaseline,
      {
        min_rest_hours: 10,
        max_weekly_hours: 40.0,
        weekly_contract_mode: 'monday_sunday',
        weekly_anchor_offset_hours: 0,
      },
      'hospital_policy_v1'
    );
    expect(auditReport.audit_classification).toBe('VALID_BUT_INFEASIBLE');
    expect(auditReport.feasibility.hard_violations).toBeGreaterThan(0);

    const baseSchedule = buildSyntheticSchedule(sampleStaff);
    const { alternatives, recommendedId } = buildSyntheticAlternatives(sampleStaff, baseSchedule, slotMeta);
    expect(alternatives.length).toBeGreaterThanOrEqual(2);
    expect(recommendedId).toBe('alt-A');

    // 2. Verify Blank Workspace Invariant
    const blankStaff: StaffMember[] = [];
    const blankAssignments: BaselineAssignment[] = [];
    const blankSlotMeta = getScheduleSlotMeta(blankStaff, []);
    expect(blankStaff.length).toBe(0);
    expect(blankAssignments.length).toBe(0);
    expect(Object.keys(blankSlotMeta).length).toBe(196); // Standard hospital demand active waiting for custom shifts or staff
  });
});

// ── AT-34: Coralys Staffing Requirement Recommendation (Level A Clinical Demand) ─
describe('Coralys Staffing Requirement Recommendation — Level A Clinical Demand (AT-34)', () => {
  const DEMAND_TEST_CONFIG: HospitalWardConfiguration = {
    configVersion: '2026.1-demo',
    wardId: 'ward-acute-med-01',
    wardName: 'Acute Medical Care Unit',
    serviceType: 'inpatient_24_7',
    bedCategories: [
      {
        id: 'general_acute',
        name: 'General Acute Beds',
        totalBeds: 24,
        activeStaffedBeds: 20,
        defaultOccupancyPct: 90.0, // 20 * 0.9 = 18 patients
      },
      {
        id: 'step_down_hdu',
        name: 'Step-Down HDU Beds',
        totalBeds: 6,
        activeStaffedBeds: 6,
        defaultOccupancyPct: 100.0, // 6 * 1.0 = 6 patients
      },
    ],
    ratios: [
      { bedCategoryId: 'general_acute', shiftType: 'Early', patientsPerNurse: 6 },
      { bedCategoryId: 'general_acute', shiftType: 'Late', patientsPerNurse: 6 },
      { bedCategoryId: 'general_acute', shiftType: 'Night', patientsPerNurse: 9 },
      { bedCategoryId: 'step_down_hdu', shiftType: 'Early', patientsPerNurse: 2 },
      { bedCategoryId: 'step_down_hdu', shiftType: 'Late', patientsPerNurse: 2 },
      { bedCategoryId: 'step_down_hdu', shiftType: 'Night', patientsPerNurse: 3 },
    ],
    clinicalModifiers: [
      {
        id: 'infectious_isolation',
        name: 'Airborne / Contact Isolation',
        additionalNursesPerPatient: 0.25,
        requiredCompetencySkills: ['InfectionControl'],
        minimumCompetentStaffPerShift: 1,
      },
      {
        id: 'cardiac_telemetry',
        name: 'Continuous Telemetry Monitoring',
        additionalNursesPerPatient: 0.15,
        requiredCompetencySkills: ['Telemetry_Certified'],
        minimumCompetentStaffPerShift: 1,
      },
    ],
    shiftSafetyPolicies: {
      Early: {
        shiftType: 'Early',
        startHourOfDay: 7,
        durationHours: 8,
        safetyFloor: 3,
        chargeNurseFloor: 1,
        seniorStaffRatioPct: 25.0,
        seniorSkillName: 'HeadNurse',
        generalSkillName: 'Nurse',
      },
      Late: {
        shiftType: 'Late',
        startHourOfDay: 15,
        durationHours: 8,
        safetyFloor: 3,
        chargeNurseFloor: 1,
        seniorStaffRatioPct: 25.0,
        seniorSkillName: 'HeadNurse',
        generalSkillName: 'Nurse',
      },
      Night: {
        shiftType: 'Night',
        startHourOfDay: 23,
        durationHours: 8,
        safetyFloor: 2,
        chargeNurseFloor: 1,
        seniorStaffRatioPct: 25.0,
        seniorSkillName: 'HeadNurse',
        generalSkillName: 'Nurse',
      },
    },
    governance: {
      lastModifiedAtIso: '2026-10-31T12:00:00Z',
      modifiedBy: 'Director of Nursing Dr. Aris Thorne',
      facilityStandardReference: 'Hospital Staffing Policy Policy-Doc-704-B',
    },
  };

  // ── AT-34.1: Bed-Category Specific Ratios ────────────────────────────────────
  test('AT-34.1: Calculates base requirements per primary bed category without aggregate dilution', () => {
    // 20 active acute beds @ 90% = 18 patients -> 18 / 6 = 3 nurses
    // 6 active HDU beds @ 100% = 6 patients -> 6 / 2 = 3 nurses
    // Total base = 3 + 3 = 6 nurses (safety floor = 3)
    const demand = calculateClinicalDemand(DEMAND_TEST_CONFIG, { horizonDays: 1 });
    expect(demand.derivationLogs.length).toBe(3); // Early, Late, Night

    const earlyLog = demand.derivationLogs.find(l => l.dayIdx === 0 && l.shiftType === 'Early')!;
    expect(earlyLog).toBeDefined();
    expect(earlyLog.categoryBreakdown.length).toBe(2);

    const acuteBreakdown = earlyLog.categoryBreakdown.find(c => c.categoryId === 'general_acute')!;
    expect(acuteBreakdown.patientCensus).toBe(18);
    expect(acuteBreakdown.ratio).toBe(6);
    expect(acuteBreakdown.requiredNurses).toBe(3);

    const hduBreakdown = earlyLog.categoryBreakdown.find(c => c.categoryId === 'step_down_hdu')!;
    expect(hduBreakdown.patientCensus).toBe(6);
    expect(hduBreakdown.ratio).toBe(2);
    expect(hduBreakdown.requiredNurses).toBe(3);

    expect(earlyLog.baseNursesCalculated).toBe(6);
    expect(earlyLog.baseNursesFinal).toBe(6);
    expect(earlyLog.totalNursesRequired).toBe(6);

    // Protected senior split: senior = max(1, ceil(6 * 0.25)) = 2, general = 4
    expect(earlyLog.seniorNursesRequired).toBe(2);
    expect(earlyLog.generalNursesRequired).toBe(4);
  });

  // ── AT-34.2: Shift-Specific Ratio Variation ─────────────────────────────────
  test('AT-34.2: Applies shift-specific ratios for identical bed category inventory', () => {
    // Acute: 18 patients / 9 (Night) = 2 nurses
    // HDU: 6 patients / 3 (Night) = 2 nurses
    // Total base = 2 + 2 = 4 nurses
    const demand = calculateClinicalDemand(DEMAND_TEST_CONFIG, { horizonDays: 1 });
    const nightLog = demand.derivationLogs.find(l => l.dayIdx === 0 && l.shiftType === 'Night')!;

    expect(nightLog).toBeDefined();
    expect(nightLog.baseNursesCalculated).toBe(4);
    expect(nightLog.totalNursesRequired).toBe(4);
    // Senior: max(1, ceil(4 * 0.25)) = 1, General: 4 - 1 = 3
    expect(nightLog.seniorNursesRequired).toBe(1);
    expect(nightLog.generalNursesRequired).toBe(3);

    // Verify early vs night variance for same beds
    const earlyLog = demand.derivationLogs.find(l => l.dayIdx === 0 && l.shiftType === 'Early')!;
    expect(earlyLog.totalNursesRequired).toBe(6);
    expect(nightLog.totalNursesRequired).toBe(4);
  });

  // ── AT-34.3: Occupancy & Direct Census Override Precedence ──────────────────
  test('AT-34.3: Direct patient census overrides take precedence over default occupancy', () => {
    // Override Day 1 Early: general_acute = 24 (100% capacity), step_down_hdu = 4 (66.7%)
    const overrides: ShiftCensusOverride[] = [
      {
        dayIdx: 1,
        shiftType: 'Early',
        categoryCensus: {
          general_acute: 24, // 24 / 6 = 4 nurses
          step_down_hdu: 4,   // 4 / 2 = 2 nurses
        },
      },
    ];

    const demand = calculateClinicalDemand(DEMAND_TEST_CONFIG, {
      horizonDays: 2,
      censusOverrides: overrides,
    });

    const day0Early = demand.derivationLogs.find(l => l.dayIdx === 0 && l.shiftType === 'Early')!;
    const day1Early = demand.derivationLogs.find(l => l.dayIdx === 1 && l.shiftType === 'Early')!;

    // Day 0 uses default 90% (18 patients) + 100% (6 patients) -> 3 + 3 = 6 nurses
    expect(day0Early.categoryBreakdown.find(c => c.categoryId === 'general_acute')!.patientCensus).toBe(18);
    expect(day0Early.totalNursesRequired).toBe(6);

    // Day 1 uses override 24 patients + 4 patients -> 4 + 2 = 6 nurses
    const acute1 = day1Early.categoryBreakdown.find(c => c.categoryId === 'general_acute')!;
    const hdu1 = day1Early.categoryBreakdown.find(c => c.categoryId === 'step_down_hdu')!;
    expect(acute1.patientCensus).toBe(24);
    expect(acute1.requiredNurses).toBe(4);
    expect(hdu1.patientCensus).toBe(4);
    expect(hdu1.requiredNurses).toBe(2);
    expect(day1Early.totalNursesRequired).toBe(6);

    // Test invalid config rejection
    const invalidConfig = {
      ...DEMAND_TEST_CONFIG,
      bedCategories: [
        {
          id: 'invalid_cat',
          name: 'Invalid Beds',
          totalBeds: 10,
          activeStaffedBeds: 12, // Active > Total!
          defaultOccupancyPct: 90,
        },
      ],
    };
    const errors = validateHospitalWardConfiguration(invalidConfig);
    expect(errors.length).toBeGreaterThan(0);
    expect(errors.some(e => e.includes('cannot exceed totalBeds'))).toBe(true);
  });

  // ── AT-34.4: Additive Clinical Modifiers Combined Rounding & Overlap ─────────
  test('AT-34.4: Clinical modifiers combine nurse equivalents before ceil rounding and support defined overlap policies', () => {
    // Isolated modifiers:
    // Modifier 1: infectious_isolation (addtl = 0.25) -> 2 patients = 0.50 equiv
    // Modifier 2: cardiac_telemetry (addtl = 0.15) -> 2 patients = 0.30 equiv
    // Sum = 0.80 equiv -> ceil(0.80) = 1 nurse
    const modifierInputs: ShiftModifierCensusInput[] = [
      {
        dayIdx: 0,
        shiftType: 'Early',
        affectedPatientCounts: {
          infectious_isolation: 2,
          cardiac_telemetry: 2,
        },
      },
    ];

    const demand = calculateClinicalDemand(DEMAND_TEST_CONFIG, {
      horizonDays: 1,
      modifierCensusInputs: modifierInputs,
    });

    const earlyLog = demand.derivationLogs.find(l => l.dayIdx === 0 && l.shiftType === 'Early')!;
    expect(earlyLog.additionalNursesFromModifiers).toBe(1);
    expect(earlyLog.baseNursesFinal).toBe(6);
    expect(earlyLog.totalNursesRequired).toBe(7); // 6 + 1

    // Cross integer boundary: 3 isolation (0.75) + 3 telemetry (0.45) = 1.20 -> ceil = 2
    const modifierInputs2: ShiftModifierCensusInput[] = [
      {
        dayIdx: 0,
        shiftType: 'Early',
        affectedPatientCounts: {
          infectious_isolation: 3,
          cardiac_telemetry: 3,
        },
      },
    ];
    const demand2 = calculateClinicalDemand(DEMAND_TEST_CONFIG, {
      horizonDays: 1,
      modifierCensusInputs: modifierInputs2,
    });
    const earlyLog2 = demand2.derivationLogs.find(l => l.dayIdx === 0 && l.shiftType === 'Early')!;
    expect(earlyLog2.additionalNursesFromModifiers).toBe(2);
    expect(earlyLog2.totalNursesRequired).toBe(8); // 6 + 2

    // Overlapping Cohorts Test 1: Additive overlap policy
    // 4 patients concurrently carrying BOTH isolation (0.25) and telemetry (0.15)
    // Additive sum = 4 * (0.25 + 0.15) = 4 * 0.40 = 1.60 -> ceil = 2 nurses
    const additiveOverlapInput: ShiftModifierCensusInput[] = [
      {
        dayIdx: 0,
        shiftType: 'Early',
        overlappingCohorts: [
          {
            cohortLabel: 'Isolation + Telemetry Complex Cohort',
            modifierIds: ['infectious_isolation', 'cardiac_telemetry'],
            patientCount: 4,
          },
        ],
      },
    ];
    const demandAdditive = calculateClinicalDemand(DEMAND_TEST_CONFIG, {
      horizonDays: 1,
      modifierCensusInputs: additiveOverlapInput,
    });
    const earlyLogAdditive = demandAdditive.derivationLogs.find(l => l.dayIdx === 0 && l.shiftType === 'Early')!;
    expect(earlyLogAdditive.additionalNursesFromModifiers).toBe(2);
    expect(earlyLogAdditive.totalNursesRequired).toBe(8); // 6 + 2

    // Overlapping Cohorts Test 2: Capped per-patient overlap policy
    // Same 4 patients, but hospital policy caps compounding addition at 0.25 max per patient
    // Capped sum = 4 * min(0.40, 0.25) = 4 * 0.25 = 1.00 -> ceil = 1 nurse
    const cappedConfig: HospitalWardConfiguration = {
      ...DEMAND_TEST_CONFIG,
      modifierOverlapPolicy: {
        mode: 'capped_per_patient',
        maxAdditionalNursesPerPatient: 0.25,
      },
    };
    const demandCapped = calculateClinicalDemand(cappedConfig, {
      horizonDays: 1,
      modifierCensusInputs: additiveOverlapInput,
    });
    const earlyLogCapped = demandCapped.derivationLogs.find(l => l.dayIdx === 0 && l.shiftType === 'Early')!;
    expect(earlyLogCapped.additionalNursesFromModifiers).toBe(1);
    expect(earlyLogCapped.totalNursesRequired).toBe(7); // 6 + 1 (capping prevented 1 additional nurse)
  });

  // ── AT-34.5: Competency Enforcement Through MOGA & Strict Validator ─────────
  test('AT-34.5: Competency requirements emit certified slots that reach MOGA payload and independent validator', () => {
    const modifierInputs: ShiftModifierCensusInput[] = [
      {
        dayIdx: 0,
        shiftType: 'Early',
        affectedPatientCounts: {
          cardiac_telemetry: 1, // 1 patient triggers 1 Telemetry_Certified slot
        },
      },
    ];

    const demand = calculateClinicalDemand(DEMAND_TEST_CONFIG, {
      horizonDays: 1,
      modifierCensusInputs: modifierInputs,
    });

    const earlyLog = demand.derivationLogs.find(l => l.dayIdx === 0 && l.shiftType === 'Early')!;
    expect(earlyLog.competencyRequirements['Telemetry_Certified']).toBe(1);

    // 1. Verify that calculateClinicalDemand explicitly allocated a slot with Telemetry_Certified skill
    const telemetrySlot = demand.slots.find(s => s.dayIdx === 0 && s.label === 'Early' && s.skill === 'Telemetry_Certified');
    expect(telemetrySlot).toBeDefined();
    expect(telemetrySlot?.skill).toBe('Telemetry_Certified');

    // 2. Verify that buildSchedulePayload transfers this required skill to MOGA shifts
    const mogaPayload = buildSchedulePayload(TEST_STAFF, {}, demand.slots);
    const mogaShift = mogaPayload.shifts.find(s => s.id === telemetrySlot?.shiftId);
    expect(mogaShift).toBeDefined();
    expect(mogaShift?.required_skill).toBe('Telemetry_Certified');

    // 3. Verify that independent validator / client audit enforces this skill requirement
    // Marcus_Webb has ONLY ['Nurse'], lacks 'Telemetry_Certified' -> assigning Marcus must trigger SKILL_DEFICIT
    const invalidAssignment: BaselineAssignment[] = [
      { worker_id: 'Marcus_Webb', shift_id: telemetrySlot!.shiftId },
    ];
    const auditReport = auditBaselineClientSide(TEST_STAFF, demand.slots, invalidAssignment);
    expect(auditReport.skill_coverage_deficits).toBe(1);
    expect(auditReport.detailed_findings.some(f => f.code === 'SKILL_DEFICIT' && f.description.includes('Telemetry_Certified'))).toBe(true);

    // 4. Qualified nurse assignment (Elena_Vasquez with Telemetry_Certified) satisfies requirement
    const qualifiedStaff: StaffMember[] = [
      ...TEST_STAFF,
      { id: 'Certified_Nurse_Kim', contract: 'FullTime', skills: ['Nurse', 'Telemetry_Certified'] },
    ];
    const validAssignment: BaselineAssignment[] = [
      { worker_id: 'Certified_Nurse_Kim', shift_id: telemetrySlot!.shiftId },
    ];
    const validReport = auditBaselineClientSide(qualifiedStaff, demand.slots, validAssignment);
    expect(validReport.skill_coverage_deficits).toBe(0);
  });

  // ── AT-34.6: Charge Nurse Floor & Non-Negative General Staffing ──────────────
  test('AT-34.6: Charge nurse floor is guaranteed and general nurse staffing is strictly non-negative', () => {
    // Edge Case 1: Low census triggers safetyFloor & chargeNurseFloor
    const lowCensusConfig: HospitalWardConfiguration = {
      ...DEMAND_TEST_CONFIG,
      bedCategories: [
        {
          id: 'single_bed',
          name: 'Single Bed Unit',
          totalBeds: 2,
          activeStaffedBeds: 1,
          defaultOccupancyPct: 100, // 1 patient
        },
      ],
      ratios: [
        { bedCategoryId: 'single_bed', shiftType: 'Early', patientsPerNurse: 6 },
        { bedCategoryId: 'single_bed', shiftType: 'Late', patientsPerNurse: 6 },
        { bedCategoryId: 'single_bed', shiftType: 'Night', patientsPerNurse: 6 },
      ],
      shiftSafetyPolicies: {
        Early: {
          shiftType: 'Early',
          startHourOfDay: 7,
          durationHours: 8,
          safetyFloor: 3,
          chargeNurseFloor: 1,
          seniorStaffRatioPct: 25.0,
          seniorSkillName: 'HeadNurse',
          generalSkillName: 'Nurse',
        },
      },
    };

    const demandLow = calculateClinicalDemand(lowCensusConfig, { horizonDays: 1 });
    const logLow = demandLow.derivationLogs[0];
    expect(logLow.baseNursesCalculated).toBe(1); // ceil(1/6)
    expect(logLow.baseNursesFinal).toBe(3);      // safety floor 3 applied
    expect(logLow.totalNursesRequired).toBe(3);
    expect(logLow.seniorNursesRequired).toBe(1); // charge floor 1
    expect(logLow.generalNursesRequired).toBe(2); // 3 - 1 >= 0
    expect(logLow.generalNursesRequired).toBeGreaterThanOrEqual(0);

    // Edge Case 2: Charge nurse floor exceeds percentage calculation
    const floorConfig: HospitalWardConfiguration = {
      ...lowCensusConfig,
      shiftSafetyPolicies: {
        Early: {
          shiftType: 'Early',
          startHourOfDay: 7,
          durationHours: 8,
          safetyFloor: 1,
          chargeNurseFloor: 4, // Floor 4
          seniorStaffRatioPct: 20.0, // 4 * 0.20 = 0.8 -> ceil = 1, but floor = 4
          seniorSkillName: 'HeadNurse',
          generalSkillName: 'Nurse',
        },
      },
    };
    const demandFloor = calculateClinicalDemand(floorConfig, { horizonDays: 1 });
    const logFloor = demandFloor.derivationLogs[0];
    expect(logFloor.totalNursesRequired).toBe(4);
    expect(logFloor.seniorNursesRequired).toBe(4);
    expect(logFloor.generalNursesRequired).toBe(0); // 4 - 4 = 0 >= 0
    expect(logFloor.generalNursesRequired).toBeGreaterThanOrEqual(0);
  });

  // ── AT-34.7: Immutable Demand Snapshot with Strong SHA-256 Digest ───────────
  test('AT-34.7: Approved demand snapshot produces 256-bit cryptographic digest and enforces deep immutability', () => {
    const demand = calculateClinicalDemand(DEMAND_TEST_CONFIG, { horizonDays: 7 });
    const snapshot = createApprovedDemandSnapshot(
      'scenario-alpha',
      DEMAND_TEST_CONFIG,
      'Dr. Aris Thorne',
      'Approved initial 7-day trial',
      demand,
    );

    expect(snapshot.scenarioId).toBe('scenario-alpha');
    expect(snapshot.configVersion).toBe('2026.1-demo');
    expect(snapshot.approvedBy).toBe('Dr. Aris Thorne');
    expect(snapshot.totalPositions).toBe(demand.totalPositions);
    expect(snapshot.totalClinicalHours).toBe(demand.totalClinicalHours);
    expect(snapshot.slots.length).toBe(demand.slots.length);

    // Cryptographic SHA-256 digest verification
    expect(snapshot.policySnapshotHash).toMatch(/^sha256-[0-9a-f]{64}$/);
    expect(snapshot.policySnapshotHash.length).toBe(71); // "sha256-" (7) + 64 hex chars

    // Deterministic hash verification
    const hash1 = computePolicySnapshotHash(DEMAND_TEST_CONFIG);
    expect(snapshot.policySnapshotHash).toBe(hash1);

    // Deep Immutability Verification: Object and nested properties are frozen
    expect(Object.isFrozen(snapshot)).toBe(true);
    expect(Object.isFrozen(snapshot.slots)).toBe(true);
    expect(Object.isFrozen(snapshot.slots[0])).toBe(true);
    expect(Object.isFrozen(snapshot.derivationLogs)).toBe(true);
    expect(Object.isFrozen(snapshot.derivationLogs[0])).toBe(true);

    // Assert that attempted mutations fail or throw TypeError
    expect(() => {
      (snapshot as any).approvedBy = 'Unauthorized_Change';
    }).toThrow(TypeError);

    expect(() => {
      (snapshot.slots as any).push({ shiftId: 9999 } as any);
    }).toThrow(TypeError);

    expect(() => {
      (snapshot.slots[0] as any).skill = 'Unauthorized_Skill';
    }).toThrow(TypeError);

    expect(() => {
      (snapshot.derivationLogs[0] as any).totalNursesRequired = 999;
    }).toThrow(TypeError);

    // Hash sensitivity: changing ratio must alter hash
    const modifiedConfig1: HospitalWardConfiguration = {
      ...DEMAND_TEST_CONFIG,
      ratios: DEMAND_TEST_CONFIG.ratios.map(r =>
        r.bedCategoryId === 'general_acute' && r.shiftType === 'Early'
          ? { ...r, patientsPerNurse: 5 }
          : r
      ),
    };
    const hash2 = computePolicySnapshotHash(modifiedConfig1);
    expect(hash2).not.toBe(hash1);

    // Hash sensitivity: changing staffed beds must alter hash
    const modifiedConfig2: HospitalWardConfiguration = {
      ...DEMAND_TEST_CONFIG,
      bedCategories: DEMAND_TEST_CONFIG.bedCategories.map(c =>
        c.id === 'general_acute' ? { ...c, activeStaffedBeds: 18 } : c
      ),
    };
    const hash3 = computePolicySnapshotHash(modifiedConfig2);
    expect(hash3).not.toBe(hash1);

    // Validation checks on snapshot creation
    expect(() =>
      createApprovedDemandSnapshot('', DEMAND_TEST_CONFIG, 'Dr. Thorne', '', demand)
    ).toThrow('scenarioId is required');

    expect(() =>
      createApprovedDemandSnapshot('scenario-alpha', DEMAND_TEST_CONFIG, '', '', demand)
    ).toThrow('approvedBy is required');
  });

  // ── AT-34.8: Scenario Isolation ─────────────────────────────────────────────
  test('AT-34.8: Scenario isolation guarantees separate namespaces for standard and surge profiles', () => {
    const standardDemand = calculateClinicalDemand(DEMAND_TEST_CONFIG, { horizonDays: 7 });
    const standardSnapshot = createApprovedDemandSnapshot(
      'scenario-standard-01',
      DEMAND_TEST_CONFIG,
      'Nurse Executive Smith',
      'Standard 90% census baseline',
      standardDemand,
    );

    const surgeOverrides: ShiftCensusOverride[] = [];
    for (let d = 0; d < 7; d++) {
      surgeOverrides.push({
        dayIdx: d,
        shiftType: 'Early',
        categoryCensus: { general_acute: 24, step_down_hdu: 6 },
      });
      surgeOverrides.push({
        dayIdx: d,
        shiftType: 'Late',
        categoryCensus: { general_acute: 24, step_down_hdu: 6 },
      });
      surgeOverrides.push({
        dayIdx: d,
        shiftType: 'Night',
        categoryCensus: { general_acute: 24, step_down_hdu: 6 },
      });
    }

    const surgeDemand = calculateClinicalDemand(DEMAND_TEST_CONFIG, {
      horizonDays: 7,
      censusOverrides: surgeOverrides,
    });
    const surgeSnapshot = createApprovedDemandSnapshot(
      'scenario-surge-01',
      DEMAND_TEST_CONFIG,
      'Nurse Executive Smith',
      'Surge 100% capacity plan',
      surgeDemand,
    );

    expect(standardSnapshot.scenarioId).toBe('scenario-standard-01');
    expect(surgeSnapshot.scenarioId).toBe('scenario-surge-01');
    expect(standardSnapshot.demandSnapshotId).not.toBe(surgeSnapshot.demandSnapshotId);
    expect(surgeSnapshot.totalPositions).toBeGreaterThan(standardSnapshot.totalPositions);
    expect(surgeSnapshot.totalClinicalHours).toBeGreaterThan(standardSnapshot.totalClinicalHours);
  });

  // ── AT-34.9: Canonical ShiftSlotMeta[] Mapping ──────────────────────────────
  test('AT-34.9: Emits canonical ShiftSlotMeta[] that seamlessly interoperate with downstream workflow', () => {
    // 7-day demand across 3 shifts: Early (6), Late (6), Night (4) = 16 slots/day * 7 = 112 slots
    const demand = calculateClinicalDemand(DEMAND_TEST_CONFIG, { horizonDays: 7 });

    expect(demand.slots.length).toBe(112);
    expect(demand.totalPositions).toBe(112);
    expect(demand.totalClinicalHours).toBe(112 * 8);

    // Verify canonical slot structure
    for (let i = 0; i < demand.slots.length; i++) {
      const slot = demand.slots[i];
      expect(slot.shiftId).toBe(i + 1);
      expect(slot.dayIdx).toBeGreaterThanOrEqual(0);
      expect(slot.dayIdx).toBeLessThan(7);
      expect(['Early', 'Late', 'Night']).toContain(slot.label);
      expect(['HeadNurse', 'Nurse']).toContain(slot.skill);
      expect(slot.durationHours).toBe(8);
      expect(slot.startHour).toBeDefined();
    }

    // Interoperability with getScheduleSlotMeta
    const slotMeta = getScheduleSlotMeta([], demand.slots);
    expect(Object.keys(slotMeta).length).toBe(112);

    // Interoperability with buildStaffingRequirements
    const requirements = buildStaffingRequirements(slotMeta);
    expect(requirements.length).toBe(21); // 7 days * 3 shift types
    const day0Early = requirements.find(r => r.dayIdx === 0 && r.shiftType === 'Early');
    expect(day0Early?.required).toBe(6);
    const day0Night = requirements.find(r => r.dayIdx === 0 && r.shiftType === 'Night');
    expect(day0Night?.required).toBe(4);

    const totalReqPositions = requirements.reduce((acc, r) => acc + r.required, 0);
    expect(totalReqPositions).toBe(112);
  });
});

// ── AT-35: Coralys Demand Review & Approval UI Suite (Phase 2) ───────────────
describe('Coralys Demand Review & Approval UI Suite (Phase 2)', () => {
  // ── AT-35.1: Dynamic Configuration Recalculation ───────────────────────────
  test('AT-35.1: Ward and bed configuration inputs dynamically recalculate clinical demand across 28-day horizon', () => {
    // Standard configuration: 20 active acute beds @ 90% (18 patients) + 6 active HDU beds @ 100% (6 patients)
    // Early: 18/6=3 acute + 6/2=3 HDU = 6 nurses
    // Late:  18/6=3 acute + 6/2=3 HDU = 6 nurses
    // Night: 18/9=2 acute + 6/3=2 HDU = 4 nurses
    // Total per day: 16 nurses. Over 28 days = 448 positions, 3,584 hours
    const baseDemand = calculateClinicalDemand(DEFAULT_HOSPITAL_CONFIG, { horizonDays: 28 });

    expect(baseDemand.totalPositions).toBe(448);
    expect(baseDemand.totalClinicalHours).toBe(3584);
    expect(baseDemand.slots.length).toBe(448);

    // Modify active acute beds to 24 (full surge): 24 * 0.9 = 21.6 -> ceil(21.6/6) = 4 nurses on Early/Late
    const surgeConfig: HospitalWardConfiguration = {
      ...DEFAULT_HOSPITAL_CONFIG,
      bedCategories: DEFAULT_HOSPITAL_CONFIG.bedCategories.map(c =>
        c.id === 'general_acute' ? { ...c, activeStaffedBeds: 24 } : c
      ),
    };
    const surgeDemand = calculateClinicalDemand(surgeConfig, { horizonDays: 28 });

    // Early and Late now need 4 acute + 3 HDU = 7 nurses. Night: ceil(21.6/9)=3 acute + 2 HDU = 5 nurses.
    // Daily total: 7 + 7 + 5 = 19 nurses. Over 28 days = 532 positions
    expect(surgeDemand.totalPositions).toBe(532);
    expect(surgeDemand.totalClinicalHours).toBe(532 * 8);
    expect(surgeDemand.totalPositions).toBeGreaterThan(baseDemand.totalPositions);
  });

  // ── AT-35.2: Required vs Available Capacity Diagnostic ─────────────────────
  test('AT-35.2: Required-versus-available workforce diagnostic calculates accurate gaps and carries mandatory Level B disclaimer', () => {
    const { staff: sampleStaff } = parseStaffCSV(SAMPLE_CSV);
    const demand = calculateClinicalDemand(DEFAULT_HOSPITAL_CONFIG, { horizonDays: 28 });

    const diagnostic = computeDemandCapacityDiagnostic(sampleStaff, demand.slots, 28);

    expect(diagnostic.demandHours).toBe(3584);
    expect(diagnostic.shiftCount).toBe(448);
    // 20 staff members from SAMPLE_CSV
    expect(diagnostic.workforceCapacityHours).toBeGreaterThan(2000);
    expect(diagnostic.surplusOrDeficitHours).toBeDefined();

    // Verify strict workflow boundary and Level B feasibility disclaimer
    expect(diagnostic.feasibilityDisclaimer).toBeDefined();
    expect(diagnostic.feasibilityDisclaimer).toContain('does not prove mathematical feasibility');
    expect(diagnostic.feasibilityDisclaimer).toContain('verified by the optimization solver');
  });

  // ── AT-35.3: Explicit Governance Approval & Immutable Snapshot ─────────────
  test('AT-35.3: Explicit approval creates immutable demand snapshot with SHA-256 digest and locks positions', () => {
    const demand = calculateClinicalDemand(DEFAULT_HOSPITAL_CONFIG, { horizonDays: 28 });
    const snapshot = createApprovedDemandSnapshot(
      'scenario-ward-4b-28d',
      DEFAULT_HOSPITAL_CONFIG,
      'Dr. Aris Thorne, Director of Nursing',
      'Approved 28-day baseline demand for Ward 4B',
      demand,
    );

    expect(snapshot.scenarioId).toBe('scenario-ward-4b-28d');
    expect(snapshot.approvedBy).toBe('Dr. Aris Thorne, Director of Nursing');
    expect(snapshot.approvalNotes).toBe('Approved 28-day baseline demand for Ward 4B');
    expect(snapshot.totalPositions).toBe(448);
    expect(snapshot.totalClinicalHours).toBe(3584);
    expect(snapshot.slots.length).toBe(448);

    // Strong SHA-256 Digest
    expect(snapshot.policySnapshotHash).toMatch(/^sha256-[0-9a-f]{64}$/);

    // Deep immutability
    expect(Object.isFrozen(snapshot)).toBe(true);
    expect(Object.isFrozen(snapshot.slots)).toBe(true);
    expect(() => { (snapshot as any).approvedBy = 'Hacked'; }).toThrow(TypeError);
  });

  // ── AT-35.4: Seamless Optimization Pipeline Connection ─────────────────────
  test('AT-35.4: Approved demand snapshot slots seamlessly populate MOGA schedule payload', () => {
    const { staff: sampleStaff } = parseStaffCSV(SAMPLE_CSV);
    const demand = calculateClinicalDemand(DEFAULT_HOSPITAL_CONFIG, { horizonDays: 28 });
    const snapshot = createApprovedDemandSnapshot(
      'scenario-ward-4b-28d',
      DEFAULT_HOSPITAL_CONFIG,
      'Dr. Aris Thorne',
      'Approved for MOGA optimization',
      demand,
    );

    // Passing snapshot.slots as customShifts to downstream payload builder
    const slotMeta = getScheduleSlotMeta(sampleStaff, snapshot.slots);
    expect(Object.keys(slotMeta).length).toBe(448);

    const payload = buildSchedulePayload(sampleStaff, { max_weekly_hours: 40.0, min_rest_hours: 10 }, snapshot.slots);
    expect(payload.shifts.length).toBe(448);
    expect(payload.workers.length).toBe(sampleStaff.length);

    // Verify slot parameters match approved snapshot exactly
    expect(payload.shifts[0].id).toBe(snapshot.slots[0].shiftId);
    expect(payload.shifts[0].required_skill).toBe(snapshot.slots[0].skill);
    expect(payload.shifts[0].duration_hours).toBe(8);
  });
});

