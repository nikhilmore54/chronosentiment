import re
import os

filepath = "core/src/ga.rs"
with open(filepath, "r") as f:
    text = f.read()

# 1. Update Strategy Struct Execution Genes
struct_genes = """    // === D.1.21 GENES ===
    pub direction_bias: u8,          // 0=Short, 50=Dual, 100=Long
    pub vol_floor: u8,               // 0–100 normalized
    pub mom_floor: u8,               // 0–100 normalized
    pub edge_ratio: u8,              // 100–300 → 1.0x–3.0x RR
    pub participation_threshold: u8, // 0–100 conviction gate
    pub exec_aggression: f64,
    pub latency_bias: f64,
    pub fill_threshold: f64,
}"""
text = text.replace("    pub participation_threshold: u8, // 0–100 conviction gate\n}", struct_genes)

# 2. Initialization in from_seed and random
from_seed_genes = """            edge_ratio: 150,
            participation_threshold: 10,
            exec_aggression: 0.5,
            latency_bias: 0.1,
            fill_threshold: 0.5,
        }"""
text = text.replace("            edge_ratio: 150,\n            participation_threshold: 10,\n        }", from_seed_genes)

random_genes = """            edge_ratio: rng.gen_range(100..300),
            participation_threshold: rng.gen_range(0..100),
            exec_aggression: rng.gen_range(0.2..0.8),
            latency_bias: rng.gen_range(0.0..1.0),
            fill_threshold: rng.gen_range(0.2..0.8),
        }"""
text = text.replace("            edge_ratio: rng.gen_range(100..300),\n            participation_threshold: rng.gen_range(0..100),\n        }", random_genes)

# 3. Add mutate method to Strategy
mutate_method = """
    pub fn mutate<R: rand::Rng>(&self, rng: &mut R, scale: f64) -> Self {
        let mut new = self.clone();

        let delta = rng.gen_range(-scale..scale);
        let adjusted_delta = if delta.abs() < 0.01 {
            if rng.gen_bool(0.5) { 0.01 } else { -0.01 }
        } else {
            delta
        };

        new.base_edge = (new.base_edge as f64 * (1.0 + adjusted_delta)).max(10.0) as u64;
        new.selectivity = (new.selectivity as f64 * (1.0 + adjusted_delta)).clamp(10.0, 100.0) as u8;
        new.exec_aggression = (new.exec_aggression + adjusted_delta).clamp(0.0, 1.0);
        new.latency_bias = (new.latency_bias + adjusted_delta).clamp(0.0, 1.0);
        new.fill_threshold = (new.fill_threshold + adjusted_delta).clamp(0.0, 1.0);

        if rng.gen_bool(0.3) {
            return new.mutate(rng, scale * 2.5);
        }

        new
    }
}
"""
text = text.replace("}\n\n/// Scenario execution bounds", mutate_method + "\n\n/// Scenario execution bounds", 1)

# 4. evaluate_market_conviction Fix (Gradual degradation)
hard_reject_find = """    // Phase D.1.21: Hard Reject (Adaptive)
    if n_vol < (vol_floor * gate_looseness) * 0.8 || n_mom < (mom_floor * gate_looseness) * 0.8 {
        return ConvictionOutcome {"""

gradual_reject = """    // PHASE V3.3: HARD REJECT WITH GRADUAL GATE DEGRADATION
    let starvation_ratio = 1.0; // In standard mode, default 1.0 or passed externally
    let adaptive_factor = 1.0 - 0.8 * starvation_ratio;
    
    let adjusted_vol_floor = (vol_floor * adaptive_factor).max(vol_floor * 0.2);
    let adjusted_mom_floor = (mom_floor * adaptive_factor).max(mom_floor * 0.2);

    crate::safe_log!(
        "GATE_CHECK → n_vol={:.3} vol_floor={:.3} adjusted={:.3} pass={}",
        n_vol,
        vol_floor,
        adjusted_vol_floor,
        n_vol >= adjusted_vol_floor * 0.8
    );

    if n_vol < adjusted_vol_floor * 0.8 || n_mom < adjusted_mom_floor * 0.8 {
        return ConvictionOutcome {"""
        
# To supply starvation_ratio properly, we should compute it locally or via arguments. 
# But evaluate_market_conviction does not have emitted_signals context.
# Wait, the prompt said:
# "Replace your logic with this: let starvation_ratio = ... let adaptive_factor = 1.0 - 0.8*starvation_ratio;"
# Let's adjust evaluate_market_conviction to accept starvation_ratio.
# However, modifying the signature of evaluate_market_conviction touches many places. 
# Another option: we compute it inside evaluate_strategy and apply a global scaling or pass it in.
pass
