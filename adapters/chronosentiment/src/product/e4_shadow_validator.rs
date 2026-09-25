use std::fs::OpenOptions;
use std::io::Write;
use serde::{Deserialize, Serialize};
use super::deferred_live::{LivePaperLedger, MarketObservation, LivePaperEvent};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E4ShadowPosition {
    pub position_id: String,
    pub session_date: String,
    pub ticker: String,
    pub direction: String,
    pub entry_timestamp: i64,
    pub entry_price: f64,
    pub original_target: f64,
    pub original_stop: f64,
    pub horizon_timestamp: i64,

    pub triggered: bool,
    pub trigger_timestamp: Option<i64>,
    pub trigger_price: Option<f64>,
    pub shadow_stop: Option<f64>,
    #[serde(skip)]
    pub shadow_stop_active_next: bool,

    pub baseline_exit_timestamp: Option<i64>,
    pub baseline_exit_price: Option<f64>,
    pub baseline_exit_reason: Option<String>,
    pub baseline_realized: Option<f64>,

    pub shadow_exit_timestamp: Option<i64>,
    pub shadow_exit_price: Option<f64>,
    pub shadow_exit_reason: Option<String>,
    pub shadow_realized: Option<f64>,

    pub imv: Option<f64>,

    pub baseline_mfe: f64,
    pub baseline_mae: f64,
    pub shadow_mfe: f64,
    pub shadow_mae: f64,

    pub baseline_duration_bars: i64,
    pub shadow_duration_bars: i64,

    pub round_trip_baseline: i64,
    pub round_trip_shadow: i64,

    pub data_integrity: String,
    #[serde(skip)]
    pub prod_exited: bool,
    #[serde(skip)]
    pub prod_exit_price: Option<f64>,
    pub prod_exit_timestamp: Option<i64>,
    #[serde(skip)]
    pub prod_exit_reason: Option<String>,
    #[serde(skip)]
    pub prod_realized: Option<f64>,
}

pub struct E4ShadowValidator {
    pub active_positions: Vec<E4ShadowPosition>,
}

impl E4ShadowValidator {
    pub fn new() -> Self {
        Self {
            active_positions: Vec::new(),
        }
    }

