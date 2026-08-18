//! Historical Evidence Store — HDV-001 frozen analogue matching (v0) and
//! REC-001-H ticker-specific analogue store (v1).
//!
//! # Governance
//! HDV-001 is a FROZEN evidence base. It must not be re-tuned or extended here.
//! HDV-002 is a separate programme for discovering improved risk boundaries.
//! When HDV-002 produces a validated policy, it feeds Recommendation Engine v1.
//!
//! # Analogue Matching (deterministic, two-level fallback)
//!
//! Level 1 — Narrow: direction + coralys_state (trend × momentum)
//!   Requires sample_size >= MIN_NARROW_SAMPLE (15)
//!
//! Level 2 — Broad: coralys_state only (direction-agnostic)
//!   Used only when Level 1 < MIN_NARROW_SAMPLE
//!   Requires sample_size >= MIN_BROAD_SAMPLE (15)
//!
//! Insufficient: sample_size < MIN_BROAD_SAMPLE — legitimate outcome, not an error.
//!
//! There is NO silent broadening beyond these two levels.
//! The analogue key records which level was used for full auditability.
//!
//! # Evidence Classification Thresholds (v0, frozen with HDV-001)
//!
//! These thresholds are documented here and must not be changed without a new
//! evidence version and a corresponding recommendation_policy_version bump.
//!
//!   Favourable   : target_before_risk_rate >= 0.40 AND sample_size >= 30
//!   Mixed        : target_before_risk_rate >= 0.30 AND sample_size >= 15
//!   Unfavourable : target_before_risk_rate <  0.30 AND sample_size >= 15
//!   Insufficient : sample_size < 15 (regardless of rate)
//!
//! # Important: rates ≠ probabilities
//! Historical target-before-risk rates describe what happened in comparable
//! past decisions. They are NOT forward probabilities of success. Do not
//! present them as such in any downstream display.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Minimum sample sizes (versioned with HDV-001 policy v0)
// ---------------------------------------------------------------------------

/// Minimum analogues required for a narrow (direction + state) match.
const MIN_NARROW_SAMPLE: usize = 15;

/// Minimum analogues required for a broad (state-only) match.
const MIN_BROAD_SAMPLE: usize = 15;

// ---------------------------------------------------------------------------
// Evidence classification thresholds (versioned with HDV-001 policy v0)
// ---------------------------------------------------------------------------

/// target_before_risk_rate threshold for Favourable classification.
const FAVOURABLE_RATE_THRESHOLD: f64 = 0.40;

/// Minimum sample size for Favourable classification.
const FAVOURABLE_MIN_SAMPLE: usize = 30;

/// target_before_risk_rate threshold for Mixed classification (lower bound).
const MIXED_RATE_THRESHOLD: f64 = 0.30;

/// Minimum sample size for Mixed or Unfavourable classification.
const MIXED_MIN_SAMPLE: usize = 15;

// ---------------------------------------------------------------------------
// Analogue key — the full matching key for auditability
// ---------------------------------------------------------------------------

/// The complete key used to select analogues. Stored in [`HistoricalEvidence`]
/// so callers can audit exactly why observations were considered comparable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalogueKey {
    /// Direction filter applied ("LONG", "SHORT", or "ANY" for broad fallback).
    pub direction: String,
    /// Coralys state: "{trend}_{momentum}" (e.g. "Bullish_Positive").
    pub coralys_state: String,
    /// Whether the narrow (direction-filtered) match was used.
    /// false = fell back to state-only (direction-agnostic) pool.
    pub narrow_match: bool,
    /// Number of comparable historical decisions found.
    pub sample_size: usize,
}

// ---------------------------------------------------------------------------
// Evidence classification
// ---------------------------------------------------------------------------

/// Evidence classification for a set of historical analogues.
///
/// Thresholds are documented in the module-level doc comment and must not
/// be changed without bumping `recommendation_policy_version`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceClass {
    /// target_before_risk_rate >= 0.40 AND sample_size >= 30
    Favourable,
    /// target_before_risk_rate >= 0.30 AND sample_size >= 15
    Mixed,
    /// target_before_risk_rate < 0.30 AND sample_size >= 15
    Unfavourable,
    /// sample_size < 15 — legitimate outcome, not an error
    Insufficient,
}

impl EvidenceClass {
    pub fn classify(rate: f64, sample_size: usize) -> Self {
        if sample_size < MIXED_MIN_SAMPLE {
            return EvidenceClass::Insufficient;
        }
        if rate >= FAVOURABLE_RATE_THRESHOLD && sample_size >= FAVOURABLE_MIN_SAMPLE {
            EvidenceClass::Favourable
        } else if rate >= MIXED_RATE_THRESHOLD {
            EvidenceClass::Mixed
        } else {
            EvidenceClass::Unfavourable
        }
    }
}

