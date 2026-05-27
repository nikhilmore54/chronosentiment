use chronosentiment_core::ga::{Strategy, stable_deterministic_hash, BehavioralSignature};
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stable_deterministic_hash() {
        let h1 = stable_deterministic_hash((1, 2, 3));
        let h2 = stable_deterministic_hash((1, 2, 3));
        let h3 = stable_deterministic_hash((3, 2, 1));

        assert_eq!(h1, h2, "Hash must be stable");
        assert_ne!(h1, h3, "Hash must vary with input");
    }

    #[test]
    fn test_strategy_from_seed_determinism() {
        let seed = 42;
        let s1 = Strategy::from_seed(seed);
        let s2 = Strategy::from_seed(seed);
        let s3 = Strategy::from_seed(43);

        assert_eq!(s1, s2, "Strategy from same seed must be identical");
        assert_ne!(s1, s3, "Strategy from different seeds must differ");
    }

    #[test]
    fn test_behavioral_fingerprint_stability() {
        let s = Strategy::from_seed(100);
        let f1 = s.behavioral_fingerprint();
        let f2 = s.behavioral_fingerprint();
        
        assert_eq!(f1, f2, "Fingerprint must be stable for same genotype");
    }

    #[test]
    fn test_orthogonal_mutant_guarantee() {
        let parent = Strategy::from_seed(500);
        let parent_axes = parent.get_behavioral_axes(None);
        
        for i in 0..100 {
            let seed = 1000 + i;
            let mutant = parent.orthogonal_mutant(seed);
            let mutant_axes = mutant.get_behavioral_axes(None);
            
            let mut hamming_dist = 0;
            if mutant_axes.0 != parent_axes.0 { hamming_dist += 1; }
            if mutant_axes.1 != parent_axes.1 { hamming_dist += 1; }
            if mutant_axes.2 != parent_axes.2 { hamming_dist += 1; }
            if mutant_axes.3 != parent_axes.3 { hamming_dist += 1; }

            assert!(hamming_dist >= 2, "Orthogonal mutant must change at least 2 axes (iteration {})", i);
            assert_ne!(parent.behavioral_fingerprint(), mutant.behavioral_fingerprint());
        }
    }

    #[test]
    fn test_population_replay_truth() {
        // Mock a population generation and verify bit-level parity
        let gen = 5;
        let sig = 0.45; // effective diversity
        
        let mut pop1 = Vec::new();
        let mut pop2 = Vec::new();
        
        for i in 0..10 {
            let seed = stable_deterministic_hash((gen as u64, i as u64, (sig * 1000.0) as u64));
            pop1.push(Strategy::from_seed(seed));
        }
        
        for i in 0..10 {
            let seed = stable_deterministic_hash((gen as u64, i as u64, (sig * 1000.0) as u64));
            pop2.push(Strategy::from_seed(seed));
        }
        
        for (idx, (s1, s2)) in pop1.iter().zip(pop2.iter()).enumerate() {
            assert_eq!(s1, s2, "Population divergence at index {}", idx);
        }
    }
}
