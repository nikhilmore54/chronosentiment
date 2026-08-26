use std::time::Duration;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TerminationPolicy {
    FixedGenerations(usize),
    TargetFitness(f64),
    NoImprovement(usize),
    MaxRuntime(Duration),
    And(Box<TerminationPolicy>, Box<TerminationPolicy>),
    Or(Box<TerminationPolicy>, Box<TerminationPolicy>),
}

#[derive(Debug, Clone)]
pub struct TerminationState {
    pub generation: usize,
    pub elapsed_time: Duration,
    pub best_fitness: f64,
    pub average_fitness: f64,
    pub fitness_stddev: f64,
    pub stagnation_generations: usize,
}

impl TerminationPolicy {
    pub fn should_terminate(&self, state: &TerminationState) -> bool {
        match self {
            TerminationPolicy::FixedGenerations(limit) => state.generation >= *limit,
            TerminationPolicy::TargetFitness(target) => state.best_fitness >= *target,
            TerminationPolicy::NoImprovement(limit) => state.stagnation_generations >= *limit,
            TerminationPolicy::MaxRuntime(limit) => state.elapsed_time >= *limit,
            TerminationPolicy::And(p1, p2) => {
                p1.should_terminate(state) && p2.should_terminate(state)
            }
            TerminationPolicy::Or(p1, p2) => {
                p1.should_terminate(state) || p2.should_terminate(state)
            }
        }
    }

    pub fn and(self, other: TerminationPolicy) -> Self {
        TerminationPolicy::And(Box::new(self), Box::new(other))
    }

    pub fn or(self, other: TerminationPolicy) -> Self {
        TerminationPolicy::Or(Box::new(self), Box::new(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_termination_policies() {
        let state = TerminationState {
            generation: 10,
            elapsed_time: Duration::from_secs(5),
            best_fitness: 100.0,
            average_fitness: 80.0,
            fitness_stddev: 5.0,
            stagnation_generations: 3,
        };

        // FixedGenerations
        assert!(TerminationPolicy::FixedGenerations(10).should_terminate(&state));
        assert!(!TerminationPolicy::FixedGenerations(15).should_terminate(&state));

        // TargetFitness
        assert!(TerminationPolicy::TargetFitness(100.0).should_terminate(&state));
        assert!(TerminationPolicy::TargetFitness(90.0).should_terminate(&state));
        assert!(!TerminationPolicy::TargetFitness(110.0).should_terminate(&state));

        // NoImprovement
        assert!(TerminationPolicy::NoImprovement(3).should_terminate(&state));
        assert!(!TerminationPolicy::NoImprovement(5).should_terminate(&state));

        // MaxRuntime
        assert!(TerminationPolicy::MaxRuntime(Duration::from_secs(5)).should_terminate(&state));
        assert!(!TerminationPolicy::MaxRuntime(Duration::from_secs(10)).should_terminate(&state));
    }

    #[test]
    fn test_composite_policies() {
        let state = TerminationState {
            generation: 10,
            elapsed_time: Duration::from_secs(5),
            best_fitness: 100.0,
            average_fitness: 80.0,
            fitness_stddev: 5.0,
            stagnation_generations: 3,
        };

        let p1 = TerminationPolicy::FixedGenerations(10);
        let p2 = TerminationPolicy::MaxRuntime(Duration::from_secs(10)); // false

        // And
        assert!(!p1.clone().and(p2.clone()).should_terminate(&state));

        // Or
        assert!(
            p1.and(p2)
                .or(TerminationPolicy::NoImprovement(3))
                .should_terminate(&state)
        );
    }
}
