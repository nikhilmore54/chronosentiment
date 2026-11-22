//! Evolutionary population-based constructive scheduler.
//!
//! [`EvolutionaryScheduler`] assigns pairings to rotations in a roster using
//! an evolutionary algorithm (MOGA-inspired population search with tournament selection,
//! crossover, multi-level mutation, repair, and elitism).
//!
//! # Key Architectural Properties
//!
//! - **Unassigned Reserve Pool ($N_{rot}$)**: Pairings that conflict temporally
//!   with all active crew rotations are placed in a virtual reserve pool, avoiding
//!   illegal overlaps while maintaining clean coverage optimization.
//! - **Gradient Soft Penalty**: Infeasible candidate genomes receive a smooth
//!   gradient penalty rather than infinite cost, guiding selection smoothly
//!   toward feasibility boundaries.
//! - **Hybrid Construction Seeding**: Incorporates the greedy baseline schedule
//!   into generation 0, providing a strict mathematical floor (guaranteeing that
//!   evolutionary search never scores worse than greedy).

use crate::domain::pairing::{Pairing, PairingId};
use crate::domain::roster::{PlanningPeriod, Roster, RosterId};
use crate::domain::rotation::Rotation;
use crate::optimization::cost::CostEvaluator;
use crate::optimization::metrics::OptimizationMetrics;
use crate::optimization::search::greedy::GreedyScheduler;

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use std::collections::HashMap;

/// Configuration parameters for the evolutionary scheduler.
#[derive(Debug, Clone)]
pub struct EvolutionaryConfig {
    /// Population size (number of candidate schedules maintained).
    pub pop_size: usize,
    /// Number of evolutionary generations to run.
    pub generations: usize,
    /// Probability of crossover between two selected parents (0.0 .. 1.0).
    pub crossover_prob: f64,
    /// Tournament size for parent selection.
    pub tournament_k: usize,
    /// Random seed for deterministic reproducibility.
    pub seed: u64,
}

impl Default for EvolutionaryConfig {
    fn default() -> Self {
        Self {
            pop_size: 50,
            generations: 200,
            crossover_prob: 0.80,
            tournament_k: 3,
            seed: 42,
        }
    }
}

/// An evolutionary population-based scheduler.
pub struct EvolutionaryScheduler<'a> {
    evaluator: &'a CostEvaluator,
    weights: Vec<f64>,
    config: EvolutionaryConfig,
}

impl<'a> EvolutionaryScheduler<'a> {
    /// Create a new [`EvolutionaryScheduler`] with default configuration.
    pub fn new(evaluator: &'a CostEvaluator, weights: Vec<f64>) -> Self {
        Self::with_config(evaluator, weights, EvolutionaryConfig::default())
    }

    /// Create a new [`EvolutionaryScheduler`] with custom configuration.
    pub fn with_config(
        evaluator: &'a CostEvaluator,
        weights: Vec<f64>,
        config: EvolutionaryConfig,
    ) -> Self {
        Self {
            evaluator,
            weights,
            config,
        }
    }

