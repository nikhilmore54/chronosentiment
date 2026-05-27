use chronosentiment_core::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_absolute_determinism_n_times() {
        const N: usize = 100;
        let mut first_run_hash = None;

        for i in 0..N {
            let res = run_simulation(ExecutionMode::Real);

            // Using PnL and event count as a proxy for the hash for this test
            let current_hash = format!("PnL:{} Events:{}", res.pnl, res.events.len());

            if i == 0 {
                first_run_hash = Some(current_hash);
            } else {
                assert_eq!(
                    Some(current_hash),
                    first_run_hash,
                    "DIVERGENCE DETECTED at iteration {}. The simulation engine is non-deterministic.", 
                    i
                );
            }
        }

        println!("PASSED: 100 iterations of Absolute Determinism Check.");
    }

    #[test]
    fn test_event_sequence_identity() {
        let run1 = run_simulation(ExecutionMode::Real);
        let run2 = run_simulation(ExecutionMode::Real);

        assert_eq!(
            run1.events.len(),
            run2.events.len(),
            "Event sequence length mismatch"
        );

        for (idx, (e1, e2)) in run1.events.iter().zip(run2.events.iter()).enumerate() {
            assert_eq!(e1, e2, "Event content mismatch at index {}", idx);
            assert_eq!(
                e1.sequence_id(),
                e2.sequence_id(),
                "Sequence ID mismatch at index {}",
                idx
            );
        }
    }
}
