//! # OR-Library Benchmark Modules for Coralys
//!
//! This module implements parsers, domain representations, bitmask evaluators,
//! and heuristic solvers for canonical Operations Research Library (OR-Library)
//! benchmarks:
//! - [`spp`]: Set Partitioning Problem (Northwest Airlines `sppnw01`–`sppnw43`)
//! - [`airland`]: Aircraft Landing & Runway Arrival Sequencing (`airland1`–`airland13`)
//! - [`gerad`]: Crew Rostering Benchmark & Optimization (`Instance1`–`Instance7`)

pub mod airland;
pub mod gerad;
pub mod mixed_runway;
pub mod spp;

pub use airland::{
    AircraftLandingEvaluator, AirlandAction, AirlandDecisionPlugin, AirlandInstance,
    AirlandOptimizationConfig, AirlandPlane, AirlandSolution, AirlandSolver, AirlandState,
    CoralysAirlandEngine,
};
pub use gerad::{
    AirportTransitionGraph, CoralysGeradEngine, GeradAction, GeradCrewRosteringEvaluator,
    GeradDecisionPlugin, GeradState, OperationalKpis, OptimizationMode, OptimizationParameters,
    PilotPreference, PrecomputedDutyData, PrecomputedPairingData, RawCrew,
};
pub use mixed_runway::{
    ATCDisruption, CoralysMixedRunwayEngine, MixedRunwayAction, MixedRunwayDecisionPlugin,
    MixedRunwayEvaluator, MixedRunwayInstance, MixedRunwayOptimizationConfig, MixedRunwaySolution,
    MixedRunwayState, RunwayOp, RunwayOpType, WakeCategory,
};
pub use spp::{
    CoralysSppEngine, SetPartitioningEvaluator, SppAction, SppColumn, SppDecisionPlugin,
    SppInstance, SppOptimizationConfig, SppSolution, SppSolver, SppState,
};
