// P3.3 Regression tests for redistributeWithLocks()
//
// G-8: Locked-cell preservation — scheduler edit on cell X → redistribution → cell X exactly unchanged.
// G-9: Adjacent provenance — redistribution changes surrounding assignments → every changed cell
//      has an individual system_reassignment provenance record with all required fields.
//
// These tests prove both preservation AND useful redistribution (not a no-op).

import { redistributeWithLocks, buildSyntheticSchedule } from './WorkflowUtils';
import type { StaffMember } from './WorkflowTypes';

// Minimal staff fixture — 8 members, enough to produce real redistribution
const STAFF: StaffMember[] = [
  { id: 'Alice',   contract: 'FullTime', skills: ['Nurse'] },
  { id: 'Bob',     contract: 'FullTime', skills: ['Nurse'] },
  { id: 'Carol',   contract: 'FullTime', skills: ['Nurse'] },
  { id: 'David',   contract: 'PartTime', skills: ['Nurse'] },
  { id: 'Eve',     contract: 'FullTime', skills: ['Nurse'] },
  { id: 'Frank',   contract: 'Night',    skills: ['Nurse'] },
  { id: 'Grace',   contract: 'FullTime', skills: ['Nurse'] },
  { id: 'Hank',    contract: 'PartTime', skills: ['Nurse'] },
];

// ─── G-8: Locked-cell preservation ───────────────────────────────────────────

test('G-8: locked cell is exactly unchanged after redistribution', () => {
  const baseSchedule = buildSyntheticSchedule(STAFF);

  // Scheduler edits Alice day 0 to 'Night' and locks it
  const editedSchedule = { ...baseSchedule, Alice: [...baseSchedule['Alice']] };
  editedSchedule['Alice'][0] = 'Night';
  const lockedCells = new Set(['Alice:0']);

  const result = redistributeWithLocks(STAFF, editedSchedule, lockedCells);

  // G-8: Alice day 0 must be exactly 'Night' — the scheduler's edit
  expect(result.schedule['Alice'][0]).toBe('Night');

  // G-8: provenance for Alice:0 must be 'scheduler_edit'
  expect(result.log.provenanceMap['Alice:0']).toBe('scheduler_edit');

  // G-8: lockedAssignmentsChanged must be 0 — the invariant
  expect(result.log.lockedAssignmentsChanged).toBe(0);
});

test('G-8: multiple locked cells are all preserved', () => {
  const baseSchedule = buildSyntheticSchedule(STAFF);

  const editedSchedule = {
    ...baseSchedule,
    Alice: [...baseSchedule['Alice']],
    Bob:   [...baseSchedule['Bob']],
  };
  editedSchedule['Alice'][3] = 'Late';
  editedSchedule['Bob'][7]   = '';

  const lockedCells = new Set(['Alice:3', 'Bob:7']);
  const result = redistributeWithLocks(STAFF, editedSchedule, lockedCells);

  expect(result.schedule['Alice'][3]).toBe('Late');
  expect(result.schedule['Bob'][7]).toBe('');
  expect(result.log.provenanceMap['Alice:3']).toBe('scheduler_edit');
  expect(result.log.provenanceMap['Bob:7']).toBe('scheduler_edit');
  expect(result.log.lockedAssignmentsChanged).toBe(0);
  expect(result.log.schedulerEditsPreserved).toBe(2);
});

// ─── G-9: Adjacent provenance — every changed cell has a ChangeRecord ─────────

test('G-9: every system_reassignment cell has a ChangeRecord with all required fields', () => {
  // Build a schedule that is DIFFERENT from the synthetic base so that
  // unlocked cells will actually be changed by redistribution.
  // We do this by filling every cell with a fixed value ('Late') — the
  // synthetic base uses a rotating pattern, so most unlocked cells will differ.
  const differentSchedule: Record<string, string[]> = {};
  for (const s of STAFF) {
    differentSchedule[s.id] = Array(28).fill('Late');
  }

  // Lock Carol day 5 to 'Night' — redistribution must not touch it
  differentSchedule['Carol'] = [...differentSchedule['Carol']];
  differentSchedule['Carol'][5] = 'Night';
  const lockedCells = new Set(['Carol:5']);

  const result = redistributeWithLocks(STAFF, differentSchedule, lockedCells);

  // Find all system_reassignment cells
  const reassignedKeys = Object.entries(result.log.provenanceMap)
    .filter(([, state]) => state === 'system_reassignment')
    .map(([key]) => key);

  // G-9: there must be at least one actual reassignment (not a no-op)
  expect(reassignedKeys.length).toBeGreaterThan(0);

  // G-9: every reassigned cell must have a ChangeRecord with all required fields
  for (const key of reassignedKeys) {
    const record = result.log.changeRecords.find(r => r.assignmentId === key);
    expect(record).toBeDefined();
    if (!record) continue;

    expect(typeof record.assignmentId).toBe('string');
    expect(record.assignmentId).toBe(key);
    expect(typeof record.previousValue).toBe('string');
    expect(typeof record.newValue).toBe('string');
    expect(typeof record.redistributionOperationId).toBe('string');
    expect(record.redistributionOperationId.length).toBeGreaterThan(0);
    expect(typeof record.reason).toBe('string');
    expect(record.reason.length).toBeGreaterThan(0);
    expect(typeof record.timestamp).toBe('string');
    // timestamp must be a valid ISO-8601 date
    expect(new Date(record.timestamp).getTime()).not.toBeNaN();
  }
});

test('G-9: ChangeRecord count matches system_reassignment count in provenanceMap', () => {
  const baseSchedule = buildSyntheticSchedule(STAFF);
  const lockedCells = new Set(['David:10', 'Eve:15']);
  const result = redistributeWithLocks(STAFF, baseSchedule, lockedCells);

  const reassignedCount = Object.values(result.log.provenanceMap)
    .filter(s => s === 'system_reassignment').length;

  expect(result.log.changeRecords.length).toBe(reassignedCount);
  expect(result.log.assignmentsReassigned).toBe(reassignedCount);
});

// ─── Invariant: log fields are consistent ─────────────────────────────────────

test('log.operationId is stable and non-empty', () => {
  const baseSchedule = buildSyntheticSchedule(STAFF);
  const result = redistributeWithLocks(STAFF, baseSchedule, new Set());

  expect(typeof result.log.operationId).toBe('string');
  expect(result.log.operationId.length).toBeGreaterThan(0);

  // All ChangeRecords reference the same operationId
  for (const record of result.log.changeRecords) {
    expect(record.redistributionOperationId).toBe(result.log.operationId);
  }
});

test('provenance map covers every cell in the schedule', () => {
  const baseSchedule = buildSyntheticSchedule(STAFF);
  const lockedCells = new Set(['Alice:0']);
  const editedSchedule = { ...baseSchedule, Alice: [...baseSchedule['Alice']] };
  editedSchedule['Alice'][0] = 'Night';

  const result = redistributeWithLocks(STAFF, editedSchedule, lockedCells);

  // Every staffId × dayIdx combination must appear in the provenanceMap
  for (const s of STAFF) {
    for (let d = 0; d < 28; d++) {
      const key = `${s.id}:${d}`;
      expect(result.log.provenanceMap[key]).toBeDefined();
      expect(['original', 'scheduler_edit', 'system_reassignment', 'unchanged'])
        .toContain(result.log.provenanceMap[key]);
    }
  }
});