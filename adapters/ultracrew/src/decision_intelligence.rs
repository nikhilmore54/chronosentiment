// Decision Intelligence module for UltraCrew

use std::collections::HashMap;
use crate::schedule_solution::ScheduleSolution;

/// Analyzes a `ScheduleSolution` and returns key metrics as a map.
/// The keys are descriptive metric names and the values are numeric scores.
pub fn analyze_solution(solution: &ScheduleSolution) -> HashMap<String, f64> {
    let mut metrics = HashMap::new();
    metrics.insert("fitness".to_string(), solution.fitness);
    metrics.insert("hard_violations".to_string(), solution.hard_violations as f64);
    metrics.insert("fairness_penalty".to_string(), solution.fairness_penalty);
    metrics.insert("fatigue_penalty".to_string(), solution.fatigue_penalty);
    metrics.insert("rest_violations".to_string(), solution.rest_violations as f64);
    metrics
}

/// Generates human‑readable insights from a `ScheduleSolution`.
/// Returns a vector of strings, each describing an aspect of the schedule.
pub fn generate_insights(solution: &ScheduleSolution) -> Vec<String> {
    let mut insights = Vec::new();
    insights.push(format!("Overall fitness: {:.2}", solution.fitness));
    if solution.hard_violations == 0 {
        insights.push("No hard‑constraint violations detected.".to_string());
    } else {
        insights.push(format!("Hard‑constraint violations: {}", solution.hard_violations));
    }
    if solution.fairness_penalty > 0.0 {
        insights.push(format!("Fairness penalty: {:.2}", solution.fairness_penalty));
    }
    if solution.fatigue_penalty > 0.0 {
        insights.push(format!("Fatigue penalty: {:.2}", solution.fatigue_penalty));
    }
    if solution.rest_violations > 0 {
        insights.push(format!("Rest period violations: {}", solution.rest_violations));
    }
    insights
}
