//! TIME-001 — Historical Clock abstraction.
//!
//! # Purpose
//!
//! Every LIVE-00x binary calls `Utc::now()` to stamp its artifacts.  When the
//! Time Machine replays those same binaries at a historical date `T`, every
//! call to "what time is it?" must return `T`, not the wall-clock.
//!
//! `HistoricalClock` is the single injection point that replaces all direct
//! `Utc::now()` calls in the replay path.  It has exactly two modes:
//!
//! - **Live** — returns `Utc::now()` (identical behaviour to the existing
//!   LIVE-00x binaries; no regression).
//! - **Replay** — returns the fixed `as_of` timestamp for every call.  No
//!   future data can leak through this clock because the clock never advances
//!   past `as_of`.
//!
//! # Leakage invariant
//!
//! `HistoricalClock::now()` in Replay mode is **monotonically constant**: it
//! returns the same `as_of` value on every call within a single pipeline run.
//! This is intentional — the entire pipeline run is stamped as if it happened
//! at `T`, which is the prospective-style replay guarantee.
//!
//! # CLI integration
//!
//! The existing `--now` / `--as-of` flag pattern already present in several
//! binaries maps directly to `ClockMode::Replay { as_of }`.  TIME-00x replay
//! binaries will parse `--as-of <RFC3339>` and construct a `HistoricalClock`
//! in Replay mode.  LIVE-00x binaries continue to construct a `HistoricalClock`
//! in Live mode (or omit the flag entirely, defaulting to Live).
//!
//! # Example
//!
//! ```rust
//! use chrono::Utc;
//! use chronosentiment::time_machine::clock::{HistoricalClock, ClockMode};
//!
//! // Live mode — returns Utc::now()
//! let live_clock = HistoricalClock::live();
//! let t = live_clock.now(); // ≈ wall clock
//!
//! // Replay mode — always returns 2024-01-15T09:30:00Z
//! let as_of: chrono::DateTime<Utc> = "2024-01-15T09:30:00Z".parse().unwrap();
//! let replay_clock = HistoricalClock::replay(as_of);
//! assert_eq!(replay_clock.now(), as_of);
//! assert_eq!(replay_clock.now(), as_of); // constant — no leakage
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── ClockMode ────────────────────────────────────────────────────────────────

/// Determines whether the clock returns the live wall-clock or a fixed
/// historical point-in-time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClockMode {
    /// Live mode: `now()` delegates to `Utc::now()`.
    Live,
    /// Replay mode: `now()` always returns `as_of`.
    Replay {
        /// The fixed point-in-time for this pipeline run.
        /// All artifact timestamps within the run are stamped with this value.
        as_of: DateTime<Utc>,
    },
}

// ── HistoricalClock ──────────────────────────────────────────────────────────

/// A deterministic clock that can be injected into any pipeline stage.
///
/// In **Live** mode it behaves identically to `Utc::now()`.
/// In **Replay** mode it returns the same `as_of` timestamp on every call,
/// enforcing the no-future-leakage invariant for Time Machine runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalClock {
    mode: ClockMode,
}

impl HistoricalClock {
    // ── Constructors ─────────────────────────────────────────────────────────

    /// Create a Live clock.  `now()` will return `Utc::now()`.
    pub fn live() -> Self {
        Self {
            mode: ClockMode::Live,
        }
    }

    /// Create a Replay clock fixed at `as_of`.
    /// `now()` will always return `as_of` — never the wall clock.
    pub fn replay(as_of: DateTime<Utc>) -> Self {
        Self {
            mode: ClockMode::Replay { as_of },
        }
    }

    /// Parse a Replay clock from an RFC 3339 string.
    ///
    /// Returns `Err` if the string is not a valid RFC 3339 timestamp.
    /// This is the canonical entry point for `--as-of <RFC3339>` CLI flags.
    pub fn replay_from_str(s: &str) -> Result<Self, String> {
        let as_of: DateTime<Utc> = s
            .parse()
            .map_err(|e| format!("--as-of must be RFC3339 (e.g. 2024-01-15T09:30:00Z): {e}"))?;
        Ok(Self::replay(as_of))
    }

