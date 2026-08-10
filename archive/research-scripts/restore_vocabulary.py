import re
import os

def extract_block(text, name, block_type="struct"):
    # matches `pub struct Name { ... }` or `pub enum Name { ... }`
    pattern = r"(#\[.*?\]\n)*pub " + block_type + r"\s+" + name + r"\s*(\{.*?\})"
    match = re.search(pattern, text, re.DOTALL | re.MULTILINE)
    if match:
        return match.group(0)
    return None

def main():
    with open("archive/research_outputs/original_engine.rs", "r") as f:
        code = f.read()
    
    # Extract enums
    scenario_context = extract_block(code, "ScenarioContext", "enum")
    behavioral_archetype = extract_block(code, "BehavioralArchetype", "enum")
    execution_directive = extract_block(code, "ExecutionDirective", "enum")
    alpha_porosity = extract_block(code, "AlphaPorosity", "enum")
    signal_type = extract_block(code, "SignalType", "enum")
    reco_status = extract_block(code, "RecommendationStatus", "enum")
    signal_source = extract_block(code, "SignalSource", "enum")
    conviction_outcome = extract_block(code, "ConvictionOutcome", "enum")
    
    # Extract structs
    trade_reco = extract_block(code, "TradeRecommendation", "struct")
    alpha_consensus = extract_block(code, "AlphaConsensus", "struct")
    signal_alpha = extract_block(code, "SignalAlpha", "struct")
    
    # Also extract the Display impl for ScenarioContext
    display_impl = ""
    match = re.search(r"impl std::fmt::Display for ScenarioContext \{.*?\}", code, re.DOTALL)
    if match:
        display_impl = match.group(0)

    # Write market_regime.rs
    market_regime_path = "financial/strategies/src/market_regime.rs"
    with open(market_regime_path, "w") as f:
        f.write(f"use serde::{{Serialize, Deserialize}};\n\n")
        if scenario_context: f.write(scenario_context + "\n\n" + display_impl + "\n\n")
        if behavioral_archetype: f.write(behavioral_archetype + "\n\n")
        
        # Add the functions that were missing
        f.write("""
#[inline]
pub fn classify_direction_bias(direction_bias: u8) -> BehavioralArchetype {
    if direction_bias > 70 {
        BehavioralArchetype::LongSpecialist
    } else if direction_bias < 30 {
        BehavioralArchetype::ShortSpecialist
    } else {
        BehavioralArchetype::DualCore
    }
}
""")
        
    # Write execution vocabulary into a new module `vocabulary.rs` or `execution.rs`
    # Let's create `domain.rs` inside strategies
    domain_path = "financial/strategies/src/domain.rs"
    with open(domain_path, "w") as f:
        f.write("use serde::{Serialize, Deserialize};\n\n")
        f.write("pub use chronosentiment_optimization::Candidate as Strategy;\n")
        f.write("pub use chronosentiment_optimization::CandidateEvaluation as StrategyEvaluation;\n\n")
        if execution_directive: f.write(execution_directive + "\n\n")
        if alpha_porosity: f.write(alpha_porosity + "\n\n")
        if signal_type: f.write(signal_type + "\n\n")
        if reco_status: f.write(reco_status + "\n\n")
        if signal_source: f.write(signal_source + "\n\n")
        if conviction_outcome: f.write(conviction_outcome + "\n\n")
        if trade_reco: f.write(trade_reco + "\n\n")
        if alpha_consensus: f.write(alpha_consensus + "\n\n")
        if signal_alpha: f.write(signal_alpha + "\n\n")
        f.write("""
pub fn strategy_to_id(strategy: &Strategy) -> String {
    format!("{:02x}{:02x}{:02x}{:02x}", 
        strategy.queue_threshold % 256,
        strategy.base_edge % 256,
        strategy.take_profit % 256,
        strategy.stop_loss % 256
    )
}
""")

    # Fix lib.rs
    lib_path = "financial/strategies/src/lib.rs"
    with open(lib_path, "r") as f:
        lib_code = f.read()
    
    # Add domain module and remove optimization aliases
    lib_code = lib_code.replace("pub type Strategy = chronosentiment_optimization::Candidate;", "")
    lib_code = lib_code.replace("pub type StrategyEvaluation = chronosentiment_optimization::CandidateEvaluation;", "")
    lib_code = lib_code.replace("pub type RankStats = chronosentiment_optimization::EvaluationMetrics;", "")
    lib_code = lib_code.replace("pub type AlphaConsensus = chronosentiment_optimization::ConsensusMetric;", "")
    lib_code = lib_code.replace("pub type MarketRegime = chronosentiment_optimization::ScenarioContext;", "")
    lib_code = lib_code.replace("pub type DirectionArchetype = chronosentiment_optimization::BehavioralArchetype;", "")
    lib_code = lib_code.replace("pub type OrderIntent = chronosentiment_optimization::ExecutionDirective;", "")
    lib_code = lib_code.replace("pub type TradeRecommendation = chronosentiment_optimization::ExecutionProposal;", "")
    
    lib_code = lib_code.replace("pub mod ensemble;", "pub mod domain;\npub use domain::*;\npub mod ensemble;")
    lib_code = lib_code.replace("pub use edge_decay::*;", "pub mod edge_decay;\npub use edge_decay::*;")
    lib_code = lib_code.replace("pub use edge_half_life_estimator::*;", "pub mod edge_half_life_estimator;\npub use edge_half_life_estimator::*;")

    with open(lib_path, "w") as f:
        f.write(lib_code)

    # Fix pipeline.rs imports
    pipeline_path = "financial/strategies/src/pipeline.rs"
    if os.path.exists(pipeline_path):
        with open(pipeline_path, "r") as f:
            pipeline_code = f.read()
        pipeline_code = pipeline_code.replace("use crate::ga::{self, GaConfig};", "use chronosentiment_optimization::GaConfig;\nuse crate::domain::*;")
        pipeline_code = pipeline_code.replace("use crate::{MarketEvent, SimEvent};", "use chronosentiment_core::{MarketEvent, SimEvent};")
        pipeline_code = pipeline_code.replace("use crate::market_adapter::", "use chronosentiment_core::market_adapter::")
        pipeline_code = pipeline_code.replace("pub use crate::exit::ExitReason;", "pub use chronosentiment_core::exit::ExitReason;")
        pipeline_code = pipeline_code.replace("use crate::data_source::CandleSource;", "use chronosentiment_core::data_source::CandleSource;")
        pipeline_code = pipeline_code.replace("use crate::csv_source::CsvCandleSource;", "use chronosentiment_core::csv_source::CsvCandleSource;")
        pipeline_code = pipeline_code.replace("use crate::folder_source::FolderCandleSource;", "use chronosentiment_core::folder_source::FolderCandleSource;")
        pipeline_code = pipeline_code.replace("for (asset_name, csv_path) in assets_to_process {", "for (asset_name, csv_path) in assets_to_process.into_iter() {")
        with open(pipeline_path, "w") as f:
            f.write(pipeline_code)

    # Fix reco.rs imports
    reco_path = "financial/strategies/src/reco.rs"
    if os.path.exists(reco_path):
        with open(reco_path, "r") as f:
            reco_code = f.read()
        reco_code = reco_code.replace("use crate::Side;", "use chronosentiment_core::Side;")
        reco_code = reco_code.replace("use crate::ga::{StrategyEvaluation};", "use crate::domain::StrategyEvaluation;")
        reco_code = reco_code.replace("(1.0 + size).ln()", "(1.0_f64 + size as f64).ln()")
        reco_code = reco_code.replace("(1.0 - latency_impact).max(0.0)", "(1.0_f64 - latency_impact).max(0.0_f64)")
        with open(reco_path, "w") as f:
            f.write(reco_code)

    # Fix ensemble.rs imports
    ensemble_path = "financial/strategies/src/ensemble.rs"
    if os.path.exists(ensemble_path):
        with open(ensemble_path, "r") as f:
            ensemble_code = f.read()
        ensemble_code = ensemble_code.replace("use crate::ga::StrategyEvaluation;", "use crate::domain::StrategyEvaluation;")
        with open(ensemble_path, "w") as f:
            f.write(ensemble_code)

    # Fix paper.rs imports
    paper_path = "financial/strategies/src/paper.rs"
    if os.path.exists(paper_path):
        with open(paper_path, "r") as f:
            paper_code = f.read()
        paper_code = paper_code.replace("use crate::ga::{AlphaConsensus, SignalType, TradeRecommendation, RecommendationStatus, AlphaPorosity};", "use crate::domain::*;")
        paper_code = paper_code.replace("use crate::market_adapter::Candle;", "use chronosentiment_core::market_adapter::Candle;")
        with open(paper_path, "w") as f:
            f.write(paper_code)

    # Fix strategy_id.rs imports
    strategy_id_path = "financial/strategies/src/strategy_id.rs"
    if os.path.exists(strategy_id_path):
        with open(strategy_id_path, "r") as f:
            strategy_id_code = f.read()
        strategy_id_code = strategy_id_code.replace("use crate::ga::{strategy_to_id, Strategy};", "use crate::domain::*;")
        with open(strategy_id_path, "w") as f:
            f.write(strategy_id_code)

    # Fix strategy_ranking.rs imports
    ranking_path = "financial/strategies/src/strategy_ranking.rs"
    if os.path.exists(ranking_path):
        with open(ranking_path, "r") as f:
            ranking_code = f.read()
        ranking_code = ranking_code.replace("use crate::ga::Strategy;", "use crate::domain::*;")
        ranking_code = ranking_code.replace("use crate::NormalizedMarketEvent;", "use chronosentiment_core::NormalizedMarketEvent;")
        with open(ranking_path, "w") as f:
            f.write(ranking_code)

    # Fix market_regime.rs imports (remove missing ones)
    market_regime_path = "financial/strategies/src/market_regime.rs"
    if os.path.exists(market_regime_path):
        with open(market_regime_path, "r") as f:
            mr_code = f.read()
        mr_code = mr_code.replace("use chronosentiment_optimization::{ScenarioContext, BehavioralArchetype};", "")
        with open(market_regime_path, "w") as f:
            f.write(mr_code)
            
if __name__ == "__main__":
    main()
