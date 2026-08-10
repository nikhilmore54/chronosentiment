export function calculateCoverage(metrics) {
  if (!metrics) return 100.0;
  return metrics.coverage_pct || 100.0;
}

export function summarizeConstraintReport(cr) {
  if (!cr) return { hardViolations: 0, restViolations: 0, fitness: 0.0 };
  return {
    hardViolations: cr.hard_violations || 0,
    restViolations: cr.rest_violations || 0,
    fitness: cr.fitness || 0.0,
  };
}
