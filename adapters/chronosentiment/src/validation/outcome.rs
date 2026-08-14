use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use crate::repository::knowledge::{ArtifactMetadata, ArtifactType, KnowledgeArtifact, ArtifactLineage};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeRecord {
    pub metadata: ArtifactMetadata,
    pub instrument_id: Option<Uuid>,
    pub decision_id: Uuid,
    pub strategy_id: Uuid,
    
    pub evaluation_timestamp: DateTime<Utc>,
    pub horizon: String,
    pub horizon_expiry_timestamp: DateTime<Utc>,
    pub observation_end_timestamp: DateTime<Utc>,
    
    pub entry_reached: bool,
    pub target_hit: bool,
    pub stop_hit: bool,
    
    pub holding_period_days: u32,
    pub exit_reason: String,
    
    pub outcome_return: f64,
    pub mfe: f64, // Maximum Favourable Excursion
    pub mae: f64, // Maximum Adverse Excursion
    
    pub maximum_drawdown: f64,
    pub realized_volatility: f64,
}

impl KnowledgeArtifact for OutcomeRecord {
    fn metadata(&self) -> &ArtifactMetadata {
        &self.metadata
    }
    fn instrument_id(&self) -> Option<Uuid> {
        self.instrument_id
    }
}

pub struct OutcomeEngine;

impl OutcomeEngine {
    pub fn measure_outcome(
        &self, 
        decision_id: Uuid,
        strategy: &crate::reasoning::strategy::OpportunityStrategy,
        strategy_metadata: &ArtifactMetadata,
        future_observations: &[crate::observation::ValidatedObservation],
        evaluation_timestamp: DateTime<Utc>,
        measurement_horizon_days: u32,
        instrument_id: Option<Uuid>,
    ) -> OutcomeRecord {
        let horizon_str = format!("{}D", measurement_horizon_days);
        let horizon_expiry = evaluation_timestamp + chrono::Duration::days(measurement_horizon_days as i64);
        
        let mut entered = false;
        let mut target_hit = false;
        let mut stop_hit = false;
        let mut ambiguous = false;
        
        let mut mfe: f64 = 0.0;
        let mut mae: f64 = 0.0;
        
        let mut exit_reason = "Expired".to_string();
        let mut final_price = 0.0;
        let mut entry_price = 0.0;
        let mut holding_days = 0;
        let mut obs_end = evaluation_timestamp;
        
        for obs in future_observations {
            // Temporal Firewall: strictly after Decision(T)
            if obs.effective_from <= evaluation_timestamp {
                continue; 
            }
            if obs.effective_from > horizon_expiry {
                break;
            }
            
            obs_end = obs.effective_from;
            
            let payload = &obs.normalized_payload;
            let close = payload.get("close").and_then(|v| v.as_f64()).unwrap_or(0.0);
            
            let high = payload.get("high").and_then(|v| v.as_f64()).unwrap_or(close);
            let low = payload.get("low").and_then(|v| v.as_f64()).unwrap_or(close);
            let unadj_close = payload.get("unadjusted_close").and_then(|v| v.as_f64()).unwrap_or(close);
            
            let adj_ratio = if unadj_close > 0.0 { close / unadj_close } else { 1.0 };
            let adj_high = high * adj_ratio;
            let adj_low = low * adj_ratio;
            
            if !entered {
                if adj_low <= strategy.entry_zone.max && adj_high >= strategy.entry_zone.min {
                    entered = true;
                    entry_price = close;
                }
            } else {
                holding_days += 1;
                
                let cur_mfe = (adj_high - entry_price) / entry_price;
                let cur_mae = (adj_low - entry_price) / entry_price;
                
                if cur_mfe > mfe { mfe = cur_mfe; }
                if cur_mae < mae { mae = cur_mae; }
                
                let hit_target = adj_high >= strategy.target_zone.min;
                let hit_stop = adj_low <= strategy.stop_loss_zone.max;
                
                if hit_target && hit_stop {
                    ambiguous = true;
                    exit_reason = "Ambiguous".to_string();
                    target_hit = true;
                    stop_hit = true;
                    final_price = close;
                    break;
                } else if hit_target {
                    target_hit = true;
                    exit_reason = "Target Hit".to_string();
                    final_price = strategy.target_zone.min;
                    break;
                } else if hit_stop {
                    stop_hit = true;
                    exit_reason = "Stop Hit".to_string();
                    final_price = strategy.stop_loss_zone.max;
                    break;
                }
            }
            final_price = close;
        }
        
        let outcome_return = if entered {
            (final_price - entry_price) / entry_price
        } else {
            0.0
        };
        
        if !entered {
            exit_reason = "Entry Not Reached".to_string();
        }

        let mut record = OutcomeRecord {
            metadata: ArtifactMetadata {
                artifact_id: Uuid::new_v4(),
                artifact_schema_version: "1.0.0".to_string(),
                artifact_type: ArtifactType::Outcome,
                created_at: Utc::now(),
                evaluation_timestamp,
                engine_versions: strategy_metadata.engine_versions.clone(),
                lineage: ArtifactLineage {
                    produced_by: "OutcomeEngine:v1".to_string(),
                    consumed_artifacts: vec![decision_id, strategy_metadata.artifact_id],
                    parent_artifacts: vec![decision_id, strategy_metadata.artifact_id],
                },
                replay_context_hash: strategy_metadata.replay_context_hash.clone(),
                knowledge_lake_version: strategy_metadata.knowledge_lake_version.clone(),
                content_hash: "".to_string(),
            },
            instrument_id,
            decision_id,
            strategy_id: strategy_metadata.artifact_id,
            evaluation_timestamp,
            horizon: horizon_str.clone(),
            horizon_expiry_timestamp: horizon_expiry,
            observation_end_timestamp: obs_end,
            entry_reached: entered,
            target_hit,
            stop_hit,
            holding_period_days: holding_days,
            exit_reason,
            outcome_return,
            mfe,
            mae,
            maximum_drawdown: mae.abs(),
            realized_volatility: 0.0,
        };
        
        let mut hasher = Sha256::new();
        hasher.update(record.strategy_id.as_bytes());
        hasher.update(record.horizon.as_bytes());
        hasher.update(record.outcome_return.to_be_bytes());
        hasher.update(record.exit_reason.as_bytes());
        hasher.update(record.mfe.to_be_bytes());
        hasher.update(record.mae.to_be_bytes());
        hasher.update(record.holding_period_days.to_be_bytes());
        
        record.metadata.content_hash = format!("{:x}", hasher.finalize());
        
        record
    }
}
