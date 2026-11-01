// @vitest-environment jsdom
// P3.3-CR Regression tests for redistributeWithLocks()
//
// G-8: Locked-cell preservation — scheduler edit on cell X → redistribution → cell X exactly unchanged.
// G-9: Adjacent provenance — redistribution changes surrounding assignments → every changed cell
//      has an individual system_reassignment provenance record with all required fields.
//
// P3.3-CR: redistributeWithLocks() now calls POST /api/reschedule (Coralys-backed).
// fetch is mocked to return a controlled schedule so tests remain deterministic.
// The invariants (locked cells preserved, provenance correct) are unchanged.

import { describe, test, expect, vi, beforeEach } from 'vitest';
import {
  redistributeWithLocks,
  buildSyntheticSchedule,
  buildSchedulePayload,
  buildLockedShiftIds,
  getScheduleSlotMeta,
  scheduleToAssignments,
  buildEditableSchedule,
  parseStaffCSV,
  VERIFIED_SAMPLE_SCHEDULE,
  computeCanonicalCoverage,
  buildStaffingRequirements,
} from './WorkflowUtils';
import type { StaffMember } from './WorkflowTypes';
import { SAMPLE_CSV } from './WorkflowTypes';

// Minimal staff fixture — 8 members, all Nurse skill
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

// Build a valid apiSchedule (shift_id → worker_id) for the STAFF fixture.
// For 8 staff all with skill 'Nurse', buildSchedulePayload generates 5 shifts (IDs 1–5).
// We assign each shift to a different worker (1-indexed).
function makeApiSchedule(overrides: Record<number, number> = {}): Record<string, number> {
  const base: Record<string, number> = {
    '1': 1, // shift 1 (Mon Early) → Alice (worker 1)
    '2': 2, // shift 2 (Tue Late)  → Bob   (worker 2)
    '3': 3, // shift 3 (Wed Early) → Carol (worker 3)
    '4': 4, // shift 4 (Thu Late)  → David (worker 4)
    '5': 5, // shift 5 (Fri Early) → Eve   (worker 5)
  };
  for (const [k, v] of Object.entries(overrides)) {
    base[k] = v;
  }
  return base;
}

// Mock fetch to return a controlled reschedule response.
// The mock returns the same apiSchedule by default (no changes), or a custom one.
function mockFetch(responseSchedule: Record<string, number>) {
  vi.stubGlobal('fetch', vi.fn().mockImplementation((url: string) => {
    if (url === '/api/csrf-token') {
      return Promise.resolve({ ok: true, json: () => Promise.resolve({ csrf_token: 'test-token' }) });
    }
    if (url === '/api/reschedule') {
      return Promise.resolve({
        ok: true,
        json: () => Promise.resolve({ schedule: responseSchedule }),
      });
    }
    return Promise.reject(new Error(`Unexpected fetch: ${url}`));
  }));
}

// ─── G-8: Locked-cell preservation ───────────────────────────────────────────

test('G-8: locked cell is exactly unchanged after redistribution', async () => {
  const baseSchedule = buildSyntheticSchedule(STAFF);
  const apiSchedule = makeApiSchedule();

  // Scheduler edits Alice day 0 to 'Night' and locks it
  const editedSchedule = { ...baseSchedule, Alice: [...baseSchedule['Alice']] };
  editedSchedule['Alice'][0] = 'Night';
  const lockedCells = new Set(['Alice:0']);

  // Mock: API returns the same apiSchedule (no changes to unlocked cells)
  mockFetch(apiSchedule);

  const result = await redistributeWithLocks(STAFF, editedSchedule, lockedCells, apiSchedule, {}, 'test-token');

  // G-8: Alice day 0 must be exactly 'Night' — the scheduler's edit
  expect(result.schedule['Alice'][0]).toBe('Night');

  // G-8: provenance for Alice:0 must be 'scheduler_edit'
  expect(result.log.provenanceMap['Alice:0']).toBe('scheduler_edit');

  // G-8: lockedAssignmentsChanged must be 0 — the invariant
  expect(result.log.lockedAssignmentsChanged).toBe(0);
});