// ---------------------------------------------------------------------------
// Historical evidence result
// ---------------------------------------------------------------------------

/// The result of looking up historical analogues for a decision.
/// All fields are present for auditability; the engine uses them for ranking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalEvidence {
    /// The key that was used to select analogues (for audit trail).
    pub analogue_key: AnalogueKey,
    /// Number of comparable historical decisions found.
    pub sample_size: usize,
    /// Fraction of analogues where target was hit before risk (not a probability).
    pub target_before_risk_rate: f64,
    /// Fraction of analogues where risk was hit before target.
    pub risk_before_target_rate: f64,
    /// Fraction of analogues that reached the horizon without hitting either level.
    pub horizon_rate: f64,
    /// Median max-favourable-excursion across analogues (fraction, e.g. 0.031 = 3.1%).
    pub median_mfe: f64,
    /// Median max-adverse-excursion across analogues (fraction, negative, e.g. -0.014).
    pub median_mae: f64,
    /// Median sessions to target (None when no analogues hit target).
    pub median_sessions_to_target: Option<f64>,
    /// Evidence classification derived from rate + sample_size.
    pub evidence_class: EvidenceClass,
}

// ---------------------------------------------------------------------------
// Raw HDV-001 outcome record (for deserialising the frozen JSON file)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct HdvOutcomeRecord {
    pub decision_id: String,
    pub direction: String,
    pub coralys_trend: String,
    pub coralys_momentum: String,
    pub observation_status: String,
    pub outcome: Option<String>,
    pub time_to_target: Option<f64>,
    pub mfe_10: Option<f64>,
    pub mae_10: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct HdvOutcomesFile {
    outcomes: Vec<HdvOutcomeRecord>,
}

// ---------------------------------------------------------------------------
// Evidence store
// ---------------------------------------------------------------------------

/// Pre-built index of HDV-001 evidence, keyed for fast analogue lookup.
///
/// Build once at server startup via [`EvidenceStore::from_json`]; the store
/// is immutable thereafter.
pub struct EvidenceStore {
    /// All COMPLETE outcome records, indexed by (direction, coralys_state).
    narrow_index: HashMap<(String, String), Vec<OutcomeStats>>,
    /// All COMPLETE outcome records, indexed by coralys_state only (broad fallback).
    broad_index: HashMap<String, Vec<OutcomeStats>>,
}

#[derive(Debug, Clone)]
struct OutcomeStats {
    outcome: String,
    mfe_10: f64,
    mae_10: f64,
    time_to_target: Option<f64>,
}

