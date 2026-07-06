pub mod traits;
pub mod recommender;

pub use traits::{Explainer, Ranker};
pub use recommender::{InterventionRecommendation, RecommendationReport, EcologyRecommender};


#[cfg(test)]
mod tests {
    use super::*;

    // 1. The minimal Ranker
    struct DescendingRanker;

    impl Ranker<i32> for DescendingRanker {
        fn rank(&self, mut candidates: Vec<i32>) -> Vec<i32> {
            // Sort in descending order
            candidates.sort_by(|a, b| b.cmp(a));
            candidates
        }
    }

    // 2. The minimal Explainer
    struct NumberExplainer;

    impl Explainer<i32> for NumberExplainer {
        type Explanation = String;

        fn explain(&self, _item: &i32) -> Self::Explanation {
            "Highest value candidate".to_string()
        }
    }

    #[test]
    fn test_recommendation_pipeline_compilation() {
        let candidates = vec![1, 5, 3, 9];
        
        let ranker = DescendingRanker;
        let ranked = ranker.rank(candidates);
        
        assert_eq!(ranked, vec![9, 5, 3, 1]);
        
        let explainer = NumberExplainer;
        let explanation = explainer.explain(&ranked[0]);
        
        assert_eq!(explanation, "Highest value candidate");
    }
}
