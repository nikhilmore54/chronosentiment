use crate::data_source::CandleSource;
use crate::market_adapter::Candle;

pub struct LiveCandleSource;

impl CandleSource for LiveCandleSource {
    fn get_candles(&self) -> Vec<Candle> {
        unimplemented!("Live data integration not yet implemented")
    }
}
