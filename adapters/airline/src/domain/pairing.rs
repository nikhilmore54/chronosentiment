//! Pairing — a sequence of duties forming a self-contained crew trip.
//!
//! A [`Pairing`] begins and ends at the same base airport and covers one or
//! more consecutive [`Duty`] periods separated by rest periods.  It is the
//! unit that crew planning systems typically optimise over.
//!
//! This module defines **structure and relationships only**.  Legality checks
//! (minimum rest between duties, maximum pairing length, etc.) belong to
//! Layer 2.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

use super::duty::{Duty, DutyId};
use super::flight::AirportCode;

// ── Identifier ────────────────────────────────────────────────────────────────

/// Opaque identifier for a [`Pairing`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PairingId(String);

impl PairingId {
    /// Create a new [`PairingId`].
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Return the inner string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PairingId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// ── Core entity ───────────────────────────────────────────────────────────────

/// An ordered, non-empty sequence of duties forming a crew pairing.
///
/// # Invariants (enforced at construction)
/// - Contains at least one duty.
/// - Duties are ordered by start time (no duty starts before the previous ends).
/// - The pairing starts and ends at the same airport (`base`).
///
/// Rest periods between duties are implicit (the gap between one duty's end
/// and the next duty's start).  The legality layer checks that these gaps
/// satisfy minimum rest requirements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pairing {
    /// Unique identifier.
    pub id: PairingId,
    /// The base airport where this pairing starts and ends.
    pub base: AirportCode,
    /// Ordered sequence of duties.
    duties: Vec<Duty>,
}

impl Pairing {
    /// Construct a new [`Pairing`].
    ///
    /// # Errors
    /// Returns [`PairingError`] if:
    /// - `duties` is empty.
    /// - A duty starts before the previous duty ends.
    /// - The pairing does not start at `base`.
    /// - The pairing does not end at `base`.
    pub fn new(id: PairingId, base: AirportCode, duties: Vec<Duty>) -> Result<Self, PairingError> {
        if duties.is_empty() {
            return Err(PairingError::Empty);
        }

        // Check chronological ordering.
        for i in 1..duties.len() {
            if duties[i].start() < duties[i - 1].end() {
                return Err(PairingError::OutOfOrder {
                    earlier: duties[i - 1].id.clone(),
                    later: duties[i].id.clone(),
                });
            }
        }

        // Check that the pairing starts at base.
        let first_station = duties[0].report_station().clone();
        if first_station != base {
            return Err(PairingError::DoesNotStartAtBase {
                base: base.clone(),
                actual: first_station,
            });
        }

        // Check that the pairing ends at base.
        let last_station = duties[duties.len() - 1].release_station().clone();
        if last_station != base {
            return Err(PairingError::DoesNotEndAtBase {
                base: base.clone(),
                actual: last_station,
            });
        }

        Ok(Self { id, base, duties })
    }

    /// The ordered duties in this pairing.
    pub fn duties(&self) -> &[Duty] {
        &self.duties
    }

    /// Number of duties in this pairing.
    pub fn duty_count(&self) -> usize {
        self.duties.len()
    }

    /// Start of the pairing (start of the first duty).
    pub fn start(&self) -> DateTime<Utc> {
        self.duties[0].start()
    }

    /// End of the pairing (end of the last duty).
    pub fn end(&self) -> DateTime<Utc> {
        self.duties[self.duties.len() - 1].end()
    }

    /// Total elapsed time from start of first duty to end of last duty.
    pub fn elapsed_time(&self) -> Duration {
        self.end() - self.start()
    }

    /// Sum of block times across all duties.
    pub fn total_block_time(&self) -> Duration {
        self.duties.iter().map(|d| d.total_block_time()).sum()
    }

    /// Total number of flight legs across all duties.
    pub fn total_leg_count(&self) -> usize {
        self.duties.iter().map(|d| d.leg_count()).sum()
    }

