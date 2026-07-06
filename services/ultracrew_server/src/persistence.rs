use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

fn ensure_dir(path: &Path) {
    if let Err(e) = fs::create_dir_all(path) {
        eprintln!("Failed to create directory {:?}: {}", path, e);
    }
}

pub fn load_collection<T: for<'de> Deserialize<'de>>(dir: &Path) -> Vec<T> {
    ensure_dir(dir);
    let mut items = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(file) = File::open(&path) {
                    let reader = BufReader::new(file);
                    if let Ok(item) = serde_json::from_reader(reader) {
                        items.push(item);
                    } else {
                        eprintln!("Failed to deserialize {:?}", path);
                    }
                }
            }
        }
    }
    items
}

pub fn save_item<T: Serialize>(dir: &Path, filename: &str, item: &T) {
    ensure_dir(dir);
    let file_path = dir.join(format!("{}.json", filename));
    match File::create(&file_path) {
        Ok(file) => {
            let writer = BufWriter::new(file);
            if let Err(e) = serde_json::to_writer_pretty(writer, item) {
                eprintln!("Failed to write {}: {}", file_path.display(), e);
            }
        }
        Err(e) => eprintln!("Failed to create file {}: {}", file_path.display(), e),
    }
}

pub fn delete_item(dir: &Path, filename: &str) {
    let file_path = dir.join(format!("{}.json", filename));
    if let Err(e) = fs::remove_file(&file_path) {
        eprintln!("Failed to delete {}: {}", file_path.display(), e);
    }
}