impl EvidenceStore {
    /// Load an [`EvidenceStore`] from a file path on disk.
    ///
    /// Reads the file and delegates to [`EvidenceStore::from_json`].
    /// Returns a boxed error so callers can use `?` without depending on a
    /// specific error type.
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let store = Self::from_json(&contents)?;
        Ok(store)
    }

    /// Build an [`EvidenceStore`] from the raw HDV-001 outcomes JSON bytes.
    ///
    /// Only COMPLETE decisions are indexed. Incomplete observations are excluded
    /// to avoid survivorship bias in the evidence.
    pub fn from_json(outcomes_json: &str) -> Result<Self, serde_json::Error> {
        let file: HdvOutcomesFile = serde_json::from_str(outcomes_json)?;

        let mut narrow_index: HashMap<(String, String), Vec<OutcomeStats>> = HashMap::new();
        let mut broad_index: HashMap<String, Vec<OutcomeStats>> = HashMap::new();

        for record in file.outcomes {
            // Only COMPLETE decisions are used for evidence.
            if record.observation_status != "COMPLETE" {
                continue;
            }
            let outcome = match &record.outcome {
                Some(o) => o.clone(),
                None => continue,
            };
            let stats = OutcomeStats {
                outcome: outcome.clone(),
                mfe_10: record.mfe_10.unwrap_or(0.0),
                mae_10: record.mae_10.unwrap_or(0.0),
                time_to_target: record.time_to_target,
            };
            let state = format!("{}_{}", record.coralys_trend, record.coralys_momentum);
            let dir = record.direction.to_uppercase();

            narrow_index
                .entry((dir.clone(), state.clone()))
                .or_default()
                .push(stats.clone());
            broad_index
                .entry(state)
                .or_default()
                .push(stats);
        }

        Ok(EvidenceStore {
            narrow_index,
            broad_index,
        })
    }

    /// Look up historical evidence for a given decision.
    ///
    /// Matching hierarchy (deterministic, no silent broadening):
    ///   Level 1 (narrow): direction + coralys_state — if sample >= MIN_NARROW_SAMPLE
    ///   Level 2 (broad):  coralys_state only         — if Level 1 < MIN_NARROW_SAMPLE
    ///   Insufficient:     if Level 2 < MIN_BROAD_SAMPLE
    pub fn for_decision(
        &self,
        direction: &str,
        trend: &str,
        momentum: &str,
    ) -> HistoricalEvidence {
        let coralys_state = format!("{}_{}", trend, momentum);
        let dir = direction.to_uppercase();

        // Level 1 — narrow match
        let narrow_key = (dir.clone(), coralys_state.clone());
        let narrow_pool = self.narrow_index.get(&narrow_key);
        let narrow_size = narrow_pool.map(|v| v.len()).unwrap_or(0);

        let (pool, narrow_match) = if narrow_size >= MIN_NARROW_SAMPLE {
            (narrow_pool.unwrap().as_slice(), true)
        } else {
            // Level 2 — broad fallback
            let broad_pool = self.broad_index.get(&coralys_state);
            let broad_size = broad_pool.map(|v| v.len()).unwrap_or(0);
            if broad_size >= MIN_BROAD_SAMPLE {
                (broad_pool.unwrap().as_slice(), false)
            } else {
                // Insufficient
                let analogue_key = AnalogueKey {
                    direction: dir,
                    coralys_state,
                    narrow_match: false,
                    sample_size: broad_size,
                };
                return HistoricalEvidence {
                    analogue_key,
                    sample_size: broad_size,
                    target_before_risk_rate: 0.0,
                    risk_before_target_rate: 0.0,
                    horizon_rate: 0.0,
                    median_mfe: 0.0,
                    median_mae: 0.0,
                    median_sessions_to_target: None,
                    evidence_class: EvidenceClass::Insufficient,
                };
            }
        };

        let sample_size = pool.len();
        let analogue_key = AnalogueKey {
            direction: if narrow_match { dir } else { "ANY".to_string() },
            coralys_state,
            narrow_match,
            sample_size,
        };

        // Compute rates
        let mut target_count = 0usize;
        let mut risk_count = 0usize;
        let mut horizon_count = 0usize;
        let mut mfe_values: Vec<f64> = Vec::with_capacity(sample_size);
        let mut mae_values: Vec<f64> = Vec::with_capacity(sample_size);
        let mut sessions_to_target: Vec<f64> = Vec::new();

        for s in pool {
            match s.outcome.as_str() {
                "TARGET_BEFORE_RISK" => {
                    target_count += 1;
                    if let Some(t) = s.time_to_target {
                        sessions_to_target.push(t);
                    }
                }
                "RISK_BEFORE_TARGET" => risk_count += 1,
                _ => horizon_count += 1,
            }
            mfe_values.push(s.mfe_10);
            mae_values.push(s.mae_10);
        }

        let n = sample_size as f64;
        let target_before_risk_rate = target_count as f64 / n;
        let risk_before_target_rate = risk_count as f64 / n;
        let horizon_rate = horizon_count as f64 / n;
        let median_mfe = median(&mut mfe_values);
        let median_mae = median(&mut mae_values);
        let median_sessions_to_target = if sessions_to_target.is_empty() {
            None
        } else {
            Some(median(&mut sessions_to_target))
        };

        let evidence_class = EvidenceClass::classify(target_before_risk_rate, sample_size);

        HistoricalEvidence {
            analogue_key,
            sample_size,
            target_before_risk_rate,
            risk_before_target_rate,
            horizon_rate,
            median_mfe,
            median_mae,
            median_sessions_to_target,
            evidence_class,
        }
    }
}

// ---------------------------------------------------------------------------
// Utility
// ---------------------------------------------------------------------------

fn median(values: &mut Vec<f64>) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = values.len() / 2;
    if values.len() % 2 == 0 {
        (values[mid - 1] + values[mid]) / 2.0
    } else {
        values[mid]
    }
}

// ---------------------------------------------------------------------------
// REC-001-H v1 — Ticker-specific analogue store (Rec001hStore)
// ---------------------------------------------------------------------------
//
// Loads the 101-ticker JSONL evidence base produced by the REC-001-H pipeline.
// Implements the ARCH-006 §6.2 graceful degradation hierarchy:
//
//   Level 1 (exact):      ticker + direction + trend + momentum + vol_regime + vol_regime
//   Level 2 (relax vol):  ticker + direction + trend + momentum + vol_regime (any volume)
//   Level 3 (relax both): ticker + direction + trend + momentum (any vol + volume)
//   Level 4 (state only): ticker + direction (any state conditions)
//   Insufficient:         < MIN_V1_SAMPLE analogues at any level → NO_TRADE
//
// Adaptive geometry:
//   target_pct  = 25th-percentile of mfe_pct[4] (session 5, conservative)
//   risk_pct    = median of |mae_pct[4]| (session 5 MAE magnitude)
//   horizon     = median sessions_to_outcome across analogues

