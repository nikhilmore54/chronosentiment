/// Fleet Utilization Certificate — FUC-001
/// GOV-009 / GOV-008 — Execution Evidence
///
/// Computes per-route load, slack, utilization, and fleet-level statistics
/// from the best solution produced by the optimizer.
///
/// This is a post-optimization certificate. It requires a feasible solution
/// (routes non-empty, total_distance < 1_000_000).
///
/// Design note: Min/Max utilization are diagnostic values, not performance
/// indicators. The summary is structured into four sections:
///   1. Fleet Packing   — avg/median utilization, total demand, capacity used
///   2. Distribution    — histogram of utilization buckets
///   3. Residual        — packed/high/residual route counts, RCI
///   4. Balance         — mean, median, std dev, coefficient of variation
///
/// Reference: benchmarks/campaign/qualification_decision_register.md §Stage B

use serde::{Deserialize, Serialize};
use crate::CvrpInstance;

// ---------------------------------------------------------------------------
// Per-route record
// ---------------------------------------------------------------------------

/// Load, slack, and utilization for a single route.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteLoad {
    /// 1-based vehicle index (for display).
    pub vehicle_id: usize,
    /// Number of customers on this route.
    pub customer_count: usize,
    /// Total demand served on this route.
    pub load: i32,
    /// Vehicle capacity.
    pub capacity: i32,
    /// Remaining capacity: `capacity - load`.
    pub slack: i32,
    /// Load / capacity, in [0.0, 1.0+].
    pub utilization: f64,
    /// True if load > capacity (capacity violation).
    pub capacity_violation: bool,
    /// True if route is empty (unused vehicle).
    pub empty: bool,
}

impl RouteLoad {
    fn compute(vehicle_id: usize, route: &[usize], instance: &CvrpInstance) -> Self {
        let capacity = instance.capacity;
        // Routes store node IDs (matching customer.id), not array indices.
        let load: i32 = route.iter().map(|&node_id| {
            if node_id == instance.depot.id {
                0
            } else {
                instance.customers.iter()
                    .find(|c| c.id == node_id)
                    .map(|c| c.demand)
                    .unwrap_or(0)
            }
        }).sum();
        let slack = capacity - load;
        let utilization = if capacity > 0 { load as f64 / capacity as f64 } else { 0.0 };
        RouteLoad {
            vehicle_id,
            customer_count: route.len(),
            load,
            capacity,
            slack,
            utilization,
            capacity_violation: load > capacity,
            empty: route.is_empty(),
        }
    }
}

// ---------------------------------------------------------------------------
// Fleet-level certificate
// ---------------------------------------------------------------------------

