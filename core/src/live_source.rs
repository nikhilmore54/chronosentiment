use crate::data_source::CandleSource;
use crate::market_adapter::Candle;
use async_trait::async_trait;

pub struct LiveCandleSource;

#[async_trait]
impl CandleSource for LiveCandleSource {
    fn get_candles_sync(&self) -> Vec<Candle> {
        Vec::new()
    }

    async fn get_candles_async(&self) -> Vec<Candle> {
        Vec::new()
    }
}