/// Minimum analogues required for a v1 match at any degradation level.
pub const MIN_V1_SAMPLE: usize = 15;

/// Volatility regime derived from the `volatility` field in REC-001-H records.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VolatilityRegime {
    Present,
    Absent,
}

impl VolatilityRegime {
    pub fn from_str(s: &str) -> Self {
        if s.eq_ignore_ascii_case("present") {
            VolatilityRegime::Present
        } else {
            VolatilityRegime::Absent
        }
    }
}

impl std::fmt::Display for VolatilityRegime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VolatilityRegime::Present => write!(f, "present"),
            VolatilityRegime::Absent => write!(f, "absent"),
        }
    }
}

/// Volume regime derived from `relative_volume_20`.
/// < 0.75 → Low, 0.75–1.25 → Normal, > 1.25 → High
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VolumeRegime {
    Low,
    Normal,
    High,
}

impl VolumeRegime {
    pub fn from_relative_volume(rv: f64) -> Self {
        if rv < 0.75 {
            VolumeRegime::Low
        } else if rv <= 1.25 {
            VolumeRegime::Normal
        } else {
            VolumeRegime::High
        }
    }
}

impl std::fmt::Display for VolumeRegime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VolumeRegime::Low => write!(f, "Low"),
            VolumeRegime::Normal => write!(f, "Normal"),
            VolumeRegime::High => write!(f, "High"),
        }
    }
}

/// Degradation level used to satisfy the minimum sample requirement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DegradationLevel {
    /// Exact match: ticker + direction + trend + momentum + vol_regime + volume_regime
    Exact,
    /// Relaxed volume: ticker + direction + trend + momentum + vol_regime (any volume)
    RelaxVolume,
    /// Relaxed both: ticker + direction + trend + momentum (any vol + volume)
    RelaxBoth,
    /// State only: ticker + direction (any conditions)
    StateOnly,
    /// Insufficient: fewer than MIN_V1_SAMPLE analogues even at StateOnly
    Insufficient,
}

impl std::fmt::Display for DegradationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DegradationLevel::Exact => write!(f, "Exact"),
            DegradationLevel::RelaxVolume => write!(f, "RelaxVolume"),
            DegradationLevel::RelaxBoth => write!(f, "RelaxBoth"),
            DegradationLevel::StateOnly => write!(f, "StateOnly"),
            DegradationLevel::Insufficient => write!(f, "Insufficient"),
        }
    }
}

/// Adaptive geometry and outcome distribution derived from a v1 analogue population.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V1Evidence {
    /// Ticker this evidence was drawn from.
    pub ticker: String,
    /// Direction filter applied ("LONG" or "SHORT").
    pub direction: String,
    /// C3-002 trend label (e.g. "Bullish", "Bearish", "absent").
    pub trend: String,
    /// C3-002 momentum label (e.g. "Positive", "Negative").
    pub momentum: String,
    /// Volatility regime filter applied (or None if relaxed).
    pub vol_regime: Option<VolatilityRegime>,
    /// Volume regime filter applied (or None if relaxed).
    pub volume_regime: Option<VolumeRegime>,
    /// Degradation level used to reach MIN_V1_SAMPLE.
    pub degradation_level: DegradationLevel,
    /// Number of analogues in the matched population.
    pub sample_size: usize,
    /// Fraction of analogues where outcome == "TARGET_BEFORE_RISK".
    pub target_rate: f64,
    /// Adaptive target distance (25th-percentile of mfe_pct[4], as a positive fraction).
    pub adaptive_target_pct: f64,
    /// Adaptive risk distance (median of |mae_pct[4]|, as a positive fraction).
    pub adaptive_risk_pct: f64,
    /// Adaptive horizon (median sessions_to_outcome across all analogues).
    pub adaptive_horizon_sessions: f64,
    /// Evidence classification derived from target_rate + sample_size.
    pub evidence_class: EvidenceClass,
}

