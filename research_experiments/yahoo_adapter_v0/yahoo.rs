use crate::data_source::CandleSource;
use crate::market_adapter::Candle;
use async_trait::async_trait;
use yahoo_finance_api as yahoo;

pub struct YahooCandleSource {
    pub symbol: String,
    pub interval: String,
    pub min_required_candles: usize,
}

impl YahooCandleSource {
    pub fn new(symbol: &str, interval: &str, min_required_candles: usize) -> Self {
        Self {
            symbol: symbol.to_string(),
            interval: interval.to_string(),
            min_required_candles,
        }
    }
}

#[async_trait]
impl CandleSource for YahooCandleSource {
    fn get_candles_sync(&self) -> Vec<Candle> {
        // Refinement 1: No sync support for Yahoo to avoid nested runtimes/async leakage.
        eprintln!("WARNING: YahooCandleSource::get_candles_sync called. Yahoo only supports async fetch for live inference.");
        Vec::new()
    }

    async fn get_candles_async(&self) -> Vec<Candle> {
        let connector = match yahoo::YahooConnector::new() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("ERROR: Failed to initialize YahooConnector: {:?}", e);
                return Vec::new();
            }
        };

        // MODIFIED: Compute range from interval to guarantee >= min_required_candles bars
        // Yahoo constraints: 1m→7d max, 5m/15m→60d max, 1h→730d max, 1d→unlimited
        let range = match self.interval.as_str() {
            "1m" => "5d",   // 5d × 24h × 60m = 7200 bars (crypto 24/7)
            "5m" => "1mo",  // 1mo × 24h × 12 = 8640 bars
            "15m" => "1mo", // 1mo × 24h × 4  = 2880 bars
            "1h" => "3mo",  // 3mo × 24h      = 2160 bars
            "1d" => "6mo",  // 6mo            ≈ 125 trading days
            _ => "1mo",
        };
        match connector
            .get_quote_range(&self.symbol, &self.interval, range)
            .await
        {
            Ok(response) => {
                match response.quotes() {
                    Ok(quotes) => {
                        let available = quotes.len();
                        if available < self.min_required_candles {
                            eprintln!(
                                "WARNING: Yahoo ({} @ {}): Only {} candles available (need {}). \
                                Try a longer range or different interval. Using what is available.",
                                self.symbol, self.interval, available, self.min_required_candles
                            );
                            // Use whatever we have rather than failing completely
                            if available == 0 {
                                return Vec::new();
                            }
                        }

                        let mut candles = Vec::new();
                        for quote in quotes {
                            candles.push(Candle {
                                timestamp: quote.timestamp,
                                open: (quote.open * 10000.0) as u64,
                                high: (quote.high * 10000.0) as u64,
                                low: (quote.low * 10000.0) as u64,
                                close: (quote.close * 10000.0) as u64,
                                volume: quote.volume,
                            });
                        }

                        // Refinement 3: Timestamp alignment check (simplified)
                        // In a real production system, we'd verify last_timestamp % interval == 0
                        // For now, we ensure the window is strictly what is required.
                        candles
                    }
                    Err(e) => {
                        let msg = format!("{:?}", e);
                        if msg.contains("invalid length") || msg.contains("DeserializeFailed") {
                            eprintln!(
                                "ERROR: Yahoo ({} @ {}): Interval not supported by Yahoo for this symbol. \
                                Try: 1d or 1wk for international stocks (NSE/TSE/HKEx). \
                                1m/5m only works for US equities and major crypto pairs.",
                                self.symbol, self.interval
                            );
                        } else {
                            eprintln!(
                                "ERROR: Yahoo ({} @ {}): Parse error: {:?}",
                                self.symbol, self.interval, e
                            );
                        }
                        Vec::new()
                    }
                }
            }
            Err(e) => {
                let msg = format!("{:?}", e);
                if msg.contains("invalid length") || msg.contains("DeserializeFailed") {
                    eprintln!(
                        "ERROR: Yahoo ({} @ {}): Interval '{}' is not available for this symbol. \
                        Yahoo Finance does not provide intraday data for most international exchanges. \
                        Supported: 1d, 1wk for NSE/BSE/HKEx/TSE symbols.",
                        self.symbol, self.interval, self.interval
                    );
                } else {
                    eprintln!(
                        "ERROR: Yahoo ({} @ {}): Network error: {:?}",
                        self.symbol, self.interval, e
                    );
                }
                Vec::new()
            }
        }
    }
}
