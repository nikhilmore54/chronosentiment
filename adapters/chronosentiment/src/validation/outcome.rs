use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum Horizon {
    Intraday,
    Swing,
    Position,
    Investment,
}

#[derive(Debug, Clone)]
pub struct OutcomeRecord {
    pub decision_id: Uuid,
    pub evaluation_timestamp: DateTime<Utc>,
    pub observation_end_timestamp: DateTime<Utc>,
    
    pub horizon: Horizon,
    pub holding_period_days: u32,
    pub exit_reason: String,
    
    pub outcome_return: f64,
    pub mfe: f64, // Maximum Favourable Excursion
    pub mae: f64, // Maximum Adverse Excursion
    
    pub maximum_drawdown: f64,
    pub realized_volatility: f64,
}

pub struct OutcomeEngine;

impl OutcomeEngine {
    pub fn measure_outcome(
        &self, 
        strategy: &crate::reasoning::strategy::OpportunityStrategy,
        future_observations: &[crate::observation::ValidatedObservation],
        evaluation_timestamp: DateTime<Utc>
    ) -> OutcomeRecord {
        let expected_days = strategy.expected_holding_period_days.1;
        let horizon_expiry = evaluation_timestamp + chrono::Duration::days(expected_days as i64);
        
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
        
        for obs in future_observations {
            // Temporal Firewall: Must be strictly after Decision(T)
            if obs.effective_from <= evaluation_timestamp {
                continue; 
            }
            if obs.effective_from > horizon_expiry {
                break;
            }
            
            let payload = &obs.normalized_payload;
            let close = payload.get("close").and_then(|v| v.as_f64()).unwrap_or(0.0);
            
            // Note: In real life we'd use high/low adjusted for splits. Here we use close for simplicity to avoid unadjusted H/L bugs,
            // or if we use high/low, we must adjust them. Let's use close for conservative daily hits.
            let high = payload.get("high").and_then(|v| v.as_f64()).unwrap_or(close);
            let low = payload.get("low").and_then(|v| v.as_f64()).unwrap_or(close);
            let unadj_close = payload.get("unadjusted_close").and_then(|v| v.as_f64()).unwrap_or(close);
            
            let adj_ratio = if unadj_close > 0.0 { close / unadj_close } else { 1.0 };
            let adj_high = high * adj_ratio;
            let adj_low = low * adj_ratio;
            
            if !entered {
                if adj_low <= strategy.entry_zone.max && adj_high >= strategy.entry_zone.min {
                    entered = true;
                    entry_price = close; // Approximation
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

        OutcomeRecord {
            decision_id: strategy.decision_id,
            evaluation_timestamp,
            observation_end_timestamp: horizon_expiry,
            horizon: match strategy.expected_horizon {
                crate::reasoning::strategy::Horizon::Intraday => Horizon::Intraday,
                crate::reasoning::strategy::Horizon::Swing => Horizon::Swing,
                crate::reasoning::strategy::Horizon::Position => Horizon::Position,
                crate::reasoning::strategy::Horizon::Strategic => Horizon::Investment,
                crate::reasoning::strategy::Horizon::Investment => Horizon::Investment,
            },
            holding_period_days: holding_days,
            exit_reason,
            outcome_return,
            mfe,
            mae,
            maximum_drawdown: mae.abs(),
            realized_volatility: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use crate::reasoning::strategy::{OpportunityStrategy, PriceRange};
    use crate::observation::ValidatedObservation;
    use serde_json::json;
    
    #[test]
    fn test_ambiguous_outcome() {
        let engine = OutcomeEngine;
        let eval_time = Utc.timestamp_opt(1600000000, 0).unwrap();
        
        let strategy = OpportunityStrategy {
            decision_id: Uuid::new_v4(),
            expected_horizon: crate::reasoning::strategy::Horizon::Swing,
            expected_holding_period_days: (10, 20),
            entry_zone: PriceRange { min: 99.0, max: 101.0 },
            target_zone: PriceRange { min: 110.0, max: 120.0 },
            stop_loss_zone: PriceRange { min: 80.0, max: 90.0 },
            expected_return: 0.1,
            expected_drawdown: 0.1,
            expected_volatility: 0.05,
            risk_reward_ratio: 1.0,
            confidence: 0.5,
        };
        
        // Day 1: Entry
        let mut obs1 = ValidatedObservation {
            id: Uuid::new_v4(),
            research_session_id: None,
            instrument_id: None,
            observation_type: "MarketPrice".to_string(),
            source: "Test".to_string(),
            source_identifier: None,
            observed_at: eval_time + chrono::Duration::days(1),
            effective_from: eval_time + chrono::Duration::days(1),
            effective_to: None,
            recorded_at: eval_time,
            raw_payload: json!({}),
            normalized_payload: json!({
                "open": 100.0,
                "high": 100.0,
                "low": 100.0,
                "close": 100.0,
            }),
            confidence: 1.0,
            freshness: 0.0,
            coverage: "".to_string(),
            consistency: None,
            quality_score: 1.0,
            provenance_hash: "".to_string(),
            schema_version: 1,
        };
        
        // Day 2: Ambiguous (hits both target and stop)
        let mut obs2 = obs1.clone();
        obs2.effective_from = eval_time + chrono::Duration::days(2);
        obs2.normalized_payload = json!({
            "open": 100.0,
            "high": 115.0, // Hits target (> 110)
            "low": 85.0,   // Hits stop (< 90)
            "close": 100.0,
        });
        
        let outcome = engine.measure_outcome(&strategy, &[obs1, obs2], eval_time);
        
        assert_eq!(outcome.exit_reason, "Ambiguous");
    }
}
