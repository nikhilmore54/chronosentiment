use async_trait::async_trait;
use chrono::{DateTime, Utc, TimeZone, NaiveTime, Datelike};
use std::collections::HashMap;
use std::error::Error;
use uuid::Uuid;
use sqlx::PgPool;

use chronosentiment_adapter::repository::postgres_knowledge::PostgresKnowledgeRepository;
use chronosentiment_adapter::repository::knowledge::ArtifactRepository;

use chronosentiment_adapter::instrument::Instrument;
use chronosentiment_adapter::observation::ValidatedObservation;
use chronosentiment_adapter::repository::observation_repository::ValidatedObservationRepository;
use chronosentiment_adapter::validation::replay::{ReplayEngine, ReplayRequest};
use chronosentiment_adapter::metrics::instrument::{InstrumentMetricEngine, SimpleMovingAverageMetric, RateOfChangeMetric, AverageTrueRangeMetric};
use coralys_moga::runtime::optimization::metric::MetricEngine;
use chronosentiment_adapter::metrics::concepts::Concept;
use chronosentiment_adapter::reasoning::assessment::AssessmentEngine;
use chronosentiment_adapter::reasoning::evidence::EvidenceEngine;
use chronosentiment_adapter::reasoning::historical_reasoning::HistoricalReasoningEngine;
use chronosentiment_adapter::reasoning::hypothesis::HypothesisEngine;
use chronosentiment_adapter::reasoning::decision::{DecisionEngine, Opportunity};
use chronosentiment_adapter::reasoning::strategy::StrategyEngine;
use chronosentiment_adapter::ingestion::provider::{MarketDataProvider, ValidatedObservationTranslator};
use chronosentiment_adapter::ingestion::yahoo::{YahooProvider, YahooTranslator};
use chronosentiment_adapter::validation::outcome::OutcomeEngine;

struct InMemoryObservationRepo {
    observations: Vec<ValidatedObservation>,
}

impl InMemoryObservationRepo {
    fn new() -> Self {
        Self { observations: Vec::new() }
    }
    
    fn insert_batch(&mut self, obs: Vec<ValidatedObservation>) {
        self.observations.extend(obs);
        self.observations.sort_by_key(|o| o.effective_from);
    }
}

#[async_trait]
impl ValidatedObservationRepository for InMemoryObservationRepo {
    async fn store_observation(&self, _observation: &ValidatedObservation) -> Result<(), Box<dyn Error>> { Ok(()) }

    async fn get_observations_as_of(
        &self,
        instrument_id: Uuid,
        evaluation_timestamp: DateTime<Utc>,
    ) -> Result<Vec<ValidatedObservation>, Box<dyn Error>> {
        Ok(self.observations.iter()
            .filter(|o| o.instrument_id == Some(instrument_id) && o.effective_from <= evaluation_timestamp)
            .cloned()
            .collect())
    }

    async fn get_complete_history(&self, instrument_id: Uuid) -> Result<Vec<ValidatedObservation>, Box<dyn Error>> {
        Ok(self.observations.iter()
            .filter(|o| o.instrument_id == Some(instrument_id))
            .cloned()
            .collect())
    }
}

