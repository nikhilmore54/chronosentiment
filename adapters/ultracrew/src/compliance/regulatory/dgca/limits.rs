/// DGCA FDTL regulatory limits.
///
/// Source: CAR Section 7 Series J Part III (Flight Crew FDTL) and
///         Cabin Crew FDTL CAR 2018.
///
/// All hour values are in hours unless otherwise noted.
/// These are the *default* regulatory minima/maxima; an airline may apply
/// more restrictive values via `DgcaLimits::builder()`.

#[derive(Debug, Clone)]
pub struct DgcaLimits {
    // ── Rest ─────────────────────────────────────────────────────────────────
    /// Minimum rest period between two consecutive FDPs (§6.1).
    /// Default: 12 h for flight crew; 10 h for cabin crew (we use the stricter).
    pub min_rest_hours: u64,

    // ── Flight Duty Period ────────────────────────────────────────────────────
    /// Maximum FDP for a single-pilot operation (§5.1 basic limit).
    /// Default: 11 h (day sector, 2-pilot crew).
    pub max_fdp_hours: u64,

    /// Maximum FDP when augmented crew is carried (§5.3).
    /// Default: 14 h.
    pub max_fdp_augmented_hours: u64,

    // ── Block / Flight Hours ──────────────────────────────────────────────────
    /// Maximum block hours in any rolling 28-day window (§7.1).
    /// Default: 100 h.
    pub max_block_hours_28d: u64,

    /// Maximum block hours in any rolling 365-day window (§7.2).
    /// Default: 1 000 h.
    pub max_block_hours_365d: u64,

    // ── Standby ──────────────────────────────────────────────────────────────
    /// Maximum continuous standby duty (§8.2).
    /// Default: 12 h.
    pub max_standby_hours: u64,

    /// Minimum callout notice before a standby crew member must report (§8.3).
    /// Default: 2 h.
    pub min_callout_notice_hours: u64,
}

impl DgcaLimits {
    /// Regulatory defaults as published in CAR Section 7 Series J Pt III.
    pub fn regulatory_defaults() -> Self {
        DgcaLimits {
            min_rest_hours: 12,
            max_fdp_hours: 11,
            max_fdp_augmented_hours: 14,
            max_block_hours_28d: 100,
            max_block_hours_365d: 1_000,
            max_standby_hours: 12,
            min_callout_notice_hours: 2,
        }
    }

    /// Builder: override minimum rest (must be ≥ regulatory default).
    pub fn with_min_rest(mut self, hours: u64) -> Self {
        self.min_rest_hours = hours;
        self
    }

    /// Builder: override maximum FDP (must be ≤ regulatory default).
    pub fn with_max_fdp(mut self, hours: u64) -> Self {
        self.max_fdp_hours = hours;
        self
    }

    /// Builder: override 28-day block hour cap.
    pub fn with_max_block_28d(mut self, hours: u64) -> Self {
        self.max_block_hours_28d = hours;
        self
    }

    /// Builder: override 365-day block hour cap.
    pub fn with_max_block_365d(mut self, hours: u64) -> Self {
        self.max_block_hours_365d = hours;
        self
    }
}