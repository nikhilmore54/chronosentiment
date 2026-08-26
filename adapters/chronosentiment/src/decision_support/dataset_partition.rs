//! Chronological dataset partition.
//!
//! Domain kinds are development / selection / evaluation. Protocol documents
//! may map those to provenance roles. This module does not search, evolve, or
//! score policies. Outcomes are not inputs.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::csp006_protocol::RESEARCH_UNIVERSE;
use super::enrichment_certify::replay_month_ends_2021_10_to_2024_12;

pub const PARTITION_METHOD: &str = "chronological_partition.contiguous_equal_thirds.v1";
pub const ATOMIC_UNIT: &str = "timestamp";
pub const PARTITION_COUNT: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PartitionKind {
    Development,
    Selection,
    Evaluation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchOutcomeAccess {
    Evolution,
    SelectionFeedback,
    Forbidden,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimePartition {
    pub kind: PartitionKind,
    pub timestamps: Vec<DateTime<Utc>>,
    pub n_timestamps: usize,
    pub n_observations: usize,
    pub inclusive_start: DateTime<Utc>,
    pub exclusive_end: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChronologicalPartition {
    pub method: String,
    pub atomic_unit: String,
    pub instruments: Vec<String>,
    pub n_instruments_per_timestamp: usize,
    pub timestamps: Vec<DateTime<Utc>>,
    pub development: TimePartition,
    pub selection: TimePartition,
    pub evaluation: TimePartition,
    pub tie_break: String,
    pub partition_hash: String,
}

#[derive(Serialize)]
struct PartitionIdentity<'a> {
    method: &'a str,
    atomic_unit: &'a str,
    instruments: &'a [String],
    n_instruments_per_timestamp: usize,
    development: &'a [DateTime<Utc>],
    selection: &'a [DateTime<Utc>],
    evaluation: &'a [DateTime<Utc>],
    tie_break: &'a str,
}

pub fn search_outcome_access(kind: PartitionKind) -> SearchOutcomeAccess {
    match kind {
        PartitionKind::Development => SearchOutcomeAccess::Evolution,
        PartitionKind::Selection => SearchOutcomeAccess::SelectionFeedback,
        PartitionKind::Evaluation => SearchOutcomeAccess::Forbidden,
    }
}

pub fn search_may_use_for_evolution(kind: PartitionKind) -> bool {
    matches!(search_outcome_access(kind), SearchOutcomeAccess::Evolution)
}

pub fn search_may_use_for_selection(kind: PartitionKind) -> bool {
    matches!(
        search_outcome_access(kind),
        SearchOutcomeAccess::SelectionFeedback
    )
}

pub fn search_may_observe_outcomes(kind: PartitionKind) -> bool {
    !matches!(search_outcome_access(kind), SearchOutcomeAccess::Forbidden)
}

/// Equal contiguous thirds of a sorted unique timestamp grid.
/// Remainder timestamps, if any, stay in development so evaluation never gains extra history.
pub fn partition_contiguous_equal_thirds(
    timestamps: &[DateTime<Utc>],
    instruments: &[&str],
) -> Result<ChronologicalPartition, String> {
    if timestamps.is_empty() {
        return Err("timestamp grid is empty".into());
    }
    if instruments.is_empty() {
        return Err("instrument universe is empty".into());
    }
    let mut unique = timestamps.to_vec();
    unique.sort();
    unique.dedup();
    if unique.len() != timestamps.len() {
        return Err("timestamp grid contains duplicates".into());
    }
    for pair in unique.windows(2) {
        if pair[0] >= pair[1] {
            return Err("timestamp grid is not strictly increasing".into());
        }
    }
    let n = unique.len();
    let base = n / PARTITION_COUNT;
    let remainder = n % PARTITION_COUNT;
    if base == 0 {
        return Err("timestamp grid is shorter than three partitions".into());
    }
    let development_n = base + remainder;
    let selection_n = base;
    let evaluation_n = base;
    let tie_break = if remainder == 0 {
        "none_applicable".to_string()
    } else {
        "remainder_timestamps_assigned_to_development".to_string()
    };
    let selection_start = development_n;
    let evaluation_start = development_n + selection_n;
    let development_ts = unique[..selection_start].to_vec();
    let selection_ts = unique[selection_start..evaluation_start].to_vec();
    let evaluation_ts = unique[evaluation_start..].to_vec();
    if development_ts.len() != development_n
        || selection_ts.len() != selection_n
        || evaluation_ts.len() != evaluation_n
    {
        return Err("partition lengths do not cover the grid".into());
    }
    if development_ts.last() >= selection_ts.first() || selection_ts.last() >= evaluation_ts.first()
    {
        return Err("partitions are not strictly chronological".into());
    }
    let n_instruments = instruments.len();
    let instrument_names: Vec<String> = instruments.iter().map(|s| (*s).to_string()).collect();
    let development = time_partition(PartitionKind::Development, development_ts, n_instruments);
    let selection = time_partition(PartitionKind::Selection, selection_ts, n_instruments);
    let evaluation = time_partition(PartitionKind::Evaluation, evaluation_ts, n_instruments);
    if development.exclusive_end > selection.inclusive_start
        || selection.exclusive_end > evaluation.inclusive_start
    {
        return Err("partition windows overlap".into());
    }
    let mut partition = ChronologicalPartition {
        method: PARTITION_METHOD.to_string(),
        atomic_unit: ATOMIC_UNIT.to_string(),
        instruments: instrument_names,
        n_instruments_per_timestamp: n_instruments,
        timestamps: unique,
        development,
        selection,
        evaluation,
        tie_break,
        partition_hash: String::new(),
    };
    partition.partition_hash = compute_partition_hash(&partition);
    Ok(partition)
}

fn time_partition(
    kind: PartitionKind,
    timestamps: Vec<DateTime<Utc>>,
    n_instruments: usize,
) -> TimePartition {
    let inclusive_start = timestamps[0];
    let exclusive_end = timestamps[timestamps.len() - 1] + Duration::seconds(1);
    TimePartition {
        kind,
        n_timestamps: timestamps.len(),
        n_observations: timestamps.len() * n_instruments,
        inclusive_start,
        exclusive_end,
        timestamps,
    }
}

fn compute_partition_hash(partition: &ChronologicalPartition) -> String {
    let payload = PartitionIdentity {
        method: &partition.method,
        atomic_unit: &partition.atomic_unit,
        instruments: &partition.instruments,
        n_instruments_per_timestamp: partition.n_instruments_per_timestamp,
        development: &partition.development.timestamps,
        selection: &partition.selection.timestamps,
        evaluation: &partition.evaluation.timestamps,
        tie_break: &partition.tie_break,
    };
    let bytes = serde_json::to_vec(&payload).expect("partition identity serializes");
    format!("{:x}", Sha256::digest(&bytes))
}

/// Certified seven-instrument month-end grid (CS-P-006-S1 coverage).
pub fn certified_research_partition() -> ChronologicalPartition {
    partition_contiguous_equal_thirds(&replay_month_ends_2021_10_to_2024_12(), &RESEARCH_UNIVERSE)
        .expect("certified 39-timestamp grid partitions")
}

pub fn assign_timestamp(
    partition: &ChronologicalPartition,
    t: DateTime<Utc>,
) -> Option<PartitionKind> {
    if partition.development.timestamps.binary_search(&t).is_ok() {
        Some(PartitionKind::Development)
    } else if partition.selection.timestamps.binary_search(&t).is_ok() {
        Some(PartitionKind::Selection)
    } else if partition.evaluation.timestamps.binary_search(&t).is_ok() {
        Some(PartitionKind::Evaluation)
    } else {
        None
    }
}

/// Every instrument at timestamp T belongs to the same partition.
pub fn timestamp_cohort_is_atomic(
    rows: &[(String, DateTime<Utc>)],
    partition: &ChronologicalPartition,
) -> Result<(), String> {
    let mut by_t: BTreeMap<DateTime<Utc>, BTreeSet<PartitionKind>> = BTreeMap::new();
    for (_instrument, t) in rows {
        let Some(kind) = assign_timestamp(partition, *t) else {
            return Err(format!("{t} is not on the certified timestamp grid"));
        };
        by_t.entry(*t).or_default().insert(kind);
    }
    for (t, kinds) in by_t {
        if kinds.len() != 1 {
            return Err(format!("timestamp {t} split across partitions {kinds:?}"));
        }
    }
    Ok(())
}

pub fn render_partition_manifest(partition: &ChronologicalPartition) -> String {
    let mut md = String::from("# Chronological dataset partition\n\n");
    md.push_str("Domain kinds: **development / selection / evaluation**.\n\n");
    md.push_str("**Authorization:** PASS\n\n");
    md.push_str(&format!("- method: `{}`\n", partition.method));
    md.push_str(&format!("- atomic unit: `{}`\n", partition.atomic_unit));
    md.push_str(&format!(
        "- instruments: {}\n",
        partition.instruments.join(", ")
    ));
    md.push_str(&format!(
        "- instruments per timestamp: {}\n",
        partition.n_instruments_per_timestamp
    ));
    md.push_str(&format!("- timestamps: {}\n", partition.timestamps.len()));
    md.push_str(&format!("- tie-break: {}\n", partition.tie_break));
    md.push_str(&format!(
        "- partition hash: `{}`\n\n",
        partition.partition_hash
    ));
    md.push_str("| Partition | Timestamps | Observations | First timestamp | Last timestamp | Inclusive start | Exclusive end |\n");
    md.push_str("|-----------|------------|--------------|-----------------|----------------|-----------------|---------------|\n");
    for part in [
        &partition.development,
        &partition.selection,
        &partition.evaluation,
    ] {
        md.push_str(&format!(
            "| {:?} | {} | {} | {} | {} | {} | {} |\n",
            part.kind,
            part.n_timestamps,
            part.n_observations,
            part.timestamps[0],
            part.timestamps[part.timestamps.len() - 1],
            part.inclusive_start,
            part.exclusive_end
        ));
    }
    md.push_str("\nSearch may use development outcomes for evolution and selection outcomes for selection feedback only. Evaluation outcomes, performance, and fitness are forbidden to search.\n");
    md
}
