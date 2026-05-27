//! Serialization-only ratification probe for V-006 Phase B.
//!
//! Bypasses network/runtime ingestion. Exercises:
//!   NormalizedTick -> serde_json -> line bytes -> chronology_hash
//! against frozen fixture excerpts. Does not grant producer authority.

use clap::Parser;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "chronology_serialize_probe")]
#[command(about = "V-006 serialization injection probe (fixture -> emit -> hash)")]
struct Args {
    /// Fixture directory containing substrate_excerpt.jsonl and fixture_meta.json
    #[arg(long)]
    fixture_dir: PathBuf,

    /// Producer label for reporting (capture_daemon | historical_importer | yahoo_importer)
    #[arg(long, default_value = "capture_daemon")]
    producer: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct NormalizedTick {
    symbol: String,
    timestamp: u64,
    price: f64,
    volume: f64,
    is_buyer_maker: bool,
}

#[derive(Debug, Deserialize)]
struct FixtureMeta {
    fixture_id: String,
    excerpt_chronology_hash: String,
}

#[derive(Debug, Serialize)]
struct LineDiff {
    line: usize,
    byte_identical: bool,
    expected_len: usize,
    emitted_len: usize,
}

#[derive(Debug, Serialize)]
struct ProbeReport {
    fixture_id: String,
    producer: String,
    line_count: usize,
    lines_byte_identical: bool,
    first_line_hex_match: bool,
    expected_excerpt_hash: String,
    emitted_excerpt_hash: String,
    hash_identical: bool,
    line_diffs: Vec<LineDiff>,
    suggested_classification: String,
}

fn tick_to_line_bytes(tick: &NormalizedTick) -> Vec<u8> {
    format!("{}\n", serde_json::to_string(tick).unwrap()).into_bytes()
}

fn streaming_chronology_hash(lines: &[Vec<u8>]) -> String {
    let mut hasher = Sha256::new();
    for line in lines {
        hasher.update(line);
    }
    hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect()
}

fn main() {
    let args = Args::parse();

    let excerpt_path = args.fixture_dir.join("substrate_excerpt.jsonl");
    let meta_path = args.fixture_dir.join("fixture_meta.json");
    let first_line_hex_path = args.fixture_dir.join("first_line.hex");

    let meta: FixtureMeta =
        serde_json::from_str(&fs::read_to_string(&meta_path).expect("read fixture_meta.json"))
            .expect("parse fixture_meta.json");

    let input_bytes = fs::read(&excerpt_path).expect("read substrate_excerpt.jsonl");
    let input_lines: Vec<Vec<u8>> = input_bytes
        .split(|&b| b == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| {
            let mut v = line.to_vec();
            v.push(b'\n');
            v
        })
        .collect();

    let mut emitted_lines: Vec<Vec<u8>> = Vec::new();
    let mut line_diffs: Vec<LineDiff> = Vec::new();
    let mut lines_byte_identical = true;

    for (idx, input_line) in input_lines.iter().enumerate() {
        let tick: NormalizedTick = serde_json::from_slice(input_line).unwrap_or_else(|e| {
            panic!("parse tick line {}: {}", idx + 1, e);
        });
        let emitted = tick_to_line_bytes(&tick);
        let identical = emitted == *input_line;
        if !identical {
            lines_byte_identical = false;
        }
        line_diffs.push(LineDiff {
            line: idx + 1,
            byte_identical: identical,
            expected_len: input_line.len(),
            emitted_len: emitted.len(),
        });
        emitted_lines.push(emitted);
    }

    let expected_hash = meta.excerpt_chronology_hash.clone();
    let emitted_hash = streaming_chronology_hash(&emitted_lines);
    let hash_identical = expected_hash == emitted_hash;

    let first_line_hex_match = if first_line_hex_path.exists() && !emitted_lines.is_empty() {
        let expected = fs::read_to_string(&first_line_hex_path)
            .expect("read first_line.hex")
            .trim()
            .to_string();
        let emitted_hex: String = emitted_lines[0]
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();
        expected == emitted_hex
    } else {
        false
    };

    let suggested_classification = if lines_byte_identical && hash_identical && first_line_hex_match {
        "byte_identical".to_string()
    } else if !lines_byte_identical {
        "serialization_drift".to_string()
    } else if !hash_identical {
        "hash_identical_semantic_drift".to_string()
    } else {
        "ratification_blocked".to_string()
    };

    let report = ProbeReport {
        fixture_id: meta.fixture_id,
        producer: args.producer,
        line_count: input_lines.len(),
        lines_byte_identical,
        first_line_hex_match,
        expected_excerpt_hash: expected_hash,
        emitted_excerpt_hash: emitted_hash,
        hash_identical,
        line_diffs,
        suggested_classification,
    };

    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}
