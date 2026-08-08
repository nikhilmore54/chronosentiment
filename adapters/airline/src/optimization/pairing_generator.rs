use chrono::Duration;
use crate::domain::duty::{Duty, DutyId};
use crate::domain::flight::{AirportCode, FlightLeg};
use crate::domain::pairing::{Pairing, PairingId};

pub struct PairingGenerator {
    pub min_sit_mins: i64,
    pub max_sit_mins: i64,
    pub max_fdp_mins: i64,
    pub min_rest_mins: i64,
    pub max_tafb_mins: i64,
    pub max_duties_per_pairing: usize,
}

impl Default for PairingGenerator {
    fn default() -> Self {
        Self {
            min_sit_mins: 45,
            max_sit_mins: 4 * 60,
            max_fdp_mins: 13 * 60,
            min_rest_mins: 10 * 60,
            max_tafb_mins: 96 * 60,
            max_duties_per_pairing: 5,
        }
    }
}

impl PairingGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Generate all valid duties from the given flight schedule.
    pub fn generate_duties(&self, flights: &[FlightLeg]) -> Vec<Duty> {
        let mut sorted_flights = flights.to_vec();
        sorted_flights.sort_by_key(|f| f.scheduled_departure);

        let mut valid_duties = Vec::new();

        for i in 0..sorted_flights.len() {
            let mut current_legs = vec![sorted_flights[i].clone()];
            self.dfs_duty(&sorted_flights, i, &mut current_legs, &mut valid_duties);
        }

        // Convert generated legs into Duty structs
        let mut duties = Vec::new();
        for (idx, legs) in valid_duties.into_iter().enumerate() {
            if let Ok(duty) = Duty::new(DutyId::new(format!("D{}", idx)), legs) {
                duties.push(duty);
            }
        }
        duties
    }

    fn dfs_duty(
        &self,
        flights: &[FlightLeg],
        current_idx: usize,
        current_legs: &mut Vec<FlightLeg>,
        valid_duties: &mut Vec<Vec<FlightLeg>>,
    ) {
        // Every valid partial path is a valid duty
        valid_duties.push(current_legs.clone());

        let last_leg = &flights[current_idx];
        let fdp_start = current_legs[0].scheduled_departure;

        for next_idx in (current_idx + 1)..flights.len() {
            let next_leg = &flights[next_idx];

            // Must depart after the last arrival
            if next_leg.scheduled_departure <= last_leg.scheduled_arrival {
                continue;
            }

            // Connectivity check
            if next_leg.origin != last_leg.destination {
                continue;
            }

            let sit_time = next_leg.scheduled_departure - last_leg.scheduled_arrival;
            let sit_mins = sit_time.num_minutes();

            // Connection time constraints
            if sit_mins < self.min_sit_mins || sit_mins > self.max_sit_mins {
                continue;
            }

            // FDP limits (report to block-off of last leg)
            let fdp_mins = (next_leg.scheduled_departure - fdp_start).num_minutes();
            if fdp_mins > self.max_fdp_mins {
                continue;
            }

            current_legs.push(next_leg.clone());
            self.dfs_duty(flights, next_idx, current_legs, valid_duties);
            current_legs.pop();
        }
    }

    /// Generate all valid pairings originating from the given base airport.
    pub fn generate_pairings(&self, duties: &[Duty], base: &AirportCode) -> Vec<Pairing> {
        let mut sorted_duties = duties.to_vec();
        sorted_duties.sort_by_key(|d| d.start());

        let mut valid_pairings = Vec::new();

        for i in 0..sorted_duties.len() {
            if sorted_duties[i].report_station() == base {
                let mut current_duties = vec![sorted_duties[i].clone()];
                self.dfs_pairing(&sorted_duties, i, base, &mut current_duties, &mut valid_pairings);
            }
        }

        let mut pairings = Vec::new();
        for (idx, duties_seq) in valid_pairings.into_iter().enumerate() {
            if let Ok(pairing) = Pairing::new(PairingId::new(format!("P{}", idx)), base.clone(), duties_seq) {
                pairings.push(pairing);
            }
        }
        pairings
    }

    fn dfs_pairing(
        &self,
        duties: &[Duty],
        current_idx: usize,
        base: &AirportCode,
        current_duties: &mut Vec<Duty>,
        valid_pairings: &mut Vec<Vec<Duty>>,
    ) {
        let last_duty = &duties[current_idx];

        // If the current sequence ends at the base, it forms a valid pairing.
        // We record it but also allow extending it if the user wants multi-return pairings.
        // Typically a pairing ends when returning to base, so we could stop here, but
        // for completeness, we add it to the valid list.
        if last_duty.release_station() == base {
            valid_pairings.push(current_duties.clone());
            // Standard assumption: pairing terminates upon returning to base.
            return;
        }

        if current_duties.len() >= self.max_duties_per_pairing {
            return;
        }

        let pairing_start = current_duties[0].start();

        for next_idx in (current_idx + 1)..duties.len() {
            let next_duty = &duties[next_idx];

            if next_duty.start() <= last_duty.end() {
                continue;
            }

            // Connectivity check
            if next_duty.report_station() != last_duty.release_station() {
                continue;
            }

            let rest_time = next_duty.start() - last_duty.end();
            let rest_mins = rest_time.num_minutes();

            // Minimum rest
            if rest_mins < self.min_rest_mins {
                continue;
            }

            // TAFB limit
            let tafb_mins = (next_duty.end() - pairing_start).num_minutes();
            if tafb_mins > self.max_tafb_mins {
                continue;
            }

            current_duties.push(next_duty.clone());
            self.dfs_pairing(duties, next_idx, base, current_duties, valid_pairings);
            current_duties.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use crate::domain::flight::{AircraftType, FlightLegId, FlightNumber};

    fn make_leg(id: &str, origin: &str, dest: &str, dep_h: u32, arr_h: u32) -> FlightLeg {
        let base_time = Utc.with_ymd_and_hms(2026, 7, 1, 0, 0, 0).unwrap();
        FlightLeg::new(
            FlightLegId::new(id),
            FlightNumber::new(format!("XX{id}")),
            AirportCode::new(origin),
            AirportCode::new(dest),
            base_time + Duration::hours(dep_h as i64),
            base_time + Duration::hours(arr_h as i64),
            AircraftType::new("B738"),
        )
    }

    #[test]
    fn test_duty_generation() {
        let flights = vec![
            make_leg("1", "LHR", "CDG", 8, 10),
            make_leg("2", "CDG", "FRA", 11, 13), // 1h sit (valid)
            make_leg("3", "CDG", "FRA", 15, 17), // 5h sit (invalid, > max 4h)
        ];

        let generator = PairingGenerator::default();
        let duties = generator.generate_duties(&flights);
        // Valid duties:
        // [1]
        // [1, 2]
        // [2]
        // [3]
        assert_eq!(duties.len(), 4);
    }

    #[test]
    fn test_pairing_generation() {
        let flights = vec![
            make_leg("1", "LHR", "CDG", 8, 10),
            make_leg("2", "CDG", "LHR", 22, 24), // 12h rest (valid)
        ];

        let generator = PairingGenerator::default();
        let duties = generator.generate_duties(&flights);
        let pairings = generator.generate_pairings(&duties, &AirportCode::new("LHR"));

        // Duty pool: [1], [2] (sit is 12h > 4h, so no [1, 2] duty)
        // Pairings:
        // [1] -> [2] is a valid pairing since rest is 12h > 10h min_rest.
        assert_eq!(pairings.len(), 1);
        assert_eq!(pairings[0].duties().len(), 2);
    }
}
