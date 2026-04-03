/**
 * Mirrors `services/api/src/strategy_id_parse.rs` for inspect/compare clients.
 * - Short: strat_{queue}_{base}_{tp}_{sl} or + _seed
 * - Long: strat_{scenario_with_underscores}_{queue}_{base}_{tp}_{sl}
 * - Legacy: last four numeric segments (reversed walk)
 */

function isAllDigits(s) {
  return s != null && s.length > 0 && /^[0-9]+$/.test(s);
}

export function parseStrategyParamsFromId(strategyId) {
  const parts = strategyId.split('_');
  if (parts[0] !== 'strat') {
    throw new Error(`strategy_id must start with strat_: ${strategyId}`);
  }

  if (parts.length >= 5 && parts.length <= 6) {
    const shortOk = [1, 2, 3, 4].every((i) => isAllDigits(parts[i]));
    if (shortOk) {
      return {
        queue_threshold: Number.parseInt(parts[1], 10),
        base_edge: Number.parseInt(parts[2], 10),
        take_profit: Number.parseInt(parts[3], 10),
        stop_loss: Number.parseInt(parts[4], 10),
      };
    }
  }

  if (parts.length >= 6) {
    const n = parts.length;
    const q = parts[n - 4];
    const b = parts[n - 3];
    const tp = parts[n - 2];
    const sl = parts[n - 1];
    if (isAllDigits(q) && isAllDigits(b) && isAllDigits(tp) && isAllDigits(sl)) {
      const scenario = parts.slice(1, n - 4).join('_');
      if (scenario) {
        return {
          queue_threshold: Number.parseInt(q, 10),
          base_edge: Number.parseInt(b, 10),
          take_profit: Number.parseInt(tp, 10),
          stop_loss: Number.parseInt(sl, 10),
        };
      }
    }
  }

  const nums = [];
  for (const part of [...parts].reverse()) {
    const v = Number.parseInt(part, 10);
    if (!Number.isNaN(v)) {
      nums.push(v);
      if (nums.length === 4) break;
    }
  }
  if (nums.length < 4) {
    throw new Error(
      `Could not parse strategy parameters from strategy_id: ${strategyId}`
    );
  }
  return {
    stop_loss: nums[0],
    take_profit: nums[1],
    base_edge: nums[2],
    queue_threshold: nums[3],
  };
}
