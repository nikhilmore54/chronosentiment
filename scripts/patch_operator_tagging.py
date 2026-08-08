#!/usr/bin/env python3
"""
RP-410 operator tagging patch for moga_impl.rs and telemetry.rs

Changes:
1. telemetry.rs: Add histogram fields to GenerationRecord
2. moga_impl.rs: Add operator field to RoadefEvaluation
3. moga_impl.rs: Tag children with their origin operator during next-gen build
4. moga_impl.rs: Accumulate per-generation histogram and operator counts
5. moga_impl.rs: Read gen_best.operator when emitting MoveRecord
6. moga_impl.rs: Pass histogram/counts into GenerationRecord
"""

# ============================================================
# Patch 1: telemetry.rs — add histogram fields to GenerationRecord
# ============================================================
with open('adapters/roadef/src/telemetry.rs', 'r') as f:
    tel = f.read()

old_gen_record = (
    '/// Emitted once per generation for the population best.\n'
    '#[derive(Debug, Clone, serde::Serialize)]\n'
    'pub struct GenerationRecord {\n'
    '    /// Record type tag for JSONL filtering.\n'
    '    pub record_type: &\'static str,\n'
    '    /// Instance identifier.\n'
    '    pub instance: String,\n'
    '    /// Random seed.\n'
    '    pub seed: u64,\n'
    '    /// Generation index.\n'
    '    pub generation: u32,\n'
    '    /// Best objective in this generation (scalar).\n'
    '    pub best_obj: f64,\n'
    '    /// Best MLU in this generation.\n'
    '    pub best_mlu: f64,\n'
    '    /// SDI of the best individual.\n'
    '    pub best_sdi: f64,\n'
    '    /// Top-20 prefix of the best individual\'s load vector.\n'
    '    pub top20_prefix: Vec<f64>,\n'
    '    /// Number of valid individuals in the population.\n'
    '    pub valid_count: usize,\n'
    '    /// Population size.\n'
    '    pub population_size: usize,\n'
    '    /// Number of unique fitness values (diversity proxy).\n'
    '    pub unique_fitness_count: usize,\n'
    '    /// Stagnation counter at this generation.\n'
    '    pub stagnation: usize,\n'
    '}'
)

new_gen_record = (
    '/// Emitted once per generation for the population best.\n'
    '#[derive(Debug, Clone, serde::Serialize)]\n'
    'pub struct GenerationRecord {\n'
    '    /// Record type tag for JSONL filtering.\n'
    '    pub record_type: &\'static str,\n'
    '    /// Instance identifier.\n'
    '    pub instance: String,\n'
    '    /// Random seed.\n'
    '    pub seed: u64,\n'
    '    /// Generation index.\n'
    '    pub generation: u32,\n'
    '    /// Best objective in this generation (scalar).\n'
    '    pub best_obj: f64,\n'
    '    /// Best MLU in this generation.\n'
    '    pub best_mlu: f64,\n'
    '    /// SDI of the best individual.\n'
    '    pub best_sdi: f64,\n'
    '    /// Top-20 prefix of the best individual\'s load vector.\n'
    '    pub top20_prefix: Vec<f64>,\n'
    '    /// Number of valid individuals in the population.\n'
    '    pub valid_count: usize,\n'
    '    /// Population size.\n'
    '    pub population_size: usize,\n'
    '    /// Number of unique fitness values (diversity proxy).\n'
    '    pub unique_fitness_count: usize,\n'
    '    /// Stagnation counter at this generation.\n'
    '    pub stagnation: usize,\n'
    '    // --- RP-410 per-generation improvement histogram ---\n'
    '    /// Accepted global-best improvements this generation classified as Peak.\n'
    '    pub moves_peak: u32,\n'
    '    /// Accepted global-best improvements this generation classified as Shoulder.\n'
    '    pub moves_shoulder: u32,\n'
    '    /// Accepted global-best improvements this generation classified as Transition.\n'
    '    pub moves_transition: u32,\n'
    '    /// Accepted global-best improvements this generation classified as Tail.\n'
    '    pub moves_tail: u32,\n'
    '    /// Accepted global-best improvements this generation classified as Mixed.\n'
    '    pub moves_mixed: u32,\n'
    '    /// Accepted global-best improvements this generation classified as Neutral.\n'
    '    pub moves_neutral: u32,\n'
    '    // --- RP-410 per-generation operator usage counts ---\n'
    '    /// Number of crossover operations applied this generation.\n'
    '    pub crossover_count: u32,\n'
    '    /// Number of mutation-only operations applied this generation.\n'
    '    pub mutation_count: u32,\n'
    '}'
)

