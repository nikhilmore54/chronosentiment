pub trait CandleSource {
    fn get_candles(&self) -> Vec<crate::market_adapter::Candle>;
}