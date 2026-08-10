/**
 * DTO for Optimization Responses
 * Normalizes backend responses into frontend-safe ViewModels.
 */

export function parseOptimizationResponse(data) {
  if (!data) return null;
  return {
    schedule: data.schedule || {},
    constraintReport: data.constraint_report || {},
    metrics: data.metrics || {},
    recommendations: data.recommendations || []
  };
}