/// FUC-001: Fleet Utilization Certificate.
///
/// Produced after every successful optimization run.
/// Contains per-route loads and fleet-level statistics in four sections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetUtilizationCertificate {
    /// Instance name.
    pub instance_name: String,
    /// Benchmark vehicle count (K from registry).
    pub benchmark_vehicles: usize,
    /// Routes actually used in the best solution.
    pub routes_used: usize,
    /// Unused vehicles: benchmark_vehicles - routes_used (may be negative if routes_used > K).
    pub unused_vehicles: i32,
    /// Vehicle capacity (Q).
    pub capacity: i32,
    /// Total demand served across all routes.
    pub total_demand: i32,
    /// Per-route load records.
    pub route_loads: Vec<RouteLoad>,
    /// Number of capacity violations (routes where load > Q).
    pub capacity_violations: usize,
    /// Total customers served (sum of customer_count across routes).
    pub customers_served: usize,

    // --- Section 1: Fleet Packing ---
    /// Average load utilization across non-empty routes.
    pub avg_utilization: f64,
    /// Median load utilization across non-empty routes.
    pub median_utilization: f64,
    /// Fleet capacity used: total_demand / (routes_used * capacity).
    pub fleet_capacity_used: f64,

    // --- Section 2: Utilization Distribution ---
    /// Routes at exactly 100% utilization.
    pub dist_100pct: usize,
    /// Routes at 95–99% utilization.
    pub dist_95_99: usize,
    /// Routes at 90–94% utilization.
    pub dist_90_94: usize,
    /// Routes at 80–89% utilization.
    pub dist_80_89: usize,
    /// Routes at 70–79% utilization.
    pub dist_70_79: usize,
    /// Routes below 70% utilization.
    pub dist_below_70: usize,

    // --- Section 3: Residual Analysis ---
    /// Packed routes: utilization >= 95%.
    pub packed_routes: usize,
    /// High utilization routes: utilization >= 90%.
    pub high_util_routes: usize,
    /// Residual routes: utilization < 70%.
    pub residual_routes: usize,
    /// Total residual capacity: sum of slack across all non-empty routes.
    pub total_residual_capacity: i32,
    /// Largest residual slack (single route with most unused capacity).
    pub largest_residual_slack: i32,
    /// Residual Concentration Ratio: largest_slack / total_residual_capacity.
    /// 1.0 = all unused capacity in one route (concentrated residual).
    /// 0.0 = no unused capacity.
    /// High RCR (>0.7) indicates residual-route packing, not distributed inefficiency.
    pub residual_concentration_ratio: f64,

    // --- Section 4: Balance ---
    /// Mean load across non-empty routes.
    pub load_mean: f64,
    /// Median load across non-empty routes.
    pub load_median: f64,
    /// Load standard deviation across non-empty routes.
    pub load_stddev: f64,
    /// Coefficient of variation: stddev / mean (0 = perfectly balanced).
    pub load_cv: f64,
    /// Derived packing classification from median utilization, RCR, CV, and fleet capacity.
    pub packing_classification: PackingClassification,
}

/// Derived packing pattern classification for the fleet solution.
///
/// Computed from: median_utilization, residual_concentration_ratio, load_cv, fleet_capacity_used.
/// Allows quick scanning of campaign logs without interpreting individual metrics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackingClassification {
    /// Median ≥95%, RCR ≥0.80 — nearly all capacity used, residual concentrated in one route.
    HighlyConsolidated,
    /// Median ≥95%, RCR 0.50–0.80 — well packed with moderate residual spread.
    WellPacked,
    /// Median 85–95%, CV <0.15 — evenly distributed loads, no dominant residual.
    Balanced,
    /// CV >0.30 — significant load imbalance across routes.
    Uneven,
    /// Fleet capacity used <70% — instance is capacity-loose; optimizer has wide slack.
    CapacityLoose,
    /// No routes (empty solution).
    Empty,
}

impl PackingClassification {
    pub fn label(&self) -> &'static str {
        match self {
            PackingClassification::HighlyConsolidated => "HIGHLY_CONSOLIDATED",
            PackingClassification::WellPacked => "WELL_PACKED",
            PackingClassification::Balanced => "BALANCED",
            PackingClassification::Uneven => "UNEVEN",
            PackingClassification::CapacityLoose => "CAPACITY_LOOSE",
            PackingClassification::Empty => "EMPTY",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            PackingClassification::HighlyConsolidated =>
                "median≥95% RCR≥0.80 — residual concentrated in one route",
            PackingClassification::WellPacked =>
                "median≥95% RCR≥0.50 — well packed with moderate residual spread",
            PackingClassification::Balanced =>
                "median 85–95% CV<0.15 — evenly distributed loads",
            PackingClassification::Uneven =>
                "CV>0.30 — significant load imbalance across routes",
            PackingClassification::CapacityLoose =>
                "fleet capacity <70% — instance is capacity-loose",
            PackingClassification::Empty => "no routes",
        }
    }

    /// Derive classification from computed certificate metrics.
    pub fn derive(
        median_utilization: f64,
        rcr: f64,
        cv: f64,
        fleet_capacity_used: f64,
        routes_used: usize,
    ) -> Self {
        if routes_used == 0 {
            return PackingClassification::Empty;
        }
        if fleet_capacity_used < 0.70 {
            return PackingClassification::CapacityLoose;
        }
        if cv > 0.30 {
            return PackingClassification::Uneven;
        }
        if median_utilization >= 0.95 && rcr >= 0.80 {
            return PackingClassification::HighlyConsolidated;
        }
        if median_utilization >= 0.95 && rcr >= 0.50 {
            return PackingClassification::WellPacked;
        }
        if median_utilization >= 0.85 && cv < 0.15 {
            return PackingClassification::Balanced;
        }
        // Default: well packed if median is high enough
        if median_utilization >= 0.90 {
            PackingClassification::WellPacked
        } else {
            PackingClassification::Uneven
        }
    }
}