if old_gen_record in tel:
    tel = tel.replace(old_gen_record, new_gen_record, 1)
    print("telemetry.rs: GenerationRecord extended with histogram fields.")
else:
    print("WARNING: GenerationRecord not found in telemetry.rs — check manually.")

with open('adapters/roadef/src/telemetry.rs', 'w') as f:
    f.write(tel)

# ============================================================
# Patch 2–6: moga_impl.rs — operator tagging and histogram
# ============================================================
with open('adapters/roadef/src/moga_impl.rs', 'r') as f:
    src = f.read()

original_len = len(src)

# 2. Add operator field to RoadefEvaluation
old_eval_struct = (
    'pub struct RoadefEvaluation {\n'
    '    pub genome: RoadefGenome,\n'
    '    pub obj: f64,\n'
    '    pub valid: bool,\n'
    '    pub mlu: f64,\n'
    '    /// Sorted arc-saturation load vector (descending). Used by RP-410 telemetry.\n'
    '    pub load_vector: Vec<f64>,\n'
    '}'
)
new_eval_struct = (
    'pub struct RoadefEvaluation {\n'
    '    pub genome: RoadefGenome,\n'
    '    pub obj: f64,\n'
    '    pub valid: bool,\n'
    '    pub mlu: f64,\n'
    '    /// Sorted arc-saturation load vector (descending). Used by RP-410 telemetry.\n'
    '    pub load_vector: Vec<f64>,\n'
    '    /// Origin operator tag for RP-410 telemetry.\n'
    '    /// Values: "crossover", "crossover+mutation", "mutation", "elite", "initial"\n'
    '    pub operator: &\'static str,\n'
    '}'
)
if old_eval_struct in src:
    src = src.replace(old_eval_struct, new_eval_struct, 1)
    print("moga_impl.rs: operator field added to RoadefEvaluation.")
else:
    print("WARNING: RoadefEvaluation struct not found.")

# 3. Add operator: "initial" to the evaluate() return (initial population)
old_eval_return = (
    '        RoadefEvaluation {\n'
    '            genome: genome.clone(),\n'
    '            obj: result.obj,\n'
    '            valid,\n'
    '            mlu,\n'
    '            load_vector,\n'
    '        }'
)
new_eval_return = (
    '        RoadefEvaluation {\n'
    '            genome: genome.clone(),\n'
    '            obj: result.obj,\n'
    '            valid,\n'
    '            mlu,\n'
    '            load_vector,\n'
    '            operator: "initial",\n'
    '        }'
)
if old_eval_return in src:
    src = src.replace(old_eval_return, new_eval_return, 1)
    print("moga_impl.rs: evaluate() return tagged with operator: initial.")
else:
    print("WARNING: evaluate() return not found.")

