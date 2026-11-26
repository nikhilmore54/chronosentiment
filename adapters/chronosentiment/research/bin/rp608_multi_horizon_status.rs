//! Emit multi-horizon dashboard JSON (Rust port of rp608_multi_horizon_status.py).

use chronosentiment_adapter::research::multi_horizon_status::{
    get_multi_horizon_dashboard_state, workspace_from_manifest,
};
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let workspace = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(workspace_from_manifest);
    let state = get_multi_horizon_dashboard_state(&workspace);
    match serde_json::to_string_pretty(&state) {
        Ok(s) => {
            println!("{s}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
