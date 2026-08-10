import os
import shutil
import re

def main():
    opt_dir = "infrastructure/optimization/src"
    fin_dir = "financial/strategies/src"

    # 1. Move edge_decay and edge_half_life_estimator
    for f in ["edge_decay.rs", "edge_half_life_estimator.rs"]:
        src = os.path.join(opt_dir, f)
        dst = os.path.join(fin_dir, f)
        if os.path.exists(src):
            shutil.move(src, dst)

    # 2. Fix lib.rs
    lib_path = os.path.join(opt_dir, "lib.rs")
    if os.path.exists(lib_path):
        with open(lib_path, "r") as f:
            lib_content = f.read()
        lib_content = lib_content.replace("pub mod edge_decay;\n", "")
        lib_content = lib_content.replace("pub mod edge_half_life_estimator;\n", "")
        with open(lib_path, "w") as f:
            f.write(lib_content)

    # 3. Clean CandidateEvaluation in evolution_engine.rs
    evo_path = os.path.join(opt_dir, "evolution_engine.rs")
    with open(evo_path, "r") as f:
        evo_content = f.read()

    # Add CandidateAnnotation struct if it doesn't exist
    annotation_struct = """
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandidateAnnotation {
    pub key: String,
    pub value: String,
}
"""
    if "pub struct CandidateAnnotation" not in evo_content:
        evo_content = evo_content.replace("pub struct CandidateEvaluation", annotation_struct + "\npub struct CandidateEvaluation")

    # Replace emitted_signals and pnl_history
    evo_content = evo_content.replace("pub emitted_signals: Vec<SignalAlpha>,", "pub annotations: Vec<CandidateAnnotation>,")
    evo_content = evo_content.replace("pub pnl_history: Vec<GaRoundTripOutcome>,", "pub score_history: Vec<f64>,")

    # 4. Fix f64f64.clamp and ambiguous clamps
    evo_content = evo_content.replace("f64f64.clamp", "f64.clamp")
    evo_content = evo_content.replace(".clamp(-1.0, 1.0)", ".clamp(-1.0_f64, 1.0_f64)")
    evo_content = evo_content.replace(".clamp(0.0, 1.0)", ".clamp(0.0_f64, 1.0_f64)")
    evo_content = evo_content.replace(".clamp(-2.0, 2.0)", ".clamp(-2.0_f64, 2.0_f64)")
    
    # 5. Missing generic variable types in max/min calls
    evo_content = evo_content.replace("let mut total_weight = 0.0;", "let mut total_weight: f64 = 0.0;")
    evo_content = evo_content.replace("let mut agreement = 0.5 * agreement_delta + 0.5 * agreement_capacity;", "let mut agreement: f64 = 0.5 * agreement_delta + 0.5 * agreement_capacity;")
    
    with open(evo_path, "w") as f:
        f.write(evo_content)

if __name__ == "__main__":
    main()