# 4. Add per-generation counters before the next-gen build loop
# Insert after the GenerationRecord emit block, before "// --- Build next generation ---"
old_build_header = (
    '        // --- Build next generation ---\n'
    '        let elite_count = config.elite_count.min(evals.len());\n'
    '        let mut next_pop: Vec<RoadefGenome> = evals[..elite_count]\n'
    '            .iter()\n'
    '            .map(|e| e.genome().clone())\n'
    '            .collect();\n'
    '\n'
    '        while next_pop.len() < config.population_size {\n'
    '            // Tournament selection (k=3)\n'
    '            let select = |rng: &mut StdRng| -> &RoadefEvaluation {\n'
    '                let k = 3.min(evals.len());\n'
    '                let mut best_idx = rng.gen_range(0..evals.len());\n'
    '                for _ in 1..k {\n'
    '                    let idx = rng.gen_range(0..evals.len());\n'
    '                    if evals[idx].fitness() > evals[best_idx].fitness() {\n'
    '                        best_idx = idx;\n'
    '                    }\n'
    '                }\n'
    '                &evals[best_idx]\n'
    '            };\n'
    '\n'
    '            if rng.gen_bool(config.crossover_rate) && next_pop.len() + 1 < config.population_size {\n'
    '                let pa = select(&mut rng).genome().clone();\n'
    '                let pb = select(&mut rng).genome().clone();\n'
    '                let (mut ca, mut cb) = crossover.crossover(&pa, &pb, &mut rng);\n'
    '                if rng.gen_bool(config.mutation_rate) { mutator.mutate(&mut ca, &mut rng); }\n'
    '                if rng.gen_bool(config.mutation_rate) { mutator.mutate(&mut cb, &mut rng); }\n'
    '                next_pop.push(ca);\n'
    '                if next_pop.len() < config.population_size { next_pop.push(cb); }\n'
    '            } else {\n'
    '                let mut child = select(&mut rng).genome().clone();\n'
    '                mutator.mutate(&mut child, &mut rng);\n'
    '                next_pop.push(child);\n'
    '            }\n'
    '        }'
)

new_build_header = (
    '        // --- Build next generation ---\n'
    '        // RP-410: per-generation operator usage counters\n'
    '        let mut gen_crossover_count: u32 = 0;\n'
    '        let mut gen_mutation_count: u32 = 0;\n'
    '\n'
    '        let elite_count = config.elite_count.min(evals.len());\n'
    '        let mut next_pop: Vec<(RoadefGenome, &\'static str)> = evals[..elite_count]\n'
    '            .iter()\n'
    '            .map(|e| (e.genome().clone(), "elite"))\n'
    '            .collect();\n'
    '\n'
    '        while next_pop.len() < config.population_size {\n'
    '            // Tournament selection (k=3)\n'
    '            let select = |rng: &mut StdRng| -> &RoadefEvaluation {\n'
    '                let k = 3.min(evals.len());\n'
    '                let mut best_idx = rng.gen_range(0..evals.len());\n'
    '                for _ in 1..k {\n'
    '                    let idx = rng.gen_range(0..evals.len());\n'
    '                    if evals[idx].fitness() > evals[best_idx].fitness() {\n'
    '                        best_idx = idx;\n'
    '                    }\n'
    '                }\n'
    '                &evals[best_idx]\n'
    '            };\n'
    '\n'
    '            if rng.gen_bool(config.crossover_rate) && next_pop.len() + 1 < config.population_size {\n'
    '                let pa = select(&mut rng).genome().clone();\n'
    '                let pb = select(&mut rng).genome().clone();\n'
    '                let (mut ca, mut cb) = crossover.crossover(&pa, &pb, &mut rng);\n'
    '                let mut ca_tag: &\'static str = "crossover";\n'
    '                let mut cb_tag: &\'static str = "crossover";\n'
    '                if rng.gen_bool(config.mutation_rate) { mutator.mutate(&mut ca, &mut rng); ca_tag = "crossover+mutation"; }\n'
    '                if rng.gen_bool(config.mutation_rate) { mutator.mutate(&mut cb, &mut rng); cb_tag = "crossover+mutation"; }\n'
    '                gen_crossover_count += 1;\n'
    '                next_pop.push((ca, ca_tag));\n'
    '                if next_pop.len() < config.population_size { next_pop.push((cb, cb_tag)); }\n'
    '            } else {\n'
    '                let mut child = select(&mut rng).genome().clone();\n'
    '                mutator.mutate(&mut child, &mut rng);\n'
    '                gen_mutation_count += 1;\n'
    '                next_pop.push((child, "mutation"));\n'
    '            }\n'
    '        }'
)

