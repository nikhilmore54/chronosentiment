//! Deterministic market-event engine: order, dedupe, provenance, stale gate, OF linkage.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

use super::decode::{content_fingerprint, decode_full_mode_frame, decode_full_mode_packet};
use super::depth_journal::{depth_snapshot_from_event, DepthSnapshotRecord};
use super::event::{MarketEvent, SourceProvenance};
use super::of_mark::{inspect_from_market_event, OfMarkRecord, OfSide};
use super::stale::{FeedHealth, FeedStatus, StaleReason};

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub max_quote_age_ms: f64,
    /// Exchange-time forward jump that triggers EventGap (0 disables).
    pub max_exchange_gap_ms: u64,
    /// Keep last N content fingerprints per token for dedupe.
    pub dedupe_window: usize,
    /// Primary side for observational OF badges (frozen threshold applies).
    pub of_primary_side: OfSide,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_quote_age_ms: 60_000.0,
            max_exchange_gap_ms: 0,
            dedupe_window: 64,
            of_primary_side: OfSide::Long,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptedEvent {
    pub event: MarketEvent,
    pub depth_snapshot: DepthSnapshotRecord,
    pub of_mark: OfMarkRecord,
    /// True when feed was not Live for this token (still recorded for history).
    pub observed_while_stale: bool,
}

/// Traceable triple: event ↔ depth ↔ OF (same ingest_seq / content_fp).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedObservation {
    pub ingest_seq: u64,
    pub content_fp: u64,
    pub instrument_token: u32,
    pub event: MarketEvent,
    pub depth_snapshot: DepthSnapshotRecord,
    pub of_mark: OfMarkRecord,
}

#[derive(Debug)]
pub struct MarketEventEngine {
    cfg: EngineConfig,
    next_seq: u64,
    health: FeedHealth,
    subscribed: BTreeSet<u32>,
    recent_fp: BTreeMap<u32, VecDeque<u64>>,
    depth_history: Vec<DepthSnapshotRecord>,
    of_history: Vec<OfMarkRecord>,
    accepted: Vec<AcceptedEvent>,
    /// Latest *current* OF mark per token (cleared on stale; never served while stale).
    current_of: BTreeMap<u32, OfMarkRecord>,
    dropped_duplicates: u64,
}

impl MarketEventEngine {
    pub fn new(cfg: EngineConfig) -> Self {
        let health = FeedHealth::new(cfg.max_quote_age_ms, cfg.max_exchange_gap_ms);
        Self {
            cfg,
            next_seq: 1,
            health,
            subscribed: BTreeSet::new(),
            recent_fp: BTreeMap::new(),
            depth_history: Vec::new(),
            of_history: Vec::new(),
            accepted: Vec::new(),
            current_of: BTreeMap::new(),
            dropped_duplicates: 0,
        }
    }

    pub fn subscribe(&mut self, tokens: impl IntoIterator<Item = u32>) {
        for t in tokens {
            self.subscribed.insert(t);
        }
    }

    pub fn feed_status(&self) -> &FeedStatus {
        self.health.status()
    }

    pub fn dropped_duplicates(&self) -> u64 {
        self.dropped_duplicates
    }

    pub fn depth_history(&self) -> &[DepthSnapshotRecord] {
        &self.depth_history
    }

    pub fn of_history(&self) -> &[OfMarkRecord] {
        &self.of_history
    }

    pub fn accepted(&self) -> &[AcceptedEvent] {
        &self.accepted
    }

    pub fn linked_observations(&self) -> Vec<LinkedObservation> {
        self.accepted
            .iter()
            .map(|a| LinkedObservation {
                ingest_seq: a.event.ingest_seq,
                content_fp: a.event.content_fp,
                instrument_token: a.event.instrument_token,
                event: a.event.clone(),
                depth_snapshot: a.depth_snapshot.clone(),
                of_mark: a.of_mark.clone(),
            })
            .collect()
    }

    /// Current OF for display — `None` while stale (never present stale as current).
    pub fn current_of_mark(&self, token: u32) -> Option<&OfMarkRecord> {
        if self.health.token_is_stale(token) {
            return None;
        }
        self.current_of.get(&token).filter(|m| m.is_current)
    }

