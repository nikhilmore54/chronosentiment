use axum::{
    routing::{get, post},
    Router, Json, extract::State, response::IntoResponse, http::StatusCode
};
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};

pub trait ReachabilityPredictor {
    fn predict_elite_prob(&self, parent_dist: f64, damage: f64) -> f64;
}

pub struct EmpiricalReachabilityPredictor;

impl ReachabilityPredictor for EmpiricalReachabilityPredictor {
    fn predict_elite_prob(&self, parent_dist: f64, damage: f64) -> f64 {
        let p_bucket = if parent_dist <= 810.0 { 0 }
                       else if parent_dist <= 820.0 { 1 }
                       else if parent_dist <= 840.0 { 2 }
                       else { 3 };
                       
        let d_bucket = if damage < 0.0 { 0 }
                       else if damage <= 50.0 { 1 }
                       else if damage <= 100.0 { 2 }
                       else if damage <= 200.0 { 3 }
                       else if damage <= 400.0 { 4 }
                       else { 5 };

        let matrix = [
            [96.4, 59.2, 51.1, 40.7, 18.7, 7.9],
            [16.5, 16.1, 20.6, 17.0, 10.0, 5.3],
            [17.6, 12.0, 15.7, 12.6, 7.4, 4.8],
            [18.7, 11.9, 11.4, 8.7, 6.3, 4.2],
        ];
        matrix[p_bucket][d_bucket]
    }
}

pub trait GuidancePolicy {
    fn accept(&self, p_elite: f64, rng: &mut impl Rng) -> bool;
}

pub struct SoftGuidancePolicy;
impl GuidancePolicy for SoftGuidancePolicy {
    fn accept(&self, p_elite: f64, rng: &mut impl Rng) -> bool {
        rng.gen_range(0.0..1.0) < (p_elite / 100.0)
    }
}

use tower_http::cors::{Any, CorsLayer};
use serde::Serialize;
use std::cmp::Ordering;
use rand::SeedableRng;

use coralys_ecology::diagnostics::{AccumulationFailureDetector, EcologyLockInDetector, OperatorExpressivenessFailureDetector, DiagnosticDetector, EcologyState, SearchObservation, CandidateObservation, ObjectiveVector, DiagnosticResult};
use coralys_recommendation::{EcologyRecommender, RecommendationReport};
use coralys_moga::traits::{FitnessEvaluator, CrossoverOperator, GenomeFactory, Evaluated, Genome, ImprovementOperator};
use coralys_moga::runtime::optimization::metric::MetricReport;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn get_canonical_signature(routes: &Vec<Vec<usize>>) -> u64 {
    let mut canonical = routes.clone();
    for r in &mut canonical { r.sort_unstable(); }
    canonical.sort_unstable();
    calculate_hash(&canonical)
}

fn get_edges(routes: &Vec<Vec<usize>>) -> std::collections::HashSet<(usize, usize)> {
    let mut edges = std::collections::HashSet::new();
    for r in routes {
        let mut prev = 0; // Depot
        for &node in r {
            let min_n = prev.min(node);
            let max_n = prev.max(node);
            edges.insert((min_n, max_n));
            prev = node;
        }
        edges.insert((prev.min(0), prev.max(0)));
    }
    edges
}

use cvrp::{CvrpInstance, CvrpGenerationState, moga_impl::*};
use cvrp::CvrpGenomeFactory;

#[derive(Clone)]
struct AppState {
    pub is_running: Arc<Mutex<bool>>,
    pub current_generation: Arc<Mutex<Option<CvrpGenerationState>>>,
    pub current_recommendations: Arc<Mutex<Option<RecommendationReport>>>,
    pub current_diagnostics: Arc<Mutex<Vec<NamedDiagnosticResult>>>,
    pub entropy_scale: Arc<Mutex<f64>>,
}

#[derive(Serialize, Clone)]
pub struct NamedDiagnosticResult {
    name: String,
    confidence: f64,
    severity: f64,
}

