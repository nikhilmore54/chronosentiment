//! Factor-availability diagnostics for Assessment Enrichment v0.1.
//!
//! Counts AVAILABLE vs UNAVAILABLE from `factor_status`. Does not score policies.

use std::collections::BTreeMap;

use chrono::{Datelike, Timelike};
use serde::Serialize;
use uuid::Uuid;

use crate::reasoning::assessment::{AssessmentProfile, FactorAvailability};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct FactorAvailabilityCounts {
    pub available: u32,
    pub unavailable: u32,
}

impl FactorAvailabilityCounts {
    pub fn total(&self) -> u32 {
        self.available + self.unavailable
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FactorAvailabilityReport {
    pub n_profiles: u32,
    pub by_concept: BTreeMap<String, FactorAvailabilityCounts>,
    pub by_instrument: BTreeMap<String, BTreeMap<String, FactorAvailabilityCounts>>,
    pub by_year: BTreeMap<i32, BTreeMap<String, FactorAvailabilityCounts>>,
}

pub fn report_factor_availability(
    profiles: &[AssessmentProfile],
    instrument_labels: &BTreeMap<Uuid, String>,
) -> FactorAvailabilityReport {
    let mut by_concept: BTreeMap<String, FactorAvailabilityCounts> = BTreeMap::new();
    let mut by_instrument: BTreeMap<String, BTreeMap<String, FactorAvailabilityCounts>> =
        BTreeMap::new();
    let mut by_year: BTreeMap<i32, BTreeMap<String, FactorAvailabilityCounts>> = BTreeMap::new();

    for profile in profiles {
        let label = profile
            .instrument_id
            .and_then(|id| instrument_labels.get(&id).cloned())
            .or_else(|| profile.instrument_id.map(|id| id.to_string()))
            .unwrap_or_else(|| "unknown".to_string());
        let year = profile.metadata.evaluation_timestamp.year();
        for status in &profile.factor_status {
            let concept = format!("{:?}", status.concept);
            bump(
                by_concept.entry(concept.clone()).or_default(),
                status.availability,
            );
            bump(
                by_instrument
                    .entry(label.clone())
                    .or_default()
                    .entry(concept.clone())
                    .or_default(),
                status.availability,
            );
            bump(
                by_year
                    .entry(year)
                    .or_default()
                    .entry(concept)
                    .or_default(),
                status.availability,
            );
        }
    }

    FactorAvailabilityReport {
        n_profiles: profiles.len() as u32,
        by_concept,
        by_instrument,
        by_year,
    }
}

fn bump(counts: &mut FactorAvailabilityCounts, availability: FactorAvailability) {
    match availability {
        FactorAvailability::Available => counts.available += 1,
        FactorAvailability::Unavailable => counts.unavailable += 1,
    }
}

pub fn render_factor_availability(report: &FactorAvailabilityReport) -> String {
    let mut md = String::from("# Factor availability\n\n");
    md.push_str("Information-fidelity report. Not a trading-performance result.\n\n");
    md.push_str(&format!("Profiles: {}\n\n", report.n_profiles));
    md.push_str("## By concept\n\n");
    md.push_str("| Concept | Available | Unavailable | n |\n|---|---:|---:|---:|\n");
    for (concept, c) in &report.by_concept {
        md.push_str(&format!(
            "| {concept} | {} | {} | {} |\n",
            c.available,
            c.unavailable,
            c.total()
        ));
    }
    md.push_str("\n## By instrument\n\n");
    for (instrument, concepts) in &report.by_instrument {
        md.push_str(&format!("### {instrument}\n\n"));
        md.push_str("| Concept | Available | Unavailable | n |\n|---|---:|---:|---:|\n");
        for (concept, c) in concepts {
            md.push_str(&format!(
                "| {concept} | {} | {} | {} |\n",
                c.available,
                c.unavailable,
                c.total()
            ));
        }
        md.push('\n');
    }
    md.push_str("## By year\n\n");
    for (year, concepts) in &report.by_year {
        md.push_str(&format!("### {year}\n\n"));
        md.push_str("| Concept | Available | Unavailable | n |\n|---|---:|---:|---:|\n");
        for (concept, c) in concepts {
            md.push_str(&format!(
                "| {concept} | {} | {} | {} |\n",
                c.available,
                c.unavailable,
                c.total()
            ));
        }
        md.push('\n');
    }
    md
}

/// Structural checks on persisted enrichment profiles. Does not score a policy.
pub fn certify_enrichment_profiles(profiles: &[AssessmentProfile]) -> Vec<String> {
    let mut failures = Vec::new();
    for (i, profile) in profiles.iter().enumerate() {
        if profile.factor_status.len() != 3 {
            failures.push(format!(
                "profile[{i}] factor_status len {} != 3",
                profile.factor_status.len()
            ));
        }
        let t = profile.metadata.evaluation_timestamp;
        if t.hour() != 15 || t.minute() != 30 || t.second() != 0 {
            failures.push(format!(
                "profile[{i}] evaluation_timestamp {} is not replay month-end 15:30:00",
                t
            ));
        }
        if profile.metadata.created_at <= t {
            failures.push(format!(
                "profile[{i}] created_at {} is not after evaluation_timestamp {t}",
                profile.metadata.created_at
            ));
        }
        for status in &profile.factor_status {
            match status.availability {
                FactorAvailability::Available => {
                    if !status.missing_metrics.is_empty() {
                        failures.push(format!(
                            "profile[{i}] {:?} AVAILABLE but missing_metrics={:?}",
                            status.concept, status.missing_metrics
                        ));
                    }
                    if status.supporting_metrics.is_empty() {
                        failures.push(format!(
                            "profile[{i}] {:?} AVAILABLE with empty supporting_metrics",
                            status.concept
                        ));
                    }
                }
                FactorAvailability::Unavailable => {
                    if status.missing_metrics.is_empty() {
                        failures.push(format!(
                            "profile[{i}] {:?} UNAVAILABLE without missing_metrics",
                            status.concept
                        ));
                    }
                }
            }
        }
    }
    failures
}