// generate_replay_hash removed

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("============================================================");
    println!("       CHRONOSENTIMENT — KNOWLEDGE LAKE POPULATION");
    println!("============================================================\n");
    
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://nikhil@localhost:5432/postgres".to_string());
    let pool = PgPool::connect(&db_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    let knowledge_repo = PostgresKnowledgeRepository::new(pool.clone());
    
    let tickers = vec!["RELIANCE.NS", "TCS.NS", "INFY.NS", "HDFCBANK.NS", "ICICIBANK.NS"];
    let mut repo = InMemoryObservationRepo::new();
    let mut instrument_map = HashMap::new();
    let yahoo = YahooProvider::new();
    let translator = YahooTranslator;
    
    for ticker in &tickers {
        let id = Uuid::new_v4();
        let mut provider_ids = HashMap::new();
        provider_ids.insert("yahoo".to_string(), ticker.to_string());
        
        let instrument = Instrument {
            id,
            exchange: "NSE".to_string(),
            display_symbol: ticker.to_string(),
            provider_ids,
            created_at: Utc::now(),
        };
        
        instrument_map.insert(id, instrument.clone());
        
        sqlx::query("INSERT INTO instruments (id, exchange, display_symbol) VALUES ($1, $2, $3) ON CONFLICT (id) DO NOTHING")
            .bind(id)
            .bind(&instrument.exchange)
            .bind(&instrument.display_symbol)
            .execute(&pool)
            .await?;
        
        let raw_bars = yahoo.fetch_historical(&instrument, chronosentiment_adapter::ingestion::provider::TimeRange::FiveYears).await;
        if let Ok(bars) = raw_bars {
            let mut validated = Vec::new();
            for bar in bars {
                let raw_obs = translator.translate(bar, &instrument);
                let v_obs = ValidatedObservation {
                    id: Uuid::new_v4(),
                    research_session_id: None,
                    instrument_id: Some(id),
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
                validated.push(v_obs);
            }
            repo.insert_batch(validated);
        }
    }
    
    let mut timestamps = Vec::new();
    for year in 2021..=2024 {
        for month in 1..=12 {
            let next_month = if month == 12 { 1 } else { month + 1 };
            let next_year = if month == 12 { year + 1 } else { year };
            let d = chrono::NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap().pred_opt().unwrap();
            let dt = Utc.from_utc_datetime(&d.and_time(NaiveTime::from_hms_opt(15, 30, 0).unwrap()));
            timestamps.push(dt);
        }
    }

    let mut temporal_violations = 0;
    let mut hash_mismatches = 0;
    let mut decisions_replayed = 0;
    let mut pos_decisions = 0;
    let mut neu_decisions = 0;
    let mut neg_decisions = 0;
    
    let mut strategies_generated = 0;
    let mut outcomes_evaluated = 0;
    
    let mut entry_reached = 0;
    let mut entry_not_reached = 0;
    let mut target_hit = 0;
    let mut stop_hit = 0;
    let mut ambiguous_hit = 0;
    let mut expired = 0;
    
    let mut win_count = 0;
    // Unused metric arrays removed
    
    let mut metric_engine = InstrumentMetricEngine::new();
    metric_engine.add_model(Box::new(SimpleMovingAverageMetric::new(20)));
    metric_engine.add_model(Box::new(SimpleMovingAverageMetric::new(50)));
    metric_engine.add_model(Box::new(RateOfChangeMetric::new(14)));
    metric_engine.add_model(Box::new(AverageTrueRangeMetric::new(14)));
    
    let replay_engine = ReplayEngine::new(&repo);
    let decision_engine = DecisionEngine;
    let strategy_engine = StrategyEngine;
    
    for dt in &timestamps {
        for (inst_id, _inst) in &instrument_map {
            let req = ReplayRequest {
                research_session_id: "val_gate".to_string(),
                universe: "Nifty50".to_string(),
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
                continue; // Not enough data
            }

            for obs in &inst_context.observations {
                if obs.effective_from > *dt {
                    temporal_violations += 1;
                    panic!("TEMPORAL VIOLATION: Observation leaked.");
                }
            }
            
            let metric_report = metric_engine.evaluate(inst_context);
            let profile = AssessmentEngine.assess_at(
                &metric_report,
                &[Concept::Trend, Concept::Momentum, Concept::Volatility],
                *dt,
                Some(*inst_id),
            );
            let _evidence = EvidenceEngine.evaluate(&profile);
            
            knowledge_repo.store(&profile).await?;

            let decision = decision_engine.evaluate(&profile, *dt, *inst_id);
            knowledge_repo.store(&decision).await?;
            decisions_replayed += 1;
            
            match decision.opportunity {
                Opportunity::Positive => pos_decisions += 1,
                Opportunity::Neutral => neu_decisions += 1,
                Opportunity::Negative => neg_decisions += 1,
            }
            
            let current_close = inst_context.observations.last().and_then(|o| o.normalized_payload.get("close")).and_then(|v| v.as_f64()).unwrap_or(0.0);
            let atr = metric_report.get_float("atr_14").unwrap_or(current_close * 0.02);
            
            let strategy_opt = strategy_engine.generate(&decision, current_close, atr);
            
            if let Some(strategy) = strategy_opt {
                strategies_generated += 1;
                knowledge_repo.store(&strategy).await?;
                
                let full_history = repo.get_complete_history(*inst_id).await.unwrap();
                let outcome_engine = OutcomeEngine;
                
                for horizon in [5, 10, 20, 60] {
                    let outcome = outcome_engine.measure_outcome(
                        decision.decision_id,
                        &strategy,
                        &strategy.metadata,
                        &full_history,
                        *dt,
                        horizon,
                        None
                    );
                    
                    knowledge_repo.store(&outcome).await?;
                    outcomes_evaluated += 1;
                }
            }
        }
    }
    
    println!("PHASE 4 DATA POPULATION");
    println!("────────────────────────────────────");
    println!("Knowledge Lake Populated Successfully.");
    println!("Outcomes generated per strategy: 4 (5D, 10D, 20D, 60D)");
    println!("────────────────────────────────────\n");

    println!("Total Decisions: {}", decisions_replayed);
    println!("Total Strategies: {}", strategies_generated);
    println!("Total Outcomes: {}", outcomes_evaluated);
    
    println!("\n============================================================");
    println!("KNOWLEDGE LAKE POPULATION: COMPLETE");
    println!("============================================================");

    Ok(())
}
