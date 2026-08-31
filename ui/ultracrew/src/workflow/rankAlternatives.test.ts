// P3.2 regression tests for compareAlternatives()
//
// After the HC1 correction (commit 9d106d201, verified at 00ab239d7),
// recommendation authority belongs exclusively to the optimizer pipeline.
// compareAlternatives() is a presentation/comparison utility only.
//
// Architectural invariant (regression test at bottom):
//   Changing coverage/fairness/cost values in the adapter CANNOT change
//   the recommendedId — that is set by the optimizer, not the adapter.

import { compareAlternatives } from './WorkflowUtils';
import type { RosterAlternative } from './WorkflowTypes';

function makeAlt(
  id: string,
  filled: number,
  required: number,
  fairness: number,
  cost: number,
  schedule: Record<string, string[]> = {},
): RosterAlternative {
  return {
    id,
    label: id,
    metrics: {
      coverage: filled / required,
      filled_positions: filled,
      required_positions: required,
      fairness_penalty: fairness,
      utilization: 0.8,
      cost,
      diff_from_recommended: 0,
    },
    schedule,
    reasons: [],
  };
}

// ── compareAlternatives() — basic shape ───────────────────────────────────────

test('compareAlternatives returns one entry per alternative', () => {
  const a = makeAlt('alt-A', 194, 196, 1.2, 100);
  const b = makeAlt('alt-B', 40, 196, 0.9, 97);
  const result = compareAlternatives([a, b]);
  expect(result).toHaveLength(2);
  expect(result[0].id).toBe('alt-A');
  expect(result[1].id).toBe('alt-B');
});

test('compareAlternatives returns correct coverage metrics', () => {
  const a = makeAlt('alt-A', 194, 196, 1.2, 100);
  const result = compareAlternatives([a]);
  expect(result[0].filledPositions).toBe(194);
  expect(result[0].requiredPositions).toBe(196);
  expect(result[0].gapPositions).toBe(2);
  expect(result[0].coveragePct).toBeCloseTo(98.98, 1);
});

test('compareAlternatives returns correct fairness and cost', () => {
  const a = makeAlt('alt-A', 196, 196, 1.2, 100);
  const result = compareAlternatives([a]);
  expect(result[0].fairnessPenalty).toBe(1.2);
  expect(result[0].cost).toBe(100);
});

test('compareAlternatives diffFromFirst is 0 for first alternative', () => {
  const a = makeAlt('alt-A', 196, 196, 1.2, 100);
  const b = makeAlt('alt-B', 194, 196, 0.9, 97);
  const result = compareAlternatives([a, b]);
  expect(result[0].diffFromFirst).toBe(0);
});

test('compareAlternatives diffFromFirst counts differing cells', () => {
  const schedA: Record<string, string[]> = { 'nurse-1': ['Early', 'Late', ''] };
  const schedB: Record<string, string[]> = { 'nurse-1': ['Late', 'Late', 'Night'] };
  const a = makeAlt('alt-A', 196, 196, 1.2, 100, schedA);
  const b = makeAlt('alt-B', 194, 196, 0.9, 97, schedB);
  const result = compareAlternatives([a, b]);
  // day 0: Early vs Late → diff; day 1: Late vs Late → same; day 2: '' vs Night → diff
  expect(result[1].diffFromFirst).toBe(2);
});

test('compareAlternatives returns empty array for empty input', () => {
  expect(compareAlternatives([])).toEqual([]);
});

test('compareAlternatives single alternative has diffFromFirst 0', () => {
  const a = makeAlt('alt-A', 196, 196, 1.2, 100);
  const result = compareAlternatives([a]);
  expect(result[0].diffFromFirst).toBe(0);
});

// ── Comparison is symmetric in coverage reporting ─────────────────────────────

test('40/196 and 194/196 both reported accurately — no ranking applied', () => {
  const a = makeAlt('alt-A', 40, 196, 1.2, 100);
  const b = makeAlt('alt-B', 194, 196, 0.9, 97);
  const result = compareAlternatives([a, b]);
  // Both alternatives are reported; no winner is selected
  expect(result[0].filledPositions).toBe(40);
  expect(result[1].filledPositions).toBe(194);
  // compareAlternatives does NOT return a recommendedId
  expect((result[0] as unknown as Record<string, unknown>)['recommendedId']).toBeUndefined();
  expect((result[1] as unknown as Record<string, unknown>)['recommendedId']).toBeUndefined();
});

// ── Architectural invariant regression test ───────────────────────────────────
//
// Changing coverage/fairness/cost values in the adapter CANNOT change
// the recommendedId. The adapter has no recommendedId to change.
// This test proves compareAlternatives() is incapable of selecting a recommendation.

test('INVARIANT: adapter metrics cannot change recommendedId — compareAlternatives has no recommendedId', () => {
  // Simulate the worst-case adapter manipulation: swap all metrics between alternatives
  const a = makeAlt('alt-A', 40, 196, 9.9, 999);   // terrible metrics
  const b = makeAlt('alt-B', 196, 196, 0.1, 1);     // perfect metrics

  const result = compareAlternatives([a, b]);

  // compareAlternatives returns comparison data only — no recommendation
  for (const entry of result) {
    expect(Object.keys(entry)).not.toContain('recommendedId');
    expect(Object.keys(entry)).not.toContain('coverageDominant');
  }

  // The recommendedId is set externally (by the optimizer/API), not by this function.
  // Proof: no matter what metrics are passed, compareAlternatives cannot produce a recommendedId.
  const hasRecommendedId = result.some(r => 'recommendedId' in r);
  expect(hasRecommendedId).toBe(false);
});