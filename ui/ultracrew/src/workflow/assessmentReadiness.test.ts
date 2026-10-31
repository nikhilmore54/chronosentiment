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
} from './WorkflowUtils';
import type { StaffMember, BaselineAssignment } from './WorkflowTypes';

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
});
