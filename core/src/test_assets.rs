use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestAssetsPathError {
    message: String,
}

impl std::fmt::Display for TestAssetsPathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TestAssetsPathError {}

fn invalid_path(message: impl Into<String>) -> TestAssetsPathError {
    TestAssetsPathError {
        message: message.into(),
    }
}

/// Lawful test-asset root resolver (V-008).
///
/// Precedence:
/// 1. `TEST_ASSETS_PATH` when set and non-empty
/// 2. `{CARGO_MANIFEST_DIR}/../test_assets` from the `chronosentiment_core` crate
pub fn resolve_test_assets_dir() -> Result<PathBuf, TestAssetsPathError> {
    if let Ok(raw) = env::var("TEST_ASSETS_PATH") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return validate_directory(&PathBuf::from(trimmed), "TEST_ASSETS_PATH");
        }
    }

    let default = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../test_assets");
    validate_directory(&default, "default manifest-relative test_assets")
}

fn validate_directory(path: &Path, label: &str) -> Result<PathBuf, TestAssetsPathError> {
    if !path.exists() {
        return Err(invalid_path(format!(
            "{label} path does not exist: {}",
            path.display()
        )));
    }
    if !path.is_dir() {
        return Err(invalid_path(format!(
            "{label} path is not a directory: {}",
            path.display()
        )));
    }
    Ok(path.canonicalize().unwrap_or_else(|_| path.to_path_buf()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_manifest_relative_resolution() {
        let path = resolve_test_assets_dir().expect("test_assets should resolve in repo checkout");
        assert!(
            path.ends_with("test_assets"),
            "expected test_assets directory, got {}",
            path.display()
        );
    }
}
