use api::certify::handle_certify;
use chronosentiment_core::harness::run_simulation;
use chronosentiment_core::ExecutionMode;

fn main() {
    println!("=== ChronoSentiment MVP One-Command Certification ===\n");
    
    // 1. Run the primary simulation
    let sim = run_simulation(ExecutionMode::Real);
    println!("Primary Simulation: PnL={}, Events={}", sim.pnl, sim.events.len());
    
    // 2. Call the Certify API
    match handle_certify(&sim) {
        Ok(res) => {
            println!("\nCertification Result:");
            println!("Status: {}", res.status);
            println!("Hash 1: {}", res.hash_1);
            println!("Hash 2: {}", res.hash_2);
            if let Some(f) = res.fingerprint {
                println!("\nFingerprint:");
                println!("  Engine Version: {}", f.engine_version);
                println!("  Event Count: {}", f.event_count);
                println!("  Final Hash: {}", f.final_hash);
                println!("  Config Hash: {}", f.config_hash);
            }
            
            if res.status == "PASS" {
                println!("\nSUCCESS: The system is 100% deterministic and certified.");
                std::process::exit(0);
            } else {
                println!("\nFAILURE: Determinism divergence detected!");
                if let Some(dp) = res.divergence_point {
                    println!("Divergence Point (Sequence ID): {}", dp);
                }
                std::process::exit(1);
            }
        },
        Err(e) => {
            println!("Certification Error: {:?}", e);
            std::process::exit(1);
        }
    }
}