    // ── Core API ─────────────────────────────────────────────────────────────

    /// Return the current time according to this clock.
    ///
    /// - Live mode: returns `Utc::now()` (wall clock).
    /// - Replay mode: returns the fixed `as_of` timestamp.
    pub fn now(&self) -> DateTime<Utc> {
        match &self.mode {
            ClockMode::Live => Utc::now(),
            ClockMode::Replay { as_of } => *as_of,
        }
    }

    /// Format `now()` as an RFC 3339 / ISO 8601 string with microsecond
    /// precision, matching the existing artifact timestamp format used across
    /// all LIVE-00x binaries (`%Y-%m-%dT%H:%M:%S%.6fZ`).
    pub fn now_str(&self) -> String {
        self.now().format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string()
    }

    // ── Introspection ─────────────────────────────────────────────────────────

    /// Returns `true` if this clock is in Live mode.
    pub fn is_live(&self) -> bool {
        matches!(self.mode, ClockMode::Live)
    }

    /// Returns `true` if this clock is in Replay mode.
    pub fn is_replay(&self) -> bool {
        matches!(self.mode, ClockMode::Replay { .. })
    }

    /// Returns the `as_of` timestamp if in Replay mode, or `None` if Live.
    pub fn as_of(&self) -> Option<DateTime<Utc>> {
        match &self.mode {
            ClockMode::Live => None,
            ClockMode::Replay { as_of } => Some(*as_of),
        }
    }

    /// Returns the mode label string used in artifact metadata.
    /// - Live → `"LIVE"`
    /// - Replay → `"REPLAY"`
    pub fn mode_label(&self) -> &'static str {
        match &self.mode {
            ClockMode::Live => "LIVE",
            ClockMode::Replay { .. } => "REPLAY",
        }
    }

    // ── CLI helper ────────────────────────────────────────────────────────────

    /// Build a `HistoricalClock` from an optional `--as-of` CLI argument.
    ///
    /// - `Some(s)` → Replay mode at the parsed RFC 3339 timestamp.
    /// - `None`    → Live mode (wall clock).
    ///
    /// This is the canonical one-liner for all TIME-00x and LIVE-00x `main()`
    /// functions:
    ///
    /// ```rust,ignore
    /// let clock = HistoricalClock::from_cli_arg(args.as_of.as_deref())?;
    /// ```
    pub fn from_cli_arg(as_of: Option<&str>) -> Result<Self, String> {
        match as_of {
            Some(s) => Self::replay_from_str(s),
            None => Ok(Self::live()),
        }
    }
}

// ── Display ──────────────────────────────────────────────────────────────────

