/// Configuration governing the strength of the ecology guidance.
#[derive(Clone, Debug)]
pub struct EcologyPolicy {
    pub alpha: f64,
}

impl EcologyPolicy {
    pub fn new(alpha: f64) -> Self {
        Self { alpha }
    }
}
