use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Concept {
    Trend,
    Momentum,
    Volatility,
    Liquidity,
    Valuation,
    Quality,
    Macro,
    Sector,
}

pub trait ConceptModel {
    fn concept(&self) -> Concept;
    fn name(&self) -> &str;
}
