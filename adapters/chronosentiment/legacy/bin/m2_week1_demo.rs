use chronosentiment_adapter::instrument::Instrument;
use chronosentiment_adapter::metrics::{concepts::Concept, profile::ProfileAssigner};

fn main() {
    println!("=== ChronoSentiment M2 Demo: Concept Evaluation Profile ===");

    // 1. Target Instrument
    let instrument = Instrument::new("NSE".to_string(), "RELIANCE".to_string());
    println!(
        "Target Instrument: {} ({})",
        instrument.display_symbol, instrument.exchange
    );

    // 2. Assign Profile
    let profile = ProfileAssigner::assign(&instrument);
    println!(
        "Assigned Evaluation Profile: {} v{}",
        profile.name(),
        profile.version()
    );
    println!();

    // 3. Extract Active Concepts
    let active_concepts = profile.active_concepts();
    println!("--- Active Concepts ---");
    for concept in &active_concepts {
        println!("  - {:?}", concept);
    }
    println!();

    // 4. Map Metrics for each Concept
    println!("--- Active Metric Models ---");
    for concept in &active_concepts {
        let metrics = profile.metrics_for_concept(concept);
        println!("{:?} Models:", concept);
        for metric in metrics {
            println!("  [x] {}", metric);
        }
    }
    println!("===========================================================");
}
