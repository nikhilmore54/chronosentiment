use uuid::Uuid;
use crate::reasoning::decision::{Decision, Opportunity};

#[derive(Debug, Clone, PartialEq)]
pub enum Horizon {
    Intraday,
    Swing,
    Position,
    Investment,
    Strategic,
}

#[derive(Debug, Clone)]
pub struct PriceRange {
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone)]
pub struct OpportunityStrategy {
    pub decision_id: Uuid,
    pub expected_horizon: Horizon,
    pub expected_holding_period_days: (u32, u32),
    
    pub entry_zone: PriceRange,
    pub target_zone: PriceRange,
    pub stop_loss_zone: PriceRange,
    
    pub expected_return: f64,
    pub expected_drawdown: f64,
    pub expected_volatility: f64,
    pub risk_reward_ratio: f64,
    pub confidence: f64,
}

pub struct StrategyEngine;

impl StrategyEngine {
    pub fn generate(&self, decision: &Decision, current_close: f64, atr: f64) -> Option<OpportunityStrategy> {
        if decision.opportunity != Opportunity::Positive {
            return None;
        }
        
        // Baseline Strategy Policy v1.0
        // Target = +2 ATR, Stop = -1 ATR
        let entry_min = current_close - (atr * 0.1);
        let entry_max = current_close + (atr * 0.1);
        let target_min = current_close + (atr * 1.9);
        let target_max = current_close + (atr * 2.1);
        let stop_min = current_close - (atr * 1.1);
        let stop_max = current_close - (atr * 0.9);
        
        Some(OpportunityStrategy {
            decision_id: decision.decision_id,
            expected_horizon: Horizon::Swing,
            expected_holding_period_days: (10, 20),
            entry_zone: PriceRange { min: entry_min, max: entry_max },
            target_zone: PriceRange { min: target_min, max: target_max },
            stop_loss_zone: PriceRange { min: stop_min, max: stop_max },
            expected_return: 2.0 * atr / current_close,
            expected_drawdown: 1.0 * atr / current_close,
            expected_volatility: atr / current_close,
            risk_reward_ratio: 2.0,
            confidence: 0.5,
        })
    }
}
