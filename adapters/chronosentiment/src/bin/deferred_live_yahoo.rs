use std::time::Duration;

use chronosentiment_adapter::product::{
    load_intraday_briefs,
    AsOfSessionDriver,
    DeferredLiveConfig,
    ObservationProducer,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let date = std::env::args()
        .nth(1)
        .ok_or("usage: deferred_live_yahoo YYYY-MM-DD [dataset]")?;

    let dataset = std::env::args()
        .nth(2)
        .unwrap_or_else(|| format!("datasets/live005_{date}.json"));

    let briefs = load_intraday_briefs(&dataset)
        .map_err(|e| format!("load dataset {dataset}: {e}"))?;

    println!("[deferred-live-yahoo] date={date}");
    println!("[deferred-live-yahoo] dataset={dataset}");
    println!("[deferred-live-yahoo] briefs={}", briefs.len());

    let mut config = DeferredLiveConfig::default();

    // Rolling ASOF may evaluate later evidence, but a ticker that was not
    // ACT at T0 must not become an ACT admission later.
    config.strict_t0_admission = true;

    let mut driver = AsOfSessionDriver::new(config);

    let frozen_ids = driver.install_session(&briefs, &date);

    println!(
        "[deferred-live-yahoo] frozen session briefs={}",
        frozen_ids.len()
    );

    let mut producer = ObservationProducer::from_env();

    let tickers = driver.watched_tickers();

    if tickers.is_empty() {
        return Err("no watched tickers for session".into());
    }

    println!(
        "[deferred-live-yahoo] watched tickers={}",
        tickers.len()
    );

    println!(
        "[deferred-live-yahoo] source={:?}",
        producer.snapshot()
    );

    let mut last_event_count = 0usize;
    let mut last_position_count = 0usize;

    loop {
        let wait = producer.wait_millis_for_next();

        if wait > 0 {
            tokio::time::sleep(Duration::from_millis(wait)).await;
        }

        let batch = producer.poll_async(&tickers).await?;

        if batch.is_empty() {
            let snap = producer.snapshot();

            println!(
                "[deferred-live-yahoo] poll empty status={:?} last_ingest={:?}",
                snap.status,
                snap.last_ingest_unix
            );

            tokio::time::sleep(Duration::from_secs(5)).await;
            continue;
        }

        for sourced in batch {
            let ticker = sourced.observation.ticker.clone();
            let unix = sourced.observation.unix;

            let before_events = driver.asof_events().len();
            let before_positions = driver.ledger().positions.len();

            let surface = driver.ingest_sourced(sourced);

            let after_events = driver.asof_events().len();
            let after_positions = driver.ledger().positions.len();

            if after_events > before_events {
                for event in &driver.asof_events()[before_events..after_events] {
                    println!(
                        "[ASOF] {} unix={} offer={:?} action={} oqs={} h60={}",
                        event.ticker,
                        event.as_of_unix,
                        event.offer,
                        event.entry_action,
                        event.oqs,
                        event.h60_class
                    );
                }
            }

            if after_positions > before_positions {
                for position in &driver.ledger().positions[before_positions..after_positions] {
                    println!(
                        "[PAPER_ENTER] {} direction={} price={} decision={:?}",
                        position.paper.ticker,
                        position.paper.direction,
                        position.paper.paper_entry_price,
                        position.paper.decision_id
                    );
                }
            }

            for event in &surface.events {
                if event.kind != "HOLD" {
                    println!(
                        "[LIFECYCLE] {} {} @ {}",
                        event.kind,
                        ticker,
                        unix
                    );
                }
            }

            if after_events != last_event_count {
                last_event_count = after_events;
            }

            if after_positions != last_position_count {
                last_position_count = after_positions;
            }
        }
    }
}
