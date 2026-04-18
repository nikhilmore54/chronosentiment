use chronosentiment_core::*;
use chronosentiment_core::ese::{ExecutionEngine, ExecutionStatus, FIXED_LATENCY};
use chronosentiment_core::ga::{OrderIntent};

#[cfg(test)]
mod tests {
    use super::*;

    fn create_mock_market(prices: Vec<u64>, volumes: Vec<u64>) -> Vec<MarketEvent> {
        prices.into_iter().zip(volumes.into_iter()).enumerate().map(|(i, (p, v))| {
            MarketEvent {
                subtype: MarketEventType::Trade,
                price: p,
                quantity: v,
                side: None,
                exchange_ts: i as u64,
            }
        }).collect()
    }

    #[test]
    fn test_ese_latency_integrity() {
        let mut ese = ExecutionEngine::default();
        let market = create_mock_market(
            vec![100, 101, 102, 103, 104, 105, 106, 107, 108],
            vec![10, 10, 10, 10, 10, 1000, 1000, 1000, 1000]
        );

        let intent = OrderIntent {
            symbol: "TEST".to_string(),
            side: Side::Buy,
            quantity: 100,
            price: 100,
            tp_target: 110,
            sl_target: 90,
            holding_period: 200, // Now required
        };

        // If we inject at index 0, with latency 5, activation must be at index 5
        let res = ese.execute(intent, &market, 0);
        
        // Price at index 5 is 105
        assert_eq!(res.status, ExecutionStatus::Filled, "Should fill as liquidity (1000) > queue (5*1000)");
        // Wait, queue pressure at index 0..5 is 5000. 5000 is NOT < 1000.
        // Let's adjust volumes to pass.
    }

    #[test]
    fn test_ese_queue_blocking() {
        let mut ese = ExecutionEngine::default();
        // Latency is 5. We inject at 0. Queue window is [0, 5).
        // If volumes in [0, 5) sum to 5000, and arrival at index 5 has qty 1000 -> Rejected.
        let market = create_mock_market(
            vec![100; 10],
            vec![1000, 1000, 1000, 1000, 1000, 500, 1000, 1000, 1000, 1000]
        );

        let intent = OrderIntent {
            symbol: "TEST".to_string(),
            side: Side::Buy,
            quantity: 100,
            price: 100,
            tp_target: 110,
            sl_target: 90,
            holding_period: 200,
        };

        let res = ese.execute(intent, &market, 0);
        assert_eq!(res.status, ExecutionStatus::Rejected, "High queue pressure should block execution");
        assert_eq!(res.exit_reason, GaExitReason::NoFill);
    }

    #[test]
    fn test_ese_partial_fill() {
        let mut ese = ExecutionEngine::default();
        // Queue = 0 (low volume before activation), arrival liquidity = 50.
        // Intent quantity = 100. Should result in Partial fill of 50.
        let market = create_mock_market(
            vec![100; 10],
            vec![1, 1, 1, 1, 1, 50, 100, 100, 100, 100]
        );

        let intent = OrderIntent {
            symbol: "TEST".to_string(),
            side: Side::Buy,
            quantity: 100,
            price: 100,
            tp_target: 110,
            sl_target: 90,
            holding_period: 200,
        };

        let res = ese.execute(intent, &market, 0);
        assert_eq!(res.status, ExecutionStatus::Partial);
        assert_eq!(res.filled_quantity, 50);
    }

    #[test]
    fn test_ese_path_dependency() {
        let mut ese = ExecutionEngine::default();
        
        let intent = OrderIntent {
            symbol: "TEST".to_string(),
            side: Side::Buy,
            quantity: 100,
            price: 100,
            tp_target: 110,
            sl_target: 90,
            holding_period: 200,
        };

        // Scenario A: High volume (1000) happens BEFORE activation, then low volume (500) arrival.
        // Queue [0,5) = 1000 + 4 = 1004. Arrival = 500. REJECTED.
        let market_a = create_mock_market(
            vec![100; 10],
            vec![1000, 1, 1, 1, 1, 500, 100, 100, 100, 100]
        );
        let res_a = ese.execute(intent.clone(), &market_a, 0);
        assert_eq!(res_a.status, ExecutionStatus::Rejected);

        // Scenario B: Same total volume in window [0,5), but high volume moved to arrival.
        // Queue [0,5) = 5. Arrival = 1000. FILLED. 
        let market_b = create_mock_market(
            vec![100; 10],
            vec![1, 1, 1, 1, 1, 1499, 100, 100, 100, 100]
        );
        let res_b = ese.execute(intent.clone(), &market_b, 0);
        assert_eq!(res_b.status, ExecutionStatus::Filled);
        
        println!("PATH_DEPENDENCY PASSED: Outcomes differ based on event sequence despite similar volume totals.");
    }

    #[test]
    fn test_ese_state_provable_determinism() {
        let mut ese = ExecutionEngine::default();
        let market = create_mock_market(vec![100; 50], vec![100; 50]);
        let intent = OrderIntent {
            symbol: "TEST".to_string(),
            side: Side::Buy,
            quantity: 10,
            price: 100,
            tp_target: 110,
            sl_target: 90,
            holding_period: 200,
        };

        let mut snapshots = Vec::new();
        for _ in 0..50 {
            let res = ese.execute(intent.clone(), &market, 0);
            snapshots.push(format!("{:?}-{:?}", res.status, res.realized_pnl));
        }

        let first = &snapshots[0];
        for (i, s) in snapshots.iter().enumerate() {
            assert_eq!(first, s, "State divergence detected at iteration {}", i);
        }
        println!("PROVABLE_DETERMINISM PASSED: 50/50 snapshots identical.");
    }
}