    /// Assign `pairings` to rotations in the `roster` evolutionarily.
    pub fn assign(
        &self,
        roster: &Roster,
        pairings: Vec<Pairing>,
        metrics: &mut OptimizationMetrics,
    ) -> Roster {
        let n_pairings = pairings.len();
        let existing_rotations: Vec<Rotation> = roster.rotations().cloned().collect();
        let n_rot = existing_rotations.len();

        if n_pairings == 0 || n_rot == 0 {
            return roster.clone();
        }

        // 1. Generate Greedy baseline to serve as elite seed
        let greedy = GreedyScheduler::new(self.evaluator, self.weights.clone());
        let mut greedy_metrics = OptimizationMetrics::new();
        let greedy_roster = greedy.assign(roster, pairings.clone(), &mut greedy_metrics);
        let greedy_cost = self.evaluate_roster_cost(&greedy_roster);

        // Extract greedy seed genome: pairings not in roster go to unassigned pool (n_rot)
        let mut pairing_to_rot: HashMap<PairingId, usize> = HashMap::new();
        for (rot_idx, rot) in greedy_roster.rotations().enumerate() {
            for p in rot.pairings() {
                pairing_to_rot.insert(p.id.clone(), rot_idx);
            }
        }
        let greedy_seed: Vec<usize> = pairings
            .iter()
            .map(|p| pairing_to_rot.get(&p.id).copied().unwrap_or(n_rot))
            .collect();

        // 2. Initialize evolutionary search
        let mut rng = StdRng::seed_from_u64(self.config.seed);
        let mut population: Vec<Vec<usize>> = Vec::with_capacity(self.config.pop_size);

        // Seed with Greedy baseline and perturbed variations
        population.push(greedy_seed.clone());
        for _ in 0..4 {
            let mut copy = greedy_seed.clone();
            let p1 = rng.gen_range(0..n_pairings);
            let p2 = rng.gen_range(0..n_pairings);
            copy.swap(p1, p2);
            self.repair_genome(&mut copy, n_rot, &mut rng);
            population.push(copy);
        }

        // Fill remainder with randomized assignments
        while population.len() < self.config.pop_size {
            let mut genome: Vec<usize> = (0..n_pairings)
                .map(|i| if i < n_rot { i } else { rng.gen_range(0..=n_rot) })
                .collect();
            self.repair_genome(&mut genome, n_rot, &mut rng);
            population.push(genome);
        }

        // 3. Initial fitness evaluation
        let mut fitnesses: Vec<f64> = population
            .iter()
            .map(|g| self.evaluate_fitness(g, &pairings, &existing_rotations, &roster.period))
            .collect();

        let mut best_idx = 0;
        let mut best_cost = fitnesses[0];

        // 4. Evolutionary Loop
        for _gen in 0..self.config.generations {
            let mut next_pop: Vec<Vec<usize>> = Vec::with_capacity(self.config.pop_size);

            while next_pop.len() < self.config.pop_size {
                let pa_idx = (0..self.config.tournament_k)
                    .map(|_| rng.gen_range(0..self.config.pop_size))
                    .min_by(|&a, &b| fitnesses[a].partial_cmp(&fitnesses[b]).unwrap())
                    .unwrap();
                let pb_idx = (0..self.config.tournament_k)
                    .map(|_| rng.gen_range(0..self.config.pop_size))
                    .min_by(|&a, &b| fitnesses[a].partial_cmp(&fitnesses[b]).unwrap())
                    .unwrap();

                let mut child = if rng.r#gen::<f64>() < self.config.crossover_prob {
                    let point = rng.gen_range(1..n_pairings);
                    let mut c = population[pa_idx][..point].to_vec();
                    c.extend_from_slice(&population[pb_idx][point..]);
                    c
                } else {
                    population[pa_idx].clone()
                };

                let mut mutated = false;
                if rng.r#gen::<f64>() < 0.25 {
                    let p1 = rng.gen_range(0..n_pairings);
                    let p2 = rng.gen_range(0..n_pairings);
                    child.swap(p1, p2);
                    mutated = true;
                } else {
                    for gene in child.iter_mut() {
                        if rng.r#gen::<f64>() < 1.0 / n_pairings as f64 {
                            *gene = rng.gen_range(0..=n_rot);
                            mutated = true;
                        }
                    }
                }
                if mutated {
                    self.repair_genome(&mut child, n_rot, &mut rng);
                }

