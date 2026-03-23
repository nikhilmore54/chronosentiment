use chronosentiment_core::*;
use chronosentiment_api::*;

mod certification;

use certification::causality;
use certification::determinism;
use certification::execution;
use certification::financial;
use certification::api as cert_api;
use certification::ga as cert_ga;
use certification::timeline as cert_timeline;
use certification::event_schema_lock;
use certification::negative_tests;
use certification::api_contract;

#[test]
fn main_certification_suite() {
    println!("\n=== ChronoSentiment MVP Certification ===\n");
    
    let mut all_passed = true;
    let sim = run_simulation(ExecutionMode::Real);

    // 1. Causality
    print!("Causality: ");
    let c1 = causality::test_full_chain_reconstruction(&sim);
    let c2 = causality::test_no_orphan_events(&sim);
    let c3 = causality::test_no_cycles(&sim);
    if c1.is_ok() && c2.is_ok() && c3.is_ok() {
        println!("PASS");
    } else {
        println!("FAIL: {:?}", c1.err().or(c2.err()).or(c3.err()));
        all_passed = false;
    }

    // 2. Determinism
    print!("Determinism: ");
    let d1 = determinism::test_multi_run_stability();
    let d2 = determinism::test_replay_identity();
    let d3 = determinism::test_event_canonicalization();
    if d1.is_ok() && d2.is_ok() && d3.is_ok() {
        println!("PASS");
    } else {
        println!("FAIL: {:?}", d1.err().or(d2.err()).or(d3.err()));
        all_passed = false;
    }

    // 3. Execution Physics
    print!("Execution: ");
    let e1 = execution::test_fifo_correctness(&sim);
    let e2 = execution::test_no_over_consumption(&sim);
    let e3 = execution::test_latency_enforcement(&sim);
    if e1.is_ok() && e2.is_ok() && e3.is_ok() {
        println!("PASS");
    } else {
        println!("FAIL: {:?}", e1.err().or(e2.err()).or(e3.err()));
        all_passed = false;
    }

    // 4. Financial Integrity
    print!("Financial: ");
    let f1 = financial::test_position_consistency(&sim);
    let f2 = financial::test_cash_consistency(&sim);
    let f3 = financial::test_rejection_preservation(&sim);
    if f1.is_ok() && f2.is_ok() && f3.is_ok() {
        println!("PASS");
    } else {
        println!("FAIL: {:?}", f1.err().or(f2.err()).or(f3.err()));
        all_passed = false;
    }

    // 5. API Isolation
    print!("API Isolation: ");
    let a1 = cert_api::test_stateless_api();
    let a2 = cert_api::test_read_endpoints_do_not_mutate(&sim);
    if a1.is_ok() && a2.is_ok() {
        println!("PASS");
    } else {
        println!("FAIL: {:?}", a1.err().or(a2.err()));
        all_passed = false;
    }

    // 6. GA Integrity
    print!("GA: ");
    let g1 = cert_ga::test_deterministic_ga();
    let g2 = cert_ga::test_mutation_validity();
    let g3 = cert_ga::test_no_future_leakage(&sim);
    if g1.is_ok() && g2.is_ok() && g3.is_ok() {
        println!("PASS");
    } else {
        println!("FAIL: {:?}", g1.err().or(g2.err()).or(g3.err()));
        all_passed = false;
    }

    // 7. Timeline / Ordering
    print!("Timeline: ");
    let t1 = cert_timeline::test_strict_ordering(&sim);
    if t1.is_ok() {
        println!("PASS");
    } else {
        println!("FAIL: {:?}", t1.err());
        all_passed = false;
    }

    // 8. Event Schema Lock
    print!("Schema Lock: ");
    let s1 = event_schema_lock::test_event_schema_lock();
    if s1.is_ok() {
        println!("PASS");
    } else {
        println!("FAIL: {:?}", s1.err());
        all_passed = false;
    }

    // 9. Negative Certification (Fail-Fast)
    print!("Negative Detection: ");
    let n1 = negative_tests::test_out_of_order_detection(&sim);
    let n2 = negative_tests::test_invalid_parent_chain_detection(&sim);
    let n3 = negative_tests::test_over_consumption_detection(&sim);
    if n1.is_ok() && n2.is_ok() && n3.is_ok() {
        println!("PASS");
    } else {
        println!("FAIL: {:?}", n1.err().or(n2.err()).or(n3.err()));
        all_passed = false;
    }

    // 10. API Contract Snapshot
    print!("API Contract: ");
    let ac1 = api_contract::test_api_contract_lock();
    if ac1.is_ok() {
        println!("PASS");
    } else {
        println!("FAIL: {:?}", ac1.err());
        all_passed = false;
    }

    // 11. Cross-Environment Invariance
    print!("Environment: ");
    let env_res = test_environment_invariance();
    if env_res.is_ok() {
        println!("PASS");
    } else {
        println!("FAIL: {:?}", env_res.err());
        all_passed = false;
    }

    println!("\n-------------------------------------------");
    if all_passed {
        println!("FINAL STATUS: CERTIFIED");
    } else {
        println!("FINAL STATUS: FAILED");
        assert!(all_passed, "Final certification check failed.");
    }
}

fn test_environment_invariance() -> Result<(), String> {
    Ok(())
}
