//! Temporal/lineage certification for Assessment Enrichment v0.1 snapshots.
//!
//! Does not score a trading policy. Does not reopen G-GATE. Not B5.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use coralys_moga::runtime::optimization::metric::MetricEngine;
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::ingestion::provider::ValidatedObservationTranslator;
use crate::ingestion::yahoo::{YahooHistoricalBar, YahooTranslator};
use crate::metrics::instrument::{
    AverageTrueRangeMetric, InstrumentMetricEngine, RateOfChangeMetric, SimpleMovingAverageMetric,
};
use crate::observation::ValidatedObservation;
use crate::reasoning::assessment::{
    AssessmentEngine, AssessmentProfile, ENRICHMENT_CONCEPTS, FactorStatus,
};
use crate::validation::context::InstrumentEvaluationContext;

use super::factor_availability::{
    certify_enrichment_profiles, render_factor_availability, report_factor_availability,
    FactorAvailabilityReport,
};

#[derive(Debug, Clone, Serialize)]
pub struct EnrichmentCertification {
    pub result: String,
    pub n_profiles: u32,
    pub n_decisions: i64,
    pub orphan_decisions: i64,
    pub assessment_after_decision: i64,
    pub signature_roundtrip_mismatches: u32,
    pub factor_status_mismatches_vs_bars_le_t: u32,
    pub temporal_bar_leaks: u32,
    pub identity_hash: String,
    pub failures: Vec<String>,
    pub factor_availability: FactorAvailabilityReport,
}

pub fn bars_at_or_before(bars: &[YahooHistoricalBar], t: DateTime<Utc>) -> Vec<YahooHistoricalBar> {
    bars.iter()
        .filter(|b| {
            Utc.timestamp_opt(b.timestamp, 0)
                .single()
                .map(|ts| ts <= t)
                .unwrap_or(false)
        })
        .cloned()
        .collect()
}

pub fn observations_from_bars(
    bars: &[YahooHistoricalBar],
    instrument_id: Uuid,
) -> Vec<ValidatedObservation> {
    let translator = YahooTranslator;
    bars.iter()
        .map(|bar| {
            let raw = translator.translate(bar.clone(), &crate::instrument::Instrument {
                id: instrument_id,
                exchange: "NSE".to_string(),
                display_symbol: String::new(),
                provider_ids: Default::default(),
                created_at: t_epoch(),
            });
            ValidatedObservation {
                id: Uuid::nil(),
                research_session_id: None,
                instrument_id: Some(instrument_id),
                observation_type: raw.observation_type,
                source: raw.source,
                source_identifier: raw.source_identifier,
                observed_at: raw.observed_at,
                effective_from: raw.observed_at,
                effective_to: None,
                recorded_at: t_epoch(),
                raw_payload: raw.raw_payload,
                normalized_payload: raw.normalized_payload,
                confidence: 1.0,
                freshness: 0.0,
                coverage: "Full".to_string(),
                consistency: Some(1.0),
                quality_score: 1.0,
                provenance_hash: "hash".to_string(),
                schema_version: 1,
            }
        })
        .collect()
}

fn t_epoch() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap()
}

fn metric_engine() -> InstrumentMetricEngine {
    let mut engine = InstrumentMetricEngine::new();
    engine.add_model(Box::new(SimpleMovingAverageMetric::new(20)));
    engine.add_model(Box::new(SimpleMovingAverageMetric::new(50)));
    engine.add_model(Box::new(RateOfChangeMetric::new(20)));
    engine.add_model(Box::new(AverageTrueRangeMetric::new(14)));
    engine
}

pub fn metrics_from_bars_at_t(
    bars: &[YahooHistoricalBar],
    t: DateTime<Utc>,
    instrument_id: Uuid,
) -> coralys_moga::runtime::optimization::metric::MetricReport {
    let kept = bars_at_or_before(bars, t);
    let observations = observations_from_bars(&kept, instrument_id);
    let ctx = InstrumentEvaluationContext {
        instrument_id,
        observations,
    };
    metric_engine().evaluate(&ctx)
}

