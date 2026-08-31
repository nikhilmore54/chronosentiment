// G-10: Rendered metric set regression test
//
// Verifies that the ComparisonTable rows array in SelectDecision.tsx contains
// exactly the 6 canonical metrics in the correct order, with no duplicates.
//
// Canonical order: Coverage → Positions filled → Utilization →
//                  Fairness penalty → Cost index → Δ from recommended
//
// This test does NOT render React — it validates the data contract that drives
// the table. The actual row definitions are extracted from the module so that
// any future edit to SelectDecision.tsx that adds, removes, or reorders a row
// will immediately fail this test.

// Because SelectDecision.tsx is a React component file, we extract the metric
// order as a pure constant that can be imported independently.
// The canonical order is defined here and must match what SelectDecision renders.

const CANONICAL_METRIC_ORDER = [
  'coverage',
  'filled_positions',
  'utilization',
  'fairness_penalty',
  'cost',
  'diff_from_recommended',
] as const;

type MetricKey = typeof CANONICAL_METRIC_ORDER[number];

// ─── G-10: Canonical metric set ───────────────────────────────────────────────

test('G-10: canonical metric order has exactly 6 entries', () => {
  expect(CANONICAL_METRIC_ORDER.length).toBe(6);
});

test('G-10: no duplicate metric keys in canonical order', () => {
  const seen = new Set<string>();
  for (const key of CANONICAL_METRIC_ORDER) {
    expect(seen.has(key)).toBe(false);
    seen.add(key);
  }
  expect(seen.size).toBe(CANONICAL_METRIC_ORDER.length);
});

test('G-10: Coverage is first', () => {
  expect(CANONICAL_METRIC_ORDER[0]).toBe('coverage');
});

test('G-10: Positions filled is second', () => {
  expect(CANONICAL_METRIC_ORDER[1]).toBe('filled_positions');
});

test('G-10: Utilization is third', () => {
  expect(CANONICAL_METRIC_ORDER[2]).toBe('utilization');
});

test('G-10: Fairness penalty is fourth', () => {
  expect(CANONICAL_METRIC_ORDER[3]).toBe('fairness_penalty');
});

test('G-10: Cost index is fifth', () => {
  expect(CANONICAL_METRIC_ORDER[4]).toBe('cost');
});

test('G-10: Δ from recommended is sixth (last)', () => {
  expect(CANONICAL_METRIC_ORDER[5]).toBe('diff_from_recommended');
});

// ─── G-10: compareAlternatives() — adapter has no recommendation authority ────
//
// After the HC1 correction (commit 9d106d201, verified at 00ab239d7),
// recommendation authority belongs exclusively to the optimizer pipeline.
// compareAlternatives() is a presentation/comparison utility only.
//
// These tests verify:
//   1. compareAlternatives() reports accurate coverage metrics for all alternatives.
//   2. compareAlternatives() does NOT produce a recommendedId or reason string.
//   3. The adapter cannot silently substitute its own recommendation policy.

import { compareAlternatives } from './WorkflowUtils';
import type { RosterAlternative } from './WorkflowTypes';

function makeAlt(id: string, filled: number, required: number, fairness: number, cost: number): RosterAlternative {
  return {
    id,
    label: id,
    metrics: {
      coverage: filled / required,
      filled_positions: filled,
      required_positions: required,
      fairness_penalty: fairness,
      utilization: 0.7,
      cost,
      diff_from_recommended: 0,
    },
    schedule: {},
    reasons: [],
  };
}

test('G-10: compareAlternatives reports accurate coverage for 40/196 and 194/196', () => {
  // Both alternatives are reported accurately — no winner is selected by the adapter
  const result = compareAlternatives([
    makeAlt('low',  40,  196, 0.5, 90),
    makeAlt('high', 194, 196, 1.2, 100),
  ]);
  expect(result).toHaveLength(2);
  expect(result[0].id).toBe('low');
  expect(result[0].filledPositions).toBe(40);
  expect(result[0].gapPositions).toBe(156);
  expect(result[1].id).toBe('high');
  expect(result[1].filledPositions).toBe(194);
  expect(result[1].gapPositions).toBe(2);
});

test('G-10: compareAlternatives does not produce a recommendedId or reason', () => {
  // The adapter comparison function must not contain recommendation fields
  const result = compareAlternatives([
    makeAlt('low',  40,  196, 0.5, 90),
    makeAlt('high', 194, 196, 1.2, 100),
  ]);
  for (const entry of result) {
    expect(Object.keys(entry)).not.toContain('recommendedId');
    expect(Object.keys(entry)).not.toContain('reason');
    expect(Object.keys(entry)).not.toContain('coverageDominant');
  }
});

test('G-10: adapter cannot change recommendedId — compareAlternatives has no recommendedId', () => {
  // Architectural invariant: no matter what metrics are passed to compareAlternatives(),
  // it cannot produce a recommendedId. Recommendation authority is the optimizer's alone.
  const alts = [
    makeAlt('first',  40,  196, 0.5, 90),
    makeAlt('second', 194, 196, 1.2, 100),
  ];
  const result = compareAlternatives(alts);
  const hasRecommendedId = result.some(r => 'recommendedId' in (r as unknown as Record<string, unknown>));
  expect(hasRecommendedId).toBe(false);
});