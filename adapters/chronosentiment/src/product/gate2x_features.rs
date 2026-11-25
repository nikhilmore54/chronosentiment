//! Gate 2X feature snapshot DTO (adapter-owned).
//!
//! Shadow builds this snapshot; [`crate::product::gate2x_decide`] owns the decision.
//! `coralys-*` crates remain frozen (PVP-003 / CS-P-AUDIT).

use serde::{Deserialize, Serialize};

/// Volume surge bucket at soft-breach (candle / bar relative volume).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum VolSurgeBucket {
    High,
    Med,
    Low,
    Unk,
}

impl VolSurgeBucket {
    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_uppercase().as_str() {
            "HIGH" => Self::High,
            "MED" => Self::Med,
            "LOW" => Self::Low,
            _ => Self::Unk,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::High => "HIGH",
            Self::Med => "MED",
            Self::Low => "LOW",
            Self::Unk => "UNK",
        }
    }
}

/// Protect floor mode (v1.1+).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProtectFloorMode {
    CostClear,
    Entry,
}

impl ProtectFloorMode {
    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_uppercase().as_str() {
            "ENTRY" => Self::Entry,
            _ => Self::CostClear,
        }
    }
}

/// Default Gate 2X v1.3 protocol parameters (crypto / NSE share geometry).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate2xParams {
    pub enabled: bool,
    pub arm_min_tp_frac: f64,
    pub min_hold_s_before_exit: f64,
    pub meaningful_floor_bps: f64,
    pub protect_floor: ProtectFloorMode,
    pub reversal_volume_confirm: bool,
    pub immediate_vol_surges: Vec<String>,
    pub med_vol_dwell_ms: u64,
    pub low_vol_dwell_ms: u64,
    pub ratchet_peak_frac: f64,
    pub deep_giveback_frac: f64,
    pub deep_giveback_dwell_scale: f64,
    pub protocol: String,
    /// Observational floor-lock twin (authority 0%): exit at protect floor
    /// immediately — no soft-breach dwell / HARD_FLOOR bleed.
    /// Stamp: `MFE_FLOOR_LOCK_AT_PROTECT`. Default false → PRIMARY/BASELINE/r50 unchanged.
    #[serde(default)]
    pub floor_lock_immediate: bool,
}

impl Default for Gate2xParams {
    fn default() -> Self {
        Self {
            enabled: true,
            arm_min_tp_frac: 0.40,
            min_hold_s_before_exit: 120.0,
            meaningful_floor_bps: 5.0,
            protect_floor: ProtectFloorMode::CostClear,
            reversal_volume_confirm: true,
            immediate_vol_surges: vec!["HIGH".into()],
            med_vol_dwell_ms: 40_000,
            low_vol_dwell_ms: 90_000,
            ratchet_peak_frac: 0.30,
            deep_giveback_frac: 0.50,
            deep_giveback_dwell_scale: 0.40,
            protocol: "RP-608-GATE2X-MFE-GIVEBACK-001".into(),
            floor_lock_immediate: false,
        }
    }
}

/// Named twin / book presets (R4). Python picks an id; Rust owns geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gate2xPresetId {
    /// PRIMARY / BASELINE — ratchet 0.30, floor_lock off.
    BaselineR30,
    /// Observational r50 booking twin — ratchet 0.50, floor_lock off.
    R50Booking,
    /// Observational floor-lock twin — ratchet 0.50 + immediate protect exit.
    FloorLockR50,
}

impl Gate2xPresetId {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "baseline_r30" | "baseline" | "primary" | "r30" => Some(Self::BaselineR30),
            "r50_booking" | "r50" | "gate2x_r50" => Some(Self::R50Booking),
            "floor_lock_r50" | "floor_lock" | "floorlock" | "gate2x_floor_lock" => {
                Some(Self::FloorLockR50)
            }
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::BaselineR30 => "baseline_r30",
            Self::R50Booking => "r50_booking",
            Self::FloorLockR50 => "floor_lock_r50",
        }
    }
}

impl Gate2xParams {
    pub fn crypto_defaults() -> Self {
        Self {
            protocol: "RP-608-GATE2X-CRYPTO-MFE-GIVEBACK-001".into(),
            ..Self::default()
        }
    }

