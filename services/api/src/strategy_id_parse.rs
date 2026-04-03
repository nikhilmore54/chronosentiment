//! Shared parsing for `strat_*` IDs from `pipeline::deterministic_strategy_id` and `ga` per-scenario eval.

use chronosentiment_core::Strategy;

fn is_all_digits(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

/// Returns `(genome, optional scenario key)` for inspect/compare.
pub fn parse_strategy_id_full(id: &str) -> Result<(Strategy, Option<String>), String> {
    let parts: Vec<&str> = id.split('_').collect();
    if parts.first().copied() != Some("strat") {
        return Err(format!("strategy_id must start with strat_: {}", id));
    }

    if (5..=6).contains(&parts.len()) && (1..5).all(|i| is_all_digits(parts[i])) {
        let q: u64 = parts[1]
            .parse()
            .map_err(|_| format!("invalid queue_threshold in {}", id))?;
        let b: u64 = parts[2]
            .parse()
            .map_err(|_| format!("invalid base_edge in {}", id))?;
        let tp: u64 = parts[3]
            .parse()
            .map_err(|_| format!("invalid take_profit in {}", id))?;
        let sl: u64 = parts[4]
            .parse()
            .map_err(|_| format!("invalid stop_loss in {}", id))?;
        return Ok((
            Strategy {
                queue_threshold: q,
                base_edge: b,
                take_profit: tp,
                stop_loss: sl,
            },
            None,
        ));
    }

    if parts.len() >= 6 {
        let n = parts.len();
        if !(is_all_digits(parts[n - 4])
            && is_all_digits(parts[n - 3])
            && is_all_digits(parts[n - 2])
            && is_all_digits(parts[n - 1]))
        {
            return Err(format!(
                "strategy_id long form must end with _queue_base_takeprofit_stoploss: {}",
                id
            ));
        }
        let q: u64 = parts[n - 4]
            .parse()
            .map_err(|_| format!("invalid queue_threshold in {}", id))?;
        let b: u64 = parts[n - 3]
            .parse()
            .map_err(|_| format!("invalid base_edge in {}", id))?;
        let tp: u64 = parts[n - 2]
            .parse()
            .map_err(|_| format!("invalid take_profit in {}", id))?;
        let sl: u64 = parts[n - 1]
            .parse()
            .map_err(|_| format!("invalid stop_loss in {}", id))?;
        let scenario = parts[1..n - 4].join("_");
        if scenario.is_empty() {
            return Err(format!(
                "strategy_id long form must include scenario segments: {}",
                id
            ));
        }
        return Ok((
            Strategy {
                queue_threshold: q,
                base_edge: b,
                take_profit: tp,
                stop_loss: sl,
            },
            Some(scenario),
        ));
    }

    let mut nums: Vec<u64> = Vec::new();
    for part in id.split('_').rev() {
        if let Ok(v) = part.parse::<u64>() {
            nums.push(v);
            if nums.len() == 4 {
                break;
            }
        }
    }
    if nums.len() < 4 {
        return Err(format!(
            "Could not parse strategy parameters from strategy_id: {}",
            id
        ));
    }
    Ok((
        Strategy {
            stop_loss: nums[0],
            take_profit: nums[1],
            base_edge: nums[2],
            queue_threshold: nums[3],
        },
        None,
    ))
}