#[derive(Serialize)]
struct StatusResponse {
    running: bool,
    generation: Option<CvrpGenerationState>,
    diagnostics: Vec<NamedDiagnosticResult>,
    recommendations: Option<RecommendationReport>,
    instance: CvrpInstance,
    entropy_scale: f64,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        is_running: Arc::new(Mutex::new(false)),
        current_generation: Arc::new(Mutex::new(None)),
        current_recommendations: Arc::new(Mutex::new(None)),
        current_diagnostics: Arc::new(Mutex::new(Vec::new())),
        entropy_scale: Arc::new(Mutex::new(
            std::env::var("MUTATION_SCALE").unwrap_or_else(|_| "1.0".to_string()).parse::<f64>().unwrap_or(1.0)
        )),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/state", get(get_state))
        .route("/api/run", post(start_run))
        .route("/api/stop", post(stop_run))
        .route("/api/intervene", post(apply_intervention))
        .route("/api/reset_entropy", post(reset_entropy))
        .layer(cors)
        .with_state(state.clone());
    if std::env::var("FAST_MODE").is_ok() {
        *state.is_running.lock().await = true;
        let state_clone = state.clone();
        tokio::spawn(async move {
            run_evolution_loop(state_clone).await;
        }).await.unwrap();
        
        // Wait briefly for the loop to complete if it finishes quickly
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let gen_state = state.current_generation.lock().await.clone().unwrap();
        println!("FINAL_RESULT:{},{},{},{},{},{}", gen_state.best_distance, gen_state.p10_distance, gen_state.median_distance, gen_state.elite_similarity, gen_state.elite_offspring_survival_rate, gen_state.top10_offspring_rate);

        // Serialize the last telemetry observation to mock a run for Coralys Server
        let mock_run = serde_json::json!({
            "global_best_fitness": gen_state.best_distance,
            "detector_applicability_matrix": {},
            "policy_action": "None",
            "policy_confidence": 0.0,
            "telemetry": {
                "generation": gen_state.generation,
                "population": {
                    "best_fitness": gen_state.best_distance,
                    "median_fitness": gen_state.median_distance,
                    "worst_fitness": gen_state.worst_distance,
                    "diversity_score": gen_state.diversity_score,
                    "elite_diversity_score": gen_state.elite_diversity_score,
                    "feasible_ratio": gen_state.feasible_population_ratio
                },
                "attachments": [
                    {
                        "namespace": "moga.operator_ecology",
                        "version": 1,
                        "payload": {
                            "parent_similarity": gen_state.parent_similarity,
                            "offspring_novelty": gen_state.offspring_novelty,
                            "structural_damage_ratio": gen_state.crossover_structural_damage_ratio,
                            "elite_survival_rate": gen_state.elite_offspring_survival_rate,
                            "top10_offspring_rate": gen_state.top10_offspring_rate
                        }
                    }
                ]
            }
        });
        std::fs::write("cvrp_seed_42.json", serde_json::to_string_pretty(&mock_run).unwrap()).unwrap();
        return;
    }

    println!("Starting CVRP Server on 0.0.0.0:4002");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4002").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_state(State(state): State<AppState>) -> impl IntoResponse {
    let running = *state.is_running.lock().await;
    let generation = state.current_generation.lock().await.clone();
    let diags = state.current_diagnostics.lock().await.clone();
    let recs = state.current_recommendations.lock().await.clone();
    let entropy = *state.entropy_scale.lock().await;

    Json(StatusResponse {
        running,
        generation,
        diagnostics: diags,
        recommendations: recs,
        instance: CvrpInstance::a_n32_k5(),
        entropy_scale: entropy,
    })
}

async fn apply_intervention(State(state): State<AppState>) -> impl IntoResponse {
    let mut entropy = state.entropy_scale.lock().await;
    *entropy = (*entropy * 3.0).min(10.0); // Increase but cap at 10.0x
    StatusCode::OK
}

async fn reset_entropy(State(state): State<AppState>) -> impl IntoResponse {
    let mut entropy = state.entropy_scale.lock().await;
    *entropy = 1.0;
    StatusCode::OK
}

async fn stop_run(State(state): State<AppState>) -> impl IntoResponse {
    let mut running = state.is_running.lock().await;
    *running = false;
    StatusCode::OK
}

async fn start_run(State(state): State<AppState>) -> impl IntoResponse {
    let mut running = state.is_running.lock().await;
    if *running {
        return StatusCode::BAD_REQUEST;
    }
    *running = true;
    
    let state_clone = state.clone();
    tokio::spawn(async move {
        run_evolution_loop(state_clone).await;
    });

    StatusCode::OK
}

