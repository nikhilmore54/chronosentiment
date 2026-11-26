//! Gzip JSONL helpers (parity with former Python `archive_dedupe.iter_gzip_jsonl`).

use flate2::read::GzDecoder;
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Yield `(line_no, Ok(record))` or `(line_no, Err(message))` for a gzip JSONL file.
/// On unreadable file, yields a single `(0, Err(...))`.
pub fn iter_gzip_jsonl(path: &Path) -> Vec<(usize, Result<Value, String>)> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => return vec![(0, Err(e.to_string()))],
    };
    let decoder = GzDecoder::new(file);
    let reader = BufReader::new(decoder);
    let mut out = Vec::new();
    for (idx, line) in reader.lines().enumerate() {
        let line_no = idx + 1;
        match line {
            Ok(raw) => {
                let trimmed = raw.trim();
                if trimmed.is_empty() {
                    continue;
                }
                match serde_json::from_str::<Value>(trimmed) {
                    Ok(v) => out.push((line_no, Ok(v))),
                    Err(e) => out.push((line_no, Err(e.to_string()))),
                }
            }
            Err(e) => {
                out.push((0, Err(e.to_string())));
                break;
            }
        }
    }
    out
}
