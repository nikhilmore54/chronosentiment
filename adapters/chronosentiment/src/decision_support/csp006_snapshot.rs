//! Disposable CS-P-006 7-instrument research snapshot.
//!
//! Not B4. Not B5. Not a G-GATE dataset. Outcomes are not used to construct state.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use chrono::{DateTime, Duration, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::ingestion::yahoo::YahooHistoricalBar;
use crate::metrics::concepts::Concept;
use crate::reasoning::assessment::FactorAvailability;

use super::csp006_protocol::{CERTIFIED_FIVE_INSTRUMENT_SNAPSHOT, RESEARCH_UNIVERSE};
use super::enrichment_certify::{
    assess_from_bars_at_t, bars_at_or_before, certify_snapshot, load_yahoo_cache_dir,
    replay_month_ends_2021_10_to_2024_12, EnrichmentCertification,
};
use super::forward_tick::instrument_id_for;
use super::policy::BaselineTrendMappingPolicy;
use super::replay::{decide_from_inputs, ReplayAssessment, ReplayInputs};
use super::{DecisionAction, TradingDecision};

pub const SNAPSHOT_KIND: &str = "csp006.research_snapshot.7instrument";
pub const SNAPSHOT_PRODUCER: &str = "csp006.snapshot.v0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotRow {
    pub instrument: String,
    pub as_of: DateTime<Utc>,
    pub evaluation_timestamp: DateTime<Utc>,
    pub instrument_id: Uuid,
    pub assessment_id: Uuid,
    pub signature_hash: String,
    pub trend_available: bool,
    pub momentum_available: bool,
    pub volatility_available: bool,
    pub action: DecisionAction,
    pub decision_id: Uuid,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResearchSnapshot {
    pub kind: String,
    pub n_rows: usize,
    pub instruments: Vec<String>,
    pub rows: Vec<SnapshotRow>,
    pub identity_hash: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SnapshotCertification {
    pub result: String,
    pub discovery_ready: String,
    pub n_rows: usize,
    pub instruments: Vec<String>,
    pub duplicate_instrument_t: u32,
    pub five_instrument_signature_mismatches: u32,
    pub tmv_complete_rows: u32,
    pub tmv_incomplete_rows: u32,
    pub identity_hash: String,
    pub enrichment: EnrichmentCertification,
    pub failures: Vec<String>,
}

pub fn load_required_yahoo_cache(
    dir: &Path,
) -> Result<BTreeMap<String, Vec<YahooHistoricalBar>>, String> {
    let cache = load_yahoo_cache_dir(dir)?;
    for ticker in RESEARCH_UNIVERSE {
        if !cache.contains_key(ticker) {
            return Err(format!("yahoo cache missing {ticker}"));
        }
    }
    Ok(cache)
}

pub fn build_research_snapshot(
    cache: &BTreeMap<String, Vec<YahooHistoricalBar>>,
) -> Result<ResearchSnapshot, String> {
    let grid = replay_month_ends_2021_10_to_2024_12();
    let mut rows = Vec::new();
    let mut seen = BTreeSet::new();
    for ticker in RESEARCH_UNIVERSE {
        let bars = cache
            .get(ticker)
            .ok_or_else(|| format!("yahoo cache missing {ticker}"))?;
        let instrument_id = instrument_id_for(ticker);
        for &t in &grid {
            if !seen.insert((ticker.to_string(), t)) {
                return Err(format!("duplicate ({ticker}, {t}) during construction"));
            }
            let (mut profile, _n, max_from) = assess_from_bars_at_t(bars, t, instrument_id);
            if let Some(max_from) = max_from {
                if max_from > t {
                    return Err(format!("{ticker} at {t} used bar {max_from} > T"));
                }
            }
            profile.metadata.evaluation_timestamp = t;
            profile.metadata.created_at = t + Duration::seconds(1);
            profile.instrument_id = Some(instrument_id);
            let assessment_id = stable_id(&format!("csp006.assessment.{ticker}.{t}"));
            profile.metadata.artifact_id = assessment_id;
            let signature_hash = profile.to_hash();
            let decision = decide_from_inputs(
                ReplayInputs {
                    instrument_id,
                    as_of: t,
                    engine_version: "unfrozen-dev".to_string(),
                    produced_by: SNAPSHOT_PRODUCER.to_string(),
                    assessments: vec![ReplayAssessment {
                        id: assessment_id,
                        evaluation_timestamp: t,
                        signature_hash: signature_hash.clone(),
                        profile: profile.clone(),
                    }],
                    lake_decisions: vec![],
                    observations: vec![],
                },
                &BaselineTrendMappingPolicy,
            )
            .map_err(|e| format!("decide {ticker} {t}: {e}"))?;
            rows.push(SnapshotRow {
                instrument: ticker.to_string(),
                as_of: t,
                evaluation_timestamp: profile.metadata.evaluation_timestamp,
                instrument_id,
                assessment_id,
                signature_hash,
                trend_available: available(&profile, Concept::Trend),
                momentum_available: available(&profile, Concept::Momentum),
                volatility_available: available(&profile, Concept::Volatility),
                action: decision.action,
                decision_id: decision.decision_id,
            });
            let _: TradingDecision = decision;
        }
    }
    rows.sort_by(|a, b| a.instrument.cmp(&b.instrument).then(a.as_of.cmp(&b.as_of)));
    let identity_hash = snapshot_identity_hash(&rows);
    Ok(ResearchSnapshot {
        kind: SNAPSHOT_KIND.to_string(),
        n_rows: rows.len(),
        instruments: RESEARCH_UNIVERSE.iter().map(|s| (*s).to_string()).collect(),
        rows,
        identity_hash,
    })
}

fn available(
    profile: &crate::reasoning::assessment::AssessmentProfile,
    concept: Concept,
) -> bool {
    profile
        .factor_status
        .iter()
        .any(|s| s.concept == concept && s.availability == FactorAvailability::Available)
}

fn snapshot_identity_hash(rows: &[SnapshotRow]) -> String {
    let payload: Vec<_> = rows
        .iter()
        .map(|r| {
            (
                &r.instrument,
                r.as_of,
                &r.signature_hash,
                r.decision_id,
                r.action,
            )
        })
        .collect();
    let bytes = serde_json::to_vec(&payload).expect("snapshot identity serializes");
    format!("{:x}", Sha256::digest(&bytes))
}

fn stable_id(tag: &str) -> Uuid {
    let digest = Sha256::digest(tag.as_bytes());
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    Uuid::from_bytes(bytes)
}

pub fn parse_enrichment_identity_file(
    text: &str,
) -> Result<BTreeMap<(String, DateTime<Utc>), String>, String> {
    let mut out = BTreeMap::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split('\t');
        let symbol = parts
            .next()
            .ok_or_else(|| format!("identity line {i} missing symbol"))?
            .to_string();
        let ts = parts
            .next()
            .ok_or_else(|| format!("identity line {i} missing timestamp"))?;
        let hash = parts
            .next()
            .ok_or_else(|| format!("identity line {i} missing hash"))?
            .to_string();
        let parsed = DateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S%:z")
            .or_else(|_| DateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S%z"))
            .map_err(|e| format!("identity line {i} timestamp {ts}: {e}"))?
            .with_timezone(&Utc);
        out.insert((symbol, parsed), hash);
    }
    Ok(out)
}

pub fn certify_research_snapshot(
    snapshot: &ResearchSnapshot,
    cache: &BTreeMap<String, Vec<YahooHistoricalBar>>,
    five_instrument_identity: Option<&BTreeMap<(String, DateTime<Utc>), String>>,
) -> SnapshotCertification {
    let mut failures = Vec::new();
    let expected: BTreeSet<&str> = RESEARCH_UNIVERSE.iter().copied().collect();
    let got: BTreeSet<&str> = snapshot.instruments.iter().map(|s| s.as_str()).collect();
    if expected != got {
        failures.push(format!("instruments {got:?} != {expected:?}"));
    }

    let mut seen = BTreeSet::new();
    let mut duplicate_instrument_t = 0u32;
    let mut tmv_complete_rows = 0u32;
    let mut tmv_incomplete_rows = 0u32;
    for row in &snapshot.rows {
        if row.as_of != row.evaluation_timestamp {
            failures.push(format!(
                "{} evaluation_timestamp {} != as_of {}",
                row.instrument, row.evaluation_timestamp, row.as_of
            ));
        }
        if !seen.insert((row.instrument.clone(), row.as_of)) {
            duplicate_instrument_t += 1;
            failures.push(format!("duplicate ({}, {})", row.instrument, row.as_of));
        }
        if row.trend_available && row.momentum_available && row.volatility_available {
            tmv_complete_rows += 1;
        } else {
            tmv_incomplete_rows += 1;
        }
        if let Some(bars) = cache.get(&row.instrument) {
            let leaked = bars_at_or_before(bars, row.as_of);
            if leaked.iter().any(|b| {
                Utc.timestamp_opt(b.timestamp, 0)
                    .single()
                    .map(|ts| ts > row.as_of)
                    .unwrap_or(false)
            }) {
                failures.push(format!(
                    "{} at {} reconstruction included a bar after T",
                    row.instrument, row.as_of
                ));
            }
        } else {
            failures.push(format!("no yahoo cache for {}", row.instrument));
        }
    }
    if duplicate_instrument_t != 0 {
        failures.push(format!(
            "duplicate (instrument,T) count {duplicate_instrument_t}"
        ));
    }

    let grid_n = replay_month_ends_2021_10_to_2024_12().len();
    let expected_rows = RESEARCH_UNIVERSE.len() * grid_n;
    if snapshot.n_rows != expected_rows {
        failures.push(format!(
            "n_rows {} != 7×{} ({expected_rows})",
            snapshot.n_rows, grid_n
        ));
    }

    let mut five_instrument_signature_mismatches = 0u32;
    if let Some(prior) = five_instrument_identity {
        for ticker in CERTIFIED_FIVE_INSTRUMENT_SNAPSHOT {
            for row in snapshot.rows.iter().filter(|r| r.instrument == *ticker) {
                match prior.get(&(row.instrument.clone(), row.as_of)) {
                    Some(expected_hash) if expected_hash == &row.signature_hash => {}
                    Some(_) => {
                        five_instrument_signature_mismatches += 1;
                        failures.push(format!(
                            "{} at {} signature does not match CS-P-004-E1-S1",
                            row.instrument, row.as_of
                        ));
                    }
                    None => {
                        five_instrument_signature_mismatches += 1;
                        failures.push(format!(
                            "{} at {} missing from CS-P-004-E1-S1 identity",
                            row.instrument, row.as_of
                        ));
                    }
                }
            }
        }
    }

    let mut labels = BTreeMap::new();
    let mut profiles = Vec::new();
    let mut hashes = Vec::new();
    for row in &snapshot.rows {
        labels.insert(row.instrument_id, row.instrument.clone());
        let Some(bars) = cache.get(&row.instrument) else {
            continue;
        };
        let (mut profile, _, _) = assess_from_bars_at_t(bars, row.as_of, row.instrument_id);
        profile.metadata.evaluation_timestamp = row.as_of;
        profile.metadata.created_at = row.as_of + Duration::seconds(1);
        profile.instrument_id = Some(row.instrument_id);
        profile.metadata.artifact_id = row.assessment_id;
        hashes.push(row.signature_hash.clone());
        profiles.push(profile);
    }
    let enrichment = certify_snapshot(
        &profiles,
        &labels,
        &hashes,
        snapshot.n_rows as i64,
        0,
        0,
        Some(cache),
    );
    if enrichment.result != "PASS" {
        failures.extend(enrichment.failures.iter().cloned());
    }

    let discovery_ready = if tmv_incomplete_rows == 0 && failures.is_empty() {
        "READY"
    } else {
        "NOT_READY"
    };
    let result = if failures.is_empty() {
        "PASS"
    } else {
        "FAIL"
    };
    SnapshotCertification {
        result: result.to_string(),
        discovery_ready: discovery_ready.to_string(),
        n_rows: snapshot.n_rows,
        instruments: snapshot.instruments.clone(),
        duplicate_instrument_t,
        five_instrument_signature_mismatches,
        tmv_complete_rows,
        tmv_incomplete_rows,
        identity_hash: snapshot.identity_hash.clone(),
        enrichment,
        failures,
    }
}

pub fn repeated_identity_matches(a: &ResearchSnapshot, b: &ResearchSnapshot) -> bool {
    a.identity_hash == b.identity_hash
        && a.rows.len() == b.rows.len()
        && a.rows.iter().zip(b.rows.iter()).all(|(x, y)| {
            x.signature_hash == y.signature_hash && x.decision_id == y.decision_id
        })
}

pub fn render_snapshot_certification(cert: &SnapshotCertification) -> String {
    let mut md = String::from("# CS-P-006 7-instrument research snapshot — certification\n\n");
    md.push_str(&format!("**Result:** {}\n\n", cert.result));
    md.push_str(&format!("**Discovery-ready:** {}\n\n", cert.discovery_ready));
    md.push_str("Disposable CS-P-006 research universe. **Not B4. Not B5.** Not G-GATE. Outcomes were not consumed during state construction.\n\n");
    md.push_str(&format!("- rows: {}\n", cert.n_rows));
    md.push_str(&format!("- instruments: {}\n", cert.instruments.join(", ")));
    md.push_str(&format!(
        "- duplicate (instrument,T): {}\n",
        cert.duplicate_instrument_t
    ));
    md.push_str(&format!(
        "- five-instrument signature mismatches vs CS-P-004-E1-S1: {}\n",
        cert.five_instrument_signature_mismatches
    ));
    md.push_str(&format!("- TMV-complete rows: {}\n", cert.tmv_complete_rows));
    md.push_str(&format!(
        "- TMV-incomplete rows: {}\n",
        cert.tmv_incomplete_rows
    ));
    md.push_str(&format!("- identity hash: `{}`\n\n", cert.identity_hash));
    if cert.failures.is_empty() {
        md.push_str("No snapshot certification failures.\n");
    } else {
        md.push_str("## Failures\n\n");
        for f in &cert.failures {
            md.push_str(&format!("- {f}\n"));
        }
    }
    md
}

pub fn write_identity_file(snapshot: &ResearchSnapshot) -> String {
    let mut lines = String::new();
    for row in &snapshot.rows {
        lines.push_str(&format!(
            "{}\t{}\t{}\n",
            row.instrument, row.as_of, row.signature_hash
        ));
    }
    lines
}
