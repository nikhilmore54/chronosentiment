//! Feed health: Live → Stale on disconnect/gap; Live only after reconcile.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StaleReason {
    Disconnect,
    EventGap,
    QuoteAgeExceeded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum FeedStatus {
    Live,
    Stale { reason: StaleReason },
    Reconciling { prior_reason: StaleReason },
}

#[derive(Debug, Clone)]
pub struct FeedHealth {
    status: FeedStatus,
    /// Tokens marked stale until reconcile completes.
    stale_tokens: std::collections::BTreeSet<u32>,
    max_quote_age_ms: f64,
    /// Max allowed exchange-time jump forward without treating as gap (ms). 0 = disabled.
    max_exchange_gap_ms: u64,
    last_exchange_ms_by_token: std::collections::BTreeMap<u32, u64>,
}

impl FeedHealth {
    pub fn new(max_quote_age_ms: f64, max_exchange_gap_ms: u64) -> Self {
        Self {
            status: FeedStatus::Live,
            stale_tokens: std::collections::BTreeSet::new(),
            max_quote_age_ms,
            max_exchange_gap_ms,
            last_exchange_ms_by_token: std::collections::BTreeMap::new(),
        }
    }

    pub fn status(&self) -> &FeedStatus {
        &self.status
    }

    pub fn is_live(&self) -> bool {
        matches!(self.status, FeedStatus::Live)
    }

    pub fn on_disconnect(&mut self, subscribed: &[u32]) {
        self.status = FeedStatus::Stale {
            reason: StaleReason::Disconnect,
        };
        for t in subscribed {
            self.stale_tokens.insert(*t);
        }
    }

    pub fn note_quote_age(&mut self, token: u32, quote_age_ms: f64) {
        if quote_age_ms > self.max_quote_age_ms {
            self.mark_stale(token, StaleReason::QuoteAgeExceeded);
        }
    }

    pub fn note_exchange_time(&mut self, token: u32, exchange_ms: u64) -> Option<StaleReason> {
        if let Some(prev) = self.last_exchange_ms_by_token.get(&token).copied() {
            if exchange_ms < prev {
                // Out-of-order exchange clock → gap / integrity issue.
                self.mark_stale(token, StaleReason::EventGap);
                return Some(StaleReason::EventGap);
            }
            if self.max_exchange_gap_ms > 0 && exchange_ms.saturating_sub(prev) > self.max_exchange_gap_ms
            {
                self.mark_stale(token, StaleReason::EventGap);
                return Some(StaleReason::EventGap);
            }
        }
        self.last_exchange_ms_by_token.insert(token, exchange_ms);
        None
    }

    fn mark_stale(&mut self, token: u32, reason: StaleReason) {
        self.stale_tokens.insert(token);
        match self.status {
            FeedStatus::Live => {
                self.status = FeedStatus::Stale { reason };
            }
            FeedStatus::Stale { .. } | FeedStatus::Reconciling { .. } => {}
        }
    }

    /// Begin reconcile; live is not restored until `complete_reconcile`.
    pub fn begin_reconcile(&mut self) {
        let prior = match self.status {
            FeedStatus::Stale { reason } => reason,
            FeedStatus::Reconciling { prior_reason } => prior_reason,
            FeedStatus::Live => StaleReason::Disconnect,
        };
        self.status = FeedStatus::Reconciling {
            prior_reason: prior,
        };
    }

    /// Restore live only after explicit reconcile completion.
    /// Clears per-token exchange-time baselines so post-reconcile live
    /// observations are not stuck on pre-disconnect gap detection.
    pub fn complete_reconcile(&mut self) {
        if matches!(self.status, FeedStatus::Reconciling { .. }) {
            self.status = FeedStatus::Live;
            self.stale_tokens.clear();
            self.last_exchange_ms_by_token.clear();
        }
    }

    pub fn token_is_stale(&self, token: u32) -> bool {
        !self.is_live() || self.stale_tokens.contains(&token)
    }

    /// Attempting to treat quotes as live without reconcile must fail closed.
    pub fn try_observe_as_live(&self, token: u32) -> Result<(), StaleReason> {
        if self.token_is_stale(token) {
            return Err(match &self.status {
                FeedStatus::Stale { reason } => *reason,
                FeedStatus::Reconciling { prior_reason } => *prior_reason,
                FeedStatus::Live => StaleReason::EventGap,
            });
        }
        Ok(())
    }
}
