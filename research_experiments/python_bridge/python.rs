use crate::data_source::CandleSource;
use crate::market_adapter::Candle;
use async_trait::async_trait;
use serde_json::Value;

/// Fetches candles by calling `scripts/fetch_candles.py` as a subprocess.
/// Uses yfinance under the hood — supports NSE, BSE, HKEx, TSE and any
/// symbol that the Rust yahoo_finance_api crate cannot handle intraday.
pub struct PythonCandleSource {
    pub symbol: String,
    pub interval: String,
    pub min_required_candles: usize,
}

impl PythonCandleSource {
    pub fn new(symbol: &str, interval: &str, min_required_candles: usize) -> Self {
        Self {
            symbol: symbol.to_string(),
            interval: interval.to_string(),
            min_required_candles,
        }
    }

    fn parse_candles(json_bytes: &[u8]) -> Vec<Candle> {
        let json_str = String::from_utf8_lossy(json_bytes);
        let parsed: Vec<Value> = match serde_json::from_str(&json_str) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("ERROR: PythonCandleSource failed to parse JSON: {e}");
                return Vec::new();
            }
        };

        parsed
            .iter()
            .filter_map(|c| {
                let timestamp = c["timestamp"].as_u64()?;
                let open = (c["open"].as_f64()? * 10000.0) as u64;
                let high = (c["high"].as_f64()? * 10000.0) as u64;
                let low = (c["low"].as_f64()? * 10000.0) as u64;
                let close = (c["close"].as_f64()? * 10000.0) as u64;
                let volume = c["volume"].as_u64().unwrap_or(0);
                Some(Candle {
                    timestamp,
                    open,
                    high,
                    low,
                    close,
                    volume,
                })
            })
            .collect()
    }
}

#[async_trait]
impl CandleSource for PythonCandleSource {
    fn get_candles_sync(&self) -> Vec<Candle> {
        let output = std::process::Command::new("python3")
            .arg("scripts/fetch_candles.py")
            .arg(&self.symbol)
            .arg(&self.interval)
            .arg(self.min_required_candles.to_string())
            .output();

        match output {
            Ok(out) => {
                if !out.stderr.is_empty() {
                    eprintln!("{}", String::from_utf8_lossy(&out.stderr).trim_end());
                }
                Self::parse_candles(&out.stdout)
            }
            Err(e) => {
                eprintln!("ERROR: PythonCandleSource: Failed to launch python3: {e}");
                eprintln!("       Ensure python3 and yfinance are installed: pip install yfinance");
                Vec::new()
            }
        }
    }

    async fn get_candles_async(&self) -> Vec<Candle> {
        let symbol = self.symbol.clone();
        let interval = self.interval.clone();
        let n = self.min_required_candles;

        // Spawn in a blocking thread so we don't block the tokio executor
        let result = tokio::task::spawn_blocking(move || {
            let output = std::process::Command::new("python3")
                .arg("scripts/fetch_candles.py")
                .arg(&symbol)
                .arg(&interval)
                .arg(n.to_string())
                .output();

            match output {
                Ok(out) => {
                    if !out.stderr.is_empty() {
                        eprintln!("{}", String::from_utf8_lossy(&out.stderr).trim_end());
                    }
                    Self::parse_candles(&out.stdout)
                }
                Err(e) => {
                    eprintln!("ERROR: PythonCandleSource: Failed to launch python3: {e}");
                    eprintln!("       Install deps: pip install yfinance");
                    Vec::new()
                }
            }
        })
        .await;

        result.unwrap_or_default()
    }
}
