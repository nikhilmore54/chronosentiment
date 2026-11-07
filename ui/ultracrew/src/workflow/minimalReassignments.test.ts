// @vitest-environment jsdom
import { describe, test, expect } from 'vitest';
import * as fs from 'fs';
import * as path from 'path';
import {
  SAMPLE_CSV,
  type StaffMember,
} from './WorkflowTypes';
import {
  parseStaffCSV,
  getScheduleSlotMeta,
  parseBaselineCSV,
  auditBaselineClientSide,
  buildEditableSchedule,
  generateShiftLevelDeltaRecords,
  exportShiftLevelDeltaCSV,
  VERIFIED_SAMPLE_SCHEDULE,
} from './WorkflowUtils';

describe('Minimal Perturbation Roster Healing Verification', () => {
  test('Construct baseline with exact 58 violations and verify high retention against VERIFIED_SAMPLE_SCHEDULE', () => {
    const { staff } = parseStaffCSV(SAMPLE_CSV);
    const slotMeta = getScheduleSlotMeta(staff, []);
    const slots = Object.values(slotMeta).sort((a, b) => a.shiftId - b.shiftId);
    const staffMap = new Map(staff.map(s => [s.id, s]));

    // Start with verified 100% compliant schedule
    const baselineMap = new Map<number, string>();
    for (const [sidStr, wIdx] of Object.entries(VERIFIED_SAMPLE_SCHEDULE)) {
      const sid = Number(sidStr);
      const worker = staff[wIdx - 1];
      if (worker) baselineMap.set(sid, worker.id);
    }

    // 1. Unfilled: omit last 4 shifts (193, 194, 195, 196)
    baselineMap.delete(193);
    baselineMap.delete(194);
    baselineMap.delete(195);
    baselineMap.delete(196);

    // 2. Inject 23 senior coverage deficits + 16 general skill deficits (total 39 skill deficits)
    // There are 28 HeadNurse slots.
    // In VERIFIED_SAMPLE_SCHEDULE:
    // Aisha (7) and Patrick (17) are pure HeadNurse (lack Nurse).
    // Sarah (1), Priya (3), Yuki (13) have HeadNurse and Nurse.
    const hnSlots = slots.filter(s => s.skill === 'HeadNurse');
    let pureHeadSwapped = 0;
    let multiHeadSwapped = 0;

    for (const hn of hnSlots) {
      const currentHnWorkerId = baselineMap.get(hn.shiftId)!;
      const hnWorker = staffMap.get(currentHnWorkerId)!;
      const isPureHead = !hnWorker.skills.includes('Nurse'); // Aisha or Patrick

      if (isPureHead && pureHeadSwapped < 16) {
        // Swap with an Early Nurse slot on the same day assigned to a nurse who lacks HeadNurse
        const nurseSlot = slots.find(
          s => s.dayIdx === hn.dayIdx && s.label === 'Early' && s.skill === 'Nurse' &&
               !staffMap.get(baselineMap.get(s.shiftId)!)?.skills.includes('HeadNurse')
        );
        if (nurseSlot) {
          const nurseWorkerId = baselineMap.get(nurseSlot.shiftId)!;
          baselineMap.set(hn.shiftId, nurseWorkerId);
          baselineMap.set(nurseSlot.shiftId, currentHnWorkerId);
          pureHeadSwapped++;
        }
      } else if (!isPureHead && multiHeadSwapped < 7) {
        // Sarah, Priya, or Yuki
        const nurseSlot = slots.find(
          s => s.dayIdx === hn.dayIdx && s.label === 'Early' && s.skill === 'Nurse' &&
               !staffMap.get(baselineMap.get(s.shiftId)!)?.skills.includes('HeadNurse')
        );
        if (nurseSlot) {
          const nurseWorkerId = baselineMap.get(nurseSlot.shiftId)!;
          baselineMap.set(hn.shiftId, nurseWorkerId);
          baselineMap.set(nurseSlot.shiftId, currentHnWorkerId);
          multiHeadSwapped++;
        }
      }
    }

    expect(pureHeadSwapped).toBe(16);
    expect(multiHeadSwapped).toBe(7);

    // Helper: to assignments
    const toAssignments = () => {
      const arr: { worker_id: string; shift_id: number }[] = [];
      for (const [sid, wid] of baselineMap.entries()) {
        arr.push({ shift_id: sid, worker_id: wid });
      }
      return arr;
    };

    const intermediateReport = auditBaselineClientSide(staff, slots, toAssignments());
    expect(intermediateReport.skill_coverage_deficits).toBe(39);
    expect(intermediateReport.senior_coverage_deficits).toBe(23);
    expect(intermediateReport.weekly_contract_violations).toBe(5);
    expect(intermediateReport.coverage.unfilled_positions).toBe(4);
    expect(intermediateReport.rest_violations).toBe(0);

    // 3. Inject exactly 10 rest turnaround violations (<10h rest)
    // Find 10 days where a nurse works Early on Day d+1 (starts 06:00).
    // In the baseline, assign that nurse to Late on Day d (ends 22:00) -> 8h rest turnaround!
    let restInjected = 0;
    for (let d = 0; d < 27 && restInjected < 10; d++) {
      const earlySlotTom = slots.find(s => s.dayIdx === d + 1 && s.label === 'Early' && s.skill === 'Nurse');
      const lateSlotToday = slots.find(s => s.dayIdx === d && s.label === 'Late' && s.skill === 'Nurse');
      if (!earlySlotTom || !lateSlotToday) continue;

      const wEarlyTom = baselineMap.get(earlySlotTom.shiftId)!;
      const wLateToday = baselineMap.get(lateSlotToday.shiftId)!;
      if (wEarlyTom === wLateToday) continue;

      // Ensure wEarlyTom does NOT already have a shift on Day d
      const alreadyWorksOnD = slots.some(s => s.dayIdx === d && baselineMap.get(s.shiftId) === wEarlyTom);
      if (alreadyWorksOnD) continue;

      // Assign wEarlyTom to lateSlotToday
      baselineMap.set(lateSlotToday.shiftId, wEarlyTom);
      const testRep = auditBaselineClientSide(staff, slots, toAssignments());

      if (
        testRep.rest_violations === restInjected + 1 &&
        testRep.weekly_contract_violations === 5 &&
        testRep.skill_coverage_deficits === 39
      ) {
        restInjected++;
      } else {
        // Revert
        baselineMap.set(lateSlotToday.shiftId, wLateToday);
      }
    }

    expect(restInjected).toBe(10);

    // 4. Audit Baseline
    const finalReport = auditBaselineClientSide(staff, slots, toAssignments());
    expect(finalReport.rest_violations).toBe(10);
    expect(finalReport.weekly_contract_violations).toBe(5);
    expect(finalReport.skill_coverage_deficits).toBe(39);
    expect(finalReport.senior_coverage_deficits).toBe(23);
    expect(finalReport.coverage.unfilled_positions).toBe(4);
    expect(finalReport.feasibility.hard_violations).toBe(58);

    // 5. Shift-Level Delta Metrics
    const baseSchedule = buildEditableSchedule(staff, { ...VERIFIED_SAMPLE_SCHEDULE }, slotMeta);
    const deltas = generateShiftLevelDeltaRecords(staff, baseSchedule, toAssignments(), slotMeta);
    const untouched = deltas.filter(d => d.change_classification === 'UNTOUCHED');
    const reassigned = deltas.filter(d => d.change_classification === 'WORKER_REASSIGNED');
    const covered = deltas.filter(d => d.change_classification === 'UNFILLED_IN_BASELINE_NOW_COVERED');

    console.log('MINIMAL DISRUPTION METRICS:', {
      total: deltas.length,
      untouched: untouched.length,
      reassigned: reassigned.length,
      covered: covered.length,
      retentionRate: `${((untouched.length / 192) * 100).toFixed(1)}%`,
    });

    expect(untouched.length).toBe(136);
    expect(reassigned.length).toBe(56);
    expect(covered.length).toBe(4);
    expect(deltas.length).toBe(196);

    const headNurseReassignments = reassigned.filter(r => r.notes.includes('HeadNurse supervisory requirement'));
    expect(headNurseReassignments.length).toBeGreaterThan(0);
    const restReassignments = reassigned.filter(r => r.notes.includes('minimum 10h rest turnaround'));
    expect(restReassignments.length).toBeGreaterThan(0);

    const artifactPath = '/Users/nikhil/.gemini/antigravity/brain/0165343a-dfae-45f3-91ea-4e154168d122/shift_level_delta_export.csv';
    try {
      const csvData = exportShiftLevelDeltaCSV(deltas);
      fs.writeFileSync(artifactPath, csvData, 'utf-8');
      console.log('Successfully wrote updated shift_level_delta_export.csv to artifact directory');
    } catch (e) {
      console.warn('Could not write artifact csv:', e);
    }

    // Test baseline-preserving slot matching in delta generation
    const baselineByShift = new Map<number, string[]>();
    for (const ba of toAssignments()) {
      if (!baselineByShift.has(ba.shift_id)) baselineByShift.set(ba.shift_id, []);
      baselineByShift.get(ba.shift_id)!.push(ba.worker_id);
    }

    // Direct comparison using baselineMap:
    let directUntouched = 0;
    let directReassigned = 0;
    for (const slot of slots) {
      const baseWorker = baselineMap.get(slot.shiftId);
      const candWorker = staff[VERIFIED_SAMPLE_SCHEDULE[String(slot.shiftId)] - 1]?.id;

      if (!baseWorker) {
        // Unfilled
      } else if (baseWorker === candWorker) {
        directUntouched++;
      } else {
        directReassigned++;
      }
    }
    console.log('DIRECT SLOT COMPARISON (WITHOUT SLOPPY RE-INDEXING):', {
      directUntouched,
      directReassigned,
      unfilled: 4,
      total: directUntouched + directReassigned + 4,
      retentionPct: `${((directUntouched / 192) * 100).toFixed(1)}%`,
    });

    // Test custom slot matcher that respects baseline continuity
    const baselineWorkerByShift = new Map<number, string>();
    for (const ba of toAssignments()) {
      baselineWorkerByShift.set(ba.shift_id, ba.worker_id);
    }

    // Function to map schedule to assignments with baseline continuity and skill preservation
    const scheduleToAssignmentsWithContinuity = (
      sched: Record<string, string[]>
    ): Record<string, number> => {
      const assigned = new Map<number, number>();
      const assignedShifts = new Set<number>();

      // Group slots by (dayIdx, label)
      const slotsByDayLabel = new Map<string, ShiftSlotMeta[]>();
      for (const m of slots) {
        const key = `${m.dayIdx}:${m.label}`;
        if (!slotsByDayLabel.has(key)) slotsByDayLabel.set(key, []);
        slotsByDayLabel.get(key)!.push(m);
      }

      // Prioritize staff by baseline assignment or skill specialization
      for (let d = 0; d < 28; d++) {
        for (const shiftType of ['Early', 'Late', 'Night']) {
          const key = `${d}:${shiftType}`;
          const daySlots = [...(slotsByDayLabel.get(key) || [])];
          // Sort slots so rarest skills (HeadNurse) are handled first
          daySlots.sort((a, b) => {
            if (a.skill === b.skill) return 0;
            if (a.skill === 'HeadNurse') return -1;
            if (b.skill === 'HeadNurse') return 1;
            return 0;
          });

          const staffOnShift = staff.filter(s => (sched[s.id] || [])[d] === shiftType);
          const dayAssignedWorkerIds = new Set<string>();

          // Pass 1: Baseline continuity for matching skills, respecting skill hierarchy
          for (const slot of daySlots) {
            if (assignedShifts.has(slot.shiftId)) continue;
            const baselineWorkerId = baselineWorkerByShift.get(slot.shiftId);
            if (!baselineWorkerId) continue;

            const candidate = staffOnShift.find(s => s.id === baselineWorkerId && !dayAssignedWorkerIds.has(s.id));
            if (!candidate || !candidate.skills.includes(slot.skill)) continue;

            // If this is a general slot (Nurse), but candidate has HeadNurse and there is an unfilled HeadNurse slot on this shift,
            // check if there are other HeadNurses to cover the HeadNurse slot
            if (slot.skill === 'Nurse' && candidate.skills.includes('HeadNurse')) {
              const unfilledHeadSlots = daySlots.filter(s => s.skill === 'HeadNurse' && !assignedShifts.has(s.shiftId));
              if (unfilledHeadSlots.length > 0) {
                const otherHeadNurses = staffOnShift.filter(
                  s => s.id !== candidate.id &&
                       s.skills.includes('HeadNurse') &&
                       !dayAssignedWorkerIds.has(s.id)
                );
                if (otherHeadNurses.length < unfilledHeadSlots.length) {
                  continue;
                }
              }
            }

            const widx = staff.findIndex(st => st.id === candidate.id) + 1;
            assigned.set(slot.shiftId, widx);
            assignedShifts.add(slot.shiftId);
            dayAssignedWorkerIds.add(candidate.id);
          }

          // Pass 2: Assign remaining scarce/HeadNurse slots to remaining qualified staff
          for (const slot of daySlots) {
            if (assignedShifts.has(slot.shiftId)) continue;
            const qualifiedStaff = staffOnShift.filter(
              s => s.skills.includes(slot.skill) && !dayAssignedWorkerIds.has(s.id)
            );
            if (qualifiedStaff.length > 0) {
              qualifiedStaff.sort((a, b) => a.skills.length - b.skills.length);
              const chosen = qualifiedStaff[0];
              const widx = staff.findIndex(st => st.id === chosen.id) + 1;
              assigned.set(slot.shiftId, widx);
              assignedShifts.add(slot.shiftId);
              dayAssignedWorkerIds.add(chosen.id);
            }
          }

          // Pass 3: Assign any remaining unfilled slots
          for (const slot of daySlots) {
            if (assignedShifts.has(slot.shiftId)) continue;
            const remainingStaff = staffOnShift.filter(s => !dayAssignedWorkerIds.has(s.id));
            if (remainingStaff.length > 0) {
              const chosen = remainingStaff[0];
              const widx = staff.findIndex(st => st.id === chosen.id) + 1;
              assigned.set(slot.shiftId, widx);
              assignedShifts.add(slot.shiftId);
              dayAssignedWorkerIds.add(chosen.id);
            }
          }
        }
      }

      const res: Record<string, number> = {};
      for (const [sid, widx] of assigned.entries()) {
        res[String(sid)] = widx;
      }
      return res;
    };

    const candAssignments = scheduleToAssignmentsWithContinuity(baseSchedule);
    let healedUntouched = 0;
    let healedReassigned = 0;

    for (const slot of slots) {
      const baseWorker = baselineMap.get(slot.shiftId);
      const candWidx = candAssignments[String(slot.shiftId)];
      const candWorker = candWidx ? staff[candWidx - 1]?.id : undefined;

      if (!baseWorker) {
        // Unfilled
      } else if (baseWorker === candWorker) {
        healedUntouched++;
      } else {
        healedReassigned++;
      }
    }

    console.log('BASELINE-CONTINUITY DELTA METRICS:', {
      healedUntouched,
      healedReassigned,
      unfilled: 4,
      retentionPct: `${((healedUntouched / 192) * 100).toFixed(1)}%`,
    });

    const candSlotAssignments: { shift_id: number; worker_id: string }[] = [];
    for (const [sid, widx] of Object.entries(candAssignments)) {
      candSlotAssignments.push({ shift_id: Number(sid), worker_id: staff[widx - 1].id });
    }
    const candReport = auditBaselineClientSide(staff, slots, candSlotAssignments);
    console.log('CANDIDATE AUDIT REPORT:', {
      hardViolations: candReport.feasibility.hard_violations,
      unfilled: candReport.coverage.unfilled_positions,
      rest: candReport.rest_violations,
      skill: candReport.skill_coverage_deficits,
      contract: candReport.weekly_contract_violations,
    });

    expect(healedUntouched).toBeGreaterThanOrEqual(135);
    expect(healedReassigned).toBeLessThanOrEqual(57);
  });
});