    pub fn step(&mut self, obs: &MarketObservation, events: &[LivePaperEvent], ledger: &LivePaperLedger) {
        // 1. Process new opens from production ledger
        for ev in events {
            if ev.kind == "PAPER_ENTER" {
                if let Some(pos) = ledger.positions.iter().find(|p| p.paper.ticker == obs.ticker && p.opened_at == ev.unix) {
                    // LONG-only fail-closed handling
                    if pos.paper.direction != "LONG" {
                        continue;
                    }
                    let h300 = match pos.horizon {
                        super::paper_lifecycle::HorizonPolicy::Unix { horizon_unix } => horizon_unix,
                        _ => ev.unix + 300 * 60,
                    };
                    // Ensure we don't duplicate
                    if !self.active_positions.iter().any(|p| p.position_id == pos.paper.decision_id.clone().unwrap_or_default()) {
                        self.active_positions.push(E4ShadowPosition {
                            position_id: pos.paper.decision_id.clone().unwrap_or_default(),
                            session_date: pos.paper.date.clone(),
                            ticker: pos.paper.ticker.clone(),
                            direction: pos.paper.direction.clone(),
                            entry_timestamp: pos.opened_at,
                            entry_price: pos.paper_entry_price(),
                            original_target: pos.paper.paper_target,
                            original_stop: pos.paper.candidate_stop_price.unwrap_or(pos.paper.paper_risk),
                            horizon_timestamp: h300,
                            
                            triggered: false,
                            trigger_timestamp: None,
                            trigger_price: None,
                            shadow_stop: None,
                            shadow_stop_active_next: false,
                            
                            baseline_exit_timestamp: None,
                            baseline_exit_price: None,
                            baseline_exit_reason: None,
                            baseline_realized: None,
                            
                            shadow_exit_timestamp: None,
                            shadow_exit_price: None,
                            shadow_exit_reason: None,
                            shadow_realized: None,
                            
                            imv: None,
                            
                            baseline_mfe: 0.0,
                            baseline_mae: 0.0,
                            shadow_mfe: 0.0,
                            shadow_mae: 0.0,
                            
                            baseline_duration_bars: 0,
                            shadow_duration_bars: 0,
                            
                            round_trip_baseline: 0,
                            round_trip_shadow: 0,
                            
                            data_integrity: "ACTIVE".to_string(),
                            prod_exited: false,
                            prod_exit_price: None,
                            prod_exit_timestamp: None,
                            prod_exit_reason: None,
                            prod_realized: None,
                        });
                    }
                }
            }
        }

        // 2. Poll production exits precisely by decision_id
        for p in &mut self.active_positions {
            if p.ticker == obs.ticker && p.data_integrity == "ACTIVE" && !p.prod_exited {
                if let Some(prod_pos) = ledger.positions.iter().find(|prod| prod.paper.decision_id.as_deref() == Some(p.position_id.as_str())) {
                    if prod_pos.status() != super::deferred_live::LivePaperStatus::Open {
                        p.prod_exited = true;
                        p.prod_exit_price = prod_pos.paper_exit_price();
                        p.prod_exit_reason = prod_pos.exit_reason().map(|s| s.to_string());
                        p.prod_realized = prod_pos.realized_return();
                        // Get timestamp from the last event or walk exit
                        if let Some(exit_ev) = prod_pos.walk.exit.as_ref() {
                            p.prod_exit_timestamp = Some(exit_ev.unix);
                        } else {
                            p.prod_exit_timestamp = Some(prod_pos.last_tick_at);
                        }
                    }
                }
            }
        }

        // 3. Tick observations
        for p in &mut self.active_positions {
            if p.ticker != obs.ticker || p.data_integrity != "ACTIVE" {
                continue;
            }

            let entry = p.entry_price;
            let current_price = obs.price;
            let high = obs.high.unwrap_or(current_price);
            let low = obs.low.unwrap_or(current_price);

            let ret_high = (high - entry) / entry;
            let ret_low = (low - entry) / entry;
            let ret_close = (current_price - entry) / entry;

            // --- SHADOW PATH ---
            if p.shadow_exit_timestamp.is_none() {
                p.shadow_duration_bars += 1;
                
                if ret_high > p.shadow_mfe { p.shadow_mfe = ret_high; }
                if ret_low < p.shadow_mae { p.shadow_mae = ret_low; }

                if obs.unix >= p.horizon_timestamp {
                    p.shadow_exit_timestamp = Some(obs.unix);
                    p.shadow_exit_reason = Some("HORIZON".to_string());
                    p.shadow_exit_price = Some(current_price);
                    p.shadow_realized = Some(ret_close);
                } else if p.shadow_stop_active_next && low <= p.shadow_stop.unwrap() {
                    let exec_price = p.shadow_stop.unwrap();
                    p.shadow_exit_timestamp = Some(obs.unix);
                    p.shadow_exit_reason = Some("E4-PP-050".to_string());
                    p.shadow_exit_price = Some(exec_price);
                    p.shadow_realized = Some((exec_price - entry) / entry);
                } else if high >= p.original_target {
                    p.shadow_exit_timestamp = Some(obs.unix);
                    p.shadow_exit_reason = Some("TARGET".to_string());
                    p.shadow_exit_price = Some(p.original_target);
                    p.shadow_realized = Some((p.original_target - entry) / entry);
                } else if low <= p.original_stop {
                    p.shadow_exit_timestamp = Some(obs.unix);
                    p.shadow_exit_reason = Some("STOP".to_string());
                    p.shadow_exit_price = Some(p.original_stop);
                    p.shadow_realized = Some((p.original_stop - entry) / entry);
                }

                // Check trigger for next bar (Close-only 0.50% trigger, records actual price)
                if p.shadow_exit_timestamp.is_none() && !p.shadow_stop_active_next && ret_close >= 0.005 {
                    p.shadow_stop_active_next = true;
                    p.triggered = true;
                    p.trigger_timestamp = Some(obs.unix);
                    p.trigger_price = Some(current_price);
                    p.shadow_stop = Some(entry * 1.0005);
                }
            }

            // --- BASELINE PATH ---
            if p.baseline_exit_timestamp.is_none() {
                p.baseline_duration_bars += 1;
                
                if ret_high > p.baseline_mfe { p.baseline_mfe = ret_high; }
                if ret_low < p.baseline_mae { p.baseline_mae = ret_low; }

                if obs.unix >= p.horizon_timestamp {
                    p.baseline_exit_timestamp = Some(obs.unix);
                    p.baseline_exit_reason = Some("HORIZON".to_string());
                    p.baseline_exit_price = Some(current_price);
                    p.baseline_realized = Some(ret_close);
                } else if high >= p.original_target {
                    p.baseline_exit_timestamp = Some(obs.unix);
                    p.baseline_exit_reason = Some("TARGET".to_string());
                    p.baseline_exit_price = Some(p.original_target);
                    p.baseline_realized = Some((p.original_target - entry) / entry);
                } else if low <= p.original_stop {
                    p.baseline_exit_timestamp = Some(obs.unix);
                    p.baseline_exit_reason = Some("STOP".to_string());
                    p.baseline_exit_price = Some(p.original_stop);
                    p.baseline_realized = Some((p.original_stop - entry) / entry);
                }
            }

            // --- RESOLUTION ---
            if p.baseline_exit_timestamp.is_some() && p.shadow_exit_timestamp.is_some() && p.prod_exited {
                p.imv = Some(p.shadow_realized.unwrap_or(0.0) - p.baseline_realized.unwrap_or(0.0));
                p.round_trip_baseline = if p.baseline_mfe >= 0.005 && p.baseline_realized.unwrap_or(0.0) < 0.0 { 1 } else { 0 };
                p.round_trip_shadow = if p.shadow_mfe >= 0.005 && p.shadow_realized.unwrap_or(0.0) < 0.0 { 1 } else { 0 };
                
                // Reconciliation Check (Timestamp, Reason, Price, Return)
                let mut recon_failure = false;
                
                if p.baseline_exit_timestamp != p.prod_exit_timestamp { recon_failure = true; }
                if p.baseline_exit_reason != p.prod_exit_reason { recon_failure = true; }
                
                let rel_price_diff = (p.baseline_exit_price.unwrap_or(0.0) - p.prod_exit_price.unwrap_or(0.0)).abs() / p.prod_exit_price.unwrap_or(1.0).max(1e-6);
                if rel_price_diff > 0.0001 { recon_failure = true; }
                
                let abs_ret_diff = (p.baseline_realized.unwrap_or(0.0) - p.prod_realized.unwrap_or(0.0)).abs();
                if abs_ret_diff > 0.0001 { recon_failure = true; }

                if recon_failure {
                    p.data_integrity = "RECONCILIATION_FAILURE".to_string();
                } else {
                    p.data_integrity = "COMPLETE".to_string();
                }
                
                Self::flush_position(p.clone());
            }
        }

        self.active_positions.retain(|p| p.data_integrity == "ACTIVE");
    }

    fn flush_position(pos: E4ShadowPosition) {
        let date_str = pos.session_date.replace("-", "");
        // Append to the requested ledger file
        let file_path = format!("live_capture/ledger/shadow_ledger_e4_050_{}.jsonl", date_str);
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(file_path) {
            if let Ok(json) = serde_json::to_string(&pos) {
                let _ = writeln!(file, "{}", json);
            }
        }
    }
}

#[cfg(test)]
#[path = "e4_shadow_validator_tests.rs"]
mod e4_shadow_validator_tests;
