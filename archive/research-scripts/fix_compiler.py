import re

def main():
    path = "infrastructure/optimization/src/evolution_engine.rs"
    with open(path, "r") as f:
        content = f.read()

    # Fix )f64.clamp
    content = re.sub(r'\)f64\.clamp', ').clamp', content)
    
    # Fix variablef64.clamp
    content = re.sub(r'([a-zA-Z_]+)f64\.clamp', r'\1.clamp', content)
    
    # Fix dangling f64.clamp
    content = re.sub(r'\n\s*f64\.clamp', '\n            .clamp', content)
    
    # Also fix any extra parens that caused warnings:
    # "if rng.gen_bool((0.35 * evo.mutation_scale)f64.clamp(0.0, 1.0)) {"
    # after the above replace becomes "if rng.gen_bool((0.35 * evo.mutation_scale).clamp(0.0, 1.0)) {"
    # The compiler warned about `gen_bool( (x) )`. We don't need to fix the warning immediately, just the compile errors.

    # Let's fix the specific warnings just in case it's easy:
    content = content.replace("rng.gen_bool((0.35 * evo.mutation_scale).clamp(0.0, 1.0))", "rng.gen_bool((0.35 * evo.mutation_scale).clamp(0.0, 1.0))")
    
    with open(path, "w") as f:
        f.write(content)

if __name__ == "__main__":
    main()
