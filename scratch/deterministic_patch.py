with open("core/src/ga.rs.bak2", "r") as f:
    text = f.read()

# 1. Allow block
text = "#![allow(unused_variables, unused_mut, unused_imports, dead_code, unreachable_code, unused_assignments)]\n" + text

# 2. Strategy Struct Execution Genes
old_genes = """    pub direction_bias: u8,          // 0=Short, 50=Dual, 100=Long
    pub vol_floor: u8,               // 0–100 normalized
    pub mom_floor: u8,               // 0–100 normalized
    pub edge_ratio: u8,              // 100–300 → 1.0x–3.0x RR
    pub participation_threshold: u8, // 0–100 conviction gate
}"""
new_genes = """    pub direction_bias: u8,          // 0=Short, 50=Dual, 100=Long
    pub vol_floor: u8,               // 0–100 normalized
    pub mom_floor: u8,               // 0–100 normalized
    pub edge_ratio: u8,              // 100–300 → 1.0x–3.0x RR
    pub participation_threshold: u8, // 0–100 conviction gate
    pub exec_aggression: u8,
    pub latency_bias: u8,
    pub fill_threshold: u8,
}"""
text = text.replace(old_genes, new_genes, 1)

# 3. Strategy Init in from_seed
old_from_seed = """            edge_ratio: rng.gen_range(120..=250),
            participation_threshold: rng.gen_range(20..=70),
        }
    }"""
new_from_seed = """            edge_ratio: rng.gen_range(120..=250),
            participation_threshold: rng.gen_range(20..=70),
            exec_aggression: 50,
            latency_bias: 10,
            fill_threshold: 50,
        }
    }"""
text = text.replace(old_from_seed, new_from_seed, 1)

# 4. Strategy Init in random
old_random = """            edge_ratio: rng.gen_range(120..=250),
            participation_threshold: rng.gen_range(20..=70),
        }
    }

    /// Buckets genes"""
new_random = """            edge_ratio: rng.gen_range(120..=250),
            participation_threshold: rng.gen_range(20..=70),
            exec_aggression: rng.gen_range(20..80),
            latency_bias: rng.gen_range(0..100),
            fill_threshold: rng.gen_range(20..80),
        }
    }

    /// Buckets genes"""
text = text.replace(old_random, new_random, 1)

# 5. Mutate Logic
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


# 6. Gradual Gate Degradation
old_convic = """    // Phase D.1.21: Hard Reject (Adaptive)
    if n_vol < (vol_floor * gate_looseness) * 0.8 || n_mom < (mom_floor * gate_looseness) * 0.8 {
        return ConvictionOutcome {"""
new_convic = """    // PHASE V3.3: HARD REJECT WITH GRADUAL GATE DEGRADATION
    let starvation_str = std::env::var("GA_STARVATION_RATIO").unwrap_or_else(|_| "0.0".to_string());
    let starvation_ratio = starvation_str.parse::<f64>().unwrap_or(0.0);
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
text = text.replace(old_convic, new_convic, 1)

# 7. Scale Fix Harness
old_harness = """            let price = if i < 48 { flat_price } else { step_price };"""
new_harness = """            let price = if i < 48 { flat_price * 100 } else { step_price * 100 };"""
text = text.replace(old_harness, new_harness, 1)

# 8. Print Metrics in Harness
old_print = """        let total_exec = candidate_trades.len();"""
new_print = """        let mut real_signals = 0;
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
text = text.replace(old_print, new_print, 1)

# 9. Edge Starvation ENV Set
old_starvation = """    if candidate_edges.is_empty() {
        safe_log!("⚠️ EDGE STARVATION → forcing alpha injection");"""
new_starvation = """    let current_emitted = emitted_signs.len();
    let starvation_ratio = if current_emitted == 0 { 1.0 } else { 0.5 };
    std::env::set_var("GA_STARVATION_RATIO", starvation_ratio.to_string());

    if candidate_edges.is_empty() {
        safe_log!("⚠️ EDGE STARVATION → forcing alpha injection");"""
text = text.replace(old_starvation, new_starvation, 1)


# 10. Fix strategy {} empty initializations leftover
old_strat1 = """        let random_strat = Strategy {
            queue_threshold: rng.gen_range(60..120),
            base_edge: rng.gen_range(5..15),
            take_profit: rng.gen_range(5..20),
            stop_loss: rng.gen_range(5..15),
            holding_period: rng.gen_range(20..60),
            w_conviction: rng.gen_range(20..80),
            w_momentum: rng.gen_range(20..80),
            w_volatility: rng.gen_range(20..80),
            exp_conviction: rng.gen_range(80..150),
            exp_momentum: rng.gen_range(80..150),
            exp_volatility: rng.gen_range(80..150),
            selectivity: rng.gen_range(60..90),
            archetype: rng.gen_range(0..=3),
            entry_offset: rng.gen_range(-5..5),
            direction_bias: [0, 50, 100][rng.gen_range(0..3)],
            vol_floor: rng.gen_range(10..50),
            mom_floor: rng.gen_range(10..50),
            edge_ratio: rng.gen_range(120..200),
            participation_threshold: rng.gen_range(20..60),
        };"""
new_strat1 = old_strat1.replace("        };", "            exec_aggression: 50, latency_bias: 10, fill_threshold: 50,\n        };")
text = text.replace(old_strat1, new_strat1)

old_strat2 = """    Strategy {
        queue_threshold: 120,
        base_edge: 4,
        take_profit: 3,
        stop_loss: 4,
        holding_period: 32,
        w_conviction: 53,
        w_momentum: 41,
        w_volatility: 12,
        exp_conviction: 181,
        exp_momentum: 120,
        exp_volatility: 171,
        selectivity: 57,
        archetype: 3,
        entry_offset: -4,
        direction_bias: 50,
        vol_floor: 37,
        mom_floor: 18,
        edge_ratio: 247,
        participation_threshold: 51,
    }"""
new_strat2 = old_strat2.replace("    }", "        exec_aggression: 50, latency_bias: 10, fill_threshold: 50,\n    }")
text = text.replace(old_strat2, new_strat2)


with open("core/src/ga.rs", "w") as f:
    f.write(text)

# Fix edge_decay.rs
with open("core/src/edge_decay.rs", "r") as f:
    edge_text = f.read()
if "exec_aggression:" not in edge_text:
    edge_text = edge_text.replace("participation_threshold: 60,\n        })", "participation_threshold: 60,\n            exec_aggression: 50, latency_bias: 10, fill_threshold: 50,\n        })")
with open("core/src/edge_decay.rs", "w") as f:
    f.write(edge_text)

