use crate::csv_source::CsvCandleSource;
use crate::data_source::CandleSource;
use crate::market_adapter::Candle;
use std::fs;
use std::path::Path;

pub struct FolderCandleSource {
    pub folder_path: String,
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

            let is_expected_schema = file_name.ends_with("_5m_clean.csv")
                || file_name.ends_with(".ns.csv")
                || file_name.ends_with(".bo.csv");
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
            let candles = source.get_candles();

            let file_stem = Path::new(&path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("UNKNOWN");
            let asset = file_stem
                .split('_')
                .next()
                .unwrap_or("UNKNOWN")
                .to_ascii_uppercase();

            datasets.push((asset, candles));
        }

        datasets
    }
}