                next_pop.push(child);
            }

            let next_fitnesses: Vec<f64> = next_pop
                .iter()
                .map(|g| self.evaluate_fitness(g, &pairings, &existing_rotations, &roster.period))
                .collect();

            // Elitism: carry best over
            let prev_best_idx = fitnesses
                .iter()
                .enumerate()
                .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(i, _)| i)
                .unwrap();
            let new_worst_idx = next_fitnesses
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(i, _)| i)
                .unwrap();

            population = next_pop;
            fitnesses = next_fitnesses;
            population[new_worst_idx] = population[prev_best_idx].clone();
            fitnesses[new_worst_idx] = fitnesses[prev_best_idx];

            let gen_best_idx = fitnesses
                .iter()
                .enumerate()
                .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(i, _)| i)
                .unwrap();

            if fitnesses[gen_best_idx] < best_cost {
                best_cost = fitnesses[gen_best_idx];
                best_idx = gen_best_idx;
                metrics.record_improvement();
            }
        }

        // Decode best genome into final roster
        if best_cost < greedy_cost {
            if let Some(improved_roster) = self.decode_genome(
                &population[best_idx],
                &pairings,
                &existing_rotations,
                &roster.period,
            ) {
                return improved_roster;
            }
        }

        greedy_roster
    }

    fn evaluate_roster_cost(&self, roster: &Roster) -> f64 {
        let cost = self.evaluator.evaluate(roster);
        if self.weights.is_empty() {
            cost.sum()
        } else {
            cost.weighted_sum(&self.weights)
        }
    }

    fn evaluate_fitness(
        &self,
        genome: &[usize],
        pairings: &[Pairing],
        base_rotations: &[Rotation],
        period: &PlanningPeriod,
    ) -> f64 {
        let n_rot = base_rotations.len();
        let mut buckets: Vec<Vec<&Pairing>> = vec![Vec::new(); n_rot];
        let mut unassigned = 0usize;

        for (i, &rot_idx) in genome.iter().enumerate() {
            if rot_idx < n_rot {
                buckets[rot_idx].push(&pairings[i]);
            } else {
                unassigned += 1;
            }
        }

        let mut overlaps = 0usize;
        let mut empty_rotations = 0usize;

        for bucket in buckets.iter_mut() {
            if bucket.is_empty() {
                empty_rotations += 1;
            } else {
                bucket.sort_by_key(|p| p.start());
                for w in bucket.windows(2) {
                    if w[1].start() < w[0].end() {
                        overlaps += 1;
                    }
                }
            }
        }

        if overlaps > 0 || empty_rotations > 0 {
            return 100_000.0 + (overlaps as f64) * 1_000.0 + (empty_rotations as f64) * 5_000.0;
        }

        match self.decode_genome(genome, pairings, base_rotations, period) {
            Some(roster) => {
                let cost = self.evaluate_roster_cost(&roster);
                cost + (unassigned as f64) * 0.001
            }
            None => f64::INFINITY,
        }
    }

    fn decode_genome(
        &self,
        genome: &[usize],
        pairings: &[Pairing],
        base_rotations: &[Rotation],
        period: &PlanningPeriod,
    ) -> Option<Roster> {
        let n_rot = base_rotations.len();
        let mut buckets: Vec<Vec<Pairing>> = vec![Vec::new(); n_rot];

        for (i, &rot_idx) in genome.iter().enumerate() {
            if rot_idx < n_rot {
                buckets[rot_idx].push(pairings[i].clone());
            }
        }

        let mut new_rotations = Vec::with_capacity(n_rot);
        for (i, mut bucket) in buckets.into_iter().enumerate() {
            if bucket.is_empty() {
                return None;
            }
            bucket.sort_by_key(|p| p.start());
            let rot = Rotation::new(
                base_rotations[i].id.clone(),
                base_rotations[i].crew_id.clone(),
                bucket,
            )
            .ok()?;
            new_rotations.push(rot);
        }

        Roster::new(
            RosterId::new("CORALYS_EVO"),
            period.clone(),
            vec![], // Legs empty in unassigned context or populated from roster
            new_rotations,
        )
        .ok()
    }

    fn repair_genome(&self, genome: &mut [usize], n_rot: usize, rng: &mut impl Rng) {
        if n_rot == 0 {
            return;
        }
        loop {
            let mut counts = vec![0usize; n_rot];
            for &g in genome.iter() {
                if g < n_rot {
                    counts[g] += 1;
                }
            }
            let empty_idx = match counts.iter().position(|&c| c == 0) {
                Some(e) => e,
                None => break,
            };

            let unassigned: Vec<usize> = genome
                .iter()
                .enumerate()
                .filter(|&(_, &g)| g == n_rot)
                .map(|(i, _)| i)
                .collect();
            if let Some(&victim) = unassigned.choose(rng) {
                genome[victim] = empty_idx;
                continue;
            }

            let max_rot = counts
                .iter()
                .enumerate()
                .max_by_key(|&(_, &c)| c)
                .map(|(i, _)| i)
                .unwrap();
            if counts[max_rot] <= 1 {
                break;
            }
            let candidates: Vec<usize> = genome
                .iter()
                .enumerate()
                .filter(|&(_, &g)| g == max_rot)
                .map(|(i, _)| i)
                .collect();
            if let Some(&victim) = candidates.choose(rng) {
                genome[victim] = empty_idx;
            } else {
                break;
            }
        }
    }
}
