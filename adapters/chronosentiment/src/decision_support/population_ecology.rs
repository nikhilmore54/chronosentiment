//! CS-P-006-C.2-P — population ecology of the Search #1 archive.
//!
//! Consumes a C.2-O `SearchArchive`. Does not evolve, retune, choose a
//! volatility encoding, or score the evaluation slice.

use std::collections::BTreeSet;

use serde::Serialize;

use super::search_observability::{
    archive_satisfies_contract, SearchArchive, SelectedInstrumentVisibility, SerializedGenome,
};
use super::DecisionAction;

pub const ECOLOGY_CONTRACT_ID: &str = "csp006c2p.population_ecology.1";
pub const VERDICT_EXPLORED: &str = "SEARCH-SPACE EXPLORED";
pub const VERDICT_UNDER_EXPLORED: &str = "SEARCH-SPACE UNDER-EXPLORED";
pub const VERDICT_INDETERMINATE: &str = "INDETERMINATE";

/// A family is recurrent if it occupies at least this many population-slots
/// and appears in at least this many generations. Pre-declared before replay.
pub const RECURRENT_SLOT_FLOOR: u32 = 8;
pub const RECURRENT_GENERATION_FLOOR: usize = 3;
pub const DIVERSITY_HIGH_MEAN_UNIQUE: f64 = 8.0;
pub const DIVERSITY_LOW_MEAN_UNIQUE: f64 = 4.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FamilyOccupancy {
    Absent,
    Trace,
    Recurrent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DiversityBand {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MomentumQuestion {
    ExploredAndUnusedByWinner,
    BarelyExplored,
    Indeterminate,
}

#[derive(Debug, Clone, Serialize)]
pub struct FamilyCensus {
    pub slots: u32,
    pub generations_present: usize,
    pub occupancy: FamilyOccupancy,
}

#[derive(Debug, Clone, Serialize)]
pub struct GenerationEcology {
    pub generation: usize,
    pub population_size: usize,
    pub unique_genome_count: usize,
    pub best_fitness: f64,
    pub median_fitness: f64,
    pub mean_fitness: f64,
    pub worst_fitness: f64,
    pub genomes_with_long: u32,
    pub genomes_with_short: u32,
    pub genomes_with_no_trade: u32,
    pub genomes_using_trend: u32,
    pub genomes_using_momentum: u32,
    pub genomes_using_volatility: u32,
    pub near_best_count: usize,
    pub generation_best_identity: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DistinctNearBest {
    pub identity: String,
    pub development_fitness: f64,
    pub uses_trend: bool,
    pub uses_momentum: bool,
    pub uses_volatility: bool,
    pub emits_long: bool,
    pub emits_short: bool,
    pub emits_no_trade: bool,
    pub unmatched_action: DecisionAction,
}

#[derive(Debug, Clone, Serialize)]
pub struct PopulationEcologyReport {
    pub contract_id: String,
    pub search_two_authorized: bool,
    pub evaluation_scored: bool,
    pub archive_satisfies_observability: bool,
    pub n_generations: usize,
    pub population_slots: u32,
    pub unique_genome_count_by_generation: Vec<usize>,
    pub mean_unique_genome_count: f64,
    pub min_unique_genome_count: usize,
    pub max_unique_genome_count: usize,
    pub diversity_band: DiversityBand,
    pub generations: Vec<GenerationEcology>,
    pub trend: FamilyCensus,
    pub momentum: FamilyCensus,
    pub volatility: FamilyCensus,
    pub long_action: FamilyCensus,
    pub short_action: FamilyCensus,
    pub no_trade_action: FamilyCensus,
    pub unique_generation_best_identities: Vec<String>,
    pub unique_near_best_identities: Vec<String>,
    pub distinct_near_best_from_selected: Vec<DistinctNearBest>,
    pub selected_identity: Option<String>,
    pub winner_uses_momentum: bool,
    pub winner_emits_short: bool,
    pub selected_instruments: Option<SelectedInstrumentVisibility>,
    pub momentum_question: MomentumQuestion,
    pub verdict: String,
    pub verdict_reason: String,
}

fn occupancy(slots: u32, generations_present: usize) -> FamilyOccupancy {
    if slots == 0 {
        FamilyOccupancy::Absent
    } else if slots >= RECURRENT_SLOT_FLOOR && generations_present >= RECURRENT_GENERATION_FLOOR {
        FamilyOccupancy::Recurrent
    } else {
        FamilyOccupancy::Trace
    }
}

fn census(slots: u32, generations_present: usize) -> FamilyCensus {
    FamilyCensus {
        slots,
        generations_present,
        occupancy: occupancy(slots, generations_present),
    }
}

fn genome_uses(genome: &SerializedGenome, concept: &str) -> bool {
    genome
        .rules
        .iter()
        .any(|r| r.when.iter().any(|p| p.concept == concept))
}

fn genome_emits(genome: &SerializedGenome, action: DecisionAction) -> bool {
    genome.unmatched_action == action || genome.rules.iter().any(|r| r.action == action)
}

fn distinct_profile(genome: &SerializedGenome) -> DistinctNearBest {
    DistinctNearBest {
        identity: genome.identity.clone(),
        development_fitness: genome.development_fitness,
        uses_trend: genome_uses(genome, "Trend"),
        uses_momentum: genome_uses(genome, "Momentum"),
        uses_volatility: genome_uses(genome, "Volatility"),
        emits_long: genome_emits(genome, DecisionAction::Long),
        emits_short: genome_emits(genome, DecisionAction::Short),
        emits_no_trade: genome_emits(genome, DecisionAction::NoTrade),
        unmatched_action: genome.unmatched_action,
    }
}

fn profiles_differ(a: &DistinctNearBest, b: &DistinctNearBest) -> bool {
    a.uses_trend != b.uses_trend
        || a.uses_momentum != b.uses_momentum
        || a.uses_volatility != b.uses_volatility
        || a.emits_long != b.emits_long
        || a.emits_short != b.emits_short
        || a.emits_no_trade != b.emits_no_trade
        || a.unmatched_action != b.unmatched_action
}

fn classify(
    momentum: FamilyOccupancy,
    short: FamilyOccupancy,
    diversity: DiversityBand,
    winner_uses_momentum: bool,
) -> (String, String, MomentumQuestion) {
    let momentum_question = match (momentum, winner_uses_momentum) {
        (FamilyOccupancy::Recurrent, false) => MomentumQuestion::ExploredAndUnusedByWinner,
        (FamilyOccupancy::Absent | FamilyOccupancy::Trace, false) => {
            MomentumQuestion::BarelyExplored
        }
        _ => MomentumQuestion::Indeterminate,
    };

    let under = matches!(momentum, FamilyOccupancy::Absent | FamilyOccupancy::Trace)
        && matches!(short, FamilyOccupancy::Absent | FamilyOccupancy::Trace);
    let explored = momentum == FamilyOccupancy::Recurrent
        && short == FamilyOccupancy::Recurrent
        && diversity == DiversityBand::High;

    if explored {
        (
            VERDICT_EXPLORED.to_string(),
            "Momentum and SHORT were recurrent across generations, and mean unique-genome count stayed high.".to_string(),
            momentum_question,
        )
    } else if under {
        (
            VERDICT_UNDER_EXPLORED.to_string(),
            "Momentum and SHORT were absent or only trace occupants of the evaluated population."
                .to_string(),
            momentum_question,
        )
    } else {
        (
            VERDICT_INDETERMINATE.to_string(),
            "Factor, action, and diversity signals do not all point the same way.".to_string(),
            momentum_question,
        )
    }
}

pub fn analyze_search_archive(
    archive: &SearchArchive,
    selected_identity: Option<&str>,
) -> Result<PopulationEcologyReport, String> {
    if !archive_satisfies_contract(archive) {
        return Err("archive does not satisfy the C.2-O observability contract".into());
    }
    let n_generations = archive.generations.len();
    let population_slots: u32 = archive
        .generations
        .iter()
        .map(|g| g.population_size as u32)
        .sum();
    let unique_genome_count_by_generation: Vec<usize> = archive
        .generations
        .iter()
        .map(|g| g.unique_genome_count)
        .collect();
    let mean_unique =
        unique_genome_count_by_generation.iter().sum::<usize>() as f64 / n_generations as f64;
    let min_unique = *unique_genome_count_by_generation.iter().min().unwrap();
    let max_unique = *unique_genome_count_by_generation.iter().max().unwrap();
    let diversity_band = if mean_unique >= DIVERSITY_HIGH_MEAN_UNIQUE {
        DiversityBand::High
    } else if mean_unique < DIVERSITY_LOW_MEAN_UNIQUE {
        DiversityBand::Low
    } else {
        DiversityBand::Medium
    };

    let mut trend_slots = 0u32;
    let mut momentum_slots = 0u32;
    let mut volatility_slots = 0u32;
    let mut long_slots = 0u32;
    let mut short_slots = 0u32;
    let mut no_trade_slots = 0u32;
    let mut trend_gens = 0usize;
    let mut momentum_gens = 0usize;
    let mut volatility_gens = 0usize;
    let mut long_gens = 0usize;
    let mut short_gens = 0usize;
    let mut no_trade_gens = 0usize;
    let mut generation_bests = BTreeSet::new();
    let mut near_best_ids = BTreeSet::new();
    let mut generations = Vec::new();

    for g in &archive.generations {
        if g.factor_consumption.genomes_using_trend > 0 {
            trend_gens += 1;
        }
        if g.factor_consumption.genomes_using_momentum > 0 {
            momentum_gens += 1;
        }
        if g.factor_consumption.genomes_using_volatility > 0 {
            volatility_gens += 1;
        }
        if g.action_symbols.genomes_with_long > 0 {
            long_gens += 1;
        }
        if g.action_symbols.genomes_with_short > 0 {
            short_gens += 1;
        }
        if g.action_symbols.genomes_with_no_trade > 0 {
            no_trade_gens += 1;
        }
        trend_slots += g.factor_consumption.genomes_using_trend;
        momentum_slots += g.factor_consumption.genomes_using_momentum;
        volatility_slots += g.factor_consumption.genomes_using_volatility;
        long_slots += g.action_symbols.genomes_with_long;
        short_slots += g.action_symbols.genomes_with_short;
        no_trade_slots += g.action_symbols.genomes_with_no_trade;
        generation_bests.insert(g.generation_best.identity.clone());
        for nb in &g.near_best {
            near_best_ids.insert(nb.identity.clone());
        }
        generations.push(GenerationEcology {
            generation: g.generation,
            population_size: g.population_size,
            unique_genome_count: g.unique_genome_count,
            best_fitness: g.best_fitness,
            median_fitness: g.median_fitness,
            mean_fitness: g.mean_fitness,
            worst_fitness: g.worst_fitness,
            genomes_with_long: g.action_symbols.genomes_with_long,
            genomes_with_short: g.action_symbols.genomes_with_short,
            genomes_with_no_trade: g.action_symbols.genomes_with_no_trade,
            genomes_using_trend: g.factor_consumption.genomes_using_trend,
            genomes_using_momentum: g.factor_consumption.genomes_using_momentum,
            genomes_using_volatility: g.factor_consumption.genomes_using_volatility,
            near_best_count: g.near_best.len(),
            generation_best_identity: g.generation_best.identity.clone(),
        });
    }

    let trend = census(trend_slots, trend_gens);
    let momentum = census(momentum_slots, momentum_gens);
    let volatility = census(volatility_slots, volatility_gens);
    let long_action = census(long_slots, long_gens);
    let short_action = census(short_slots, short_gens);
    let no_trade_action = census(no_trade_slots, no_trade_gens);

    let selected = selected_identity.map(|s| s.to_string()).or_else(|| {
        archive
            .generations
            .first()
            .map(|g| g.generation_best.identity.clone())
    });
    let winner = selected.as_ref().and_then(|id| {
        archive
            .generations
            .iter()
            .flat_map(|g| std::iter::once(&g.generation_best).chain(g.near_best.iter()))
            .find(|g| &g.identity == id)
    });
    let winner_uses_momentum = winner.map(|g| genome_uses(g, "Momentum")).unwrap_or(false);
    let winner_emits_short = winner
        .map(|g| genome_emits(g, DecisionAction::Short))
        .unwrap_or(false);
    let winner_profile = winner.map(distinct_profile);

    let mut distinct_near_best_from_selected = Vec::new();
    let mut seen = BTreeSet::new();
    for g in &archive.generations {
        for cand in std::iter::once(&g.generation_best).chain(g.near_best.iter()) {
            if selected.as_ref().is_some_and(|id| &cand.identity == id) {
                continue;
            }
            if !seen.insert(cand.identity.clone()) {
                continue;
            }
            let profile = distinct_profile(cand);
            if winner_profile
                .as_ref()
                .map(|w| profiles_differ(w, &profile))
                .unwrap_or(true)
            {
                distinct_near_best_from_selected.push(profile);
            }
        }
    }

    let (verdict, verdict_reason, momentum_question) = classify(
        momentum.occupancy,
        short_action.occupancy,
        diversity_band,
        winner_uses_momentum,
    );

    Ok(PopulationEcologyReport {
        contract_id: ECOLOGY_CONTRACT_ID.to_string(),
        search_two_authorized: false,
        evaluation_scored: false,
        archive_satisfies_observability: true,
        n_generations,
        population_slots,
        unique_genome_count_by_generation,
        mean_unique_genome_count: mean_unique,
        min_unique_genome_count: min_unique,
        max_unique_genome_count: max_unique,
        diversity_band,
        generations,
        trend,
        momentum,
        volatility,
        long_action,
        short_action,
        no_trade_action,
        unique_generation_best_identities: generation_bests.into_iter().collect(),
        unique_near_best_identities: near_best_ids.into_iter().collect(),
        distinct_near_best_from_selected,
        selected_identity: selected,
        winner_uses_momentum,
        winner_emits_short,
        selected_instruments: archive.selected_instruments.clone(),
        momentum_question,
        verdict,
        verdict_reason,
    })
}

pub fn render_ecology(report: &PopulationEcologyReport) -> String {
    let mut out = String::new();
    out.push_str("# Search #1 population ecology\n\n");
    out.push_str("Identity-gated C.2-O replay of Search #1. Not Search #2.\n\n");
    out.push_str(&format!("**Verdict:** `{}`\n\n", report.verdict));
    out.push_str(&format!("{}\n\n", report.verdict_reason));
    out.push_str(&format!(
        "Momentum question: `{:?}`\n\n",
        report.momentum_question
    ));
    out.push_str("| Generation | Unique | Best | Median | Mean | Worst | LONG | SHORT | NO_TRADE | Trend | Momentum | Volatility | Near-best |\n");
    out.push_str("|------------|--------|------|--------|------|-------|------|-------|----------|-------|----------|------------|-----------|\n");
    for g in &report.generations {
        out.push_str(&format!(
            "| {} | {} | {:.6} | {:.6} | {:.6} | {:.6} | {} | {} | {} | {} | {} | {} | {} |\n",
            g.generation,
            g.unique_genome_count,
            g.best_fitness,
            g.median_fitness,
            g.mean_fitness,
            g.worst_fitness,
            g.genomes_with_long,
            g.genomes_with_short,
            g.genomes_with_no_trade,
            g.genomes_using_trend,
            g.genomes_using_momentum,
            g.genomes_using_volatility,
            g.near_best_count
        ));
    }
    out.push_str("\n## Family occupancy\n\n");
    out.push_str(&format!(
        "- Trend: {:?} ({} slots, {} generations)\n",
        report.trend.occupancy, report.trend.slots, report.trend.generations_present
    ));
    out.push_str(&format!(
        "- Momentum: {:?} ({} slots, {} generations)\n",
        report.momentum.occupancy, report.momentum.slots, report.momentum.generations_present
    ));
    out.push_str(&format!(
        "- Volatility: {:?} ({} slots, {} generations)\n",
        report.volatility.occupancy, report.volatility.slots, report.volatility.generations_present
    ));
    out.push_str(&format!(
        "- LONG / SHORT / NO_TRADE slots: {} / {} / {}\n",
        report.long_action.slots, report.short_action.slots, report.no_trade_action.slots
    ));
    out.push_str(&format!(
        "- Unique genomes: mean {:.2}, min {}, max {}\n",
        report.mean_unique_genome_count,
        report.min_unique_genome_count,
        report.max_unique_genome_count
    ));
    if let Some(vis) = &report.selected_instruments {
        out.push_str("\n## Selected candidate per-instrument (development / selection only)\n\n");
        out.push_str("| Instrument | Dev mean | Dev traded | Sel mean | Sel traded |\n");
        out.push_str("|------------|----------|------------|----------|------------|\n");
        for (d, s) in vis.development.iter().zip(vis.selection.iter()) {
            out.push_str(&format!(
                "| {} | {:.6} | {} | {:.6} | {} |\n",
                d.instrument,
                d.mean_signed_traded_return,
                d.n_traded,
                s.mean_signed_traded_return,
                s.n_traded
            ));
        }
    }
    out.push_str("\nEvaluation was not scored. Search #2 is not authorized.\n");
    out
}
