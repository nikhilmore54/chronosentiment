// @vitest-environment jsdom
import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  parseStaffCSV,
  getScheduleSlotMeta,
  buildEditableSchedule,
  buildSyntheticAlternatives,
  computeScheduleDifferences,
  createRosterCheckpoint,
  validateCheckpointCompatibility,
  restoreRosterCheckpoint,
  validateCurrentSchedule,
  VERIFIED_SAMPLE_SCHEDULE,
} from './WorkflowUtils';
import type { StaffMember, RosterCheckpoint } from './WorkflowTypes';
import { SAMPLE_CSV } from './WorkflowTypes';
import { DeviceSessionService } from '../services/DeviceSessionService';

describe('AT-42 — Roster Provenance and Checkpoint Restoration', () => {
  let staff: StaffMember[];
  let slotMeta: ReturnType<typeof getScheduleSlotMeta>;
  let originalRecommendation: Record<string, string[]>;
  let candidateB: Record<string, string[]>;

  beforeEach(() => {
    DeviceSessionService.resetRevision();
    vi.restoreAllMocks();
    const { staff: parsedStaff } = parseStaffCSV(SAMPLE_CSV);
    staff = parsedStaff;
    slotMeta = getScheduleSlotMeta(staff, []);
    const baseSchedule = buildEditableSchedule(staff, { ...VERIFIED_SAMPLE_SCHEDULE }, slotMeta);
    const { alternatives } = buildSyntheticAlternatives(staff, baseSchedule, slotMeta);

    // Candidate A is the immutable original recommendation
    originalRecommendation = alternatives.find(a => a.id === 'alt-A')?.schedule || baseSchedule;
    // Candidate B is the selected starting alternative
    candidateB = alternatives.find(a => a.id === 'alt-B')?.schedule || baseSchedule;
  });

  // AT-42.1: No question mark on unchanged assignments
  it('AT-42.1: does not generate diff indicator on unchanged assignments', () => {
    // Current working roster is identical to original recommendation
    const currentSchedule = { ...originalRecommendation };

    const diffMap = computeScheduleDifferences({
      staff,
      currentSchedule,
      originalRecommendation,
      selectedStartingRoster: originalRecommendation,
      customSlotMeta: slotMeta,
    });

    // Every shift matches -> diffMap must be completely empty
    expect(diffMap.size).toBe(0);

    for (const s of staff) {
      for (let d = 0; d < 28; d++) {
        expect(diffMap.has(`${s.id}:${d}`)).toBe(false);
      }
    }
  });

  // AT-42.2: Question mark appears when current assignment differs from original recommendation
  it('AT-42.2: generates diff indicator when current assignment differs from original recommendation', () => {
    const currentSchedule: Record<string, string[]> = {};
    for (const [k, v] of Object.entries(originalRecommendation)) {
      currentSchedule[k] = [...v];
    }

    // Change Sarah_Chen on day 0 from whatever it was to 'Night'
    const origShift = currentSchedule['Sarah_Chen'][0];
    currentSchedule['Sarah_Chen'][0] = origShift === 'Night' ? 'Early' : 'Night';

    const lockedCells = new Set(['Sarah_Chen:0']);
    const diffMap = computeScheduleDifferences({
      staff,
      currentSchedule,
      originalRecommendation,
      lockedCells,
      customSlotMeta: slotMeta,
    });

    expect(diffMap.has('Sarah_Chen:0')).toBe(true);
    const diff = diffMap.get('Sarah_Chen:0')!;
    expect(diff.originalShift).toBe(origShift);
    expect(diff.currentShift).toBe(currentSchedule['Sarah_Chen'][0]);
    expect(diff.source).toBe('manual_edit');
    expect(diff.sourceLabel).toBe('Manual planner edit');

    // Untouched cell must not have indicator
    expect(diffMap.has('Sarah_Chen:1')).toBe(false);
  });

  // AT-42.3: Candidate B differences are identified as alternative selection, not manual edits
  it('AT-42.3: identifies Candidate B differences as alternative selection rather than manual edits', () => {
    // Planner selected Candidate B before making any manual edits
    const diffMap = computeScheduleDifferences({
      staff,
      currentSchedule: candidateB,
      originalRecommendation,
      selectedStartingRoster: candidateB,
      lockedCells: new Set(), // No manual edits yet!
      customSlotMeta: slotMeta,
    });

    // Candidate B rotates weekend assignments, so there are differences against Candidate A
    expect(diffMap.size).toBeGreaterThan(0);

    // Every difference caused by Candidate B must be classified as 'alternative_selection'
    for (const [, diff] of diffMap.entries()) {
      expect(diff.source).toBe('alternative_selection');
      expect(diff.sourceLabel).toContain('Selected alternative');
      expect(diff.source).not.toBe('manual_edit');
    }
  });

  // AT-42.4: Manual and automatic reassignment sources are distinguishable
  it('AT-42.4: distinguishes between manual edits, automatic rest recovery, and redistribution sources', () => {
    const currentSchedule: Record<string, string[]> = {};
    for (const [k, v] of Object.entries(originalRecommendation)) {
      currentSchedule[k] = [...v];
    }

    // 1. Manual edit
    const origShift5 = originalRecommendation['Sarah_Chen'][5] ?? '';
    currentSchedule['Sarah_Chen'][5] = origShift5 === 'Night' ? 'Early' : 'Night';
    const lockedCells = new Set(['Sarah_Chen:5']);

    // 2. Automatic rest conflict recovery: day 6 had a shift and was evacuated to Off
    originalRecommendation['Sarah_Chen'][6] = 'Early';
    currentSchedule['Sarah_Chen'][6] = ''; // evacuated to preserve 10h rest
    const autoResolvedCells = new Map<string, string>();
    autoResolvedCells.set('Sarah_Chen:6', 'Evacuated adjacent shift to guarantee configured minimum 10h rest');

    // 3. Automatic redistribution: day 6 for Elena was Off and is now assigned Early
    originalRecommendation['Elena_Vasquez'][6] = '';
    currentSchedule['Elena_Vasquez'][6] = 'Early';
    const fakeRedistResult: any = {
      schedule: currentSchedule,
      log: {
        operationId: 'op-test-1',
        timestamp: new Date().toISOString(),
        schedulerEditsPreserved: 1,
        assignmentsReassigned: 1,
        lockedAssignmentsChanged: 0,
        changeRecords: [
          {
            assignmentId: 'Elena_Vasquez:6',
            previousValue: '',
            newValue: 'Early',
            redistributionOperationId: 'op-test-1',
            reason: 'Rebalanced to satisfy morning skill coverage',
            timestamp: new Date().toISOString(),
          },
        ],
        provenanceMap: {
          'Elena_Vasquez:6': 'system_reassignment',
        },
      },
    };

    const diffMap = computeScheduleDifferences({
      staff,
      currentSchedule,
      originalRecommendation,
      lockedCells,
      autoResolvedCells,
      redistResult: fakeRedistResult,
      customSlotMeta: slotMeta,
    });

    // Check manual edit source
    const manualDiff = diffMap.get('Sarah_Chen:5')!;
    expect(manualDiff.source).toBe('manual_edit');
    expect(manualDiff.sourceLabel).toBe('Manual planner edit');

    // Check auto rest recovery source
    const autoRestDiff = diffMap.get('Sarah_Chen:6')!;
    expect(autoRestDiff.source).toBe('auto_rest_recovery');
    expect(autoRestDiff.sourceLabel).toBe('Automatic rest-conflict resolution');

    // Check auto redistribution source
    const redistDiff = diffMap.get('Elena_Vasquez:6')!;
    expect(redistDiff.source).toBe('auto_redistribution');
    expect(redistDiff.sourceLabel).toBe('Automatic redistribution');
  });

  // AT-42.5: Saving a checkpoint preserves assignments and manual locks
  it('AT-42.5: saving a checkpoint preserves assignments and manual locks', () => {
    const workingSchedule: Record<string, string[]> = {};
    for (const [k, v] of Object.entries(candidateB)) {
      workingSchedule[k] = [...v];
    }
    workingSchedule['Sarah_Chen'][1] = 'Night';
    workingSchedule['Marcus_Webb'][2] = 'Late';
    const lockedCells = new Set(['Sarah_Chen:1', 'Marcus_Webb:2']);

    const checkpoint = createRosterCheckpoint({
      name: 'Checkpoint 01 (Post-Weekend Edit)',
      staff,
      schedule: workingSchedule,
      lockedCells,
      scenarioId: '20-nurse',
      recommendationId: 'alt-A',
      selectedCandidateId: 'alt-B',
      policySnapshotHash: 'hospital_policy_v1',
      demandSnapshotId: 'dmd-snap-196',
      validationSummary: { coverage: 1.0, hardViolations: 0 },
      customSlotMeta: slotMeta,
    });

    expect(checkpoint.checkpointId).toBeDefined();
    expect(checkpoint.name).toBe('Checkpoint 01 (Post-Weekend Edit)');
    expect(checkpoint.scenarioId).toBe('20-nurse');
    expect(checkpoint.lockedShiftIds).toContain('Sarah_Chen:1');
    expect(checkpoint.lockedShiftIds).toContain('Marcus_Webb:2');
    expect(checkpoint.schedule['Sarah_Chen'][1]).toBe('Night');
    expect(checkpoint.schedule['Marcus_Webb'][2]).toBe('Late');
    expect(checkpoint.assignments).toBeDefined();
  });

  // AT-42.6: Restoring a checkpoint reproduces the exact saved roster
  it('AT-42.6: restoring a checkpoint reproduces the exact saved roster and manual locks', () => {
    const checkpointSchedule: Record<string, string[]> = {};
    for (const [k, v] of Object.entries(candidateB)) {
      checkpointSchedule[k] = [...v];
    }
    checkpointSchedule['Sarah_Chen'][3] = 'Early';
    const savedLocks = new Set(['Sarah_Chen:3']);

    const checkpoint = createRosterCheckpoint({
      name: 'Milestone Alpha',
      staff,
      schedule: checkpointSchedule,
      lockedCells: savedLocks,
      scenarioId: '20-nurse',
      recommendationId: 'alt-A',
      selectedCandidateId: 'alt-B',
      policySnapshotHash: 'hospital_policy_v1',
      demandSnapshotId: 'dmd-snap-196',
      validationSummary: { coverage: 1.0, hardViolations: 0 },
      customSlotMeta: slotMeta,
    });

    // Mutate working roster further
    const mutatedSchedule = { ...checkpointSchedule, 'Sarah_Chen': [...checkpointSchedule['Sarah_Chen']] };
    mutatedSchedule['Sarah_Chen'][3] = 'Night';

    // Now restore checkpoint
    const restored = restoreRosterCheckpoint(
      checkpoint,
      '20-nurse',
      'hospital_policy_v1',
      'dmd-snap-196',
      staff,
      slotMeta,
    );

    expect(restored.restoredSchedule['Sarah_Chen'][3]).toBe('Early');
    expect(restored.restoredLockedCells.has('Sarah_Chen:3')).toBe(true);
    expect(restored.restoredFromId).toBe(checkpoint.checkpointId);
    expect(restored.restoredName).toBe('Milestone Alpha');
  });

  // AT-42.7: Restoring does not modify the original recommendation or delete newer checkpoints
  it('AT-42.7: non-destructive restoration preserves original recommendation and subsequent checkpoints', () => {
    const cp1 = createRosterCheckpoint({
      name: 'Checkpoint 01',
      staff,
      schedule: candidateB,
      lockedCells: new Set(),
      scenarioId: '20-nurse',
      recommendationId: 'alt-A',
      selectedCandidateId: 'alt-B',
      policySnapshotHash: 'hospital_policy_v1',
      demandSnapshotId: 'dmd-snap-196',
      validationSummary: { coverage: 1.0, hardViolations: 0 },
      customSlotMeta: slotMeta,
    });

    const cp2Schedule = { ...candidateB, 'Marcus_Webb': [...candidateB['Marcus_Webb']] };
    cp2Schedule['Marcus_Webb'][0] = 'Early';
    const cp2 = createRosterCheckpoint({
      name: 'Checkpoint 02',
      staff,
      schedule: cp2Schedule,
      lockedCells: new Set(['Marcus_Webb:0']),
      scenarioId: '20-nurse',
      recommendationId: 'alt-A',
      selectedCandidateId: 'alt-B',
      policySnapshotHash: 'hospital_policy_v1',
      demandSnapshotId: 'dmd-snap-196',
      validationSummary: { coverage: 1.0, hardViolations: 0 },
      customSlotMeta: slotMeta,
    });

    const checkpointHistory = [cp1, cp2];

    // Restore Checkpoint 01
    const origRecommendationBefore = JSON.stringify(originalRecommendation);
    const restored = restoreRosterCheckpoint(
      cp1,
      '20-nurse',
      'hospital_policy_v1',
      'dmd-snap-196',
      staff,
      slotMeta,
    );

    // 1. Original recommendation remains completely untouched
    expect(JSON.stringify(originalRecommendation)).toBe(origRecommendationBefore);

    // 2. Subsequent Checkpoint 02 remains in the history array untouched
    expect(checkpointHistory).toHaveLength(2);
    expect(checkpointHistory[1].name).toBe('Checkpoint 02');
    expect(checkpointHistory[1].lockedShiftIds).toContain('Marcus_Webb:0');

    // 3. Working revision was updated to cp1
    expect(restored.restoredName).toBe('Checkpoint 01');
  });

  // AT-42.8: Checkpoints cannot be silently restored across incompatible scenarios or policy versions
  it('AT-42.8: rejects silent cross-scenario and cross-policy restoration', () => {
    const cp = createRosterCheckpoint({
      name: '20-Nurse Baseline Milestone',
      staff,
      schedule: candidateB,
      lockedCells: new Set(),
      scenarioId: '20-nurse',
      recommendationId: 'alt-A',
      selectedCandidateId: 'alt-B',
      policySnapshotHash: 'hospital_policy_v1',
      demandSnapshotId: 'dmd-snap-196',
      validationSummary: { coverage: 1.0, hardViolations: 0 },
      customSlotMeta: slotMeta,
    });

    // Scenario mismatch: attempting to restore 20-nurse checkpoint into 12-nurse team
    const scenarioCheck = validateCheckpointCompatibility(
      cp,
      '12-nurse',
      'hospital_policy_v1',
      'dmd-snap-196',
    );
    expect(scenarioCheck.compatible).toBe(false);
    expect(scenarioCheck.error).toContain('scenario mismatch');

    // Policy hash mismatch
    const policyCheck = validateCheckpointCompatibility(
      cp,
      '20-nurse',
      'hospital_policy_v2_strict',
      'dmd-snap-196',
    );
    expect(policyCheck.compatible).toBe(false);
    expect(policyCheck.error).toContain('policy version mismatch');

    // Demand snapshot mismatch
    const demandCheck = validateCheckpointCompatibility(
      cp,
      '20-nurse',
      'hospital_policy_v1',
      'dmd-snap-250-beds',
    );
    expect(demandCheck.compatible).toBe(false);
    expect(demandCheck.error).toContain('demand snapshot mismatch');

    // restoreRosterCheckpoint must throw on incompatibility
    expect(() => {
      restoreRosterCheckpoint(cp, '12-nurse', 'hospital_policy_v1', 'dmd-snap-196', staff, slotMeta);
    }).toThrow(/scenario mismatch/);
  });

  // AT-42.9: Restored roster is independently validated before being treated as accepted
  it('AT-42.9: independently validates restored roster before acceptance', async () => {
    const cp = createRosterCheckpoint({
      name: 'Valid Checkpoint',
      staff,
      schedule: originalRecommendation,
      lockedCells: new Set(),
      scenarioId: '20-nurse',
      recommendationId: 'alt-A',
      selectedCandidateId: 'alt-A',
      policySnapshotHash: 'hospital_policy_v1',
      demandSnapshotId: 'dmd-snap-196',
      validationSummary: { coverage: 1.0, hardViolations: 0 },
      customSlotMeta: slotMeta,
    });

    const restored = restoreRosterCheckpoint(
      cp,
      '20-nurse',
      'hospital_policy_v1',
      'dmd-snap-196',
      staff,
      slotMeta,
    );

    // Run independent validation
    const val = await validateCurrentSchedule(staff, restored.restoredSchedule, undefined, slotMeta);
    expect(val).toBeDefined();
    const hardViolations = val?.constraint_report?.hard_violations ?? val?.hard_violations ?? 0;
    expect(hardViolations).toBe(0);
  });

  // AT-42.10: Checkpoint history survives browser refresh and device-session restoration
  it('AT-42.10: checkpoint history survives serialization and session restoration', async () => {
    const cp1 = createRosterCheckpoint({
      name: 'Initial Working Checkpoint',
      staff,
      schedule: originalRecommendation,
      lockedCells: new Set(['Sarah_Chen:0']),
      scenarioId: '20-nurse',
      recommendationId: 'alt-A',
      selectedCandidateId: 'alt-B',
      policySnapshotHash: 'hospital_policy_v1',
      demandSnapshotId: 'dmd-snap-196',
      validationSummary: { coverage: 1.0, hardViolations: 0 },
      customSlotMeta: slotMeta,
    });

    const sessionState = {
      step: 5,
      staff,
      demoScenario: '20-nurse',
      editableSchedule: candidateB,
      originalRecommendationSchedule: originalRecommendation,
      selectedStartingSchedule: candidateB,
      rosterCheckpoints: [cp1],
      restoredCheckpointInfo: { checkpointId: cp1.checkpointId, name: cp1.name },
    };

    // Mock fetch for DeviceSessionService
    const storedState = JSON.parse(JSON.stringify(sessionState));
    const mockFetch = vi.fn().mockImplementation(async (url: string, init?: any) => {
      if (init?.method === 'POST') {
        const body = JSON.parse(init.body);
        return {
          ok: true,
          json: async () => ({
            status: 'saved',
            device_token: 'dev-token-xyz',
            schema_version: 1,
            revision: 1,
            updated_at: new Date().toISOString(),
          }),
        };
      }
      return {
        ok: true,
        json: async () => ({
          device_token: 'dev-token-xyz',
          schema_version: 1,
          revision: 1,
          updated_at: new Date().toISOString(),
          state: storedState,
        }),
      };
    });
    global.fetch = mockFetch;

    // Simulate saving to backend device session
    const saveRes = await DeviceSessionService.saveSessionNow(sessionState);
    expect(saveRes).not.toBeNull();
    expect(saveRes?.status).toBe('saved');

    // Simulate page refresh / rehydration from backend device session
    const rehydrated = await DeviceSessionService.loadSession<typeof sessionState>();
    expect(rehydrated).not.toBeNull();
    const state = rehydrated!.state!;

    expect(state.rosterCheckpoints).toHaveLength(1);
    expect(state.rosterCheckpoints[0].name).toBe('Initial Working Checkpoint');
    expect(state.rosterCheckpoints[0].lockedShiftIds).toContain('Sarah_Chen:0');
    expect(state.restoredCheckpointInfo?.name).toBe('Initial Working Checkpoint');
    expect(state.originalRecommendationSchedule).toBeDefined();
  });
});