    pub fn ordered_events(&self) -> Vec<&MarketEvent> {
        let mut v: Vec<&MarketEvent> = self.accepted.iter().map(|a| &a.event).collect();
        v.sort_by(|a, b| {
            a.exchange_timestamp_utc_ms
                .cmp(&b.exchange_timestamp_utc_ms)
                .then(a.ingest_seq.cmp(&b.ingest_seq))
        });
        v
    }

    pub fn on_disconnect(&mut self) {
        let toks: Vec<u32> = self.subscribed.iter().copied().collect();
        self.health.on_disconnect(&toks);
        self.current_of.clear();
    }

    pub fn begin_reconcile(&mut self) {
        self.health.begin_reconcile();
        self.current_of.clear();
    }

    pub fn complete_reconcile(&mut self) {
        self.health.complete_reconcile();
        // Current OF restored only by subsequent live events.
        self.current_of.clear();
    }

    pub fn ingest_packet(
        &mut self,
        pkt: &[u8],
        receipt_utc_ms: u64,
        source: SourceProvenance,
        symbol: Option<&str>,
    ) -> Result<Option<AcceptedEvent>, StaleReason> {
        let ev = decode_full_mode_packet(pkt, receipt_utc_ms, source)
            .map_err(|_| StaleReason::EventGap)?;
        Ok(self.accept_decoded(ev, symbol))
    }

    pub fn ingest_frame(
        &mut self,
        frame: &[u8],
        receipt_utc_ms: u64,
        source: SourceProvenance,
        symbol_for_token: &dyn Fn(u32) -> Option<String>,
    ) -> Result<Vec<AcceptedEvent>, StaleReason> {
        let decoded = decode_full_mode_frame(frame, receipt_utc_ms, source)
            .map_err(|_| StaleReason::EventGap)?;
        let mut out = Vec::new();
        for ev in decoded {
            let sym = symbol_for_token(ev.instrument_token);
            if let Some(acc) = self.accept_decoded(ev, sym.as_deref()) {
                out.push(acc);
            }
        }
        Ok(out)
    }

    fn accept_decoded(&mut self, mut ev: MarketEvent, symbol: Option<&str>) -> Option<AcceptedEvent> {
        self.subscribed.insert(ev.instrument_token);
        self.health
            .note_quote_age(ev.instrument_token, ev.quote_age_ms);
        let _ = self
            .health
            .note_exchange_time(ev.instrument_token, ev.exchange_timestamp_utc_ms);

        ev.content_fp = content_fingerprint(&ev);
        let fp = ev.content_fp;
        let tok = ev.instrument_token;
        let window = self.recent_fp.entry(tok).or_default();
        if window.contains(&fp) {
            self.dropped_duplicates += 1;
            return None;
        }
        window.push_back(fp);
        while window.len() > self.cfg.dedupe_window {
            window.pop_front();
        }

        ev.ingest_seq = self.next_seq;
        self.next_seq = self.next_seq.saturating_add(1);

        let observed_while_stale = self.health.token_is_stale(tok);
        let snap = depth_snapshot_from_event(&ev, symbol.unwrap_or("UNKNOWN"), ev.receipt_utc_ms);

        let inspection = inspect_from_market_event(&ev, self.cfg.of_primary_side);
        let suppressed_reason = if observed_while_stale {
            match self.health.status() {
                FeedStatus::Stale { reason } => Some(*reason),
                FeedStatus::Reconciling { prior_reason } => Some(*prior_reason),
                FeedStatus::Live => Some(StaleReason::EventGap),
            }
        } else {
            None
        };
        let is_current = !observed_while_stale;
        let of_mark = OfMarkRecord {
            ingest_seq: ev.ingest_seq,
            content_fp: ev.content_fp,
            instrument_token: tok,
            exchange_timestamp_utc_ms: ev.exchange_timestamp_utc_ms,
            receipt_utc_ms: ev.receipt_utc_ms,
            quote_source: ev.source.as_str().to_string(),
            inspection,
            is_current,
            suppressed_reason,
        };

        self.depth_history.push(snap.clone());
        self.of_history.push(of_mark.clone());
        if is_current {
            self.current_of.insert(tok, of_mark.clone());
        } else {
            self.current_of.remove(&tok);
        }

        let accepted = AcceptedEvent {
            event: ev,
            depth_snapshot: snap,
            of_mark,
            observed_while_stale,
        };
        self.accepted.push(accepted.clone());
        Some(accepted)
    }

    pub fn require_live(&self, token: u32) -> Result<(), StaleReason> {
        self.health.try_observe_as_live(token)
    }
}
