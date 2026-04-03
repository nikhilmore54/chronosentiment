pub mod yahoo;
pub use yahoo::*;

pub mod python;
pub use python::PythonCandleSource;

#[async_trait::async_trait]
pub trait CandleSource: Send + Sync {
    /// Synchronous access for GA core / legacy loops.
    fn get_candles_sync(&self) -> Vec<crate::market_adapter::Candle>;

    /// Asynchronous access for live inference.
    async fn get_candles_async(&self) -> Vec<crate::market_adapter::Candle> {
        self.get_candles_sync()
    }
}