/// Raw JSONL record from the REC-001-H evidence base.
#[derive(Debug, Deserialize)]
struct Rec001hRecord {
    ticker: String,
    trend: String,
    momentum: String,
    volatility: String,
    direction: String,
    relative_volume_20: f64,
    /// MFE percentages at each session checkpoint. Null entries mean the trade
    /// was still open at that session (incomplete observation window).
    mfe_pct: Vec<Option<f64>>,
    /// MAE percentages at each session checkpoint. Null entries mean the trade
    /// was still open at that session (incomplete observation window).
    mae_pct: Vec<Option<f64>>,
    outcome: String,
    /// Null when the trade is still open (observation window not yet closed).
    sessions_to_outcome: Option<f64>,
}

/// Compact in-memory representation of a single analogue (post-parse).
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct AnalogueObs {
    vol_regime: VolatilityRegime,
    volume_regime: VolumeRegime,
    mfe5: f64,   // mfe_pct[4] — session-5 MFE
    mae5: f64,   // mae_pct[4] — session-5 MAE (negative)
    outcome: String,
    sessions_to_outcome: f64,
}

/// Full key for exact-match lookup.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct V1Key {
    ticker: String,
    direction: String,
    trend: String,
    momentum: String,
    vol_regime: VolatilityRegime,
    volume_regime: VolumeRegime,
}

/// Partial key for relaxed-volume lookup (vol_regime only, any volume).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct V1KeyRelaxVol {
    ticker: String,
    direction: String,
    trend: String,
    momentum: String,
    vol_regime: VolatilityRegime,
}

/// Partial key for relax-both lookup (any vol + volume).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct V1KeyRelaxBoth {
    ticker: String,
    direction: String,
    trend: String,
    momentum: String,
}

/// Partial key for state-only lookup (ticker + direction, any conditions).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct V1KeyStateOnly {
    ticker: String,
    direction: String,
}

/// The v1 evidence store — loaded once at startup from the REC-001-H JSONL files.
pub struct Rec001hStore {
    exact: HashMap<V1Key, Vec<AnalogueObs>>,
    relax_vol: HashMap<V1KeyRelaxVol, Vec<AnalogueObs>>,
    relax_both: HashMap<V1KeyRelaxBoth, Vec<AnalogueObs>>,
    state_only: HashMap<V1KeyStateOnly, Vec<AnalogueObs>>,
}

