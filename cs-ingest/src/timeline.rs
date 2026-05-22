use crate::frozen_loader::FrozenBar;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct TimelineAlignment {
    pub timestamps: Vec<i64>,
    pub fingerprint: String,
    pub symbol_count: usize,
    pub total_bars: usize,
    /// ts -> symbols present at barrier
    pub coverage: HashMap<i64, Vec<String>>,
}

/// Match Python `build_timeline_fingerprint`: sha256 of sorted comma-separated ts, first 16 hex chars.
pub fn timeline_fingerprint(timestamps: &[i64]) -> String {
    let mut sorted: Vec<i64> = timestamps.to_vec();
    sorted.sort_unstable();
    let joined = sorted
        .iter()
        .map(|t| t.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let digest = Sha256::digest(joined.as_bytes());
    format!("{:x}", digest)[..16].to_string()
}

/// Union of all bar timestamps across symbols (chronosynchrony substrate).
pub fn align_timeline(data: &HashMap<String, Vec<FrozenBar>>) -> TimelineAlignment {
    let mut all_ts: HashSet<i64> = HashSet::new();
    let mut total_bars = 0usize;
    for bars in data.values() {
        total_bars += bars.len();
        for b in bars {
            all_ts.insert(b.ts);
        }
    }
    let mut timestamps: Vec<i64> = all_ts.into_iter().collect();
    timestamps.sort_unstable();

    let mut coverage: HashMap<i64, Vec<String>> = HashMap::new();
    for (sym, bars) in data {
        for b in bars {
            coverage.entry(b.ts).or_default().push(sym.clone());
        }
    }
    for syms in coverage.values_mut() {
        syms.sort();
    }

    let fingerprint = timeline_fingerprint(&timestamps);
    TimelineAlignment {
        symbol_count: data.len(),
        total_bars,
        fingerprint,
        timestamps,
        coverage,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_deterministic() {
        let ts = vec![100_i64, 200, 300];
        assert_eq!(timeline_fingerprint(&ts), timeline_fingerprint(&ts));
    }
}
