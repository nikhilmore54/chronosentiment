use crate::csv_source::CsvCandleSource;
use crate::data_source::CandleSource;
use crate::market_adapter::Candle;
use std::fs;
use std::path::Path;

pub struct FolderCandleSource {
    pub folder_path: String,
}

fn extract_symbol(path: &Path) -> String {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    
    file_name
        .to_lowercase()
        .replace("_5m_clean.csv", "")
        .replace(".csv", "")
        .to_uppercase()
}

impl FolderCandleSource {
    pub fn load_all(&self) -> Vec<(String, Vec<Candle>)> {
        let mut csv_files = Vec::new();

        for entry in fs::read_dir(&self.folder_path).expect("Failed to read folder path") {
            let entry = entry.expect("Failed to read folder entry");
            let path = entry.path();

            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();

            let is_csv = path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("csv"))
                .unwrap_or(false);

            let is_expected_schema = file_name.ends_with("_5m_clean.csv");
            let ignored_file = file_name == "results.csv";

            if is_csv && is_expected_schema && !ignored_file {
                csv_files.push(path);
            }
        }

        csv_files.sort_by(|a, b| {
            let a_name = a.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            let b_name = b.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            a_name.cmp(b_name)
        });

        let mut datasets: Vec<(String, Vec<Candle>)> = Vec::new();
        for path in csv_files {
            let path_str = path
                .to_str()
                .expect("CSV path contains non-UTF8 characters")
                .to_string();
            let source = CsvCandleSource { path: path_str };
            let candles = source.get_candles_sync();

            let asset = extract_symbol(&path);
            datasets.push((asset, candles));
        }

        datasets
    }

    /// NEW: Load all CSV files in the folder (any name, not restricted to _5m_clean.csv).
    /// Used by the NSE multi-asset training pipeline.
    /// Symbol name is derived from the filename (strips .csv extension, uppercases).
    pub fn load_all_flexible(&self) -> Vec<(String, Vec<Candle>)> {
        let mut csv_files = Vec::new();

        let entries = match fs::read_dir(&self.folder_path) {
            Ok(e) => e,
            Err(err) => {
                eprintln!("ERROR: Cannot read folder '{}': {}", self.folder_path, err);
                return Vec::new();
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let path = entry.path();

            let file_name_lc = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();

            // Skip non-csv and results.csv
            if !file_name_lc.ends_with(".csv") || file_name_lc == "results.csv" {
                continue;
            }

            csv_files.push(path);
        }

        csv_files.sort_by(|a, b| {
            a.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .cmp(b.file_name().and_then(|n| n.to_str()).unwrap_or_default())
        });

        let mut datasets: Vec<(String, Vec<Candle>)> = Vec::new();
        for path in csv_files {
            // Symbol = filename without extension, uppercased (e.g. "RELIANCE.NS.csv" → "RELIANCE.NS")
            let symbol = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("UNKNOWN")
                .to_uppercase();

            let path_str = match path.to_str() {
                Some(s) => s.to_string(),
                None => continue,
            };

            let source = CsvCandleSource { path: path_str };
            let candles = source.get_candles_sync();
            datasets.push((symbol, candles));
        }

        datasets
    }
    pub fn list_symbols(&self) -> Vec<String> {
        let mut symbols = Vec::new();
        let entries = match fs::read_dir(&self.folder_path) {
            Ok(e) => e,
            Err(_) => return symbols,
        };
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_ascii_lowercase();
                if file_name.ends_with(".csv") && file_name != "results.csv" {
                    let symbol = path.file_stem().and_then(|s| s.to_str()).unwrap_or("UNKNOWN").to_uppercase();
                    symbols.push(symbol);
                }
            }
        }
        symbols.sort();
        symbols
    }
}
