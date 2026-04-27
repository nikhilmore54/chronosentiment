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
    if let Ok(n) = v.parse::<u64>() {
        return Some(n);
    }
    v.parse::<f64>().ok().map(|x| x.max(0.0).round() as u64)
}

fn find_col(headers: &[&str], names: &[&str]) -> Option<usize> {
    headers.iter().position(|h| names.iter().any(|n| h.eq_ignore_ascii_case(n)))
}

impl CandleSource for CsvCandleSource {
    fn get_candles(&self) -> Vec<Candle> {
        let file = File::open(&self.path).expect("Failed to open CSV file");
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let header_line = match lines.next() {
            Some(Ok(h)) => h,
            Some(Err(_)) | None => return Vec::new(),
        };

        let header_parts: Vec<&str> = header_line.split(',').map(|x| x.trim()).collect();

        let ts_idx = find_col(&header_parts, &["timestamp", "time", "datetime"]).unwrap_or(0);
        let open_idx = find_col(&header_parts, &["open"]).unwrap_or(1);
        let high_idx = find_col(&header_parts, &["high"]).unwrap_or(2);
        let low_idx = find_col(&header_parts, &["low"]).unwrap_or(3);
        let close_idx = find_col(&header_parts, &["close"]).unwrap_or(4);
        let vol_idx = find_col(&header_parts, &["volume", "vol"]).unwrap_or(5);

        let mut candles = Vec::new();

        for (row_idx, line) in lines.enumerate() {
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

            let timestamp = parse_u64_cell(get(ts_idx)).unwrap_or((row_idx + 1) as u64);
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
}
