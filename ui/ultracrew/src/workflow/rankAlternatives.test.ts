// P3.2 regression tests for rankAlternatives()
//
// Hard gate: 40/196 vs 194/196 → alt-B must be recommended.
// These tests must pass before any Render deployment of P3.2.

import { rankAlternatives } from './WorkflowUtils';
import type { RosterAlternative } from './WorkflowTypes';

function makeAlt(
  id: string,
  filled: number,
  required: number,
  fairness: number,
  cost: number,
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
    schedule: {},
    reasons: [],
  };
}

// ── Regression gate ───────────────────────────────────────────────────────────

test('40/196 vs 194/196 → 194/196 wins (coverage dominant)', () => {
  const a = makeAlt('alt-A', 40, 196, 1.2, 100);
  const b = makeAlt('alt-B', 194, 196, 0.9, 97);
  const result = rankAlternatives([a, b]);
  expect(result.recommendedId).toBe('alt-B');
  expect(result.coverageDominant).toBe(true);
});

// ── Tolerance boundary ────────────────────────────────────────────────────────

test('191/196 vs 196/196 → 196/196 wins (gap=5, boundary, secondary decides)', () => {
  // gap = 5 which equals GAP_TOLERANCE — both are candidates, secondary decides
  const a = makeAlt('alt-A', 191, 196, 1.2, 100);
  const b = makeAlt('alt-B', 196, 196, 0.9, 97);
  const result = rankAlternatives([a, b]);
  // Both within tolerance → secondary (fairness+cost) decides → B wins
  expect(result.recommendedId).toBe('alt-B');
  expect(result.coverageDominant).toBe(false);
});

test('190/196 vs 196/196 → 196/196 wins (gap=6 > tolerance, coverage dominant)', () => {
  const a = makeAlt('alt-A', 190, 196, 1.2, 100);
  const b = makeAlt('alt-B', 196, 196, 0.9, 97);
  const result = rankAlternatives([a, b]);
  expect(result.recommendedId).toBe('alt-B');
  expect(result.coverageDominant).toBe(true);
});

// ── Equal coverage → secondary objectives ────────────────────────────────────

test('194/196 vs 194/196 → lower fairness+cost wins', () => {
  const a = makeAlt('alt-A', 194, 196, 1.2, 100);
  const b = makeAlt('alt-B', 194, 196, 0.9, 97);
  const result = rankAlternatives([a, b]);
  expect(result.recommendedId).toBe('alt-B');
  expect(result.coverageDominant).toBe(false);
});

test('196/196 vs 196/196 equal secondary → first wins (stable)', () => {
  const a = makeAlt('alt-A', 196, 196, 1.2, 100);
  const b = makeAlt('alt-B', 196, 196, 1.2, 100);
  const result = rankAlternatives([a, b]);
  expect(result.recommendedId).toBe('alt-A');
  expect(result.coverageDominant).toBe(false);
});

// ── Edge cases ────────────────────────────────────────────────────────────────

test('single alternative → that alternative is recommended', () => {
  const a = makeAlt('alt-A', 196, 196, 1.2, 100);
  const result = rankAlternatives([a]);
  expect(result.recommendedId).toBe('alt-A');
});

test('empty alternatives → empty recommendedId', () => {
  const result = rankAlternatives([]);
  expect(result.recommendedId).toBe('');
});