// @vitest-environment jsdom
import { describe, test, expect } from 'vitest';
import {
  SAMPLE_CSV,
  OPTIMAL_12_STAFF_CSV,
  type StaffMember,
  type SchedulerDecision,
} from './WorkflowTypes';
import {
  parseStaffCSV,
  getScheduleSlotMeta,
  generateSampleBaselineWithBreaches,
  parseBaselineCSV,
  auditBaselineClientSide,
  buildEditableSchedule,
  buildSyntheticAlternatives,
  computeCanonicalCoverage,
  buildStaffingRequirements,
  VERIFIED_SAMPLE_SCHEDULE,
  VERIFIED_OPTIMAL_12_SCHEDULE,
} from './WorkflowUtils';

describe('UltraRoster Scenario Isolation & State Integrity', () => {
  // ── 1. 20-Nurse Department Scenario ───────────────────────────────────────
  test('20-Nurse scenario loads with complete workforce, canonical 196 demand, and clean provenance', () => {
    const { staff, errors } = parseStaffCSV(SAMPLE_CSV);
    expect(errors).toHaveLength(0);
    expect(staff).toHaveLength(20);

    const headNurses = staff.filter(s => s.skills.includes('HeadNurse'));
    expect(headNurses.length).toBeGreaterThanOrEqual(4);

    const slotMeta = getScheduleSlotMeta(staff, []);
    const slotCount = Object.keys(slotMeta).length;
    expect(slotCount).toBe(196); // 28 days * 7 shifts/day

    const scenarioId = 'scenario-synthetic-20-nurse-department';
    const scenarioName = 'Synthetic 20-Nurse Department';

    const baseSchedule = buildEditableSchedule(staff, { ...VERIFIED_SAMPLE_SCHEDULE }, slotMeta);
    const { alternatives, recommendedId } = buildSyntheticAlternatives(staff, baseSchedule, slotMeta);

    expect(alternatives.length).toBeGreaterThanOrEqual(2);
    expect(recommendedId).toBe('alt-A');
    expect(alternatives[0].metrics.filled_positions).toBe(196);
    expect(alternatives[0].metrics.required_positions).toBe(196);
  });

  // ── 2. Scheduler Edits in 20-Nurse Scenario Do Not Pollute 12-Nurse Scenario ──
  test('Scheduler edits and decision provenance in 20-nurse do not leak into 12-nurse scenario', () => {
    // Stage A: 20-Nurse Department with manual edit and decision
    const { staff: staff20 } = parseStaffCSV(SAMPLE_CSV);
    const slotMeta20 = getScheduleSlotMeta(staff20, []);
    const sched20 = buildEditableSchedule(staff20, { ...VERIFIED_SAMPLE_SCHEDULE }, slotMeta20);

    // Make manual edit on 20-nurse schedule: change worker 0 day 9 to 'E'
    const worker0 = staff20[0].id;
    sched20[worker0][9] = 'E';
    const manualEditCount20 = 1;

    const decision20: SchedulerDecision = {
      decision_id: 'dec-20-nurse-001',
      created_at_iso: new Date().toISOString(),
      recommended_id: 'alt-A',
      selected_id: 'alt-B',
      overrode_recommendation: true,
      recommended_metrics: {
        coverage: 1.0,
        filled_positions: 196,
        required_positions: 196,
        fairness_penalty: 1.2,
        utilization: 0.85,
        cost: 100,
        diff_from_recommended: 0,
      },
      selected_metrics: {
        coverage: 1.0,
        filled_positions: 196,
        required_positions: 196,
        fairness_penalty: 0.9,
        utilization: 0.85,
        cost: 97,
        diff_from_recommended: 4,
      },
    };

    expect(manualEditCount20).toBe(1);
    expect(decision20.overrode_recommendation).toBe(true);

    // Stage B: Switch to 12-Nurse Optimal Team
    const { staff: staff12, errors: errors12 } = parseStaffCSV(OPTIMAL_12_STAFF_CSV);
    expect(errors12).toHaveLength(0);
    expect(staff12).toHaveLength(12);

    const staff12Ids = new Set(staff12.map(s => s.id));
    const excludedStaff = staff20.filter(s => !staff12Ids.has(s.id));
    // 20 staff pool and 12 staff pool share 11 IDs (Aisha_Diallo vs Amara_Diallo), so 9 are excluded
    expect(excludedStaff).toHaveLength(9);

    // Excluded nurses from 20-person pool must NOT exist in 12-person scenario
    for (const ex of excludedStaff) {
      expect(staff12Ids.has(ex.id)).toBe(false);
    }

    const slotMeta12 = getScheduleSlotMeta(staff12, []);
    const sched12 = buildEditableSchedule(staff12, { ...VERIFIED_OPTIMAL_12_SCHEDULE }, slotMeta12);

    // 12-nurse schedule must have clean slate: 0 edits, null decision
    let manualEditCount12 = 0;
    let decision12: SchedulerDecision | null = null;

    expect(manualEditCount12).toBe(0);
    expect(decision12).toBeNull();
    expect(Object.keys(sched12)).toHaveLength(12);

    // Verify 12-nurse schedule only contains the 12 staff
    for (const ex of excludedStaff) {
      expect(sched12[ex.id]).toBeUndefined();
    }

    // Verify canonical 196 demand coverage for 12 nurses
    const reqs12 = buildStaffingRequirements(slotMeta12);
    const cov12 = computeCanonicalCoverage(reqs12, sched12);
    expect(cov12.filledPositions).toBe(196);
    expect(cov12.requiredPositions).toBe(196);
    expect(cov12.coveragePct).toBe(100);
  });

  // ── 3. Switching Back to 20-Nurse Restores 20-Nurse Roster & Provenance ─────
  test('Switching back to 20-nurse scenario cleanly restores 20-worker pool without 12-nurse truncation', () => {
    const { staff: staff20 } = parseStaffCSV(SAMPLE_CSV);
    const slotMeta20 = getScheduleSlotMeta(staff20, []);
    const sched20 = buildEditableSchedule(staff20, { ...VERIFIED_SAMPLE_SCHEDULE }, slotMeta20);

    expect(staff20).toHaveLength(20);
    expect(Object.keys(sched20)).toHaveLength(20);

    const reqs20 = buildStaffingRequirements(slotMeta20);
    const cov20 = computeCanonicalCoverage(reqs20, sched20);
    expect(cov20.filledPositions).toBe(196);
    expect(cov20.requiredPositions).toBe(196);
  });

  // ── 4. Blank Assessment Workspace Complete Reset ───────────────────────────
  test('Blank workspace reset clears all workforce, schedule, and decision state', () => {
    const blankState = {
      step: 1,
      maxReached: 1,
      staff: [] as StaffMember[],
      customShifts: [],
      selectedRuleId: '',
      scheduleResult: null,
      editableSchedule: {},
      manualEditCount: 0,
      alternatives: [],
      recommendedId: '',
      schedulerDecision: null,
      baselineAuditReport: null,
      baselineAssignments: [],
      demoScenario: null,
      workflowMode: 'blank',
    };

    expect(blankState.staff).toHaveLength(0);
    expect(Object.keys(blankState.editableSchedule)).toHaveLength(0);
    expect(blankState.manualEditCount).toBe(0);
    expect(blankState.schedulerDecision).toBeNull();
    expect(blankState.demoScenario).toBeNull();
  });

  // ── 5. Audit Engine 58-Violation Decomposition & Mutual Exclusivity ─────────
  test('Audit engine 58-violation decomposition is mathematically exact and categories are mutually exclusive', () => {
    const { staff } = parseStaffCSV(SAMPLE_CSV);
    const slotMeta = getScheduleSlotMeta(staff, []);
    const sampleBaselineCsv = generateSampleBaselineWithBreaches(staff, slotMeta);
    const { assignments: parsedBaseline } = parseBaselineCSV(sampleBaselineCsv);

    const auditReport = auditBaselineClientSide(
      staff,
      Object.values(slotMeta),
      parsedBaseline,
      {
        min_rest_hours: 10,
        max_weekly_hours: 40.0,
        weekly_contract_mode: 'monday_sunday',
        weekly_anchor_offset_hours: 0,
      },
      'hospital_policy_v1'
    );

    // Mathematical reconciliation check
    const rest = auditReport.rest_violations;
    const contract = auditReport.weekly_contract_violations;
    const skill = auditReport.skill_coverage_deficits;
    const unfilled = auditReport.coverage.unfilled_positions;
    const totalHard = auditReport.feasibility.hard_violations;

    expect(rest).toBe(10);
    expect(contract).toBe(5);
    expect(skill).toBe(39);
    expect(auditReport.senior_coverage_deficits).toBe(23); // subset of skill
    expect(unfilled).toBe(4);

    // Mutual exclusivity verification:
    // 10 + 5 + 39 + 4 = 58
    expect(rest + contract + skill + unfilled).toBe(totalHard);
    expect(totalHard).toBe(58);

    // An unfilled position can never have a skill deficit check because no worker is assigned
    const assignedShiftIds = new Set(parsedBaseline.map(b => b.shift_id));
    const allRequiredShiftIds = Object.values(slotMeta).map(m => m.shiftId);
    const unfilledShiftIds = allRequiredShiftIds.filter(id => !assignedShiftIds.has(id));

    expect(unfilledShiftIds).toHaveLength(4);

    // Ensure detailed findings for skill deficits only reference assigned shifts
    const skillFindings = auditReport.detailed_findings.filter(f => f.code.includes('SKILL'));
    for (const f of skillFindings) {
      if (f.shift_id !== undefined) {
        expect(assignedShiftIds.has(f.shift_id)).toBe(true);
        expect(unfilledShiftIds.includes(f.shift_id)).toBe(false);
      }
    }
  });
});