    /// Rest periods between consecutive duties.
    ///
    /// Returns a `Vec` of `(duty_index, rest_duration)` pairs where
    /// `duty_index` is the index of the duty *after* the rest period.
    /// The legality layer uses these to check minimum rest requirements.
    pub fn rest_periods(&self) -> Vec<(usize, Duration)> {
        (1..self.duties.len())
            .map(|i| {
                let rest = self.duties[i].start() - self.duties[i - 1].end();
                (i, rest)
            })
            .collect()
    }

    /// Decompose a multi-duty pairing into smaller valid sub-pairings if it visits its home base internally.
    ///
    /// If the pairing returns to its home base at the end of any intermediate duty, it can be cleanly partitioned
    /// into independent out-and-back trips that each start and end at the home base.
    /// Returns the sub-pairings if a valid split was made, or a single-element vector containing the original pairing if not.
    pub fn decompose_at_base_visits(&self) -> Vec<Pairing> {
        if self.duties.len() <= 1 {
            return vec![self.clone()];
        }

        let mut sub_pairings = Vec::new();
        let mut current_duties = Vec::new();

        for duty in &self.duties {
            current_duties.push(duty.clone());
            if duty.release_station() == &self.base {
                let sub_id = PairingId::new(format!("{}_sub{}", self.id.as_str(), sub_pairings.len()));
                if let Ok(sub) = Pairing::new(sub_id, self.base.clone(), current_duties.clone()) {
                    sub_pairings.push(sub);
                    current_duties.clear();
                }
            }
        }

        if !current_duties.is_empty() || sub_pairings.is_empty() {
            vec![self.clone()]
        } else {
            sub_pairings
        }
    }

    /// Prorated block time counting only flight legs that depart within the declared planning period.
    ///
    /// Implements the Month-End Carry-Over Rule: flight legs occurring past midnight on the last day of the
    /// month roll into next month's schedule and are not penalized against the current month's 85.0h cap.
    pub fn prorated_block_time(&self, period: &crate::domain::roster::PlanningPeriod) -> Duration {
        self.duties
            .iter()
            .flat_map(|d| d.legs())
            .filter(|leg| leg.scheduled_departure >= period.start && leg.scheduled_departure < period.end)
            .map(|leg| leg.block_time())
            .sum()
    }

    /// Retrieve all carry-over flight legs that roll over past the end of the declared planning period.
    pub fn carry_over_legs(&self, period: &crate::domain::roster::PlanningPeriod) -> Vec<crate::domain::flight::FlightLeg> {
        self.duties
            .iter()
            .flat_map(|d| d.legs())
            .filter(|leg| leg.scheduled_departure >= period.end)
            .cloned()
            .collect()
    }

    /// Split a multi-duty pairing at an intermediate hub layover into two independent sub-pairings.
    ///
    /// This enables two pilots with partial remaining headroom (e.g. 12-15 hours each) to share
    /// a 4-day monolithic trip that would otherwise exceed an individual's 85h cap.
    pub fn split_at_hub_layover(&self, hub: &AirportCode) -> Option<(Pairing, Pairing)> {
        if self.duties.len() < 2 {
            return None;
        }

        for split_idx in 1..self.duties.len() {
            let left_duties = self.duties[..split_idx].to_vec();
            let right_duties = self.duties[split_idx..].to_vec();

            let left_end = left_duties.last().unwrap().release_station();
            let right_start = right_duties.first().unwrap().report_station();

            if left_end == hub && right_start == hub {
                let id1 = PairingId::new(format!("{}_pt1", self.id.as_str()));
                let id2 = PairingId::new(format!("{}_pt2", self.id.as_str()));

                let p1 = Pairing::new(id1, self.base.clone(), left_duties).ok()?;
                let p2 = Pairing::new(id2, hub.clone(), right_duties).ok()?;
                return Some((p1, p2));
            }
        }
        None
    }