impl std::fmt::Display for HistoricalClock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.mode {
            ClockMode::Live => write!(f, "HistoricalClock(LIVE)"),
            ClockMode::Replay { as_of } => {
                write!(
                    f,
                    "HistoricalClock(REPLAY as_of={})",
                    as_of.format("%Y-%m-%dT%H:%M:%SZ")
                )
            }
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn fixed_ts() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2024, 1, 15, 9, 30, 0).unwrap()
    }

    // AC-T1-01: Live clock returns a non-zero timestamp (wall clock is running)
    #[test]
    fn ac_t1_01_live_clock_returns_wall_clock() {
        let clock = HistoricalClock::live();
        let t = clock.now();
        // Wall clock must be after the Unix epoch
        assert!(
            t.timestamp() > 0,
            "Live clock must return a positive timestamp"
        );
        assert!(clock.is_live());
        assert!(!clock.is_replay());
        assert_eq!(clock.as_of(), None);
        assert_eq!(clock.mode_label(), "LIVE");
    }

    // AC-T1-02: Replay clock always returns the fixed as_of — no leakage
    #[test]
    fn ac_t1_02_replay_clock_is_constant() {
        let as_of = fixed_ts();
        let clock = HistoricalClock::replay(as_of);
        // Multiple calls must return the same value
        assert_eq!(clock.now(), as_of);
        assert_eq!(clock.now(), as_of);
        assert_eq!(clock.now(), as_of);
        assert!(!clock.is_live());
        assert!(clock.is_replay());
        assert_eq!(clock.as_of(), Some(as_of));
        assert_eq!(clock.mode_label(), "REPLAY");
    }

    // AC-T1-03: Replay clock never returns a time after as_of
    #[test]
    fn ac_t1_03_replay_clock_no_future_leakage() {
        let as_of = fixed_ts();
        let clock = HistoricalClock::replay(as_of);
        let t = clock.now();
        assert!(
            t <= as_of,
            "Replay clock must never return a time after as_of (leakage invariant)"
        );
    }

    // AC-T1-04: replay_from_str parses valid RFC 3339
    #[test]
    fn ac_t1_04_replay_from_str_valid() {
        let clock = HistoricalClock::replay_from_str("2024-01-15T09:30:00Z").unwrap();
        assert_eq!(clock.now(), fixed_ts());
    }

    // AC-T1-05: replay_from_str rejects invalid strings
    #[test]
    fn ac_t1_05_replay_from_str_invalid() {
        let result = HistoricalClock::replay_from_str("not-a-date");
        assert!(result.is_err(), "Invalid RFC3339 must return Err");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("--as-of must be RFC3339"),
            "Error message must mention --as-of"
        );
    }

    // AC-T1-06: from_cli_arg(None) → Live
    #[test]
    fn ac_t1_06_from_cli_arg_none_is_live() {
        let clock = HistoricalClock::from_cli_arg(None).unwrap();
        assert!(clock.is_live());
    }

    // AC-T1-07: from_cli_arg(Some(...)) → Replay
    #[test]
    fn ac_t1_07_from_cli_arg_some_is_replay() {
        let clock = HistoricalClock::from_cli_arg(Some("2024-01-15T09:30:00Z")).unwrap();
        assert!(clock.is_replay());
        assert_eq!(clock.as_of(), Some(fixed_ts()));
    }

    // AC-T1-08: now_str() format matches LIVE-00x artifact timestamp format
    #[test]
    fn ac_t1_08_now_str_format() {
        let as_of = fixed_ts();
        let clock = HistoricalClock::replay(as_of);
        let s = clock.now_str();
        // Must match %Y-%m-%dT%H:%M:%S%.6fZ
        assert_eq!(s, "2024-01-15T09:30:00.000000Z");
        // Must be parseable back to the same timestamp
        let parsed: DateTime<Utc> = s.parse().unwrap();
        assert_eq!(parsed, as_of);
    }

    // AC-T1-09: Display output is human-readable and mode-labelled
    #[test]
    fn ac_t1_09_display_live() {
        let clock = HistoricalClock::live();
        assert_eq!(format!("{clock}"), "HistoricalClock(LIVE)");
    }

    #[test]
    fn ac_t1_10_display_replay() {
        let clock = HistoricalClock::replay(fixed_ts());
        assert_eq!(
            format!("{clock}"),
            "HistoricalClock(REPLAY as_of=2024-01-15T09:30:00Z)"
        );
    }

    // AC-T1-11: Serde round-trip — clock survives JSON serialization
    #[test]
    fn ac_t1_11_serde_round_trip_live() {
        let clock = HistoricalClock::live();
        let json = serde_json::to_string(&clock).unwrap();
        let restored: HistoricalClock = serde_json::from_str(&json).unwrap();
        assert!(restored.is_live());
    }

    #[test]
    fn ac_t1_12_serde_round_trip_replay() {
        let as_of = fixed_ts();
        let clock = HistoricalClock::replay(as_of);
        let json = serde_json::to_string(&clock).unwrap();
        let restored: HistoricalClock = serde_json::from_str(&json).unwrap();
        assert!(restored.is_replay());
        assert_eq!(restored.as_of(), Some(as_of));
        assert_eq!(restored.now(), as_of);
    }
}
