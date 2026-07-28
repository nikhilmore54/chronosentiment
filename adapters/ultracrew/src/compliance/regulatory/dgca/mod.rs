/// DGCA Flight Duty Time Limitations Compliance Pack
///
/// Assembles all DGCA FDTL rules into a single [`CompliancePack`] that can be
/// installed into a [`ComplianceRegistry`] at optimizer startup.
///
/// Rules included:
///   dgca.fdtl.minimum_rest         — CAR S7 J III §6.1  (Hard)
///   dgca.fdtl.maximum_fdp          — CAR S7 J III §5.1  (Hard)
///   dgca.fdtl.max_block_hours_28d  — CAR S7 J III §7.1  (Hard)
///   dgca.fdtl.max_block_hours_365d — CAR S7 J III §7.2  (Hard)
///   dgca.fdtl.standby_limits       — CAR S7 J III §8.2-3 (Hard)

pub mod limits;
pub mod minimum_rest;
pub mod maximum_fdp;
pub mod flight_hours;
pub mod standby;

use limits::DgcaLimits;
use minimum_rest::MinimumRestRule;
use maximum_fdp::MaximumFdpRule;
use flight_hours::{MaxFlightHours28DaysRule, MaxFlightHours365DaysRule};
use standby::StandbyRule;

use crate::compliance::traits::CompliancePack;
use crate::compliance::registry::ComplianceRegistry;
use crate::compliance::metadata::ComplianceDescriptor;

/// The DGCA FDTL compliance pack.
///
/// Construct with regulatory defaults:
/// ```rust
/// let pack = DgcaCompliancePack::default();
/// ```
/// Or with custom limits (e.g. airline applies stricter rest than regulatory minimum):
/// ```rust
/// let limits = DgcaLimits::regulatory_defaults().with_min_rest(14);
/// let pack = DgcaCompliancePack::with_limits(limits);
/// ```
pub struct DgcaCompliancePack {
    limits: DgcaLimits,
}

impl DgcaCompliancePack {
    pub fn with_limits(limits: DgcaLimits) -> Self {
        DgcaCompliancePack { limits }
    }
}

impl Default for DgcaCompliancePack {
    fn default() -> Self {
        DgcaCompliancePack {
            limits: DgcaLimits::regulatory_defaults(),
        }
    }
}

impl CompliancePack for DgcaCompliancePack {
    fn descriptor(&self) -> ComplianceDescriptor {
        ComplianceDescriptor::new(
            "dgca-fdtl-2024",
            "DGCA FDTL (CAR Section 7 Series J Part III)",
            "DGCA",
            "2024.1",
        )
        .with_effective_from("2024-01-01")
    }

    fn load_into(&self, registry: &mut ComplianceRegistry) {
        registry.register(MinimumRestRule::new(self.limits.clone()));
        registry.register(MaximumFdpRule::new(self.limits.clone()));
        registry.register(MaxFlightHours28DaysRule::new(self.limits.clone()));
        registry.register(MaxFlightHours365DaysRule::new(self.limits.clone()));
        registry.register(StandbyRule::new(self.limits.clone()));
    }
}

// Backward-compatible alias — remove once all call sites are updated.
pub type DgcaRulePack = DgcaCompliancePack;