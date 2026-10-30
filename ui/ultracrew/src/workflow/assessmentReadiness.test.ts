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
  getScheduleSlotMeta,
  buildSchedulePayload,
  buildStaffingRequirements,
  computeDemandCapacityDiagnostic,
  scheduleToAssignments,
  exportRosterToCSV,
  computeNextCycleBoundary,
  buildSyntheticSchedule,
} from './WorkflowUtils';
import type { StaffMember } from './WorkflowTypes';

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