impl FleetUtilizationCertificate {
    /// Compute FUC-001 from the best solution routes.
    ///
    /// `routes` — customer node ID lists from `best_eval.routes`
    /// `benchmark_k` — vehicle count from benchmark registry
    pub fn compute(
        instance_name: &str,
        instance: &CvrpInstance,
        routes: &[Vec<usize>],
        benchmark_k: usize,
    ) -> Self {
        let capacity = instance.capacity;
        let routes_used = routes.len();
        let unused_vehicles = benchmark_k as i32 - routes_used as i32;

        let route_loads: Vec<RouteLoad> = routes
            .iter()
            .enumerate()
            .map(|(i, r)| RouteLoad::compute(i + 1, r, instance))
            .collect();

        let capacity_violations = route_loads.iter().filter(|r| r.capacity_violation).count();
        let customers_served: usize = route_loads.iter().map(|r| r.customer_count).sum();
        let total_demand: i32 = route_loads.iter().map(|r| r.load).sum();

        // Work over non-empty routes only
        let non_empty: Vec<&RouteLoad> = route_loads.iter().filter(|r| !r.empty).collect();
        let n = non_empty.len();

        if n == 0 {
            return FleetUtilizationCertificate {
                instance_name: instance_name.to_string(),
                benchmark_vehicles: benchmark_k,
                routes_used,
                unused_vehicles,
                capacity,
                total_demand,
                route_loads,
                capacity_violations,
                customers_served,
                avg_utilization: 0.0,
                median_utilization: 0.0,
                fleet_capacity_used: 0.0,
                dist_100pct: 0, dist_95_99: 0, dist_90_94: 0,
                dist_80_89: 0, dist_70_79: 0, dist_below_70: 0,
                packed_routes: 0, high_util_routes: 0, residual_routes: 0,
                total_residual_capacity: 0, largest_residual_slack: 0,
                residual_concentration_ratio: 0.0,
                load_mean: 0.0, load_median: 0.0, load_stddev: 0.0, load_cv: 0.0,
                packing_classification: PackingClassification::Empty,
            };
        }

        let mut utils: Vec<f64> = non_empty.iter().map(|r| r.utilization).collect();
        let mut loads: Vec<f64> = non_empty.iter().map(|r| r.load as f64).collect();
        let slacks: Vec<i32> = non_empty.iter().map(|r| r.slack).collect();

        // Section 1: Fleet Packing
        let avg_utilization = utils.iter().sum::<f64>() / n as f64;
        utils.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median_utilization = if n % 2 == 0 {
            (utils[n / 2 - 1] + utils[n / 2]) / 2.0
        } else {
            utils[n / 2]
        };
        let fleet_capacity_used = if routes_used > 0 && capacity > 0 {
            total_demand as f64 / (routes_used as f64 * capacity as f64)
        } else {
            0.0
        };

        // Section 2: Utilization Distribution
        let mut dist_100pct = 0usize;
        let mut dist_95_99 = 0usize;
        let mut dist_90_94 = 0usize;
        let mut dist_80_89 = 0usize;
        let mut dist_70_79 = 0usize;
        let mut dist_below_70 = 0usize;
        for &u in &utils {
            let pct = u * 100.0;
            if pct >= 100.0 { dist_100pct += 1; }
            else if pct >= 95.0 { dist_95_99 += 1; }
            else if pct >= 90.0 { dist_90_94 += 1; }
            else if pct >= 80.0 { dist_80_89 += 1; }
            else if pct >= 70.0 { dist_70_79 += 1; }
            else { dist_below_70 += 1; }
        }

        // Section 3: Residual Analysis
        let packed_routes = non_empty.iter().filter(|r| r.utilization >= 0.95).count();
        let high_util_routes = non_empty.iter().filter(|r| r.utilization >= 0.90).count();
        let residual_routes = non_empty.iter().filter(|r| r.utilization < 0.70).count();
        let total_residual_capacity: i32 = slacks.iter().sum();
        let largest_residual_slack = slacks.iter().cloned().max().unwrap_or(0);
        let residual_concentration_ratio = if total_residual_capacity > 0 {
            largest_residual_slack as f64 / total_residual_capacity as f64
        } else {
            0.0
        };

        // Section 4: Balance
        loads.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let load_median = if n % 2 == 0 {
            (loads[n / 2 - 1] + loads[n / 2]) / 2.0
        } else {
            loads[n / 2]
        };
        let load_mean = loads.iter().sum::<f64>() / n as f64;
        let variance = loads.iter().map(|l| (l - load_mean).powi(2)).sum::<f64>() / n as f64;
        let load_stddev = variance.sqrt();
        let load_cv = if load_mean > 0.0 { load_stddev / load_mean } else { 0.0 };

        let packing_classification = PackingClassification::derive(
            median_utilization,
            residual_concentration_ratio,
            load_cv,
            fleet_capacity_used,
            routes_used,
        );

        FleetUtilizationCertificate {
            instance_name: instance_name.to_string(),
            benchmark_vehicles: benchmark_k,
            routes_used,
            unused_vehicles,
            capacity,
            total_demand,
            route_loads,
            capacity_violations,
            customers_served,
            avg_utilization,
            median_utilization,
            fleet_capacity_used,
            dist_100pct,
            dist_95_99,
            dist_90_94,
            dist_80_89,
            dist_70_79,
            dist_below_70,
            packed_routes,
            high_util_routes,
            residual_routes,
            total_residual_capacity,
            largest_residual_slack,
            residual_concentration_ratio,
            load_mean,
            load_median,
            load_stddev,
            load_cv,
            packing_classification,
        }
    }

