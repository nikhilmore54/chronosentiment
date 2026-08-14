use async_trait::async_trait;
use chrono::{DateTime, Utc, TimeZone, NaiveTime, Datelike};
use std::collections::HashMap;
use std::error::Error;
use uuid::Uuid;
use sha2::{Sha256, Digest};

use chronosentiment_adapter::instrument::Instrument;
use chronosentiment_adapter::observation::ValidatedObservation;
use chronosentiment_adapter::repository::observation_repository::ValidatedObservationRepository;
use chronosentiment_adapter::validation::replay::{ReplayEngine, ReplayRequest};
use chronosentiment_adapter::metrics::instrument::{InstrumentMetricEngine, SimpleMovingAverageMetric, RateOfChangeMetric, AverageTrueRangeMetric};
use coralys_moga::runtime::optimization::metric::MetricEngine;
use chronosentiment_adapter::metrics::concepts::Concept;
use chronosentiment_adapter::reasoning::assessment::AssessmentEngine;
use chronosentiment_adapter::ingestion::provider::{MarketDataProvider, ValidatedObservationTranslator, TimeRange};
use chronosentiment_adapter::ingestion::yahoo::{YahooProvider, YahooTranslator};

struct InMemoryObservationRepo {
    observations: Vec<ValidatedObservation>,
}

#[async_trait]
impl ValidatedObservationRepository for InMemoryObservationRepo {
    async fn get_observations_as_of(
        &self,
        _instrument_id: Uuid,
        timestamp: DateTime<Utc>,
    ) -> Result<Vec<ValidatedObservation>, Box<dyn Error>> {
        let mut filtered = Vec::new();
        for obs in &self.observations {
            if obs.effective_from <= timestamp {
                filtered.push(obs.clone());
            }
        }
        filtered.sort_by_key(|o| o.effective_from);
        Ok(filtered)
    }

    async fn store_observation(&self, _observation: &ValidatedObservation) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    
    async fn get_complete_history(&self, _instrument_id: Uuid) -> Result<Vec<ValidatedObservation>, Box<dyn Error>> {
        Ok(self.observations.clone())
    }
}

struct ProfileStats {
    signature: String,
    hash: String,
    n_samples: usize,
    
    returns_5d: Vec<f64>, returns_10d: Vec<f64>, returns_20d: Vec<f64>, returns_60d: Vec<f64>,
    mfes_5d: Vec<f64>, mfes_10d: Vec<f64>, mfes_20d: Vec<f64>, mfes_60d: Vec<f64>,
    maes_5d: Vec<f64>, maes_10d: Vec<f64>, maes_20d: Vec<f64>, maes_60d: Vec<f64>,
    drawdowns_5d: Vec<f64>, drawdowns_10d: Vec<f64>, drawdowns_20d: Vec<f64>, drawdowns_60d: Vec<f64>,
}

impl ProfileStats {
    fn new(signature: String, hash: String) -> Self {
        Self {
            signature, hash, n_samples: 0,
            returns_5d: Vec::new(), returns_10d: Vec::new(), returns_20d: Vec::new(), returns_60d: Vec::new(),
            mfes_5d: Vec::new(), mfes_10d: Vec::new(), mfes_20d: Vec::new(), mfes_60d: Vec::new(),
            maes_5d: Vec::new(), maes_10d: Vec::new(), maes_20d: Vec::new(), maes_60d: Vec::new(),
            drawdowns_5d: Vec::new(), drawdowns_10d: Vec::new(), drawdowns_20d: Vec::new(), drawdowns_60d: Vec::new(),
        }
    }
}

fn median(mut list: Vec<f64>) -> f64 {
    if list.is_empty() { return 0.0; }
    list.sort_by(|a, b| a.partial_cmp(b).unwrap());
    list[list.len() / 2]
}

fn mean(list: &[f64]) -> f64 {
    if list.is_empty() { return 0.0; }
    list.iter().sum::<f64>() / list.len() as f64
}