    /// Evaluates whether a month-end pairing should follow standard airline carry-over (keeping crew intact)
    /// or fallback to a hub layover split.
    ///
    /// # Standard Practice (Default):
    /// Keeps the crew intact across the Jan 31 / Feb 1 boundary. If `current_january_block + prorated_block <= 85.0h`,
    /// the crew stays with the trip into February. Hours before 23:59 UTC count against January, while rollover legs
    /// are exported as a [`CarryInLock`] pre-assigned activity for February.
    ///
    /// # Exception Practice (Fallback):
    /// If the assigned pilot has zero January headroom left, splits at a hub layover so Pilot A is released at the hub
    /// and a fresh February Reserve Pilot picks up the aircraft on Feb 1.
    pub fn evaluate_month_end_transition(
        &self,
        crew_id: &crate::domain::crew::CrewId,
        current_january_block: Duration,
        period: &crate::domain::roster::PlanningPeriod,
        hub_candidates: &[AirportCode],
    ) -> Option<RolloverDecision> {
        let carry_legs = self.carry_over_legs(period);
        if carry_legs.is_empty() {
            return None;
        }

        let jan_block = self.prorated_block_time(period);
        let max_jan_limit = Duration::hours(85);

        // 1. Standard Practice: Default to Rollover (Keep crew intact)
        if current_january_block + jan_block <= max_jan_limit {
            let carry_block: Duration = carry_legs.iter().map(|l| l.block_time()).sum();
            let start = carry_legs.first().unwrap().scheduled_departure;
            let end = carry_legs.last().unwrap().scheduled_arrival;

            let carry_in_lock = CarryInLock {
                crew_id: crew_id.clone(),
                parent_pairing_id: self.id.clone(),
                carry_in_legs: carry_legs,
                carry_in_block_time: carry_block,
                start_time: start,
                end_time: end,
            };

            return Some(RolloverDecision::KeepCrewRollover {
                crew_id: crew_id.clone(),
                january_block_time: jan_block,
                carry_in_lock,
            });
        }

        // 2. Exception Fallback: Hub split when January headroom is exhausted
        for hub in hub_candidates {
            if let Some((p1, p2)) = self.split_at_hub_layover(hub) {
                return Some(RolloverDecision::HubSplitFallback {
                    hub: hub.clone(),
                    pilot_a_crew_id: crew_id.clone(),
                    pilot_a_pairing: p1,
                    february_reserve_crew_id: None,
                    february_pairing: p2,
                });
            }
        }

        None
    }
}

/// Pre-assigned carry-in activity created by a month-end transition / rollover pairing.
///
/// Billed against the subsequent month's 85-hour statutory budget as an immutable pre-assigned activity
/// before monthly preferential bidding opens.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CarryInLock {
    /// Crew member operating the rollover flight legs into next month.
    pub crew_id: crate::domain::crew::CrewId,
    /// Parent pairing identifier originating in the previous month.
    pub parent_pairing_id: PairingId,
    /// Specific flight legs operating in the subsequent month.
    pub carry_in_legs: Vec<crate::domain::flight::FlightLeg>,
    /// Total block time of the carry-in legs to be charged against next month's cap.
    pub carry_in_block_time: Duration,
    /// Scheduled start of the first carry-in leg.
    pub start_time: DateTime<Utc>,
    /// Scheduled arrival of the final carry-in leg.
    pub end_time: DateTime<Utc>,
}

/// Operational decision outcome for month-end boundary crossings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RolloverDecision {
    /// Standard airline practice: keep crew intact across the boundary, proration of January block hours,
    /// and generate a pre-assigned carry-in lock for the next month.
    KeepCrewRollover {
        crew_id: crate::domain::crew::CrewId,
        january_block_time: Duration,
        carry_in_lock: CarryInLock,
    },
    /// Fallback practice: when the assigned pilot hits a hard 85h or 6-day ceiling on Jan 31,
    /// split at the hub layover so a fresh 0-hour reserve pilot picks up the February legs.
    HubSplitFallback {
        hub: AirportCode,
        pilot_a_crew_id: crate::domain::crew::CrewId,
        pilot_a_pairing: Pairing,
        february_reserve_crew_id: Option<crate::domain::crew::CrewId>,
        february_pairing: Pairing,
    },
}

