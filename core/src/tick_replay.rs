use crate::binance_adapter::{
    parse_binance_depth_event, parse_binance_trade_event, NormalizedMarketEvent,
};
use crate::MarketEvent;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayMode {
    Fast,
    RealTime,
}

#[derive(Debug, Clone)]
pub struct ReplayedEvent {
    pub event: NormalizedMarketEvent,
    pub simulated_ts: u64,
    pub decision_ts: u64,
    pub execution_ts: u64,
    /// In real-time mode this indicates how long caller should wait before emitting/executing.
    pub wait_ms: u64,
}

#[derive(Debug, Clone)]
pub struct ReplayConfig {
    pub mode: ReplayMode,
    pub decision_latency_ms: u64,
    pub base_execution_latency_ms: u64,
    pub variable_execution_latency_ms: u64,
}

impl Default for ReplayConfig {
    fn default() -> Self {
        Self {
            mode: ReplayMode::Fast,
            decision_latency_ms: 10,
            base_execution_latency_ms: 50,
            variable_execution_latency_ms: 150,
        }
    }
}

pub struct TickReplayEngine {
    events: Vec<NormalizedMarketEvent>,
    cursor: usize,
    config: ReplayConfig,
    replay_start_wall_ms: Option<u64>,
    replay_base_exchange_ts: Option<u64>,
}

impl TickReplayEngine {
    pub fn from_events(mut events: Vec<NormalizedMarketEvent>, config: ReplayConfig) -> Self {
        events.sort_by_key(|e| e.exchange_ts);
        Self {
            events,
            cursor: 0,
            config,
            replay_start_wall_ms: None,
            replay_base_exchange_ts: None,
        }
    }

    /// Parse JSONL where each line is a Binance trade/depth payload (possibly wrapped in metadata).
    pub fn from_binance_jsonl(path: &str, config: ReplayConfig, depth_top_k: usize) -> std::io::Result<Self> {
        let events = crate::binance_adapter::load_binance_events_from_jsonl(path, depth_top_k)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(Self::from_events(events, config))
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn next_event(&mut self) -> Option<ReplayedEvent> {
        let event = self.events.get(self.cursor)?.clone();
        self.cursor += 1;

        let event_ts = event.exchange_ts;
        let exec_latency = deterministic_latency(
            event_ts,
            self.config.base_execution_latency_ms,
            self.config.variable_execution_latency_ms,
        );
        let decision_ts = event_ts.saturating_add(self.config.decision_latency_ms);
        let execution_ts = decision_ts.saturating_add(exec_latency);

        let wait_ms = match self.config.mode {
            ReplayMode::Fast => 0,
            ReplayMode::RealTime => self.compute_wait_ms(event_ts),
        };

        Some(ReplayedEvent {
            event,
            simulated_ts: execution_ts,
            decision_ts,
            execution_ts,
            wait_ms,
        })
    }

    /// Replay all events into legacy MarketEvents so existing pipeline code can consume them.
    pub fn to_legacy_market_events(&self) -> Vec<MarketEvent> {
        self.events
            .iter()
            .filter_map(|e| e.to_legacy_market_event())
            .collect()
    }

    fn compute_wait_ms(&mut self, event_ts: u64) -> u64 {
        let now_ms = now_unix_ms();
        if self.replay_start_wall_ms.is_none() {
            self.replay_start_wall_ms = Some(now_ms);
            self.replay_base_exchange_ts = Some(event_ts);
            return 0;
        }
        let base_wall = self.replay_start_wall_ms.unwrap_or(now_ms);
        let base_exchange = self.replay_base_exchange_ts.unwrap_or(event_ts);
        let target_wall = base_wall.saturating_add(event_ts.saturating_sub(base_exchange));
        target_wall.saturating_sub(now_ms)
    }
}

/// Deterministic bounded latency function (no randomness).
pub fn deterministic_latency(ts_ms: u64, base_ms: u64, variable_ms: u64) -> u64 {
    if variable_ms == 0 {
        return base_ms;
    }
    base_ms.saturating_add(ts_ms % variable_ms)
}

fn now_unix_ms() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_millis() as u64,
        Err(_) => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Side;

    #[test]
    fn deterministic_latency_is_stable() {
        let t1 = deterministic_latency(1_710_000_000_123, 50, 150);
        let t2 = deterministic_latency(1_710_000_000_123, 50, 150);
        assert_eq!(t1, t2);
        assert!(t1 >= 50 && t1 < 200);
    }

    #[test]
    fn replay_orders_by_exchange_ts() {
        let mut eng = TickReplayEngine::from_events(
            vec![
                NormalizedMarketEvent {
                    exchange_ts: 3,
                    price: 101.0,
                    volume: 1.0,
                    side: Some(Side::Buy),
                    best_bid: None,
                    best_ask: None,
                    bids: None,
                    asks: None,
                },
                NormalizedMarketEvent {
                    exchange_ts: 1,
                    price: 100.0,
                    volume: 1.0,
                    side: Some(Side::Sell),
                    best_bid: None,
                    best_ask: None,
                    bids: None,
                    asks: None,
                },
            ],
            ReplayConfig::default(),
        );
        let a = eng.next_event().expect("first");
        let b = eng.next_event().expect("second");
        assert!(a.event.exchange_ts < b.event.exchange_ts);
    }
}