    /// True if any capacity violations exist.
    pub fn has_violations(&self) -> bool {
        self.capacity_violations > 0
    }

    /// One-line summary for campaign log.
    pub fn log_summary(&self) -> String {
        format!(
            "FUC-001: routes={}/{} avg={:.1}% median={:.1}% packed={}/{} residual={} cap_viol={} served={}",
            self.routes_used,
            self.benchmark_vehicles,
            self.avg_utilization * 100.0,
            self.median_utilization * 100.0,
            self.packed_routes,
            self.routes_used,
            self.residual_routes,
            self.capacity_violations,
            self.customers_served,
        )
    }

    /// Render a bar of `n` filled blocks (max_width total).
    fn bar(n: usize, max_n: usize, max_width: usize) -> String {
        if max_n == 0 { return String::new(); }
        let filled = (n * max_width + max_n - 1) / max_n;
        "█".repeat(filled)
    }

    /// Multi-line certificate for campaign log — 4-section format.
    pub fn log_certificate(&self) -> String {
        let mut lines = Vec::new();
        let sep = "  ─────────────────────────────────────────────────────────────";

        lines.push("╔══ FUC-001: Fleet Utilization Certificate ══════════════════════╗".to_string());
        lines.push(format!("  Instance          : {}", self.instance_name));
        lines.push(format!(
            "  Benchmark K       : {}    Routes used: {}    Unused: {}",
            self.benchmark_vehicles, self.routes_used, self.unused_vehicles.max(0)
        ));
        lines.push(format!(
            "  Vehicle capacity  : {}    Total demand: {}    Customers: {}",
            self.capacity, self.total_demand, self.customers_served
        ));

        // Per-route table
        lines.push(sep.to_string());
        for r in &self.route_loads {
            if r.empty {
                lines.push(format!("  V{:02}  [EMPTY]", r.vehicle_id));
            } else {
                let viol = if r.capacity_violation { " ⚠VIOLATION" } else { "" };
                lines.push(format!(
                    "  V{:02}  load={:4}/{:4}  slack={:4}  util={:5.1}%  n={:3}{}",
                    r.vehicle_id, r.load, r.capacity, r.slack,
                    r.utilization * 100.0, r.customer_count, viol,
                ));
            }
        }

        // Section 1: Fleet Packing
        lines.push(sep.to_string());
        lines.push("  § Fleet Packing".to_string());
        lines.push(format!(
            "    Avg utilization   : {:5.1}%    Median: {:5.1}%",
            self.avg_utilization * 100.0, self.median_utilization * 100.0
        ));
        lines.push(format!(
            "    Fleet capacity    : {:5.1}%    ({} / {} total)",
            self.fleet_capacity_used * 100.0,
            self.total_demand,
            self.routes_used as i32 * self.capacity
        ));

        // Section 2: Utilization Distribution
        let max_bucket = [
            self.dist_100pct, self.dist_95_99, self.dist_90_94,
            self.dist_80_89, self.dist_70_79, self.dist_below_70,
        ].iter().cloned().max().unwrap_or(1).max(1);
        lines.push(sep.to_string());
        lines.push("  § Utilization Distribution".to_string());
        lines.push(format!("    100%      {:2}  {}", self.dist_100pct,  Self::bar(self.dist_100pct,  max_bucket, 12)));
        lines.push(format!("    95–99%    {:2}  {}", self.dist_95_99,   Self::bar(self.dist_95_99,   max_bucket, 12)));
        lines.push(format!("    90–94%    {:2}  {}", self.dist_90_94,   Self::bar(self.dist_90_94,   max_bucket, 12)));
        lines.push(format!("    80–89%    {:2}  {}", self.dist_80_89,   Self::bar(self.dist_80_89,   max_bucket, 12)));
        lines.push(format!("    70–79%    {:2}  {}", self.dist_70_79,   Self::bar(self.dist_70_79,   max_bucket, 12)));
        lines.push(format!("    <70%      {:2}  {}", self.dist_below_70, Self::bar(self.dist_below_70, max_bucket, 12)));

        // Section 3: Residual Analysis
        lines.push(sep.to_string());
        lines.push("  § Residual Analysis".to_string());
        lines.push(format!(
            "    Packed  (≥95%)    : {:2} / {}",
            self.packed_routes, self.routes_used
        ));
        lines.push(format!(
            "    High    (≥90%)    : {:2} / {}",
            self.high_util_routes, self.routes_used
        ));
        lines.push(format!(
            "    Residual (<70%)   : {:2} / {}",
            self.residual_routes, self.routes_used
        ));
        lines.push(format!(
            "    Total residual cap: {}    Largest residual: {}    RCR: {:.3}",
            self.total_residual_capacity, self.largest_residual_slack,
            self.residual_concentration_ratio
        ));
        // Interpret RCR
        let rcr_label = if self.residual_concentration_ratio >= 0.70 {
            "concentrated residual (one cleanup route)"
        } else if self.residual_concentration_ratio >= 0.40 {
            "moderate residual spread"
        } else if self.total_residual_capacity == 0 {
            "fully packed"
        } else {
            "distributed residual (possible inefficiency)"
        };
        lines.push(format!("    RCR interpretation: {}", rcr_label));

        // Section 4: Balance
        lines.push(sep.to_string());
        lines.push("  § Balance".to_string());
        lines.push(format!(
            "    Mean load: {:.1}    Median: {:.1}    Std dev: {:.1}    CV: {:.3}",
            self.load_mean, self.load_median, self.load_stddev, self.load_cv
        ));
        lines.push(format!(
            "    Capacity violations: {}",
            self.capacity_violations
        ));
        lines.push(format!(
            "    Packing classification: {} — {}",
            self.packing_classification.label(),
            self.packing_classification.description(),
        ));

        lines.push("╚═════════════════════════════════════════════════════════════════╝".to_string());
        lines.join("\n")
    }
}