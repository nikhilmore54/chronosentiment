#!/usr/bin/env python3
"""
RP-410 telemetry patch for campaign.rs
Updates the run_roadef_evolution() call site to pass a telemetry sink.

Behaviour:
  - If env var RP410_TELEMETRY_DIR is set, creates JSONL files in that directory
    and passes a JsonlTelemetrySink.
  - Otherwise passes NullTelemetrySink (zero overhead, existing behaviour preserved).
"""

with open('adapters/roadef/src/bin/campaign.rs', 'r') as f:
    src = f.read()

original_len = len(src)

# 1. Add telemetry import after the existing moga_impl import block
old_import = ('use roadef::moga_impl::{\n'
              '    RoadefGenomeFactory, RoadefFitnessEvaluator, RoadefMutator, RoadefCrossover,\n'
              '    EvolutionRunConfig, run_roadef_evolution,\n'
              '};')
new_import = (old_import +
              '\nuse roadef::telemetry::{NullTelemetrySink, JsonlTelemetrySink};')

if old_import in src:
    src = src.replace(old_import, new_import, 1)
    print("Import patched.")
else:
    print("WARNING: import block not found — check manually.")

# 2. Replace the run_roadef_evolution call site
old_call = ('        let run_result = run_roadef_evolution(\n'
            '            &factory,\n'
            '            &fitness_eval,\n'
            '            &mutator,\n'
            '            &crossover,\n'
            '            &evo_config,\n'
            '            name,\n'
            '            &mut *log_buf,\n'
            '        );')

new_call = (
    '        // RP-410: construct telemetry sink.\n'
    '        // Set RP410_TELEMETRY_DIR env var to enable JSONL output.\n'
    '        // Default: NullTelemetrySink (zero overhead, existing behaviour preserved).\n'
    '        let telemetry_dir = std::env::var("RP410_TELEMETRY_DIR").ok();\n'
    '        let seed_str = evo_config.seed.map(|s| s.to_string()).unwrap_or_else(|| "rand".to_string());\n'
    '        let run_result = if let Some(ref tdir) = telemetry_dir {\n'
    '            let _ = fs::create_dir_all(tdir);\n'
    '            let moves_path = format!("{}/rp410_moves_{}_{}.jsonl", tdir, name, seed_str);\n'
    '            let gens_path  = format!("{}/rp410_generations_{}_{}.jsonl", tdir, name, seed_str);\n'
    '            let moves_file = fs::File::create(&moves_path).map(|f| BufWriter::new(f));\n'
    '            let gens_file  = fs::File::create(&gens_path).map(|f| BufWriter::new(f));\n'
    '            match (moves_file, gens_file) {\n'
    '                (Ok(mf), Ok(gf)) => {\n'
    '                    let mut sink = JsonlTelemetrySink::new(mf, gf);\n'
    '                    run_roadef_evolution(\n'
    '                        &factory, &fitness_eval, &mutator, &crossover,\n'
    '                        &evo_config, name, &mut *log_buf, &mut sink,\n'
    '                    )\n'
    '                }\n'
    '                _ => {\n'
    '                    eprintln!("  [RP410] Warning: could not create telemetry files in {}", tdir);\n'
    '                    run_roadef_evolution(\n'
    '                        &factory, &fitness_eval, &mutator, &crossover,\n'
    '                        &evo_config, name, &mut *log_buf, &mut NullTelemetrySink,\n'
    '                    )\n'
    '                }\n'
    '            }\n'
    '        } else {\n'
    '            run_roadef_evolution(\n'
    '                &factory, &fitness_eval, &mutator, &crossover,\n'
    '                &evo_config, name, &mut *log_buf, &mut NullTelemetrySink,\n'
    '            )\n'
    '        };'
)

if old_call in src:
    src = src.replace(old_call, new_call, 1)
    print("Call site patched successfully.")
else:
    print("WARNING: call site not found — check manually.")
    # Debug: show what's around the call
    idx = src.find('let run_result = run_roadef_evolution(')
    if idx >= 0:
        print(f"  Found at index {idx}, context:")
        print(repr(src[idx:idx+200]))

with open('adapters/roadef/src/bin/campaign.rs', 'w') as f:
    f.write(src)

print(f"Original length: {original_len}, New length: {len(src)}")
print("Verifying key strings present:")
checks = [
    ('NullTelemetrySink import', 'NullTelemetrySink' in src),
    ('JsonlTelemetrySink import', 'JsonlTelemetrySink' in src),
    ('RP410_TELEMETRY_DIR', 'RP410_TELEMETRY_DIR' in src),
    ('NullTelemetrySink call', 'NullTelemetrySink,' in src),
]
all_ok = True
for name_c, ok in checks:
    status = 'OK' if ok else 'FAIL'
    if not ok:
        all_ok = False
    print(f"  {status}: {name_c}")
print("ALL OK" if all_ok else "SOME CHECKS FAILED")