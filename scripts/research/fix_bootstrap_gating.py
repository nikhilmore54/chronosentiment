import re

with open("core/examples/live_engine.rs", "r") as f:
    content = f.read()

# Fix conf and voters calculation for bootstrap
old_conf_calc = """                let total_strength = buy_strength + sell_strength + 0.001;
                let conf = (buy_strength - sell_strength).abs() / total_strength;"""

new_conf_calc = """                let total_strength = buy_strength + sell_strength + 0.001;
                let mut conf = (buy_strength - sell_strength).abs() / total_strength;
                if bootstrap_active {
                    conf = 1.0;
                }"""

content = content.replace(old_conf_calc, new_conf_calc)

# Fix is_high_conf
old_high_conf = """                let is_high_conf = conf >= min_conf
                    && (bootstrap_active || (buy_voters + sell_voters) >= min_voters_required);"""

new_high_conf = """                let is_high_conf = bootstrap_active || (conf >= min_conf
                    && (buy_voters + sell_voters) >= min_voters_required);"""

content = content.replace(old_high_conf, new_high_conf)

# Fix passes_reco_gate to bypass conf check if bootstrap_active
old_reco_gate = """                                    let passes_reco_gate = passes_edge_floor
                                        && feas_gate >= feas_min
                                        && conf >= conf_min
                                        && (bootstrap_active || voters >= reco_min_voters)"""

new_reco_gate = """                                    let passes_reco_gate = passes_edge_floor
                                        && feas_gate >= feas_min
                                        && (bootstrap_active || conf >= conf_min)
                                        && (bootstrap_active || voters >= reco_min_voters)"""

content = content.replace(old_reco_gate, new_reco_gate)

# Fix [DIAG] log to show actual voters instead of raw count
old_diag_log = """                        avg_feasibility,
                        buy_voters + sell_voters,"""

new_diag_log = """                        avg_feasibility,
                        voters,"""

content = content.replace(old_diag_log, new_diag_log)

with open("core/examples/live_engine.rs", "w") as f:
    f.write(content)
print("Fix applied")
