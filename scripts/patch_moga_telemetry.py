#!/usr/bin/env python3
"""
RP-410 telemetry patch for moga_impl.rs
Applies 7 targeted string replacements to wire telemetry into the evolution loop.
"""

with open('adapters/roadef/src/moga_impl.rs', 'r') as f:
    src = f.read()

original_len = len(src)

# 1. Add telemetry use statement
src = src.replace(
    'use crate::evaluator::RoadefEvaluator;\nuse crate::models::{Solution, SrPath};',
    'use crate::evaluator::RoadefEvaluator;\nuse crate::models::{Solution, SrPath};\nuse crate::telemetry::{TelemetrySink, MoveRecord, GenerationRecord, ZoneDeltas, sorted_load_vector, compute_sdi};',
    1
)

# 2. Add load_vector field to RoadefEvaluation
src = src.replace(
    'pub struct RoadefEvaluation {\n    pub genome: RoadefGenome,\n    pub obj: f64,\n    pub valid: bool,\n    pub mlu: f64,\n}',
    'pub struct RoadefEvaluation {\n    pub genome: RoadefGenome,\n    pub obj: f64,\n    pub valid: bool,\n    pub mlu: f64,\n    /// Sorted arc-saturation load vector (descending). Used by RP-410 telemetry.\n    pub load_vector: Vec<f64>,\n}',
    1
)

# 3. Populate load_vector in evaluate()
src = src.replace(
    '        RoadefEvaluation {\n            genome: genome.clone(),\n            obj: result.obj,\n            valid,\n            mlu,\n        }',
    '        // Compute sorted load vector for RP-410 telemetry (only for valid solutions)\n        let load_vector = if valid {\n            let mut all_sats: Vec<f64> = Vec::new();\n            for t in 0..genome.num_time_slots {\n                if let Some(loads) = self.evaluator.compute_loads(t, &solution) {\n                    for sat in loads.arc_saturations.values() {\n                        all_sats.push(*sat);\n                    }\n                }\n            }\n            sorted_load_vector(&all_sats)\n        } else {\n            Vec::new()\n        };\n\n        RoadefEvaluation {\n            genome: genome.clone(),\n            obj: result.obj,\n            valid,\n            mlu,\n            load_vector,\n        }',
    1
)

# 4. Add telemetry parameter to run_roadef_evolution signature
src = src.replace(
    'pub fn run_roadef_evolution(\n    factory: &RoadefGenomeFactory,\n    fitness_eval: &RoadefFitnessEvaluator,\n    mutator: &RoadefMutator,\n    crossover: &RoadefCrossover,\n    config: &EvolutionRunConfig,\n    instance_name: &str,\n    log_sink: &mut dyn Write,\n) -> EvolutionRunResult {',
    'pub fn run_roadef_evolution(\n    factory: &RoadefGenomeFactory,\n    fitness_eval: &RoadefFitnessEvaluator,\n    mutator: &RoadefMutator,\n    crossover: &RoadefCrossover,\n    config: &EvolutionRunConfig,\n    instance_name: &str,\n    log_sink: &mut dyn Write,\n    telemetry: &mut dyn TelemetrySink,\n) -> EvolutionRunResult {',
    1
)

# 5. Emit MoveRecord at the improvement point
src = src.replace(
    '            global_best = Some(gen_best.clone());\n            best_found_at_gen = gen;\n            stagnation = 0;',
    '            // RP-410: emit MoveRecord for this accepted improvement\n            if let Some(ref prev) = global_best {\n                let deltas = ZoneDeltas::compute(&prev.load_vector, &gen_best.load_vector);\n                let move_class = deltas.classify(1e-9).to_string();\n                let new_sdi = compute_sdi(&gen_best.load_vector);\n                let move_rec = MoveRecord {\n                    record_type: "move",\n                    instance: instance_name.to_string(),\n                    seed: config.seed.unwrap_or(0),\n                    generation: gen as u32,\n                    operator: "evolution",\n                    deltas,\n                    move_class,\n                    new_obj: if gen_best.is_valid() { -gen_best.fitness() } else { f64::INFINITY },\n                    prev_obj: if prev.is_valid() { -prev.fitness() } else { f64::INFINITY },\n                    new_mlu: gen_best.mlu,\n                    new_sdi,\n                };\n                telemetry.emit_move(&move_rec);\n            }\n            global_best = Some(gen_best.clone());\n            best_found_at_gen = gen;\n            stagnation = 0;',
    1
)

# 6. Emit GenerationRecord before building next generation
src = src.replace(
    '        // --- Build next generation ---\n        let elite_count = config.elite_count.min(evals.len());',
    '        // --- RP-410: emit GenerationRecord ---\n        {\n            let best_sdi = global_best.as_ref()\n                .map(|g| compute_sdi(&g.load_vector))\n                .unwrap_or(0.0);\n            let top20_prefix: Vec<f64> = global_best.as_ref()\n                .map(|g| g.load_vector.iter().take(20).cloned().collect())\n                .unwrap_or_default();\n            let unique_fitness_count = {\n                let unique: std::collections::HashSet<String> = evals.iter()\n                    .map(|e| format!("{:.6}", e.fitness()))\n                    .collect();\n                unique.len()\n            };\n            let gen_rec = GenerationRecord {\n                record_type: "generation",\n                instance: instance_name.to_string(),\n                seed: config.seed.unwrap_or(0),\n                generation: gen as u32,\n                best_obj: global_best.as_ref()\n                    .map(|g| if g.is_valid() { -g.fitness() } else { f64::INFINITY })\n                    .unwrap_or(f64::INFINITY),\n                best_mlu: global_best.as_ref().map(|g| g.mlu).unwrap_or(f64::INFINITY),\n                best_sdi,\n                top20_prefix,\n                valid_count: evals.iter().filter(|e| e.is_valid()).count(),\n                population_size: config.population_size,\n                unique_fitness_count,\n                stagnation,\n            };\n            telemetry.emit_generation(&gen_rec);\n        }\n\n        // --- Build next generation ---\n        let elite_count = config.elite_count.min(evals.len());',
    1
)

# 7. Flush telemetry before returning
src = src.replace(
    '    EvolutionRunResult {\n        best_genome: best.map(|g| g.genome().clone()).unwrap_or_else(|| factory.create(&mut rng)),',
    '    telemetry.flush();\n\n    EvolutionRunResult {\n        best_genome: best.map(|g| g.genome().clone()).unwrap_or_else(|| factory.create(&mut rng)),',
    1
)

with open('adapters/roadef/src/moga_impl.rs', 'w') as f:
    f.write(src)

print(f"Original length: {original_len}, New length: {len(src)}")
print("Verifying key strings present:")
checks = [
    ('pub mod telemetry in lib.rs', 'pub mod telemetry' in open('adapters/roadef/src/lib.rs').read()),
    ('use crate::telemetry', 'use crate::telemetry' in src),
    ('pub load_vector: Vec<f64>', 'pub load_vector: Vec<f64>' in src),
    ('telemetry: &mut dyn TelemetrySink', 'telemetry: &mut dyn TelemetrySink' in src),
    ('emit_move', 'emit_move' in src),
    ('emit_generation', 'emit_generation' in src),
    ('telemetry.flush()', 'telemetry.flush()' in src),
]
all_ok = True
for name, ok in checks:
    status = 'OK' if ok else 'FAIL'
    if not ok:
        all_ok = False
    print(f"  {status}: {name}")
print("ALL OK" if all_ok else "SOME CHECKS FAILED")