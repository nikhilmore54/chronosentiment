use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::Hash;

pub trait ContextKey: Clone + Hash + Eq + PartialEq {}

pub struct OpportunityStats {
    pub observations: u64,
    pub champions: u64,
}

pub struct OpportunityMemory<K: ContextKey> {
    pub map: HashMap<K, OpportunityStats>,
    pub alpha: f64,
    pub beta: f64,
}

impl<K: ContextKey> OpportunityMemory<K> {
    pub fn new(prior_weight: f64, global_rate: f64) -> Self {
        Self {
            map: HashMap::new(),
            alpha: global_rate * prior_weight,
            beta: (1.0 - global_rate) * prior_weight,
        }
    }

    pub fn is_known(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    pub fn score(&self, key: &K) -> f64 {
        if let Some(stats) = self.map.get(key) {
            (stats.champions as f64 + self.alpha)
                / (stats.observations as f64 + self.alpha + self.beta)
        } else {
            self.alpha / (self.alpha + self.beta)
        }
    }

    pub fn record(&mut self, key: K, is_champ: bool) {
        let stats = self.map.entry(key).or_insert(OpportunityStats {
            observations: 0,
            champions: 0,
        });
        stats.observations += 1;
        if is_champ {
            stats.champions += 1;
        }
    }
}

pub trait AdvisoryCandidate {
    type Context: ContextKey;
    fn fitness_bucket(&self) -> i64;
    fn parent_context(&self) -> Option<&Self::Context>;
    /// Direction: true if lower fitness bucket is better, false if higher is better
    fn lower_is_better() -> bool;

    // In case two candidates are exactly equal in bucket and score, we need a fallback comparison.
    // E.g. raw fitness comparison
    fn fallback_cmp(&self, other: &Self) -> Ordering;
}

pub struct AdvisoryRanker;

impl AdvisoryRanker {
    pub fn sort<C: AdvisoryCandidate>(
        candidates: &mut [C],
        memory: &OpportunityMemory<C::Context>,
        default_context: &C::Context,
    ) {
        let lower_better = C::lower_is_better();

        candidates.sort_by(|a, b| {
            let bucket_a = a.fitness_bucket();
            let bucket_b = b.fitness_bucket();

            let bucket_cmp = if lower_better {
                bucket_a.cmp(&bucket_b)
            } else {
                bucket_b.cmp(&bucket_a) // Higher is better
            };

            match bucket_cmp {
                Ordering::Equal => {
                    let score_a = a
                        .parent_context()
                        .map(|ctx| memory.score(ctx))
                        .unwrap_or_else(|| memory.score(default_context));
                    let score_b = b
                        .parent_context()
                        .map(|ctx| memory.score(ctx))
                        .unwrap_or_else(|| memory.score(default_context));

                    // Higher opportunity score is ALWAYS better
                    score_b.partial_cmp(&score_a).unwrap_or(Ordering::Equal)
                }
                other => other,
            }
        });
    }
}