fn p10_p90(mut list: Vec<f64>) -> (f64, f64) {
    if list.is_empty() { return (0.0, 0.0); }
    list.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p10_idx = (list.len() as f64 * 0.1).floor() as usize;
    let p90_idx = (list.len() as f64 * 0.9).floor() as usize;
    (list[p10_idx], list[p90_idx.min(list.len() - 1)])
}

fn win_rate(list: &[f64]) -> f64 {
    if list.is_empty() { return 0.0; }
    let wins = list.iter().filter(|&&r| r > 0.0).count();
    (wins as f64 / list.len() as f64) * 100.0
}

fn compute_forward_stats(start_idx: usize, offset: usize, obs_list: &[ValidatedObservation]) -> Option<(f64, f64, f64, f64)> {
    if start_idx + offset >= obs_list.len() {
        return None;
    }
    
    let entry_obs = &obs_list[start_idx];
    let exit_obs = &obs_list[start_idx + offset];
    
    let entry_close = entry_obs.normalized_payload.get("close").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let exit_close = exit_obs.normalized_payload.get("close").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let entry_adj = entry_obs.normalized_payload.get("adj_close").and_then(|v| v.as_f64()).unwrap_or(entry_close);
    let exit_adj = exit_obs.normalized_payload.get("adj_close").and_then(|v| v.as_f64()).unwrap_or(exit_close);
    
    if entry_close <= 0.0 || entry_adj <= 0.0 {
        return None;
    }
    
    let ret = (exit_adj - entry_adj) / entry_adj;
    
    let mut highest_adj = entry_adj;
    let mut lowest_adj = entry_adj;
    
    for i in (start_idx + 1)..=(start_idx + offset) {
        let obs = &obs_list[i];
        let c = obs.normalized_payload.get("close").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let a = obs.normalized_payload.get("adj_close").and_then(|v| v.as_f64()).unwrap_or(c);
        let ratio = if c > 0.0 { a / c } else { 1.0 };
        
        let h = obs.normalized_payload.get("high").and_then(|v| v.as_f64()).unwrap_or(c);
        let l = obs.normalized_payload.get("low").and_then(|v| v.as_f64()).unwrap_or(c);
        
        let h_adj = h * ratio;
        let l_adj = l * ratio;
        
        if h_adj > highest_adj { highest_adj = h_adj; }
        if l_adj < lowest_adj { lowest_adj = l_adj; }
    }
    
    let mfe = (highest_adj - entry_adj) / entry_adj;
    let mae = (lowest_adj - entry_adj) / entry_adj;
    let drawdown = -mae; 
    
    Some((ret, mfe, mae, drawdown))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("============================================================");
    println!("    CHRONOSENTIMENT — PHASE 5.1 PREDICTIVE VALUE LAB");
    println!("============================================================");
    
    let provider = YahooProvider::new();
    let translator = YahooTranslator;
    let symbols = vec!["AAPL", "MSFT", "GOOGL", "AMZN", "META"];
    
    let start_date = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();
    let end_date = Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap();
    
    let mut all_obs = Vec::new();
    let mut instrument_map = HashMap::new();
    let mut all_timestamps = Vec::new();
    
    println!("Ingesting historical data for 5-stock corpus (2021-2024)...");
    
    
    
    for symbol in &symbols {
        let inst_id = Uuid::new_v4();
        
        let mut p_ids = HashMap::new();
        p_ids.insert("yahoo".to_string(), symbol.to_string());
        
        let inst = Instrument {
            id: inst_id,
            exchange: "NASDAQ".to_string(),
            display_symbol: symbol.to_string(),
            provider_ids: p_ids,
            created_at: Utc::now(),
        };
        instrument_map.insert(inst_id, inst.clone());
        
        if let Ok(raw_data) = provider.fetch_historical(&inst, TimeRange::FiveYears).await {
            for bar in raw_data {
                let raw_obs = translator.translate(bar, &inst);
                let obs = ValidatedObservation {
                    id: Uuid::new_v4(),
                    research_session_id: None,
                    instrument_id: Some(inst_id),
                    observation_type: raw_obs.observation_type,
                    source: raw_obs.source,
                    source_identifier: raw_obs.source_identifier,
                    observed_at: raw_obs.observed_at,
                    effective_from: raw_obs.observed_at,
                    effective_to: None,
                    recorded_at: Utc::now(),
                    raw_payload: raw_obs.raw_payload,
                    normalized_payload: raw_obs.normalized_payload,
                    confidence: 1.0,
                    freshness: 0.0,
                    coverage: "Full".to_string(),
                    consistency: Some(1.0),
                    quality_score: 1.0,
                    provenance_hash: "hash".to_string(),
                    schema_version: 1,
                };
                all_timestamps.push(obs.effective_from);
                all_obs.push(obs);
            }
        }
    }
    
    all_timestamps.sort();
    all_timestamps.dedup();
    
    let repo = InMemoryObservationRepo { observations: all_obs.clone() };
    let replay_engine = ReplayEngine::new(&repo);
    let assessment_engine = AssessmentEngine;
    
    let mut metric_engine = InstrumentMetricEngine::new();
    metric_engine.add_model(Box::new(SimpleMovingAverageMetric::new(20)));
    metric_engine.add_model(Box::new(SimpleMovingAverageMetric::new(50)));
    metric_engine.add_model(Box::new(RateOfChangeMetric::new(14)));
    metric_engine.add_model(Box::new(AverageTrueRangeMetric::new(14)));
    
    let mut profile_stats: HashMap<String, ProfileStats> = HashMap::new();

    println!("Simulating Time Machine...");
    let mut progress = 0;
    
    for dt in &all_timestamps {
        progress += 1;
        if progress % 500 == 0 {
            println!("  Processed {} timestamps...", progress);
        }
        
        for (inst_id, _inst) in &instrument_map {
            let req = ReplayRequest {
                research_session_id: "m5_lab".to_string(),
                universe: "Corpus".to_string(),
                evaluation_timestamp: *dt,
                portfolio_snapshot: None,
                policy_snapshot: None,
                target_instrument_id: *inst_id,
            };
            
            let context = match replay_engine.generate_context(req).await {
                Ok(ctx) => ctx,
                Err(_) => continue,
            };
            
            let inst_context = match context.instrument_contexts.get(inst_id) {
                Some(ctx) => ctx,
                None => continue,
            };
            
            if inst_context.observations.len() < 50 {
                continue; // Burn-in
            }
            
            // Generate Profile safely without future leak
            let metric_report = metric_engine.evaluate(inst_context);
            let profile = assessment_engine.assess(&metric_report, &[Concept::Trend, Concept::Momentum]);
            
            let sig = profile.to_signature();
            let hash = profile.to_hash();
            
            let stats = profile_stats.entry(hash.clone()).or_insert_with(|| ProfileStats::new(sig, hash));
            stats.n_samples += 1;
            
            // FORWARD MEASUREMENT OUTSIDE KNOWLEDGE(T)
            // Get full unbounded history
            let mut instrument_history = Vec::new();
            for o in &all_obs {
                if o.instrument_id == Some(*inst_id) {
                    instrument_history.push(o.clone());
                }
            }
            instrument_history.sort_by_key(|o| o.effective_from);
            
            let current_idx = instrument_history.iter().position(|o| o.effective_from == *dt);
            if let Some(i) = current_idx {
                if let Some((ret, mfe, mae, dd)) = compute_forward_stats(i, 5, &instrument_history) {
                    stats.returns_5d.push(ret); stats.mfes_5d.push(mfe); stats.maes_5d.push(mae); stats.drawdowns_5d.push(dd);
                }
                if let Some((ret, mfe, mae, dd)) = compute_forward_stats(i, 10, &instrument_history) {
                    stats.returns_10d.push(ret); stats.mfes_10d.push(mfe); stats.maes_10d.push(mae); stats.drawdowns_10d.push(dd);
                }
                if let Some((ret, mfe, mae, dd)) = compute_forward_stats(i, 20, &instrument_history) {
                    stats.returns_20d.push(ret); stats.mfes_20d.push(mfe); stats.maes_20d.push(mae); stats.drawdowns_20d.push(dd);
                }
                if let Some((ret, mfe, mae, dd)) = compute_forward_stats(i, 60, &instrument_history) {
                    stats.returns_60d.push(ret); stats.mfes_60d.push(mfe); stats.maes_60d.push(mae); stats.drawdowns_60d.push(dd);
                }
            }
        }
    }
    
    println!("\nRESEARCH COMPLETE. Aggregating results...\n");
    
    let mut keys: Vec<_> = profile_stats.keys().cloned().collect();
    keys.sort();
    
    for key in keys {
        let stats = &profile_stats[&key];
        if stats.n_samples == 0 { continue; }
        
        let sig = &stats.signature;
        println!("### Profile: {}", sig);
        println!("Hash: {}", stats.hash);
        println!("N = {}\n", stats.n_samples);
        
        println!("| Metric            | 5D | 10D | 20D | 60D |");
        println!("| ----------------- | -: | --: | --: | --: |");
        
        let r5 = &stats.returns_5d; let r10 = &stats.returns_10d; let r20 = &stats.returns_20d; let r60 = &stats.returns_60d;
        
        let (p10_5, p90_5) = p10_p90(r5.clone()); let (p10_10, p90_10) = p10_p90(r10.clone());
        let (p10_20, p90_20) = p10_p90(r20.clone()); let (p10_60, p90_60) = p10_p90(r60.clone());
        
        println!("| Positive return % | {:.1}% | {:.1}% | {:.1}% | {:.1}% |", win_rate(r5), win_rate(r10), win_rate(r20), win_rate(r60));
        println!("| Median return     | {:.2}% | {:.2}% | {:.2}% | {:.2}% |", median(r5.clone())*100.0, median(r10.clone())*100.0, median(r20.clone())*100.0, median(r60.clone())*100.0);
        println!("| Mean return       | {:.2}% | {:.2}% | {:.2}% | {:.2}% |", mean(r5)*100.0, mean(r10)*100.0, mean(r20)*100.0, mean(r60)*100.0);
        println!("| Median MFE        | {:.2}% | {:.2}% | {:.2}% | {:.2}% |", median(stats.mfes_5d.clone())*100.0, median(stats.mfes_10d.clone())*100.0, median(stats.mfes_20d.clone())*100.0, median(stats.mfes_60d.clone())*100.0);
        println!("| Median MAE        | {:.2}% | {:.2}% | {:.2}% | {:.2}% |", median(stats.maes_5d.clone())*100.0, median(stats.maes_10d.clone())*100.0, median(stats.maes_20d.clone())*100.0, median(stats.maes_60d.clone())*100.0);
        println!("| Median drawdown   | {:.2}% | {:.2}% | {:.2}% | {:.2}% |", median(stats.drawdowns_5d.clone())*100.0, median(stats.drawdowns_10d.clone())*100.0, median(stats.drawdowns_20d.clone())*100.0, median(stats.drawdowns_60d.clone())*100.0);
        println!("| P10 return        | {:.2}% | {:.2}% | {:.2}% | {:.2}% |", p10_5*100.0, p10_10*100.0, p10_20*100.0, p10_60*100.0);
        println!("| P90 return        | {:.2}% | {:.2}% | {:.2}% | {:.2}% |", p90_5*100.0, p90_10*100.0, p90_20*100.0, p90_60*100.0);
        
        let mut best_horizon = "5D";
        let mut best_win_rate = win_rate(r5);
        if win_rate(r10) > best_win_rate { best_win_rate = win_rate(r10); best_horizon = "10D"; }
        if win_rate(r20) > best_win_rate { best_win_rate = win_rate(r20); best_horizon = "20D"; }
        if win_rate(r60) > best_win_rate { best_win_rate = win_rate(r60); best_horizon = "60D"; }
        
        println!("\nBest empirical horizon: {} (based on positive return probability of {:.1}%)\n", best_horizon, best_win_rate);
    }
    
    Ok(())
}
