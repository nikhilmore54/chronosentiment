use std::fmt::Debug;

/// Represents the confidence state of a thesis in memory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LifecycleState {
    Candidate,
    Observed,
    Strengthened,
    Dormant,
    Archived,
    Recalled,
    Validated,
    Expired,
}

/// A hypothesis stored in the memory engine.
#[derive(Clone, Debug)]
pub struct VaultEntry<S, C, E> {
    pub structure: S,
    pub context: C,
    pub evidence: Vec<E>,
    pub state: LifecycleState,
    pub support: usize,
    pub score: f64,
    pub timestamp: u64,
}

/// The Discovery Layer.
/// Explores the combinatorial space to find locally optimal structures.
pub trait Discovery<S, C> {
    fn step(&mut self) -> Vec<(S, f64)>;
    fn inject_diversity(&mut self, structures: Vec<S>);
}

/// The Memory Layer.
/// Persists useful partial structures across hostile time horizons.
pub trait Memory<S, C, E> {
    fn store(&mut self, entry: VaultEntry<S, C, E>);
    fn query(&self, context: &C) -> Vec<VaultEntry<S, C, E>>;
    fn capacity(&self) -> usize;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn forget(&mut self); // Periodic purge of expired/evicted entries
}

/// The Recall Layer.
/// Constantly evaluates contextual triggers to retrieve historical hypotheses.
pub trait Recall<S, C, E> {
    fn evaluate_context(&self) -> Option<C>;
    fn retrieve(&self, memory: &dyn Memory<S, C, E>, context: &C) -> Vec<VaultEntry<S, C, E>>;
}

/// The Refinement Layer.
/// Exploits recalled hypotheses to assemble the final solution.
pub trait Refinement<S> {
    fn refine(&mut self, structures: Vec<S>) -> (S, f64);
}

/// The Observation Layer.
/// Ingests ongoing events and data for the runtime.
pub trait Observation<O> {
    fn ingest(&mut self, observation: O);
    fn recent(&self) -> Vec<O>;
}

/// The Memory Policy API.
/// Dictates allocation, strengthening, merging, and eviction logic for the Vault.
pub trait MemoryPolicy<S, C, E> {
    /// Evaluate if a new discovery deserves to be stored.
    fn should_store(&self, artifact: &VaultEntry<S, C, E>) -> bool;

    /// Handle a rediscovered structure (e.g., increase support/score, accumulate evidence).
    fn strengthen(&self, existing: &mut VaultEntry<S, C, E>, new_observation: &VaultEntry<S, C, E>);

    /// Attempt to merge multiple overlapping entries into a stronger artifact.
    fn merge(&self, entries: &[VaultEntry<S, C, E>]) -> Option<VaultEntry<S, C, E>>;

    /// Determine if an entry should be forgotten (evicted).
    fn should_evict(&self, entry: &VaultEntry<S, C, E>) -> bool;
}

/// A standard Vault implementation of Memory using MemoryPolicy.
pub struct PolicyVault<S, C, E, P> {
    pub entries: Vec<VaultEntry<S, C, E>>,
    pub max_capacity: usize,
    pub policy: P,
}

impl<S, C, E, P: MemoryPolicy<S, C, E>> PolicyVault<S, C, E, P> {
    pub fn new(max_capacity: usize, policy: P) -> Self {
        Self {
            entries: Vec::with_capacity(max_capacity),
            max_capacity,
            policy,
        }
    }
}

impl<S: Clone + PartialEq, C: Clone + PartialEq, E: Clone + PartialEq, P: MemoryPolicy<S, C, E>>
    Memory<S, C, E> for PolicyVault<S, C, E, P>
{
    fn store(&mut self, mut entry: VaultEntry<S, C, E>) {
        if !self.policy.should_store(&entry) {
            return;
        }

        // Check for strengthening
        for existing in self.entries.iter_mut() {
            if existing.structure == entry.structure && existing.context == entry.context {
                self.policy.strengthen(existing, &entry);
                return;
            }
        }

        // Check for merging
        if let Some(merged) = self.policy.merge(&self.entries) {
            entry = merged;
        }

        if self.entries.len() < self.max_capacity {
            self.entries.push(entry);
        } else {
            // If at capacity, force an eviction check
            self.forget();
            if self.entries.len() < self.max_capacity {
                self.entries.push(entry);
            }
            // else drop it (or we could force evict the lowest score, but policy handles should_evict)
        }
    }

    fn query(&self, _context: &C) -> Vec<VaultEntry<S, C, E>> {
        self.entries.clone()
    }

    fn capacity(&self) -> usize {
        self.max_capacity
    }
    fn len(&self) -> usize {
        self.entries.len()
    }

    fn forget(&mut self) {
        let mut i = 0;
        while i < self.entries.len() {
            if self.policy.should_evict(&self.entries[i]) {
                self.entries.remove(i);
            } else {
                i += 1;
            }
        }
    }
}
