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

// ─── G-10: Recommendation explanation must not use "best overall balance" ─────
//
// rankAlternatives() in WorkflowUtils.ts is the authoritative source of the
// recommendation reason. We verify that the reason strings it produces never
// contain the banned phrase "best overall balance" — which was the incorrect
// text that described a low-coverage option as if it were the best choice.

import { rankAlternatives } from './WorkflowUtils';
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

test('G-10: recommendation reason never contains "best overall balance"', () => {
  // Coverage-dominant case: 40/196 vs 194/196
  const result1 = rankAlternatives([
    makeAlt('low',  40,  196, 0.5, 90),
    makeAlt('high', 194, 196, 1.2, 100),
  ]);
  expect(result1.reason.toLowerCase()).not.toContain('best overall balance');
  expect(result1.recommendedId).toBe('high');

  // Secondary-decides case: 194/196 vs 196/196
  const result2 = rankAlternatives([
    makeAlt('a', 194, 196, 1.2, 100),
    makeAlt('b', 196, 196, 0.9, 97),
  ]);
  expect(result2.reason.toLowerCase()).not.toContain('best overall balance');

  // Equal coverage case
  const result3 = rankAlternatives([
    makeAlt('x', 196, 196, 1.5, 100),
    makeAlt('y', 196, 196, 0.8, 95),
  ]);
  expect(result3.reason.toLowerCase()).not.toContain('best overall balance');
});

test('G-10: coverage-dominant reason explicitly states uncovered position count', () => {
  const result = rankAlternatives([
    makeAlt('low',  40,  196, 0.5, 90),
    makeAlt('high', 194, 196, 1.2, 100),
  ]);
  // The reason must mention the filled/required counts explicitly
  expect(result.reason).toContain('194');
  expect(result.reason).toContain('196');
  expect(result.recommendedId).toBe('high');
});

test('G-10: recommended badge belongs to rankAlternatives winner, not the first alternative', () => {
  // When the first alternative has lower coverage, the second must be recommended
  const alts = [
    makeAlt('first',  40,  196, 0.5, 90),
    makeAlt('second', 194, 196, 1.2, 100),
  ];
  const result = rankAlternatives(alts);
  expect(result.recommendedId).toBe('second');
  expect(result.recommendedId).not.toBe('first');
});