use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum MetricValue {
    Float(f64),
    Integer(i64),
    Boolean(bool),
    String(String),
}

impl fmt::Display for MetricValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetricValue::Float(v) => write!(f, "{}", v),
            MetricValue::Integer(v) => write!(f, "{}", v),
            MetricValue::Boolean(v) => write!(f, "{}", v),
            MetricValue::String(v) => write!(f, "{}", v),
        }
    }
}

pub trait MetricModel<M> {
    fn name(&self) -> String;
    fn compute(&self, model: &M) -> MetricValue;
}

#[derive(Debug, Clone, Default)]
pub struct MetricReport {
    pub metrics: std::collections::HashMap<String, MetricValue>,
}

impl MetricReport {
    pub fn get_float(&self, key: &str) -> Option<f64> {
        match self.metrics.get(key) {
            Some(MetricValue::Float(v)) => Some(*v),
            _ => None,
        }
    }
}

pub trait MetricEngine<M> {
    fn evaluate(&self, model: &M) -> MetricReport;
}
