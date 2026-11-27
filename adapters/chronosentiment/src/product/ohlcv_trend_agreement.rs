//! RP-608-GATE2E-15M-OHLCV-TREND-AGREE-001
//!
//! Multi-lookback direction labels on **completed** 15m closes (L3/L8/L20).
//! Adapter-owned Gate 2E entry selector. Does not touch `coralys-*`.
//! Does not change frozen traj15 ≥ 0.05 / OF ±0.30 / stops.

use serde::{Deserialize, Serialize};

pub const PROTOCOL_ID: &str = "RP-608-GATE2E-15M-OHLCV-TREND-AGREE-001";
pub const LOOKBACKS: [usize; 3] = [3, 8, 20];
const FLAT_EPS: f64 = 1e-12;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrendSide {
    Long,
    Short,
    Flat,
}

impl TrendSide {
    pub fn as_str(self) -> &'static str {
        match self {
            TrendSide::Long => "LONG",
            TrendSide::Short => "SHORT",
            TrendSide::Flat => "FLAT",
        }
    }

    pub fn parse(side: &str) -> Option<Self> {
        match side.trim().to_ascii_uppercase().as_str() {
            "LONG" => Some(TrendSide::Long),
            "SHORT" => Some(TrendSide::Short),
            "FLAT" | "" => Some(TrendSide::Flat),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LookbackVote {
    pub lookback: usize,
    pub side: TrendSide,
    pub ret: Option<f64>,
    pub eligible: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrendAgreementStamp {
    pub protocol_id: String,
    pub candidate_side: String,
    pub votes: Vec<LookbackVote>,
    pub agree_count: u32,
    pub oppose_count: u32,
    pub flat_or_missing: u32,
    pub agreement: String,
    pub blocks_entry: bool,
    pub block_reason: Option<String>,
}

/// Direction of `closes[n-1] - closes[n-k]` for lookback `k`.
pub fn label_lookback(closes: &[f64], lookback: usize) -> LookbackVote {
    if lookback == 0 || closes.len() < lookback {
        return LookbackVote {
            lookback,
            side: TrendSide::Flat,
            ret: None,
            eligible: false,
        };
    }
    let c1 = closes[closes.len() - 1];
    let c0 = closes[closes.len() - lookback];
    let ret = (c1 - c0) / (c0.abs() + 1e-9);
    let side = if ret > FLAT_EPS {
        TrendSide::Long
    } else if ret < -FLAT_EPS {
        TrendSide::Short
    } else {
        TrendSide::Flat
    };
    LookbackVote {
        lookback,
        side,
        ret: Some(ret),
        eligible: true,
    }
}

pub fn label_lookbacks(closes: &[f64]) -> Vec<LookbackVote> {
    LOOKBACKS
        .iter()
        .map(|&k| label_lookback(closes, k))
        .collect()
}

/// Gate 2E B selector: candidate side must collect ≥2 agreeing lookback votes
/// and zero opposing votes among L3/L8/L20.
pub fn agreement(closes: &[f64], candidate_side: &str) -> TrendAgreementStamp {
    let side = TrendSide::parse(candidate_side).unwrap_or(TrendSide::Flat);
    let votes = label_lookbacks(closes);
    let mut agree = 0u32;
    let mut oppose = 0u32;
    let mut flat_or_missing = 0u32;
    for v in &votes {
        if !v.eligible || v.side == TrendSide::Flat {
            flat_or_missing += 1;
            continue;
        }
        if side == TrendSide::Long || side == TrendSide::Short {
            if v.side == side {
                agree += 1;
            } else {
                oppose += 1;
            }
        } else {
            flat_or_missing += 1;
        }
    }

    let (agreement, blocks, reason) = if side != TrendSide::Long && side != TrendSide::Short {
        (
            "STANDASIDE".to_string(),
            true,
            Some("CANDIDATE_SIDE_NOT_TRADEABLE".to_string()),
        )
    } else if agree >= 2 && oppose == 0 {
        ("ALLOW".to_string(), false, None)
    } else if oppose > 0 {
        (
            "STANDASIDE".to_string(),
            true,
            Some("LOOKBACK_OPPOSE".to_string()),
        )
    } else {
        (
            "STANDASIDE".to_string(),
            true,
            Some("INSUFFICIENT_AGREE_VOTES".to_string()),
        )
    };

    TrendAgreementStamp {
        protocol_id: PROTOCOL_ID.to_string(),
        candidate_side: side.as_str().to_string(),
        votes,
        agree_count: agree,
        oppose_count: oppose,
        flat_or_missing,
        agreement,
        blocks_entry: blocks,
        block_reason: reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_history_standaside() {
        let closes = vec![100.0, 101.0];
        let stamp = agreement(&closes, "LONG");
        assert!(stamp.blocks_entry);
        assert_eq!(
            stamp.block_reason.as_deref(),
            Some("INSUFFICIENT_AGREE_VOTES")
        );
    }

    #[test]
    fn rising_closes_allow_long() {
        let closes: Vec<f64> = (0..25).map(|i| 100.0 + i as f64).collect();
        let stamp = agreement(&closes, "LONG");
        assert!(!stamp.blocks_entry, "{stamp:?}");
        assert_eq!(stamp.agreement, "ALLOW");
        assert_eq!(stamp.agree_count, 3);
        assert_eq!(stamp.oppose_count, 0);
    }

    #[test]
    fn rising_closes_block_short() {
        let closes: Vec<f64> = (0..25).map(|i| 100.0 + i as f64).collect();
        let stamp = agreement(&closes, "SHORT");
        assert!(stamp.blocks_entry);
        assert_eq!(stamp.block_reason.as_deref(), Some("LOOKBACK_OPPOSE"));
        assert!(stamp.oppose_count >= 2);
    }

    #[test]
    fn no_partial_bar_leakage_uses_completed_only() {
        let completed: Vec<f64> = (0..20).map(|i| 50.0 + i as f64).collect();
        let with_forming: Vec<f64> = {
            let mut v = completed.clone();
            v.push(0.0);
            v
        };
        let a = agreement(&completed, "LONG");
        let b = agreement(&with_forming[..with_forming.len() - 1], "LONG");
        assert_eq!(a.agreement, b.agreement);
        assert_eq!(a.agree_count, b.agree_count);
    }

    #[test]
    fn flat_candidate_blocks() {
        let closes: Vec<f64> = (0..25).map(|i| 100.0 + i as f64).collect();
        let stamp = agreement(&closes, "NO_TRADE");
        assert!(stamp.blocks_entry);
        assert_eq!(
            stamp.block_reason.as_deref(),
            Some("CANDIDATE_SIDE_NOT_TRADEABLE")
        );
    }
}
