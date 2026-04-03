use crate::data_source::CandleSource;
use crate::market_adapter::Candle;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvCandleSource {
    pub path: String,
}

fn parse_u64_cell(raw: &str) -> Option<u64> {
    let v = raw.trim();
    if v.is_empty() {
        return None;
    }
    // We use crate::PRICE_SCALE to preserve precision
    let scale = crate::PRICE_SCALE as f64;
    if let Ok(n) = v.parse::<u64>() {
        let p = n * (crate::PRICE_SCALE as u64);
        return Some(crate::round_to_tick(p));
    }
    v.parse::<f64>().ok().map(|x| {
        let p = (x * scale).max(0.0).round() as u64;
        crate::round_to_tick(p)
    })
}

fn find_col(headers: &[&str], names: &[&str]) -> Option<usize> {
    headers.iter().position(|h| names.iter().any(|n| h.eq_ignore_ascii_case(n)))
}

fn parse_timestamp_strict(raw: &str) -> Option<u64> {
    let v = raw.trim();
    if v.is_empty() { return None; }
    
    // Try Unix timestamp first (integer)
    if let Ok(n) = v.parse::<u64>() {
        return Some(n);
    }

    // Try EXACT formatted timestamp: %Y-%m-%d %H:%M:%S
    use chrono::{NaiveDateTime, DateTime};
    if let Ok(dt) = NaiveDateTime::parse_from_str(v, "%Y-%m-%d %H:%M:%S") {
        return Some(dt.and_utc().timestamp() as u64);
    }
    
    // Try ISO 8601 with offset (e.g. 2024-03-22 09:15:00+05:30 or +0530)
    if let Ok(dt) = DateTime::parse_from_str(v, "%Y-%m-%d %H:%M:%S%:z") {
        return Some(dt.timestamp() as u64);
    }
    if let Ok(dt) = DateTime::parse_from_str(v, "%Y-%m-%d %H:%M:%S%z") {
        return Some(dt.timestamp() as u64);
    }

    // Try DATE-ONLY: %Y-%m-%d
    if let Ok(dt) = chrono::NaiveDate::parse_from_str(v, "%Y-%m-%d") {
        return Some(dt.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp() as u64);
    }
    
    None
}

#[async_trait::async_trait]
impl CandleSource for CsvCandleSource {
    fn get_candles_sync(&self) -> Vec<Candle> {
        let file = match File::open(&self.path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("ERROR: Failed to open CSV file {}: {:?}", self.path, e);
                return Vec::new();
            }
        };
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let header_line = match lines.next() {
            Some(Ok(h)) => h,
            Some(Err(_)) | None => return Vec::new(),
        };

        let header_parts: Vec<&str> = header_line.split(',').map(|x| x.trim()).collect();

        let ts_idx = find_col(&header_parts, &["timestamp", "time", "datetime", "date"]).unwrap_or(0);
        let open_idx = find_col(&header_parts, &["open"]).unwrap_or(1);
        let high_idx = find_col(&header_parts, &["high"]).unwrap_or(2);
        let low_idx = find_col(&header_parts, &["low"]).unwrap_or(3);
        let close_idx = find_col(&header_parts, &["close"]).unwrap_or(4);
        let vol_idx = find_col(&header_parts, &["volume", "vol"]).unwrap_or(5);

        let mut candles = Vec::new();

        for (_row_idx, line) in lines.enumerate() {
            let line = match line {
                Ok(l) => l,
                Err(_) => continue,
            };
            let parts: Vec<&str> = line.split(',').collect();

            let get = |idx: usize| -> &str {
                if idx < parts.len() { parts[idx] } else { "" }
            };

            let open = match parse_u64_cell(get(open_idx)) {
                Some(v) => v,
                None => continue,
            };
            let high = match parse_u64_cell(get(high_idx)) {
                Some(v) => v,
                None => continue,
            };
            let low = match parse_u64_cell(get(low_idx)) {
                Some(v) => v,
                None => continue,
            };
            let close = match parse_u64_cell(get(close_idx)) {
                Some(v) => v,
                None => continue,
            };

            let timestamp = match parse_timestamp_strict(get(ts_idx)) {
                Some(v) => v,
                None => {
                    eprintln!("❌ FATAL_DATA_ERROR: Invalid timestamp format '{}' in {}. Expecting Unix or %Y-%m-%d %H:%M:%S", get(ts_idx), self.path);
                    panic!("CSV_TIMESTAMP_FORMAT_ERROR");
                }
            };
            let volume = parse_u64_cell(get(vol_idx)).unwrap_or(0);

            candles.push(Candle {
                timestamp,
                open,
                high,
                low,
                close,
                volume,
            });
        }

        candles.sort_by_key(|c| c.timestamp);
        candles
    }

    async fn get_candles_async(&self) -> Vec<Candle> {
        self.get_candles_sync()
    }
}
