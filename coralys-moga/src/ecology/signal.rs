/// A scalar representing the optimization pressure for a resource.
/// > 0 implies under-utilized (should be used more)
/// < 0 implies over-utilized (should be used less)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EcologySignal {
    pub pressure: f64,
}

impl EcologySignal {
    pub fn new(pressure: f64) -> Self {
        Self { pressure }
    }
}