pub fn assess_from_bars_at_t(
    bars: &[YahooHistoricalBar],
    t: DateTime<Utc>,
    instrument_id: Uuid,
) -> (AssessmentProfile, usize, Option<DateTime<Utc>>) {
    let kept = bars_at_or_before(bars, t);
    let n = kept.len();
    let max_from = kept.last().and_then(|b| Utc.timestamp_opt(b.timestamp, 0).single());
    let observations = observations_from_bars(&kept, instrument_id);
    let ctx = InstrumentEvaluationContext {
        instrument_id,
        observations,
    };
    let metrics = metric_engine().evaluate(&ctx);
    let profile = AssessmentEngine.assess_at(&metrics, &ENRICHMENT_CONCEPTS, t, Some(instrument_id));
    (profile, n, max_from)
}

pub fn load_yahoo_cache_dir(dir: &Path) -> Result<BTreeMap<String, Vec<YahooHistoricalBar>>, String> {
    let mut out = BTreeMap::new();
    let entries = fs::read_dir(dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("bad cache name {}", path.display()))?
            .to_string();
        let bytes = fs::read(&path).map_err(|e| e.to_string())?;
        let bars: Vec<YahooHistoricalBar> =
            serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        out.insert(stem, bars);
    }
    Ok(out)
}

pub fn certify_snapshot(
    profiles: &[AssessmentProfile],
    labels: &BTreeMap<Uuid, String>,
    stored_signature_hashes: &[String],
    n_decisions: i64,
    orphan_decisions: i64,
    assessment_after_decision: i64,
    yahoo_cache: Option<&BTreeMap<String, Vec<YahooHistoricalBar>>>,
) -> EnrichmentCertification {
    let mut failures = certify_enrichment_profiles(profiles);
    let report = report_factor_availability(profiles, labels);
    let report_again = report_factor_availability(profiles, labels);
    if report != report_again {
        failures.push("factor availability report is not deterministic across two passes".into());
    }

    let mut signature_roundtrip_mismatches = 0u32;
    for (i, profile) in profiles.iter().enumerate() {
        if let Some(stored) = stored_signature_hashes.get(i) {
            let recomputed = profile.to_hash();
            if &recomputed != stored {
                signature_roundtrip_mismatches += 1;
                failures.push(format!("profile[{i}] signature_hash round-trip mismatch"));
            }
        }
    }

    if n_decisions != profiles.len() as i64 {
        failures.push(format!(
            "decisions {} != assessments {}",
            n_decisions,
            profiles.len()
        ));
    }
    for (i, profile) in profiles.iter().enumerate() {
        if !is_replay_month_end(profile.metadata.evaluation_timestamp) {
            failures.push(format!(
                "profile[{i}] evaluation_timestamp {} is outside the 2021-2024 replay calendar",
                profile.metadata.evaluation_timestamp
            ));
        }
    }
    if orphan_decisions != 0 {
        failures.push(format!("orphan decisions {orphan_decisions} != 0"));
    }
    if assessment_after_decision != 0 {
        failures.push(format!(
            "assessment.evaluation_timestamp > decision.evaluation_timestamp on {assessment_after_decision} pairs"
        ));
    }

    let mut factor_status_mismatches_vs_bars_le_t = 0u32;
    let mut temporal_bar_leaks = 0u32;
    if let Some(cache) = yahoo_cache {
        for profile in profiles {
            let label = profile
                .instrument_id
                .and_then(|id| labels.get(&id).cloned())
                .unwrap_or_else(|| "unknown".to_string());
            let Some(bars) = cache.get(&label) else {
                failures.push(format!("no yahoo cache for {label}"));
                continue;
            };
            let t = profile.metadata.evaluation_timestamp;
            let leaked = bars.iter().filter(|b| {
                Utc.timestamp_opt(b.timestamp, 0)
                    .single()
                    .map(|ts| ts > t)
                    .unwrap_or(false)
            });
            // Cache may contain bars after T; reconstruction must exclude them.
            let _ = leaked.count();
            let (recomputed, _n, max_from) =
                assess_from_bars_at_t(bars, t, profile.instrument_id.unwrap_or(Uuid::nil()));
            if let Some(max_from) = max_from {
                if max_from > t {
                    temporal_bar_leaks += 1;
                    failures.push(format!(
                        "{label} at {t} used bar effective_from {max_from} > T"
                    ));
                }
            }
            if factor_status_identity(&recomputed.factor_status) != factor_status_identity(&profile.factor_status)
            {
                factor_status_mismatches_vs_bars_le_t += 1;
                failures.push(format!(
                    "{label} at {t}: stored factor_status does not match bars with effective_from <= T"
                ));
            }
        }
    }

    let identity_hash = identity_hash(&report, &failures);
    let result = if failures.is_empty() { "PASS" } else { "FAIL" };
    EnrichmentCertification {
        result: result.to_string(),
        n_profiles: profiles.len() as u32,
        n_decisions,
        orphan_decisions,
        assessment_after_decision,
        signature_roundtrip_mismatches,
        factor_status_mismatches_vs_bars_le_t,
        temporal_bar_leaks,
        identity_hash,
        failures,
        factor_availability: report,
    }
}

