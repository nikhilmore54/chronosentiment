// deterministic sweep projection test
use sha2::{Digest, Sha256};
use serde::Serialize;
use serde_json;
use chronosentiment_strategies::pipeline::sweep::run_threshold_sweep;
use chronosentiment_strategies::pipeline::reporting::ThresholdSweepRow;

fn stable_hash<T: Serialize>(value: &T) -> String {
    let bytes = serde_json::to_vec(value).expect("serialization should succeed");
    let digest = Sha256::digest(&bytes);
    digest.iter().map(|b| format!("{:02x}", b)).collect::<String>()
}

#[test]
fn threshold_sweep_projection_is_stable() {
    // deterministic fixtures
    let assets = vec!["BTC".to_string(), "ETH".to_string()];
    let global_lambda = 0.5_f64;
    let confidence_floor = 0.30_f64;
    let score_floor = 0.40_f64;

    // first run
    let rows_run_1 = run_threshold_sweep(
        assets.clone(),
        global_lambda,
        &[confidence_floor],
        &[score_floor],
    );
    // second run
    let rows_run_2 = run_threshold_sweep(
        assets,
        global_lambda,
        &[confidence_floor],
        &[score_floor],
    );

    // structural and serialization equality
    assert_eq!(rows_run_1, rows_run_2, "Sweep rows differ between runs");
    let json_1 = serde_json::to_string_pretty(&rows_run_1).expect("serialization should succeed");
    let json_2 = serde_json::to_string_pretty(&rows_run_2).expect("serialization should succeed");
    assert_eq!(json_1, json_2, "Serialized sweep rows differ between runs");

    // deterministic ordering checks
    assert!(rows_run_1
        .windows(2)
        .all(|w| w[0].confidence_floor <= w[1].confidence_floor),
        "Confidence floor ordering not deterministic");
    assert!(rows_run_1
        .windows(2)
        .all(|w| w[0].score_floor <= w[1].score_floor),
        "Score floor ordering not deterministic");
    assert!(rows_run_1
        .windows(2)
        .all(|w| w[0].participation <= w[1].participation),
        "Participation ordering not deterministic");

    // serialization stability via hashing
    let hash_1 = stable_hash(&rows_run_1);
    let hash_2 = stable_hash(&rows_run_2);
    assert_eq!(hash_1, hash_2, "Stable hash of sweep output differed");
}
