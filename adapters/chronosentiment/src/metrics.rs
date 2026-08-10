use coralys_moga::runtime::optimization::metric::{MetricEngine, MetricReport, MetricValue};
use crate::validation::context::EvaluationContext;

pub struct ChronoMetricEngine;

impl MetricEngine<EvaluationContext> for ChronoMetricEngine {
    fn evaluate(&self, context: &EvaluationContext) -> MetricReport {
        let mut report = MetricReport::default();
        
        let closes: Vec<f64> = context.observations.iter()
            .filter(|obs| obs.observation_type == "MarketPrice")
            .filter_map(|obs| obs.normalized_payload.get("close").and_then(|v| v.as_f64()))
            .collect();

        let volumes: Vec<f64> = context.observations.iter()
            .filter(|obs| obs.observation_type == "MarketPrice")
            .filter_map(|obs| obs.normalized_payload.get("volume").and_then(|v| v.as_f64()))
            .collect();

        if closes.is_empty() {
            return report;
        }

        // 1. Daily Return (latest close / previous close - 1)
        if closes.len() >= 2 {
            let last = closes[closes.len() - 1];
            let prev = closes[closes.len() - 2];
            report.metrics.insert("daily_return".to_string(), MetricValue::Float((last / prev) - 1.0));
        }

        // 2. Rolling Volatility (20-day standard deviation of returns)
        if closes.len() >= 21 {
            let mut returns = Vec::with_capacity(20);
            for i in closes.len() - 20..closes.len() {
                returns.push((closes[i] / closes[i - 1]) - 1.0);
            }
            let mean = returns.iter().sum::<f64>() / returns.len() as f64;
            let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / returns.len() as f64;
            let volatility = variance.sqrt() * (252_f64).sqrt(); // Annualized
            report.metrics.insert("volatility_20d".to_string(), MetricValue::Float(volatility));
        }

        // 3. Moving Averages (20 / 50 / 200)
        let compute_ma = |window: usize| -> Option<f64> {
            if closes.len() >= window {
                let sum: f64 = closes[closes.len() - window..].iter().sum();
                Some(sum / window as f64)
            } else {
                None
            }
        };

        if let Some(ma20) = compute_ma(20) {
            report.metrics.insert("ma_20".to_string(), MetricValue::Float(ma20));
        }
        if let Some(ma50) = compute_ma(50) {
            report.metrics.insert("ma_50".to_string(), MetricValue::Float(ma50));
        }
        if let Some(ma200) = compute_ma(200) {
            report.metrics.insert("ma_200".to_string(), MetricValue::Float(ma200));
        }

        // 4. Volume Averages (20-day)
        if volumes.len() >= 20 {
            let sum: f64 = volumes[volumes.len() - 20..].iter().sum();
            report.metrics.insert("volume_avg_20d".to_string(), MetricValue::Float(sum / 20.0));
        }

        // 5. Drawdown
        let max_peak = closes.iter().fold(0.0f64, |acc, &val| acc.max(val));
        let current = closes.last().unwrap();
        let drawdown = if max_peak > 0.0 { (current - max_peak) / max_peak } else { 0.0 };
        report.metrics.insert("drawdown".to_string(), MetricValue::Float(drawdown));

        report
    }
}
