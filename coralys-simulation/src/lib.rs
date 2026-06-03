pub mod traits;

pub use traits::{Action, Environment, Policy, Simulator};

#[cfg(test)]
mod tests {
    use super::*;

    // 1. Define the Action
    #[derive(Debug, PartialEq)]
    enum CounterAction {
        Increment,
        Decrement,
    }

    impl Action for CounterAction {}

    // 2. Define the Environment
    struct CounterEnvironment;

    impl Environment for CounterEnvironment {
        type State = i32;
    }

    // 3. Define the Policy
    struct AlwaysIncrementPolicy;

    impl Policy<CounterEnvironment, CounterAction> for AlwaysIncrementPolicy {
        fn choose_action(&self, _state: &i32) -> CounterAction {
            CounterAction::Increment
        }
    }

    // 4. Define the Simulator
    struct CounterSimulator {
        state: i32,
    }

    impl Simulator<CounterEnvironment, CounterAction> for CounterSimulator {
        type Outcome = i32;

        fn step(&mut self, action: CounterAction) -> Self::Outcome {
            match action {
                CounterAction::Increment => self.state += 1,
                CounterAction::Decrement => self.state -= 1,
            }
            self.state
        }
    }

    #[test]
    fn test_simulation_abstraction() {
        let policy = AlwaysIncrementPolicy;
        let mut simulator = CounterSimulator { state: 0 };

        // Emulate a loop where the policy observes state and simulator steps
        let current_state = simulator.state;
        let action = policy.choose_action(&current_state);
        assert_eq!(action, CounterAction::Increment);

        let new_state = simulator.step(action);
        assert_eq!(new_state, 1);
        
        let next_action = policy.choose_action(&new_state);
        let final_state = simulator.step(next_action);
        assert_eq!(final_state, 2);
    }
}
