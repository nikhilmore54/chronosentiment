use crate::instrument::Instrument;
use crate::observation::RawObservation;
use async_trait::async_trait;
use std::error::Error;

pub enum TimeRange {
    OneMonth,
    ThreeMonths,
    OneYear,
    FiveYears,
}

#[async_trait]
pub trait MarketDataProvider {
    type RawRecord;

    async fn fetch_historical(
        &self,
        instrument: &Instrument,
        range: TimeRange,
    ) -> Result<Vec<Self::RawRecord>, Box<dyn Error>>;
}

pub trait ValidatedObservationTranslator<R> {
    fn translate(&self, raw: R, instrument: &Instrument) -> RawObservation;
}
