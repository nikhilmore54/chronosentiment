use crate::ecology::metrics::compute_spearman;
use crate::traits::Genome;

pub trait ExternalObserver<G: Genome> {
    fn observe(&self, genome: &G) -> f64;

    // Optional extension for returning full components if the domain supports it
    fn observe_components(&self, _genome: &G) -> Option<Vec<f64>> {
        None
    }
}

pub struct EcologyObserver {
    pub correlation_history: Vec<(u64, Vec<f64>)>, // (Generation, [Spearmans of O1..On vs Ext])
    pub best_external_history: Vec<(u64, f64)>,    // (Generation, BestExtSeen)
}

impl EcologyObserver {
    pub fn new() -> Self {
        Self {
            correlation_history: Vec::new(),
            best_external_history: Vec::new(),
        }
    }

    pub fn compute_and_record_correlations(
        &mut self,
        generation: u64,
        proxy_objectives: &[Vec<f64>],
        external_scores: &[f64],
    ) -> Vec<f64> {
        if proxy_objectives.is_empty() || external_scores.is_empty() {
            return Vec::new();
        }

        let num_objs = proxy_objectives[0].len();
        let mut spearmans = Vec::with_capacity(num_objs);

        for o in 0..num_objs {
            let o_scores: Vec<f64> = proxy_objectives.iter().map(|objs| objs[o]).collect();
            let s = compute_spearman(&o_scores, external_scores);
            spearmans.push(s);
        }

        self.correlation_history
            .push((generation, spearmans.clone()));
        spearmans
    }

    pub fn record_best_external(&mut self, generation: u64, best_ext: f64) {
        if let Some(last) = self.best_external_history.last() {
            if best_ext < last.1 {
                self.best_external_history.push((generation, best_ext));
            }
        } else {
            self.best_external_history.push((generation, best_ext));
        }
    }

    pub fn calculate_discovery_velocity(&self) -> Vec<(u64, u64, f64)> {
        // Returns (StartGen, EndGen, Velocity (pts per 1000 gen))
        let mut velocities = Vec::new();
        let bucket_size = 1000;

        if self.best_external_history.is_empty() {
            return velocities;
        }

        let max_gen = self.best_external_history.last().unwrap().0;
        let mut current_bucket = 0;

        while current_bucket * bucket_size <= max_gen {
            let start_gen = current_bucket * bucket_size;
            let end_gen = start_gen + bucket_size;

            let score_start = self.get_best_at_gen(start_gen);
            let score_end = self.get_best_at_gen(end_gen);

            if let (Some(s), Some(e)) = (score_start, score_end) {
                let diff = s - e;
                velocities.push((start_gen, end_gen, diff));
            }

            current_bucket += 1;
        }

        velocities
    }

    fn get_best_at_gen(&self, target_gen: u64) -> Option<f64> {
        if self.best_external_history.is_empty() {
            return None;
        }
        let mut best = self.best_external_history[0].1;
        for &(g, score) in &self.best_external_history {
            if g <= target_gen {
                best = score;
            } else {
                break;
            }
        }
        Some(best)
    }
}

impl Default for EcologyObserver {
    fn default() -> Self {
        Self::new()
    }
}
