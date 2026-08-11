use coralys_moga::runtime::optimization::metric::{MetricEngine, MetricReport, MetricValue};
use crate::validation::context::MarketEvaluationContext;

pub trait MarketMetricModel {
    fn name(&self) -> &str;
    fn evaluate(&self, context: &MarketEvaluationContext, report: &mut MetricReport);
}

pub struct MarketMetricEngine {
    models: Vec<Box<dyn MarketMetricModel>>,
}

impl MarketMetricEngine {
    pub fn new() -> Self {
        Self {
            models: Vec::new(),
        }
    }

    pub fn add_model(&mut self, model: Box<dyn MarketMetricModel>) {
        self.models.push(model);
    }
}

impl MetricEngine<MarketEvaluationContext> for MarketMetricEngine {
    fn evaluate(&self, context: &MarketEvaluationContext) -> MetricReport {
        let mut report = MetricReport::default();
        for model in &self.models {
            model.evaluate(context, &mut report);
        }
        report
    }
}

// Example Market Metric: Advance/Decline (mocked for now until we have breadth data)
pub struct AdvanceDeclineMetric;
impl MarketMetricModel for AdvanceDeclineMetric {
    fn name(&self) -> &str {
        "advance_decline"
    }

    fn evaluate(&self, _context: &MarketEvaluationContext, report: &mut MetricReport) {
        // Mocked logic for breadth
        report.metrics.insert(self.name().to_string(), MetricValue::Float(1.2));
    }
}
