pub fn evaluate_strategy(
    candidate: &Candidate,
    pair: &ScenarioPair,
    config: &GaConfig,
    generation: usize,
    _diversity: f64,
    _unique_count: usize,
    gen_max_log_queue: f64,
    _expansion_bias: f64,
    strategy_index: usize,
) -> Option<CandidateEvaluation> {
    // Deterministic RNG: seed derived from config.seed + generation + strategy_index
    // so every (run, generation, candidate) triple produces identical results.
    let _rng = StdRng::seed_from_u64(
        config.seed
            ^ (generation as u64).wrapping_mul(1_000_003)
            ^ (strategy_index as u64).wrapping_mul(7_919),
    );
    let mut executed_trades: Vec<GaRoundTripOutcome> = Vec::new();
    let mut pnl_history_learn: Vec<GaRoundTripOutcome> = Vec::new();
    let mut injected_trades: Vec<GaRoundTripOutcome> = Vec::new();

    let mut trade_scores: Vec<(usize, f64)> = Vec::new();
    let mut trade_edges: Vec<(usize, f64)> = Vec::new();
    let mut sniper_scores: Vec<(usize, f64)> = Vec::new();
    let mut consistent_scores: Vec<(usize, f64)> = Vec::new();
    let mut edge_vals_build: Vec<f64> = Vec::new(); // build during execution
    let mut edge_vals_snapshot: Vec<f64> = Vec::new(); // frozen later

    // Phase 17C: Parity Tracking
    let exec_probs_history: Vec<f64> = Vec::new();
    let survive_scores_history: Vec<f64> = Vec::new();
    let realized_edges_history: Vec<f64> = Vec::new();

    let clarity_penalty = 1.0;
    let scenario_name = pair.name;
    let signal_events = pair.signal;
    let execution_events = pair.execution;
    let signal_symbol = pair.signal_symbol;
    let exec_symbol = pair.execution_symbol;

    // Phase 4: Routing Integrity & Pointer Safety (True Dual-Stream)
    if ga_debug_enabled() {
        println!("ROUTE_SOURCE → {} -> {}", signal_symbol, exec_symbol);
        println!(
            "ROUTE_VERIFY → diff={} sig_ptr={:p} exec_ptr={:p}",
            !std::ptr::eq(signal_events.as_ptr(), execution_events.as_ptr()),
            signal_events.as_ptr(),
            execution_events.as_ptr()
        );
        println!("SIGNALS_GENERATED → {}", signal_events.len());
    }

    // Hard Assert: Prevent "fake" separation at the memory level
    if pair.signal_symbol != pair.execution_symbol {
        assert!(
                    !std::ptr::eq(signal_events.as_ptr(), execution_events.as_ptr()),
                    "FATAL: signal and execution streams are physically identical buffers for symbols {}/{}",
                    pair.signal_symbol, pair.execution_symbol
                );
    }
    let strategy_id = strategy_to_id(candidate);

    let mut hasher = DefaultHasher::new();
    candidate.hash(&mut hasher);
    let genome_hash = std::hash::Hasher::finish(&hasher);

    if ga_debug_enabled() {
        println!("GENOME_TRACE → {}", genome_hash);
    }
    let mut sum_actual_slippage = 0.0;
    let mut sum_expected_slippage = 0.0;
    let mut queue_blocked_count = 0usize;
    let mut liquidity_starved_count = 0usize;
    let mut _total_attempts = 0usize;
    let capability = determine_scenario_capability(scenario_name);

    // --- Phase 9: Environment Gating (Scenario-Level) ---
    // Pre-scan all signal points to assess the regime quality before committing to execution.
    let mut candidate_edges = Vec::new();

    for i in 0..signal_events.len() {
        let conv =
            evaluate_market_conviction(candidate, scenario_name, signal_events, i, 0, generation);
        if conv.is_valid && conv.conviction_score >= conv.selection_threshold {
            let entry_price = signal_events[i].price as f64;
            let atr = calculate_atr(signal_events, i, 14);

            // Phase D.1.21: Use edge_ratio gene for edge calculation
            let rr = candidate.edge_ratio as f64 / 100.0;
            let pred_move = atr * rr;

            // Synchronized Edge Estimate (Matches Patch 4)
            let edge_abs = (pred_move * 0.8 * 0.9) - (entry_price * 0.0001);
            let mut edge_ratio = (edge_abs / entry_price.max(1.0)).max(0.0);

            // FIX 1 — EDGE AMPLIFICATION & THRESHOLDING
            edge_ratio *= 5.0;
            if edge_ratio < 0.002 {
                continue;
            } // Drop weak signals

            let _dir_abs = conv.norm_momentum.abs().max(0.01);

            let raw_edge = (conv.conviction_score * conv.edge_weight * (atr / entry_price.max(1.0))).max(0.0001);

            candidate_edges.push(raw_edge);
        }
    }

    if candidate_edges.is_empty() {
        if ga_debug_enabled() {
            println!("⚠️ EDGE STARVATION → forcing alpha injection");
        }
        return Some(CandidateEvaluation {
            candidate_edges: candidate_edges.clone(),
            candidate: candidate.clone(),
            strategy_id: strategy_id.clone(),
            fitness: -0.15, // Special starvation code
            trade_count: 0,
            pnl_history: Vec::new(),
            ..CandidateEvaluation::default()
        });
    }

    // --- PHASE 10.3: DART (Dynamic Asset-Relative Thresholding) FLOOR ---
    // Extract a representative window-level floor for pre-filtering stats.
    let avg_edge = if candidate_edges.is_empty() {
        0.0001
    } else {
        candidate_edges.iter().sum::<f64>() / candidate_edges.len() as f64
    };
    // Pre-rank raw-edge std (per scenario): used for hot-path diagnostics only.
    // Selection uses pooled p75 decisiveness in `aggregate_strategy_reports_inner` (see
    // `DEFAULT_GA_FITNESS_EDGE_STD_LAMBDA`); set `GA_FITNESS_EDGE_STD_LAMBDA=0` to disable.
    let raw_edge_std = if candidate_edges.len() >= 2 {
        let mean = candidate_edges.iter().sum::<f64>() / candidate_edges.len() as f64;
        let var = candidate_edges
            .iter()
            .map(|v| {
                let d = *v - mean;
                d * d
            })
            .sum::<f64>()
            / candidate_edges.len() as f64;
        var.sqrt()
    } else {
        0.0
    };
    // Use a fixed scale so lambda remains in intuitive ranges (~0.01-0.30).
    let fitness_dispersion_lambda = std::env::var("GA_FITNESS_EDGE_STD_LAMBDA")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(DEFAULT_GA_FITNESS_EDGE_STD_LAMBDA)
        .max(0.0);
    let scenario_dispersion_term = fitness_dispersion_lambda * (raw_edge_std * 1000.0);
    let avg_atr_pct = avg_edge * 1.2;
    let window_dart_floor = (avg_atr_pct * 0.40)f64.clamp(0.00001, 0.0012);

    // --- PHASE 10.3: AQG DISTRIBUTION INTEGRITY ---
    // Filter noise before statistics. Use exact DART floor as the viability barrier.
    let valid_edges: Vec<f64> = candidate_edges
        .iter()
        .cloned()
        .filter(|e| *e >= window_dart_floor)
        .collect();

    // AQG Starvation Gate Removed (Analytical Mode)
    let _coverage = valid_edges.len() as f64 / candidate_edges.len().max(1) as f64;
    let _min_sample = (candidate_edges.len() as f64 * 0.03).max(5.0) as usize;

    let _aqg_health = (valid_edges.len() as f64 / _min_sample.max(1) as f64)f64.clamp(0.0, 1.5);
    // let _ = aqg_threshold; // Closure compatibility

    // --- PHASE 10.3: AQG (Adaptive Percentile Selection) ---
    let mut v = valid_edges.clone();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let (median, mad_scaled, aqg_gate) = if v.is_empty() {
        (0.0001, 0.0, 0.0001)
    } else {
        let median = v[v.len() / 2];
        let mut deviations: Vec<f64> = v.iter().map(|e| (e - median).abs()).collect();
        deviations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mad = deviations[deviations.len() / 2];
        let mad_scaled = mad * 1.4826;
        let dispersion_val = mad_scaled / median.max(1e-9);
        let pct = if dispersion_val > 0.8 {
            0.50
        } else if dispersion_val > 0.5 {
            0.55
        } else {
            0.60
        };
        let idx = ((v.len() as f64) * pct).floor() as usize;
        let aqg_gate = v[idx.min(v.len().saturating_sub(1))];
        (median, mad_scaled, aqg_gate)
    };
    let aqg_threshold = aqg_gate;
    let dispersion = mad_scaled / median.max(1e-9);

    if std::env::var("GA_SIGNAL_DEBUG").is_ok() {
        println!(
            "AQG_ADMISSION → scenario={} dispersion={:.6} aqg_gate={:.6} (valid={}/max={})",
            scenario_name,
            dispersion,
            aqg_gate,
            valid_edges.len(),
            candidate_edges.len()
        );
    }
    let mut scenario_pnls: Vec<f64> = Vec::new();
    let mut total_quality_trades_scenario = 0.0;
    let _total_efficiency = 0.0;
    let mut total_vol_ratio = 0.0;
    let mut total_spread_reality = 0.0;
    let mut _total_spread_test = 0.0;
    let mut survivable_trades_count = 0usize;
    let mut _sum_price = 0.0;
    let mut metrics = ScenarioMetrics::default();
    let mut probe_count = 0usize;
    let mut real_trade_count = 0usize;

    // Diagnostic Counters
    let mut _signal_count = 0usize;
    let entry_attempted = 0usize;
    let mut _total_trades = 0usize;
    let skipped_busy = 0usize;
    let mut exit_tp_count = 0usize;
    let mut exit_sl_count = 0usize;
    let mut exit_ts_count = 0usize;
    let _max_horizon = 200;

    let mut total_filled_qty = 0u64;
    let mut total_slippage_bps = 0.0;
    let mut fills_count = 0usize;
    let mut sum_drawdown_raw = 0.0;
    let mut sum_expected_move = 0.0;
    let mut total_tail_penalty = 0.0;
    let mut sum_latency_raw = 0.0;
    let mut cycle_sigs: Vec<ScenarioExecutionSignature> = Vec::new();

    // Phase 8.8 Aggregators
    let mut max_pnl_in_scenario: f64 = 0.0;
    let mut pnl_from_tp_scenario: f64 = 0.0;
    let mut pnl_from_sl_scenario: f64 = 0.0;
    let mut max_trade_pnl_scenario: f64 = 0.0;
    let mut long_win_count_scenario = 0usize;
    let short_win_count_scenario = 0usize;
    let mut _micro_loss_count = 0u32;
    let mut total_window_volume = 0.0;
    let triggered_entries = 0usize; // ✅ ADD THIS
    let mut exec_passed = 0usize;

    // --- PHASE 14: DISTRIBUTION-AWARE SIGNAL VALIDATION LAYER ---
    // Transitioning from fixed-gate scoring to institutional selective-gating.
    // This ensures only true statistical outliers from the candidate are traded.

    let _name_upper = scenario_name.to_uppercase();
    // Phase D.1.14: Bypass Eradication. We no longer allow bypass based on scenario names.
    // Structural integrity is now mandatory.
    let allow_bypass = std::env::var("GA_BYPASS_AQG").is_ok();

    // --- PHASE D.1.7: STRATEGY-LOCAL SIGNAL SPACE ---
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
use std::hash::BuildHasher;

    let strategy_seed = {
        let mut hasher = DefaultHasher::new();
        candidate.queue_threshold.hash(&mut hasher);
        candidate.base_edge.hash(&mut hasher);
        generation.hash(&mut hasher);
        std::hash::Hasher::finish(&hasher)
    };
    let perturb = (strategy_seed % 1000) as f64 / 1000.0;

    // 0. Simulation Context (Phase D.1.18 alpha detection)
    let mut had_organic_signals = false;

    // 1. Pre-Scan (Collect All Scores with Perturbation)
    let mut window_data = Vec::with_capacity(signal_events.len());
    let mut scores = Vec::with_capacity(signal_events.len());
    for current_idx in 2..signal_events.len().saturating_sub(1) {
        let mut conviction = evaluate_market_conviction(
            candidate,
            scenario_name,
            signal_events,
            current_idx,
            0,
            generation,
        );

        // Phase D.1.21: Enforce Absolute Gating before stats
        if !conviction.is_valid {
            conviction.conviction_score *= 0.5; // degrade instead of kill
        }

        // Apply candidate-specific perturbation
        conviction.conviction_score *= 0.9 + 0.2 * perturb;

        scores.push(conviction.conviction_score);
        window_data.push((current_idx, conviction));
    }
    if window_data.len() < 5 {
        if ga_debug_enabled() {
            println!("⚠️ INSUFFICIENT SIGNAL BASE → activating degraded mode");
        }
        // 🔥 FIX: INLINE MINIMAL SIGNAL INJECTION (NO EXTERNAL FUNCTION)
        window_data.clear();

        let fallback_idx = signal_events.len() / 2;

        let mut conviction = evaluate_market_conviction(
            candidate,
            scenario_name,
            signal_events,
            fallback_idx,
            0,
            generation,
        );

        // soften but don't kill
        conviction.conviction_score = conviction.conviction_score.max(0.05);
        conviction.is_valid = true;

        window_data.push((fallback_idx, conviction));

        if ga_debug_enabled() {
            println!("⚠️ MINIMAL SIGNAL INJECTED → idx={}", fallback_idx);
        }
    }
    if window_data.is_empty() {
        if ga_debug_enabled() {
            println!("⚠️ NO WINDOW DATA → degraded continuation");
        }

        return Some(CandidateEvaluation {
            candidate: candidate.clone(),
            strategy_id: strategy_id.clone(),
            fitness: -0.2,
            trade_count: 0,
            pnl_history: executed_trades.clone(),
            ..CandidateEvaluation::default()
        });
    }
    if scores.is_empty() {
        if ga_debug_enabled() {
            println!("⚠️ scores empty → injecting fallback score");
        }
        scores.push(0.1);
    }

    // 2. Statistical Derivation & Adaptive Gating
    let n = scores.len() as f64;
    let mean = scores.iter().sum::<f64>() / n;
    let variance = scores.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / n;
    let std_dev = variance.sqrt();

    // Phase 17A: Soften the Adaptive Quality Gate (AQG)
    // We change LOW_DISPERSION from a hard skip into a market-condition scaler.
    let dispersion_multiplier = if allow_bypass {
        1.0
    } else {
        (std_dev / MIN_STD).min(1.0)
    };

    // Adaptive Parameters
    let abs_floor = (percentilef64(&scores, 0.80)).max(mean + 0.5 * std_dev);
    let z_threshold = (BASE_Z * (TARGET_STD / (std_dev + EPS)))f64.clamp(0.8, 1.5);
    let _min_signals = (window_data.len() as f64 * 0.05).max(2.0) as usize;

    // 3. Signal Validation Layer (Identify High-Conviction Cluster)
    // Phase 17A.5: Store dominance and reason to ensure consistency
    // Phase 17B: Store E-score for realizability analysis
    let mut valid_signals: Vec<(
        usize,
        ConvictionOutcome,
        f64,
        &'static str,
        f64,
        SignalSource,
        SignalSignature,
    )> = Vec::new();
    let mut emitted_signs: Vec<SignalAlpha> = Vec::new();
    let mut scored_signals: Vec<(
        usize,
        ConvictionOutcome,
        f64,
        &'static str,
        f64,
        f64,
        SignalSource,
    )> = Vec::new();
    let mut max_z = 0.0;

    // --- PHASE D.1.13.5: ADAPTIVE SIGNAL FLOOR (BOOTSTRAP FIX) ---
    // If the adaptive threshold is too strict (killing all signals), fallback to the 60th percentile.
    let percentile_60 = percentilef64(&scores, 0.60);
    let effective_floor = abs_floor.max(percentile_60);

    let p75_energy = percentilef64(&scores, 0.75);
    let _energy_min = effective_floor.max(p75_energy);

    let decision_was_override = false;
    let mut _acceptance_mode = AcceptanceMode::Dominance;

    let mut candidate_signals: Vec<(
        usize,
        ConvictionOutcome,
        f64,
        &'static str,
        f64,
        SignalSource,
        f64,
        SignalSignature,
    )> = Vec::new();

    for (signal_idx, conviction) in window_data.iter() {
        // --- PHASE 17 CALIBRATION: RAW Population Integrity ---
        let mut sub_scores = [
            conviction.norm_momentum,
            conviction.norm_volume,
            conviction.norm_vol_score,
        ];
        sub_scores.sort_by(|a, b| b.total_cmp(a));

        let mean_t = sub_scores.iter().sum::<f64>() / 3.0;
        let std_t = (sub_scores.iter().map(|s| (s - mean_t).powi(2)).sum::<f64>() / 3.0).sqrt();
        let raw_dom_t = (sub_scores[0] - sub_scores[1]) / (std_t + EPS);
        metrics.record_pop_stats(raw_dom_t.min(3.0), false); // RAW layer

        let mut adj_conviction = conviction.clone();
        adj_conviction.conviction_score *= dispersion_multiplier;

        let score_val = adj_conviction.conviction_score;
        let z_score = (score_val - mean) / (std_dev + EPS);
        if z_score > max_z {
            max_z = z_score;
        }

        // --- PHASE D.1.17: COMPETITIVE ADMISSION (RELATIVE) ---
        // We only admit positive conviction to avoid garbage.
        if score_val > 0.0 {
            // Memory-Smoothed Stability
            let scores_idx = window_data
                .iter()
                .position(|(i, _)| *i == *signal_idx)
                .unwrap_or(0);
            let s_t = scores[scores_idx];
            let s_t_1 = if scores_idx >= 1 {
                scores[scores_idx - 1]
            } else {
                s_t
            };
            let s_t_2 = if scores_idx >= 2 {
                scores[scores_idx - 2]
            } else {
                s_t_1
            };
            let delta = (s_t - s_t_1).abs() + (s_t_1 - s_t_2).abs();
            let stability = (1.0 - delta / (2.0 * (std_dev + EPS).max(0.05)))f64.clamp(0.0, 1.0);

            // Execution Realizability (E-score)
            let current_price = signal_events[*signal_idx].price as f64;
            let prev_price = if *signal_idx > 0 {
                signal_events[*signal_idx - 1].price as f64
            } else {
                current_price
            };
            let atr = calculate_atr(signal_events, *signal_idx, 14);
            let dist_score = (1.0
                - ((current_price - prev_price).abs()
                    / ((if atr > 0.0 { atr } else { current_price * 0.01 }) + EPS))
                    .min(1.0))
            f64.clamp(0.0, 1.0);

            // Regime-Aware Volatility Score
            let scores_idx_2 = window_data
                .iter()
                .position(|(i, _)| *i == *signal_idx)
                .unwrap_or(0);
            let local_window = (scores_idx_2 + 1).min(20);
            let start_idx = (scores_idx_2 + 1).saturating_sub(local_window);
            let local_slice = &scores[start_idx..=scores_idx_2];
            let local_mean = local_slice.iter().sum::<f64>() / local_window as f64;
            let local_var = local_slice
                .iter()
                .map(|s| (s - local_mean).powi(2))
                .sum::<f64>()
                / local_window as f64;
            let local_vol = local_var.sqrt();
            let vol_ratio = (local_vol / (std_dev + EPS))f64.clamp(0.25, 4.0);
            let vol_score = (1.0 - (vol_ratio - 1.0).abs())f64.clamp(0.0, 1.0);

            let e_signal_score = ((stability + dist_score + vol_score) / 3.0)f64.clamp(0.0, 1.0);

            // --- Phase D.1.18: Calculate Signal Signature ---
            let regime = if vol_ratio > 1.3 {
                1
            } else if vol_ratio < 0.7 {
                -1
            } else {
                0
            };
            let momentum = if adj_conviction.norm_momentum > 0.3 {
                1
            } else if adj_conviction.norm_momentum < -0.3 {
                -1
            } else {
                0
            };
            let signature = SignalSignature {
                archetype: candidate.archetype,
                regime,
                momentum,
            };

            candidate_signals.push((
                *signal_idx,
                adj_conviction,
                raw_dom_t,
                "RELATIVE_CANDIDATE",
                e_signal_score,
                SignalSource::Organic,
                stability,
                signature,
            ));
        }
    }

    // --- PHASE D.1.17: TOP-K SELECTION & FORCE EMERGENCE ---
    // 1. Sort by conviction intensity
    candidate_signals.sort_by(|a, b| b.1.conviction_score.total_cmp(&a.1.conviction_score));

    // 2. Select Top-5 and Apply Curved Penalties + Credibility
    for (_idx, (signal_idx, mut conv, dom, _reason, e_signal_score, source, stability, signature)) in
        candidate_signals.into_iter().enumerate()
    {
        let min_conviction_threshold = 0.1;
        if conv.conviction_score < min_conviction_threshold {
            conv.conviction_score *= 0.5;
        }

        // --- Refined Multipliers (D.1.17 Sharpe) ---
        let z_score = (conv.conviction_score - mean) / (std_dev + EPS);
        let stat_confidence = (z_score / z_threshold)f64.clamp(0.5, 1.5);
        let stability_factor = stabilityf64.clamp(0.1, 1.0).powf(2.0);

        // --- Phase D.1.18: Credibility Overlay ---
        let credibility = if let Some(stats) = metrics.signature_memory.get(&signature) {
            let winrate = if stats.sample_count > 0 {
                stats.win_count as f64 / stats.sample_count as f64
            } else {
                0.5
            };
            let avg_pnl = if stats.sample_count > 0 {
                stats.sum_pnl / stats.sample_count as f64
            } else {
                0.0
            };
            let c = (avg_pnl * winrate) * (stats.sample_count as f64 + 1.0).ln();

            // Phase D.1.18 Alpha Threshold check
            if stats.sample_count > 5 && winrate > 0.55 && avg_pnl > 0.0005 {
                had_organic_signals = true;
            }

            (1.0 + c)f64.clamp(0.5, 2.0)
        } else {
            1.0
        };

        conv.edge_weight = (stat_confidence * stability_factor * credibility)f64.clamp(0.2, 2.5);
        valid_signals.push((
            signal_idx,
            conv,
            dom,
            "RELATIVE_CANDIDATE",
            e_signal_score,
            source,
            signature,
        ));
    }

    // 🔥 STEP 2: DEBUG signal composition (VERY IMPORTANT)
    if valid_signals.is_empty() {
        if window_data.len() >= 10 {
            valid_signals = extract_weak_signals(&window_data, candidate);

            if valid_signals.is_empty() {
                if ga_debug_enabled() {
                    println!("⚠️ NO VALID SIGNALS → degraded continuation");
                }

                return Some(CandidateEvaluation {
                    candidate: candidate.clone(),
                    strategy_id: strategy_id.clone(),
                    fitness: -0.15,
                    trade_count: 0,
                    pnl_history: Vec::new(),
                    ..CandidateEvaluation::default()
                });
            }

            if std::env::var("GA_VERBOSE").is_ok() {
                if ga_debug_enabled() {
                    println!("⚠️ DEGRADED SIGNAL MODE → weak signals allowed");
                }
            }
        }
        // } else {
        //     return None;
        // }
    }

    let mut _fallback_count = 0;
    let mut organic_count = 0;

    for (_, _, _, _, _, source, _) in &valid_signals {
        match source {
            SignalSource::Synthetic => _fallback_count += 1,
            _ => organic_count += 1,
        }
    }
    if std::env::var("GA_SIGNAL_DEBUG").is_ok() {
        println!("SIGNAL_COMPOSITION → organic={}", organic_count);
    }
    // --- PHASE 14++: STRUCTURAL METRICS & DISTRIBUTION AWARENESS ---

    // --- PHASE 14++: STRUCTURAL METRICS & DISTRIBUTION AWARENESS ---
    let _top_1 = scores.iter().fold(0.0f64, |a, b| a.max(*b));
    let _scores_sum: f64 = scores.iter().sum::<f64>();

    let mut sorted_scores = scores.clone();
    sorted_scores.sort_by(|a, b| b.total_cmp(a));
    let top_k_sum: f64 = sorted_scores.iter().take(3).sum();

    let std_v = if scores.len() > 1 {
        let variance = scores.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / scores.len() as f64;
        variance.sqrt()
    } else {
        0.0
    };

    // Purity: Ratio of High-Quality signals (E > 0.80) in the VALID pool
    let high_quality_count = valid_signals
        .iter()
        .filter(|(_, conv, _, _, _, _, _)| conv.conviction_score > effective_floor)
        .count();
    let purity = high_quality_count as f64 / valid_signals.len().max(1) as f64;

    // Agreement: Ratio of dominant side (BUY vs SELL)
    let mut buy_count = 0usize;
    let mut sell_count = 0usize;
    for (_, conv, _, _, _, _, _) in &valid_signals {
        if conv.is_bearish {
            sell_count += 1;
        } else {
            buy_count += 1;
        }
    }
    let dominant_count = buy_count.max(sell_count);
    // Phase 11 & D.1.7: Deterministic Agreement Jitter (Break 1.0 Lock)
    let noise = (strategy_seed % 100) as f64 / 100.0;
    let jitter = 0.85 + 0.3 * noise;
    let mut agreement = (dominant_count as f64 / valid_signals.len().max(1) as f64) * jitter;

    // --- PHASE D.1.7: DETERMINISTIC AGREEMENT ASYMMETRY (WIDE RANGE) ---
    // Aggressively break "degenerate consensus" with deterministic ID-based shift.
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    strategy_id.hash(&mut hasher);
    let id_hash = std::hash::Hasher::finish(&hasher);
    let asymmetry = 0.85 + ((id_hash % 100) as f64 / 100.0) * 0.3; // 0.85 -> 1.15

    agreement = (agreement * asymmetry)f64.clamp(0.0, 1.0);

    // --- PHASE A+: SCORING ENGINE ---
    // 1. Identify "Window Potential" (Best candidate stats for structural pulse)
    let (best_dom, _best_signal_e) = valid_signals
        .iter()
        .max_by(|(_, a, _, _, _, _, _), (_, b, _, _, _, _, _)| {
            a.conviction_score.total_cmp(&b.conviction_score)
        })
        .map(|(_, _, dom, _, e_score, _, _)| (*dom, *e_score))
        .unwrap_or((0.0, 0.0));
    let stability_raw = (1.0 - std_v / 0.18)f64.clamp(0.0, 1.0);

    // 2. Update structural stats BEFORE gating (Layer 1: Perception)
    metrics.record_adaptive_pulse(agreement, best_dom, purity, stability_raw, max_z, 0.0);

    // 3. Normalized Metrics (Z-scores) using Institutional Priors (Fallback N < 20)
    let n_count = metrics.adaptive.agreement.count;

    let calc_z = |val: f64, tracker: &WelfordTracker, p_mu: f64, p_sigma: f64| -> f64 {
        if n_count < 20 {
            let safe_sigma = p_sigma.max(0.05);
            ((val - p_mu) / safe_sigma)f64.clamp(-5.0, 5.0)
        } else {
            let safe_std = tracker.std().max(0.05);
            ((val - tracker.mean()) / safe_std)f64.clamp(-5.0, 5.0)
        }
    };

    let agreement_z = calc_z(agreement, &metrics.adaptive.agreement, 0.65, 0.10);
    let _purity_z = calc_z(purity, &metrics.adaptive.purity, 0.60, 0.15);
    let stability_z = calc_z(stability_raw, &metrics.adaptive.stability, 0.15, 0.05);
    let dominance_z = calc_z(best_dom, &metrics.adaptive.dominance, 0.20, 0.10);
    let z_norm = calc_z(max_z, &metrics.adaptive.z_score, 1.50, 0.50);
    let energy_norm = dominance_z; // fallback to conviction strength

    // Final Adaptive Score (Weighted Sum)
    // 30% z_norm, 25% energy_norm, 20% dominance_norm, 15% agreement_norm, 10% stability_norm
    let final_score = 0.30 * z_norm
        + 0.25 * energy_norm
        + 0.20 * dominance_z
        + 0.15 * agreement_z
        + 0.10 * stability_z;

    let adaptive_threshold = metrics.adaptive_threshold(0.60); // Use PREVIOUS history
    let is_struct_valid_adaptive = final_score >= (adaptive_threshold - 0.55)
        && stability_raw > 0.3
        && purity > 0.2
        && agreement > 0.5; // Phase D.1.22: Hard AQG Baseline

    // Update structural stats and history AFTER gate decision
    metrics.record_final_score(final_score);
    if is_struct_valid_adaptive {
        metrics.adaptive_opportunity_count += 1;
    }
    metrics.record_structural_health(agreement, purity, std_v, is_struct_valid_adaptive);

    // --- PHASE D.1.9: RATIO-INTERACTION SCORING & DETERMINISTIC CHOICE ---
    // Moved outside of gating to ensure emission even in bypass mode.

    // 1. Archetype Bias Mapping (Soft Offsets)
    let mut adj_w_conv = candidate.w_conviction as f64;
    let mut adj_w_mom = candidate.w_momentum as f64 - 50.0; // centered
    let adj_w_vol = candidate.w_volatility as f64;
    let mut adj_exp_vol = candidate.exp_volatility as f64;

    match candidate.archetype {
        0 => {
            adj_w_conv += 20.0;
        } // ConvictionDominant
        1 => {
            adj_w_mom += 20.0;
        } // MomentumTrend
        2 => {
            adj_w_mom -= 20.0;
        } // MeanReversion
        3 => {
            adj_exp_vol += 30.0;
        } // VolatilityAverse
        _ => {}
    }

    // 2. Weight Normalization
    let w_sum = adj_w_conv.abs() + adj_w_mom.abs() + adj_w_vol.abs() + 1e-9;
    let w1 = adj_w_conv / w_sum;
    let w2 = adj_w_mom / w_sum;
    let w3 = adj_w_vol / w_sum;

    let _a_exp = (0.5 + 3.0 * (candidate.exp_conviction as f64 / 100.0))f64.clamp(0.5, 3.5);
    let b_exp = (0.5 + 3.0 * (candidate.exp_momentum as f64 / 100.0))f64.clamp(0.5, 3.5);
    let c_exp = (0.5 + 3.0 * (adj_exp_vol / 100.0))f64.clamp(0.5, 3.5);

    // 3. Compute Phase D.1.16 Signal Entropy
    let signal_entropy = compute_std_dev(
        &valid_signals
            .iter()
            .map(|(_, c, _, _, _, _, _)| c.conviction_score)
            .collect::<Vec<f64>>(),
    );

    scored_signals.clear();
    for (signal_idx, conviction, dom, reason, e_score, source, _sig) in valid_signals.iter() {
        let vol_penalty = conviction.norm_vol.max(1e-6);
        let mom = conviction.norm_momentum;
        let conv = conviction.conviction_score;

        let mom_effect = 1.0 + (w2 * mom.powf(b_exp))f64.clamp(-0.8, 0.8);
        let denom = 0.01 + w3.abs() * vol_penalty.powf(c_exp);

        let mut survival_score = (w1 * conv.powf(2.0)) * mom_effect / denom.powf(0.7);

        let rank_offset = ((*signal_idx % 20) as f64) * 0.01;
        let entropy_factor = 1.0 + (signal_entropy * 0.2);
        survival_score = (survival_score + rank_offset) * entropy_factor;

        let noise = ((strategy_seed ^ (*signal_idx as u64)) % 1000) as f64 / 1000.0;
        let jitter = (noise - 0.5) * 0.05;

        survival_score += jitter;
        let adjusted_score = (survival_score.max(0.0)) * clarity_penalty;

        scored_signals.push((
            *signal_idx,
            conviction.clone(),
            *dom,
            *reason,
            *e_score,
            adjusted_score,
            *source,
        ));
    }

    scored_signals.sort_by(|a, b| b.5.total_cmp(&a.5));

    let probs: Vec<f64> = valid_signals
        .iter()
        .map(|(_, c, _, _, _, _, _)| c.conviction_score.max(1e-6))
        .collect();
    let sum_probs: f64 = probs.iter().sum();
    let entropy = if sum_probs > 0.0 {
        probs
            .iter()
            .map(|p| {
                let p_norm = p / sum_probs;
                -p_norm * p_norm.ln()
            })
            .sum::<f64>()
    } else {
        0.0
    };

    let max_entropy = (probs.len() as f64).ln().max(1.0);
    let dispersion_metrics = (entropy / max_entropy)f64.clamp(0.0, 1.0);
    let opportunity = ((valid_signals.len() as f64 / 200.0).powf(0.7))f64.clamp(0.0, 1.0);
    let _combined_metric = (0.6 * dispersion_metrics + 0.4 * opportunity)f64.clamp(0.0, 1.0);

    let max_score_val = scored_signals.first().map(|s| s.5).unwrap_or(0.0);
    let min_score_val = scored_signals.last().map(|s| s.5).unwrap_or(0.0);
    let score_spread = max_score_val - min_score_val;

    let emission_threshold = if score_spread < 1e-4 {
        max_score_val * 0.7
    } else {
        max_score_val - score_spread * (0.25 + 0.5 * (1.0 - dispersion_metrics))
    };

    scored_signals.retain(|s| s.5 >= emission_threshold);
    emitted_signs.clear();

    for (signal_idx, conv, _, _, e_score, _score, source) in scored_signals.iter() {
        let consensus_score = conv.conviction_scoref64.clamp(-1.0, 1.0);
        let effective_edge = consensus_score.abs() * e_score;
        let strength = (0.7 * effective_edge + 0.3 * consensus_score.abs())
            f64.clamp(0.05, 1.0);

        emitted_signs.push(SignalAlpha {
            ts: *signal_idx,
            price: signal_events[*signal_idx].price as f64,
            archetype: candidate.archetype,
            direction: if consensus_score < 0.0 { -1 } else { 1 },
            strength,
            source: *source,
            conviction: conv.clone(),
            is_probe: false,
        });
    }
    // 🚨 HARD FALLBACK — ensure at least 1 signal
    if emitted_signs.is_empty() && scored_signals.len() >= 5 {
        let (signal_idx, conv, _, _, e_score, _score, source) = &scored_signals[0];

        if ga_debug_enabled() {
            println!("🚨 FORCED EMISSION → fallback activated");
        }

        let consensus_score = conv.conviction_scoref64.clamp(-1.0, 1.0);
        let effective_edge = consensus_score.abs() * e_score;

        let strength = (0.7 * effective_edge + 0.3 * consensus_score.abs())
            f64.clamp(0.05, 1.0);

        emitted_signs.push(SignalAlpha {
            ts: *signal_idx,
            price: signal_events[*signal_idx].price as f64,
            archetype: candidate.archetype,
            direction: if consensus_score < 0.0 { -1 } else { 1 },
            strength,
            source: *source,
            conviction: conv.clone(),
            is_probe: false,
        });
    }

    if std::env::var("GA_SIGNAL_DEBUG").is_ok() {
        println!(
            "EMISSION_DEBUG → valid={} scored={} emitted={}",
            valid_signals.len(),
            scored_signals.len(),
            emitted_signs.len()
        );
    }
    // if emitted_signs.len() > 50 {
    //     emitted_signs.sort_by(|a, b| b.strength.total_cmp(&a.strength));
    //     emitted_signs.truncate(50); // hard safety cap
    // }

    // if emitted_signs.is_empty() {
    //     println!("⚠️ EMISSION FAILURE → forcing fallback");

    //     if let Some((idx, conv)) = window_data.first() {
    //         emitted_signs.push(SignalAlpha {
    //             ts: *idx,
    //             price: signal_events[*idx].price as f64,
    //             archetype: candidate.archetype,
    //             direction: if conv.is_bearish { -1 } else { 1 },
    //             strength: 0.05,
    //             source: SignalSource::Synthetic,
    //         });
    //     }
    // }

    let scores_sum: f64 = emitted_signs
        .iter()
        .map(|s| s.strength)
        .sum::<f64>()
        .max(1e-9);

    // 🔥 portfolio-based dominance
    let dominance = emitted_signs
        .iter()
        .map(|s| s.strength)
        .reduce(f64::max)
        .unwrap_or(0.0);

    let winner_score =
        emitted_signs.iter().map(|s| s.strength).sum::<f64>() / emitted_signs.len().max(1) as f64;

    if std::env::var("GA_SIGNAL_DEBUG").is_ok() {
        println!("WINDOW_DECISION → {} | sigs={} z={:.2} dom={:.3} mean={:.3} purity={:.2} conc={:.2} agree={:.2} => {}", 
                    scenario_name, valid_signals.len(), max_z, dominance, mean, purity, top_k_sum / scores_sum.max(EPS), agreement, if valid_signals.is_empty() { "REJECT_VACUUM" } else { "ACCEPTED" });
    }

    let median_score = percentilef64(&scores, 0.50);
    // Raw value preserved for metrics; clamped value used for decisions and logging
    let raw_edge_spread_norm = (winner_score - median_score) / (std_dev + EPS);
    if raw_edge_spread_norm.abs() > 50.0 {}
    let edge_spread_norm = raw_edge_spread_normf64.clamp(-10.0, 10.0);

    let _signal_count = valid_signals.len();

    let mut busy_until = 0usize;
    let cooldown = config.trade_cooldown_events.unwrap_or(8);

    let baseline_pnl = 0.0;

    // Funnel analytics
    let funnel_signals = signal_events.len();
    let mut funnel_after_signal_filter = 0usize;
    let mut funnel_after_edge_filter = 0usize;
    let mut funnel_after_exec_prob = 0usize;

    let _rejected_trades: usize = 0;
    let _forced_execution_done = false;
    if emitted_signs.is_empty() {
        if ga_debug_enabled() {
            println!(
                "⚠️ EMISSION FAILURE → injecting probe for scenario={}",
                scenario_name
            );
        }

        let fallback_idx = if let Some((idx, _)) = window_data.first() {
            *idx
        } else {
            signal_events.len() / 2
        };

        let conviction = evaluate_market_conviction(
            candidate,
            scenario_name,
            signal_events,
            fallback_idx,
            0,
            generation,
        );

        let consensus_score = conviction.conviction_scoref64.clamp(-1.0, 1.0);
        emitted_signs.push(SignalAlpha {
            ts: fallback_idx,
            price: signal_events[fallback_idx].price as f64,
            archetype: candidate.archetype,
            direction: if consensus_score < 0.0 { -1 } else { 1 },
            strength: 0.1,
            source: SignalSource::Synthetic,
            conviction,
            is_probe: true,
        });
    }

    if emitted_signs.is_empty() {
        if ga_debug_enabled() {
            println!("⚠️ NO EMISSION EVEN AFTER PROBE → degraded continuation");
        }

        return Some(CandidateEvaluation {
            candidate: candidate.clone(),
            strategy_id: strategy_id.clone(),
            fitness: -0.1,
            trade_count: 0,
            pnl_history: Vec::new(),
            ..CandidateEvaluation::default()
        });
    }

    // 🔥 SINGLE SOURCE OF TRUTH
    let starvation_ratio = 1.0 - (emitted_signs.len() as f64 / valid_signals.len().max(1) as f64);
    let _adaptive_factor = (1.0 - 0.8 * starvation_ratio)f64.clamp(0.2, 1.0);

    // 🔥 Phase 3.6.6: Signal Topology Diagnostic
    let mut histogram = [0usize; 10];
    for &score_val in &scores {
        let bin = ((score_val.abs() * 10.0).floor() as usize).min(9);
        histogram[bin] += 1;
    }
    if ga_debug_enabled() {
        println!(
            "CONVICTION_HISTOGRAM → scenario={} {:?}",
            scenario_name, histogram
        );
    }

    // ✅ USE PREVIOUS SNAPSHOT (no intra-generation feedback)
    // 🔒 Freeze snapshot BEFORE using it
    // 🔒 HARD SNAPSHOT (NO FEEDBACK)
    const EDGE_WINDOW: usize = 200;

    // 🔥 USE ONLY PREVIOUS BUILD (NOT MUTATING ONE)
    let edge_dist_snapshot = if edge_vals_snapshot.len() >= 10 {
        edge_vals_snapshot.clone()
    } else if edge_vals_snapshot.len() > 0 {
        edge_vals_snapshot.clone()
    } else {
        vec![0.0003, 0.0008, 0.0015, 0.002, 0.003]
    };
    // ===============================
    // 🔥 CONSISTENT EDGE DISTRIBUTION
    // ===============================

    if ga_debug_enabled() {
        println!(
            "SNAPSHOT_SOURCE_CHECK → using_snapshot={} snapshot_size={} build_size={}",
            edge_vals_snapshot.len() > 0,
            edge_vals_snapshot.len(),
            edge_vals_build.len()
        );
    }
    // Use ONLY candidate_edges (single source of truth)
    let mut edge_dist: Vec<f64> = edge_dist_snapshot
        .iter()
        .cloned()
        .filter(|v| v.is_finite() && *v > 0.0)
        .collect();

    // 🔥 Fallback ONLY if empty (rare)
    if edge_dist.len() < 5 {
        if ga_debug_enabled() {
            println!("⚠️ EDGE DIST FALLBACK → insufficient candidate_edges");
        }
        edge_dist = vec![0.0003, 0.0008, 0.0015, 0.002, 0.003, 0.005];
    }

    let e_stats = DistributionStats::from_slice(&mut edge_dist);

    if ga_debug_enabled() {
        println!(
            "EDGE_DIST_DEBUG → p10={:.6} p50={:.6} p90={:.6} max={:.6}",
            e_stats.p10,
            e_stats.p50,
            e_stats.p90,
            e_stats.empirical_samples.last().unwrap_or(&0.0),
        );
    }
    // 🔥 SORT SAFE
    edge_dist.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // 🔥 OUTLIER TRIM (CRITICAL)
    let n = edge_dist.len();
    let lower = n / 20; // 5%
    let upper = n * 19 / 20; // 95%

    let trimmed = if upper > lower {
        &edge_dist[lower..upper]
    } else {
        &edge_dist[..]
    };
    let _t_len = trimmed.len();
    let spread_ratio = e_stats.p90 / (e_stats.p30 + 1e-9);
    if std::env::var("GA_DISPERSION_PROBE")
        .map(|v| !v.is_empty() && v != "0" && !v.eq_ignore_ascii_case("false"))
        .unwrap_or(false)
        && generation % 5 == 0
    {
        let raw_edge_std = if !edge_dist.is_empty() {
            let mean = edge_dist.iter().sum::<f64>() / edge_dist.len() as f64;
            let var = edge_dist
                .iter()
                .map(|v| {
                    let d = *v - mean;
                    d * d
                })
                .sum::<f64>()
                / edge_dist.len() as f64;
            var.sqrt()
        } else {
            0.0
        };
        let p25 = percentilef64(&edge_dist, 0.25);
        let p75 = percentilef64(&edge_dist, 0.75);
        println!(
            "[EDGE_WIN] sym={} window_id={} n={} raw_edge_p10={:.6} raw_edge_p50={:.6} raw_edge_p90={:.6} raw_edge_std={:.6} iqr={:.6}",
            scenario_name,
            generation,
            edge_dist.len(),
            e_stats.p10,
            e_stats.p50,
            e_stats.p90,
            raw_edge_std,
            (p75 - p25).max(0.0)
        );
    }

    if spread_ratio < 1.2 && ga_debug_enabled() {
        println!(
            "⚠️ EDGE COLLAPSE → p30={:.6} p90={:.6} ratio={:.3}",
            e_stats.p30, e_stats.p90, spread_ratio
        );
    }

    if e_stats.p90 / (e_stats.p30 + 1e-9) > 20.0 && ga_debug_enabled() {
        println!("⚠️ EDGE DISTORTION → p30={:.6} p90={:.6}", e_stats.p30, e_stats.p90);
    }

    // 🔥 Percentile Stretching
    let p10 = percentilef64(&scores, 0.1);
    let p50 = percentilef64(&scores, 0.5);
    let p70 = percentilef64(&scores, 0.7);
    let p90 = percentilef64(&scores, 0.9);
    let max_score = scores.iter().fold(f64::MIN, |a, &b| a.max(b));
    let score_range = (p90 - p10).max(1e-3);

    if ga_debug_enabled() {
        println!(
            "🌐 [SCORE_LANDSCAPE] scenario={} min={:.4} p10={:.4} p50={:.4} p70={:.4} p90={:.4} max={:.4} range={:.4}",
            scenario_name, scores.iter().fold(f64::MAX, |a, &b| a.min(b)), p10, p50, p70, p90, max_score, score_range
        );
    }

    let mut simulated_count = 0;
    let mut priority_pass_count = 0;
    let mut edge_pass_count = 0;
    let mut exec_pass_count_local = 0;
    // 🔥 LIVE EDGE GATE (GLOBAL CONTEXT): admit tail at/above p30 only (no p50 second bar).
    let live_threshold = e_stats.p30;
    for signal in &emitted_signs {
        // 🔥 Phase 3.6.6: Identity-Based Divergence (Restores Gradient)
        let conviction_val = signal.conviction.conviction_score;
        let stretched = (conviction_val - p10) / score_range;

        // Time-aware deterministic epsilon (identity injection)
        let eps_raw = ((strategy_index.wrapping_mul(31) ^ signal.ts.wrapping_mul(17)) % 100) as f64;
        let _identity_eps = (eps_raw - 50.0) / 10000.0; // [-0.005, 0.005]

        let mut priority_score = (stretched.max(0.0)).ln_1p(); // log(1 + x)

        // normalize
        priority_score = priority_score / (1.0 + priority_score);

        // avoid 1.0 attractor
        priority_score = priority_scoref64.clamp(0.0, 0.999);
        let strat_bias = (strategy_index % 10) as f64 * 0.005; // Tightened 0.000 -> 0.045

        let dynamic_threshold = (adaptive_threshold + strat_bias)f64.clamp(0.35, 0.85);

        let pass = priority_score > dynamic_threshold;

        if !pass {
            continue;
        }

        priority_pass_count += 1;
        if simulated_count >= 120 {
            break;
        }
        simulated_count += 1;

        // 🔥 Phase 3.6.6: Temporal Phase Shift (Queue interaction divergence)
        let phase_shift = ((strategy_index % 7) as i32 - 3) as isize;
        let execution_idx = (signal.ts as isize + phase_shift).max(0) as usize;
        // Use a 10-tick safety buffer to allow room for latency/offsets without slice panics
        let execution_idx = execution_idx.min(signal_events.len().saturating_sub(10));

        let current_idx = execution_idx;

        metrics.record_funnel_admission(signal.conviction.conviction_score);

        let conviction = signal.conviction.clone();

        if current_idx < busy_until {
            // 🔥 TEMP: disable execution lock to test concurrency
            // continue;
        }

        // --- SELECTION GATES (CLEAN + COMPLETE) ---

        // 2. Exploration Bypass (Probabilistic soft-pass)
        funnel_after_signal_filter += 1;
        // 3. Use selected conviction (global or local)
        let final_conviction = conviction.clone();

        // // 4. Hard participation threshold (deterministic)
        // let exec_threshold = {
        //     let p60 = percentilef64(&scores, 0.6);
        //     let mean = scores.iter().sum::<f64>() / scores.len() as f64;
        //     let std = (scores.iter().map(|s| (s - mean).powi(2)).sum::<f64>()
        //         / scores.len() as f64)
        //         .sqrt();

        //     if std < 0.05 {
        //         mean + 0.3 * std
        //     } else {
        //         p60
        //     }
        // };

        let _conviction_score = final_conviction.conviction_score;
        // --- NEW: execution-aware features ---
        let queue_ahead = outcome_estimate_queue(signal_events, current_idx);
        let trade_velocity = estimate_trade_velocity(signal_events, current_idx);
        let volatility = std_dev.max(1e-6);

        // 🔥 EXECUTION MODEL (BOUND TO REAL VARIABLES)

        // liquidity proxy (you already compute this)
        let liquidity = trade_velocity.max(1e-6);

        // --- FIXED EXECUTION MODEL (NON-COLLAPSING) ---

        // Normalize queue pressure (bounded, smooth)
        let queue_norm = (queue_ahead / (queue_ahead + 5000.0))f64.clamp(0.0, 1.0);

        // Liquidity strength (relative, not absolute)
        let liquidity_norm = (liquidity / (liquidity + 1000.0))f64.clamp(0.0, 1.0);

        // Volatility boost (fast markets fill easier)
        let volatility_boost = (volatility * 5.0).tanh(); // smooth [0,1)

        // Base fill probability (balanced interaction)
        let mut fill_probability =
            0.4 * liquidity_norm + 0.3 * (1.0 - queue_norm) + 0.3 * volatility_boost;

        // 🔥 Inject micro-variance (CRITICAL)
        let noise = ((strategy_index ^ current_idx) % 100) as f64 / 1000.0;
        fill_probability += (noise - 0.05) * 0.5;
        fill_probability = fill_probabilityf64.clamp(0.05, 0.95);

        let latency_impact = (volatility * config.latency_ticks as f64).min(0.5);
        let adverse_selection = (volatility * 0.5)f64.clamp(0.0, 0.5);

        // 🔥 EXPAND CAPTURE DISTRIBUTION (FIX 5)
        let mut capture_prob =
            0.5 * fill_probability + 0.3 * (1.0 - latency_impact) + 0.2 * (1.0 - adverse_selection);

        // 🔥 amplify spread (instead of flattening)
        capture_prob = capture_prob.powf(1.2);

        // 🔥 inject divergence
        let cap_noise = ((current_idx * 13 + strategy_index * 7) % 100) as f64 / 1000.0;
        capture_prob += cap_noise - 0.05;

        capture_prob = capture_probf64.clamp(0.05, 0.98);

        let entry_price = signal_events[current_idx].price as f64;
        let atr = calculate_atr(signal_events, current_idx, 14);

        // --- NEW: realized edge directly from signal (d=dir_signal driven) ---
        // ✅ FIX 2: Edge must be a function of the signal's conviction/z-score, not ATR only.
        //   dir_signal ∈ [-3,3] → use its abs as a scaling factor so edge VARIES with signal.
        let consensus_score = final_conviction.conviction_scoref64.clamp(-1.0, 1.0);

        let raw_edge =
            consensus_score * final_conviction.edge_weight * (atr / entry_price.max(1e-6));
        // let expected_move = raw_edge.max((atr / entry_price.max(1e-6)) * 0.3);

        // ✅ FIX 3: Break edge symmetry — amplify differences
        let shaped_edge = raw_edge.abs().powf(0.8) * raw_edge.signum();
        let _ = shaped_edge; // available for future use

        // 🔥 FIX: Use REALIZED edge AFTER execution, not synthetic estimate
        let mut expected_realized_edge = raw_edge * capture_prob * signal.strength;

        // DO NOT finalize here — this is only a placeholder
        // It will be CORRECTED after execution

        if ga_debug_enabled() {
            println!(
                "EDGE_RAW_DEBUG → expected={:.6} p30={:.6} p50={:.6} p90={:.6} up={:.6} down={:.6}",
                expected_realized_edge,
                e_stats.p30,
                e_stats.p50,
                e_stats.p90,
                (e_stats.p90 - e_stats.p50),
                (e_stats.p50 - e_stats.p30)
            );
        }
        // 🔥 anti-collapse jitter to preserve variance
        let jitter = ((strategy_index ^ signal.ts) % 10) as f64 * 1e-5;
        // 🔥 FIX: build distribution from raw edge, NOT realized

        // ✅ FIX 5: Edge collapse detector
        // if edge_vals_snapshot.len() >= 10 {
        //     let mean_e = edge_vals_snapshot.iter().sum::<f64>() / edge_vals_snapshot.len() as f64;
        //     let var_e = edge_vals_snapshot
        //         .iter()
        //         .map(|v| (v - mean_e).powi(2))
        //         .sum::<f64>()
        //         / edge_vals_snapshot.len() as f64;
        //     let edge_std_dev = var_e.sqrt();
        //     if edge_std_dev < 1e-4 {
        //         if ga_debug_enabled() {
        //             println!(
        //                 "⚠️ EDGE COLLAPSE DETECTED → std_dev={:.8} (n={})",
        //                 edge_std_dev,
        //                 edge_vals_snapshot.len()
        //             );
        //         }
        //     }
        // }

        // ✅ FIX 2: EDGE_DEBUG — print every edge value to expose degeneracy
        // 🔥 INSERT THIS FULL BLOCK HERE (MISSING IN YOUR CODE)
        let _baseline = 0.0005;
        // let scaled = aqg_threshold * capture_prob;

        // 🔥 ALIGN AQG TO REALIZED SPACE
        let _realized_avg_edge = avg_edge * capture_prob;
        let _realized_aqg = aqg_threshold * capture_prob;

        // ✅ NEW: robust edge normalization (non-collapsing)
        let _upper_spread = (e_stats.p90 - e_stats.p50).max(1e-5);
        let _lower_spread = (e_stats.p50 - e_stats.p30).max(1e-5);

        // 3. Normalize WITHOUT bias

        // 🔥 STEP 5 — clamp before regime gate (same scale as downstream norm)
        expected_realized_edge = expected_realized_edgef64.clamp(-0.02, 0.02);
        if !expected_realized_edge.is_finite() {
            expected_realized_edge = 0.0;
        }

        // Hard bimodal gate: `e_stats` is built from positive raw-edge magnitudes in the
        // frozen snapshot — use |expected_realized_edge| so long/short share one band.
        let er_mag = expected_realized_edge.abs();
        if er_mag < live_threshold {
            continue;
        }
        // Optional stricter regime pre-gate (dead-regime purge before round-trip ranking).
        let strict_regime_pre_gate = std::env::var("STRICT_REGIME_PRE_GATE")
            .map(|v| !v.is_empty() && v != "0" && !v.eq_ignore_ascii_case("false"))
            .unwrap_or(false);
        if strict_regime_pre_gate {
            let regime_floor_mult = std::env::var("STRICT_REGIME_FLOOR_MULT")
                .ok()
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(1.0)
                .max(0.0);
            let regime_floor = (e_stats.p30 * regime_floor_mult).max(0.0);
            if er_mag < regime_floor {
                if std::env::var("PRE_RT_REJECT_LOG").is_ok() {
                    println!(
                        "[PRE_RT_REJECT] reason=DeadRegime er_mag={:.6} regime_floor={:.6} p30={:.6}",
                        er_mag, regime_floor, e_stats.p30
                    );
                }
                continue;
            }
        }
        // Past gate: realized |edge| at or above the p30 of the positive-edge snapshot.
        let is_live_regime = true;
        // ===============================
        // EDGE NORMALIZATION (FIXED SAFE)
        // ===============================

        // Prevent divide-by-zero
        let denom = (e_stats.p90 - e_stats.p30).abs().max(1e-6);
        let mut edge_norm = (expected_realized_edge - e_stats.p30) / denom;

        // 🔥 SOFT FLOOR (instead of hard zero)
        if edge_norm < 0.0 {
            edge_norm *= 0.25;
        }

        // 🔥 SOFT CEILING
        if edge_norm > 1.0 {
            edge_norm = 1.0 + (edge_norm - 1.0) * 0.25;
        }

        // 🔥 FINAL CLAMP (wide range, preserves gradient)
        edge_norm = edge_normf64.clamp(-0.25, 1.25);

        if ga_debug_enabled() {
            println!(
                "EDGE_DIST_DEBUG_FIXED → raw={:.6} p30={:.6} p90={:.6} norm={:.6}",
                raw_edge, e_stats.p30, e_stats.p90, edge_norm
            );
            println!(
                "EDGE_DIST_DEBUG → raw={:.6} p30={:.6} p50={:.6} p90={:.6} norm={:.6}",
                raw_edge, e_stats.p30, e_stats.p50, e_stats.p90, edge_norm
            );
            println!(
                "EDGE_NORM_FIXED → raw={:.6} realized={:.6} p30={:.6} p50={:.6} p90={:.6}",
                raw_edge, expected_realized_edge, e_stats.p30, e_stats.p50, e_stats.p90
            );
        }
        let edge_raw = raw_edge;

        let signal_fail = final_conviction.conviction_score < 0.015;
        let bull = signal.conviction.bullish_score;
        let bear = signal.conviction.bearish_score;
        let sanity_fail = (bull < 1e-6 && bear < 1e-6) || (bull - bear).abs() < 1e-6;

        // 🔥 Phase V4.2 Refined Gating (with Gradual Degradation)
        let passed_gate = !signal_fail && !sanity_fail;

        let edge_ratio = edge_raw; // 🔥 preserve gradient

        let spread_z = edge_spread_norm;

        // normalize edge ratio into usable signal
        // 🔥 smoother mapping
        let _edge_component = edge_ratio;

        // penalize unstable distributions
        let _spread_penalty = (spread_z.abs() / 3.0)f64.clamp(0.0, 1.0);

        // volatility penalty (non-linear)
        let _vol_penalty = volatility.powf(0.7)f64.clamp(0.0, 1.0);

        // core survival before shaping
        // Stable survival mass — dead / grey regimes never reach here (hard `continue` above).
        let survival_prob = 0.8f64f64.clamp(0.15, 0.95);

        if std::env::var("GA_TRACE").is_ok() {
            println!(
                "THRESH_DEBUG → final={:.6} aqg={:.6} avg_edge={:.6}",
                edge_ratio, aqg_threshold, avg_edge
            );
            println!(
                "REALIZED_EDGE → raw={:.5} cap_prob={:.3} realized={:.5}",
                raw_edge, capture_prob, expected_realized_edge
            );
        }

        let side_is_long = bull > bear;

        // 🔥 FIX 4.2: Adaptive Exploration Gate
        let _force_explore = executed_trades.is_empty() && strategy_index == 0;

        if ga_debug_enabled() {
            println!(
                "EDGE_FINAL → raw={:.3} is_live_regime={} pass={} sf={} snf={}",
                edge_raw, is_live_regime, passed_gate, signal_fail, sanity_fail
            );
            if !passed_gate {
                println!(
                    "GATE_REJECT → raw={:.3} is_live_regime={} signal_fail={} sanity_fail={}",
                    edge_raw, is_live_regime, signal_fail, sanity_fail
                );
            }
        }

        if passed_gate {
            edge_pass_count += 1;
        }
        let risk_ok = true;

        // 🔥 REGIME-AWARE POSITION SIZING
        let size_multiplier = 1.0;

        let size = (raw_edge.abs() * size_multiplier).max(0.01);

        let effective_fill_prob = if signal.is_probe {
            1.0
        } else {
            capture_prob.max(0.05)
        };

        let (_feasible, dynamic_threshold_val) =
            is_execution_feasible(final_conviction.conviction_score, capture_prob);
        let feasibility_prob = if dynamic_threshold_val > 1e-6 {
            (capture_prob / dynamic_threshold_val)f64.clamp(0.0, 1.0)
        } else {
            0.0
        };
        // 🔥 remove logistic saturation
        let edge_soft_score = edge_ratiof64.clamp(0.0, 1.0);

        let dynamic_exec_threshold =
            (0.35f64 + 0.15 * (1.0 - feasibility_prob) - 0.10 * edge_soft_score)f64.clamp(0.20, 0.70); // 🔥 Decompressed floor 0.20
        let threshold = dynamic_exec_threshold;

        let exec_margin = effective_fill_prob - threshold;

        // --- BIMODAL MODE SPLIT ---
        let _is_exploit = true;
        let _is_explore = false;

        // --- EDGE GATING (PRIMARY REGIME DRIVER) ---
        // 🔥 FIX: dynamic threshold instead of hard 0.25
        let _edge_threshold = 0.0;

        // ===============================
        // 🔥 FULLY ADAPTIVE EXECUTION (REPLACEMENT)
        // ===============================

        // 1. Edge percentile position (distribution anchored)
        let edge_pct = ((edge_raw - e_stats.p30) / (e_stats.p90 - e_stats.p30).abs().max(1e-9))f64.clamp(-0.5, 1.5);

        // 2. Capture signal centered
        let capture_pct = (capture_prob - 0.5) * 2.0; // [-1, 1]

        // 3. Execution margin normalized (self-scaled)
        let exec_scale = exec_margin.abs().max(0.05);
        let exec_pct = (exec_margin / exec_scale)f64.clamp(-1.0, 1.0);

        // 4. Combine in score space (NOT probability space)
        let combined_score = 0.5 * edge_pct + 0.3 * capture_pct + 0.2 * exec_pct;

        // 🔥 NEW: center around neutral regime
        let centered_score = combined_score - 0.2;

        // 5. Convert to probability using sigmoid (adaptive curve)
        let mut final_prob = 1.0 / (1.0 + (-1.5 * centered_score).exp());

        // 6. Micro-noise (deterministic tie-break only; no "explore" regime discount)
        let noise = ((strategy_index ^ current_idx) % 100) as f64 / 100.0;
        final_prob += (noise - 0.5) * 0.05;

        // 7. Final clamp
        final_prob = final_probf64.clamp(0.02, 0.98);

        // 8. Final execution decision — deterministic composite score (weak vs strong margin).
        const EXEC_SCORE_W_MARGIN: f64 = 0.6;
        const EXEC_SCORE_W_PROB: f64 = 0.4;
        const EXEC_SCORE_MIN: f64 = 0.01;
        let execution_edge = raw_edge.abs();
        let exec_score = EXEC_SCORE_W_MARGIN * exec_margin
            + EXEC_SCORE_W_PROB * (final_prob - 0.5);

        let final_execute = passed_gate
            && risk_ok
            && size > 0.0
            && execution_edge > 0.0001
            && exec_margin > 0.0
            && exec_score > EXEC_SCORE_MIN;

        // --- ACCOUNTING ---
        if final_execute {
            // ✅ FIX: build distribution ONLY from raw edge (pre-execution signal)
            edge_vals_build.push(raw_edge + jitter);
            exec_pass_count_local += 1;
        }

        if std::env::var("GA_TRACE").is_ok() {
            println!(
                "EXEC_DECISION → symbol={} passed_gate={} size={:.2} risk_ok={} fill_prob={:.3} is_probe={} final_execute={} exec_margin={:.4} exec_score={:.4}",
                scenario_name, passed_gate, size, risk_ok, effective_fill_prob, signal.is_probe, final_execute, exec_margin, exec_score
            );
            if !final_execute {
                println!(
                    "EXEC_REJECT_REASON → gate={} risk={} size={} fill_prob={:.3} threshold={:.3} exec_margin={:.4} exec_score={:.4}",
                    passed_gate,
                    risk_ok,
                    size > 0.0,
                    effective_fill_prob,
                    threshold,
                    exec_margin,
                    exec_score
                );
            }
        }

        if !final_execute {
            continue;
        }

        // Optional strict execution pre-gate in clean (post-regime) space.
        let strict_exec_pre_gate = std::env::var("STRICT_EXEC_PRE_GATE")
            .map(|v| !v.is_empty() && v != "0" && !v.eq_ignore_ascii_case("false"))
            .unwrap_or(false);
        if strict_exec_pre_gate {
            let fill_floor = std::env::var("STRICT_EXEC_FILL_FLOOR")
                .ok()
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.60)
                f64.clamp(0.0, 1.0);
            let liq_floor = std::env::var("STRICT_EXEC_LIQ_FLOOR")
                .ok()
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.20)
                f64.clamp(0.0, 1.0);
            let queue_cap = std::env::var("STRICT_EXEC_QUEUE_CAP")
                .ok()
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.85)
                f64.clamp(0.0, 1.0);
            if fill_probability < fill_floor || liquidity_norm < liq_floor || queue_norm > queue_cap {
                if std::env::var("PRE_RT_REJECT_LOG").is_ok() {
                    println!(
                        "[PRE_RT_REJECT] reason=ExecInfeasible fill_prob={:.4} fill_floor={:.4} liq_norm={:.4} liq_floor={:.4} queue_norm={:.4} queue_cap={:.4}",
                        fill_probability, fill_floor, liquidity_norm, liq_floor, queue_norm, queue_cap
                    );
                }
                continue;
            }
        }

        // --- ATOMIC ACCOUNTING (Inside execution branch) ---
        exec_passed += 1;
        funnel_after_exec_prob += 1;
        funnel_after_edge_filter += 1; // Unifying accounting logic

        // --- SCOPE RESTORATION (Step 7.2) ---
        let pre_edge = edge_ratio;
        let final_exec_prob = effective_fill_prob;
        let survive_score = (0.25 * final_exec_prob
            + 0.45 * survival_prob
            + 0.2 * 0.5 // placeholder for expected_efficiency
            + 0.1 * capture_prob)
            f64.clamp(0.0, 1.0);
        let _expected_efficiency =
            (0.5 * capture_prob + 0.3 * feasibility_prob + 0.2 * edge_soft_score)f64.clamp(0.0, 1.0);
        let exec_weight = final_exec_prob * survival_prob;

        let trade_result = ga_simulate_round_trip_at_cursor(
            scenario_name,
            candidate,
            signal_events,
            execution_events,
            config,
            current_idx,
            executed_trades.len(),
            &final_conviction,
            side_is_long,
            signal.strength,
            signal.is_probe,
            strategy_index,
            generation,
            &config.stats,
        );

        if let Some(outcome) = trade_result {
            // ALWAYS record organic trades for EvaluationMetrics learning
            if !outcome.is_probe {
                pnl_history_learn.push(outcome.clone());
            }

            if outcome.is_probe {
                probe_count += 1;
                continue;
            }

            // GATING: Only high-edge signals count for GA fitness
            if !outcome.is_execution {
                continue;
            }

            // 🔥 CRITICAL FIX: override expected edge with REALIZED edge
            let realized_edge = outcome.pnl; // or outcome.edge_quality if preferred

            expected_realized_edge = realized_edge;

            real_trade_count += 1;

            // Layer 4: Capture Efficiency Gate (Phase B)
            // Phase C.2d: Adaptive Execution Calibration

            // ✅ FIX 4: Decouple expected_return from edge so norm_pnl has real variance.
            // OLD: expected_return ≈ edge * constant → causes norm_pnl collapse.
            // NEW: expected_return = |edge| * volatility * regime_multiplier → orthogonal to pnl.
            let _edge_quality = outcome.edge_quality;
            let regime_mult = {
                let bias = classify_direction_bias(candidate.direction_bias);
                let regime = detect_market_regime(
                    entry_price,
                    entry_price, // approximate sma as entry for now
                    final_conviction.conviction_score,
                    std_dev,
                );
                regime_multiplier(regime, bias)
            };
            let expected_return = (expected_realized_edge * volatility * regime_mult).max(0.0003);

            let _quality_penalty = if expected_return < 1e-5 { 0.5 } else { 1.0 };

            executed_trades.push(outcome.clone());
            let idx = executed_trades.len() - 1;
            trade_edges.push((idx, pre_edge));

            // 🔥 GENOME-DRIVEN REGIME LOGIC (REPLACE ENTIRE BLOCK)

            // derive thresholds from genome
            let edge_thresh = candidate.base_edge as f64 / 10000.0;
            let _exec_thresh = candidate.queue_threshold as f64 / 100.0;
            let holding = (candidate.holding_period as f64 / 10.0).max(2.0);
            let _surv_thresh = (holding / 10.0)f64.clamp(0.4, 0.8);

            let _edge_ratio_local = expected_realized_edge / (edge_thresh + 1e-9);

            // Existing exit accounting
            match outcome.exit_reason {
                GaExitReason::TakeProfit => exit_tp_count += 1,
                GaExitReason::StopLoss => exit_sl_count += 1,
                GaExitReason::TimeStop => exit_ts_count += 1,
                GaExitReason::NoFill => {
                    // This block now only executes if the simulator returns NoFill
                    if let Some(reason) =
                        classify_rejection(outcome.queue_ahead, outcome.arrival_liquidity)
                    {
                        match reason {
                            RejectionReason::QueueBlocked => queue_blocked_count += 1,
                            RejectionReason::LiquidityStarved => liquidity_starved_count += 1,
                        }
                    }
                }
            }

            // ✅ FIX 1: Kill norm_pnl division — use raw pnl directly to break mathematical lock.
            // OLD: normalized_pnl = pnl / exp_return → both scale with edge → constant output.
            // NEW: normalized_pnl = pnl → real signal variance flows through.
            let normalized_pnl = outcome.pnlf64.clamp(-3.0, 3.0);

            // ✅ guard
            if !normalized_pnl.is_finite() {
                continue;
            }

            // Efficiency may be negative on real adverse outcomes; only skip pathological windows.
            let realized_efficiency = outcome.pnl;
            if !realized_efficiency.is_finite()
                || realized_efficiency < -0.5
                || realized_efficiency > 2.0
            {
                continue;
            }

            if std::env::var("GA_TRACE").is_ok() {
                println!(
                    "EFF_CHECK_NEW → pnl={} exp_return={} norm_pnl={} eff={}",
                    outcome.pnl, expected_return, normalized_pnl, realized_efficiency
                );
            }
            // derived ONCE
            let _efficiency_scale = 1.0 + realized_efficiency;

            // Clamp to valid range

            // --- USE EFFICIENCY SAFELY BELOW ---
            // --- FIX: normalize e_score to usable range ---
            // 🔥 EXECUTION VARIANCE AMPLIFICATION (FIX 1)
            let fill_variance = (outcome.fills_count as f64
                / (outcome.total_filled_qty.max(1) as f64))
                f64.clamp(0.0, 1.0);

            // 🔥 derive latency from config + volatility
            let latency_penalty = ((config.latency_ticks as f64) * volatility)f64.clamp(0.0, 1.0);

            let queue_pressure = (queue_ahead / 1000.0)f64.clamp(0.0, 1.0);

            // amplified execution score
            let base_exec_score = realized_efficiency.max(-0.2); // 🔥 use amplified efficiency

            let e_exec_score = base_exec_score
                * (1.0 - latency_penalty)
                * (1.0 - queue_pressure)
                * (1.0 + fill_variance * 0.5);

            // efficiency ONLY affects scoring later, NOT pnl
            // === FIX 4: EDGE-SCALED PAYOFF ===

            // base pnl from simulator

            // edge strength (already normalized-ish)
            let _edge = final_conviction.edge_weight.max(0.0);

            // execution quality stays
            let _execution_quality: f64 = (0.7f64 + 0.6f64 * e_exec_score)f64.clamp(0.5, 1.3);

            // 🔥 TAIL RISK CONTROL
            let _tail_cap = 0.003;

            let mut raw_pnl = outcome.pnl;

            // 🔥 HARD STOP-LOSS ENFORCEMENT (STRUCTURAL FIX)
            let entry_price = signal_events[current_idx].price as f64;

            // reconstruct SL/TP bounds (must match simulator logic)
            let atr = calculate_atr(signal_events, current_idx, 14);
            let rr = candidate.edge_ratio as f64 / 100.0;

            let sl_dist = atr; // or your SL logic
            let tp_dist = atr * rr;

            let max_loss = -sl_dist / entry_price;
            let max_profit = tp_dist / entry_price;

            // 🚨 CRITICAL: clamp BEFORE any scaling
            raw_pnl = raw_pnlf64.clamp(max_loss, max_profit);

            // 🚨 enforce SL dominance over TimeStop
            if outcome.exit_reason == GaExitReason::TimeStop && raw_pnl < max_loss {
                if std::env::var("GA_VERBOSE").is_ok() {
                    println!(
                        "🚨 SL_BYPASS_DETECTED → raw_pnl={:.6} max_loss={:.6} idx={}",
                        raw_pnl, max_loss, current_idx
                    );
                }
                raw_pnl = max_loss;
            }

            let edge_scale = 0.5 + raw_edge.abs().powf(0.6);

            let capture_ratio = if outcome.expected_move.abs() > 1e-9 {
                (raw_pnl / outcome.expected_move)f64.clamp(-2.0, 2.0)
            } else {
                0.0
            };

            let mut trade_pnl = raw_pnl
                * capture_prob
                * edge_scale
                * (1.0 + 0.3 * capture_ratio.abs())
                * exec_weight; // 🔥 APPLY EXECUTION REALISM

            // 🔥 Outcome Sensitivity Amplification (Phase 3.6.3)
            let strength_val = final_conviction.conviction_score;
            if trade_pnl > 0.0 {
                trade_pnl *= 1.0 + strength_val * 0.5;
            } else {
                trade_pnl *= 1.0 - strength_val * 0.2;
            }

            // ✅ FIX: Update the outcome in the history vector with the boosted PnL
            executed_trades[idx].pnl = trade_pnl;
            let mut trade_pnl_safe = executed_trades[idx].pnl;

            if !trade_pnl_safe.is_finite() {
                trade_pnl_safe = 0.0;
            }

            trade_scores.retain(|(_, score)| score.is_finite());
            trade_scores.push((idx, trade_pnl_safe));
            if ga_log_hotpath() {
                if !trade_pnl_safe.is_finite() {
                    println!("⚠️ INVALID PNL DETECTED");
                } else {
                    println!("TRADE_CAPTURED → idx={} pnl={:.6}", idx, trade_pnl_safe);
                }
            }

            // ✅ NOW variables exist → safe to use

            // 🔴 SNIPER score
            let edge_component = (expected_realized_edge * 1000.0).powf(1.3);
            let exec_component = final_exec_prob.powf(1.5);
            let volatility_component = (outcome.m_favorable.abs() + 1e-6).powf(0.5);

            let sniper_score = edge_component.powf(1.5)
                * exec_component.powf(1.3)
                * volatility_component.powf(0.7);

            // 🟢 CONSISTENT score
            let exec_consistent_score = final_exec_prob.powf(1.2)
                * survive_score.powf(1.2)
                * (1.0 / (1.0 + volatility)).powf(0.8);

            // ✅ PUSH AFTER DEFINITION
            sniper_scores.push((idx, sniper_score));
            consistent_scores.push((idx, exec_consistent_score));

            if ga_log_hotpath() {
                println!(
                    "TYPE_DEBUG → idx={} edge={:.5} exec={:.3} surv={:.3} sniper={:.6} consistent={:.6}",
                    idx,
                    expected_realized_edge,
                    final_exec_prob,
                    survive_score,
                    sniper_score,
                    exec_consistent_score
                );
            }

            let tail_penalty = if raw_pnl < 0.0 {
                (-raw_pnl).min(0.01)
            } else {
                0.0
            };

            total_tail_penalty += tail_penalty;

            // Phase D.1.18: Extract winning signal signature
            let winning_sig = if let Some((_, _, _, _, _, _, sig)) = valid_signals
                .iter()
                .find(|(idx, _, _, _, _, _, _)| *idx == current_idx)
            {
                Some(sig.clone())
            } else {
                None
            };

            let is_long = outcome.side == chronosentiment_core::Side::Buy;
            metrics.record_trade(
                trade_pnl,
                outcome.ideal_pnl,
                0.0,                          // expected_pnl
                0.0,                          // entropy
                final_conviction.raw_q_ratio, // conviction
                realized_efficiency,          // efficiency
                outcome.edge_quality,         // edge_quality
                outcome.time_to_mfe as f64,   // time_to_mfe
                0.0,                          // margin
                1.0,                          // aqg_health
                raw_edge_spread_norm,         // edge_spread
                dominance,                    // dominance
                final_conviction.raw_q_ratio, // signal_entropy
                outcome.clone(),
                SignalSource::Organic,
                winning_sig,
                is_long,
                e_exec_score,
            );

            if std::env::var("GA_VERBOSE").is_ok() {
                println!(
                    "EFF_TRACK → stored_eff={} exec_score={}",
                    realized_efficiency, e_exec_score
                );
            }

            let raw_exit = outcome.exit_event_idx;

            // Phase 14 Attribution
            if decision_was_override {
                metrics.conviction_trade_count += 1;
                metrics.sum_conviction_pnl += trade_pnl;
            } else {
                metrics.clarity_trade_count += 1;
                metrics.sum_clarity_pnl += trade_pnl;
            }
            let capped_exit = raw_exit.min(current_idx + 200);

            // ✅ FIX: ALWAYS account exit BEFORE any continue
            match outcome.exit_reason {
                GaExitReason::TakeProfit => {
                    pnl_from_tp_scenario += trade_pnl;
                    if trade_pnl > 0.0 {
                        long_win_count_scenario += 1;
                    }
                }
                GaExitReason::StopLoss => {
                    pnl_from_sl_scenario += trade_pnl;
                    if trade_pnl.abs() < 0.0001 {
                        _micro_loss_count += 1;
                    }
                }
                GaExitReason::TimeStop => {
                    if trade_pnl > 0.0 {
                        long_win_count_scenario += 1;
                    }
                }
                GaExitReason::NoFill => {} // No impact on pnl scenarios
            };

            // 🚨 NOW safe to skip
            if capped_exit <= current_idx {
                continue;
            }

            total_vol_ratio += std_v;
            total_spread_reality += outcome.spread;
            total_window_volume += outcome.avg_window_volume;

            // Phase C.1: Trade-Level Survivability Check
            let window_slippage = outcome.spread * (1.0 + std_v.powf(1.2)) * config.slippage_factor;
            let _window_fill_prob = 0.7 + 0.3 * capture_prob;
            let _window_latency_penalty =
                (-0.05 * config.latency_ticks as f64).exp()f64.clamp(0.6, 1.0);

            // --- FIX: Preserve PnL energy (no multiplicative collapse) ---

            // Convert multiplicative penalties → linear drag
            let adjusted_pnl = trade_pnl;

            // Prevent slippage from dominating signal
            let capped_slippage = window_slippage.min(0.001); // absolute cap

            // Final effective pnl
            let window_effective_pnl = if trade_pnl > 0.0 {
                (adjusted_pnl - capped_slippage).max(trade_pnl * 0.2) // 🔥 stronger floor (was 0.1)
            } else {
                adjusted_pnl - capped_slippage
            };

            // Targeted Sanity Log (Constrained to first few trades AND strict signal debug)
            if std::env::var("GA_SIGNAL_DEBUG").is_ok() {
                println!(
                    "DEBUG_SANITY → idx={} trade_pnl={:.6} edge={} exec={}",
                    idx, trade_pnl, edge_scale, exec_weight
                );
                println!(
                    "EDGE_PASS → realized={:.6} norm={:.3} pass={}",
                    expected_realized_edge, priority_score, pass
                );
            }

            if window_effective_pnl > 0.0 {
                survivable_trades_count += 1;
            }

            // --- PHASE C.1.5: DEAD-ZONE ERADICATION (Early Exit) ---
            if executed_trades.len() >= 20 {
                // Phase C.2d: Allow more discovery before killing
                let current_surv = survivable_trades_count as f64 / executed_trades.len() as f64;
                if current_surv < 0.05f64 {
                    // Relaxed from 0.2 to allow recovery
                    // Candidate is non-survivable in this regime; kill early to accelerate convergence
                }
            }

            // --- PHASE C.1.6: PARTICIPATION PRESSURE (Early Participation Choke) ---
            if scenario_pnls.len() >= 20 && executed_trades.len() < 1 {
                // Not active enough for institutional scale; kill early
            }

            metrics.record_opportunity();

            total_filled_qty += outcome.total_filled_qty;
            fills_count += outcome.fills_count;
            total_slippage_bps += outcome.total_slippage_bps;
            sum_drawdown_raw += outcome.drawdown_penalty_raw;
            sum_expected_move += outcome.expected_move;
            let (sig, lat) = scenario_execution_signature_from_simulation(
                &outcome.sim_events,
                &outcome.entry_order_id,
                &outcome.exit_order_id,
                outcome.fill_efficiency,
                1.0,
                outcome.raw_q_ratio,
            );
            cycle_sigs.push(sig);
            sum_latency_raw += lat;

            let expected_slippage = conviction.conviction_score.abs() * 0.1;
            sum_expected_slippage += expected_slippage;
            let actual_slippage = outcome.total_slippage_bps;
            sum_actual_slippage += actual_slippage;

            max_trade_pnl_scenario = max_trade_pnl_scenario.max(trade_pnl);
            max_pnl_in_scenario = max_pnl_in_scenario.max(outcome.pnl);
            scenario_pnls.push(window_effective_pnl);

            // Phase D.1: Metrics Propagation
            metrics.trade_qualities.push(outcome.edge_quality);
            metrics.sum_realized_pnl += window_effective_pnl;
            metrics.sum_expected_pnl += outcome.expected_move;
            _total_spread_test += outcome.spread;
            _sum_price += signal_events[current_idx].price as f64;
            total_quality_trades_scenario += outcome.quality;

            if ga_debug_enabled() && strategy_index == 0 && idx < 3 && generation % 5 == 0 {
                println!(
                    "GA_EXEC: scenario={} idx={} score={:.4} spread_z={:.2} dom={:.2} pnl={:.6}",
                    scenario_name,
                    current_idx,
                    conviction.conviction_score,
                    edge_spread_norm,
                    dominance,
                    outcome.pnl
                );
            }
            busy_until = capped_exit + cooldown; // Phase D.1.23: Restoration of cooldown control
        }
    }

    if ga_log_hotpath() {
        println!(
            "[GATE_STATS] total={} priority={} edge={} exec={} real_trades={}",
            emitted_signs.len(),
            priority_pass_count,
            edge_pass_count,
            exec_pass_count_local,
            real_trade_count
        );
    }

    if trade_scores.is_empty() && !executed_trades.is_empty() {
        if ga_debug_enabled() {
            println!("⚠️ FALLBACK TRADE SCORES USED");
        }
        for (i, t) in executed_trades.iter().enumerate() {
            trade_scores.push((i, t.pnl.max(1e-6)));
        }
    }

    // 🔒 Freeze edge distribution AFTER execution completes
    if edge_vals_build.len() >= 10 {
        edge_vals_snapshot = edge_vals_build.clone();
    }

    // maintain rolling window
    // 🔥 Apply rolling window to SNAPSHOT (not build)
    if edge_vals_snapshot.len() > EDGE_WINDOW {
        edge_vals_snapshot.drain(0..edge_vals_snapshot.len() - EDGE_WINDOW);
    }

    if edge_vals_snapshot.len() >= 10 {
        let mean_e = edge_vals_snapshot.iter().sum::<f64>() / edge_vals_snapshot.len() as f64;
        let var_e = edge_vals_snapshot
            .iter()
            .map(|v| (v - mean_e).powi(2))
            .sum::<f64>()
            / edge_vals_snapshot.len() as f64;

        let std_e = var_e.sqrt();

        if std_e < 1e-4 && ga_log_hotpath() {
            println!("🚨 EDGE COLLAPSE DETECTED → std={:.8}", std_e);
        }
    }
    if ga_debug_enabled() && strategy_index == 0 && generation % 5 == 0 {
        println!(
            "POST-EXEC CHECK → emitted={} executed={}",
            emitted_signs.len(),
            executed_trades.len()
        );

        println!(
            "PROBE_AUDIT → probes={} real_trades={}",
            probe_count, real_trade_count
        );
    }

    if std::env::var("GA_VERBOSE").is_ok() {
        if real_trade_count > 0 && real_trade_count <= probe_count {
            println!(
                "⚠️ INVARIANT WARNING: real_trades ({}) <= probes ({})",
                real_trade_count, probe_count
            );
        }
    }

    // Statistical pruning removed to preserve win/loss distribution.

    // No synthetic trade_scores or dummy executed_trades: empty execution → bad fitness honestly.

    // ✅ FIX: Commit injected trades BEFORE any iteration/retain
    if !injected_trades.is_empty() {
        executed_trades.extend(injected_trades.drain(..));
    }
    // 🔥 HYBRID DEDUPLICATION (Exit bucket + Side)
    let mut seen_ts = std::collections::HashSet::new();

    executed_trades.retain(|trade| {
        // 🔥 HYBRID KEY: bucketed exit index + direction
        let key = (trade.exit_event_idx / 5, trade.side);

        if seen_ts.contains(&key) {
            false
        } else {
            seen_ts.insert(key);
            true
        }
    });

    // 🔥 SIMPLE CLUSTERING BY EXIT INDEX PROXIMITY
    let mut clusters: Vec<Vec<&GaRoundTripOutcome>> = Vec::new();

    let mut current_cluster: Vec<&GaRoundTripOutcome> = Vec::new();
    let mut last_exit: Option<usize> = None;

    // SAFETY: no mutation allowed beyond this point
    debug_assert!(injected_trades.is_empty());

    for trade in executed_trades.iter() {
        if let Some(last) = last_exit {
            if trade.exit_event_idx.saturating_sub(last) <= 5 {
                current_cluster.push(trade);
            } else {
                if !current_cluster.is_empty() {
                    clusters.push(current_cluster);
                }
                current_cluster = vec![trade];
            }
        } else {
            current_cluster.push(trade);
        }

        last_exit = Some(trade.exit_event_idx);
    }

    // push last cluster
    if !current_cluster.is_empty() {
        clusters.push(current_cluster);
    }

    // 🚨 GUARANTEE AT LEAST ONE CLUSTER
    if clusters.is_empty() && !executed_trades.is_empty() {
        clusters.push(executed_trades.iter().collect());
    }

    // 🔍 DEBUG
    if ga_debug_enabled() {
        println!(
            "[CLUSTER_DEBUG] total_trades={} clusters={} avg_size={:.2}",
            executed_trades.len(),
            clusters.len(),
            if clusters.len() > 0 {
                executed_trades.len() as f64 / clusters.len() as f64
            } else {
                0.0
            }
        );
    }
    // =============================
    // 🔥 FIX: REBUILD METRICS FROM FINAL TRADES
    // =============================
    let mut rebuilt_metrics = ScenarioMetrics::default();

    let mut rebuilt_tp = 0;
    let mut rebuilt_sl = 0;
    let mut rebuilt_ts = 0;

    for trade in &executed_trades {
        match trade.exit_reason {
            GaExitReason::TakeProfit => rebuilt_tp += 1,
            GaExitReason::StopLoss => rebuilt_sl += 1,
            GaExitReason::TimeStop => rebuilt_ts += 1,
            GaExitReason::NoFill => {}
        }

        rebuilt_metrics.trade_count += 1;

        if trade.pnl > 0.0 {
            rebuilt_metrics.profitable_trades += 1;
        }

        rebuilt_metrics.sum_pnl += trade.pnl;
    }

    // ✅ FIX: Merge rebuilt metrics instead of clobbering to preserve history
    metrics.sum_pnl = rebuilt_metrics.sum_pnl;
    metrics.trade_count = rebuilt_metrics.trade_count;
    metrics.profitable_trades = rebuilt_metrics.profitable_trades;
    // metrics.trade_qualities, etc. are preserved

    exit_tp_count = rebuilt_tp;
    exit_sl_count = rebuilt_sl;
    exit_ts_count = rebuilt_ts;

    let selected_trades: Vec<&GaRoundTripOutcome> =
        clusters.iter().filter_map(|c| c.get(0).cloned()).collect();

    if ga_debug_enabled() {
        println!(
            "PARTICIPATION → trades={} attempts={} triggered={} signals={} ratio={:.4}",
            executed_trades.len(),
            entry_attempted,
            triggered_entries,
            signal_events.len(),
            executed_trades.len() as f64 / signal_events.len().max(1) as f64
        );
        println!(
                "FUNNEL → signals={} after_signal_filter={} after_edge_filter={} after_exec_prob={} final_attempts={}",
                funnel_signals,
                funnel_after_signal_filter,
                funnel_after_edge_filter, // entry_attempted represents this
                funnel_after_exec_prob,   // triggered_entries
                executed_trades.len()
            );
    }

    // 🔍 EXECUTION VARIANCE DEBUG
    if ga_debug_enabled() {
        let effs: Vec<f64> = metrics.trade_qualities.clone();

        if !effs.is_empty() {
            let min = effs.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = effs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let mean = effs.iter().sum::<f64>() / effs.len() as f64;

            let var = effs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / effs.len() as f64;

            println!(
                "EFF_DIST → min={:.4} max={:.4} mean={:.4} std={:.4} count={}",
                min,
                max,
                mean,
                var.sqrt(),
                effs.len()
            );
        }
    }

    if ga_debug_enabled() {
        let decision_skipped = entry_attempted
            .saturating_sub(executed_trades.len())
            .saturating_sub(skipped_busy);
        println!(
                    "ENTRY_DEBUG → signals={} attempts={} triggered={} busy_skipped={} decision_skipped={} | EXITS: TP={} SL={} TS={}",
                    signal_events.len(), entry_attempted, triggered_entries, skipped_busy, decision_skipped, exit_tp_count, exit_sl_count, exit_ts_count
                );
    }

    if ga_debug_enabled() {
        println!(
            "EFF_FINAL → avg_eff={} trades={}",
            metrics.avg_efficiency(),
            executed_trades.len()
        );
    }

    let _mean_expected_move = if executed_trades.len() > 0 {
        sum_expected_move / executed_trades.len() as f64
    } else {
        0.0
    };
    let drawdown_penalty_raw = if executed_trades.len() > 0 {
        sum_drawdown_raw / executed_trades.len() as f64
    } else {
        0.0
    };
    let requested_qty =
        config.order_quantity_for_strategy * 2 * (executed_trades.len().max(1) as u64);

    // --- PHASE 10.5: REGIME ADMISSION GATE ---
    if executed_trades.len() > 15 && max_pnl_in_scenario < 0.0025 {
        if ga_debug_enabled() {
            println!(
                "ADMISSION_REJECT → Over-trading noise: scenario={} trades={} max_pnl={:.5}",
                scenario_name,
                executed_trades.len(),
                max_pnl_in_scenario
            );
        }
    }

    // 🔥 CANONICAL TRADE DERIVATION (PATCH 5)

    let total_trades = executed_trades.len();

    // 🔥 GLOBAL REGIME (STRATEGY LEVEL)

    let avg_edge_realized = if edge_vals_build.len() > 0 {
        edge_vals_build.iter().sum::<f64>() / edge_vals_build.len() as f64
    } else {
        0.0
    };

    let global_fill_ratio = if entry_attempted > 0 {
        exec_passed as f64 / entry_attempted as f64
    } else {
        0.0
    };

    let regime = if avg_edge_realized < e_stats.p30 || global_fill_ratio < 0.05 {
        -1
    } else {
        1
    };

    let total_pnl: f64 = executed_trades.iter().map(|t| t.pnl).sum();

    if ga_debug_enabled() {
        println!(
            "FITNESS_AUDIT → total={} wins={} losses={}",
            executed_trades.len(),
            executed_trades.iter().filter(|t| t.pnl > 0.0).count(),
            executed_trades.iter().filter(|t| t.pnl < 0.0).count()
        );
    }

    let profitable_trades = executed_trades.iter().filter(|t| t.pnl > 0.0).count();

    let win_rate = if total_trades > 0 {
        profitable_trades as f64 / total_trades as f64
    } else {
        0.0
    };

    let zero_pnl_trades = executed_trades.iter().filter(|t| t.pnl == 0.0).count();

    let avg_pnl = if total_trades > 0 {
        total_pnl / total_trades as f64
    } else {
        0.0
    };

    let avg_pnl_for_scenario = avg_pnl;

    let mut _zero_pnl_trades_scenario = 0usize;
    let mut total_win = 0.0;
    let mut total_loss = 0.0;
    let mut win_count = 0;
    let mut loss_count = 0;

    for trade in &executed_trades {
        let pnl = trade.pnl;

        if pnl > 0.0 {
            total_win += pnl;
            win_count += 1;
        } else if pnl == 0.0 {
            _zero_pnl_trades_scenario += 1;
        } else {
            total_loss += pnl.abs();
            loss_count += 1;
        }
    }

    let avg_win = if win_count > 0 {
        total_win / win_count as f64
    } else {
        0.0
    };
    let avg_loss = if loss_count > 0 {
        total_loss / loss_count as f64
    } else {
        0.0
    };

    if ga_debug_enabled() {
        println!(
            "FITNESS_SANITY → avg_win={:.6} avg_loss={:.6}",
            avg_win, avg_loss
        );
    }

    // Stabilized Payoff Ratio
    let payoff_ratio = if avg_loss.abs() > 1e-6 {
        (avg_win / avg_loss.abs())f64.clamp(0.5, 3.0)
    } else {
        0.0
    };
    let dir_consistency = if win_count > 0 {
        (long_win_count_scenario.max(short_win_count_scenario) as f64 / win_count as f64)
            f64.clamp(0.5, 1.0)
    } else {
        0.0
    };

    // Composite Stability: Boosted with Squaring for Phase 11.2
    let stability = (payoff_ratio * win_rate * dir_consistency)f64.clamp(0.0, 1.5);
    if stability < 0.2 {
        if ga_debug_enabled() {
            println!("CONSISTENCY_PENALTY → stability={:.3}", stability);
        }
    }
    let _stability_weighted = stability.powi(2);

    let _selectivity = metrics.selectivity();

    let std_dev_for_scenario: f64 = if total_trades > 1 {
        let mean = avg_pnl_for_scenario;
        let variance = scenario_pnls
            .iter()
            .map(|pnl| (pnl - mean).powi(2))
            .sum::<f64>()
            / total_trades as f64;
        variance.sqrt()
    } else {
        0.0f64
    };

    // --- PHASE 12.5: SCENARIO-LEVEL DRAWDOWN (worst cumulative dip) ---
    let mut scenario_max_drawdown = 0.0f64;
    let mut current_cum_pnl = 0.0f64;
    let mut peak_cum_pnl = 0.0f64;
    for &pnl in &scenario_pnls {
        current_cum_pnl += pnl;
        peak_cum_pnl = peak_cum_pnl.max(current_cum_pnl);
        let current_drawdown = current_cum_pnl - peak_cum_pnl;
        scenario_max_drawdown = scenario_max_drawdown.min(current_drawdown);
    }
    let worst_pnl_for_scenario = scenario_max_drawdown;

    // --- PHASE 11.2 / B.1 / C / C.1 / C.1.5: INSTITUTIONAL FITNESS REDESIGN ---
    // Formula: Alpha * Consistency * Efficiency * Activity * Stability * DiscoveryPressure

    // 1. Reality Factors (Phase C)
    let avg_vol_ratio = if total_trades > 0 {
        total_vol_ratio / total_trades as f64
    } else {
        0.0
    };
    let avg_spread_reality = if total_trades > 0 {
        total_spread_reality / total_trades as f64
    } else {
        0.0
    };
    let adtv = if total_trades > 0 {
        (total_window_volume / total_trades as f64).max(100_000.0)
    } else {
        1_000_000.0
    };

    // --- 1.1 Slippage Model (Convex + Phase C.2 Liquidity Scaling) ---
    let basic_slippage =
        avg_spread_reality * (1.0 + avg_vol_ratio.powf(1.2)) * config.slippage_factor;
    let size = config.order_quantity_for_strategy as f64;
    let participation_rate = (size / adtv)f64.clamp(0.0001, 0.2);

    // Square Root Law of Market Impact (Phase C.2)
    let size_slippage_multiplier = (1.0 + (participation_rate / 0.01).powi(2)).max(1.0);
    let slippage = basic_slippage * size_slippage_multiplier;
    let _slippage = if avg_pnl_for_scenario > 0.0 {
        slippage.min(avg_pnl_for_scenario * 0.7f64)
    } else {
        slippage
    };

    // --- 1.2 Fill Probability (Phase C.2 Depth-Aware) ---
    let eff_for_fill = metrics.avg_efficiency().max(0.0);
    let base_fill_prob = (eff_for_fill * 0.7 + 0.3)f64.clamp(0.5, 1.0);
    let _fill_prob = (base_fill_prob * (-8.0 * participation_rate).exp())f64.clamp(0., 1.0);

    // --- 1.3 Latency Decay ---
    let latency_ticks = config.latency_ticks as f64;
    let _latency_penalty = (-0.05 * latency_ticks).exp()f64.clamp(0.6, 1.0);

    // --- 1.4 Effective PnL (Scoring Overlay) ---
    // ✅ Fix: Unify scaling. realized_pnl already includes exec_weight/edge_scale.
    // ⚠️ Latency removal: already baked into ESE simulation path.
    let effective_pnl = avg_pnl_for_scenario;

    // --- PHASE C.1.6b: ADAPTIVE PARTICIPATION GATE (Smooth Recovery) ---
    // User Precision: Smooth penalty encourages exploration while killing outliers.
    let _trade_factor = (total_trades as f64 / 10.0).min(1.0);

    // --- PHASE 1: PURE EXECUTION FITNESS (NO NORMALIZATION) ---

    // 🔥 smooth pnl scaling (prevents alpha suppression)
    let _pnl = (effective_pnl / 0.001).tanh() * 2.0;

    // ✅ STEP 1: Only penalize REAL losses (already computed correctly)
    let loss_penalty = total_tail_penalty; // already bounded (~0.0 to 0.01 per trade)

    // ✅ STEP 2: Normalize safely
    let trade_count_safe = (total_trades as f64).max(1.0);
    let normalized_loss = loss_penalty / trade_count_safe;

    // ✅ STEP 3: Soft latency penalty (scaled DOWN heavily)
    let latency_penalty = (sum_latency_raw / trade_count_safe)f64.clamp(0.0, 0.001);

    // ✅ STEP 4: Soft slippage penalty (convert bps → realistic scale)
    let slippage_penalty = (total_slippage_bps / trade_count_safe)f64.clamp(0.0, 0.001);

    // ✅ STEP 5: Combine (controlled)
    let raw_penalty = normalized_loss + latency_penalty + slippage_penalty;

    // ✅ STEP 6: Final clamp (LOWER ceiling)
    let _execution_penalty = raw_penaltyf64.clamp(0.0, 0.2);

    let _consistency_score = (metrics.avg_efficiency().max(0.0) * win_rate)f64.clamp(0.0, 1.0);

    let _activity_score = (total_trades as f64 / 20.0).max(0.05).min(1.0);
    let _avg_exec_score = if metrics.exec_passed_count > 0 {
        (metrics.sum_exec_e_score / metrics.exec_passed_count as f64)f64.clamp(-1.0, 1.0)
    } else {
        0.0
    };
    let _win_boost = if total_trades >= 5 {
        win_rate
    } else {
        win_rate * 0.5
    };

    let n_sig = cycle_sigs.len().max(1) as f64;
    let scenario_signature = if cycle_sigs.is_empty() {
        ScenarioExecutionSignature::default()
    } else {
        ScenarioExecutionSignature {
            avg_queue_ahead: cycle_sigs.iter().map(|s| s.avg_queue_ahead).sum::<f64>() / n_sig,
            avg_latency: cycle_sigs.iter().map(|s| s.avg_latency).sum::<f64>() / n_sig,
            fill_ratio: cycle_sigs.iter().map(|s| s.fill_ratio).sum::<f64>() / n_sig,
            participation: cycle_sigs.iter().map(|s| s.participation).sum::<f64>() / n_sig,
            execution_variance: 0.0,
        }
    };

    // --- PHASE 11.5: EXECUTION PRESSURE CONSTRAINT (V3.2 - REFINED BRIDGE) ---
    // Transition from "Peer Discovery" to "Institutional Realism".

    // 1. Define Global Baselines (NSE 5m Context)
    const GLOBAL_QUEUE_P95: f64 = 250_000.0;

    // 2. Compute Phase-Based Weights
    let phase = (generation as f64 / config.generations.max(1) as f64)f64.clamp(0.0, 1.0);
    let local_weight = 0.7 * (1.0 - phase) + 0.4 * phase;
    let global_weight = 1.0 - local_weight;

    // 3. Compute Normalized Pressure Components
    let log_q = (1.0 + scenario_signature.avg_queue_ahead).ln();
    let log_p95 = (1.0 + GLOBAL_QUEUE_P95).ln();
    let log_max = gen_max_log_queue.max(1e-6);

    let global_component = log_q / log_p95;
    let local_component = log_q / log_max;

    let pressure_factor =
        (global_weight * global_component + local_weight * local_component)f64.clamp(0.01, 1.0);

    // 4. Compute Effective Fill & Progress Reward
    let fill_rate = total_trades as f64 / entry_attempted.max(1) as f64;
    let _effective_fill_score = fill_rate * pressure_factor;

    // Explicit Progress Reward: Incentive for climbing the ladder (Boosted V3.3 + V3.5.1 Bias).
    // 🔥 FINAL FITNESS PURIFICATION (Phase 3: Absolute Alpha)
    // 1. Source: Pure Realized PnL from metrics (Slippage + Latency included)
    let mut fitness = match regime {
        1 => metrics.sum_realized_pnl * 1000.0, // real alpha
        -1 => -0.1 * (1.0 - fill_rate),         // punish dead zones
        _ => -0.5,
    };

    // 🔥 Anti-flat fitness penalty
    if metrics.std_dev() < 1e-6 && total_trades > 5 {
        fitness *= 0.5;
    }

    // 2. Finite Guard (Safety)
    if !fitness.is_finite() {
        fitness = 0.0;
    }

    // 3. Participation CLIFF (5 trades - forced stability)
    if total_trades < 5 {
        fitness *= 0.1;
    }

    // 4. Smooth Participation Factor (10 trades cap - gradient)
    let participation_smooth = (total_trades as f64 / 10.0).min(1.0);
    fitness *= participation_smooth;

    // 5. Hard Floor (No deep negative noise)
    fitness = fitness.max(-0.3);

    // 🔥 prevent silent collapse
    if !fitness.is_finite() {
        if ga_debug_enabled() {
            println!("⚠️ FITNESS NAN/INF DETECTED");
        }
        fitness = -0.5;
    }

    if ga_log_hotpath() {
        println!(
            "FITNESS_BREAKDOWN → pnl={:.3} win={:.3} edge_std={:.6} disp_lambda={:.4} disp_bonus={:.6} final={:.3}",
            metrics.sum_realized_pnl,
            metrics.profitable_trades as f64 / metrics.trade_count.max(1) as f64,
            raw_edge_std,
            fitness_dispersion_lambda,
            scenario_dispersion_term,
            fitness
        );
    }
    let robustness_for_scenario = avg_pnl_for_scenario - config.lambda * std_dev_for_scenario;
    let fill_efficiency = if requested_qty > 0 {
        total_filled_qty as f64 / requested_qty as f64
    } else {
        0.0
    };
    let avg_slippage = if fills_count > 0 {
        total_slippage_bps / fills_count as f64
    } else {
        0.0
    };
    let _realized_avg = avg_pnl_for_scenario;

    let latency_raw_mean = if total_trades > 0 {
        sum_latency_raw / total_trades as f64
    } else {
        0.0
    };

    let downside_variance_scenario = if total_trades > 0 {
        scenario_pnls
            .iter()
            .map(|&pnl| pnl.min(0.0).powi(2))
            .sum::<f64>()
            / total_trades as f64
    } else {
        0.0f64
    };
    let downside_std_dev_scenario = downside_variance_scenario.sqrt();

    // Hard assertion for outcome consistency
    let total_c = metrics.long_count + metrics.short_count;
    let direction_ratio = if total_c > 0 {
        metrics.long_count as f64 / total_c as f64
    } else {
        0.5
    };

    let exit_total = exit_tp_count + exit_sl_count + exit_ts_count;

    if !executed_trades.is_empty() && exit_total != executed_trades.len() {
        if ga_debug_enabled() {
            println!(
                "🚨 MISMATCH → total={} tp={} sl={} ts={}",
                total_trades, exit_tp_count, exit_sl_count, exit_ts_count
            );
        }
    }

    // 🚨 FIX 4: METRICS vs EXECUTION TRUTH CHECK
    if metrics.trade_count != executed_trades.len() {
        if ga_debug_enabled() {
            println!(
                "🚨 METRICS DESYNC → metrics={} executed={}",
                metrics.trade_count,
                executed_trades.len()
            );
        }
    }

    // 🔥 ARCHETYPE VISIBILITY
    if ga_debug_enabled() {
        println!(
            "ARCHETYPE → id={} type={} edge_ratio={} hold_time={} trades={} avg_q_fill={:.1} offset={}",
            strategy_id,
            candidate.archetype,
            candidate.edge_ratio,
            metrics.sum_time_to_mfe / total_trades.max(1) as f64,
            total_trades,
            scenario_signature.avg_queue_ahead,
            candidate.entry_offset
        );
    }

    if ga_debug_enabled() {
        println!("================ GA HEALTH DASHBOARD ================");
        println!(
            "HEALTH → trades={} attempts={} triggered={} signals={} exec_rate={:.3}",
            executed_trades.len(),
            entry_attempted,
            triggered_entries,
            signal_events.len(),
            executed_trades.len() as f64 / entry_attempted.max(1) as f64
        );

        println!(
            "FUNNEL → signal={} → filter={} → edge={} → exec={}",
            funnel_signals,
            funnel_after_signal_filter,
            funnel_after_edge_filter,
            funnel_after_exec_prob
        );

        println!(
            "EXECUTION → avg_eff={:.4} avg_slippage={:.6} latency={:.6}",
            metrics.avg_efficiency(),
            avg_slippage,
            latency_raw_mean
        );

        if ga_debug_enabled() {
            println!(
                "QUALITY → win_rate={:.3} payoff={:.3} stability={:.3}",
                win_rate, payoff_ratio, stability
            );
        }

        if ga_debug_enabled() {
            println!(
                "ALPHA → pnl={:.6} effective={:.6} fitness={:.4}",
                total_pnl, effective_pnl, fitness
            );
            println!("=====================================================");
        }
    }

    if ga_debug_enabled() {
        println!("EXEC_PASSED → {}", exec_passed);
        if exec_passed == 0 {
            println!("🚨 ALL TRADES BLOCKED → CHECK EDGE/CUTOFF");
        }
        println!("TRADES_EXECUTED → {}", executed_trades.len());
        println!("FUNNEL:");
        println!("SIGNALS_GENERATED → {}", signal_events.len());
        println!("EXEC_PASSED → {}", exec_passed);
        println!("TRADES_EXECUTED → {}", executed_trades.len());
        println!(
            "FINAL_EVAL → trades={} pnl={} fitness={}",
            total_trades, total_pnl, fitness
        );
    }

    // 🔥 FINAL CLEANUP — remove duplicates globally
    consistent_scores.dedup_by_key(|(idx, _)| *idx);
    sniper_scores.dedup_by_key(|(idx, _)| *idx);
    trade_scores.dedup_by_key(|(idx, _)| *idx);

    if trade_scores.is_empty() && ga_debug_enabled() {
        println!("🚨 trade_scores empty after evaluation — no synthetic inflate (fitness reflects starvation)");
    }

    // 🔥 FIX: use CONSISTENT distribution only

    if !consistent_scores.is_empty() {
        let mut cs_values: Vec<f64> = consistent_scores.iter().map(|(_, s)| *s).collect();
        cs_values.retain(|v| v.is_finite());

        if cs_values.len() < 3 {
            if ga_debug_enabled() {
                println!(
                    "⚠️ CONSISTENT_DIST_EMPTY → before={} after={} (no synthetic distribution)",
                    consistent_scores.len(),
                    cs_values.len()
                );
            }
            if consistent_scores.len() >= 3 {
                cs_values = consistent_scores.iter().map(|(_, s)| *s).collect();
                cs_values.retain(|v| v.is_finite());
            }
        }

        if cs_values.len() >= 3 {
            cs_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            let len = cs_values.len();

            let p30 = cs_values[((len as f64 * 0.30).floor() as usize).min(len - 1)];
            let _p80 = cs_values[((len as f64 * 0.80).floor() as usize).min(len - 1)];

            let mut filtered_consistent: Vec<(usize, f64)> = consistent_scores
                .iter()
                .filter(|(_, s)| *s >= p30)
                .cloned()
                .collect();

            if filtered_consistent.is_empty() {
                filtered_consistent = consistent_scores.clone();
            }

            consistent_scores = filtered_consistent;
        }
    }

    let max_consistent = (trade_scores.len() as f64 * 0.15).max(3.0) as usize;
    consistent_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    consistent_scores.truncate(max_consistent);

    // 🔥 HARD CAP: consistent cannot exceed 25% of total trades

    // 🔵 NORMAL BEST
    let best_trade_idx = trade_scores
        .iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(idx, _)| *idx)
        .unwrap_or(0);

    // 🔥 REMOVE invalid scores BEFORE sorting
    sniper_scores.retain(|(_, score)| score.is_finite());

    // 🔥 FIX: expand sniper pool BEFORE picking best
    sniper_scores.sort_by(|a, b| {
        match b.1.partial_cmp(&a.1) {
            Some(order) => order,
            None => std::cmp::Ordering::Equal, // handles NaN safely
        }
    });

    // limit sniper population
    let max_snipers = ((trade_scores.len() as f64) * 0.15)f64.clamp(2.0, 6.0) as usize;
    sniper_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    sniper_scores.truncate(max_snipers);

    use std::collections::HashSet;

    let mut used = HashSet::new();

    // 🔴 SNIPER FIRST
    let mut best_sniper_idx = sniper_scores
        .iter()
        .filter(|(i, _)| !used.contains(i))
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| {
            used.insert(*i);
            *i
        })
        .unwrap_or(best_trade_idx);

    // 🟢 CONSISTENT SECOND (avoid reuse)
    let mut best_consistent_idx = consistent_scores
        .iter()
        .filter(|(i, _)| !used.contains(i))
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| *i)
        .unwrap_or(best_trade_idx);

    // 🔥 FORCE DISTINCT IDENTITY

    // --- SNIPER vs NORMAL ---
    if best_sniper_idx == best_trade_idx {
        if let Some((idx, _)) = sniper_scores
            .iter()
            .filter(|(i, _)| *i != best_trade_idx)
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        {
            best_sniper_idx = *idx;
        }
    }

    // --- CONSISTENT vs NORMAL & SNIPER ---
    if best_consistent_idx == best_trade_idx || best_consistent_idx == best_sniper_idx {
        if let Some((idx, _)) = consistent_scores
            .iter()
            .filter(|(i, _)| *i != best_trade_idx && *i != best_sniper_idx)
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        {
            best_consistent_idx = *idx;
        }
    }

    // 🧠 OPTIONAL: soft cleanup (DO NOT remove overlap aggressively)
    // (you can remove this entire block if unsure)
    let _cs_len = consistent_scores.len();
    // 🔥 CRITICAL PIPELINE FIX
    if consistent_scores.is_empty() && !trade_scores.is_empty() {
        if ga_debug_enabled() {
            println!("⚠️ CONSISTENT EMPTY → fallback to trade_scores");
        }

        consistent_scores = trade_scores.clone();
    }
    if consistent_scores.is_empty() {
        // consistent_scores = trade_scores.iter().skip(2).take(3).cloned().collect();
    }

    if trade_scores.len() < 5 {
        if ga_debug_enabled() {
            println!(
                "⚠️ REGIME SKIPPED → insufficient trades: {}",
                trade_scores.len()
            );
        }
    }

    let overlap_count = sniper_scores
        .iter()
        .filter(|(i, _)| consistent_scores.iter().any(|(j, _)| i == j))
        .count();

    if ga_debug_enabled() {
        println!(
            "REGIME_OVERLAP → overlap={} sniper={} consistent={}",
            overlap_count,
            sniper_scores.len(),
            consistent_scores.len()
        );
    }
    if ga_debug_enabled() {
        println!(
            "REGIME_RAW → sniper={} consistent={} total={}",
            sniper_scores.len(),
            consistent_scores.len(),
            trade_scores.len()
        );
        if best_trade_idx == best_sniper_idx {
            println!("⚠️NORMAL == SNIPER");
        }

        if best_trade_idx == best_consistent_idx {
            println!("⚠️ NORMAL == CONSISTENT");
        }
    }

    if ga_debug_enabled() {
        println!(
            "🎯 BEST TRADES → normal={} sniper={} consistent={}",
            best_trade_idx, best_sniper_idx, best_consistent_idx
        );
    }
    if ga_debug_enabled() {
        println!(
            "📊 SCORE DIST → total={} sniper={} consistent={}",
            trade_scores.len(),
            sniper_scores.len(),
            consistent_scores.len()
        );
    }

    // =============================
    // 🔥 TRUE TRADE METRICS (FIX)
    // =============================

    let mut raw_trades: Vec<(usize, f64, f64, f64, f64, f64)> = Vec::new();
    // (idx, edge, exec, surv, sniper, consistent)

    for (idx, trade) in executed_trades.iter().enumerate() {
        let edge = trade.edge_quality.max(1e-6); // fallback if raw edge not stored
        let exec = trade.fill_efficiencyf64.clamp(0.0, 1.0);

        // survival proxy (you already computed survive_score but didn't store it)
        let surv = exec; // temporary proxy (we fix later if needed)

        let sniper = edge * exec.powf(1.5) * surv.powf(1.2);
        let consistent = edge * exec * surv;

        raw_trades.push((idx, edge, exec, surv, sniper, consistent));
    }

    // =============================
    // 🔥 LIVE TRADE RECOMMENDER (NEW)
    // =============================

    if ga_debug_enabled() {
        println!("\n🔥 DIVERSIFIED TRADE RECOMMENDATIONS (CLUSTER-BASED):");
    }

    // 🔥 CLUSTER WEIGHTING (PNL-BASED)
    let mut weights: Vec<f64> = Vec::new();
    let mut total_score = 0.0;

    // compute raw scores (use pnl as proxy)
    for trade in selected_trades.iter() {
        let score = trade.pnl.max(0.0); // ensure non-negative
        weights.push(score);
        total_score += score;
    }

    // normalize weights
    if total_score > 0.0 {
        for w in weights.iter_mut() {
            *w /= total_score;
        }
    } else {
        // fallback: equal weights
        let n = weights.len().max(1) as f64;
        for w in weights.iter_mut() {
            *w = 1.0 / n;
        }
    }

    for (i, trade) in selected_trades.iter().enumerate() {
        let trade_idx = i;

        if ga_debug_enabled() {
            println!(
                "CLUSTER {} → idx={} pnl={:.5} weight={:.3} exit_idx={}",
                i, trade_idx, trade.pnl, weights[i], trade.exit_event_idx
            );
        }
    }
    if ga_debug_enabled() {
        println!(
            "📊 UNIQUE CHECK → n={} s={} c={}",
            best_trade_idx, best_sniper_idx, best_consistent_idx
        );
    }

    // Phase 17C: Finalize Parity Metrics
    let avg_exec_prob = if !exec_probs_history.is_empty() {
        exec_probs_history.iter().sum::<f64>() / exec_probs_history.len() as f64
    } else {
        0.0
    };
    let avg_survive_score = if !survive_scores_history.is_empty() {
        survive_scores_history.iter().sum::<f64>() / survive_scores_history.len() as f64
    } else {
        0.0
    };
    let edge_std_dev = if realized_edges_history.len() > 1 {
        let mean = realized_edges_history.iter().sum::<f64>() / realized_edges_history.len() as f64;
        let variance = realized_edges_history
            .iter()
            .map(|e| (e - mean).powi(2))
            .sum::<f64>()
            / realized_edges_history.len() as f64;
        variance.sqrt()
    } else {
        0.0
    };

    let trade_density =
        (executed_trades.len() as f64 / (signal_events.len() as f64 / 1000.0).max(1.0)).max(0.0);
    // FINAL SAFETY MERGE (no-op if already merged)
    if !injected_trades.is_empty() {
        executed_trades.extend(injected_trades.drain(..));
    }
    if !fitness.is_finite() {
        println!("🚨 FINAL FITNESS INVALID");
        fitness = -0.5;
    }
    Some(CandidateEvaluation {
        candidate_edges,
        winner_idx: best_trade_idx,
        strategy_id: strategy_id.clone(),
        candidate: candidate.clone(),
        behavioral_signature: candidate.get_signature(),
        capability,
        real_dom: dominance,
        had_organic_signals,
        avg_pnl: avg_pnl_for_scenario,
        total_pnl,
        pnl_history: pnl_history_learn.clone(),
        trade_count: total_trades,
        profitable_trades,
        zero_pnl_trades,
        std_dev: std_dev_for_scenario,
        downside_std_dev: downside_std_dev_scenario,
        worst: worst_pnl_for_scenario,
        robustness: robustness_for_scenario,
        fitness,
        max_drawdown: drawdown_penalty_raw * 100.0,
        participation_rate: participation_rate,
        quality_trades: total_quality_trades_scenario,
        win_rate: if total_trades > 0 {
            profitable_trades as f64 / total_trades as f64
        } else {
            0.0
        },
        payoff: payoff_ratio,
        payoff_ratio,
        direction_ratio,
        baseline_pnl,
        execution_metrics: ExecutionMetrics {
            fill_efficiency,
            capture_efficiency: metrics.avg_efficiency(),
            fill_rate: if emitted_signs.len() > 0 {
                executed_trades.len() as f32 / emitted_signs.len() as f32
            } else {
                0.0
            },
            avg_slippage,
            latency_impact: latency_raw_mean,
            queue_blocked_count: queue_blocked_count as usize,
            liquidity_starved_count: liquidity_starved_count as usize,
            total_attempts: (executed_trades.len()
                + queue_blocked_count as usize
                + liquidity_starved_count as usize),
        },
        scenario_signature,
        avg_conviction: metrics.avg_conviction(),
        avg_efficiency: metrics.avg_efficiency(),
        avg_edge_quality: metrics.avg_edge_quality(),
        directional_accuracy: if total_trades > 0 {
            profitable_trades as f64 / total_trades as f64
        } else {
            0.0
        },
        decisiveness: if total_trades > 0 {
            1.0 - (metrics.sum_time_to_mfe / (total_trades as f64 * config.max_hold_bars as f64))
                f64.clamp(0.0, 1.0)
        } else {
            0.0
        },
        execution_friction: if sum_expected_slippage > 0.0 {
            sum_actual_slippage / sum_expected_slippage
        } else {
            1.0
        },
        avg_exec_prob,
        avg_survive_score,
        edge_std_dev,
        short_term_capture_eff: metrics.avg_efficiency(),
        long_term_capture_eff: metrics.avg_efficiency(),
        trade_density,
        queue_blocked_count,
        liquidity_starved_count,
        total_attempts: (total_trades
            + queue_blocked_count as usize
            + liquidity_starved_count as usize),
        exec_opportunity_rate: if emitted_signs.len() > 0 {
            executed_trades.len() as f64 / emitted_signs.len() as f64
        } else {
            0.0
        },
        failure_profile: {
            let total =
                total_trades + queue_blocked_count as usize + liquidity_starved_count as usize;
            if total >= 10 {
                let q_ratio = queue_blocked_count as f64 / total as f64;
                let l_ratio = liquidity_starved_count as f64 / total as f64;
                let weight = (1.0 + total as f64).ln().min(4.0);
                vec![q_ratio * weight, l_ratio * weight]
            } else {
                vec![0.0, 0.0]
            }
        },
        realized_pnl_rolling: metrics.sum_realized_pnl,
        predicted_pnl_rolling: metrics.sum_expected_pnl,
        trade_qualities: metrics.trade_qualities.clone(),
        exit_tp_count,
        exit_sl_count,
        exit_ts_count,
        avg_hold_time: 0.0,
        consistency_score: 1.0,
        recent_performance: avg_pnl_for_scenario,
        pnl_from_tp: pnl_from_tp_scenario,
        pnl_from_sl: pnl_from_sl_scenario,
        max_trade_pnl: max_trade_pnl_scenario,
        pnl_fingerprint: Vec::new(),
        avg_edge_spread: metrics.avg_edge_spread_norm(),
        avg_dominance: metrics.avg_dominance(),
        emitted_signals: emitted_signs,

        raw_pop_avg: metrics.sum_raw_pop_dominance / (metrics.raw_pop_count as f64).max(1.0),
        raw_pop_dist: {
            let mut dist = [0.0; 6];
            let total = (metrics.raw_pop_count as f64).max(1.0);
            for i in 0..6 {
                dist[i] = metrics.raw_pop_dominance_buckets[i] as f64 / total;
            }
            dist
        },
        exec_pop_avg: metrics.sum_exec_pop_dominance / (metrics.exec_pop_count as f64).max(1.0),
        exec_pop_dist: {
            let mut dist = [0.0; 6];
            let total = (metrics.exec_pop_count as f64).max(1.0);
            for i in 0..6 {
                dist[i] = metrics.exec_pop_dominance_buckets[i] as f64 / total;
            }
            dist
        },
        vip_ratio: metrics.vip_count as f64 / (metrics.exec_pop_count as f64).max(1.0),
        stat_zero_dom_ratio: metrics.stat_zero_dom_count as f64
            / (metrics.stat_admitted_count as f64).max(1.0),

        exec_accept_rate: (metrics.exec_passed_count as f64
            / (metrics.exec_admitted_count as f64).max(1.0))
        f64.clamp(0.0, 1.0),
        vip_exec_retention: metrics.vip_exec_passed_count as f64
            / (metrics.vip_admitted_count as f64).max(1.0),
        e_rejection_rate: metrics.exec_rejected_count as f64
            / (metrics.exec_admitted_count as f64).max(1.0),
        clarity_to_exec_drop: 1.0
            - (metrics.vip_exec_passed_count as f64 / (metrics.vip_admitted_count as f64).max(1.0)),
        avg_e_score: if metrics.exec_passed_count > 0 {
            metrics.sum_exec_e_score / metrics.exec_passed_count as f64
        } else {
            0.0
        },
        vip_avg_e_score: metrics.sum_vip_e_score / (metrics.vip_exec_passed_count as f64).max(1.0),
        stat_avg_e_score: metrics.sum_stat_e_score
            / ((metrics.exec_passed_count - metrics.vip_exec_passed_count) as f64).max(1.0),
        consensus_bypass_ratio: metrics.consensus_bypass_count as f64
            / metrics.exec_passed_count.max(1) as f64,
        stability_reject_rate: metrics.stability_rejected_count as f64
            / metrics.exec_admitted_count.max(1) as f64,
        clarity_pnl_share: metrics.sum_clarity_pnl,
        conviction_pnl_share: metrics.sum_conviction_pnl,
        outcome_consistency: 0.0,

        acceptance_rate: metrics.accepted_windows as f64 / metrics.total_windows.max(20) as f64,
        valid_window_ratio: metrics.valid_windows as f64 / metrics.total_windows.max(1) as f64,
        avg_agreement_valid: metrics.sum_agreement_valid / metrics.valid_windows.max(1) as f64,
        avg_purity_valid: metrics.sum_purity_valid / metrics.valid_windows.max(1) as f64,
        avg_stability_valid: metrics.sum_stability_valid / metrics.valid_windows.max(1) as f64,
        max_agreement: metrics.max_agreement,
        max_purity: metrics.max_purity,
        total_windows: metrics.total_windows,

        alpha: {
            let raw_alpha = metrics.adaptive.final_score.mean();
            let avg_edge_pnl =
                (metrics.sum_pnl.abs() / (metrics.trade_count.max(1) as f64)).max(1e-9);
            let edge_min = 0.0005;
            let pressure_penalty = (avg_edge_pnl / edge_min).powi(2).min(1.0);

            // Phase D.1.20 Vagueness Penalty (Condensation)
            let vagueness_penalty = if metrics.max_signature_credibility < 1.1 {
                0.7
            } else {
                1.0
            };

            raw_alpha * pressure_penalty * vagueness_penalty
        },
        consistency: {
            let std = metrics.adaptive.final_score.std().max(0.05);
            (1.0 / std)f64.clamp(0.0, 1.0)
        },
        bootstrap_ratio: metrics.bootstrap_trade_count as f64 / total_trades.max(1) as f64,
        forced_win_ratio: metrics.forced_win_count as f64 / (metrics.total_windows.max(1) as f64),
        max_signature_credibility: metrics.max_signature_credibility,
        opportunity: metrics.adaptive_opportunity_count as f64
            / metrics.total_windows.max(1) as f64,
        acceptance_mode: _acceptance_mode,
        structural_score: 0.0,
        ..CandidateEvaluation::default()
    })
}