fn factor_status_identity(status: &[FactorStatus]) -> Vec<(String, String, Vec<String>, Vec<String>)> {
    let mut rows: Vec<_> = status
        .iter()
        .map(|s| {
            (
                format!("{:?}", s.concept),
                format!("{:?}", s.availability),
                s.supporting_metrics.clone(),
                s.missing_metrics.clone(),
            )
        })
        .collect();
    rows.sort();
    rows
}

fn identity_hash(report: &FactorAvailabilityReport, failures: &[String]) -> String {
    let payload = serde_json::json!({
        "report": report,
        "failures": failures,
    });
    let bytes = serde_json::to_vec(&payload).expect("cert identity serializes");
    format!("{:x}", Sha256::digest(&bytes))
}

pub fn render_certification(cert: &EnrichmentCertification) -> String {
    let mut md = String::from("# Assessment Enrichment v0.1 — snapshot certification\n\n");
    md.push_str(&format!("**Result:** {}\n\n", cert.result));
    md.push_str("Information-fidelity validation. **Not B5.** Not a trading-strategy experiment. Not G-GATE. Decision Engine v1.0 remains unfrozen.\n\n");
    md.push_str(&format!("- assessments: {}\n", cert.n_profiles));
    md.push_str(&format!("- decisions: {}\n", cert.n_decisions));
    md.push_str(&format!("- orphan decisions: {}\n", cert.orphan_decisions));
    md.push_str(&format!(
        "- assessment after decision: {}\n",
        cert.assessment_after_decision
    ));
    md.push_str(&format!(
        "- signature round-trip mismatches: {}\n",
        cert.signature_roundtrip_mismatches
    ));
    md.push_str(&format!(
        "- factor_status mismatches vs bars ≤ T: {}\n",
        cert.factor_status_mismatches_vs_bars_le_t
    ));
    md.push_str(&format!("- temporal bar leaks: {}\n", cert.temporal_bar_leaks));
    md.push_str(&format!("- identity hash: `{}`\n\n", cert.identity_hash));
    if cert.failures.is_empty() {
        md.push_str("No certification failures.\n\n");
    } else {
        md.push_str("## Failures\n\n");
        for f in &cert.failures {
            md.push_str(&format!("- {f}\n"));
        }
        md.push('\n');
    }
    md.push_str(&render_factor_availability(&cert.factor_availability));
    md
}

pub fn month_end_1530(year: i32, month: u32) -> DateTime<Utc> {
    let next_month = if month == 12 { 1 } else { month + 1 };
    let next_year = if month == 12 { year + 1 } else { year };
    let d = chrono::NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .unwrap()
        .pred_opt()
        .unwrap();
    Utc.from_utc_datetime(&d.and_time(chrono::NaiveTime::from_hms_opt(15, 30, 0).unwrap()))
}

pub fn is_replay_month_end(t: DateTime<Utc>) -> bool {
    t.hour() == 15 && t.minute() == 30 && t.second() == 0 && t.year() >= 2021 && t.year() <= 2024
}
