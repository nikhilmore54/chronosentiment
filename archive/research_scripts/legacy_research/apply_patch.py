import re

with open("core/examples/live_engine.rs", "r") as f:
    content = f.read()

# 1. Replace momentum block
old_block = """                let strategy_voters_before = buy_voters + sell_voters;
                let mut momentum_bootstrap_applied = false;
                if env_flag("MOMENTUM_VOTER_BOOTSTRAP") && strategy_voters_before == 0 {
                    let floor = env_parse_f64_pos("MOMENTUM_BOOTSTRAP_FLOOR", 0.000553);
                    let k = env_parse_usize("MOMENTUM_BOOTSTRAP_CONSISTENCY_K", 3).max(1);
                    let mom_abs = momentum_contribution.abs();
                    if mom_abs >= floor {
                        let sign_i = if momentum_contribution > 0.0 {
                            1i8
                        } else if momentum_contribution < 0.0 {
                            -1i8
                        } else {
                            0i8
                        };
                        let hist = mom_sign_hist.entry(symbol.clone()).or_default();
                        hist.push_back(sign_i);
                        while hist.len() > k {
                            hist.pop_front();
                        }
                        let sign_ok =
                            sign_i != 0 && hist.len() >= k && hist.iter().all(|&s| s == sign_i);
                        if sign_ok {
                            momentum_bootstrap_applied = true;
                            if sign_i > 0 {
                                buy_voters = buy_voters.saturating_add(1);
                                buy_strength += 1.0;
                            } else {
                                sell_voters = sell_voters.saturating_add(1);
                                sell_strength += 1.0;
                            }
                        }
                    }
                }"""

new_block = """                // --- MOMENTUM BOOTSTRAP (deterministic) ---
                let mut bootstrap_active = false;
                let mut bootstrap_edge = 0.0;
                let mut bootstrap_direction: i32 = 0; // +1 buy, -1 sell

                let momentum_abs = momentum_contribution.abs();

                let floor: f64 = std::env::var("MOMENTUM_BOOTSTRAP_FLOOR")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0.00045);  // start with p92

                let k_required: usize = std::env::var("MOMENTUM_BOOTSTRAP_CONSISTENCY_K")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(3);

                // maintain rolling sign history (Vec<i8>)
                let sign = if momentum_contribution > 0.0 { 1 } else if momentum_contribution < 0.0 { -1 } else { 0 };

                let hist = mom_sign_hist.entry(symbol.clone()).or_default();
                hist.push_back(sign as i8);
                if hist.len() > k_required {
                    hist.pop_front();
                }

                let consistent = hist.len() == k_required
                    && hist.iter().all(|&s| s == (sign as i8) && s != 0);

                if std::env::var("MOMENTUM_VOTER_BOOTSTRAP").is_ok() && shared_raw_edge == 0.0 && momentum_abs >= floor && consistent {
                    bootstrap_active = true;
                    bootstrap_edge = momentum_abs;
                    bootstrap_direction = sign;
                }"""

content = content.replace(old_block, new_block)

# 2. Voters and gating
old_gating_block1 = """                let min_voters_required = if bootstrap || momentum_bootstrap_applied {
                    1
                } else {
                    2
                };
                let is_high_conf = conf >= min_conf
                    && (buy_voters + sell_voters) >= min_voters_required;
                // Bootstrap floors feasibility at 0.05; strict `>` would reject exactly 0.05 (dead zone).
                let is_capturable = if bootstrap {
                    decision_feasibility >= min_feas
                } else if momentum_bootstrap_applied {
                    decision_feasibility.max(min_feas) > min_feas - 1e-12
                } else {
                    decision_feasibility > min_feas
                };
                let raw_voter_sum = buy_voters + sell_voters;
                let active_strats = voted_count;
                let voters = if reco_single_accept_diag {
                    active_strats.max(raw_voter_sum)
                } else {
                    raw_voter_sum
                };"""

new_gating_block1 = """                let min_voters_required = if bootstrap || bootstrap_active {
                    1
                } else {
                    2
                };
                let is_high_conf = conf >= min_conf
                    && (bootstrap_active || (buy_voters + sell_voters) >= min_voters_required);
                // Bootstrap floors feasibility at 0.05; strict `>` would reject exactly 0.05 (dead zone).
                let is_capturable = if bootstrap {
                    decision_feasibility >= min_feas
                } else if bootstrap_active {
                    decision_feasibility.max(min_feas) > min_feas - 1e-12
                } else {
                    decision_feasibility > min_feas
                };
                let raw_voter_sum = buy_voters + sell_voters;
                let active_strats = voted_count;
                let raw_voters = if reco_single_accept_diag {
                    active_strats.max(raw_voter_sum)
                } else {
                    raw_voter_sum
                };
                
                let effective_voters = if bootstrap_active {
                    1
                } else {
                    raw_voters
                };
                let voters = effective_voters;"""

