
#[cfg(test)]
mod tests {
    use super::*;
    use axum::Json;
    use ultracrew::public_contracts::{Scenario, SchedulingDomain};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_airline_layover_vs_inrc_rest() {
        let mut schedule = HashMap::new();
        schedule.insert(1, 1);
        schedule.insert(2, 1);

        let shifts = vec![
            ShiftInput { id: 1, start_hour: 0, duration_hours: 8, required_skill: "Captain".to_string() },
            ShiftInput { id: 2, start_hour: 23, duration_hours: 8, required_skill: "Captain".to_string() },
        ];
        
        let workers = vec![
            WorkerInput { id: 1, skills: vec!["Captain".to_string()], max_hours: None },
        ];

        // 1. INRC Domain -> should be rejected because pairings/layovers don't exist
        let inrc_req = ScheduleAnalysisRequest {
            schedule: schedule.clone(),
            shifts: shifts.clone(),
            workers: workers.clone(),
            scenario: Some(Scenario {
                domain: Some(SchedulingDomain::Inrc),
                planning_horizon_hours: None,
                max_hours_per_worker: None,
                minimum_rest_hours: None,
                leave_requests: None,
            }),
        };

        let inrc_res = pairings_handler(Json(inrc_req)).await;
        assert!(inrc_res.is_err());
        let (status, msg) = inrc_res.unwrap_err();
        assert_eq!(status, axum::http::StatusCode::UNPROCESSABLE_ENTITY);
        assert!(msg.contains("DOMAIN_CONCEPT_NOT_SUPPORTED"));

        // 2. Airline Domain -> should construct FDPs and Layover
        let airline_req = ScheduleAnalysisRequest {
            schedule,
            shifts,
            workers,
            scenario: Some(Scenario {
                domain: Some(SchedulingDomain::Airline),
                planning_horizon_hours: None,
                max_hours_per_worker: None,
                minimum_rest_hours: None,
                leave_requests: None,
            }),
        };

        let airline_res = pairings_handler(Json(airline_req)).await.unwrap().0;
        let pairings = airline_res.pairings;
        assert_eq!(pairings.len(), 1, "Should group into one pairing");
        let p = &pairings[0];
        
        // 2 FDPs separated by a layover
        assert_eq!(p.fdp_count, 2);
        
        // FDP 1: 0 to 8, gap: 15h
        // FDP 2: 23 to 31
        // gap of 15h >= 8h layover threshold and < 34h home base rest
        assert!(p.total_layover_hours > 0.0);
        assert_eq!(p.total_layover_hours, 15.0);
    }
}