if old_build_header in src:
    src = src.replace(old_build_header, new_build_header, 1)
    print("moga_impl.rs: next-gen build loop patched with operator tagging.")
else:
    print("WARNING: next-gen build loop not found.")

# 5. Update population assignment to extract genomes from tagged tuples
old_pop_assign = '        population = next_pop;'
new_pop_assign = '        population = next_pop.into_iter().map(|(g, _)| g).collect();'
if old_pop_assign in src:
    src = src.replace(old_pop_assign, new_pop_assign, 1)
    print("moga_impl.rs: population assignment updated.")
else:
    print("WARNING: population assignment not found.")

# 6. Update MoveRecord to use gen_best.operator instead of hardcoded "evolution"
old_move_op = '                    operator: "evolution",'
new_move_op = '                    operator: gen_best.operator,'
if old_move_op in src:
    src = src.replace(old_move_op, new_move_op, 1)
    print("moga_impl.rs: MoveRecord.operator now reads from gen_best.operator.")
else:
    print("WARNING: hardcoded operator field not found.")

# 7. Add per-generation move histogram accumulation before the GenerationRecord emit
# We need to add histogram counters at the start of the generation loop body
# and accumulate them when a move is accepted.
# Insert histogram counters at the start of the generation loop (after "for gen in 0..config.max_generations {")
old_gen_loop_start = (
    '        // --- Evaluate population ---\n'
    '        let mut evals: Vec<RoadefEvaluation> = population\n'
    '            .iter()\n'
    '            .map(|g| fitness_eval.evaluate(g))\n'
    '            .collect();\n'
    '        evals.sort_by(|a, b| b.fitness().partial_cmp(&a.fitness()).unwrap_or(std::cmp::Ordering::Equal));'
)
new_gen_loop_start = (
    '        // RP-410: per-generation improvement histogram counters\n'
    '        let mut gen_moves_peak: u32 = 0;\n'
    '        let mut gen_moves_shoulder: u32 = 0;\n'
    '        let mut gen_moves_transition: u32 = 0;\n'
    '        let mut gen_moves_tail: u32 = 0;\n'
    '        let mut gen_moves_mixed: u32 = 0;\n'
    '        let mut gen_moves_neutral: u32 = 0;\n'
    '\n'
    '        // --- Evaluate population ---\n'
    '        let mut evals: Vec<RoadefEvaluation> = population\n'
    '            .iter()\n'
    '            .map(|g| fitness_eval.evaluate(g))\n'
    '            .collect();\n'
    '        evals.sort_by(|a, b| b.fitness().partial_cmp(&a.fitness()).unwrap_or(std::cmp::Ordering::Equal));'
)
if old_gen_loop_start in src:
    src = src.replace(old_gen_loop_start, new_gen_loop_start, 1)
    print("moga_impl.rs: per-generation histogram counters added.")
else:
    print("WARNING: generation loop start not found.")

# 8. Accumulate histogram when a move is accepted (after emit_move)
old_after_emit = (
    '                telemetry.emit_move(&move_rec);\n'
    '            }\n'
    '            global_best = Some(gen_best.clone());\n'
    '            best_found_at_gen = gen;\n'
    '            stagnation = 0;'
)
new_after_emit = (
    '                telemetry.emit_move(&move_rec);\n'
    '                // Accumulate histogram\n'
    '                match move_class.as_str() {\n'
    '                    "peak"       => gen_moves_peak += 1,\n'
    '                    "shoulder"   => gen_moves_shoulder += 1,\n'
    '                    "transition" => gen_moves_transition += 1,\n'
    '                    "tail"       => gen_moves_tail += 1,\n'
    '                    "mixed"      => gen_moves_mixed += 1,\n'
    '                    _            => gen_moves_neutral += 1,\n'
    '                }\n'
    '            }\n'
    '            global_best = Some(gen_best.clone());\n'
    '            best_found_at_gen = gen;\n'
    '            stagnation = 0;'
)
if old_after_emit in src:
    src = src.replace(old_after_emit, new_after_emit, 1)
    print("moga_impl.rs: histogram accumulation added after emit_move.")