content = content.replace(old_gating_block1, new_gating_block1)

# 3. Effective edge replacement
old_edge = """                let edge_after_floor = if shared_raw_edge >= edge_gate {
                    shared_raw_edge
                } else {
                    0.0
                };"""

new_edge = """                let effective_edge = if bootstrap_active {
                    bootstrap_edge
                } else {
                    shared_raw_edge
                };

                let edge_after_floor = if effective_edge >= edge_gate {
                    effective_edge
                } else {
                    0.0
                };"""

content = content.replace(old_edge, new_edge)

# 4. Simple replaces
content = content.replace("pass_edge_stability || momentum_bootstrap_applied", "pass_edge_stability || bootstrap_active")
content = content.replace("current_stats.p90 >= edge_gate || momentum_bootstrap_applied", "current_stats.p90 >= edge_gate || bootstrap_active")

# 5. Emit replace
old_emit = """                let best_reco_emit = match best_reco.clone() {
                    Some(x) => Some(x),
                    None if momentum_bootstrap_applied => {
                        let mom_abs = momentum_contribution.abs();
                        let sig = if momentum_contribution > 0.0 {
                            SignalType::BUY
                        } else {
                            SignalType::SELL
                        };
                        Some((
                            synthetic_momentum_trade_reco(symbol.as_str(), price_now, mom_abs, sig),
                            sig,
                            1usize,
                        ))
                    }
                    None => None,
                };

                // Bridge-only lift: strategy edge ∪ |momentum|; no `edge_min` injection here — gate uses `passes_edge_floor`.
                let selected_edge_gate = if momentum_bootstrap_applied {
                    selected_edge.max(momentum_contribution.abs())
                } else {
                    selected_edge
                };
                let passes_edge_floor =
                    momentum_bootstrap_applied || selected_edge_gate >= edge_min;
                let feas_gate = if momentum_bootstrap_applied {
                    decision_feasibility.max(min_feas + 1e-9)
                } else {
                    decision_feasibility
                };
                let reco_voters_req = if momentum_bootstrap_applied {
                    1usize
                } else {
                    reco_min_voters
                };

                let pre_gate_log = if momentum_bootstrap_applied {
                    selected_edge_gate
                } else {
                    shared_raw_edge
                };
                let post_gate_log = if momentum_bootstrap_applied {
                    selected_edge_gate
                } else {
                    edge_after_floor
                };"""

new_emit = """                let best_reco_emit = match best_reco.clone() {
                    Some(x) => Some(x),
                    None if bootstrap_active => {
                        let sig = if bootstrap_direction > 0 {
                            SignalType::BUY
                        } else {
                            SignalType::SELL
                        };
                        Some((
                            synthetic_momentum_trade_reco(symbol.as_str(), price_now, bootstrap_edge, sig),
                            sig,
                            1usize,
                        ))
                    }
                    None => None,
                };

                // Bridge-only lift: strategy edge ∪ |momentum|; no `edge_min` injection here — gate uses `passes_edge_floor`.
                let selected_edge_gate = if bootstrap_active {
                    bootstrap_edge
                } else {
                    selected_edge
                };
                let passes_edge_floor =
                    bootstrap_active || selected_edge_gate >= edge_min;
                let feas_gate = if bootstrap_active {
                    decision_feasibility.max(min_feas + 1e-9)
                } else {
                    decision_feasibility
                };

                let pre_gate_log = if bootstrap_active {
                    selected_edge_gate
                } else {
                    effective_edge
                };
                let post_gate_log = if bootstrap_active {
                    selected_edge_gate
                } else {
                    edge_after_floor
                };"""

content = content.replace(old_emit, new_emit)

# 6. expected_realized_edge 
content = content.replace("let expected_realized_edge = (shared_raw_edge * decision_feasibility).max(0.0);", "let expected_realized_edge = (effective_edge * decision_feasibility).max(0.0);")
content = content.replace("""                        "[EDGE_PIPE] sym={} raw_edge={:.6} capture_prob={:.6} expected_realized_edge={:.6} edge_gate={:.6} edge_min={:.6} mom_abs={:.6}",
                        symbol,
                        shared_raw_edge,""", """                        "[EDGE_PIPE] sym={} raw_edge={:.6} capture_prob={:.6} expected_realized_edge={:.6} edge_gate={:.6} edge_min={:.6} mom_abs={:.6}",
                        symbol,
                        effective_edge,""")

