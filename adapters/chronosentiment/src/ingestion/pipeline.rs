use crate::ingestion::provider::{MarketDataProvider, TimeRange, ValidatedObservationTranslator};
use crate::ingestion::validator::ValidationEngine;
use crate::instrument::Instrument;
use crate::repository::observation_repository::ValidatedObservationRepository;
use async_trait::async_trait;
use std::error::Error;
use std::time::Instant;

pub struct IngestionRequest {
    pub range: TimeRange,
}

pub struct IngestionReport {
    pub observations_fetched: usize,
    pub observations_translated: usize,
    pub observations_accepted: usize,
    pub duplicates_rejected: usize,
    pub validation_failures: usize,
    pub persistence_count: usize,
    pub elapsed_time_ms: u128,
}

#[async_trait]
pub trait IngestionPipeline {
    async fn ingest(
        &self,
        instrument: &Instrument,
        request: IngestionRequest,
    ) -> Result<IngestionReport, Box<dyn Error>>;
}

pub struct DefaultIngestionPipeline<P, T, R> {
    pub provider: P,
    pub translator: T,
    pub validator: ValidationEngine,
    pub repository: R,
}

#[async_trait]
impl<P, T, R> IngestionPipeline for DefaultIngestionPipeline<P, T, R>
where
    P: MarketDataProvider + Send + Sync,
    P::RawRecord: Send + Sync,
    T: ValidatedObservationTranslator<P::RawRecord> + Send + Sync,
    R: ValidatedObservationRepository + Send + Sync,
{
    async fn ingest(
        &self,
        instrument: &Instrument,
        request: IngestionRequest,
    ) -> Result<IngestionReport, Box<dyn Error>> {
        let start = Instant::now();

        let mut report = IngestionReport {
            observations_fetched: 0,
            observations_translated: 0,
            observations_accepted: 0,
            duplicates_rejected: 0,
            validation_failures: 0,
            persistence_count: 0,
            elapsed_time_ms: 0,
        };

        // 1. Fetch DTOs (Infrastructure)
        let raw_records = self
            .provider
            .fetch_historical(instrument, request.range)
            .await?;
        report.observations_fetched = raw_records.len();

        let mut validated = Vec::new();

        for record in raw_records {
            // 2. Translate (Canonicalization)
            let raw_obs = self.translator.translate(record, instrument);
            report.observations_translated += 1;

            // 3. Validate (Domain)
            // In a real system, validate() might return Result<ValidatedObservation, Error>
            let valid_obs = self.validator.validate(raw_obs, instrument);
            report.observations_accepted += 1;

            validated.push(valid_obs);
        }

        // 4. Persist (Repository)
        // Ignoring duplicates/failures logic here for mock brevity, but the architecture supports it
        for obs in &validated {
            self.repository.store_observation(obs).await?;
        }
        report.persistence_count = validated.len();

        report.elapsed_time_ms = start.elapsed().as_millis();

        Ok(report)
    }
}