else:
    print("WARNING: emit_move block not found for histogram accumulation.")

# 9. Pass histogram and operator counts into GenerationRecord
old_gen_rec = (
    '            let gen_rec = GenerationRecord {\n'
    '                record_type: "generation",\n'
    '                instance: instance_name.to_string(),\n'
    '                seed: config.seed.unwrap_or(0),\n'
    '                generation: gen as u32,\n'
    '                best_obj: global_best.as_ref()\n'
    '                    .map(|g| if g.is_valid() { -g.fitness() } else { f64::INFINITY })\n'
    '                    .unwrap_or(f64::INFINITY),\n'
    '                best_mlu: global_best.as_ref().map(|g| g.mlu).unwrap_or(f64::INFINITY),\n'
    '                best_sdi,\n'
    '                top20_prefix,\n'
    '                valid_count: evals.iter().filter(|e| e.is_valid()).count(),\n'
    '                population_size: config.population_size,\n'
    '                unique_fitness_count,\n'
    '                stagnation,\n'
    '            };'
)
new_gen_rec = (
    '            let gen_rec = GenerationRecord {\n'
    '                record_type: "generation",\n'
    '                instance: instance_name.to_string(),\n'
    '                seed: config.seed.unwrap_or(0),\n'
    '                generation: gen as u32,\n'
    '                best_obj: global_best.as_ref()\n'
    '                    .map(|g| if g.is_valid() { -g.fitness() } else { f64::INFINITY })\n'
    '                    .unwrap_or(f64::INFINITY),\n'
    '                best_mlu: global_best.as_ref().map(|g| g.mlu).unwrap_or(f64::INFINITY),\n'
    '                best_sdi,\n'
    '                top20_prefix,\n'
    '                valid_count: evals.iter().filter(|e| e.is_valid()).count(),\n'
    '                population_size: config.population_size,\n'
    '                unique_fitness_count,\n'
    '                stagnation,\n'
    '                moves_peak: gen_moves_peak,\n'
    '                moves_shoulder: gen_moves_shoulder,\n'
    '                moves_transition: gen_moves_transition,\n'
    '                moves_tail: gen_moves_tail,\n'
    '                moves_mixed: gen_moves_mixed,\n'
    '                moves_neutral: gen_moves_neutral,\n'
    '                crossover_count: gen_crossover_count,\n'
    '                mutation_count: gen_mutation_count,\n'
    '            };'
)
if old_gen_rec in src:
    src = src.replace(old_gen_rec, new_gen_rec, 1)
    print("moga_impl.rs: GenerationRecord construction updated with histogram fields.")
else:
    print("WARNING: GenerationRecord construction not found.")

with open('adapters/roadef/src/moga_impl.rs', 'w') as f:
    f.write(src)

print(f"\nOriginal length: {original_len}, New length: {len(src)}")
print("\nVerifying key strings:")
checks = [
    ('operator field in RoadefEvaluation', 'pub operator: &\'static str,' in src),
    ('operator: "initial" in evaluate()', 'operator: "initial",' in src),
    ('gen_crossover_count', 'gen_crossover_count' in src),
    ('gen_mutation_count', 'gen_mutation_count' in src),
    ('gen_moves_peak', 'gen_moves_peak' in src),
    ('gen_best.operator in MoveRecord', 'operator: gen_best.operator,' in src),
    ('crossover_count in GenerationRecord', 'crossover_count: gen_crossover_count,' in src),
    ('moves_peak in GenerationRecord', 'moves_peak: gen_moves_peak,' in src),
]
all_ok = True
for name_c, ok in checks:
    status = 'OK' if ok else 'FAIL'
    if not ok:
        all_ok = False
    print(f"  {status}: {name_c}")
print("\nALL OK" if all_ok else "\nSOME CHECKS FAILED")