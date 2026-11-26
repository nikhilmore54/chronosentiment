use crate::instrument::Instrument;
use async_trait::async_trait;
use std::error::Error;

/// Provides the universe of instruments to be evaluated by the Market Observatory.
#[async_trait]
pub trait UniverseProvider {
    /// Returns the name of this universe (e.g., "Nifty50", "MyPortfolio").
    fn name(&self) -> &str;

    /// Returns the list of instruments in this universe.
    async fn fetch_instruments(&self) -> Result<Vec<Instrument>, Box<dyn Error>>;
}

pub struct MockNifty50Provider;

#[async_trait]
impl UniverseProvider for MockNifty50Provider {
    fn name(&self) -> &str {
        "Nifty50"
    }

    async fn fetch_instruments(&self) -> Result<Vec<Instrument>, Box<dyn Error>> {
        let mut inst = Instrument::new("NSE".to_string(), "RELIANCE".to_string());
        inst.add_provider_id("yahoo", "RELIANCE.NS");

        let mut inst2 = Instrument::new("NSE".to_string(), "INFY".to_string());
        inst2.add_provider_id("yahoo", "INFY.NS");

        Ok(vec![inst, inst2])
    }
}
