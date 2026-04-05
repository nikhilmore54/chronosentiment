//! Shared parsing for `strat_*` IDs from `pipeline::deterministic_strategy_id` and `ga` per-scenario eval.

use chronosentiment_core::Strategy;

/// Returns `(genome, optional scenario key)` for inspect/compare.
pub fn parse_strategy_id_full(id: &str) -> Result<(Strategy, Option<String>), String> {
    let parts: Vec<&str> = id.split('v').collect();
    
    if parts.len() >= 13 {
        // Format: STRAT_QvEvTPvSLvHPvW_CONVvW_MOMvW_VOLvEXP_CONVvEXP_MOMvEXP_VOLvSELvARCH[vBIASvVFLOORvMFLOORvRRvPART]
        let q: u64 = parts[0].trim_start_matches("STRAT_").parse().map_err(|_| format!("invalid queue_threshold"))?;
        let e: u64 = parts[1].parse().map_err(|_| format!("invalid base_edge"))?;
        let tp: u64 = parts[2].parse().map_err(|_| format!("invalid take_profit"))?;
        let sl: u64 = parts[3].parse().map_err(|_| format!("invalid stop_loss"))?;
        let holding: u64 = parts[4].parse().map_err(|_| format!("invalid holding_period"))?;
        let w_conv: u64 = parts[5].parse().map_err(|_| format!("invalid w_conviction"))?;
        let w_mom: u64 = parts[6].parse().map_err(|_| format!("invalid w_momentum"))?;
        let w_vol: u64 = parts[7].parse().ok().unwrap_or(20);
        let exp_conv: u64 = parts[8].parse().ok().unwrap_or(100);
        let exp_mom: u64 = parts[9].parse().ok().unwrap_or(100);
        let exp_vol: u64 = parts[10].parse().ok().unwrap_or(100);
        let selectivity: u8 = parts[11].parse().ok().unwrap_or(75);
        let archetype: u8 = parts[12].parse().ok().unwrap_or(0);

        // Phase D.1.21 Extended Genes (Optional for backwards compatibility)
        let direction_bias: u8 = parts.get(13).and_then(|p| p.parse().ok()).unwrap_or(50);
        let vol_floor: u8 = parts.get(14).and_then(|p| p.parse().ok()).unwrap_or(20);
        let mom_floor: u8 = parts.get(15).and_then(|p| p.parse().ok()).unwrap_or(20);
        let edge_ratio: u8 = parts.get(16).and_then(|p| p.parse().ok()).unwrap_or(150);
        let participation_threshold: u8 = parts.get(17).and_then(|p| p.parse().ok()).unwrap_or(30);

        return Ok((
            Strategy {
                queue_threshold: q,
                base_edge: e,
                take_profit: tp,
                stop_loss: sl,
                holding_period: holding,
                w_conviction: w_conv,
                w_momentum: w_mom,
                w_volatility: w_vol,
                exp_conviction: exp_conv,
                exp_momentum: exp_mom,
                exp_volatility: exp_vol,
                selectivity,
                archetype,
                direction_bias,
                vol_floor,
                mom_floor,
                edge_ratio,
                participation_threshold,
            },
            None,
        ));
    }

    // Legacy or mismatched format fallback
    let mut nums: Vec<u64> = Vec::new();
    for part in id.split('_').rev() {
        if let Ok(v) = part.parse::<u64>() {
            nums.push(v);
        }
    }
    
    if nums.len() >= 4 {
        return Ok((
            Strategy {
                queue_threshold: nums.get(3).cloned().unwrap_or(100),
                base_edge: nums.get(2).cloned().unwrap_or(2),
                take_profit: nums.get(1).cloned().unwrap_or(20),
                stop_loss: nums.get(0).cloned().unwrap_or(10),
                holding_period: 0,
                w_conviction: 50,
                w_momentum: 30,
                w_volatility: 20,
                exp_conviction: 100,
                exp_momentum: 100,
                exp_volatility: 100,
                selectivity: 75,
                archetype: 0,
                direction_bias: 50,
                vol_floor: 20,
                mom_floor: 20,
                edge_ratio: 150,
                participation_threshold: 30,
            },
            None,
        ));
    }

    Err(format!("Could not parse strategy parameters from strategy_id: {}", id))
}
