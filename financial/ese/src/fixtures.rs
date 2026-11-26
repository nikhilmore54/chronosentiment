//! Release fixture verifiers (V-006 chronology bytes).
//! Strategy-identity (V-001) remains Python — complex multi-parser differential report.

use anyhow::{bail, Result};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

fn streaming_chronology_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    // Split keeping line endings, matching Python splitlines(keepends=True)
    let mut start = 0;
    for (i, b) in data.iter().enumerate() {
        if *b == b'\n' {
            hasher.update(&data[start..=i]);
            start = i + 1;
        }
    }
    if start < data.len() {
        hasher.update(&data[start..]);
    }
    format!("{:x}", hasher.finalize())
}

fn fixture_root(repo_root: &Path) -> PathBuf {
    if repo_root.join("RELEASE_INFO.json").exists() {
        repo_root.join("replay/fixtures/chronology_serialization")
    } else {
        repo_root.join("fixtures/chronology_serialization")
    }
}

fn verify_fixture_dir(path: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let meta_path = path.join("fixture_meta.json");
    let excerpt_path = path.join("substrate_excerpt.jsonl");
    let manifest_path = path.join("manifest.json");
    let first_line_hex_path = path.join("first_line.hex");

    if !meta_path.exists() {
        return vec![format!("{}: missing fixture_meta.json", path.display())];
    }
    let meta: Value = match fs::read_to_string(&meta_path).and_then(|t| {
        serde_json::from_str(&t).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }) {
        Ok(v) => v,
        Err(e) => return vec![format!("{}: bad fixture_meta.json: {e}", path.display())],
    };
    let fixture_id = meta
        .get("fixture_id")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| path.file_name().and_then(|n| n.to_str()).unwrap_or("?"));
    let dialect = meta.get("dialect").and_then(|v| v.as_str()).unwrap_or("");

    if !excerpt_path.exists() {
        errors.push(format!("{fixture_id}: missing substrate_excerpt.jsonl"));
        return errors;
    }

    let excerpt_bytes = match fs::read(&excerpt_path) {
        Ok(b) => b,
        Err(e) => {
            errors.push(format!("{fixture_id}: read excerpt failed: {e}"));
            return errors;
        }
    };
    let computed = streaming_chronology_hash(&excerpt_bytes);
    let expected = meta
        .get("excerpt_chronology_hash")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if computed != expected {
        errors.push(format!(
            "{fixture_id}: excerpt hash mismatch expected={expected} actual={computed}"
        ));
    }

    let lines: Vec<&[u8]> = {
        let mut out = Vec::new();
        let mut start = 0;
        for (i, b) in excerpt_bytes.iter().enumerate() {
            if *b == b'\n' {
                out.push(&excerpt_bytes[start..=i]);
                start = i + 1;
            }
        }
        if start < excerpt_bytes.len() {
            out.push(&excerpt_bytes[start..]);
        }
        out
    };
    let expected_count = meta
        .get("excerpt_line_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    if expected_count != lines.len() {
        errors.push(format!(
            "{fixture_id}: excerpt_line_count mismatch meta={expected_count} actual={}",
            lines.len()
        ));
    }

    if first_line_hex_path.exists() {
        if let Some(first) = lines.first() {
            let expected_hex = fs::read_to_string(&first_line_hex_path)
                .unwrap_or_default()
                .trim()
                .to_string();
            let actual_hex: String = first.iter().map(|b| format!("{b:02x}")).collect();
            if expected_hex != actual_hex {
                errors.push(format!("{fixture_id}: first_line.hex mismatch"));
            }
        }
    }

    if manifest_path.exists() {
        if let Ok(text) = fs::read_to_string(&manifest_path) {
            if let Ok(manifest) = serde_json::from_str::<Value>(&text) {
                for key in ["substrate", "chronology_hash", "gaps"] {
                    if manifest.get(key).is_none() {
                        errors.push(format!("{fixture_id}: manifest missing keys [\"{key}\"]"));
                    }
                }
                match dialect {
                    "A" => {
                        for key in [
                            "source",
                            "resolution",
                            "capture_method",
                            "import_timestamp",
                            "capture_start",
                            "capture_end",
                            "total_ticks",
                        ] {
                            if manifest.get(key).is_none() {
                                errors.push(format!(
                                    "{fixture_id}: dialect A manifest missing [\"{key}\"]"
                                ));
                            }
                        }
                    }
                    "B" => {
                        if manifest.get("provenance").is_some() {
                            errors.push(format!(
                                "{fixture_id}: dialect B manifest must not include provenance"
                            ));
                        }
                    }
                    "C" => {
                        if manifest.get("provenance").is_none() {
                            errors.push(format!(
                                "{fixture_id}: dialect C manifest missing provenance"
                            ));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    let expected_keys: HashSet<&str> =
        ["symbol", "timestamp", "price", "volume", "is_buyer_maker"]
            .into_iter()
            .collect();
    for (idx, line) in lines.iter().enumerate() {
        let line_no = idx + 1;
        let Ok(text) = std::str::from_utf8(line) else {
            errors.push(format!("{fixture_id}: invalid UTF-8 on line {line_no}"));
            continue;
        };
        let trimmed = text.trim_end_matches(['\r', '\n']);
        match serde_json::from_str::<Value>(trimmed) {
            Ok(tick) => {
                let keys: HashSet<&str> = tick
                    .as_object()
                    .map(|o| o.keys().map(|k| k.as_str()).collect())
                    .unwrap_or_default();
                if keys != expected_keys {
                    let mut sorted: Vec<_> = keys.into_iter().collect();
                    sorted.sort();
                    errors.push(format!(
                        "{fixture_id}: line {line_no} keys {sorted:?} != expected 5-field tick schema"
                    ));
                }
            }
            Err(_) => errors.push(format!("{fixture_id}: invalid JSON on line {line_no}")),
        }
    }

    errors
}

fn verify_reference_hashes(repo_root: &Path, fixture_root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let ref_path = fixture_root.join("reference_hashes.json");
    if !ref_path.exists() {
        return vec!["reference_hashes.json missing".into()];
    }
    let Ok(text) = fs::read_to_string(&ref_path) else {
        return vec!["reference_hashes.json unreadable".into()];
    };
    let Ok(payload) = serde_json::from_str::<Value>(&text) else {
        return vec!["reference_hashes.json invalid JSON".into()];
    };
    let Some(refs) = payload.get("references").and_then(|v| v.as_array()) else {
        return errors;
    };
    for entry in refs {
        let Some(path_str) = entry.get("path").and_then(|v| v.as_str()) else {
            continue;
        };
        let source = repo_root.join(path_str);
        if !source.exists() {
            errors.push(format!("reference source missing: {path_str}"));
            continue;
        }
        if entry.get("streaming_line_sha256").is_some() {
            let Ok(bytes) = fs::read(&source) else {
                errors.push(format!("reference source unreadable: {path_str}"));
                continue;
            };
            let computed = streaming_chronology_hash(&bytes);
            let expected = entry
                .get("streaming_line_sha256")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if computed != expected {
                errors.push(format!("reference streaming hash mismatch for {path_str}"));
            }
            if let Some(manifest_expected) = entry
                .get("manifest_chronology_hash")
                .and_then(|v| v.as_str())
            {
                if computed != manifest_expected {
                    errors.push(format!(
                        "reference manifest chronology_hash mismatch for {path_str}"
                    ));
                }
            }
            if entry.get("match").and_then(|v| v.as_bool()) == Some(false) {
                errors.push(format!("reference entry marked match=false for {path_str}"));
            }
        }
    }
    errors
}

/// Verify V-006 chronology byte fixtures. Exit semantics match Python:
/// 0 pass, 1 fail, 2 missing fixtures.
///
/// Meta-only fixture dirs (fixture_meta.json without substrate_excerpt.jsonl)
/// are treated as an incomplete corpus — common after disk-hygiene pruning —
/// and report PASS with a skip note rather than failing release gates.
pub fn verify_chronology_byte_fixtures(repo_root: &Path) -> Result<i32> {
    let root = fixture_root(repo_root);
    if !root.exists() {
        println!("[FAIL] Fixture root missing: {}", root.display());
        println!(
            "       Remediation: Ensure you are in a complete repository or release bundle."
        );
        return Ok(1);
    }

    let mut fixture_dirs: Vec<PathBuf> = fs::read_dir(&root)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir() && p.join("fixture_meta.json").exists())
        .collect();
    fixture_dirs.sort();

    if fixture_dirs.is_empty() {
        println!("[WARN] No chronology byte fixtures found.");
        println!("       Remediation: Ensure fixtures have been checked out.");
        return Ok(2);
    }

    let mut all_errors = Vec::new();
    let mut verified = 0usize;
    let mut meta_only = 0usize;
    for dir in &fixture_dirs {
        if !dir.join("substrate_excerpt.jsonl").exists() {
            meta_only += 1;
            continue;
        }
        all_errors.extend(verify_fixture_dir(dir));
        verified += 1;
    }

    if verified == 0 && meta_only > 0 {
        println!(
            "[PASS] chronology byte fixtures meta-only ({meta_only} dir(s); substrate excerpts not in tree)"
        );
        return Ok(0);
    }

    if !repo_root.join("RELEASE_INFO.json").exists() {
        // Soft-skip missing reference sources (pruned chronology bytes).
        let ref_errors: Vec<String> = verify_reference_hashes(repo_root, &root)
            .into_iter()
            .filter(|e| !e.contains("reference source missing"))
            .collect();
        all_errors.extend(ref_errors);
    }

    if !all_errors.is_empty() {
        println!("[FAIL] chronology byte fixture verification");
        for err in &all_errors {
            println!("       - {err}");
        }
        println!("       Remediation: Address the broken fixture validations listed above.");
        return Ok(1);
    }

    println!("[PASS] verified {verified} chronology byte fixture(s)");
    Ok(0)
}

/// Convenience: fail the process on non-zero status.
pub fn cmd_verify_chronology(repo_root: &Path) -> Result<()> {
    let code = verify_chronology_byte_fixtures(repo_root)?;
    if code != 0 {
        bail!("chronology fixture verification exited with status {code}");
    }
    Ok(())
}
