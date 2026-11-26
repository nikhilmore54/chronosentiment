use anyhow::{Context, Result};
use serde::Serialize;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};

use crate::frozen_loader::FrozenBar;

#[derive(Serialize)]
struct SymbolicCandle<'a> {
    symbol: &'a str,
    timestamp: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

pub struct ObservatoryProcess {
    child: Child,
    stdout: BufReader<std::process::ChildStdout>,
}

impl ObservatoryProcess {
    pub fn spawn(observatory_path: &Path) -> Result<Self> {
        let mut child = Command::new(observatory_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| format!("spawn {}", observatory_path.display()))?;
        let stdout = child.stdout.take().context("observatory stdout")?;
        Ok(Self {
            stdout: BufReader::new(stdout),
            child,
        })
    }

    pub fn run_barrier(&mut self, ts: i64, bars: &[(String, &FrozenBar)]) -> Result<Vec<String>> {
        let batch: Vec<SymbolicCandle> = bars
            .iter()
            .map(|(sym, b)| SymbolicCandle {
                symbol: sym,
                timestamp: ts,
                open: b.open,
                high: b.high,
                low: b.low,
                close: b.close,
                volume: b.volume,
            })
            .collect();
        if batch.is_empty() {
            return Ok(vec![]);
        }
        let n = batch.len();
        let stdin = self.child.stdin.as_mut().context("observatory stdin")?;
        let payload = serde_json::to_string(&batch)? + "\n";
        stdin.write_all(payload.as_bytes())?;
        stdin.flush()?;

        let mut telemetry_lines = Vec::new();
        while telemetry_lines.len() < n {
            let mut line = String::new();
            let read = self.stdout.read_line(&mut line)?;
            if read == 0 {
                break;
            }
            if line.starts_with("[TELEMETRY]") {
                telemetry_lines.push(line);
            }
        }
        Ok(telemetry_lines)
    }
}

impl Drop for ObservatoryProcess {
    fn drop(&mut self) {
        let _ = self.child.stdin.take();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