impl Rec001hStore {
    /// Load all JSONL files from `dir` (one file per ticker, named `TICKER_NS.jsonl`).
    ///
    /// Files that cannot be parsed are skipped with a warning; the store is
    /// still usable for all tickers that loaded successfully.
    pub fn load_from_dir(dir: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut exact: HashMap<V1Key, Vec<AnalogueObs>> = HashMap::new();
        let mut relax_vol: HashMap<V1KeyRelaxVol, Vec<AnalogueObs>> = HashMap::new();
        let mut relax_both: HashMap<V1KeyRelaxBoth, Vec<AnalogueObs>> = HashMap::new();
        let mut state_only: HashMap<V1KeyStateOnly, Vec<AnalogueObs>> = HashMap::new();

        let entries = std::fs::read_dir(dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }
            let contents = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[Rec001hStore] skipping {:?}: {}", path, e);
                    continue;
                }
            };
            for line in contents.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let rec: Rec001hRecord = match serde_json::from_str(line) {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("[Rec001hStore] parse error in {:?}: {}", path, e);
                        continue;
                    }
                };
                // Require at least 5 MFE/MAE entries (session 5 index = 4)
                if rec.mfe_pct.len() < 5 || rec.mae_pct.len() < 5 {
                    continue;
                }
                // Skip records where session-5 MFE/MAE or sessions_to_outcome is null
                // (open/incomplete trades — observation window not yet closed).
                let mfe5 = match rec.mfe_pct[4] {
                    Some(v) => v,
                    None => continue,
                };
                let mae5 = match rec.mae_pct[4] {
                    Some(v) => v,
                    None => continue,
                };
                let sessions_to_outcome = match rec.sessions_to_outcome {
                    Some(v) => v,
                    None => continue,
                };
                let vol = VolatilityRegime::from_str(&rec.volatility);
                let vol_r = VolumeRegime::from_relative_volume(rec.relative_volume_20);
                let obs = AnalogueObs {
                    vol_regime: vol.clone(),
                    volume_regime: vol_r.clone(),
                    mfe5,
                    mae5,
                    outcome: rec.outcome.clone(),
                    sessions_to_outcome,
                };
                let ticker = rec.ticker.replace(".NS", "_NS");
                let dir_upper = rec.direction.to_uppercase();

                exact
                    .entry(V1Key {
                        ticker: ticker.clone(),
                        direction: dir_upper.clone(),
                        trend: rec.trend.clone(),
                        momentum: rec.momentum.clone(),
                        vol_regime: vol.clone(),
                        volume_regime: vol_r.clone(),
                    })
                    .or_default()
                    .push(obs.clone());

                relax_vol
                    .entry(V1KeyRelaxVol {
                        ticker: ticker.clone(),
                        direction: dir_upper.clone(),
                        trend: rec.trend.clone(),
                        momentum: rec.momentum.clone(),
                        vol_regime: vol.clone(),
                    })
                    .or_default()
                    .push(obs.clone());

                relax_both
                    .entry(V1KeyRelaxBoth {
                        ticker: ticker.clone(),
                        direction: dir_upper.clone(),
                        trend: rec.trend.clone(),
                        momentum: rec.momentum.clone(),
                    })
                    .or_default()
                    .push(obs.clone());

                state_only
                    .entry(V1KeyStateOnly {
                        ticker: ticker.clone(),
                        direction: dir_upper.clone(),
                    })
                    .or_default()
                    .push(obs);
            }
        }

        Ok(Rec001hStore { exact, relax_vol, relax_both, state_only })
    }

    /// Look up v1 evidence for a ticker in a given C3-002 state.
    ///
    /// Implements the ARCH-006 §6.2 graceful degradation hierarchy.
    /// Returns `None` only when fewer than `MIN_V1_SAMPLE` analogues exist
    /// even at the StateOnly level — the engine maps this to NO_TRADE.
    pub fn for_decision(
        &self,
        ticker: &str,
        direction: &str,
        trend: &str,
        momentum: &str,
        vol_regime: &VolatilityRegime,
        volume_regime: &VolumeRegime,
    ) -> Option<V1Evidence> {
        let dir = direction.to_uppercase();

        // Level 1 — exact match
        let exact_key = V1Key {
            ticker: ticker.to_string(),
            direction: dir.clone(),
            trend: trend.to_string(),
            momentum: momentum.to_string(),
            vol_regime: vol_regime.clone(),
            volume_regime: volume_regime.clone(),
        };
        if let Some(pool) = self.exact.get(&exact_key) {
            if pool.len() >= MIN_V1_SAMPLE {
                return Some(aggregate_v1(
                    ticker, &dir, trend, momentum,
                    Some(vol_regime.clone()), Some(volume_regime.clone()),
                    DegradationLevel::Exact, pool,
                ));
            }
        }

        // Level 2 — relax volume regime
        let rv_key = V1KeyRelaxVol {
            ticker: ticker.to_string(),
            direction: dir.clone(),
            trend: trend.to_string(),
            momentum: momentum.to_string(),
            vol_regime: vol_regime.clone(),
        };
        if let Some(pool) = self.relax_vol.get(&rv_key) {
            if pool.len() >= MIN_V1_SAMPLE {
                return Some(aggregate_v1(
                    ticker, &dir, trend, momentum,
                    Some(vol_regime.clone()), None,
                    DegradationLevel::RelaxVolume, pool,
                ));
            }
        }

        // Level 3 — relax both vol + volume
        let rb_key = V1KeyRelaxBoth {
            ticker: ticker.to_string(),
            direction: dir.clone(),
            trend: trend.to_string(),
            momentum: momentum.to_string(),
        };
        if let Some(pool) = self.relax_both.get(&rb_key) {
            if pool.len() >= MIN_V1_SAMPLE {
                return Some(aggregate_v1(
                    ticker, &dir, trend, momentum,
                    None, None,
                    DegradationLevel::RelaxBoth, pool,
                ));
            }
        }

        // Level 4 — state only (ticker + direction, any conditions)
        let so_key = V1KeyStateOnly {
            ticker: ticker.to_string(),
            direction: dir.clone(),
        };
        if let Some(pool) = self.state_only.get(&so_key) {
            if pool.len() >= MIN_V1_SAMPLE {
                return Some(aggregate_v1(
                    ticker, &dir, trend, momentum,
                    None, None,
                    DegradationLevel::StateOnly, pool,
                ));
            }
        }

        // Insufficient
        None
    }
}

