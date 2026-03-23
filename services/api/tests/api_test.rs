use chronosentiment_api::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_simulate_determinism() {
        let input = SimulateInput {
            mode: "real".to_string(),
            dataset: None,
            seed: 42,
        };

        let res1 = handle_simulate(input.clone()).expect("Sim 1 failed");
        let res2 = handle_simulate(input).expect("Sim 2 failed");

        assert_eq!(res1.pnl, res2.pnl);
        assert_eq!(res1.trade_count, res2.trade_count);
        assert_eq!(res1.state_hash, res2.state_hash);
        assert_eq!(res1.events.len(), res2.events.len());
    }

    #[test]
    fn test_api_ga_determinism() {
        let input = GAInput {
            mode: "real".to_string(),
            population: 50,
            generations: 20,
            seed: 42,
        };

        let res1 = handle_ga_run(input.clone()).expect("GA 1 failed");
        let res2 = handle_ga_run(input).expect("GA 2 failed");

        assert_eq!(res1.best_config, res2.best_config);
    }

    #[test]
    fn test_api_events_exposure() {
        let sim_input = SimulateInput {
            mode: "real".to_string(),
            dataset: None,
            seed: 42,
        };
        let sim_res = handle_simulate(sim_input).unwrap();
        
        // Test raw event exposure
        let events = handle_events(&sim_res.original_result, Some(2), Some(5)).expect("Events failed");
        for e in events {
            let ts = e.timestamp();
            assert!(ts >= 2 && ts <= 5);
        }
    }

    #[test]
    fn test_api_certification_hook() {
        let sim_input = SimulateInput {
            mode: "real".to_string(),
            dataset: None,
            seed: 42,
        };
        let sim_res = handle_simulate(sim_input).unwrap();
        
        let report = handle_certify(&sim_res.original_result).expect("Certify failed");
        assert!(report.passes_identity_check);
        assert_eq!(report.last_run_hash, report.replay_hash);
    }
}
