/**
 * DTO for Optimization Requests
 * Maps frontend application state to the backend payload structure.
 */

export function buildOptimizationRequest(scenario, generationLimit, seed) {
  return {
    workers: scenario.workers,
    shifts: scenario.shifts,
    rng_seed: seed,
    generation_limit: generationLimit,
    scenario: {
      planning_horizon_hours: scenario.horizonHours,
      max_hours_per_worker: scenario.maxHoursPerWorker,
      minimum_rest_hours: 10
    }
  };
}
