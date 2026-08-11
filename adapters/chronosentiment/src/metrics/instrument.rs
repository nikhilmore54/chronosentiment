use coralys_moga::runtime::optimization::metric::{MetricEngine, MetricReport, MetricValue};
use crate::validation::context::InstrumentEvaluationContext;
use crate::metrics::concepts::{Concept, ConceptModel};

pub trait InstrumentMetricModel: ConceptModel {
    fn evaluate(&self, context: &InstrumentEvaluationContext, report: &mut MetricReport);
}

pub struct InstrumentMetricEngine {
    models: Vec<Box<dyn InstrumentMetricModel>>,
}

impl InstrumentMetricEngine {
    pub fn new() -> Self {
        Self {
            models: Vec::new(),
        }
    }

    pub fn add_model(&mut self, model: Box<dyn InstrumentMetricModel>) {
        self.models.push(model);
    }
}

impl MetricEngine<InstrumentEvaluationContext> for InstrumentMetricEngine {
    fn evaluate(&self, context: &InstrumentEvaluationContext) -> MetricReport {
        let mut report = MetricReport::default();
        for model in &self.models {
            model.evaluate(context, &mut report);
        }
        report
    }
}

pub struct SimpleMovingAverageMetric {
    window: usize,
}

impl SimpleMovingAverageMetric {
    pub fn new(window: usize) -> Self {
        Self { window }
    }
}

impl ConceptModel for SimpleMovingAverageMetric {
    fn concept(&self) -> Concept {
        Concept::Trend
    }

    fn name(&self) -> &str {
        Box::leak(format!("ma_{}", self.window).into_boxed_str())
    }
}

impl InstrumentMetricModel for SimpleMovingAverageMetric {
    fn evaluate(&self, context: &InstrumentEvaluationContext, report: &mut MetricReport) {
        let closes: Vec<f64> = context.observations.iter()
            .filter(|obs| obs.observation_type == "MarketPrice")
            .filter_map(|obs| obs.normalized_payload.get("close").and_then(|v| v.as_f64()))
            .collect();
            
        if closes.len() >= self.window {
            let sum: f64 = closes[closes.len() - self.window..].iter().sum();
            let ma = sum / self.window as f64;
            report.metrics.insert(self.name().to_string(), MetricValue::Float(ma));
        }
    }
}

pub struct RateOfChangeMetric {
    window: usize,
}

impl RateOfChangeMetric {
    pub fn new(window: usize) -> Self {
        Self { window }
    }
}

impl ConceptModel for RateOfChangeMetric {
    fn concept(&self) -> Concept {
        Concept::Momentum
    }

    fn name(&self) -> &str {
        Box::leak(format!("roc_{}", self.window).into_boxed_str())
    }
}

impl InstrumentMetricModel for RateOfChangeMetric {
    fn evaluate(&self, context: &InstrumentEvaluationContext, report: &mut MetricReport) {
        let closes: Vec<f64> = context.observations.iter()
            .filter(|obs| obs.observation_type == "MarketPrice")
            .filter_map(|obs| obs.normalized_payload.get("close").and_then(|v| v.as_f64()))
            .collect();
            
        if closes.len() >= self.window + 1 {
            let current = closes.last().unwrap();
            let previous = closes[closes.len() - self.window - 1];
            if previous > 0.0 {
                let roc = ((current - previous) / previous) * 100.0;
                report.metrics.insert(self.name().to_string(), MetricValue::Float(roc));
            }
        }
    }
}

pub struct AverageTrueRangeMetric {
    window: usize,
}

impl AverageTrueRangeMetric {
    pub fn new(window: usize) -> Self {
        Self { window }
    }
}

impl ConceptModel for AverageTrueRangeMetric {
    fn concept(&self) -> Concept {
        Concept::Volatility
    }

    fn name(&self) -> &str {
        Box::leak(format!("atr_{}", self.window).into_boxed_str())
    }
}

impl InstrumentMetricModel for AverageTrueRangeMetric {
    fn evaluate(&self, context: &InstrumentEvaluationContext, report: &mut MetricReport) {
        let obs_list: Vec<_> = context.observations.iter()
            .filter(|obs| obs.observation_type == "MarketPrice")
            .collect();
            
        if obs_list.len() >= self.window + 1 {
            let mut true_ranges = Vec::with_capacity(self.window);
            for i in (obs_list.len() - self.window)..obs_list.len() {
                let current = &obs_list[i];
                let previous = &obs_list[i - 1];
                
                let high = current.normalized_payload.get("high").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let low = current.normalized_payload.get("low").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let prev_close = previous.normalized_payload.get("close").and_then(|v| v.as_f64()).unwrap_or(0.0);
                
                let tr1 = high - low;
                let tr2 = (high - prev_close).abs();
                let tr3 = (low - prev_close).abs();
                
                let true_range = tr1.max(tr2).max(tr3);
                true_ranges.push(true_range);
            }
            let atr = true_ranges.iter().sum::<f64>() / self.window as f64;
            report.metrics.insert(self.name().to_string(), MetricValue::Float(atr));
        }
    }
}

pub struct VolumeAverageMetric {
    window: usize,
}

impl VolumeAverageMetric {
    pub fn new(window: usize) -> Self {
        Self { window }
    }
}

impl ConceptModel for VolumeAverageMetric {
    fn concept(&self) -> Concept {
        Concept::Liquidity
    }

    fn name(&self) -> &str {
        Box::leak(format!("volume_{}d", self.window).into_boxed_str())
    }
}

impl InstrumentMetricModel for VolumeAverageMetric {
    fn evaluate(&self, context: &InstrumentEvaluationContext, report: &mut MetricReport) {
        let volumes: Vec<f64> = context.observations.iter()
            .filter(|obs| obs.observation_type == "MarketPrice")
            .filter_map(|obs| obs.normalized_payload.get("volume").and_then(|v| v.as_f64()))
            .collect();
            
        if volumes.len() >= self.window {
            let sum: f64 = volumes[volumes.len() - self.window..].iter().sum();
            let avg = sum / self.window as f64;
            report.metrics.insert(self.name().to_string(), MetricValue::Float(avg));
        }
    }
}
