use serde::{Serialize, Deserialize};
use sha2::{Digest, Sha256};
use crate::pipeline::asset_loop::Asset; // assuming Asset type is defined in asset_loop module
use crate::pipeline::execution_params::ExecutionParams; // placeholder for execution params

/// Projection of deterministic orchestration execution.
#[derive(Serialize, Deserialize, PartialEq, Debug, Hash, Clone)]
pub struct ExecutionProjection {
    /// Canonical ordered list of asset identifiers used during orchestration.
    pub ordered_assets: Vec<String>,
    /// SHA‑256 hash of the deterministic execution trace.
    pub execution_trace_hash: String,
    /// Policy used for canonicalization (e.g., "sorted_lexicographically_deduplicated").
    pub canonicalization_policy: String,
    /// Version of the projection schema for migration tracking.
    pub projection_version: String,
    /// Number of assets before canonicalization.
    pub input_asset_count: usize,
    /// Number of assets after canonicalization (duplicates removed).
    pub canonicalized_asset_count: usize,
}

/// Returns a deterministic projection of the asset loop order.
pub(crate) fn asset_loop_order_is_stable(assets: &[Asset]) -> ExecutionProjection {
    // Canonical ordering: sort and dedup
    let mut ordered: Vec<String> = assets.iter().map(|a| a.id.clone()).collect();
    ordered.sort();
    ordered.dedup();
    let json = serde_json::to_string(&ordered).unwrap();
    let hash = Sha256::digest(json.as_bytes());
    let projection = ExecutionProjection {
        ordered_assets: ordered.clone(),
        execution_trace_hash: format!("{:x}", hash),
        canonicalization_policy: "sorted_lexicographically_deduplicated".to_string(),
        projection_version: "1.0.0".to_string(),
        input_asset_count: assets.len(),
        canonicalized_asset_count: ordered.len(),
    };
    debug_assert!(projection.canonicalized_asset_count > 0);
    projection
}

/// Returns a deterministic projection of multi‑asset execution.
pub(crate) fn multi_asset_execution_projection_is_stable(params: &ExecutionParams) -> ExecutionProjection {
    // Canonical ordering: sort and dedup
    let mut ordered: Vec<String> = params.assets.iter().map(|a| a.id.clone()).collect();
    ordered.sort();
    ordered.dedup();
    let json = serde_json::to_string(&ordered).unwrap();
    let hash = Sha256::digest(json.as_bytes());
    let projection = ExecutionProjection {
        ordered_assets: ordered.clone(),
        execution_trace_hash: format!("{:x}", hash),
        canonicalization_policy: "sorted_lexicographically_deduplicated".to_string(),
        projection_version: "1.0.0".to_string(),
        input_asset_count: params.assets.len(),
        canonicalized_asset_count: ordered.len(),
    };
    debug_assert!(projection.canonicalized_asset_count > 0);
    projection
}
