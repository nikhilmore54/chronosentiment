// @vitest-environment jsdom
/**
 * P4.1 Gate Tests — G-P4-B-1 through G-P4-B-7
 *
 * Acceptance gates for accumulatePatterns() as defined in
 * docs/ULTRAROSTER_P4_AUTHORIZATION_PROPOSAL.md
 *
 * Architectural invariant: P3 factual history → P4.1 pattern observation →
 * Scheduler human interpretation → Future P4 authorization (possible influence,
 * not yet authorized). accumulatePatterns() is a pure read-only function.
 */

import { describe, it, expect } from 'vitest';
import { accumulatePatterns } from './PatternAccumulator';
import type { RedistributionLog, ChangeRecord } from '../workflow/WorkflowTypes';

// ─── Helpers ─────────────────────────────────────────────────────────────────

function makeChangeRecord(reason: string, operationId: string): ChangeRecord {
  return {
    assignmentId: 'member-1:0',
    previousValue: 'Early',
    newValue: 'Late',
    redistributionOperationId: operationId,
    reason,
    timestamp: '2026-01-01T00:00:00.000Z',
  };
}

function makeLog(
  operationId: string,
  reasons: string[],
  timestamp = '2026-01-01T00:00:00.000Z',
): RedistributionLog {
  return {
    operationId,
    timestamp,
    schedulerEditsPreserved: 0,
    assignmentsReassigned: reasons.length,
    lockedAssignmentsChanged: 0,
    changeRecords: reasons.map(reason => makeChangeRecord(reason, operationId)),
    provenanceMap: {},
  };
}

// ─── G-P4-B-1: Empty logs → no patterns ──────────────────────────────────────

describe('G-P4-B-1: empty logs produce no patterns', () => {
  it('returns empty array for empty log record', () => {
    const result = accumulatePatterns({});
    expect(result).toEqual([]);
  });
});

// ─── G-P4-B-2: Below threshold → no patterns ─────────────────────────────────

describe('G-P4-B-2: reasons below threshold are not surfaced', () => {
  it('does not surface a reason appearing in only 2 distinct operations (threshold=3)', () => {
    const logs: Record<string, RedistributionLog> = {
      op1: makeLog('op-1', ['understaffed_shift']),
      op2: makeLog('op-2', ['understaffed_shift']),
    };
    const result = accumulatePatterns(logs, 3);
    expect(result).toEqual([]);
  });

  it('does not surface a reason appearing in exactly threshold-1 operations', () => {
    const logs: Record<string, RedistributionLog> = {
      op1: makeLog('op-1', ['coverage_gap']),
      op2: makeLog('op-2', ['coverage_gap']),
    };
    const result = accumulatePatterns(logs, 3);
    expect(result.find(p => p.reason === 'coverage_gap')).toBeUndefined();
  });
});

// ─── G-P4-B-3: Independence by operationId ───────────────────────────────────

describe('G-P4-B-3: operationId independence — same reason in same operation counts once', () => {
  it('counts a reason appearing multiple times in one operation as a single operationId', () => {
    // 'understaffed_shift' appears 5 times in op-1 but only 1 distinct operationId
    const logs: Record<string, RedistributionLog> = {
      op1: makeLog('op-1', [
        'understaffed_shift',
        'understaffed_shift',
        'understaffed_shift',
        'understaffed_shift',
        'understaffed_shift',
      ]),
    };
    const result = accumulatePatterns(logs, 3);
    // Only 1 distinct operationId — must NOT surface (below threshold of 3)
    expect(result).toEqual([]);
  });

  it('surfaces a reason only when it appears in ≥threshold distinct operationIds', () => {
    const logs: Record<string, RedistributionLog> = {
      op1: makeLog('op-1', ['understaffed_shift', 'understaffed_shift']),
      op2: makeLog('op-2', ['understaffed_shift']),
      op3: makeLog('op-3', ['understaffed_shift']),
    };
    const result = accumulatePatterns(logs, 3);
    const pattern = result.find(p => p.reason === 'understaffed_shift');
    expect(pattern).toBeDefined();
    expect(pattern!.operationCount).toBe(3);
  });
});

