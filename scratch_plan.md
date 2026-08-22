# Domain Configuration Refactoring Plan

The recent UC-DOMAIN-001 architectural work successfully isolated the constraint evaluator logic, but the public contract (`Scenario`) still leaks generic INRC concepts (`max_hours_per_worker`, `minimum_rest_hours`) into the Airline domain. We must resolve this before proceeding with any baseline measurements.

## Proposed Changes

### 1. Quarantine Contaminated Datasets
- Update `fixtures/demo/sunair_demo.json` to use the new isolated Airline configuration model.
- Document that any previous dataset relying on `max_hours_per_worker` under the `AIRLINE` domain is invalid.

### 2. Refactor `Scenario` Contract
Modify `adapters/ultracrew/src/public_contracts.rs` to separate domain-specific configurations from the generic contract using a tagged enum or `domain_config` struct.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub domain: SchedulingDomain,
    pub domain_config: DomainConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DomainConfig {
    Inrc(InrcConstraintConfig),
    Airline(AirlineFdtlConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct InrcConstraintConfig {
    pub planning_horizon_hours: Option<f64>,
    pub max_hours_per_worker: Option<f64>,
    pub minimum_rest_hours: Option<u64>,
    pub leave_requests: Option<Vec<LeaveRequest>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct AirlineFdtlConfig {
    pub planning_horizon_hours: Option<f64>,
    
    // DGCA FDTL Cumulative Flight Limits
    pub max_flight_hours_7d: Option<f64>,
    pub max_flight_hours_14d: Option<f64>,
    pub max_flight_hours_28d: Option<f64>,
    pub max_flight_hours_90d: Option<f64>,
    pub max_flight_hours_365d: Option<f64>,
    
    // DGCA FDTL Cumulative Duty Limits
    pub max_duty_hours_7d: Option<f64>,
    pub max_duty_hours_14d: Option<f64>,
    pub max_duty_hours_28d: Option<f64>,
    pub max_duty_hours_90d: Option<f64>,
    pub max_duty_hours_365d: Option<f64>,
    
    pub minimum_rest_hours: Option<f64>,
    pub leave_requests: Option<Vec<LeaveRequest>>,
}
```

### 3. Update Constraint Evaluators
Update `InrcConstraintEvaluator` and `AirlineConstraintEvaluator` in `adapters/ultracrew/src/constraint_engine.rs` to consume their respective configurations.
- `AirlineConstraintEvaluator` will no longer read `max_hours_per_worker`.
- The evaluation will use the FDTL thresholds defined in `AirlineFdtlConfig`.

### 4. Update Server Handlers
Update `pairings_handler` and `duties_handler` in `services/ultracrew_server/src/main.rs` to process the new scenario format, including the airline test inside `main.rs`. 
Also fix the `uc_domain_001.rs` tests to build correctly.

### 5. Domain Isolation Tests
Add strict compile/test-level checks asserting that:
- Parsing an Airline scenario with `max_hours_per_worker` fails deserialization (thanks to `deny_unknown_fields`).
- The `AirlineConstraintEvaluator` cannot access `InrcConstraintConfig`.

## User Review Required
- Is the `DomainConfig` untagged enum approach combined with explicit `domain` field preferred, or should we use an internally tagged enum like `#[serde(tag = "domain")] enum Scenario { Inrc(InrcConfig), Airline(AirlineConfig) }`? (The untagged enum approach matches your `Scenario -> domain_config -> INRCConfig/AirlineConfig` structure.)
- Should the `AirlineConstraintEvaluator` immediately enforce all these new FDTL rules in the optimization loop, or are we just laying the contract foundation first? (Writing full FDTL logic for 7d, 14d, 28d rolling windows is a large change to the evaluator).
