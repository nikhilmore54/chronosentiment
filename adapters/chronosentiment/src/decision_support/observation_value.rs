//! Observation-path decision value for discovery.
//!
//! Uses one certified horizon (20D). Does not use the 60D lake SHORT series.
//! NO_TRADE is standing aside and is not scored as a zero return.

use chrono::{DateTime, Duration, TimeZone, Utc};
use coralys_moga::runtime::optimization::metric::MetricReport;
use coralys_moga::traits::{Evaluated, FitnessEvaluator};
use serde::Serialize;

use crate::ingestion::yahoo::YahooHistoricalBar;
use crate::reasoning::assessment::AssessmentProfile;

use super::csp006_protocol::RESEARCH_UNIVERSE;
use super::dataset_partition::PartitionKind;
use super::enrichment_certify::assess_from_bars_at_t;
use super::forward_tick::instrument_id_for;
use super::policy_artifact::first_match_action;
use super::policy_genome::RuleListGenome;
use super::DecisionAction;

pub const DISCOVERY_HORIZON_DAYS: u32 = 20;

#[derive(Debug, Clone)]
pub struct ObservationRow {
    pub instrument: String,
    pub as_of: DateTime<Utc>,
    pub profile: AssessmentProfile,
    pub instrument_return: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct ObservationSlice {
    pub kind: PartitionKind,
    pub rows: Vec<ObservationRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SliceScore {
    pub fitness: f64,
    pub n_rows: usize,
    pub n_traded: u32,
    pub n_stood_aside: u32,
    pub n_unavailable: u32,
}

#[derive(Debug, Clone)]
pub struct GenomeEvaluation {
    pub genome: RuleListGenome,
    pub fitness: f64,
    pub valid: bool,
}

impl Evaluated for GenomeEvaluation {
    type Genome = RuleListGenome;
    fn fitness(&self) -> f64 {
        self.fitness
    }
    fn is_valid(&self) -> bool {
        self.valid
    }
    fn genome(&self) -> &Self::Genome {
        &self.genome
    }
}

pub struct DevelopmentFitness {
    slice: ObservationSlice,
}

impl DevelopmentFitness {
    pub fn new(slice: ObservationSlice) -> Result<Self, String> {
        if slice.kind != PartitionKind::Development {
            return Err("evolution fitness may only use the development slice".into());
        }
        Ok(Self { slice })
    }
}

impl FitnessEvaluator<RuleListGenome> for DevelopmentFitness {
    type Evaluation = GenomeEvaluation;

    fn evaluate(&self, genome: &RuleListGenome, _metrics: &MetricReport) -> Self::Evaluation {
        let score = score_genome(genome, &self.slice).expect("development slice is permitted");
        GenomeEvaluation {
            genome: genome.clone(),
            fitness: score.fitness,
            valid: true,
        }
    }
}

pub fn score_genome(
    genome: &RuleListGenome,
    slice: &ObservationSlice,
) -> Result<SliceScore, String> {
    match slice.kind {
        PartitionKind::Evaluation => {
            return Err("search must not score the evaluation slice".into());
        }
        PartitionKind::Development | PartitionKind::Selection => {}
    }
    let mut n_traded = 0u32;
    let mut n_stood_aside = 0u32;
    let mut n_unavailable = 0u32;
    let mut per_instrument: Vec<f64> = Vec::new();
    for ticker in RESEARCH_UNIVERSE {
        let mut traded = Vec::new();
        for row in slice.rows.iter().filter(|r| r.instrument == *ticker) {
            let action =
                first_match_action(&genome.rules, genome.unmatched_action, &row.profile);
            match action {
                DecisionAction::NoTrade => n_stood_aside += 1,
                DecisionAction::Long | DecisionAction::Short => match row.instrument_return {
                    Some(raw) => {
                        n_traded += 1;
                        let signed = if action == DecisionAction::Long {
                            raw
                        } else {
                            -raw
                        };
                        traded.push(signed);
                    }
                    None => n_unavailable += 1,
                },
            }
        }
        let mean = if traded.is_empty() {
            0.0
        } else {
            traded.iter().sum::<f64>() / traded.len() as f64
        };
        per_instrument.push(mean);
    }
    let fitness = if per_instrument.is_empty() {
        0.0
    } else {
        per_instrument.iter().sum::<f64>() / per_instrument.len() as f64
    };
    Ok(SliceScore {
        fitness,
        n_rows: slice.rows.len(),
        n_traded,
        n_stood_aside,
        n_unavailable,
    })
}

pub fn build_observation_slice(
    cache: &std::collections::BTreeMap<String, Vec<YahooHistoricalBar>>,
    timestamps: &[DateTime<Utc>],
    kind: PartitionKind,
) -> Result<ObservationSlice, String> {
    let horizon = Duration::days(DISCOVERY_HORIZON_DAYS as i64);
    let mut rows = Vec::new();
    for ticker in RESEARCH_UNIVERSE {
        let bars = cache
            .get(ticker)
            .ok_or_else(|| format!("yahoo cache missing {ticker}"))?;
        let instrument_id = instrument_id_for(ticker);
        for &t in timestamps {
            let (mut profile, _, _) = assess_from_bars_at_t(bars, t, instrument_id);
            profile.metadata.evaluation_timestamp = t;
            profile.instrument_id = Some(instrument_id);
            let entry = close_at_or_before(bars, t);
            let exit = close_at_or_after(bars, t + horizon);
            let instrument_return = match (entry, exit) {
                (Some(p0), Some(ph)) if p0 > 0.0 && ph.is_finite() => Some((ph - p0) / p0),
                _ => None,
            };
            rows.push(ObservationRow {
                instrument: (*ticker).to_string(),
                as_of: t,
                profile,
                instrument_return,
            });
        }
    }
    Ok(ObservationSlice { kind, rows })
}

fn close_at_or_before(bars: &[YahooHistoricalBar], t: DateTime<Utc>) -> Option<f64> {
    bars.iter()
        .filter_map(|b| {
            let ts = Utc.timestamp_opt(b.timestamp, 0).single()?;
            if ts <= t && b.adj_close.is_finite() && b.adj_close > 0.0 {
                Some((ts, b.adj_close))
            } else {
                None
            }
        })
        .max_by_key(|(ts, _)| *ts)
        .map(|(_, c)| c)
}

fn close_at_or_after(bars: &[YahooHistoricalBar], t: DateTime<Utc>) -> Option<f64> {
    bars.iter()
        .filter_map(|b| {
            let ts = Utc.timestamp_opt(b.timestamp, 0).single()?;
            if ts >= t && b.adj_close.is_finite() && b.adj_close > 0.0 {
                Some((ts, b.adj_close))
            } else {
                None
            }
        })
        .min_by_key(|(ts, _)| *ts)
        .map(|(_, c)| c)
}
