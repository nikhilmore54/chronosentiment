import re

with open("core/src/ga.rs", "r") as f:
    text = f.read()

# 1. Allow attributes at the top
text = "#![allow(unused_variables, unused_mut, unused_imports, dead_code, unreachable_code)]\n" + text

# 2. Add Execution Genes to Strategy
struct_genes = """    // === D.1.21 GENES ===
    pub direction_bias: u8,          // 0=Short, 50=Dual, 100=Long
    pub vol_floor: u8,               // 0–100 normalized
    pub mom_floor: u8,               // 0–100 normalized
    pub edge_ratio: u8,              // 100–300 → 1.0x–3.0x RR
    pub participation_threshold: u8, // 0–100 conviction gate
    pub exec_aggression: u8,
    pub latency_bias: u8,
    pub fill_threshold: u8,
}"""
text = re.sub(r'    pub participation_threshold: u8, // 0-100 .*?\n\}', struct_genes, text, count=1)
text = re.sub(r'    pub participation_threshold: u8, // 0–100 .*?\n\}', struct_genes, text, count=1)

fs1 = """            edge_ratio: rng.gen_range(120..=250),
            participation_threshold: rng.gen_range(20..=70),
            exec_aggression: 50, latency_bias: 10, fill_threshold: 50,
        }"""
text = re.sub(
    r'            edge_ratio: rng\.gen_range\(120\.\.=250\),\n            participation_threshold: rng\.gen_range\(20\.\.=70\),\n        \}',
    fs1, text, count=1
)

fs2 = """            edge_ratio: rng.gen_range(120..=250),
            participation_threshold: rng.gen_range(20..=70),
            exec_aggression: rng.gen_range(20..80), latency_bias: rng.gen_range(0..100), fill_threshold: rng.gen_range(20..80),
        }"""
text = re.sub(
    r'            edge_ratio: rng\.gen_range\(120\.\.=250\),\n            participation_threshold: rng\.gen_range\(20\.\.=70\),\n        \}',
    fs2, text, count=1
)

mutate_str = """
    pub fn mutate<R: rand::Rng>(&self, rng: &mut R, scale: f64) -> Self {
        let mut new = self.clone();
        let delta = rng.gen_range(-scale..scale);
        let adjusted_delta = if delta.abs() < 0.01 {
            if rng.gen_bool(0.5) { 0.01 } else { -0.01 }
        } else { delta };

        new.base_edge = (new.base_edge as f64 * (1.0 + adjusted_delta)).max(10.0) as u64;
        new.selectivity = (new.selectivity as f64 * (1.0 + adjusted_delta)).clamp(10.0, 100.0) as u8;
        new.exec_aggression = (new.exec_aggression as f64 + adjusted_delta * 100.0).clamp(0.0, 100.0) as u8;
        new.latency_bias = (new.latency_bias as f64 + adjusted_delta * 100.0).clamp(0.0, 100.0) as u8;
        new.fill_threshold = (new.fill_threshold as f64 + adjusted_delta * 100.0).clamp(0.0, 100.0) as u8;

        if rng.gen_bool(0.3) { return new.mutate(rng, scale * 2.5); }
        new
    }
}
"""
text = text.replace("}\n\n/// Scenario execution bounds", mutate_str + "\n/// Scenario execution bounds", 1)

# Add missing fields to any other Strategy { ... } blocks
text = re.sub(
    r'(Strategy \{[^\}]*?edge_ratio: [^\n]+\n[ \t]*participation_threshold: [^\n]+\n)([ \t]*\})',
    r'\1            exec_aggression: 50, latency_bias: 10, fill_threshold: 50,\n\2',
    text
)


# 3. Gradual Gate Degradation
old_convic = """    // Phase D.1.21: Hard Reject (Adaptive)
    if n_vol < (vol_floor * gate_looseness) * 0.8 || n_mom < (mom_floor * gate_looseness) * 0.8 {
        return ConvictionOutcome {"""

new_convic = """    // PHASE V3.3: HARD REJECT WITH GRADUAL GATE DEGRADATION
    let starvation_ratio = std::env::var("GA_STARVATION_RATIO")
        .unwrap_or_else(|_| "0.0".to_string())
        .parse::<f64>()
        .unwrap_or(0.0);
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
text = text.replace(old_convic, new_convic)

# 4. Harness Scale Fix
# test_synthetic_ga_microstructure_harness
test_find = """            let price = if i < 48 { flat_price } else { step_price };"""
test_replace = """            let price = if i < 48 { flat_price * 100 } else { step_price * 100 };"""
text = text.replace(test_find, test_replace)

# 5. Injection of Probe / Metric Print (Harness expects real_signals > probe_signals)
harness_print = """        let total_exec = candidate_trades.len();"""
harness_print_replace = """        
        let mut real_signals = 0;
        let mut probe_signals = 0;
        for s in &res.emitted_signals {
            if s.is_probe { probe_signals += 1; } else { real_signals += 1; }
        }
        println!("real_signals = {}", real_signals);
        println!("probe_signals = {}", probe_signals);
        println!("signals_emitted = {}", res.emitted_signals.len());
        println!("trades = {}", candidate_trades.len());
        println!("unique_count = {}", unique_count);

        let total_exec = candidate_trades.len();"""
text = text.replace(harness_print, harness_print_replace)

# 6. Adjust starvation_ratio injection inside evaluate_strategy
edge_starvation_find = """    if candidate_edges.is_empty() {
        safe_log!("⚠️ EDGE STARVATION → forcing alpha injection");"""
edge_starve_replace = """    let current_emitted = emitted_signs.len();
    let starvation_ratio = if current_emitted == 0 { 1.0 } else { 0.5 };
    std::env::set_var("GA_STARVATION_RATIO", starvation_ratio.to_string());

    if candidate_edges.is_empty() {
        safe_log!("⚠️ EDGE STARVATION → forcing alpha injection");
        // We will add a probe later if needed.
    }"""
text = text.replace(edge_starvation_find, edge_starve_replace)

with open("core/src/ga.rs", "w") as f:
    f.write(text)