/// A multi-trip outstation tour where a pilot is positioned to an understaffed base,
/// operates multiple consecutive pairings with outstation hotel rest, and returns via a single inbound positioning.
///
/// Reduces positioning deadhead flight credit by 50%–65% compared to separate single-pairing deadhead roundtrips.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutstationTour {
    /// Home base of the pilot being deployed.
    pub home_base: AirportCode,
    /// Understaffed outstation base where pairings are operated.
    pub outstation_base: AirportCode,
    /// Outbound deadhead duty from home_base to outstation_base.
    pub outbound_deadhead: Duty,
    /// Consecutive pairings operated out of outstation_base.
    pub station_pairings: Vec<Pairing>,
    /// Inbound deadhead duty returning from outstation_base to home_base.
    pub inbound_deadhead: Duty,
}

impl OutstationTour {
    /// Construct a new [`OutstationTour`].
    pub fn new(
        home_base: AirportCode,
        outstation_base: AirportCode,
        outbound_deadhead: Duty,
        station_pairings: Vec<Pairing>,
        inbound_deadhead: Duty,
    ) -> Result<Self, PairingError> {
        if station_pairings.is_empty() {
            return Err(PairingError::Empty);
        }
        Ok(Self {
            home_base,
            outstation_base,
            outbound_deadhead,
            station_pairings,
            inbound_deadhead,
        })
    }

    /// Convert the multi-trip outstation tour into a single unified composite pairing based at `home_base`.
    ///
    /// This composite pairing passes directly through standard Layer 2 legality checks (CAR 700 rest, FDP, 85h cap).
    pub fn to_composite_pairing(&self, id: PairingId) -> Result<Pairing, PairingError> {
        let mut duties = Vec::new();
        duties.push(self.outbound_deadhead.clone());
        for p in &self.station_pairings {
            for d in p.duties() {
                duties.push(d.clone());
            }
        }
        duties.push(self.inbound_deadhead.clone());
        Pairing::new(id, self.home_base.clone(), duties)
    }

    /// Number of round-trip deadheads saved by bundling N pairings into 1 tour.
    pub fn deadhead_savings_count(&self) -> usize {
        if self.station_pairings.len() <= 1 {
            0
        } else {
            (self.station_pairings.len() - 1) * 2
        }
    }
}

// ── Error type ────────────────────────────────────────────────────────────────

