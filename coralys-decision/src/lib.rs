pub mod traits;

pub use traits::{CandidateEvaluator, DecisionMaker, DecisionPolicy};

#[cfg(test)]
mod tests {
    use super::*;

    // 1. The Candidate is an Integer
    struct IntegerCandidate(i32);

    // 2. The Evaluation is a Score
    struct IntegerScore(i32);

    // 3. The Evaluator assigns a score to the candidate
    struct SimpleEvaluator;
    impl CandidateEvaluator<IntegerCandidate> for SimpleEvaluator {
        type Evaluation = IntegerScore;

        fn evaluate(&self, candidate: &IntegerCandidate) -> Self::Evaluation {
            // Trivial evaluation: score is just the integer value itself
            IntegerScore(candidate.0)
        }
    }

    // 4. The Decision Type
    #[derive(Debug, PartialEq)]
    enum AcceptRejectDecision {
        Accept,
        Reject,
    }

    // 5. The Decision Maker converts the score into a raw decision
    struct ThresholdDecisionMaker {
        threshold: i32,
    }
    impl DecisionMaker<IntegerScore> for ThresholdDecisionMaker {
        type Decision = AcceptRejectDecision;

        fn decide(&self, evaluation: &IntegerScore) -> Self::Decision {
            if evaluation.0 > self.threshold {
                AcceptRejectDecision::Accept
            } else {
                AcceptRejectDecision::Reject
            }
        }
    }

    // 6. The Decision Policy acts as the gatekeeper
    struct OnlyAcceptPolicy;
    impl DecisionPolicy<AcceptRejectDecision> for OnlyAcceptPolicy {
        fn accept(&self, decision: &AcceptRejectDecision) -> bool {
            *decision == AcceptRejectDecision::Accept
        }
    }

    #[test]
    fn test_decision_pipeline_compilation() {
        let evaluator = SimpleEvaluator;
        let maker = ThresholdDecisionMaker { threshold: 10 };
        let policy = OnlyAcceptPolicy;

        // Test Candidate A (Should fail)
        let candidate_a = IntegerCandidate(5);
        let eval_a = evaluator.evaluate(&candidate_a);
        let decision_a = maker.decide(&eval_a);
        let accepted_a = policy.accept(&decision_a);
        
        assert_eq!(decision_a, AcceptRejectDecision::Reject);
        assert!(!accepted_a);

        // Test Candidate B (Should pass)
        let candidate_b = IntegerCandidate(15);
        let eval_b = evaluator.evaluate(&candidate_b);
        let decision_b = maker.decide(&eval_b);
        let accepted_b = policy.accept(&decision_b);
        
        assert_eq!(decision_b, AcceptRejectDecision::Accept);
        assert!(accepted_b);
    }
}
