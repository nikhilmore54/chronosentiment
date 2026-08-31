// @vitest-environment jsdom
import { describe, it, expect, beforeEach } from 'vitest';
import { DecisionRepository } from './DecisionRepository';
import type { RedistributionLog, ChangeRecord } from '../workflow/WorkflowTypes';

// ── Helpers ──────────────────────────────────────────────────────────────────

function makeLog(operationId: string): RedistributionLog {
  const change: ChangeRecord = {
    assignmentId: 'nurse-1:3',
    previousValue: 'Early',
    newValue: 'Night',
    redistributionOperationId: operationId,
    reason: 'coverage gap on day 3 Night shift',
    timestamp: new Date().toISOString(),
  };
  return {
    operationId,
    timestamp: new Date().toISOString(),
    schedulerEditsPreserved: 2,
    assignmentsReassigned: 1,
    lockedAssignmentsChanged: 0,
    changeRecords: [change],
    provenanceMap: {
      'nurse-1:3': 'system_reassignment',
      'nurse-2:3': 'unchanged',
    },
  };
}

// ── P3.3 persistence tests (hard gate 5) ─────────────────────────────────────

describe('DecisionRepository — P3.3 redistribution log persistence (hard gate 5)', () => {
  let repo: DecisionRepository;

  beforeEach(() => {
    localStorage.clear();
    repo = new DecisionRepository();
  });

  it('G5-1: loadRedistributionLog returns null when no log has been saved', () => {
    expect(repo.loadRedistributionLog('p3_nonexistent')).toBeNull();
  });

  it('G5-2: saveRedistributionLog persists the log; loadRedistributionLog retrieves it by decision_id', () => {
    const log = makeLog('op-abc-001');
    repo.saveRedistributionLog('p3_decision_001', log);

    const loaded = repo.loadRedistributionLog('p3_decision_001');
    expect(loaded).not.toBeNull();
    expect(loaded!.operationId).toBe('op-abc-001');
    expect(loaded!.schedulerEditsPreserved).toBe(2);
    expect(loaded!.assignmentsReassigned).toBe(1);
    expect(loaded!.lockedAssignmentsChanged).toBe(0);
    expect(loaded!.changeRecords).toHaveLength(1);
    expect(loaded!.changeRecords[0].assignmentId).toBe('nurse-1:3');
    expect(loaded!.changeRecords[0].reason).toBe('coverage gap on day 3 Night shift');
    expect(loaded!.provenanceMap['nurse-1:3']).toBe('system_reassignment');
  });

  it('G5-3: logs for different decision_ids are stored independently', () => {
    const log1 = makeLog('op-001');
    const log2 = makeLog('op-002');
    repo.saveRedistributionLog('p3_decision_001', log1);
    repo.saveRedistributionLog('p3_decision_002', log2);

    expect(repo.loadRedistributionLog('p3_decision_001')!.operationId).toBe('op-001');
    expect(repo.loadRedistributionLog('p3_decision_002')!.operationId).toBe('op-002');
    expect(repo.loadRedistributionLog('p3_decision_003')).toBeNull();
  });

  it('G5-4: saveRedistributionLog is idempotent — re-saving overwrites the previous log', () => {
    const log1 = makeLog('op-original');
    const log2 = makeLog('op-updated');
    repo.saveRedistributionLog('p3_decision_001', log1);
    repo.saveRedistributionLog('p3_decision_001', log2);

    const loaded = repo.loadRedistributionLog('p3_decision_001');
    expect(loaded!.operationId).toBe('op-updated');
  });

  it('G5-5: clear() removes all redistribution logs', () => {
    repo.saveRedistributionLog('p3_decision_001', makeLog('op-001'));
    repo.saveRedistributionLog('p3_decision_002', makeLog('op-002'));
    repo.clear();

    expect(repo.loadRedistributionLog('p3_decision_001')).toBeNull();
    expect(repo.loadRedistributionLog('p3_decision_002')).toBeNull();
  });
});