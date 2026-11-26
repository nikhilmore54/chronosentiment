//! Entry-time classification — Intelligence Contract v1.
//!
//! Maps decision-time inputs (direction, OQS, h60_classification) to an
//! [`OpportunityState`] using the frozen v0.3 rules.
//!
//! # Information boundary
//!
//! This module uses **only** information available at T0 (decision time):
//! - `direction`
//! - `opportunity_quality_score` (OQS)
//! - `h60_classification` (ENTER / WAIT / AVOID from frozen Phase 4 rules)
//!
//! `momentum_persistence` is a retrospective path variable computed over the
//! full H300 window. It is **NOT** available at entry time and must **NOT** be
//! used here. See `docs/INTELLIGENCE_CONTRACT_V1.md` §2.
//!
//! # Thresholds (frozen)
//!
//! ```text
//! OQS_LONG_HIGH  = 65
//! OQS_LONG_MID   = 40
//! OQS_SHORT_HIGH = 50
//! ```

use serde::{Deserialize, Serialize};

use super::state_machine::OpportunityState;

// ── Frozen threshold constants ────────────────────────────────────────────────

/// Minimum OQS for LONG WAIT-HIGH classification.
pub const OQS_LONG_HIGH: u32 = 65;

/// Minimum OQS for LONG WAIT-MID classification (inclusive lower bound).
pub const OQS_LONG_MID: u32 = 40;

/// Minimum OQS for SHORT WAIT-HIGH classification.
pub const OQS_SHORT_HIGH: u32 = 50;

/// FAV/ADV boundary (0.2%). Returns above this are FAV; below are ADV.
pub const FAV_ADV_THRESHOLD: f64 = 0.002;

/// MFE floor (0.1%). MFE below this triggers AVOID-LATE override.
pub const MFE_FLOOR: f64 = 0.001;

// ── Input types ───────────────────────────────────────────────────────────────

/// Trade direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Long,
    Short,
}

/// H60 classification from the frozen Phase 4 state machine.
/// This is the only path-derived input permitted at entry time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum H60Classification {
    Enter,
    Wait,
    Avoid,
}

/// Decision-time inputs. All fields must be available at T0.
/// `momentum_persistence` is explicitly excluded — it is retrospective.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryInput {
    pub direction: Direction,
    /// Opportunity Quality Score (0–100). Decision-time composite.
    pub opportunity_quality_score: u32,
    /// H60 classification from frozen Phase 4 rules.
    pub h60_classification: H60Classification,
}

// ── Entry-time classifier ─────────────────────────────────────────────────────

/// Classify an opportunity at entry time using only T0 information.
///
/// Rules (frozen from Intelligence Contract v1, §3.2):
///
/// **SHORT:**
/// - ENTER:     h60 == Enter
/// - AVOID:     h60 == Avoid
/// - WAIT-HIGH: h60 == Wait AND OQS ≥ 50
/// - WAIT-LOW:  h60 == Wait AND OQS < 50
///
/// **LONG:**
/// - ENTER:     h60 == Enter
/// - AVOID:     h60 == Avoid
/// - WAIT-HIGH: h60 == Wait AND OQS ≥ 65
/// - WAIT-MID:  h60 == Wait AND 40 ≤ OQS < 65
/// - WAIT-LOW:  h60 == Wait AND OQS < 40
pub fn classify_at_entry(input: &EntryInput) -> OpportunityState {
    match input.h60_classification {
        H60Classification::Enter => OpportunityState::Enter,
        H60Classification::Avoid => OpportunityState::Avoid,
        H60Classification::Wait => classify_wait(input.direction, input.opportunity_quality_score),
    }
}

fn classify_wait(direction: Direction, oqs: u32) -> OpportunityState {
    match direction {
        Direction::Short => {
            if oqs >= OQS_SHORT_HIGH {
                OpportunityState::WaitHigh
            } else {
                OpportunityState::WaitLow
            }
        }
        Direction::Long => {
            if oqs >= OQS_LONG_HIGH {
                OpportunityState::WaitHigh
            } else if oqs >= OQS_LONG_MID {
                OpportunityState::WaitMid
            } else {
                OpportunityState::WaitLow
            }
        }
    }
}

// ── Path bucket helper ────────────────────────────────────────────────────────

/// Classify a return into FAV / FLAT / ADV using the frozen ±0.2% threshold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnBucket {
    Fav,
    Flat,
    Adv,
}

pub fn bucket(ret: f64) -> ReturnBucket {
    if ret > FAV_ADV_THRESHOLD {
        ReturnBucket::Fav
    } else if ret < -FAV_ADV_THRESHOLD {
        ReturnBucket::Adv
    } else {
        ReturnBucket::Flat
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_enter_classifies_enter() {
        let input = EntryInput {
            direction: Direction::Short,
            opportunity_quality_score: 70,
            h60_classification: H60Classification::Enter,
        };
        assert_eq!(classify_at_entry(&input), OpportunityState::Enter);
    }

    #[test]
    fn short_wait_high_oqs_boundary() {
        let at_50 = EntryInput {
            direction: Direction::Short,
            opportunity_quality_score: 50,
            h60_classification: H60Classification::Wait,
        };
        let at_49 = EntryInput {
            direction: Direction::Short,
            opportunity_quality_score: 49,
            h60_classification: H60Classification::Wait,
        };
        assert_eq!(classify_at_entry(&at_50), OpportunityState::WaitHigh);
        assert_eq!(classify_at_entry(&at_49), OpportunityState::WaitLow);
    }

    #[test]
    fn long_wait_three_way_split() {
        let high = EntryInput {
            direction: Direction::Long,
            opportunity_quality_score: 65,
            h60_classification: H60Classification::Wait,
        };
        let mid = EntryInput {
            direction: Direction::Long,
            opportunity_quality_score: 40,
            h60_classification: H60Classification::Wait,
        };
        let low = EntryInput {
            direction: Direction::Long,
            opportunity_quality_score: 39,
            h60_classification: H60Classification::Wait,
        };
        assert_eq!(classify_at_entry(&high), OpportunityState::WaitHigh);
        assert_eq!(classify_at_entry(&mid), OpportunityState::WaitMid);
        assert_eq!(classify_at_entry(&low), OpportunityState::WaitLow);
    }

    #[test]
    fn avoid_classifies_avoid() {
        let input = EntryInput {
            direction: Direction::Long,
            opportunity_quality_score: 80,
            h60_classification: H60Classification::Avoid,
        };
        assert_eq!(classify_at_entry(&input), OpportunityState::Avoid);
    }

    #[test]
    fn bucket_boundaries() {
        assert_eq!(bucket(0.003), ReturnBucket::Fav);
        assert_eq!(bucket(0.002), ReturnBucket::Flat); // exactly at threshold → FLAT
        assert_eq!(bucket(0.0), ReturnBucket::Flat);
        assert_eq!(bucket(-0.002), ReturnBucket::Flat);
        assert_eq!(bucket(-0.003), ReturnBucket::Adv);
    }
}