// ─── G-P4-B-4: Correct operationCount ────────────────────────────────────────

describe('G-P4-B-4: operationCount reflects distinct operationId count', () => {
  it('reports operationCount = number of distinct operationIds containing the reason', () => {
    const logs: Record<string, RedistributionLog> = {
      op1: makeLog('op-1', ['night_coverage_gap']),
      op2: makeLog('op-2', ['night_coverage_gap']),
      op3: makeLog('op-3', ['night_coverage_gap']),
      op4: makeLog('op-4', ['night_coverage_gap']),
    };
    const result = accumulatePatterns(logs, 3);
    const pattern = result.find(p => p.reason === 'night_coverage_gap');
    expect(pattern).toBeDefined();
    expect(pattern!.operationCount).toBe(4);
  });
});

// ─── G-P4-B-5: firstSeen / lastSeen timestamps ───────────────────────────────

describe('G-P4-B-5: firstSeen and lastSeen are correct', () => {
  it('sets firstSeen to earliest and lastSeen to latest timestamp across matching operations', () => {
    const logs: Record<string, RedistributionLog> = {
      op1: makeLog('op-1', ['shift_imbalance'], '2026-01-03T00:00:00.000Z'),
      op2: makeLog('op-2', ['shift_imbalance'], '2026-01-01T00:00:00.000Z'),
      op3: makeLog('op-3', ['shift_imbalance'], '2026-01-05T00:00:00.000Z'),
    };
    const result = accumulatePatterns(logs, 3);
    const pattern = result.find(p => p.reason === 'shift_imbalance');
    expect(pattern).toBeDefined();
    expect(pattern!.firstSeen).toBe('2026-01-01T00:00:00.000Z');
    expect(pattern!.lastSeen).toBe('2026-01-05T00:00:00.000Z');
  });
});

// ─── G-P4-B-6: Sorted by operationCount descending ───────────────────────────

describe('G-P4-B-6: results sorted by operationCount descending', () => {
  it('returns patterns in descending operationCount order', () => {
    const logs: Record<string, RedistributionLog> = {
      // 'rare_reason' in 3 ops
      op1: makeLog('op-1', ['rare_reason']),
      op2: makeLog('op-2', ['rare_reason']),
      op3: makeLog('op-3', ['rare_reason']),
      // 'common_reason' in 5 ops
      op4: makeLog('op-4', ['common_reason']),
      op5: makeLog('op-5', ['common_reason']),
      op6: makeLog('op-6', ['common_reason']),
      op7: makeLog('op-7', ['common_reason']),
      op8: makeLog('op-8', ['common_reason']),
    };
    const result = accumulatePatterns(logs, 3);
    expect(result.length).toBe(2);
    expect(result[0].reason).toBe('common_reason');
    expect(result[0].operationCount).toBe(5);
    expect(result[1].reason).toBe('rare_reason');
    expect(result[1].operationCount).toBe(3);
  });
});

// ─── G-P4-B-7: Read-only — no mutation of input ──────────────────────────────

describe('G-P4-B-7: accumulatePatterns does not mutate input logs', () => {
  it('leaves the input logs record unchanged after accumulation', () => {
    const logs: Record<string, RedistributionLog> = {
      op1: makeLog('op-1', ['immutable_reason']),
      op2: makeLog('op-2', ['immutable_reason']),
      op3: makeLog('op-3', ['immutable_reason']),
    };
    const originalKeys = Object.keys(logs);
    const originalChangeCount = logs['op1'].changeRecords.length;

    accumulatePatterns(logs, 3);

    expect(Object.keys(logs)).toEqual(originalKeys);
    expect(logs['op1'].changeRecords.length).toBe(originalChangeCount);
    expect(logs['op1'].changeRecords[0].reason).toBe('immutable_reason');
  });
});
