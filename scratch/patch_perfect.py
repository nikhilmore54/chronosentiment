with open("core/src/ga.rs", "r") as f:
    lines = f.readlines()

new_lines = []
for i, line in enumerate(lines):
    # Fix warnings
    if "let maybe_eval =" in line:
        line = line.replace("let maybe_eval =", "let _maybe_eval =")
    
    if "let mut pnl_score =" in line:
        line = line.replace("let mut pnl_score =", "let mut _pnl_score =")
    if "let mut total_avg_e_score =" in line:
        line = line.replace("let mut total_avg_e_score =", "let mut _total_avg_e_score =")
    if "let mut phase2_sum_expected =" in line:
        line = line.replace("let mut phase2_sum_expected =", "let mut _phase2_sum_expected =")

    if "let genome_rng = " in line:
        line = line.replace("let genome_rng = ", "let _genome_rng = ")
    if "let local_scenarios = " in line:
        line = line.replace("let local_scenarios = ", "let _local_scenarios = ")
    if "let mut total_fitness_sum =" in line:
        line = line.replace("let mut total_fitness_sum =", "let mut _total_fitness_sum =")
        
    # Unused imports:
    if "use crate::NormalizedMarketEvent;" in line:
        line = "// " + line
    if "use MarketEvent;" in line:
        line = "// " + line

    if "consensus_conf *= (1.0 - 0.3 * entropy_penalty);" in line:
        line = line.replace("consensus_conf *= (1.0 - 0.3 * entropy_penalty);", "consensus_conf *= 1.0 - 0.3 * entropy_penalty;")

    # Field fixes in ExecutionMetrics
    if "fill_ratio: 0.0" in line and "ScenarioExecutionSignature" not in "".join(lines[max(0, i-5):i]): # It's under ExecutionMetrics
        line = line.replace("fill_ratio: 0.0,", "fill_rate: 0.0, capture_efficiency: 0.0, fill_efficiency: 0.0,")
    if "slippage_avg: 0.0" in line:
        line = line.replace("slippage_avg: 0.0,", "avg_slippage: 0.0,")
    if "latency_avg: 0.0" in line:
        line = line.replace("latency_avg: 0.0,", "latency_impact: 0.0,")
    if "queue_avg: 0.0" in line:
        line = line.replace("queue_avg: 0.0,", "queue_blocked_count: 0,")
    if "impact_avg: 0.0" in line:
        line = line.replace("impact_avg: 0.0,", "liquidity_starved_count: 0,")
    if "total_fills: 0" in line:
        line = line.replace("total_fills: 0,", "")
    if "total_rejects: 0" in line:
        line = line.replace("total_rejects: 0,", "")

    # Field fixes in ScenarioExecutionSignature
    if "slippage_bps: 0.0" in line:
        line = line.replace("slippage_bps: 0.0,", "avg_queue_ahead: 0.0, execution_variance: 0.0,")
    if "latency_ms: 0.0," in line:
        line = line.replace("latency_ms: 0.0,", "avg_latency: 0.0,")
    if "queue_pos: 0.0" in line:
        line = line.replace("queue_pos: 0.0,", "participation: 0.0,")
    if "impact_bps: 0.0" in line:
        line = line.replace("impact_bps: 0.0,", "")

    # Fix axes: [0.0; 4]
    if "axes: [0.0; 4]" in line:
        line = line.replace("axes: [0.0; 4]", "axes: (0,0,0,0)")

    # Fix GaRoundTripOutcome missing is_probe
    if "is_diagnostic: false," in line:
        line = line.replace("is_diagnostic: false,", "is_diagnostic: false, is_probe: false,")

    # StrategyEvaluation missing evaluation_flag:
    if "confidence: 0.0," in line and "evaluation_flag" not in "".join(lines[max(0, i-5):i]):
        line = "            evaluation_flag: None,\n" + line

    new_lines.append(line)

text = "".join(new_lines)





# Now for Strategy execution genes
struct_genes = """    pub direction_bias: u8,          // 0=Short, 50=Dual, 100=Long
    pub vol_floor: u8,               // 0–100 normalized
    pub mom_floor: u8,               // 0–100 normalized
    pub edge_ratio: u8,              // 100–300 → 1.0x–3.0x RR
    pub participation_threshold: u8, // 0–100 conviction gate
    pub exec_aggression: u8,
    pub latency_bias: u8,
    pub fill_threshold: u8,
}"""
text = text.replace("    pub participation_threshold: u8, // 0-100 conviction gate\n}", struct_genes)
text = text.replace("    pub participation_threshold: u8, // 0–100 conviction gate\n}", struct_genes)

# from_seed and random
fs1 = """            edge_ratio: rng.gen_range(120..=250),
            participation_threshold: rng.gen_range(20..=70),
            exec_aggression: 50, latency_bias: 10, fill_threshold: 50,
        }"""
text = text.replace("            edge_ratio: rng.gen_range(120..=250),\n            participation_threshold: rng.gen_range(20..=70),\n        }", fs1, 1)

fs2 = """            edge_ratio: rng.gen_range(120..=250),
            participation_threshold: rng.gen_range(20..=70),
            exec_aggression: rng.gen_range(20..80), latency_bias: rng.gen_range(0..100), fill_threshold: rng.gen_range(20..80),
        }"""
text = text.replace("            edge_ratio: rng.gen_range(120..=250),\n            participation_threshold: rng.gen_range(20..=70),\n        }", fs2, 1)

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

        if rng.gen_bool(0.3) {
            return new.mutate(rng, scale * 2.5);
        }
        new
    }
}
"""
text = text.replace("}\n\n/// Scenario execution bounds", mutate_str + "\n/// Scenario execution bounds", 1)


import re

# Insert missing fields for Strategy { ... } where they were empty
text = re.sub(
    r'(Strategy \{[^}]*particip[^\n]*)(\n\s*\})',
    r'\1\n            exec_aggression: 50, latency_bias: 10, fill_threshold: 50,\2',
    text
)

# Fix evaluate_market_conviction
# The root cause debug asked for Gate Degradation:
old_convic = """    // Phase D.1.21: Hard Reject (Adaptive)
    if n_vol < (vol_floor * gate_looseness) * 0.8 || n_mom < (mom_floor * gate_looseness) * 0.8 {
        return ConvictionOutcome {"""

new_convic = """    // PHASE V3.3: HARD REJECT WITH GRADUAL GATE DEGRADATION
    let starvation_ratio = std::env::var("GA_STARVATION_RATIO").unwrap_or_else(|_| "0.0".to_string()).parse::<f64>().unwrap_or(0.0);
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

with open("core/src/ga.rs", "w") as f:
    f.write(text)