# 7. Momentum bootstrap log
old_log = """                if momentum_bootstrap_applied {
                    println!(
                        "[MOMENTUM_VOTER_BOOTSTRAP] sym={} floor={:.6} k={} mom_abs={:.6} voters_raw={} synthetic_reco={}",
                        symbol,
                        env_parse_f64_pos("MOMENTUM_BOOTSTRAP_FLOOR", 0.000553),
                        env_parse_usize("MOMENTUM_BOOTSTRAP_CONSISTENCY_K", 3).max(1),
                        momentum_contribution.abs(),
                        raw_voter_sum,
                        best_reco.is_none() as i32
                    );
                }"""

new_log = """                if bootstrap_active {
                    println!(
                        "[MOMENTUM_BOOTSTRAP] sym={} edge={:.6} mom_abs={:.6} k={} floor={:.6}",
                        symbol,
                        bootstrap_edge,
                        momentum_abs,
                        k_required,
                        floor
                    );
                }"""

content = content.replace(old_log, new_log)

# 8. EMIT_PROBE gate
old_emit_probe = """                            let pg = al
                                && passes_edge_floor
                                && feas_gate >= feas_min
                                && conf >= conf_min
                                && voters >= reco_voters_req
                                && (rs >= score_min || momentum_bootstrap_applied)"""

new_emit_probe = """                            let pg = al
                                && passes_edge_floor
                                && feas_gate >= feas_min
                                && conf >= conf_min
                                && (bootstrap_active || voters >= reco_min_voters)
                                && (rs >= score_min || bootstrap_active)"""

content = content.replace(old_emit_probe, new_emit_probe)

# 9. Final reco gate
old_reco_gate = """                                    let score_ok =
                                        rec_score >= score_min || momentum_bootstrap_applied;
                                    let passes_reco_gate = passes_edge_floor
                                        && feas_gate >= feas_min
                                        && conf >= conf_min
                                        && voters >= reco_voters_req
                                        && score_ok
                                        && pass_edge_stability_eff
                                        && pass_conf_floor
                                        && pass_reco_structure
                                        && !blocked_symbols.contains(symbol);
                                    if passes_reco_gate {
                                        let cand_primary = if momentum_bootstrap_applied && best_reco.is_none() {
                                            1usize
                                        } else {
                                            primary_id
                                        };
                                        recommendations.push(RecommendationCandidate {
                                            rec_id: next_rec_id,
                                            symbol: symbol.clone(),
                                            score: rec_score,
                                            edge: selected_edge_gate,
                                            conf,
                                            feas: feas_gate,
                                            voters,
                                            primary_id: cand_primary,
                                            signal: sig,
                                            consistency: cons,
                                            recommendation: reco,
                                            from_bootstrap_bridge: momentum_bootstrap_applied
                                                && best_reco.is_none(),
                                        });"""

new_reco_gate = """                                    let score_ok =
                                        rec_score >= score_min || bootstrap_active;
                                    let passes_reco_gate = passes_edge_floor
                                        && feas_gate >= feas_min
                                        && conf >= conf_min
                                        && (bootstrap_active || voters >= reco_min_voters)
                                        && score_ok
                                        && pass_edge_stability_eff
                                        && pass_conf_floor
                                        && pass_reco_structure
                                        && !blocked_symbols.contains(symbol);
                                    if passes_reco_gate {
                                        let cand_primary = if bootstrap_active && best_reco.is_none() {
                                            1usize
                                        } else {
                                            primary_id
                                        };
                                        recommendations.push(RecommendationCandidate {
                                            rec_id: next_rec_id,
                                            symbol: symbol.clone(),
                                            score: rec_score,
                                            edge: selected_edge_gate,
                                            conf,
                                            feas: feas_gate,
                                            voters,
                                            primary_id: cand_primary,
                                            signal: sig,
                                            consistency: cons,
                                            recommendation: reco,
                                            from_bootstrap_bridge: bootstrap_active,
                                        });"""

content = content.replace(old_reco_gate, new_reco_gate)


with open("core/examples/live_engine.rs", "w") as f:
    f.write(content)
print("Patch applied")