/// Aggregate a pool of analogues into a [`V1Evidence`] summary.
fn aggregate_v1(
    ticker: &str,
    direction: &str,
    trend: &str,
    momentum: &str,
    vol_regime: Option<VolatilityRegime>,
    volume_regime: Option<VolumeRegime>,
    degradation_level: DegradationLevel,
    pool: &[AnalogueObs],
) -> V1Evidence {
    let sample_size = pool.len();

    // Target rate — first-exit semantics: only TARGET_BEFORE_RISK counts as a win.
    // An observation with MFE10=9.8% but outcome=RISK_BEFORE_TARGET is a loss.
    let winners: Vec<&AnalogueObs> = pool.iter().filter(|o| o.outcome == "TARGET_BEFORE_RISK").collect();
    let losers: Vec<&AnalogueObs> = pool.iter().filter(|o| o.outcome == "RISK_BEFORE_TARGET").collect();
    let target_count = winners.len();
    let target_rate = target_count as f64 / sample_size as f64;

    // Adaptive target: 25th-percentile of MFE[4] from WINNING analogues only.
    //
    // Rationale: we want to know "what did the stock achieve in cases where it
    // actually reached the target before the risk boundary?" Using all analogues
    // would include MFE from losing trades, which inflates the apparent upside.
    // The 25th-percentile is conservative — it sets a target that 75% of winning
    // analogues exceeded, giving a realistic achievable level.
    let adaptive_target_pct = if !winners.is_empty() {
        let mut mfe_vals: Vec<f64> = winners.iter().map(|o| o.mfe5).collect();
        mfe_vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let p25_idx = ((mfe_vals.len() as f64 * 0.25).floor() as usize).min(mfe_vals.len() - 1);
        mfe_vals[p25_idx].max(0.0)
    } else {
        // No winners in the analogue pool — fall back to 25th-percentile of all MFE
        // so the engine can still produce a geometry (action will be NO_TRADE anyway
        // due to Unfavourable/Insufficient evidence class).
        let mut mfe_vals: Vec<f64> = pool.iter().map(|o| o.mfe5).collect();
        mfe_vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let p25_idx = ((mfe_vals.len() as f64 * 0.25).floor() as usize).min(mfe_vals.len() - 1);
        mfe_vals[p25_idx].max(0.0)
    };

    // Adaptive risk: median of |MAE[4]| from LOSING analogues only.
    //
    // Rationale: we want to know "how far did the stock go against us before
    // stopping out?" Using winning analogues would understate the adverse excursion
    // because winners often have small MAE. The median of losers' MAE gives a
    // realistic risk boundary that reflects actual stop-out behaviour.
    let adaptive_risk_pct = if !losers.is_empty() {
        let mut mae_abs: Vec<f64> = losers.iter().map(|o| o.mae5.abs()).collect();
        median(&mut mae_abs)
    } else {
        // No losers — fall back to median of all MAE (conservative)
        let mut mae_abs: Vec<f64> = pool.iter().map(|o| o.mae5.abs()).collect();
        median(&mut mae_abs)
    };

    // Adaptive horizon: median sessions_to_outcome across all closed analogues.
    // This answers "how long does it typically take to resolve?" regardless of direction.
    let mut sessions: Vec<f64> = pool.iter().map(|o| o.sessions_to_outcome).collect();
    let adaptive_horizon_sessions = median(&mut sessions);

    let evidence_class = EvidenceClass::classify(target_rate, sample_size);

    V1Evidence {
        ticker: ticker.to_string(),
        direction: direction.to_string(),
        trend: trend.to_string(),
        momentum: momentum.to_string(),
        vol_regime,
        volume_regime,
        degradation_level,
        sample_size,
        target_rate,
        adaptive_target_pct,
        adaptive_risk_pct,
        adaptive_horizon_sessions,
        evidence_class,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_store_with_outcomes(records: Vec<(&str, &str, &str, &str)>) -> EvidenceStore {
        // records: (direction, trend, momentum, outcome)
        let outcomes: Vec<serde_json::Value> = records
            .iter()
            .enumerate()
            .map(|(i, (dir, trend, mom, out))| {
                serde_json::json!({
                    "decision_id": format!("t-{:03}", i),
                    "direction": dir,
                    "coralys_trend": trend,
                    "coralys_momentum": mom,
                    "observation_status": "COMPLETE",
                    "outcome": out,
                    "time_to_target": if *out == "TARGET_BEFORE_RISK" { serde_json::json!(3.0) } else { serde_json::json!(null) },
                    "mfe_10": 0.03,
                    "mae_10": -0.015
                })
            })
            .collect();
        let json = serde_json::json!({ "outcomes": outcomes }).to_string();
        EvidenceStore::from_json(&json).unwrap()
    }

    #[test]
    fn favourable_when_high_target_rate_and_large_sample() {
        // 20 TARGET_BEFORE_RISK, 10 RISK_BEFORE_TARGET → rate = 0.667 ≥ 0.40, n=30 ≥ 30
        let mut records = vec![];
        for _ in 0..20 {
            records.push(("LONG", "Bullish", "Positive", "TARGET_BEFORE_RISK"));
        }
        for _ in 0..10 {
            records.push(("LONG", "Bullish", "Positive", "RISK_BEFORE_TARGET"));
        }
        let store = make_store_with_outcomes(records);
        let ev = store.for_decision("LONG", "Bullish", "Positive");
        assert_eq!(ev.evidence_class, EvidenceClass::Favourable);
        assert_eq!(ev.sample_size, 30);
        assert!(ev.analogue_key.narrow_match);
        assert!((ev.target_before_risk_rate - 0.667).abs() < 0.01);
    }

    #[test]
    fn mixed_when_moderate_target_rate() {
        // 5 TARGET, 10 RISK → rate = 0.333 ≥ 0.30, n=15 ≥ 15 but < 30 → Mixed
        let mut records = vec![];
        for _ in 0..5 {
            records.push(("LONG", "Bearish", "Positive", "TARGET_BEFORE_RISK"));
        }
        for _ in 0..10 {
            records.push(("LONG", "Bearish", "Positive", "RISK_BEFORE_TARGET"));
        }
        let store = make_store_with_outcomes(records);
        let ev = store.for_decision("LONG", "Bearish", "Positive");
        assert_eq!(ev.evidence_class, EvidenceClass::Mixed);
    }

    #[test]
    fn unfavourable_when_low_target_rate() {
        // 3 TARGET, 12 RISK → rate = 0.20 < 0.30, n=15 ≥ 15 → Unfavourable
        let mut records = vec![];
        for _ in 0..3 {
            records.push(("LONG", "Bearish", "Negative", "TARGET_BEFORE_RISK"));
        }
        for _ in 0..12 {
            records.push(("LONG", "Bearish", "Negative", "RISK_BEFORE_TARGET"));
        }
        let store = make_store_with_outcomes(records);
        let ev = store.for_decision("LONG", "Bearish", "Negative");
        assert_eq!(ev.evidence_class, EvidenceClass::Unfavourable);
    }

    #[test]
    fn insufficient_when_small_sample() {
        // Only 5 records → Insufficient
        let records = vec![
            ("LONG", "Bullish", "Negative", "TARGET_BEFORE_RISK"),
            ("LONG", "Bullish", "Negative", "RISK_BEFORE_TARGET"),
            ("LONG", "Bullish", "Negative", "RISK_BEFORE_TARGET"),
            ("LONG", "Bullish", "Negative", "HORIZON"),
            ("LONG", "Bullish", "Negative", "RISK_BEFORE_TARGET"),
        ];
        let store = make_store_with_outcomes(records);
        let ev = store.for_decision("LONG", "Bullish", "Negative");
        assert_eq!(ev.evidence_class, EvidenceClass::Insufficient);
        assert_eq!(ev.sample_size, 5);
    }

    #[test]
    fn broad_fallback_when_narrow_insufficient() {
        // 5 LONG + 15 SHORT for Bullish_Positive → narrow(LONG)=5 < 15, broad=20 ≥ 15
        let mut records = vec![];
        for _ in 0..5 {
            records.push(("LONG", "Bullish", "Positive", "TARGET_BEFORE_RISK"));
        }
        for _ in 0..15 {
            records.push(("SHORT", "Bullish", "Positive", "RISK_BEFORE_TARGET"));
        }
        let store = make_store_with_outcomes(records);
        let ev = store.for_decision("LONG", "Bullish", "Positive");
        // Should fall back to broad (state-only) pool of 20
        assert!(!ev.analogue_key.narrow_match);
        assert_eq!(ev.sample_size, 20);
        assert_eq!(ev.analogue_key.direction, "ANY");
    }

    #[test]
    fn incomplete_decisions_excluded() {
        // Mix of COMPLETE and INCOMPLETE — only COMPLETE should be counted
        let outcomes = serde_json::json!({
            "outcomes": [
                {
                    "decision_id": "a",
                    "direction": "LONG",
                    "coralys_trend": "Bullish",
                    "coralys_momentum": "Positive",
                    "observation_status": "COMPLETE",
                    "outcome": "TARGET_BEFORE_RISK",
                    "time_to_target": 3.0,
                    "mfe_10": 0.05,
                    "mae_10": -0.01
                },
                {
                    "decision_id": "b",
                    "direction": "LONG",
                    "coralys_trend": "Bullish",
                    "coralys_momentum": "Positive",
                    "observation_status": "INCOMPLETE",
                    "outcome": null,
                    "time_to_target": null,
                    "mfe_10": null,
                    "mae_10": null
                }
            ]
        });
        let store = EvidenceStore::from_json(&outcomes.to_string()).unwrap();
        let ev = store.for_decision("LONG", "Bullish", "Positive");
        // Only 1 COMPLETE record → Insufficient
        assert_eq!(ev.evidence_class, EvidenceClass::Insufficient);
        assert_eq!(ev.sample_size, 1);
    }
}