test('G-8: multiple locked cells are all preserved', async () => {
  const baseSchedule = buildSyntheticSchedule(STAFF);
  const apiSchedule = makeApiSchedule();

  const editedSchedule = {
    ...baseSchedule,
    Alice: [...baseSchedule['Alice']],
    Bob:   [...baseSchedule['Bob']],
  };
  editedSchedule['Alice'][3] = 'Late';
  editedSchedule['Bob'][7]   = '';

  const lockedCells = new Set(['Alice:3', 'Bob:7']);

  mockFetch(apiSchedule);

  const result = await redistributeWithLocks(STAFF, editedSchedule, lockedCells, apiSchedule, {}, 'test-token');

  expect(result.schedule['Alice'][3]).toBe('Late');
  expect(result.schedule['Bob'][7]).toBe('');
  expect(result.log.provenanceMap['Alice:3']).toBe('scheduler_edit');
  expect(result.log.provenanceMap['Bob:7']).toBe('scheduler_edit');
  expect(result.log.lockedAssignmentsChanged).toBe(0);
  expect(result.log.schedulerEditsPreserved).toBe(2);
});

// ─── G-9: Adjacent provenance — every changed cell has a ChangeRecord ─────────

test('G-9: every system_reassignment cell has a ChangeRecord with all required fields', async () => {
  // Build a schedule that is DIFFERENT from what the API will return so that
  // unlocked cells will actually be changed by redistribution.
  const differentSchedule: Record<string, string[]> = {};
  for (const s of STAFF) {
    differentSchedule[s.id] = Array(28).fill('Late');
  }

  // Lock Carol day 5 to 'Night' — redistribution must not touch it
  differentSchedule['Carol'] = [...differentSchedule['Carol']];
  differentSchedule['Carol'][5] = 'Night';
  const lockedCells = new Set(['Carol:5']);

  // API returns a schedule that differs from differentSchedule for unlocked cells
  // (shift 3 → Carol, dayOfWeek=2, so days 2,9,16,23 will change from 'Late' to 'Early')
  const apiSchedule = makeApiSchedule();
  mockFetch(apiSchedule);

  const result = await redistributeWithLocks(STAFF, differentSchedule, lockedCells, apiSchedule, {}, 'test-token');

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

test('G-9: ChangeRecord count matches system_reassignment count in provenanceMap', async () => {
  const baseSchedule = buildSyntheticSchedule(STAFF);
  const apiSchedule = makeApiSchedule();
  const lockedCells = new Set(['David:10', 'Eve:15']);

  mockFetch(apiSchedule);

  const result = await redistributeWithLocks(STAFF, baseSchedule, lockedCells, apiSchedule, {}, 'test-token');

  const reassignedCount = Object.values(result.log.provenanceMap)
    .filter(s => s === 'system_reassignment').length;

  expect(result.log.changeRecords.length).toBe(reassignedCount);
  expect(result.log.assignmentsReassigned).toBe(reassignedCount);
});

// ─── Invariant: log fields are consistent ─────────────────────────────────────

test('log.operationId is stable and non-empty', async () => {
  const baseSchedule = buildSyntheticSchedule(STAFF);
  const apiSchedule = makeApiSchedule();

  mockFetch(apiSchedule);

  const result = await redistributeWithLocks(STAFF, baseSchedule, new Set(), apiSchedule, {}, 'test-token');

  expect(typeof result.log.operationId).toBe('string');
  expect(result.log.operationId.length).toBeGreaterThan(0);

  // All ChangeRecords reference the same operationId
  for (const record of result.log.changeRecords) {
    expect(record.redistributionOperationId).toBe(result.log.operationId);
  }
});

test('provenance map covers every cell in the schedule', async () => {
  const baseSchedule = buildSyntheticSchedule(STAFF);
  const apiSchedule = makeApiSchedule();
  const lockedCells = new Set(['Alice:0']);
  const editedSchedule = { ...baseSchedule, Alice: [...baseSchedule['Alice']] };
  editedSchedule['Alice'][0] = 'Night';

  mockFetch(apiSchedule);

  const result = await redistributeWithLocks(STAFF, editedSchedule, lockedCells, apiSchedule, {}, 'test-token');

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

// ─── P3.3-CR: API path verification ──────────────────────────────────────────

test('P3.3-CR: POST /api/reschedule is called with existing_assignments and locked_shift_ids', async () => {
  const baseSchedule = buildSyntheticSchedule(STAFF);
  const apiSchedule = makeApiSchedule();
  // Lock Alice day 0 — shift 1 (Mon Early, dayOfWeek=0) is assigned to Alice (worker 1)
  const lockedCells = new Set(['Alice:0']);

  mockFetch(apiSchedule);

  await redistributeWithLocks(STAFF, baseSchedule, lockedCells, apiSchedule, {}, 'test-token');

  // Verify fetch was called with /api/reschedule
  const fetchMock = vi.mocked(fetch);
  const rescheduleCall = fetchMock.mock.calls.find(
    (call: unknown[]) => call[0] === '/api/reschedule'
  );
  expect(rescheduleCall).toBeDefined();

  const body = JSON.parse((rescheduleCall![1] as RequestInit).body as string);
  // existing_assignments must be present
  expect(body.existing_assignments).toBeDefined();
  // locked_shift_ids must include shift 1 (Alice day 0 = Mon = dayOfWeek 0 = shift 1)
  expect(body.locked_shift_ids).toContain(1);
});

test('P3.3-CR: no locked_shift_ids when no cells are locked', async () => {
  const baseSchedule = buildSyntheticSchedule(STAFF);
  const apiSchedule = makeApiSchedule();

  mockFetch(apiSchedule);

  await redistributeWithLocks(STAFF, baseSchedule, new Set(), apiSchedule, {}, 'test-token');

  const fetchMock = vi.mocked(fetch);
  const rescheduleCall = fetchMock.mock.calls.find(
    (call: unknown[]) => call[0] === '/api/reschedule'
  );
  expect(rescheduleCall).toBeDefined();

  const body = JSON.parse((rescheduleCall![1] as RequestInit).body as string);
  expect(body.locked_shift_ids).toBeNull();
});

test('AT-05: 28-day exact shift locking does not collapse across weeks', () => {
  const payload = buildSchedulePayload(STAFF, {});
  // Payload should have 196 shifts (28 days * 7 canonical hospital shift slots)
  expect(payload.shifts.length).toBe(196);

  // Map 28-day apiSchedule: shift 1..20
  const fullApiSchedule: Record<string, number> = {};
  payload.shifts.forEach((s, idx) => {
    fullApiSchedule[String(s.id)] = (idx % STAFF.length) + 1;
  });

  // Week 2 Tuesday is dayIdx = 2 * 7 + 1 = 15
  const slotMeta = getScheduleSlotMeta(STAFF);
  const week2TueShift = Object.values(slotMeta).find(m => m.dayIdx === 15);
  expect(week2TueShift).toBeDefined();

  const workerOnWeek2Tue = fullApiSchedule[String(week2TueShift!.shiftId)];
  const staffMember = STAFF[workerOnWeek2Tue - 1];

  // Lock ONLY Week 2 Tuesday
  const lockedCells = new Set([`${staffMember.id}:15`]);
  const lockedIds = buildLockedShiftIds(STAFF, fullApiSchedule, lockedCells);

  // MUST contain exactly the shift for day 15, and NOT shifts for day 1, 8, or 22!
  expect(lockedIds).toContain(week2TueShift!.shiftId);
  expect(lockedIds.length).toBe(1);

  const week0TueShift = Object.values(slotMeta).find(m => m.dayIdx === 1);
  const week1TueShift = Object.values(slotMeta).find(m => m.dayIdx === 8);
  const week3TueShift = Object.values(slotMeta).find(m => m.dayIdx === 22);
  expect(lockedIds).not.toContain(week0TueShift!.shiftId);
  expect(lockedIds).not.toContain(week1TueShift!.shiftId);
  expect(lockedIds).not.toContain(week3TueShift!.shiftId);
});

// ─── AT-36: Single Manual Edit Regression Suite ─────────────────────────────

test('AT-36.1: Verified sample roster initial state is 100% conserved with zero unmapped slots', () => {
  const { staff } = parseStaffCSV(SAMPLE_CSV);
  const slotMeta = getScheduleSlotMeta(staff);
  const baseSchedule = buildEditableSchedule(staff, VERIFIED_SAMPLE_SCHEDULE, slotMeta);

  const conversion = scheduleToAssignments(staff, baseSchedule, slotMeta);
  expect(conversion.totalUiAssignments).toBe(196);
  expect(conversion.totalMappedAssignments).toBe(196);
  expect(conversion.isConserved).toBe(true);
  expect(conversion.unmappedUiAssignments).toEqual([]);
  expect(conversion.duplicateAssignments).toEqual([]);

  const reqs = buildStaffingRequirements(slotMeta);
  const cov = computeCanonicalCoverage(reqs, baseSchedule);
  expect(cov.coveragePct).toBe(100);
  expect(cov.gapPositions).toBe(0);
});

test('AT-36.2: 1 manual edit followed by redistribution preserves >= 95% of roster and zero hard violations', async () => {
  const { staff } = parseStaffCSV(SAMPLE_CSV);
  const slotMeta = getScheduleSlotMeta(staff);
  const baseSchedule = buildEditableSchedule(staff, VERIFIED_SAMPLE_SCHEDULE, slotMeta);

  // Edit Tom_Lindqvist on Day 2 to 'Night'
  const editedSchedule: Record<string, string[]> = {};
  for (const s of staff) {
    editedSchedule[s.id] = [...(baseSchedule[s.id] || [])];
  }
  editedSchedule['Tom_Lindqvist'][2] = 'Night';
  const lockedCells = new Set(['Tom_Lindqvist:2']);

  // Mock API returning failure or cold-scramble to trigger quality gate fallback
  mockFetch({});

  const result = await redistributeWithLocks(
    staff,
    editedSchedule,
    lockedCells,
    VERIFIED_SAMPLE_SCHEDULE,
    {},
    'test-csrf-token',
    slotMeta,
  );

  // Invariant 1: Locked scheduler edit MUST be preserved
  expect(result.schedule['Tom_Lindqvist'][2]).toBe('Night');
  expect(result.log.lockedAssignmentsChanged).toBe(0);
  expect(result.log.schedulerEditsPreserved).toBe(1);

  // Invariant 2: Minimal churn — 1 edit must NOT reassign hundreds of shifts (user saw 409!)
  expect(result.changedCount).toBeLessThanOrEqual(4);
  expect(result.unchangedCount).toBeGreaterThanOrEqual(556);

  // Invariant 3: Full conservation of demand coverage
  const reqs = buildStaffingRequirements(slotMeta);
  const cov = computeCanonicalCoverage(reqs, result.schedule);
  expect(cov.coveragePct).toBe(100);
  expect(cov.gapPositions).toBe(0);
  expect(cov.filledPositions).toBe(196);

  // Invariant 4: No unmapped assignments or duplicate assignments
  const conversion = scheduleToAssignments(staff, result.schedule, slotMeta);
  expect(conversion.isConserved).toBe(true);
  expect(conversion.unmappedUiAssignments).toEqual([]);
  expect(conversion.duplicateAssignments).toEqual([]);
});

test('AT-36.3: buildLockedShiftIds correctly maps user manual edits from currentSchedule', () => {
  const { staff } = parseStaffCSV(SAMPLE_CSV);
  const slotMeta = getScheduleSlotMeta(staff);
  const baseSchedule = buildEditableSchedule(staff, VERIFIED_SAMPLE_SCHEDULE, slotMeta);

  // Edit Tom_Lindqvist on Day 2 to 'Night'
  const editedSchedule: Record<string, string[]> = {};
  for (const s of staff) {
    editedSchedule[s.id] = [...(baseSchedule[s.id] || [])];
  }
  editedSchedule['Tom_Lindqvist'][2] = 'Night';
  const lockedCells = new Set(['Tom_Lindqvist:2']);

  const lockedIds = buildLockedShiftIds(staff, {}, lockedCells, slotMeta, editedSchedule);
  expect(lockedIds.length).toBe(1);
  const lockedMeta = slotMeta[lockedIds[0]];
  expect(lockedMeta).toBeDefined();
  expect(lockedMeta.dayIdx).toBe(2);
  expect(lockedMeta.label).toBe('Night');
});

test('AT-36.4: Backend preservation of manually locked assignments and protection of last valid schedule', async () => {
  const { staff } = parseStaffCSV(SAMPLE_CSV);
  const slotMeta = getScheduleSlotMeta(staff);
  const baseSchedule = buildEditableSchedule(staff, VERIFIED_SAMPLE_SCHEDULE, slotMeta);

  // 1. Manually reassign Tom_Lindqvist on Day 2 to 'Night'
  const editedSchedule: Record<string, string[]> = {};
  for (const s of staff) {
    editedSchedule[s.id] = [...(baseSchedule[s.id] || [])];
  }
  editedSchedule['Tom_Lindqvist'][2] = 'Night';
  const lockedCells = new Set(['Tom_Lindqvist:2']);

  // Find the exact shift ID corresponding to Tom's edit on Day 2
  const activeConversion = scheduleToAssignments(staff, editedSchedule, slotMeta);
  const editedShiftId = Object.entries(activeConversion.assignments).find(
    ([sid, wid]) => wid === 6 && slotMeta[Number(sid)]?.dayIdx === 2
  )?.[0];
  expect(editedShiftId).toBeDefined();

  // 2. Mock /api/reschedule returning a response where the locked shift is preserved
  const mockApiReturn = { ...activeConversion.assignments };
  mockApiReturn[editedShiftId!] = 6; // strictly preserved
  mockFetch(mockApiReturn);

  const result = await redistributeWithLocks(
    staff,
    editedSchedule,
    lockedCells,
    VERIFIED_SAMPLE_SCHEDULE,
    {},
    'test-csrf-token',
    slotMeta,
  );

  // 3. Verify outgoing payload: ONLY the manually edited shift is in locked_shift_ids (not all 196!)
  const fetchMock = vi.mocked(fetch);
  const rescheduleCall = fetchMock.mock.calls.find(
    (call: unknown[]) => call[0] === '/api/reschedule'
  );
  expect(rescheduleCall).toBeDefined();
  const requestBody = JSON.parse((rescheduleCall![1] as RequestInit).body as string);
  expect(requestBody.locked_shift_ids).toEqual([Number(editedShiftId)]);
  expect(requestBody.locked_shift_ids.length).toBe(1);

  // 4. Invariant: manually edited (worker, shift) remains unchanged
  expect(result.schedule['Tom_Lindqvist'][2]).toBe('Night');
  expect(result.log.lockedAssignmentsChanged).toBe(0);
  expect(result.log.schedulerEditsPreserved).toBe(1);

  // 5. Independently validate returned schedule
  const finalConversion = scheduleToAssignments(staff, result.schedule, slotMeta);
  expect(finalConversion.isConserved).toBe(true);
  expect(finalConversion.unmappedUiAssignments).toEqual([]);
  expect(finalConversion.duplicateAssignments).toEqual([]);

  const reqs = buildStaffingRequirements(slotMeta);
  const cov = computeCanonicalCoverage(reqs, result.schedule);
  expect(cov.coveragePct).toBe(100);
  expect(cov.gapPositions).toBe(0);

  // 6. Protection check: If the backend throws or returns an error, the schedule is protected
  vi.stubGlobal('fetch', vi.fn().mockImplementation((url: string) => {
    if (url === '/api/reschedule') {
      return Promise.resolve({ ok: false, status: 500 });
    }
    return Promise.resolve({ ok: true, json: () => Promise.resolve({ csrf_token: 'tok' }) });
  }));

  const protectedResult = await redistributeWithLocks(
    staff,
    editedSchedule,
    lockedCells,
    VERIFIED_SAMPLE_SCHEDULE,
    {},
    'test-csrf-token',
    slotMeta,
  );

  // The local preservation fallback ensures the scheduler's edit and coverage are safely protected
  expect(protectedResult.schedule['Tom_Lindqvist'][2]).toBe('Night');
  expect(protectedResult.log.lockedAssignmentsChanged).toBe(0);
  const protectedCov = computeCanonicalCoverage(reqs, protectedResult.schedule);
  expect(protectedCov.coveragePct).toBe(100);
  expect(protectedCov.gapPositions).toBe(0);
});

