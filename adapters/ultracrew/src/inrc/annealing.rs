//! `ultracrew::inrc::annealing` — Local Simulated Annealing and Memetic Search Engine.
//!
//! Encapsulates simulated annealing logic specifically within the Ultracrew domain adapter.
//! Keeps Coralys core strictly domain-agnostic and frozen.

use rand::Rng;

/// Defines a domain's passive interface for simulated annealing.
/// The domain generates candidate moves and calculates objective deltas,
/// but makes ZERO decisions about whether to accept, reject, or cool.
pub trait AnnealingDomain<G> {
    /// Computes the initial cost of the genome.
    fn initial_cost(&self, genome: &G) -> f64;

    /// Proposes a neighboring candidate move from the current genome.
    /// Returns (candidate_genome, delta_cost) where delta_cost = new_cost - current_cost.
    /// Negative delta_cost represents an improvement.
    /// Returns None if no valid candidate move could be generated.
    fn propose_move(&self, current: &G, rng: &mut dyn rand::RngCore) -> Option<(G, f64)>;
}

/// Configuration parameters for Simulated Annealing.
#[derive(Debug, Clone)]
pub struct AnnealingConfig {
    pub initial_temperature: f64,
    pub cooling_rate: f64,
    pub min_temperature: f64,
    pub restart_temperature: f64,
    pub time_budget_ms: u64,
}

impl Default for AnnealingConfig {
    fn default() -> Self {
        Self {
            initial_temperature: 50.0,
            cooling_rate: 0.99998,
            min_temperature: 0.1,
            restart_temperature: 50.0,
            time_budget_ms: 2500,
        }
    }
}

/// Simulated Annealing Execution Telemetry.
#[derive(Debug, Clone, Default)]
pub struct AnnealingTelemetry {
    pub total_iterations: usize,
    pub accepted_moves: usize,
    pub improving_moves: usize,
    pub rejected_moves: usize,
    pub initial_cost: f64,
    pub final_cost: f64,
    pub elapsed_ms: u128,
}

/// Domain-local Simulated Annealing Engine.
#[derive(Debug, Clone)]
pub struct SimulatedAnnealingEngine {
    pub config: AnnealingConfig,
}

impl SimulatedAnnealingEngine {
    pub fn new(config: AnnealingConfig) -> Self {
        Self { config }
    }

    /// Executes simulated annealing optimization on any `AnnealingDomain`.
    pub fn optimize<G: Clone, D: AnnealingDomain<G>, R: Rng>(
        &self,
        domain: &D,
        initial_genome: &G,
        rng: &mut R,
    ) -> (G, f64, AnnealingTelemetry) {
        let start = std::time::Instant::now();
        let mut current_genome = initial_genome.clone();
        let mut current_cost = domain.initial_cost(&current_genome);

        let mut best_genome = current_genome.clone();
        let mut best_cost = current_cost;

        let mut temp = self.config.initial_temperature;
        let mut telemetry = AnnealingTelemetry {
            initial_cost: current_cost,
            ..Default::default()
        };

        while start.elapsed().as_millis() < self.config.time_budget_ms as u128 {
            telemetry.total_iterations += 1;

            if let Some((candidate_genome, delta)) = domain.propose_move(&current_genome, rng) {
                let accept = if delta <= 0.0 {
                    telemetry.improving_moves += 1;
                    true
                } else {
                    let prob = (-delta / temp.max(1e-6)).exp();
                    rng.gen_bool(prob.min(1.0))
                };

                if accept {
                    telemetry.accepted_moves += 1;
                    current_genome = candidate_genome;
                    current_cost += delta;

                    if current_cost < best_cost {
                        best_cost = current_cost;
                        best_genome = current_genome.clone();
                    }
                } else {
                    telemetry.rejected_moves += 1;
                }
            }

            temp *= self.config.cooling_rate;
            if temp < self.config.min_temperature {
                temp = self.config.restart_temperature;
            }
        }

        telemetry.final_cost = best_cost;
        telemetry.elapsed_ms = start.elapsed().as_millis();

        (best_genome, best_cost, telemetry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    struct QuadraticDomain;

    impl AnnealingDomain<f64> for QuadraticDomain {
        fn initial_cost(&self, x: &f64) -> f64 {
            (x - 7.0).powi(2)
        }

        fn propose_move(&self, current: &f64, rng: &mut dyn rand::RngCore) -> Option<(f64, f64)> {
            let mut rng_ref = rand::rngs::StdRng::seed_from_u64(rng.next_u64());
            let step = rng_ref.gen_range(-1.0..1.0);
            let next_x = current + step;
            let current_cost = (current - 7.0).powi(2);
            let next_cost = (next_x - 7.0).powi(2);
            Some((next_x, next_cost - current_cost))
        }
    }

    #[test]
    fn test_simulated_annealing_quadratic_minimization() {
        let config = AnnealingConfig {
            initial_temperature: 10.0,
            cooling_rate: 0.99,
            min_temperature: 0.01,
            restart_temperature: 10.0,
            time_budget_ms: 50,
        };
        let engine = SimulatedAnnealingEngine::new(config);
        let mut rng = StdRng::seed_from_u64(42);

        let initial_x = 50.0;
        let (best_x, best_cost, telemetry) = engine.optimize(&QuadraticDomain, &initial_x, &mut rng);

        assert!(best_cost < 1.0, "Expected near-optimal cost, got {}", best_cost);
        assert!((best_x - 7.0).abs() < 1.0, "Expected best_x close to 7.0, got {}", best_x);
        assert!(telemetry.accepted_moves > 0);
        assert!(telemetry.improving_moves > 0);
    }
}
