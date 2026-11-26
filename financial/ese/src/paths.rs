//! Shared path helpers for batch archive layout.

use std::path::{Path, PathBuf};

/// Resolve the archive directory for a cohort batch run.
///
/// Layout matches the former Python `resolve_archive_dir`:
/// - shared → `{archive_root}`
/// - labeled → `{archive_root}/batches/batch_NNN/runs/{label}`
/// - unlabeled → `{archive_root}/batches/batch_NNN`
pub fn resolve_archive_dir(
    archive_root: &Path,
    batch_id: u32,
    shared: bool,
    run_label: &str,
) -> PathBuf {
    if shared {
        return archive_root.to_path_buf();
    }
    let base = archive_root.join(format!("batches/batch_{batch_id:03}"));
    if run_label.is_empty() {
        base
    } else {
        base.join("runs").join(run_label)
    }
}

pub fn cohort_file(batch_id: u32) -> PathBuf {
    PathBuf::from(format!("cohorts/batch_{batch_id:03}.txt"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn labeled_archive_path() {
        let p = resolve_archive_dir(Path::new("state_archive"), 3, false, "replay_equiv");
        assert_eq!(
            p,
            PathBuf::from("state_archive/batches/batch_003/runs/replay_equiv")
        );
    }

    #[test]
    fn shared_archive_path() {
        let p = resolve_archive_dir(Path::new("state_archive"), 3, true, "ignored");
        assert_eq!(p, PathBuf::from("state_archive"));
    }
}
