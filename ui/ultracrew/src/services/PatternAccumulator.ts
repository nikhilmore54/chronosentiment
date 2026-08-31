import type { RedistributionLog, RecurringPattern } from '../workflow/WorkflowTypes';

/**
 * P4.1 — Recurring Operational Pattern Surfacing
 *
 * Pure function. No side effects. No optimizer changes. No adaptive weights.
 *
 * Groups ChangeRecord.reason values across distinct operationIds.
 * Returns patterns that appear in at least `threshold` independent operations,
 * sorted by operationCount descending.
 *
 * Independence criterion: distinct operationId (not scheduler identity,
 * which is not in the current data model — see G-P4-B-3).
 */
export function accumulatePatterns(
  logs: Record<string, RedistributionLog>,
  threshold: number = 3,
): RecurringPattern[] {
  // reason → Set of distinct operationIds containing that reason
  const reasonToOps = new Map<string, Set<string>>();
  // reason → all timestamps from logs containing that reason (for firstSeen/lastSeen)
  const reasonToTimestamps = new Map<string, string[]>();

  for (const log of Object.values(logs)) {
    for (const change of log.changeRecords) {
      if (!reasonToOps.has(change.reason)) {
        reasonToOps.set(change.reason, new Set());
        reasonToTimestamps.set(change.reason, []);
      }
      reasonToOps.get(change.reason)!.add(log.operationId);
      reasonToTimestamps.get(change.reason)!.push(log.timestamp);
    }
  }

  const patterns: RecurringPattern[] = [];
  for (const [reason, ops] of reasonToOps.entries()) {
    if (ops.size >= threshold) {
      const timestamps = [...reasonToTimestamps.get(reason)!].sort();
      patterns.push({
        reason,
        operationCount: ops.size,
        firstSeen: timestamps[0],
        lastSeen: timestamps[timestamps.length - 1],
      });
    }
  }

  // Sort by operationCount descending (most recurring first)
  return patterns.sort((a, b) => b.operationCount - a.operationCount);
}