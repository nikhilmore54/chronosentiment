use serde::Serialize;
use std::collections::HashMap;

#[derive(Clone, Serialize, Debug)]
pub struct ExternalChampion {
    pub uid: u64,
    pub generation: u64,
    pub fitness: f64,
}

#[derive(Clone, Serialize, Debug)]
pub struct AlignmentRecord {
    pub generation: u64,
    pub uid: u64,
    pub internal_fitness_sum: f64,
    pub external_fitness: f64,
    pub archive_member: bool,
    pub admitted: bool,
    pub dominated: bool,
    pub age: u64,
    pub novelty: f64,
    pub fitness_objs: Vec<f64>,
    pub external_objs: Vec<f64>,
}

#[derive(Clone, Serialize, Debug)]
pub struct ChampionLifecycle {
    pub uid: u64,
    pub discovery_generation: u64,
    pub eviction_generation: Option<u64>,
    pub external_fitness: f64,
    pub internal_fitness_at_discovery: f64,
    pub archive_lifetime: Option<u64>,
}

#[derive(Clone, Serialize, Debug)]
pub struct ArchiveQualitySnapshot {
    pub generation: u64,
    pub best_fitness: f64,
    pub p10: f64,
    pub p25: f64,
    pub p50: f64,
    pub p75: f64,
    pub p90: f64,
    pub worst_fitness: f64,
}

pub struct ObservabilityTracker {
    pub best_ever: Option<ExternalChampion>,
    pub alignment_records: Vec<AlignmentRecord>,
    pub lifecycles: HashMap<u64, ChampionLifecycle>,
    pub snapshots: Vec<ArchiveQualitySnapshot>,
    pub evaluations: u64,
}

impl ObservabilityTracker {
    pub fn new() -> Self {
        Self {
            best_ever: None,
            alignment_records: Vec::new(),
            lifecycles: HashMap::new(),
            snapshots: Vec::new(),
            evaluations: 0,
        }
    }
    
    pub fn record_evaluation(
        &mut self, 
        uid: u64, 
        generation: u64, 
        internal: f64, 
        external: f64,
        archive_member: bool,
        admitted: bool,
        dominated: bool,
        age: u64,
        novelty: f64,
        fitness_objs: Vec<f64>,
        external_objs: Vec<f64>
    ) {
        self.evaluations += 1;
        self.alignment_records.push(AlignmentRecord {
            generation,
            uid,
            internal_fitness_sum: internal,
            external_fitness: external,
            archive_member,
            admitted,
            dominated,
            age,
            novelty,
            fitness_objs,
            external_objs,
        });
        
        let is_new_best = match &self.best_ever {
            Some(best) => external < best.fitness,
            None => true,
        };

        if is_new_best {
            self.best_ever = Some(ExternalChampion { uid, generation, fitness: external });
            self.lifecycles.insert(uid, ChampionLifecycle {
                uid,
                discovery_generation: generation,
                eviction_generation: None,
                external_fitness: external,
                internal_fitness_at_discovery: internal,
                archive_lifetime: None,
            });
        }
    }

    pub fn record_eviction(&mut self, uid: u64, current_gen: u64) {
        if let Some(lifecycle) = self.lifecycles.get_mut(&uid) {
            lifecycle.eviction_generation = Some(current_gen);
            lifecycle.archive_lifetime = Some(current_gen - lifecycle.discovery_generation);
        }
    }

    pub fn snapshot_archive(&mut self, generation: u64, archive_external_fitnesses: &[f64]) {
        if archive_external_fitnesses.is_empty() { return; }
        let mut scores = archive_external_fitnesses.to_vec();
        scores.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let len = scores.len() as f64;
        let p10 = scores[((len * 0.10) as usize).min(scores.len() - 1)];
        let p25 = scores[((len * 0.25) as usize).min(scores.len() - 1)];
        let p50 = scores[((len * 0.50) as usize).min(scores.len() - 1)];
        let p75 = scores[((len * 0.75) as usize).min(scores.len() - 1)];
        let p90 = scores[((len * 0.90) as usize).min(scores.len() - 1)];

        self.snapshots.push(ArchiveQualitySnapshot {
            generation,
            best_fitness: scores[0],
            p10,
            p25,
            p50,
            p75,
            p90,
            worst_fitness: scores[scores.len() - 1],
        });
    }
}
