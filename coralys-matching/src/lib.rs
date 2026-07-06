use coralys_core::MatchingResult;

pub trait AssignmentSolver {
    type Worker;
    type Demand;
    type Matching;

    fn assign(
        &self,
        workers: &[Self::Worker],
        demands: &[Self::Demand],
    ) -> Self::Matching;
}

pub struct BipartiteMatchingSolver;

impl AssignmentSolver for BipartiteMatchingSolver {
    type Worker = (usize, Vec<String>);
    type Demand = (String, usize);
    type Matching = MatchingResult<(usize, String)>;

    fn assign(
        &self,
        workers: &[Self::Worker],
        demands: &[Self::Demand],
    ) -> Self::Matching {
        // Flatten demands into individual unit demand slots
        let mut demand_slots = Vec::new();
        for (skill, count) in demands {
            for _ in 0..*count {
                demand_slots.push(skill.clone());
            }
        }

        let num_workers = workers.len();
        let num_slots = demand_slots.len();

        // Build adjacency list: worker_idx -> list of demand_slot_indices they can satisfy
        let mut adj = vec![Vec::new(); num_workers];
        for (i, (_worker_idx, skills)) in workers.iter().enumerate() {
            for (j, slot_skill) in demand_slots.iter().enumerate() {
                if skills.contains(slot_skill) {
                    adj[i].push(j);
                }
            }
        }

        let mut match_worker = vec![None; num_workers];
        let mut match_slot = vec![None; num_slots];
        let mut cardinality = 0;

        for i in 0..num_workers {
            let mut visited = vec![false; num_slots];
            if dfs(i, &adj, &mut match_worker, &mut match_slot, &mut visited) {
                cardinality += 1;
            }
        }

        // Gather final assignments: (worker_original_idx, assigned_skill)
        let mut assignments = Vec::new();
        for i in 0..num_workers {
            if let Some(slot_idx) = match_worker[i] {
                let worker_original_idx = workers[i].0;
                let assigned_skill = demand_slots[slot_idx].clone();
                assignments.push((worker_original_idx, assigned_skill));
            }
        }

        MatchingResult {
            cardinality,
            unmatched_supply: num_workers - cardinality,
            unmatched_demand: num_slots - cardinality,
            assignments,
        }
    }
}

fn dfs(
    u: usize,
    adj: &Vec<Vec<usize>>,
    match_worker: &mut Vec<Option<usize>>,
    match_slot: &mut Vec<Option<usize>>,
    visited: &mut Vec<bool>,
) -> bool {
    for &v in &adj[u] {
        if !visited[v] {
            visited[v] = true;
            if match_slot[v].is_none() || dfs(match_slot[v].unwrap(), adj, match_worker, match_slot, visited) {
                match_worker[u] = Some(v);
                match_slot[v] = Some(u);
                return true;
            }
        }
    }
    false
}