    pub fn nse_defaults() -> Self {
        Self {
            protocol: "RP-608-GATE2X-NSE-MFE-GIVEBACK-001".into(),
            ..Self::default()
        }
    }

    /// Book-kind base (PRIMARY geometry) before twin preset overlays.
    pub fn book_defaults(book: &str) -> Self {
        match book.trim().to_ascii_lowercase().as_str() {
            "nse" => Self::nse_defaults(),
            _ => Self::crypto_defaults(),
        }
    }

    /// Apply a named preset onto a book base (currency-agnostic).
    pub fn from_preset(book: &str, preset: Gate2xPresetId) -> Self {
        let base = Self::book_defaults(book);
        match preset {
            Gate2xPresetId::BaselineR30 => Self {
                ratchet_peak_frac: 0.30,
                floor_lock_immediate: false,
                ..base
            },
            Gate2xPresetId::R50Booking => Self::r50_booking_defaults(base),
            Gate2xPresetId::FloorLockR50 => Self::floor_lock_twin_defaults(base),
        }
    }

    /// r50 booking twin (observational; authority 0%).
    pub fn r50_booking_defaults(base: Self) -> Self {
        Self {
            floor_lock_immediate: false,
            ratchet_peak_frac: 0.50,
            protocol: "RP-608-GATE2X-R50-BOOKING-TWIN-001".into(),
            ..base
        }
    }

    /// Floor-lock twin preset (crypto / NSE share geometry; currency-agnostic).
    ///
    /// H11 (2026-11-25): `min_hold_s_before_exit=0` — a 120s min-hold let peak MFE
    /// bleed through protect into a loss, then stamped `MFE_FLOOR_LOCK_AT_PROTECT` red.
    /// Twin intent is lock-at-protect while still green; PRIMARY/BASELINE keep 120s.
    pub fn floor_lock_twin_defaults(base: Self) -> Self {
        Self {
            floor_lock_immediate: true,
            ratchet_peak_frac: 0.50,
            min_hold_s_before_exit: 0.0,
            protocol: "RP-608-GATE2X-FLOOR-LOCK-TWIN-001".into(),
            ..base
        }
    }
}

/// Tick/bar feature snapshot for one open position (no decision authority).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate2xFeatureSnapshot {
    pub decision_id: String,
    pub instrument: String,
    pub direction: String,
    pub reference_price: Option<f64>,
    /// Peak favorable excursion in currency units (USDT / INR).
    pub peak_mfe: f64,
    /// Current favorable excursion at mark.
    pub current_mfe: f64,
    /// Peak progress toward profit target \[0, …\].
    pub peak_tp_frac: f64,
    /// Hold time in seconds since entry.
    pub hold_s: f64,
    /// Arm threshold (friction + meaningful floor) in currency units.
    pub arm_threshold: f64,
    /// Whether position already armed on a prior tick.
    pub already_armed: bool,
    pub vol_surge: VolSurgeBucket,
    /// Soft-breach start epoch ms; None if not currently soft-breaching.
    pub soft_breach_since_ms: Option<i64>,
    pub now_ms: i64,
    pub params: Gate2xParams,
    /// Training-fold volatility median for Coralys volatility label.
    #[serde(default = "default_train_vol_median")]
    pub train_vol_median: f64,
    /// Gate 5 Adaptive TP: number of post-open TP extensions applied.
    #[serde(default)]
    pub tp_level: u32,
    /// True after at least one adaptive TP extension (runner armed).
    #[serde(default)]
    pub adaptive_armed: bool,
}

fn default_train_vol_median() -> f64 {
    0.001
}

/// Soft-breach / arm state returned to the executor for persistence.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Gate2xSoftBreachState {
    pub soft_breach_since_ms: Option<i64>,
    pub soft_breach_dwell_ms: Option<u64>,
    pub soft_breach_vol_surge: Option<String>,
    pub giveback_frac: Option<f64>,
    pub protect_floor: Option<f64>,
    pub ratchet_floor: Option<f64>,
    pub armed: bool,
    pub protocol: Option<String>,
}