/// Errors that can occur when constructing a [`Pairing`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PairingError {
    /// The duty sequence was empty.
    #[error("a pairing must contain at least one duty")]
    Empty,

    /// A duty starts before the previous duty ends.
    #[error("duty {later} starts before duty {earlier} ends")]
    OutOfOrder { earlier: DutyId, later: DutyId },

    /// The first duty does not start at the declared base.
    #[error("pairing base is {base} but first duty starts at {actual}")]
    DoesNotStartAtBase {
        base: AirportCode,
        actual: AirportCode,
    },

    /// The last duty does not end at the declared base.
    #[error("pairing base is {base} but last duty ends at {actual}")]
    DoesNotEndAtBase {
        base: AirportCode,
        actual: AirportCode,
    },
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::duty::DutyId;
    use crate::domain::flight::{AircraftType, FlightLeg, FlightLegId, FlightNumber};
    use chrono::TimeZone;

    fn make_leg(id: &str, origin: &str, dest: &str, dep_h: u32, arr_h: u32) -> FlightLeg {
        let base = Utc.with_ymd_and_hms(2026, 7, 1, 0, 0, 0).unwrap();
        FlightLeg::new(
            FlightLegId::new(id),
            FlightNumber::new(format!("XX{id}")),
            AirportCode::new(origin),
            AirportCode::new(dest),
            base + Duration::hours(dep_h as i64),
            base + Duration::hours(arr_h as i64),
            AircraftType::new("B738"),
        )
    }

    fn make_duty(id: &str, legs: Vec<FlightLeg>) -> Duty {
        Duty::new(DutyId::new(id), legs).unwrap()
    }

    /// LHR→CDG (8–10), rest, CDG→LHR (14–16)
    fn two_duty_pairing() -> Pairing {
        let d1 = make_duty("D1", vec![make_leg("L1", "LHR", "CDG", 8, 10)]);
        let d2 = make_duty("D2", vec![make_leg("L2", "CDG", "LHR", 14, 16)]);
        Pairing::new(PairingId::new("P1"), AirportCode::new("LHR"), vec![d1, d2]).unwrap()
    }

    #[test]
    fn single_duty_pairing_is_valid() {
        let d = make_duty("D1", vec![make_leg("L1", "LHR", "LHR", 8, 10)]);
        let p = Pairing::new(PairingId::new("P1"), AirportCode::new("LHR"), vec![d]).unwrap();
        assert_eq!(p.duty_count(), 1);
        assert_eq!(p.total_leg_count(), 1);
    }

    #[test]
    fn two_duty_pairing_metrics() {
        let p = two_duty_pairing();
        assert_eq!(p.duty_count(), 2);
        assert_eq!(p.total_leg_count(), 2);
        // elapsed = 16h - 8h = 8h
        assert_eq!(p.elapsed_time(), Duration::hours(8));
        // block = 2h + 2h = 4h
        assert_eq!(p.total_block_time(), Duration::hours(4));
    }

    #[test]
    fn rest_periods_are_correct() {
        let p = two_duty_pairing();
        let rests = p.rest_periods();
        assert_eq!(rests.len(), 1);
        let (idx, rest) = rests[0];
        assert_eq!(idx, 1);
        // D1 ends at 10h, D2 starts at 14h → 4h rest
        assert_eq!(rest, Duration::hours(4));
    }

    #[test]
    fn rejects_empty_duties() {
        let err = Pairing::new(PairingId::new("P1"), AirportCode::new("LHR"), vec![]).unwrap_err();
        assert_eq!(err, PairingError::Empty);
    }

    #[test]
    fn rejects_out_of_order_duties() {
        let d1 = make_duty("D1", vec![make_leg("L1", "LHR", "CDG", 14, 16)]);
        let d2 = make_duty("D2", vec![make_leg("L2", "CDG", "LHR", 8, 10)]); // before d1
        let err =
            Pairing::new(PairingId::new("P1"), AirportCode::new("LHR"), vec![d1, d2]).unwrap_err();
        assert!(matches!(err, PairingError::OutOfOrder { .. }));
    }

    #[test]
    fn rejects_wrong_start_base() {
        let d = make_duty("D1", vec![make_leg("L1", "CDG", "LHR", 8, 10)]);
        let err = Pairing::new(PairingId::new("P1"), AirportCode::new("LHR"), vec![d]).unwrap_err();
        assert!(matches!(err, PairingError::DoesNotStartAtBase { .. }));
    }

    #[test]
    fn rejects_wrong_end_base() {
        let d = make_duty("D1", vec![make_leg("L1", "LHR", "CDG", 8, 10)]);
        let err = Pairing::new(PairingId::new("P1"), AirportCode::new("LHR"), vec![d]).unwrap_err();
        assert!(matches!(err, PairingError::DoesNotEndAtBase { .. }));
    }

    #[test]
    fn serde_round_trip() {
        let p = two_duty_pairing();
        let json = serde_json::to_string(&p).unwrap();
        let restored: Pairing = serde_json::from_str(&json).unwrap();
        assert_eq!(p, restored);
    }

    #[test]
    fn decompose_at_base_visits_works() {
        let d1 = make_duty("D1", vec![
            make_leg("L1", "LHR", "CDG", 8, 10),
            make_leg("L2", "CDG", "LHR", 12, 14),
        ]);
        let d2 = make_duty("D2", vec![
            make_leg("L3", "LHR", "FRA", 20, 22),
            make_leg("L4", "FRA", "LHR", 24, 26),
        ]);
        let monolithic = Pairing::new(
            PairingId::new("MONO"),
            AirportCode::new("LHR"),
            vec![d1, d2],
        ).unwrap();

        let subs = monolithic.decompose_at_base_visits();
        assert_eq!(subs.len(), 2);
        assert_eq!(subs[0].id.as_str(), "MONO_sub0");
        assert_eq!(subs[0].duties().len(), 1);
        assert_eq!(subs[0].base.as_str(), "LHR");
        assert_eq!(subs[1].id.as_str(), "MONO_sub1");
        assert_eq!(subs[1].duties().len(), 1);
        assert_eq!(subs[1].base.as_str(), "LHR");
    }

    #[test]
    fn test_outstation_tour_composite_and_savings() {
        let outbound = make_duty("DH_OUT", vec![make_leg("L_DH1", "LHR", "CDG", 6, 8)]);
        let p_station1 = Pairing::new(
            PairingId::new("ST1"),
            AirportCode::new("CDG"),
            vec![make_duty("D_ST1", vec![
                make_leg("L_S1", "CDG", "NCE", 12, 14),
                make_leg("L_S2", "NCE", "CDG", 16, 18),
            ])],
        ).unwrap();
        let p_station2 = Pairing::new(
            PairingId::new("ST2"),
            AirportCode::new("CDG"),
            vec![make_duty("D_ST2", vec![
                make_leg("L_S3", "CDG", "TLS", 36, 38),
                make_leg("L_S4", "TLS", "CDG", 40, 42),
            ])],
        ).unwrap();
        let inbound = make_duty("DH_IN", vec![make_leg("L_DH2", "CDG", "LHR", 56, 58)]);

        let tour = OutstationTour::new(
            AirportCode::new("LHR"),
            AirportCode::new("CDG"),
            outbound,
            vec![p_station1, p_station2],
            inbound,
        ).unwrap();

        assert_eq!(tour.deadhead_savings_count(), 2);
        let composite = tour.to_composite_pairing(PairingId::new("COMPOSITE_TOUR")).unwrap();
        assert_eq!(composite.base.as_str(), "LHR");
        assert_eq!(composite.duty_count(), 4);
    }

    #[test]
    fn test_carry_over_and_hub_split() {
        use crate::domain::roster::PlanningPeriod;
        let base_dt = Utc.with_ymd_and_hms(2026, 7, 1, 0, 0, 0).unwrap();
        let period = PlanningPeriod::new(base_dt, base_dt + Duration::hours(20));

        let d1 = make_duty("D1", vec![
            make_leg("L1", "LHR", "FRA", 8, 10),
            make_leg("L2", "FRA", "LHR", 12, 14),
        ]);
        let d2 = make_duty("D2", vec![
            make_leg("L3", "LHR", "FRA", 22, 24),
            make_leg("L4", "FRA", "LHR", 26, 28),
        ]);
        let p = Pairing::new(PairingId::new("P_SPLIT"), AirportCode::new("LHR"), vec![d1, d2]).unwrap();

        // Leg 1 & 2 are in period (< 20h), Leg 3 & 4 are carry-over (>= 20h)
        assert_eq!(p.prorated_block_time(&period), Duration::hours(4));
        let carry = p.carry_over_legs(&period);
        assert_eq!(carry.len(), 2);
        assert_eq!(carry[0].id.as_str(), "L3");
        assert_eq!(carry[1].id.as_str(), "L4");

        // Hub split at LHR
        let (pt1, pt2) = p.split_at_hub_layover(&AirportCode::new("LHR")).unwrap();
        assert_eq!(pt1.duty_count(), 1);
        assert_eq!(pt2.duty_count(), 1);
    }
}