async fn run_evolution_loop(state: AppState) {
    let instance = CvrpInstance::a_n32_k5();
    let evaluator = CvrpEvaluator { instance: instance.clone() };
    let radius_env = std::env::var("RADIUS_POLICY").unwrap_or_else(|_| "Control".to_string());
    let radius_policy = match radius_env.as_str() {
        "LocalBiased" => cvrp::RadiusPolicy::LocalBiased,
        "ExtremeLocal" => cvrp::RadiusPolicy::ExtremeLocal,
        _ => cvrp::RadiusPolicy::Control,
    };
    let mut mutator: Box<dyn coralys_moga::traits::MutationOperator<cvrp::CvrpCandidate> + Send + Sync> = 
        if std::env::var("USE_ROUTE_AWARE_MUTATOR").is_ok() {
            Box::new(cvrp::moga_impl::CvrpRouteAwareMutator { instance: instance.clone() })
        } else {
            Box::new(cvrp::moga_impl::CvrpMutator::new(instance.clone(), radius_policy.clone()))
        };
    let crossover_type = std::env::var("CROSSOVER_TYPE").unwrap_or_else(|_| "OX1".to_string());
    let crossover = match crossover_type.as_str() {
        "ROUTE" => cvrp::moga_impl::CvrpCrossoverVariant::RoutePreserving(cvrp::moga_impl::CvrpCrossoverRoutePreserving { instance: instance.clone() }),
        _ => cvrp::moga_impl::CvrpCrossoverVariant::OX1(cvrp::moga_impl::CvrpCrossover),
    };
    let local_search = cvrp::moga_impl::CvrpLocalSearch { instance: instance.clone() };
    let factory = CvrpGenomeFactory { num_customers: instance.customers.len() };

    let mut accum_detector = AccumulationFailureDetector::new(0, 0.75);
    let mut lockin_detector = EcologyLockInDetector::new(0, 0.15, 100);
    let rec_engine = EcologyRecommender::new(0.5, 100);
    let mut innovation_tracker = coralys_core::memory::InnovationTracker::new();
    let mut progress_tracker = coralys_ecology::progress::ProgressTracker::new();

    let population_size = 100;
    let mut rng = rand::rngs::StdRng::from_entropy();
    
    let mut population: Vec<_> = (0..population_size)
        .map(|_| {
            let mut cand = factory.create(&mut rng);
            let model = cvrp::moga_impl::CvrpConstraintModel { instance: instance.clone() };
            let budget = coralys_core::operators::OperatorBudget { max_iterations: 1, max_time_ms: 1000 };
            coralys_core::operators::ImprovementOperator::improve(&local_search, &mut cand, &model, &budget).unwrap();
            cand
        })
        .collect();

    let mut telemetry = Vec::new();
    let mut ecology_state = EcologyState::new(10000);
    let mut global_best_fitness = f64::NEG_INFINITY;
    let mut generations_since_improvement = 0;
    
    let mut basin_tracker: HashMap<u64, usize> = HashMap::new();
    let mut all_basin_distances: Vec<f64> = Vec::new();

    let init_entropy = *state.entropy_scale.lock().await;
    println!("INITIAL MUTATION SCALE = {}", init_entropy);

    for generation in 1..=10000 {
        let is_running = *state.is_running.lock().await;
        if !is_running { break; }

        let current_entropy = *state.entropy_scale.lock().await;
        // mutator.entropy_scale = current_entropy;

        if generation % 100 == 0 {
            println!("Generation {}", generation);
        }
        
        if std::env::var("FAST_MODE").is_ok() && generation >= 300 {
            break;
        }

        let mut evals: Vec<_> = population
            .iter()
            .map(|c| evaluator.evaluate(c, &MetricReport::default()))
            .filter(|e| e.is_valid())
            .collect();

        if evals.is_empty() {
            population = (0..population_size).map(|_| {
                let mut cand = factory.create(&mut rng);
                let model = cvrp::moga_impl::CvrpConstraintModel { instance: instance.clone() };
                let budget = coralys_core::operators::OperatorBudget { max_iterations: 1, max_time_ms: 1000 };
                coralys_core::operators::ImprovementOperator::improve(&local_search, &mut cand, &model, &budget).unwrap();
                cand
            }).collect();
            generations_since_improvement += 1;
            continue;
        }

        evals.sort_by(|a, b| b.fitness().partial_cmp(&a.fitness()).unwrap_or(Ordering::Equal));
        let gen_best = evals[0].clone();
        
        let archive_best_before = global_best_fitness;
        let mut improved_this_gen = false;
        
        if gen_best.fitness() > global_best_fitness {
            global_best_fitness = gen_best.fitness();
            improved_this_gen = true;
            generations_since_improvement = 0;
        } else {
            generations_since_improvement += 1;
        }

        let avg_dist = evals.iter().map(|e| e.eval.total_distance).sum::<f64>() / evals.len() as f64;
        let p10_distance = evals[evals.len() / 10].eval.total_distance;
        let p25_distance = evals[evals.len() / 4].eval.total_distance;
        let median_distance = evals[evals.len() / 2].eval.total_distance;
        let p75_distance = evals[(evals.len() * 3) / 4].eval.total_distance;
        let worst_distance = evals[evals.len() - 1].eval.total_distance;
        
        // Track global improving candidate generation
        let global_improving_generated = evals.iter().filter(|e| e.fitness() > archive_best_before).count();

        let mut candidates_obs = Vec::new();
        for eval in &evals {
            candidates_obs.push(CandidateObservation {
                objectives: ObjectiveVector { values: vec![eval.fitness()] },
                admitted: eval.fitness() > archive_best_before,
                feasible: true,
                parent_objectives: None,
            });
        }

        let obs = SearchObservation {
            generation,
            archive_size: 1,
            diversity_score: 0.1, // mock
            candidates: candidates_obs,
            archive_objectives: vec![ObjectiveVector { values: vec![gen_best.fitness()] }],
            telemetry: None,
        };
        telemetry.push(obs);
        if telemetry.len() > 500 {
            telemetry.drain(0..200);
        }

        let obs_window = 200;
        let start_idx = telemetry.len().saturating_sub(obs_window);
        let obs_slice = &telemetry[start_idx..];

        let mut results = HashMap::new();
        let res_accum = accum_detector.evaluate(obs_slice);
        let res_lockin = lockin_detector.evaluate(obs_slice);
        let expressiveness_detector = OperatorExpressivenessFailureDetector::new(100, 0.7);
        let res_express = expressiveness_detector.evaluate(obs_slice);

        results.insert("AccumulationFailure".to_string(), res_accum.clone());
        results.insert("EcologyLockIn".to_string(), res_lockin.clone());
        results.insert("OperatorExpressivenessFailure".to_string(), res_express.clone());

        // TEMPORARILY DISABLED GOVERNANCE RECORDING PER PLAN
        // ecology_state.record(generation, results);

        let mut diagnostics = Vec::new();
        diagnostics.push(NamedDiagnosticResult { name: "Accumulation Failure".into(), confidence: res_accum.confidence, severity: res_accum.severity });
        diagnostics.push(NamedDiagnosticResult { name: "Ecology Lock-In".into(), confidence: res_lockin.confidence, severity: res_lockin.severity });
        diagnostics.push(NamedDiagnosticResult { name: "Operator Expressiveness Failure".into(), confidence: res_express.confidence, severity: res_express.severity });
        
        let recs = rec_engine.recommend(&ecology_state);

        let mut next_gen = Vec::with_capacity(population_size);
        next_gen.extend(evals.iter().take(2).map(|e| e.genome().clone()));

        let mut total_pairs = 0;
        let mut total_preserved_pairs = 0;
        let mut total_parent_similarity = 0.0;
        let mut total_offspring_novelty = 0.0;
        let mut num_offspring = 0.0;
        let mut local_improving_generated = 0;
        let mut total_top_10_parents = 0;
        let mut total_top_20_parents = 0;
        let mut total_bottom_50_parents = 0;
        let mut children_better_than_median = 0;
        let mut children_entering_top10 = 0;
        let median_fitness = evals[evals.len() / 2].fitness();
        let p10_fitness = evals[evals.len() / 10].fitness();
        
        let mut total_mutation_damage = 0.0;
        let mut total_basin_depth = 0.0;
        let mut new_basin_discoveries = 0;
        let mut parent_reversions = 0;
        let mut total_memetic_children = 0;
        let mut global_basin_revisits = 0;
        let mut recent_basin_revisits = 0;
        let mut elite_basin_discoveries = 0;
        let mut total_damage_edge_count = 0;
        let mut total_repair_edge_count = 0;
        
        let mut deepest_basin_transition: Option<cvrp::BasinTransition> = None;
        let mut max_basin_depth = -1.0;

        let mut total_shadow_rejected_offspring = 0;
        let mut total_shadow_rejected_elites = 0;
        let mut total_p_elite = 0.0;
        let mut p_elite_count = 0;
        let mut calib_buckets: std::collections::HashMap<String, cvrp::CalibrationBucket> = std::collections::HashMap::new();
        let current_guidance_mode = cvrp::GuidanceMode::Shadow;
        let predictor = EmpiricalReachabilityPredictor;
        let policy = SoftGuidancePolicy;

        while next_gen.len() < population_size {
            use rand::Rng;
            
            let tournament_size: usize = std::env::var("TOURNAMENT_SIZE").unwrap_or_else(|_| "2".to_string()).parse().unwrap_or(2);
            let random_parent_prob: f64 = std::env::var("RANDOM_PARENT_PROB").unwrap_or_else(|_| "0.2".to_string()).parse().unwrap_or(0.2);

            let p1_idx = if rng.gen_bool(random_parent_prob) {
                rng.gen_range(0..evals.len())
            } else {
                let mut best_idx = rng.gen_range(0..evals.len());
                for _ in 0..(tournament_size.saturating_sub(1)) { 
                    let cand_idx = rng.gen_range(0..evals.len());
                    if evals[cand_idx].fitness() > evals[best_idx].fitness() { best_idx = cand_idx; }
                }
                best_idx
            };
            let p1_eval = &evals[p1_idx];

            let p2_idx = if rng.gen_bool(random_parent_prob) {
                rng.gen_range(0..evals.len())
            } else {
                let mut best_idx = rng.gen_range(0..evals.len());
                for _ in 0..(tournament_size.saturating_sub(1)) { 
                    let cand_idx = rng.gen_range(0..evals.len());
                    if evals[cand_idx].fitness() > evals[best_idx].fitness() { best_idx = cand_idx; }
                }
                best_idx
            };
            let p2_eval = &evals[p2_idx];

            for idx in [p1_idx, p2_idx] {
                if idx < 10 { total_top_10_parents += 1; }
                if idx < 20 { total_top_20_parents += 1; }
                if idx >= population_size / 2 { total_bottom_50_parents += 1; }
            }

            let (mut c1, mut c2) = crossover.crossover(p1_eval.genome(), p2_eval.genome(), &mut rng);
            
            // Parent similarity & Offspring novelty
            let size = p1_eval.genome().permutation.len();
            let mut identical_parents = 0;
            for k in 0..size {
                if p1_eval.genome().permutation[k] == p2_eval.genome().permutation[k] {
                    identical_parents += 1;
                }
            }
            total_parent_similarity += identical_parents as f64 / size as f64;

            for c in [&c1, &c2] {
                let mut identical_p1 = 0;
                let mut identical_p2 = 0;
                for k in 0..size {
                    if c.permutation[k] == p1_eval.genome().permutation[k] { identical_p1 += 1; }
                    if c.permutation[k] == p2_eval.genome().permutation[k] { identical_p2 += 1; }
                }
                let sim_p1 = identical_p1 as f64 / size as f64;
                let sim_p2 = identical_p2 as f64 / size as f64;
                total_offspring_novelty += 1.0 - sim_p1.max(sim_p2);
                num_offspring += 1.0;
            }

            let c1_crossover_dist = evaluator.evaluate(&c1, &MetricReport::default()).eval.total_distance;
            let c2_crossover_dist = evaluator.evaluate(&c2, &MetricReport::default()).eval.total_distance;

            let mut c1_mut_eval = evaluator.evaluate(&c1, &MetricReport::default());
            let mut c1_mutated_dist = c1_mut_eval.eval.total_distance;
            let mut c2_mut_eval = evaluator.evaluate(&c2, &MetricReport::default());
            let mut c2_mutated_dist = c2_mut_eval.eval.total_distance;
            
            let mut c1_p_elite = 0.0;
            let mut c1_bucket = "";
            let mut c2_p_elite = 0.0;
            let mut c2_bucket = "";

            for (idx, (child, p_dist, mut_e, m_dist)) in [
                (&mut c1, c1_crossover_dist, &mut c1_mut_eval, &mut c1_mutated_dist),
                (&mut c2, c2_crossover_dist, &mut c2_mut_eval, &mut c2_mutated_dist)
            ].into_iter().enumerate() {
                let mut tries = 0;
                loop {
                    let mut test_child = child.clone();
                    mutator.mutate(&mut test_child, &mut rng);
                    let mut_eval = evaluator.evaluate(&test_child, &MetricReport::default());
                    let mut_dist = mut_eval.eval.total_distance;
                    let damage = mut_dist - p_dist;
                    
                    let p_elite = predictor.predict_elite_prob(p_dist, damage);
                    let bucket_name = if p_elite < 5.0 { "0-5%" }
                                  else if p_elite < 10.0 { "5-10%" }
                                  else if p_elite < 20.0 { "10-20%" }
                                  else if p_elite < 40.0 { "20-40%" }
                                  else if p_elite < 60.0 { "40-60%" }
                                  else { "60%+" };

                    if current_guidance_mode == cvrp::GuidanceMode::Shadow {
                        total_p_elite += p_elite;
                        p_elite_count += 1;
                        let bucket = calib_buckets.entry(bucket_name.to_string()).or_insert_with(cvrp::CalibrationBucket::default);
                        bucket.predicted_count += 1;
                        if p_elite < 10.0 {
                            total_shadow_rejected_offspring += 1;
                        }
                        if idx == 0 { c1_p_elite = p_elite; c1_bucket = bucket_name; }
                        else { c2_p_elite = p_elite; c2_bucket = bucket_name; }
                        *child = test_child; *mut_e = mut_eval; *m_dist = mut_dist;
                        break;
                    } else if current_guidance_mode == cvrp::GuidanceMode::Soft {
                        if policy.accept(p_elite, &mut rng) {
                            if idx == 0 { c1_p_elite = p_elite; c1_bucket = bucket_name; }
                            else { c2_p_elite = p_elite; c2_bucket = bucket_name; }
                            *child = test_child; *mut_e = mut_eval; *m_dist = mut_dist;
                            break;
                        }
                        total_shadow_rejected_offspring += 1;
                    } else {
                        *child = test_child; *mut_e = mut_eval; *m_dist = mut_dist;
                        break;
                    }
                    tries += 1;
                    if tries > 100 { 
                        if idx == 0 { c1_p_elite = p_elite; c1_bucket = bucket_name; }
                        else { c2_p_elite = p_elite; c2_bucket = bucket_name; }
                        *child = test_child; *mut_e = mut_eval; *m_dist = mut_dist;
                        break; 
                    }
                }
            }

            // Apply True Local Search exploitation phase
            let model = cvrp::moga_impl::CvrpConstraintModel { instance: instance.clone() };
            let budget = coralys_core::operators::OperatorBudget { max_iterations: 1, max_time_ms: 1000 };
            coralys_core::operators::ImprovementOperator::improve(&local_search, &mut c1, &model, &budget).unwrap();
            coralys_core::operators::ImprovementOperator::improve(&local_search, &mut c2, &model, &budget).unwrap();

            // Calculate preservation metrics and local improvements
            let c1_eval = evaluator.evaluate(&c1, &MetricReport::default());
            let c2_eval = evaluator.evaluate(&c2, &MetricReport::default());

            let c1_opt_dist = c1_eval.eval.total_distance;
            let c2_opt_dist = c2_eval.eval.total_distance;

            total_mutation_damage += (c1_mutated_dist - c1_crossover_dist) + (c2_mutated_dist - c2_crossover_dist);
            total_basin_depth += (c1_mutated_dist - c1_opt_dist) + (c2_mutated_dist - c2_opt_dist);

            for (c, p1_perm, p2_perm, p_routes, c_eval, mut_eval, mut_dist, opt_dist) in [
                (&c1, &p1_eval.genome().permutation, &p2_eval.genome().permutation, &p1_eval.eval.routes, &c1_eval, &c1_mut_eval, c1_mutated_dist, c1_opt_dist),
                (&c2, &p1_eval.genome().permutation, &p2_eval.genome().permutation, &p2_eval.eval.routes, &c2_eval, &c2_mut_eval, c2_mutated_dist, c2_opt_dist)
            ] {
                let basin_depth = mut_dist - opt_dist;
                let mutated_edges = get_edges(&mut_eval.eval.routes);
                let optimized_edges = get_edges(&c_eval.eval.routes);
                let parent_edges = get_edges(p_routes);
                let dmg_count = mutated_edges.difference(&optimized_edges).count();
                let rep_count = optimized_edges.difference(&mutated_edges).count();
                let mutation_edge_damage = parent_edges.difference(&mutated_edges).count();
                
                total_damage_edge_count += dmg_count;
                total_repair_edge_count += rep_count;

                if basin_depth > max_basin_depth {
                    max_basin_depth = basin_depth;
                    deepest_basin_transition = Some(cvrp::BasinTransition {
                        parent_routes: p1_eval.eval.routes.clone(),
                        mutated_routes: mut_eval.eval.routes.clone(),
                        optimized_routes: c_eval.eval.routes.clone(),
                        damage_edge_count: dmg_count,
                        repair_edge_count: rep_count,
                        optimized_distance: opt_dist,
                    });
                }

                if c.permutation == *p1_perm || c.permutation == *p2_perm {
                    parent_reversions += 1;
                } else {
                    new_basin_discoveries += 1; // Structurally distinct from parents
                }
                
                let parent_dist = p1_eval.eval.total_distance;
                let basin_hash = get_canonical_signature(&c_eval.eval.routes);

                let is_elite = opt_dist <= 805.0;
                
                let p_elite = if c.permutation == c1.permutation { c1_p_elite } else { c2_p_elite };
                let bucket_name = if c.permutation == c1.permutation { c1_bucket } else { c2_bucket };

                if is_elite {
                    if let Some(bucket) = calib_buckets.get_mut(bucket_name) {
                        bucket.elite_count += 1;
                    }
                    if current_guidance_mode == cvrp::GuidanceMode::Shadow && p_elite < 10.0 {
                        total_shadow_rejected_elites += 1;
                    }
                }
                
                let op_name = c.last_mutation_op.as_deref().unwrap_or("None");
                let radius = c.last_mutation_radius.unwrap_or(0);

                let csv_name = if std::env::var("USE_ROUTE_AWARE_MUTATOR").is_ok() {
                    "m10_route_aware.csv"
                } else {
                    match radius_policy {
                        cvrp::RadiusPolicy::Control => "m10_control.csv",
                        cvrp::RadiusPolicy::LocalBiased => "m10_local.csv",
                        cvrp::RadiusPolicy::ExtremeLocal => "m10_extreme.csv",
                    }
                };

                let boundary_changes = c.route_boundary_changes.unwrap_or_else(|| {
                    use std::collections::HashSet;
                    let p_sets: Vec<HashSet<usize>> = p_routes.iter().map(|r| r.iter().cloned().collect()).collect();
                    let m_sets: Vec<HashSet<usize>> = mut_eval.eval.routes.iter().map(|r| r.iter().cloned().collect()).collect();
                    m_sets.iter().filter(|m| !p_sets.contains(m)).count()
                });

                // Log offspring telemetry
                use std::io::Write;
                if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(csv_name) {
                    let _ = writeln!(
                        file, 
                        "{},{},{},{},{},{},{},{},{},{},{},{}", 
                        generation, basin_hash, parent_dist, mut_dist, opt_dist, 
                        mut_dist - parent_dist, // Mutation Damage
                        basin_depth, rep_count, mutation_edge_damage, op_name, radius, boundary_changes
                    );
                }

                if let Some(&last_gen) = basin_tracker.get(&basin_hash) {
                    global_basin_revisits += 1;
                    if generation.saturating_sub(last_gen) <= 500 {
                        recent_basin_revisits += 1;
                    }
                } else {
                    let dist = c_eval.eval.total_distance;
                    if all_basin_distances.len() >= 20 {
                        let threshold_idx = all_basin_distances.len() / 20; // 5th percentile
                        let elite_threshold = all_basin_distances[threshold_idx];
                        if dist <= elite_threshold {
                            elite_basin_discoveries += 1;
                        }
                    } else {
                        elite_basin_discoveries += 1;
                    }
                    let pos = all_basin_distances.binary_search_by(|a| a.partial_cmp(&dist).unwrap_or(std::cmp::Ordering::Equal)).unwrap_or_else(|e| e);
                    all_basin_distances.insert(pos, dist);
                    
                    use std::io::Write;
                    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("basin_discoveries.csv") {
                        let _ = writeln!(file, "{},{}", generation, dist);
                    }
                }
                basin_tracker.insert(basin_hash, generation);
            }
            total_memetic_children += 2;

            let best_parent_fitness = p1_eval.fitness().max(p2_eval.fitness());
            if c1_eval.fitness() > best_parent_fitness { local_improving_generated += 1; }
            if c2_eval.fitness() > best_parent_fitness { local_improving_generated += 1; }
            
            for c_eval in [&c1_eval, &c2_eval] {
                if c_eval.fitness() > median_fitness { children_better_than_median += 1; }
                if c_eval.fitness() > p10_fitness { children_entering_top10 += 1; }
            }

            for c_eval in [&c1_eval, &c2_eval] {
                for route in &c_eval.eval.routes {
                    for window in route.windows(2) {
                        let a = window[0];
                        let b = window[1];
                        let mut preserved = false;
                        for p_eval in [p1_eval, p2_eval] {
                            if p_eval.eval.routes.iter().any(|r| r.windows(2).any(|w| (w[0] == a && w[1] == b) || (w[0] == b && w[1] == a))) {
                                preserved = true;
                                break;
                            }
                        }
                        total_pairs += 1;
                        if preserved { total_preserved_pairs += 1; }
                    }
                }
            }

            next_gen.push(c1);
            if next_gen.len() < population_size {
                next_gen.push(c2);
            }
        }
        
        let crossover_structural_damage_ratio = if total_pairs > 0 {
            1.0 - (total_preserved_pairs as f64 / total_pairs as f64)
        } else {
            0.0
        };
        
        let parent_similarity = if num_offspring > 0.0 { total_parent_similarity / (num_offspring / 2.0) } else { 0.0 };
        let offspring_novelty = if num_offspring > 0.0 { total_offspring_novelty / num_offspring } else { 0.0 };

        // Evaluate accepted improving candidates
        let improving_accepted = next_gen.iter().filter(|g| evaluator.evaluate(g, &MetricReport::default()).fitness() > archive_best_before).count();
        let improving_rejected = global_improving_generated.saturating_sub(improving_accepted);

        let mut unique_dists: Vec<_> = evals.iter().map(|e| (e.eval.total_distance * 1000.0).round() as i64).collect();
        unique_dists.sort_unstable();
        unique_dists.dedup();
        let diversity_score = unique_dists.len() as f64 / evals.len() as f64;

        let num_elites = (evals.len() / 5).max(1);
        let elite_evals = &evals[0..num_elites];
        let mut unique_elite_dists: Vec<_> = elite_evals.iter().map(|e| (e.eval.total_distance * 1000.0).round() as i64).collect();
        unique_elite_dists.sort_unstable();
        unique_elite_dists.dedup();
        let elite_diversity_score = unique_elite_dists.len() as f64 / num_elites as f64;

        let mut unique_route_structures = std::collections::HashSet::new();
        for eval in &evals {
            let mut canonical_routes = eval.eval.routes.clone();
            for r in &mut canonical_routes { r.sort_unstable(); }
            canonical_routes.sort_unstable();
            unique_route_structures.insert(canonical_routes);
        }
        let route_diversity_score = unique_route_structures.len() as f64 / evals.len() as f64;

        let mut total_elite_sim = 0.0;
        let elite_perm = &evals[0].genome().permutation;
        let p_size = elite_perm.len();
        for eval in &evals {
            let mut identical = 0;
            for k in 0..p_size {
                if eval.genome().permutation[k] == elite_perm[k] { identical += 1; }
            }
            total_elite_sim += identical as f64 / p_size as f64;
        }
        let elite_similarity = total_elite_sim / evals.len() as f64;

        let mut gen_state = CvrpGenerationState {
            generation,
            best_distance: gen_best.eval.total_distance,
            p10_distance,
            p25_distance,
            median_distance,
            p75_distance,
            worst_distance,
            average_distance: avg_dist,
            feasible_population_ratio: evals.len() as f64 / population_size as f64,
            diversity_score,
            elite_diversity_score,
            best_routes: gen_best.eval.routes.clone(),
            deepest_basin_transition,
            operator_counts: HashMap::new(),
            mutation_damage_avg: if total_memetic_children > 0 { total_mutation_damage / total_memetic_children as f64 } else { 0.0 },
            basin_depth_avg: if total_memetic_children > 0 { total_basin_depth / total_memetic_children as f64 } else { 0.0 },
            damage_edge_count_avg: if total_memetic_children > 0 { total_damage_edge_count as f64 / total_memetic_children as f64 } else { 0.0 },
            repair_edge_count_avg: if total_memetic_children > 0 { total_repair_edge_count as f64 / total_memetic_children as f64 } else { 0.0 },
            new_basin_discovery_rate: if total_memetic_children > 0 { new_basin_discoveries as f64 / total_memetic_children as f64 } else { 0.0 },
            parent_reversion_rate: if total_memetic_children > 0 { parent_reversions as f64 / total_memetic_children as f64 } else { 0.0 },
            unique_basins_seen: basin_tracker.len(),
            global_basin_revisit_rate: if total_memetic_children > 0 { global_basin_revisits as f64 / total_memetic_children as f64 } else { 0.0 },
            recent_basin_revisit_rate: if total_memetic_children > 0 { recent_basin_revisits as f64 / total_memetic_children as f64 } else { 0.0 },
            elite_basin_discovery_rate: if total_memetic_children > 0 { elite_basin_discoveries as f64 / total_memetic_children as f64 } else { 0.0 },
            generations_since_improvement,
            local_improving_generated,
            global_improving_generated,
            improving_accepted,
            improving_rejected,
            crossover_structural_damage_ratio,
            route_diversity_score,
            parent_similarity,
            offspring_novelty,
            elite_similarity,
            top_10_parent_ratio: total_top_10_parents as f64 / (population_size as f64),
            top_20_parent_ratio: total_top_20_parents as f64 / (population_size as f64),
            bottom_50_parent_ratio: total_bottom_50_parents as f64 / (population_size as f64),
            elite_offspring_survival_rate: if num_offspring > 0.0 { children_better_than_median as f64 / num_offspring } else { 0.0 },
            top10_offspring_rate: if num_offspring > 0.0 { children_entering_top10 as f64 / num_offspring } else { 0.0 },
            guidance_mode: current_guidance_mode,
            shadow_rejected_offspring: total_shadow_rejected_offspring,
            shadow_local_search_work_saved: total_shadow_rejected_offspring, // Offspring == LS calls saved
            shadow_rejected_elites: total_shadow_rejected_elites,
            mean_p_elite: if p_elite_count > 0 { total_p_elite / p_elite_count as f64 } else { 0.0 },
            calibration_error: 0.0, // Calculated dynamically
            calibration_buckets: calib_buckets,
            innovation_telemetry: None,
        };

        let mut generation_signatures = Vec::new();
        for eval in &evals {
            for route in &eval.eval.routes {
                let mut prev = 0; // Depot is 0
                for &node in route {
                    let min_node = prev.min(node);
                    let max_node = prev.max(node);
                    let sig = ((min_node as u64) << 32) | (max_node as u64);
                    generation_signatures.push(sig);
                    prev = node;
                }
                let min_node = prev.min(0);
                let max_node = prev.max(0);
                let sig = ((min_node as u64) << 32) | (max_node as u64);
                generation_signatures.push(sig);
            }
        }
        let innovation_telemetry = innovation_tracker.observe(&generation_signatures);
        gen_state.innovation_telemetry = Some(innovation_telemetry.clone());

        let progress_telemetry = progress_tracker.observe_minimization(generation, global_best_fitness);

        let mut attachments = Vec::new();
        attachments.push(coralys_core::telemetry::TelemetryAttachment {
            namespace: "moga.operator_ecology".to_string(),
            version: 1,
            payload: serde_json::json!({
                "parent_similarity": parent_similarity,
                "offspring_novelty": offspring_novelty,
                "structural_damage_ratio": crossover_structural_damage_ratio,
                "elite_survival_rate": if num_offspring > 0.0 { children_better_than_median as f64 / num_offspring } else { 0.0 },
                "top10_offspring_rate": if num_offspring > 0.0 { children_entering_top10 as f64 / num_offspring } else { 0.0 }
            }),
        });

        attachments.push(coralys_core::telemetry::TelemetryAttachment {
            namespace: "memory.ecology.v1".to_string(),
            version: 1,
            payload: serde_json::to_value(&innovation_telemetry).unwrap(),
        });

        attachments.push(coralys_core::telemetry::TelemetryAttachment {
            namespace: "progress.ecology.v1".to_string(),
            version: 1,
            payload: serde_json::to_value(&progress_telemetry).unwrap(),
        });

        let search_telemetry = coralys_core::telemetry::SearchTelemetry {
            generation,
            population: coralys_core::telemetry::PopulationTelemetry {
                best_fitness: gen_best.eval.total_distance,
                median_fitness,
                worst_fitness: worst_distance,
                diversity_score,
                elite_diversity_score,
                feasible_ratio: evals.len() as f64 / population_size as f64,
            },
            attachments,
        };

        if let Some(obs) = telemetry.last_mut() {
            obs.telemetry = Some(search_telemetry.clone());
        }

        {
            *state.current_generation.lock().await = Some(gen_state);
            *state.current_diagnostics.lock().await = diagnostics;
            if !recs.recommendations.is_empty() {
                *state.current_recommendations.lock().await = Some(recs);
            } else {
                *state.current_recommendations.lock().await = None;
            }
        }

        population = next_gen;

        if std::env::var("FAST_MODE").is_err() {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        } else {
            tokio::task::yield_now().await;
        }
    }

    *state.is_running.lock().await = false;
}
