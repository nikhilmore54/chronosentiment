use async_trait::async_trait;
use serde_json::json;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::research::dataset::ResearchDataset;
use crate::research::experiment::{ExperimentMeasurements, ResearchExperiment};

pub struct PredictiveValueExperiment {
    id: Uuid,
    pool: PgPool,
}

impl PredictiveValueExperiment {
    pub fn new(pool: PgPool) -> Self {
        Self {
            id: Uuid::new_v4(),
            pool,
        }
    }
}

#[async_trait]
impl ResearchExperiment for PredictiveValueExperiment {
    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        "Predictive Value Experiment"
    }

    fn description(&self) -> &str {
        "Extracts empirical outcome observations for each assessment profile signature."
    }

    async fn execute(
        &self,
        dataset: &ResearchDataset,
    ) -> Result<ExperimentMeasurements, Box<dyn std::error::Error + Send + Sync>> {
        // Query to fetch the raw evidence ledger, joining via proper JSONB containment
        let query = r#"
            SELECT 
                a.id as assessment_id,
                a.signature,
                a.signature_hash,
                a.evaluation_timestamp,
                
                d.id as decision_id,
                
                s.id as strategy_id,
                
                o.id as outcome_id,
                o.horizon,
                o.entry_reached,
                o.target_hit,
                o.stop_hit,
                o.outcome_return,
                o.mfe,
                o.mae,
                o.drawdown,
                o.exit_reason
            FROM knowledge_assessments a
            JOIN knowledge_decisions d 
                ON d.metadata_json->'lineage'->'parent_artifacts' @> to_jsonb(a.id::text)
            JOIN knowledge_strategies s 
                ON s.metadata_json->'lineage'->'parent_artifacts' @> to_jsonb(d.id::text)
            JOIN knowledge_outcomes o 
                ON o.strategy_id = s.id
            WHERE a.evaluation_timestamp >= $1 
              AND a.evaluation_timestamp <= $2
        "#;

        let rows = sqlx::query(query)
            .bind(dataset.date_range.start)
            .bind(dataset.date_range.end)
            .fetch_all(&self.pool)
            .await?;

        let mut raw_ledger = Vec::new();
        
        // Structures for aggregation
        use std::collections::HashMap;
        // Key: (signature_hash, signature, horizon)
        let mut aggregate_map: HashMap<(String, String, String), Vec<serde_json::Value>> = HashMap::new();
        // Key: (signature_hash, signature)
        let mut pop_map: HashMap<(String, String), Vec<serde_json::Value>> = HashMap::new();

        for row in rows {
            let assessment_id: Uuid = row.try_get("assessment_id")?;
            let signature: String = row.try_get("signature")?;
            let signature_hash: String = row.try_get("signature_hash")?;
            let eval_ts: chrono::DateTime<chrono::Utc> = row.try_get("evaluation_timestamp")?;
            
            let decision_id: Uuid = row.try_get("decision_id")?;
            let strategy_id: Uuid = row.try_get("strategy_id")?;
            let outcome_id: Uuid = row.try_get("outcome_id")?;
            
            let horizon: String = row.try_get("horizon")?;
            
            let entry_reached: bool = row.try_get("entry_reached")?;
            let target_hit: bool = row.try_get("target_hit")?;
            let stop_hit: bool = row.try_get("stop_hit")?;
            
            let outcome_return: f64 = row.try_get("outcome_return")?;
            let mfe: f64 = row.try_get("mfe")?;
            let mae: f64 = row.try_get("mae")?;
            let drawdown: f64 = row.try_get("drawdown")?;
            let exit_reason: String = row.try_get("exit_reason")?;
            
            let row_json = json!({
                "assessment_id": assessment_id,
                "decision_id": decision_id,
                "strategy_id": strategy_id,
                "outcome_id": outcome_id,
                "signature": signature,
                "signature_hash": signature_hash,
                "evaluation_timestamp": eval_ts,
                "horizon": horizon,
                "entry_reached": entry_reached,
                "target_hit": target_hit,
                "stop_hit": stop_hit,
                "outcome_return": outcome_return,
                "mfe": mfe,
                "mae": mae,
                "drawdown": drawdown,
                "exit_reason": exit_reason
            });
            
            raw_ledger.push(row_json.clone());
            
            let agg_key = (signature_hash.clone(), signature.clone(), horizon.clone());
            aggregate_map.entry(agg_key).or_default().push(row_json.clone());
            
            let pop_key = (signature_hash.clone(), signature.clone());
            pop_map.entry(pop_key).or_default().push(row_json);
        }

        // 1. Profile x Horizon Aggregate Matrix
        let mut aggregate_matrix = Vec::new();
        for ((sig_hash, sig, horizon), items) in aggregate_map {
            let n = items.len() as f64;
            
            let entry_reached_count = items.iter().filter(|i| i["entry_reached"].as_bool().unwrap_or(false)).count() as f64;
            let target_hit_count = items.iter().filter(|i| i["target_hit"].as_bool().unwrap_or(false)).count() as f64;
            let stop_hit_count = items.iter().filter(|i| i["stop_hit"].as_bool().unwrap_or(false)).count() as f64;
            
            let entry_pct = entry_reached_count / n;
            let target_pct = target_hit_count / n;
            let stop_pct = stop_hit_count / n;
            
            let mut returns: Vec<f64> = items.iter().map(|i| i["outcome_return"].as_f64().unwrap_or(0.0)).collect();
            let mut mfes: Vec<f64> = items.iter().map(|i| i["mfe"].as_f64().unwrap_or(0.0)).collect();
            let mut maes: Vec<f64> = items.iter().map(|i| i["mae"].as_f64().unwrap_or(0.0)).collect();
            let mut dds: Vec<f64> = items.iter().map(|i| i["drawdown"].as_f64().unwrap_or(0.0)).collect();
            
            returns.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            mfes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            maes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            dds.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            
            let mean_return = if n > 0.0 { returns.iter().sum::<f64>() / n } else { 0.0 };
            
            let median = |sorted: &Vec<f64>| -> f64 {
                let len = sorted.len();
                if len == 0 { return 0.0; }
                if len % 2 == 1 {
                    sorted[len / 2]
                } else {
                    (sorted[len / 2 - 1] + sorted[len / 2]) / 2.0
                }
            };
            
            aggregate_matrix.push(json!({
                "signature": sig,
                "signature_hash": sig_hash,
                "horizon": horizon,
                "N": items.len(),
                "entry_pct": entry_pct,
                "target_pct": target_pct,
                "stop_pct": stop_pct,
                "mean_return": mean_return,
                "median_return": median(&returns),
                "median_mfe": median(&mfes),
                "median_mae": median(&maes),
                "median_drawdown": median(&dds)
            }));
        }

        // 3. Population Accounting
        let mut population_accounting = Vec::new();
        for ((sig_hash, sig), items) in pop_map {
            let mut count_5d = 0;
            let mut count_10d = 0;
            let mut count_20d = 0;
            let mut count_60d = 0;
            
            let mut entry_5d = 0;
            let mut entry_10d = 0;
            let mut entry_20d = 0;
            let mut entry_60d = 0;
            
            for item in &items {
                let horizon = item["horizon"].as_str().unwrap_or("");
                let entry = item["entry_reached"].as_bool().unwrap_or(false);
                match horizon {
                    "5D" => { count_5d += 1; if entry { entry_5d += 1; } }
                    "10D" => { count_10d += 1; if entry { entry_10d += 1; } }
                    "20D" => { count_20d += 1; if entry { entry_20d += 1; } }
                    "60D" => { count_60d += 1; if entry { entry_60d += 1; } }
                    _ => {}
                }
            }
            
            population_accounting.push(json!({
                "signature": sig,
                "signature_hash": sig_hash,
                "5D_N": count_5d,
                "10D_N": count_10d,
                "20D_N": count_20d,
                "60D_N": count_60d,
                "5D_Entry": entry_5d,
                "10D_Entry": entry_10d,
                "20D_Entry": entry_20d,
                "60D_Entry": entry_60d
            }));
        }

        Ok(ExperimentMeasurements {
            metadata: json!({
                "status": "success",
                "experiment_type": "PredictiveValue"
            }),
            findings: vec![
                json!({
                    "table": "aggregate_matrix",
                    "data": aggregate_matrix
                }),
                json!({
                    "table": "raw_evidence_ledger",
                    "data": raw_ledger
                }),
                json!({
                    "table": "population_accounting",
                    "data": population_accounting
                }),
            ],
        })
    }
}
