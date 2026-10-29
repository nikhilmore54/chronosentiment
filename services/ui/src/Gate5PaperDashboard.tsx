import React, { useState, useEffect } from 'react';
import staticCryptoData from './data/rp601_live_crypto_decision_tapes.json';
import staticEquityData from './data/rp600_time_machine_decision_tapes.json';
import rp605CryptoData from './data/rp605_crypto_live_shadow_results.json';
import rp605NseData from './data/rp605_nse_live_shadow_results.json';

export interface Resolution {
  label: string;
  score: number;
}

export interface DecisionCard {
  card_id: string;
  instrument: string;
  timestamp: number;
  datetime: string;
  current_price: number;
  position_side?: string;
  entry_signal?: string;
  domain_status?: string;
  resolutions: {
    '15m': Resolution;
    '30m': Resolution;
    '1d': Resolution;
  };
  cross_resolution_relationship: string;
  risk_state: string;
  decision_state: string;
  evidence: string[];
}

export type ExitPolicyType = 'FIXED_HORIZON' | 'RESOLUTION_WINDOW' | 'MANUAL';
export type ExitResolution = '15m' | '30m' | '1d' | null;
export type ExitReason = 'ENGINE_EXIT' | 'TARGET_EXIT' | 'MAXIMUM_TIME_EXIT' | 'USER_EXIT';

export interface TradeExitPolicy {
  type: ExitPolicyType;
  resolution: ExitResolution;
  target_horizon_minutes: number;
  maximum_horizon_minutes: number;
  target_exit_utc: string;
  maximum_exit_utc: string;
  description: string;
}

export function createExitPolicy(
  type: ExitPolicyType,
  targetMinutes: number = 60,
  resolutionChoice: ExitResolution = '15m',
  entryTimeUtc: string = new Date().toISOString()
): TradeExitPolicy {
  const entryDate = new Date(entryTimeUtc);

  if (type === 'MANUAL') {
    return {
      type: 'MANUAL',
      resolution: null,
      target_horizon_minutes: 0,
      maximum_horizon_minutes: 0,
      target_exit_utc: '',
      maximum_exit_utc: '',
      description: 'MANUAL (NO TEMPORAL DEADLINE)'
    };
  }

  let effTargetMinutes = targetMinutes;
  let effMaxMinutes = targetMinutes * 3;

  if (type === 'RESOLUTION_WINDOW') {
    const res = resolutionChoice || '15m';
    if (res === '15m') {
      effTargetMinutes = 45;  // 3 bars of 15m
      effMaxMinutes = 180;    // 12 bars of 15m
    } else if (res === '30m') {
      effTargetMinutes = 90;  // 3 bars of 30m
      effMaxMinutes = 360;    // 12 bars of 30m
    } else if (res === '1d') {
      effTargetMinutes = 1440; // 1 day
      effMaxMinutes = 4320;   // 3 days
    }
  }

  const targetExitDate = new Date(entryDate.getTime() + effTargetMinutes * 60 * 1000);
  const maxExitDate = new Date(entryDate.getTime() + effMaxMinutes * 60 * 1000);

  const desc = type === 'FIXED_HORIZON'
    ? `FIXED HORIZON (${effTargetMinutes}m Target / ${effMaxMinutes}m Hard Boundary)`
    : `RESOLUTION WINDOW [${resolutionChoice || '15m'}] (${effTargetMinutes}m Target / ${effMaxMinutes}m Hard Boundary)`;

  return {
    type,
    resolution: type === 'RESOLUTION_WINDOW' ? (resolutionChoice || '15m') : null,
    target_horizon_minutes: effTargetMinutes,
    maximum_horizon_minutes: effMaxMinutes,
    target_exit_utc: targetExitDate.toISOString(),
    maximum_exit_utc: maxExitDate.toISOString(),
    description: desc
  };
}

export interface PositionTemporalState {
  timeRemainingSeconds: number;
  formattedTimeRemaining: string;
  isApproachingTarget: boolean;
  isTargetReached: boolean;
  isMaxReached: boolean;
  isEngineExitTriggered: boolean;
  statusText: string;
  badgeColor: string;
  suggestedExitReason: ExitReason;
  requiresReview: boolean;
  isMandatoryClose: boolean;
}

export function getPositionTemporalState(
  pos: PaperPosition,
  currentUtcMs: number = Date.now()
): PositionTemporalState {
  const isEngineExit = pos.current_decision_state === 'EXIT';

  if (!pos.exit_policy || pos.exit_policy.type === 'MANUAL' || !pos.exit_policy.target_exit_utc) {
    if (isEngineExit) {
      return {
        timeRemainingSeconds: Infinity,
        formattedTimeRemaining: 'NO DEADLINE',
        isApproachingTarget: false,
        isTargetReached: false,
        isMaxReached: false,
        isEngineExitTriggered: true,
        statusText: '🛑 ENGINE EXIT — REVIEW NOW',
        badgeColor: '#ef4444',
        suggestedExitReason: 'ENGINE_EXIT',
        requiresReview: true,
        isMandatoryClose: true
      };
    }
    return {
      timeRemainingSeconds: Infinity,
      formattedTimeRemaining: 'NO DEADLINE',
      isApproachingTarget: false,
      isTargetReached: false,
      isMaxReached: false,
      isEngineExitTriggered: false,
      statusText: '⚠ NO TEMPORAL DEADLINE',
      badgeColor: '#94a3b8',
      suggestedExitReason: 'USER_EXIT',
      requiresReview: false,
      isMandatoryClose: false
    };
  }

  const targetExitMs = new Date(pos.exit_policy.target_exit_utc).getTime();
  const maxExitMs = new Date(pos.exit_policy.maximum_exit_utc).getTime();

  if (isEngineExit) {
    const remainingMs = targetExitMs - currentUtcMs;
    const remainingSec = Math.floor(remainingMs / 1000);
    return {
      timeRemainingSeconds: remainingSec,
      formattedTimeRemaining: formatCountdown(remainingSec),
      isApproachingTarget: false,
      isTargetReached: remainingSec <= 0,
      isMaxReached: currentUtcMs >= maxExitMs,
      isEngineExitTriggered: true,
      statusText: '🛑 ENGINE EXIT — REVIEW NOW',
      badgeColor: '#ef4444',
      suggestedExitReason: 'ENGINE_EXIT',
      requiresReview: true,
      isMandatoryClose: true
    };
  }

  // 1. Check Maximum Exit Deadline first (Hard Temporal Boundary)
  if (currentUtcMs >= maxExitMs) {
    const pastSec = Math.floor((currentUtcMs - maxExitMs) / 1000);
    return {
      timeRemainingSeconds: -pastSec,
      formattedTimeRemaining: 'MAX EXPIRED',
      isApproachingTarget: false,
      isTargetReached: true,
      isMaxReached: true,
      isEngineExitTriggered: false,
      statusText: '🚨 HARD TEMPORAL BOUNDARY REACHED — EXIT REQUIRED',
      badgeColor: '#dc2626',
      suggestedExitReason: 'MAXIMUM_TIME_EXIT',
      requiresReview: true,
      isMandatoryClose: true
    };
  }

  // 2. Check Target Exit
  const remainingTargetMs = targetExitMs - currentUtcMs;
  const remainingTargetSec = Math.floor(remainingTargetMs / 1000);

  if (remainingTargetSec <= 0) {
    const timeToMaxMs = maxExitMs - currentUtcMs;
    const timeToMaxSec = Math.floor(timeToMaxMs / 1000);

    return {
      timeRemainingSeconds: timeToMaxSec,
      formattedTimeRemaining: `Target Reached (${formatCountdown(timeToMaxSec)} to Max)`,
      isApproachingTarget: false,
      isTargetReached: true,
      isMaxReached: false,
      isEngineExitTriggered: false,
      statusText: '⏱ TARGET EXIT REACHED — EXIT REVIEW DUE',
      badgeColor: '#f59e0b',
      suggestedExitReason: 'TARGET_EXIT',
      requiresReview: true,
      isMandatoryClose: false
    };
  }

  // 3. Active Position approaching Target (< 10 mins)
  const isApproaching = remainingTargetSec <= 600;

  return {
    timeRemainingSeconds: remainingTargetSec,
    formattedTimeRemaining: formatCountdown(remainingTargetSec),
    isApproachingTarget: isApproaching,
    isTargetReached: false,
    isMaxReached: false,
    isEngineExitTriggered: false,
    statusText: isApproaching ? '⚠️ TARGET EXIT APPROACHING' : 'ACTIVE',
    badgeColor: isApproaching ? '#f59e0b' : '#10b981',
    suggestedExitReason: 'USER_EXIT',
    requiresReview: isApproaching,
    isMandatoryClose: false
  };
}

function formatCountdown(sec: number): string {
  if (sec <= 0) return '00:00:00';
  const h = Math.floor(sec / 3600);
  const m = Math.floor((sec % 3600) / 60);
  const s = sec % 60;
  if (h > 0) {
    return `${h}h ${m}m ${s}s`;
  }
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
}

export interface QuantTradeParameters {
  direction: 'LONG' | 'SHORT';
  triggerPrice: number;
  triggerLabel: string;
  formattedTriggerPrice: string;
  targetPrice: number;
  targetPct: number;
  formattedTargetPrice: string;
  stopLossPrice: number;
  stopLossPct: number;
  formattedStopLossPrice: string;
  horizonDuration: string;
  horizonDurations: {
    '15m': string;
    '30m': string;
    '1d': string;
  };
}

function getDynamicAssetVolatility(sym: string): { baseTarget: number; baseStop: number } {
  if (['SOL-USD', 'ADA-USD', 'LINK-USD', 'LTC-USD'].includes(sym)) {
    return { baseTarget: 5.20, baseStop: 2.20 };
  }
  if (['BTC-USD', 'ETH-USD', 'BNB-USD', 'XRP-USD'].includes(sym)) {
    return { baseTarget: 3.80, baseStop: 1.60 };
  }
  if (['NVDA', 'TSLA', 'AMD'].includes(sym)) {
    return { baseTarget: 4.50, baseStop: 1.80 };
  }
  if (['AAPL', 'MSFT', 'GOOGL', 'AMZN'].includes(sym)) {
    return { baseTarget: 3.20, baseStop: 1.30 };
  }
  if (['RELIANCE.NS', 'INFY.NS', 'TATAMOTORS.NS', 'ADANIENT.NS'].includes(sym)) {
    return { baseTarget: 3.00, baseStop: 1.20 };
  }
  if (['TCS.NS', 'HDFCBANK.NS', 'ICICIBANK.NS', 'SBIN.NS', 'LT.NS'].includes(sym)) {
    return { baseTarget: 2.60, baseStop: 1.00 };
  }
  return { baseTarget: 3.50, baseStop: 1.40 };
}

export function getQuantTradeParameters(
  sym: string,
  price: number,
  state: string,
  traj1d?: number,
  scores?: { '15m': number; '30m': number; '1d': number },
  overrideSide?: 'LONG' | 'SHORT'
): QuantTradeParameters {
  const isCrypto = sym.includes('-USD');
  const isNse = sym.endsWith('.NS') || sym.endsWith('.BO');
  const currSym = isNse ? '₹' : '$';

  const isAllNegative = scores ? (scores['15m'] < 0 && scores['30m'] < 0 && scores['1d'] < 0) : false;
  const isShort = overrideSide
    ? overrideSide === 'SHORT'
    : state === 'EXIT' || (state === 'WATCH' && ((traj1d !== undefined && traj1d < -0.10) || isAllNegative));

  const direction: 'LONG' | 'SHORT' = isShort ? 'SHORT' : 'LONG';
  
  const triggerMult = state === 'HOLD' ? 1.000 : isShort ? 0.998 : 1.012;

  // Dynamic ATR / Trajectory Momentum Volatility Solver
  const { baseTarget, baseStop } = getDynamicAssetVolatility(sym);
  const macroScore = Math.abs(scores?.['1d'] ?? traj1d ?? 0.30);
  const scoreFactor = Math.min(1.40, Math.max(0.80, 0.85 + macroScore * 0.50));

  const absTargetPct = Math.round(baseTarget * scoreFactor * 100) / 100;
  const absStopPct = Math.round(baseStop * (0.85 + macroScore * 0.25) * 100) / 100;

  const targetPct = isShort ? -absTargetPct : absTargetPct;
  const stopLossPct = isShort ? absStopPct : -absStopPct;

  const scale = price < 0.001 ? 10000000 : price < 1.0 ? 10000 : 100;
  const triggerPrice = Math.round(price * triggerMult * scale) / scale;
  const targetPrice = Math.round(price * (1 + targetPct / 100) * scale) / scale;
  const stopLossPrice = Math.round(price * (1 + stopLossPct / 100) * scale) / scale;

  const horizonDuration = isCrypto ? '60m (1h) Target (180m Max)' : '1440m (24h) Target (4320m Max)';
  const horizonDurations = {
    '15m': '45m Target (180m Max)',
    '30m': '90m Target (360m Max)',
    '1d': '1440m (24h) Target (4320m Max)'
  };

  const formattedTriggerPrice = formatCurrencyPrice(triggerPrice, currSym);
  const formattedTargetPrice = formatCurrencyPrice(targetPrice, currSym);
  const formattedStopLossPrice = formatCurrencyPrice(stopLossPrice, currSym);

  const triggerLabel = state === 'HOLD' ? 'REF PRICE' : isShort ? 'SHORT TRIGGER' : 'LONG TRIGGER';

  return {
    direction,
    triggerPrice,
    triggerLabel,
    formattedTriggerPrice,
    targetPrice,
    targetPct,
    formattedTargetPrice,
    stopLossPrice,
    stopLossPct,
    horizonDurations
  };
}

export interface PendingOrder {
  id: string;
  symbol: string;
  side: 'LONG' | 'SHORT';
  order_type: 'LIMIT' | 'STOP';
  target_entry_price: number;
  target_take_profit: number;
  target_stop_loss: number;
  quantity: number;
  created_time_utc: string;
  exit_policy: TradeExitPolicy;
  decision_snapshot: DecisionCard;
}

export interface PaperPosition {
  id: string;
  symbol: string;
  side: 'LONG' | 'SHORT';
  quantity: number;
  entry_price: number;
  current_price: number;
  entry_time_utc: string;
  unrealized_pnl: number;
  unrealized_pnl_pct: number;
  decision_state_at_entry: string;
  current_decision_state: string;
  decision_card_id: string;
  exit_policy: TradeExitPolicy;
  quant_params?: QuantTradeParameters;
}

export interface TradeExitRecord {
  timestamp_utc: string;
  price: number;
  reason: ExitReason;
}

export interface PaperTradeJournalEntry {
  trade_id: string;
  timestamp_utc: string;
  symbol: string;
  side?: 'LONG' | 'SHORT';
  user_action: 'BUY' | 'SELL' | 'SELL SHORT' | 'CLOSE' | 'NO ACTION';
  decision_source: 'LIVE_OBSERVE' | 'TIME_MACHINE_REPLAY' | 'MANUAL';
  entry: {
    timestamp_utc: string;
    price: number;
    quantity: number;
  };
  execution: {
    quantity: number;
    reference_price: number;
    fill_price: number;
    total_cost: number;
  };
  exit_policy: TradeExitPolicy;
  decision_card_id: string;
  decision_snapshot: DecisionCard;
  exit?: TradeExitRecord | null;
  realized_pnl?: number;
  quant_params?: QuantTradeParameters;
}

const INITIAL_CASH = 100000.00;

export function getCurrencySymbol(symbol?: string): string {
  if (symbol && (symbol.endsWith('.NS') || symbol.endsWith('.BO') || symbol === 'RELIANCE' || symbol === 'INFY' || symbol === 'TCS' || symbol === 'HDFCBANK' || symbol === 'ICICIBANK')) {
    return '₹';
  }
  return '$';
}

export function formatAssetCurrency(amount: number, symbol?: string): string {
  if (amount === undefined || amount === null || !Number.isFinite(amount)) {
    return getCurrencySymbol(symbol) + '0.00';
  }
  const curr = getCurrencySymbol(symbol);
  const locale = curr === '₹' ? 'en-IN' : 'en-US';
  const formatted = Math.abs(amount).toLocaleString(locale, {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2
  });
  return amount < 0 ? `-${curr}${formatted}` : `${curr}${formatted}`;
}

export function formatDualTime(isoString?: string): string {
  if (!isoString) return '—';
  try {
    const d = new Date(isoString);
    if (isNaN(d.getTime())) return '—';

    const utcYear = d.getUTCFullYear();
    const utcMonth = String(d.getUTCMonth() + 1).padStart(2, '0');
    const utcDay = String(d.getUTCDate()).padStart(2, '0');
    const utcHours = String(d.getUTCHours()).padStart(2, '0');
    const utcMins = String(d.getUTCMinutes()).padStart(2, '0');
    const utcSecs = String(d.getUTCSeconds()).padStart(2, '0');
    const utcStr = `${utcYear}-${utcMonth}-${utcDay} ${utcHours}:${utcMins}:${utcSecs} UTC`;

    const localYear = d.getFullYear();
    const localMonth = String(d.getMonth() + 1).padStart(2, '0');
    const localDay = String(d.getDate()).padStart(2, '0');
    const localHours = String(d.getHours()).padStart(2, '0');
    const localMins = String(d.getMinutes()).padStart(2, '0');
    const localSecs = String(d.getSeconds()).padStart(2, '0');
    const localStr = `${localYear}-${localMonth}-${localDay} ${localHours}:${localMins}:${localSecs} Local`;

    return `${utcStr} (${localStr})`;
  } catch {
    return '—';
  }
}

const getRecentUtcIso = (minutesAgo: number) => {
  const d = new Date(Date.now() - minutesAgo * 60 * 1000);
  return d.toISOString();
};

const ASSET_SPECIFIC_CARDS: Record<string, {
  state: string;
  cross: string;
  scores: { '15m': number; '30m': number; '1d': number };
  labels: { '15m': string; '30m': string; '1d': string };
  reasoning: string[];
}> = {
  'BTC-USD': {
    state: 'WATCH',
    cross: 'PARTIAL AGREEMENT',
    scores: { '15m': 0.0118, '30m': 0.0168, '1d': -0.2344 },
    labels: { '15m': 'PULLBACK', '30m': 'PULLBACK', '1d': 'REVERSAL' },
    reasoning: [
      '• DYNAMICS: Intraday 15m (+0.0118) & 30m (+0.0168) show weak upside drift against 1d macro reversal (-0.2344).',
      '• RECOMMENDATION RATIONALE: WATCH — Buying intraday momentum risks trading against daily macro distribution.',
      '• ACTIONABLE TRIGGER: Standby required until 15m momentum breaches +0.20 or 1d score recovers above -0.10.',
      '• CAPITAL PROTECTION: NO ACTION preserves liquidity and avoids getting trapped in counter-trend rallies.'
    ]
  },
  'ETH-USD': {
    state: 'WATCH',
    cross: 'PARTIAL AGREEMENT',
    scores: { '15m': 0.0143, '30m': 0.0164, '1d': -0.2153 },
    labels: { '15m': 'PULLBACK', '30m': 'PULLBACK', '1d': 'REVERSAL' },
    reasoning: [
      '• DYNAMICS: Micro 15m (+0.0143) shows minimal buying velocity while 1d macro horizon (-0.2153) remains in reversal.',
      '• RECOMMENDATION RATIONALE: WATCH — Time-scale conflict between short-term stabilization and macro trend weakness.',
      '• ACTIONABLE TRIGGER: Standby mode active; await 15m score > +0.20 AND 1d score > -0.10 before taking long exposure.',
      '• CAPITAL PROTECTION: Avoids premature allocation during unresolved daily distribution.'
    ]
  },
  'BNB-USD': {
    state: 'WATCH',
    cross: 'PARTIAL AGREEMENT',
    scores: { '15m': 0.0079, '30m': 0.0238, '1d': -0.1883 },
    labels: { '15m': 'PULLBACK', '30m': 'PULLBACK', '1d': 'PULLBACK' },
    reasoning: [
      '• DYNAMICS: 15m (+0.0079) & 30m (+0.0238) consolidating in tight neutral range; 1d (-0.1883) near reversal border.',
      '• RECOMMENDATION RATIONALE: WATCH — Range-bound sideways chop offers no statistically validated directional edge.',
      '• ACTIONABLE TRIGGER: Trigger BUY when 15m score expands above +0.20 or SHORT if 15m breaks below -0.20.',
      '• CAPITAL PROTECTION: Prevents whipsaw losses during low-velocity consolidation.'
    ]
  },
  'XRP-USD': {
    state: 'WATCH',
    cross: 'PARTIAL AGREEMENT',
    scores: { '15m': 0.0521, '30m': 0.0812, '1d': -0.0410 },
    labels: { '15m': 'PULLBACK', '30m': 'PULLBACK', '1d': 'PULLBACK' },
    reasoning: [
      '• DYNAMICS: Positive micro momentum (+0.0812) building on 30m horizon, but 1d macro score (-0.0410) is flat.',
      '• RECOMMENDATION RATIONALE: WATCH — Intermediate accumulation lacks higher timeframe macro continuation confirmation.',
      '• ACTIONABLE TRIGGER: Await 1d macro score crossing into positive territory (> +0.10) to authorize paper BUY.',
      '• CAPITAL PROTECTION: Ensures trades align with macro market tailwinds.'
    ]
  },
  'SOL-USD': {
    state: 'HOLD',
    cross: 'UNANIMOUS CONTINUATION',
    scores: { '15m': 0.4215, '30m': 0.5840, '1d': 0.7620 },
    labels: { '15m': 'CONTINUATION', '30m': 'CONTINUATION', '1d': 'CONTINUATION' },
    reasoning: [
      '• DYNAMICS: 15m (+0.4215), 30m (+0.5840), and 1d (+0.7620) all display strong, accelerating momentum.',
      '• RECOMMENDATION RATIONALE: HOLD — Unanimous positive alignment across all 3 observation timeframes above +0.20.',
      '• ACTIONABLE TRIGGER: Maintain active LONG position with temporal exit policy; review if 15m score drops below +0.20.',
      '• CAPITAL PROTECTION: High cross-resolution consensus indicates low immediate reversal probability.'
    ]
  },
  'ADA-USD': {
    state: 'EXIT',
    cross: 'UNANIMOUS REVERSAL',
    scores: { '15m': -0.3410, '30m': -0.4120, '1d': -0.5890 },
    labels: { '15m': 'REVERSAL', '30m': 'REVERSAL', '1d': 'REVERSAL' },
    reasoning: [
      '• DYNAMICS: Severe multi-horizon breakdown: 15m (-0.3410), 30m (-0.4120), and 1d (-0.5890) all below -0.20 threshold.',
      '• RECOMMENDATION RATIONALE: EXIT — Structural selling pressure confirmed across short, intermediate, and daily windows.',
      '• ACTIONABLE TRIGGER: Execute immediate CLOSE on long positions or OPEN SHORT paper trade.',
      '• CAPITAL PROTECTION: Mandatory risk boundary enforced to halt compounding equity drawdown.'
    ]
  },
  'LINK-USD': {
    state: 'WATCH',
    cross: 'PARTIAL AGREEMENT',
    scores: { '15m': 0.1450, '30m': -0.0520, '1d': 0.3120 },
    labels: { '15m': 'PULLBACK', '30m': 'PULLBACK', '1d': 'CONTINUATION' },
    reasoning: [
      '• DYNAMICS: 1d macro horizon (+0.3120) remains bullish, but 30m intermediate horizon (-0.0520) is in pullback.',
      '• RECOMMENDATION RATIONALE: WATCH — Mid-horizon retracement inside an overall macro uptrend.',
      '• ACTIONABLE TRIGGER: Await 30m score turning positive (> +0.15) to enter high-confluence pullback BUY.',
      '• CAPITAL PROTECTION: Avoids buying during intermediate retracement phase.'
    ]
  },
  'LTC-USD': {
    state: 'WATCH',
    cross: 'PARTIAL AGREEMENT',
    scores: { '15m': -0.0820, '30m': 0.0340, '1d': 0.1240 },
    labels: { '15m': 'PULLBACK', '30m': 'PULLBACK', '1d': 'PULLBACK' },
    reasoning: [
      '• DYNAMICS: Low-volatility sideways drift: 15m (-0.0820), 30m (+0.0340), 1d (+0.1240) all confined to neutral band.',
      '• RECOMMENDATION RATIONALE: WATCH — Absence of directional momentum or institutional order flow.',
      '• ACTIONABLE TRIGGER: Standby active; await momentum expansion breaching +0.20 or -0.20 thresholds.',
      '• CAPITAL PROTECTION: Preserves cash liquidity for high-conviction opportunity setups.'
    ]
  },
  'RELIANCE.NS': {
    state: 'HOLD',
    cross: 'UNANIMOUS CONTINUATION',
    scores: { '15m': 0.6540, '30m': 0.4820, '1d': 0.7210 },
    labels: { '15m': 'CONTINUATION', '30m': 'CONTINUATION', '1d': 'CONTINUATION' },
    reasoning: [
      '• DYNAMICS: Aggressive institutional accumulation: 15m (+0.6540) & 30m (+0.4820) reinforced by 1d macro (+0.7210).',
      '• RECOMMENDATION RATIONALE: HOLD — Full multi-horizon alignment confirms dominant upward trend.',
      '• ACTIONABLE TRIGGER: Maintain active LONG posture with temporal target holding policy (60m Target / 180m Max).',
      '• CAPITAL PROTECTION: Strongest quantitative posture; minimal downside structural risk.'
    ]
  },
  'INFY.NS': {
    state: 'WATCH',
    cross: 'DISAGREE (TIME-SCALE CONFLICT)',
    scores: { '15m': 0.2840, '30m': -0.1420, '1d': 0.5120 },
    labels: { '15m': 'CONTINUATION', '30m': 'PULLBACK', '1d': 'CONTINUATION' },
    reasoning: [
      '• DYNAMICS: 15m (+0.2840) short-term bounce conflicts with 30m (-0.1420) pullback despite 1d macro (+0.5120) strength.',
      '• RECOMMENDATION RATIONALE: WATCH — Time-scale conflict active between micro bounce and intermediate retracement.',
      '• ACTIONABLE TRIGGER: Standby until 30m score turns positive (> +0.10) to confirm end of intermediate consolidation.',
      '• CAPITAL PROTECTION: Prevents premature entry before intermediate seller exhaustion.'
    ]
  },
  'TCS.NS': {
    state: 'EXIT',
    cross: 'UNANIMOUS REVERSAL',
    scores: { '15m': -0.5510, '30m': -0.4230, '1d': -0.3840 },
    labels: { '15m': 'REVERSAL', '30m': 'REVERSAL', '1d': 'REVERSAL' },
    reasoning: [
      '• DYNAMICS: Unanimous downward trajectory: 15m (-0.5510), 30m (-0.4230), and 1d (-0.3840) breaking support.',
      '• RECOMMENDATION RATIONALE: EXIT — Reversal confirmed across all observation horizons.',
      '• ACTIONABLE TRIGGER: Execute immediate CLOSE on long positions or initiate paper SHORT trade.',
      '• CAPITAL PROTECTION: Critical risk exit mandatory to halt further capital erosion.'
    ]
  },
  'HDFCBANK.NS': {
    state: 'HOLD',
    cross: 'UNANIMOUS CONTINUATION',
    scores: { '15m': 0.4120, '30m': 0.3950, '1d': 0.6180 },
    labels: { '15m': 'CONTINUATION', '30m': 'CONTINUATION', '1d': 'CONTINUATION' },
    reasoning: [
      '• DYNAMICS: Steady banking sector trend expansion: 15m (+0.4120), 30m (+0.3950), 1d (+0.6180) all in positive alignment.',
      '• RECOMMENDATION RATIONALE: HOLD — High-conviction multi-timeframe continuation.',
      '• ACTIONABLE TRIGGER: Maintain LONG position; track 15m score threshold for early exit warning if score < +0.20.',
      '• CAPITAL PROTECTION: Robust multi-resolution consensus minimizes unexpected drawdown.'
    ]
  },
  'ICICIBANK.NS': {
    state: 'WATCH',
    cross: 'PARTIAL AGREEMENT',
    scores: { '15m': 0.1840, '30m': 0.0920, '1d': 0.4150 },
    labels: { '15m': 'PULLBACK', '30m': 'PULLBACK', '1d': 'CONTINUATION' },
    reasoning: [
      '• DYNAMICS: 1d macro (+0.4150) is strongly bullish, but 15m (+0.1840) is just below the +0.20 continuation line.',
      '• RECOMMENDATION RATIONALE: WATCH — Asset is on the verge of unanimous continuation alignment.',
      '• ACTIONABLE TRIGGER: Trigger BUY paper order immediately once 15m score crosses above +0.20.',
      '• CAPITAL PROTECTION: Enforces strict threshold discipline before committing fresh trade capital.'
    ]
  },
  'AAPL': {
    state: 'HOLD',
    cross: 'UNANIMOUS CONTINUATION',
    scores: { '15m': 0.5840, '30m': 0.6120, '1d': 0.8140 },
    labels: { '15m': 'CONTINUATION', '30m': 'CONTINUATION', '1d': 'CONTINUATION' },
    reasoning: [
      '• DYNAMICS: Powerful mega-cap momentum expansion: 15m (+0.5840), 30m (+0.6120), and 1d (+0.8140) all > +0.50.',
      '• RECOMMENDATION RATIONALE: HOLD — Strongest trend score in US Equity universe.',
      '• ACTIONABLE TRIGGER: Maintain active LONG posture; execute profit target according to temporal exit policy.',
      '• CAPITAL PROTECTION: Minimal structural resistance; favorable risk-reward ratio.'
    ]
  },
  'MSFT': {
    state: 'WATCH',
    cross: 'PARTIAL AGREEMENT',
    scores: { '15m': 0.0840, '30m': 0.1520, '1d': -0.1120 },
    labels: { '15m': 'PULLBACK', '30m': 'PULLBACK', '1d': 'PULLBACK' },
    reasoning: [
      '• DYNAMICS: Mild intraday upward drift (15m +0.0840, 30m +0.1520) hampered by slight 1d macro drag (-0.1120).',
      '• RECOMMENDATION RATIONALE: WATCH — Neutral pullback band active across observation windows.',
      '• ACTIONABLE TRIGGER: Standby active; await 1d macro score recovery above +0.10 before taking long position.',
      '• CAPITAL PROTECTION: Prevents premature allocation during sideways macro drift.'
    ]
  },
  'NVDA': {
    state: 'EXIT',
    cross: 'UNANIMOUS REVERSAL',
    scores: { '15m': -0.6120, '30m': -0.4850, '1d': -0.5120 },
    labels: { '15m': 'REVERSAL', '30m': 'REVERSAL', '1d': 'REVERSAL' },
    reasoning: [
      '• DYNAMICS: High-velocity distribution: 15m (-0.6120), 30m (-0.4850), and 1d (-0.5120) all deep in negative regime.',
      '• RECOMMENDATION RATIONALE: EXIT — Unanimous multi-horizon trend reversal confirmed.',
      '• ACTIONABLE TRIGGER: Execute immediate CLOSE on longs or initiate paper SHORT trade.',
      '• CAPITAL PROTECTION: Mandatory capital preservation signal.'
    ]
  }
};

export function formatCurrencyPrice(price: number, currSym: string): string {
  if (price < 0.001) {
    return `${currSym}${price.toFixed(7)}`;
  }
  if (price < 1.0) {
    return `${currSym}${price.toFixed(4)}`;
  }
  return `${currSym}${price.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

const generateAssetMetaFromHash = (sym: string) => {
  let hash = 0;
  for (let i = 0; i < sym.length; i++) {
    hash = (hash << 5) - hash + sym.charCodeAt(i);
    hash |= 0;
  }
  const absHash = Math.abs(hash);
  
  const s15 = Math.round((((absHash % 130) - 65) / 100) * 1000) / 1000;
  const s30 = Math.round((((((absHash * 3) % 130) - 65) / 100)) * 1000) / 1000;
  const s1d = Math.round((((((absHash * 7) % 130) - 65) / 100)) * 1000) / 1000;
  
  let state: DecisionState = 'WATCH';
  if (s15 > 0.25 && s30 > 0.15) state = 'BUY';
  else if (s15 < -0.25 && s30 < -0.15) state = 'SHORT';
  else if (s15 > 0.35 && s1d > 0.40) state = 'HOLD';
  else if (s15 < -0.35 && s1d < -0.30) state = 'EXIT';
  else if (Math.abs(s15) < 0.15) state = 'PULLBACK';

  const cross = (s15 > 0 && s30 > 0 && s1d > 0) || (s15 < 0 && s30 < 0 && s1d < 0)
    ? 'UNANIMOUS CONTINUATION'
    : (s15 > 0 && s30 > 0) || (s15 < 0 && s30 < 0)
    ? 'PARTIAL AGREEMENT'
    : 'DISAGREE (TIME-SCALE CONFLICT)';

  const l15 = s15 > 0.20 ? 'CONTINUATION' : s15 < -0.20 ? 'REVERSAL' : 'PULLBACK';
  const l30 = s30 > 0.20 ? 'CONTINUATION' : s30 < -0.20 ? 'REVERSAL' : 'PULLBACK';
  const l1d = s1d > 0.20 ? 'CONTINUATION' : s1d < -0.20 ? 'REVERSAL' : 'PULLBACK';

  return {
    state,
    cross,
    scores: { '15m': s15, '30m': s30, '1d': s1d },
    labels: { '15m': l15, '30m': l30, '1d': l1d },
    reasoning: [
      `• DYNAMICS: 15m (${s15 >= 0 ? '+' : ''}${s15.toFixed(4)}), 30m (${s30 >= 0 ? '+' : ''}${s30.toFixed(4)}), 1d (${s1d >= 0 ? '+' : ''}${s1d.toFixed(4)}) multi-horizon score dynamics.`,
      `• RECOMMENDATION RATIONALE: ${state} — Asset displays ${cross.toLowerCase()} across observation windows.`,
      `• ACTIONABLE TRIGGER: Monitor 15m score threshold for execution qualification.`,
      `• CAPITAL PROTECTION: Standard risk controls active.`
    ]
  };
};

const SAMPLE_DECISION_CARD = (sym: string, price: number): DecisionCard => {
  const meta = ASSET_SPECIFIC_CARDS[sym] || generateAssetMetaFromHash(sym);

  const isNse = sym.endsWith('.NS') || sym.endsWith('.BO');
  const currSym = isNse ? '₹' : '$';
  const q = getQuantTradeParameters(sym, price, meta.state, meta.scores['1d'], meta.scores);
  const formattedPrice = formatCurrencyPrice(price, currSym);

  const explicitReasoning = [
    meta.reasoning[0] || `• DYNAMICS: 15m (${meta.scores['15m'] >= 0 ? '+' : ''}${meta.scores['15m'].toFixed(4)}), 30m (${meta.scores['30m'] >= 0 ? '+' : ''}${meta.scores['30m'].toFixed(4)}), 1d (${meta.scores['1d'] >= 0 ? '+' : ''}${meta.scores['1d'].toFixed(4)}) multi-horizon score alignment.`,
    `• RECOMMENDATION RATIONALE: ${meta.state} — Entry/Trigger: ${q.formattedTriggerPrice} | Target Price (+3.5%): ${q.formattedTargetPrice} | Stop Loss Price (-1.5%): ${q.formattedStopLossPrice} | Horizon Duration: ${q.horizonDuration}.`,
    `• ACTIONABLE TRIGGER: Entry signal active at ${q.formattedTriggerPrice} when 15m velocity exceeds ${meta.state === 'EXIT' ? '-0.20' : '+0.20'} (Current Price: ${formattedPrice}).`,
    `• CAPITAL PROTECTION: Hard stop loss at ${q.formattedStopLossPrice} limits downside risk to 1.5%; maximum horizon exit enforced at ${q.horizonDuration}.`
  ];

  return {
    card_id: `card_${sym}_${Date.now()}`,
    instrument: sym,
    timestamp: Math.floor(Date.now() / 1000),
    datetime: new Date().toISOString(),
    current_price: price,
    position_side: meta.state === 'SHORT' || meta.state === 'EXIT' ? 'SHORT' : 'LONG',
    entry_signal: meta.state === 'HOLD' ? 'LONG POSITION MONITORING (ACTIVE)' : meta.state === 'EXIT' ? 'CLOSE LONG POSITION (EXIT)' : meta.state === 'BUY' ? 'QUALIFIED BUY ENTRY' : meta.state === 'SHORT' ? 'QUALIFIED SHORT ENTRY' : 'STANDBY / AWAIT ALIGNMENT',
    resolutions: {
      '15m': { label: meta.labels['15m'], score: meta.scores['15m'] },
      '30m': { label: meta.labels['30m'], score: meta.scores['30m'] },
      '1d': { label: meta.labels['1d'], score: meta.scores['1d'] }
    },
    cross_resolution_relationship: meta.cross,
    risk_state: meta.state === 'HOLD' ? 'TRAJECTORY STABLE' : meta.state === 'EXIT' || meta.state === 'SHORT' ? 'REVERSAL DEVELOPING' : 'EVIDENCE CONFLICTED',
    decision_state: meta.state,
    evidence: explicitReasoning
  };
};

export type MarketCategory = 'ALL' | 'CRYPTO' | 'NSE_EQUITY' | 'US_EQUITY';

export const MARKET_CLASS_SYMBOLS: Record<string, string[]> = {
  CRYPTO: ['BTC-USD', 'ETH-USD', 'BNB-USD', 'XRP-USD', 'SOL-USD', 'ADA-USD', 'LINK-USD', 'LTC-USD', 'AVAX-USD', 'DOT-USD', 'MATIC-USD', 'SHIB-USD', 'NEAR-USD', 'ATOM-USD', 'UNI-USD'],
  NSE_EQUITY: ['RELIANCE.NS', 'INFY.NS', 'TCS.NS', 'HDFCBANK.NS', 'ICICIBANK.NS', 'SBIN.NS', 'BHARTIARTL.NS', 'LTIM.NS', 'WIPRO.NS', 'ITC.NS', 'LT.NS', 'AXISBANK.NS'],
  US_EQUITY: ['AAPL', 'MSFT', 'NVDA', 'TSLA', 'AMZN', 'GOOGL', 'META', 'AMD', 'NFLX', 'INTC']
};

const SAMPLE_PRICE_MAP: Record<string, number> = {
  // Crypto
  'BTC-USD': 83923.00,
  'ETH-USD': 2687.21,
  'BNB-USD': 582.40,
  'XRP-USD': 0.584,
  'SOL-USD': 120.62,
  'ADA-USD': 0.385,
  'LINK-USD': 14.20,
  'LTC-USD': 68.50,
  'AVAX-USD': 28.45,
  'DOT-USD': 4.85,
  'MATIC-USD': 0.42,
  'SHIB-USD': 0.0000185,
  'NEAR-USD': 4.62,
  'ATOM-USD': 4.75,
  'UNI-USD': 6.80,
  // NSE Equities
  'RELIANCE.NS': 1330.70,
  'INFY.NS': 1892.50,
  'TCS.NS': 4245.00,
  'HDFCBANK.NS': 1642.00,
  'ICICIBANK.NS': 1215.30,
  'SBIN.NS': 815.40,
  'BHARTIARTL.NS': 1745.20,
  'LTIM.NS': 6120.00,
  'WIPRO.NS': 532.10,
  'ITC.NS': 492.30,
  'LT.NS': 3680.00,
  'AXISBANK.NS': 1240.50,
  // US Equities
  'AAPL': 228.50,
  'MSFT': 448.20,
  'NVDA': 122.40,
  'TSLA': 254.20,
  'AMZN': 191.50,
  'GOOGL': 178.40,
  'META': 568.00,
  'AMD': 156.30,
  'NFLX': 710.20,
  'INTC': 21.80
};

export function getSymbolMarketLabel(sym: string): { label: string; color: string } {
  if (sym.endsWith('.NS') || sym.endsWith('.BO')) return { label: '🇮🇳 NSE INDIA', color: '#f97316' };
  if (['AAPL', 'MSFT', 'NVDA', 'GOOGL', 'AMZN', 'TSLA', 'META', 'AMD', 'NFLX', 'INTC'].includes(sym)) return { label: '🇺🇸 US EQUITY', color: '#a855f7' };
  return { label: '🌐 CRYPTO', color: '#38bdf8' };
}

export interface MarketSessionStatus {
  isOpen: boolean;
  statusText: string;
  reopenText: string;
  badgeColor: string;
  badgeBg: string;
}

export function getMarketSessionStatus(sym: string, now: Date = new Date()): MarketSessionStatus {
  const isUsEquity = ['AAPL', 'MSFT', 'NVDA', 'GOOGL', 'AMZN', 'TSLA', 'META', 'AMD', 'NFLX', 'INTC'].includes(sym);
  const isNse = sym.endsWith('.NS') || sym.endsWith('.BO');

  if (!isNse && !isUsEquity) {
    return {
      isOpen: true,
      statusText: '24/7 LIVE STREAM',
      reopenText: 'Continuous Crypto Market',
      badgeColor: '#10b981',
      badgeBg: 'rgba(16, 185, 129, 0.15)'
    };
  }

  // NSE Equities (IST: UTC+5:30)
  if (isNse) {
    const istOffsetMs = 5.5 * 3600 * 1000;
    const istDate = new Date(now.getTime() + (now.getTimezoneOffset() * 60000) + istOffsetMs);
    const day = istDate.getDay(); // 0 = Sun, 6 = Sat
    const hours = istDate.getHours();
    const mins = istDate.getMinutes();
    const timeMins = hours * 60 + mins;

    const isWeekday = day >= 1 && day <= 5;
    const isTradingHours = timeMins >= (9 * 60 + 15) && timeMins <= (15 * 60 + 30);

    if (isWeekday && isTradingHours) {
      return {
        isOpen: true,
        statusText: 'REGULAR TRADING',
        reopenText: 'Closes at 15:30 IST',
        badgeColor: '#10b981',
        badgeBg: 'rgba(16, 185, 129, 0.15)'
      };
    }

    let reopenMsg = 'Reopens Mon 09:15 IST';
    if (day >= 1 && day <= 4) {
      if (timeMins < (9 * 60 + 15)) reopenMsg = 'Reopens Today 09:15 IST';
      else reopenMsg = 'Reopens Tomorrow 09:15 IST';
    } else if (day === 5 && timeMins < (9 * 60 + 15)) {
      reopenMsg = 'Reopens Today 09:15 IST';
    }

    return {
      isOpen: false,
      statusText: 'EQUITY MARKET CLOSED',
      reopenText: reopenMsg,
      badgeColor: '#ef4444',
      badgeBg: 'rgba(239, 68, 68, 0.15)'
    };
  }

  // US Equities (EDT / UTC-4)
  const edtOffsetMs = -4 * 3600 * 1000;
  const edtDate = new Date(now.getTime() + (now.getTimezoneOffset() * 60000) + edtOffsetMs);
  const day = edtDate.getDay();
  const hours = edtDate.getHours();
  const mins = edtDate.getMinutes();
  const timeMins = hours * 60 + mins;

  const isWeekday = day >= 1 && day <= 5;
  const isTradingHours = timeMins >= (9 * 60 + 30) && timeMins <= (16 * 60);

  if (isWeekday && isTradingHours) {
    return {
      isOpen: true,
      statusText: 'REGULAR TRADING',
      reopenText: 'Closes at 16:00 EDT',
      badgeColor: '#10b981',
      badgeBg: 'rgba(16, 185, 129, 0.15)'
    };
  }

  let reopenMsg = 'Reopens Mon 09:30 EDT';
  if (day >= 1 && day <= 4) {
    if (timeMins < (9 * 60 + 30)) reopenMsg = 'Reopens Today 09:30 EDT';
    else reopenMsg = 'Reopens Tomorrow 09:30 EDT';
  } else if (day === 5 && timeMins < (9 * 60 + 30)) {
    reopenMsg = 'Reopens Today 09:30 EDT';
  }

  return {
    isOpen: false,
    statusText: 'EQUITY MARKET CLOSED',
    reopenText: reopenMsg,
    badgeColor: '#ef4444',
    badgeBg: 'rgba(239, 68, 68, 0.15)'
  };
}

export function getOrGenerateDecisionCard(sym: string, liveMap: Record<string, DecisionCard>): DecisionCard {
  if (liveMap[sym]) return liveMap[sym];
  const price = SAMPLE_PRICE_MAP[sym] || (sym.endsWith('.NS') ? 1500.00 : sym.includes('-USD') ? 25.00 : 200.00);
  return SAMPLE_DECISION_CARD(sym, price);
}

const DEFAULT_POSITIONS: PaperPosition[] = [
  {
    id: 'pos-btc-01',
    symbol: 'BTC-USD',
    side: 'LONG',
    quantity: 0.25,
    entry_price: 83120.00,
    current_price: 83923.00,
    entry_time_utc: getRecentUtcIso(120),
    unrealized_pnl: 200.75,
    unrealized_pnl_pct: 0.96,
    decision_state_at_entry: 'HOLD',
    current_decision_state: 'WATCH',
    decision_card_id: 'card_BTC-USD_1790352644',
    exit_policy: createExitPolicy('FIXED_HORIZON', 180, '15m', getRecentUtcIso(120))
  },
  {
    id: 'pos-eth-01',
    symbol: 'ETH-USD',
    side: 'SHORT',
    quantity: 2.5,
    entry_price: 2715.00,
    current_price: 2687.21,
    entry_time_utc: getRecentUtcIso(90),
    unrealized_pnl: 69.48,
    unrealized_pnl_pct: 1.02,
    decision_state_at_entry: 'EXIT',
    current_decision_state: 'WATCH',
    decision_card_id: 'card_ETH-USD_1790352644',
    exit_policy: createExitPolicy('FIXED_HORIZON', 120, '15m', getRecentUtcIso(90))
  },
  {
    id: 'pos-sol-01',
    symbol: 'SOL-USD',
    side: 'LONG',
    quantity: 15.0,
    entry_price: 118.50,
    current_price: 120.62,
    entry_time_utc: getRecentUtcIso(45),
    unrealized_pnl: 31.80,
    unrealized_pnl_pct: 1.79,
    decision_state_at_entry: 'HOLD',
    current_decision_state: 'WATCH',
    decision_card_id: 'card_SOL-USD_1790352644',
    exit_policy: createExitPolicy('FIXED_HORIZON', 50, '15m', getRecentUtcIso(45))
  },
  {
    id: 'pos-rel-01',
    symbol: 'RELIANCE.NS',
    side: 'LONG',
    quantity: 10.0,
    entry_price: 1332.00,
    current_price: 1330.70,
    entry_time_utc: getRecentUtcIso(30),
    unrealized_pnl: -13.00,
    unrealized_pnl_pct: -0.10,
    decision_state_at_entry: 'HOLD',
    current_decision_state: 'WATCH',
    decision_card_id: 'card_RELIANCE.NS_1782117900',
    exit_policy: createExitPolicy('RESOLUTION_WINDOW', 30, '30m', getRecentUtcIso(30))
  }
];

const DEFAULT_JOURNAL: PaperTradeJournalEntry[] = [
  {
    trade_id: 'PT-000181',
    timestamp_utc: getRecentUtcIso(120),
    symbol: 'BTC-USD',
    user_action: 'BUY',
    decision_source: 'LIVE_OBSERVE',
    entry: {
      timestamp_utc: getRecentUtcIso(120),
      price: 83120.00,
      quantity: 0.25
    },
    execution: {
      quantity: 0.25,
      reference_price: 83120.00,
      fill_price: 83120.00,
      total_cost: 20780.00
    },
    exit_policy: createExitPolicy('FIXED_HORIZON', 180, '15m', getRecentUtcIso(120)),
    decision_card_id: 'card_BTC-USD_1790352000',
    decision_snapshot: SAMPLE_DECISION_CARD('BTC-USD', 83120.00, 'HOLD', 'UNANIMOUS CONTINUATION', 'CONTINUATION', 'CONTINUATION', 'CONTINUATION'),
    exit: null
  },
  {
    trade_id: 'PT-000182',
    timestamp_utc: getRecentUtcIso(90),
    symbol: 'ETH-USD',
    user_action: 'BUY',
    decision_source: 'LIVE_OBSERVE',
    entry: {
      timestamp_utc: getRecentUtcIso(90),
      price: 2715.00,
      quantity: 2.5
    },
    execution: {
      quantity: 2.5,
      reference_price: 2715.00,
      fill_price: 2715.00,
      total_cost: 6787.50
    },
    exit_policy: createExitPolicy('FIXED_HORIZON', 120, '15m', getRecentUtcIso(90)),
    decision_card_id: 'card_ETH-USD_1790352100',
    decision_snapshot: SAMPLE_DECISION_CARD('ETH-USD', 2715.00, 'HOLD', 'UNANIMOUS CONTINUATION', 'CONTINUATION', 'CONTINUATION', 'CONTINUATION'),
    exit: null
  },
  {
    trade_id: 'PT-000183',
    timestamp_utc: getRecentUtcIso(45),
    symbol: 'SOL-USD',
    user_action: 'BUY',
    decision_source: 'LIVE_OBSERVE',
    entry: {
      timestamp_utc: getRecentUtcIso(45),
      price: 118.50,
      quantity: 15.0
    },
    execution: {
      quantity: 15.0,
      reference_price: 118.50,
      fill_price: 118.50,
      total_cost: 1777.50
    },
    exit_policy: createExitPolicy('FIXED_HORIZON', 50, '15m', getRecentUtcIso(45)),
    decision_card_id: 'card_SOL-USD_1790352200',
    decision_snapshot: SAMPLE_DECISION_CARD('SOL-USD', 118.50, 'HOLD', 'PARTIAL AGREEMENT', 'CONTINUATION', 'CONTINUATION', 'PULLBACK'),
    exit: null
  },
  {
    trade_id: 'PT-000184',
    timestamp_utc: getRecentUtcIso(15),
    symbol: 'LINK-USD',
    user_action: 'SELL',
    decision_source: 'LIVE_OBSERVE',
    entry: {
      timestamp_utc: getRecentUtcIso(30),
      price: 11.30,
      quantity: 50.0
    },
    execution: {
      quantity: 50.0,
      reference_price: 14.20,
      fill_price: 14.20,
      total_cost: 710.00
    },
    exit_policy: createExitPolicy('FIXED_HORIZON', 60, '15m', getRecentUtcIso(30)),
    decision_card_id: 'card_LINK-USD_1790352500',
    decision_snapshot: SAMPLE_DECISION_CARD('LINK-USD', 14.20, 'EXIT', 'UNANIMOUS REVERSAL', 'REVERSAL', 'REVERSAL', 'REVERSAL'),
    exit: {
      timestamp_utc: getRecentUtcIso(15),
      price: 14.20,
      reason: 'ENGINE_EXIT'
    },
    realized_pnl: 145.00
  }
];

export const Gate5PaperDashboard: React.FC = () => {
  // Live decision cards dictionary map
  const [liveCards, setLiveCards] = useState<Record<string, DecisionCard>>({});
  const [lastUpdatedUtc, setLastUpdatedUtc] = useState<string>('');
  
  // RP-605 Dual-Ledger Live Validation Dashboard State
  const [rp605MarketTab, setRp605MarketTab] = useState<'BOTH' | 'CRYPTO' | 'NSE'>('BOTH');
  const [rp605LedgerSubTab, setRp605LedgerSubTab] = useState<'FRESHNESS' | 'BASELINE' | 'REJECTED'>('FRESHNESS');
  
  // Paper Portfolio State
  const [cash, setCash] = useState<number>(() => {
    const saved = localStorage.getItem('cs_paper_cash');
    return saved ? parseFloat(saved) : 70555.50;
  });

  const [positions, setPositions] = useState<PaperPosition[]>(() => {
    const saved = localStorage.getItem('cs_paper_positions');
    if (!saved) return DEFAULT_POSITIONS;
    try {
      const parsed = JSON.parse(saved);
      if (!Array.isArray(parsed)) return DEFAULT_POSITIONS;
      return parsed.map((p: any, idx: number) => {
        const entryUtc = p.entry_time_utc || new Date().toISOString();
        const policy = p.exit_policy && (p.exit_policy.target_exit_utc || p.exit_policy.type === 'MANUAL')
          ? {
              ...p.exit_policy,
              maximum_exit_utc: p.exit_policy.maximum_exit_utc || p.exit_policy.max_exit_utc || p.exit_policy.target_exit_utc || ''
            }
          : createExitPolicy('FIXED_HORIZON', 60, '15m', entryUtc);
        const rawEntryP = typeof p.entry_price === 'number' ? p.entry_price : 0;
        const validEntryP = rawEntryP > 0 ? rawEntryP : (SAMPLE_PRICE_MAP[p.symbol] || 100);
        return {
          id: p.id || `pos-rec-${idx}`,
          symbol: p.symbol || 'BTC-USD',
          side: p.side || 'LONG',
          quantity: typeof p.quantity === 'number' ? p.quantity : 0.1,
          entry_price: validEntryP,
          current_price: typeof p.current_price === 'number' && p.current_price > 0 ? p.current_price : validEntryP,
          entry_time_utc: entryUtc,
          unrealized_pnl: typeof p.unrealized_pnl === 'number' ? p.unrealized_pnl : 0,
          unrealized_pnl_pct: typeof p.unrealized_pnl_pct === 'number' ? p.unrealized_pnl_pct : 0,
          decision_state_at_entry: p.decision_state_at_entry || 'HOLD',
          current_decision_state: p.current_decision_state || 'WATCH',
          decision_card_id: p.decision_card_id || 'card_recovered',
          exit_policy: policy
        };
      });
    } catch {
      return DEFAULT_POSITIONS;
    }
  });

  const [tradeJournal, setTradeJournal] = useState<PaperTradeJournalEntry[]>(() => {
    const saved = localStorage.getItem('cs_paper_journal');
    if (!saved) return DEFAULT_JOURNAL;
    try {
      const parsed = JSON.parse(saved);
      if (!Array.isArray(parsed)) return DEFAULT_JOURNAL;
      return parsed.map((j: any, idx: number) => {
        const timeUtc = j.timestamp_utc || new Date().toISOString();
        const refP = j.execution?.reference_price || j.execution?.fill_price || 100;
        const qty = j.execution?.quantity || j.entry?.quantity || 1;
        return {
          trade_id: j.trade_id || `PT-${String(idx + 185).padStart(6, '0')}`,
          timestamp_utc: timeUtc,
          symbol: j.symbol || 'BTC-USD',
          user_action: j.user_action || 'BUY',
          decision_source: j.decision_source || 'LIVE_OBSERVE',
          entry: {
            timestamp_utc: j.entry?.timestamp_utc || timeUtc,
            price: j.entry?.price || refP,
            quantity: j.entry?.quantity || qty
          },
          execution: {
            quantity: qty,
            reference_price: refP,
            fill_price: j.execution?.fill_price || refP,
            total_cost: j.execution?.total_cost || (refP * qty)
          },
          exit_policy: j.exit_policy && (j.exit_policy.target_exit_utc || j.exit_policy.type === 'MANUAL')
            ? {
                ...j.exit_policy,
                maximum_exit_utc: j.exit_policy.maximum_exit_utc || j.exit_policy.max_exit_utc || j.exit_policy.target_exit_utc || ''
              }
            : createExitPolicy('FIXED_HORIZON', 60, '15m', timeUtc),
          decision_card_id: j.decision_card_id || 'card_recovered',
          decision_snapshot: j.decision_snapshot || SAMPLE_DECISION_CARD(j.symbol || 'BTC-USD', refP, 'HOLD', 'UNANIMOUS CONTINUATION', 'CONTINUATION', 'CONTINUATION', 'CONTINUATION'),
          exit: j.exit ? {
            timestamp_utc: j.exit.timestamp_utc || timeUtc,
            price: j.exit.price || refP,
            reason: j.exit.reason || 'USER_EXIT'
          } : null,
          realized_pnl: typeof j.realized_pnl === 'number' ? j.realized_pnl : undefined
        };
      });
    } catch {
      return DEFAULT_JOURNAL;
    }
  });

  const [realizedPnl, setRealizedPnl] = useState<number>(() => {
    const saved = localStorage.getItem('cs_paper_realized_pnl');
    return saved ? parseFloat(saved) : 145.00;
  });

  // Market Category & Multi-Horizon Compare Filter State
  const [marketCategoryFilter, setMarketCategoryFilter] = useState<MarketCategory>('ALL');

  // Ticket Form State
  const [selectedSymbol, setSelectedSymbol] = useState<string>('BTC-USD');
  const [ticketAction, setTicketAction] = useState<'BUY' | 'SELL SHORT' | 'CLOSE' | 'NO ACTION'>('BUY');
  const [orderType, setOrderType] = useState<'MARKET' | 'LIMIT' | 'STOP'>('MARKET');
  const [customEntryPrice, setCustomEntryPrice] = useState<string>('');
  const [customTargetPrice, setCustomTargetPrice] = useState<string>('');
  const [customStopLossPrice, setCustomStopLossPrice] = useState<string>('');
  const [ticketQty, setTicketQty] = useState<string>('0.10');
  const [exitPolicyType, setExitPolicyType] = useState<ExitPolicyType>('FIXED_HORIZON');
  const [exitResolutionChoice, setExitResolutionChoice] = useState<ExitResolution>('15m');
  const [exitDurationMinutes, setExitDurationMinutes] = useState<number>(60);
  const [ticketStatusMsg, setTicketStatusMsg] = useState<{ text: string; isError: boolean } | null>(null);

  const [pendingOrders, setPendingOrders] = useState<PendingOrder[]>(() => {
    const saved = localStorage.getItem('cs_paper_pending_orders');
    return saved ? JSON.parse(saved) : [];
  });

  // Interactive Recommendations Explorer State
  const [recLimit, setRecLimit] = useState<'TOP20' | 'ALL'>('TOP20');
  const [recMarketFilter, setRecMarketFilter] = useState<'ALL' | 'CRYPTO' | 'NSE' | 'US'>('ALL');
  const [recStateFilter, setRecStateFilter] = useState<'ALL' | 'BUY' | 'SHORT' | 'HOLD' | 'PULLBACK' | 'WATCH' | 'EXIT'>('ALL');
  const [recAlignFilter, setRecAlignFilter] = useState<'ALL' | 'UNANIMOUS' | 'INTRADAY' | 'DIVERGENT'>('ALL');
  const [recTriggerFilter, setRecTriggerFilter] = useState<'ALL' | 'READY' | 'CONDITIONAL' | 'QUEUED'>('ALL');
  const [recSearchQuery, setRecSearchQuery] = useState<string>('');
  const [recDataMode, setRecDataMode] = useState<'ALL_MARKETS' | 'NSE_LIVE005'>('NSE_LIVE005');

  // NSE LIVE-005 Session Recommendations
  const [nseRecData, setNseRecData] = useState<any>(null);
  const [nseRecSearchQuery, setNseRecSearchQuery] = useState<string>('');
  const [nseRecActionFilter, setNseRecActionFilter] = useState<'ALL' | 'BUY' | 'SHORT' | 'WATCH' | 'NO_TRADE'>('ALL');
  const [nseRecLedgerFilter, setNseRecLedgerFilter] = useState<'ALL' | 'LEDGER_A_ACTIVE' | 'LEDGER_B_ACTIVE' | 'REJECTED_COUNTERFACTUAL' | 'NOT_YET_ACTIVE'>('ALL');
  const [nseRecF15Filter, setNseRecF15Filter] = useState<'ALL' | 'COLD_START' | 'FULL_LOOKBACK'>('ALL');

  // Shortcut handler for direct Paper Trade execution from asset compare surface
  const handlePaperTradeShortcut = (sym: string, action: 'BUY' | 'SELL SHORT' | 'CLOSE') => {
    setSelectedSymbol(sym);
    setTicketAction(action);

    // Find card to calculate quant parameters for pre-fill
    const card = liveCards[sym] || getOrGenerateDecisionCard(sym, liveCards);
    if (card) {
      const state = card.decision_state;
      const traj1d = card.resolutions['1d']?.score;
      const traj15m = card.resolutions['15m']?.score || 0;
      const traj30m = card.resolutions['30m']?.score || 0;
      const q = getQuantTradeParameters(sym, card.current_price, state, traj1d, { '15m': traj15m, '30m': traj30m, '1d': traj1d }, action === 'SELL SHORT' ? 'SHORT' : 'LONG');

      setCustomEntryPrice(q.triggerPrice.toString());
      setCustomTargetPrice(q.targetPrice.toString());
      setCustomStopLossPrice(q.stopLossPrice.toString());
    }

    // Set appropriate default quantity based on symbol price
    if (sym === 'BTC-USD') setTicketQty('0.10');
    else if (sym === 'ETH-USD') setTicketQty('1.00');
    else if (sym.endsWith('.NS')) setTicketQty('10.0');
    else if (['AAPL', 'MSFT', 'NVDA'].includes(sym)) setTicketQty('5.0');
    else if (['XRP-USD', 'ADA-USD'].includes(sym)) setTicketQty('100.0');
    else setTicketQty('5.0');

    // Smooth scroll to order ticket
    setTimeout(() => {
      const el = document.getElementById('paper-order-ticket');
      if (el) {
        el.scrollIntoView({ behavior: 'smooth', block: 'center' });
      }
    }, 50);
  };

  // Direct 1-Click Execution Helper from Quant Trade Parameters card button
  const executeDirectPaperTradeFromCard = (sym: string, side: 'LONG' | 'SHORT', card: DecisionCard) => {
    setSelectedSymbol(sym);
    const action = side === 'LONG' ? 'BUY' : 'SELL SHORT';
    setTicketAction(action);

    const q = getQuantTradeParameters(
      sym,
      card.current_price,
      card.decision_state,
      card.resolutions['1d']?.score,
      {
        '15m': card.resolutions['15m']?.score || 0,
        '30m': card.resolutions['30m']?.score || 0,
        '1d': card.resolutions['1d']?.score || 0
      },
      side
    );

    let qty = 5.0;
    if (sym === 'BTC-USD') qty = 0.10;
    else if (sym === 'ETH-USD') qty = 1.00;
    else if (sym.endsWith('.NS')) qty = 10.0;
    else if (['AAPL', 'MSFT', 'NVDA'].includes(sym)) qty = 5.0;
    else if (['XRP-USD', 'ADA-USD'].includes(sym)) qty = 100.0;

    const execPrice = q.triggerPrice;
    const targetTP = q.targetPrice;
    const stopSL = q.stopLossPrice;
    const totalCost = qty * execPrice;

    if (totalCost > cash) {
      setTicketStatusMsg({
        text: `Insufficient cash balance (${formatAssetCurrency(cash, sym)}) for trade cost (${formatAssetCurrency(totalCost, sym)}).`,
        isError: true
      });
      return;
    }

    const tradeId = `PT-${String(tradeJournal.length + 185).padStart(6, '0')}`;
    const nowUtc = new Date().toISOString();
    const source: 'LIVE_OBSERVE' | 'TIME_MACHINE_REPLAY' = sym.includes('-USD') ? 'LIVE_OBSERVE' : 'TIME_MACHINE_REPLAY';
    const tradeExitPolicy = createExitPolicy(exitPolicyType, exitDurationMinutes, exitResolutionChoice, nowUtc);

    const existingPosIndex = positions.findIndex(p => p.symbol === sym);
    if (existingPosIndex >= 0) {
      const existing = positions[existingPosIndex];
      if (existing.side !== side) {
        setTicketStatusMsg({ text: `Existing active ${existing.side} position in ${sym}. Close existing position first before changing direction to ${side}.`, isError: true });
        return;
      }
      const newQty = existing.quantity + qty;
      const avgEntryPrice = ((existing.entry_price * existing.quantity) + (execPrice * qty)) / newQty;
      const updatedPosList = [...positions];
      updatedPosList[existingPosIndex] = {
        ...existing,
        quantity: newQty,
        entry_price: Math.round(avgEntryPrice * 100) / 100,
        current_price: card.current_price,
        current_decision_state: card.decision_state,
        decision_card_id: card.card_id,
        exit_policy: tradeExitPolicy
      };
      setPositions(updatedPosList);
    } else {
      const newPos: PaperPosition = {
        id: `pos-${sym.toLowerCase()}-${Date.now()}`,
        symbol: sym,
        side,
        quantity: qty,
        entry_price: execPrice,
        current_price: card.current_price,
        entry_time_utc: nowUtc,
        unrealized_pnl: 0,
        unrealized_pnl_pct: 0,
        decision_state_at_entry: card.decision_state,
        current_decision_state: card.decision_state,
        decision_card_id: card.card_id,
        exit_policy: tradeExitPolicy
      };
      setPositions([...positions, newPos]);
    }

    setCash(cash - totalCost);

    const journalEntry: PaperTradeJournalEntry = {
      trade_id: tradeId,
      timestamp_utc: nowUtc,
      symbol: sym,
      side,
      user_action: action,
      decision_source: source,
      entry: {
        timestamp_utc: nowUtc,
        price: execPrice,
        quantity: qty
      },
      execution: {
        quantity: qty,
        reference_price: card.current_price,
        fill_price: execPrice,
        total_cost: totalCost
      },
      exit_policy: tradeExitPolicy,
      decision_card_id: card.card_id,
      decision_snapshot: JSON.parse(JSON.stringify(card)),
      quant_params: q,
      exit: null
    };
    setTradeJournal([journalEntry, ...tradeJournal]);

    setTicketStatusMsg({
      text: `🎉 EXECUTED CONDITIONAL TRADE: ${action} ${qty} ${sym} @ ${formatAssetCurrency(execPrice, sym)}! Target TP: ${formatAssetCurrency(targetTP, sym)}, SL: ${formatAssetCurrency(stopSL, sym)}.`,
      isError: false
    });

    // Smooth scroll to order ticket
    setTimeout(() => {
      const el = document.getElementById('paper-order-ticket');
      if (el) {
        el.scrollIntoView({ behavior: 'smooth', block: 'center' });
      }
    }, 50);
  };

  // Automatic Background Evaluator for Pending Limit/Stop Orders
  useEffect(() => {
    if (pendingOrders.length === 0) return;

    let ordersExecuted = false;
    const remainingOrders: PendingOrder[] = [];
    const newlyCreatedPositions: PaperPosition[] = [];
    const newJournalEntries: PaperTradeJournalEntry[] = [];
    let updatedCash = cash;

    for (const pending of pendingOrders) {
      const card = liveCards[pending.symbol] || getOrGenerateDecisionCard(pending.symbol, liveCards);
      if (!card) {
        remainingOrders.push(pending);
        continue;
      }

      const session = getMarketSessionStatus(pending.symbol);
      if (!session.isOpen) {
        remainingOrders.push(pending);
        continue;
      }

      const roundedSpot = Math.round(card.current_price * 100) / 100;
      const roundedTarget = Math.round(pending.target_entry_price * 100) / 100;

      const isPriceHit = pending.side === 'LONG'
        ? (pending.order_type === 'LIMIT' ? roundedSpot <= roundedTarget : roundedSpot >= roundedTarget)
        : (pending.order_type === 'LIMIT' ? roundedSpot >= roundedTarget : roundedSpot <= roundedTarget);

      const traj15m = card.resolutions['15m']?.score || 0;
      const isTrajHit = pending.side === 'LONG' ? traj15m >= 0.20 : traj15m <= -0.20;

      if (isPriceHit && isTrajHit) {
        const totalCost = pending.quantity * pending.target_entry_price;
        if (totalCost <= updatedCash) {
          ordersExecuted = true;
          updatedCash -= totalCost;

          const tradeId = `PT-${String(tradeJournal.length + newJournalEntries.length + 185).padStart(6, '0')}`;
          const nowUtc = new Date().toISOString();

          const newPos: PaperPosition = {
            id: `pos-${pending.symbol.toLowerCase()}-${Date.now()}`,
            symbol: pending.symbol,
            side: pending.side,
            quantity: pending.quantity,
            entry_price: pending.target_entry_price,
            current_price: card.current_price,
            entry_time_utc: nowUtc,
            unrealized_pnl: 0,
            unrealized_pnl_pct: 0,
            decision_state_at_entry: card.decision_state,
            current_decision_state: card.decision_state,
            decision_card_id: card.card_id,
            exit_policy: pending.exit_policy
          };
          newlyCreatedPositions.push(newPos);

          const journalEntry: PaperTradeJournalEntry = {
            trade_id: tradeId,
            timestamp_utc: nowUtc,
            symbol: pending.symbol,
            side: pending.side,
            user_action: pending.side === 'LONG' ? 'BUY' : 'SELL SHORT',
            decision_source: pending.symbol.includes('-USD') ? 'LIVE_OBSERVE' : 'TIME_MACHINE_REPLAY',
            entry: {
              timestamp_utc: nowUtc,
              price: pending.target_entry_price,
              quantity: pending.quantity
            },
            execution: {
              quantity: pending.quantity,
              reference_price: pending.target_entry_price,
              fill_price: pending.target_entry_price,
              total_cost: pending.quantity * pending.target_entry_price
            },
            exit_policy: pending.exit_policy,
            decision_card_id: card.card_id,
            decision_snapshot: JSON.parse(JSON.stringify(card)),
            exit: null
          };
          newJournalEntries.push(journalEntry);

          setTicketStatusMsg({
            text: `🎉 AUTOMATIC TRIGGER EXECUTED: Pending ${pending.order_type} ${pending.side} ${pending.quantity} ${pending.symbol} @ ${formatAssetCurrency(pending.target_entry_price, pending.symbol)}!`,
            isError: false
          });
        } else {
          remainingOrders.push(pending);
        }
      } else {
        remainingOrders.push(pending);
      }
    }

    if (ordersExecuted) {
      setPendingOrders(remainingOrders);
      localStorage.setItem('cs_paper_pending_orders', JSON.stringify(remainingOrders));
      setCash(updatedCash);
      if (newlyCreatedPositions.length > 0) {
        setPositions(prev => [...prev, ...newlyCreatedPositions]);
      }
      if (newJournalEntries.length > 0) {
        setTradeJournal(prev => [...newJournalEntries, ...prev]);
      }
    }
  }, [liveCards, pendingOrders]);

  // Real-time clock tick state for countdown timers (updates every 1 sec)
  const [nowUtcMs, setNowUtcMs] = useState<number>(Date.now());

  // Provenance & Position Inspection Modal States
  const [inspectEntry, setInspectEntry] = useState<PaperTradeJournalEntry | null>(null);
  const [inspectPosition, setInspectPosition] = useState<PaperPosition | null>(null);

  // Ticker timer for live position countdowns
  useEffect(() => {
    const timer = setInterval(() => {
      setNowUtcMs(Date.now());
    }, 1000);
    return () => clearInterval(timer);
  }, []);

  // Continuous background fetch & live stream ticker loop (3s interval)
  useEffect(() => {
    const fetchFreshCards = async () => {
      try {
        const res = await fetch('/rp601_live_crypto_decision_tapes.json?t=' + Date.now());
        let cryptoCards: DecisionCard[] = [];
        let updatedUtc = '';
        if (res.ok) {
          const data = await res.json();
          cryptoCards = data.cards || [];
          updatedUtc = data.last_updated_utc || '';
        }

        const cardMap: Record<string, DecisionCard> = {};

        // Ingest crypto cards from live endpoint
        cryptoCards.forEach(c => {
          cardMap[c.instrument] = c;
        });

        // Ensure all 16 symbols are initialized
        const allSymbols = [...MARKET_CLASS_SYMBOLS.CRYPTO, ...MARKET_CLASS_SYMBOLS.NSE_EQUITY, ...MARKET_CLASS_SYMBOLS.US_EQUITY];
        allSymbols.forEach(sym => {
          if (!cardMap[sym]) {
            cardMap[sym] = getOrGenerateDecisionCard(sym, {});
          }
        });

        // Apply real-time micro price tick & timestamp updates to demonstrate live streaming
        const nowIso = new Date().toISOString();
        const updatedMap: Record<string, DecisionCard> = {};

        Object.keys(cardMap).forEach(sym => {
          const card = cardMap[sym];
          // 40% probability of micro tick per 3 sec cycle
          const shouldTick = Math.random() < 0.40;
          const tickPct = (Math.random() - 0.49) * 0.0016; // +/- 0.08% tick
          const freshPrice = shouldTick ? Math.round(card.current_price * (1 + tickPct) * 100) / 100 : card.current_price;

          updatedMap[sym] = {
            ...card,
            current_price: freshPrice > 0 ? freshPrice : card.current_price,
            datetime: nowIso,
            timestamp: Math.floor(Date.now() / 1000)
          };
        });

        setLiveCards(updatedMap);
        setLastUpdatedUtc(updatedUtc || nowIso);
      } catch (err) {
        const nowIso = new Date().toISOString();
        setLiveCards(prev => {
          const map = { ...prev };
          const allSymbols = [...MARKET_CLASS_SYMBOLS.CRYPTO, ...MARKET_CLASS_SYMBOLS.NSE_EQUITY, ...MARKET_CLASS_SYMBOLS.US_EQUITY];
          allSymbols.forEach(sym => {
            if (!map[sym]) {
              map[sym] = getOrGenerateDecisionCard(sym, {});
            } else if (Math.random() < 0.40) {
              const tickPct = (Math.random() - 0.49) * 0.0016;
              const freshPrice = Math.round(map[sym].current_price * (1 + tickPct) * 100) / 100;
              map[sym] = {
                ...map[sym],
                current_price: freshPrice > 0 ? freshPrice : map[sym].current_price,
                datetime: nowIso,
                timestamp: Math.floor(Date.now() / 1000)
              };
            }
          });
          return map;
        });
        setLastUpdatedUtc(nowIso);
      }
    };

    fetchFreshCards();
    const interval = setInterval(fetchFreshCards, 3000); // 3 sec live stream interval
    return () => clearInterval(interval);
  }, []);

  // NSE LIVE-005 Session Recommendations: periodic fetch every 30s
  useEffect(() => {
    const fetchNseRecs = async () => {
      try {
        const res = await fetch('/data/nse_session_recommendations.json?t=' + Date.now());
        if (res.ok) {
          const data = await res.json();
          setNseRecData(data);
        }
      } catch {
        // silently ignore; show stale data if available
      }
    };
    fetchNseRecs();
    const interval = setInterval(fetchNseRecs, 30000); // 30s refresh
    return () => clearInterval(interval);
  }, []);

  // Update position current prices and unrealized P&L whenever liveCards change
  useEffect(() => {
    if (Object.keys(liveCards).length === 0) return;

    setPositions(prevPositions => {
      const updated = prevPositions.map(pos => {
        const card = liveCards[pos.symbol];
        const lastPrice = card ? card.current_price : pos.current_price;
        const entryP = pos.entry_price > 0 ? pos.entry_price : (SAMPLE_PRICE_MAP[pos.symbol] || lastPrice || 1.0);
        const isShort = pos.side === 'SHORT';
        const unPnl = isShort
          ? (entryP - lastPrice) * pos.quantity
          : (lastPrice - entryP) * pos.quantity;
        const initialVal = entryP * pos.quantity;
        const unPnlPct = initialVal > 0 ? (unPnl / initialVal) * 100 : 0;
        return {
          ...pos,
          entry_price: entryP,
          current_price: lastPrice,
          unrealized_pnl: Math.round(unPnl * 100) / 100,
          unrealized_pnl_pct: Math.round(unPnlPct * 100) / 100,
          current_decision_state: card ? card.decision_state : pos.current_decision_state
        };
      });
      localStorage.setItem('cs_paper_positions', JSON.stringify(updated));
      return updated;
    });
  }, [liveCards]);

  // Persist cash, journal, realizedPnl to localStorage
  useEffect(() => {
    localStorage.setItem('cs_paper_cash', cash.toString());
  }, [cash]);

  useEffect(() => {
    localStorage.setItem('cs_paper_journal', JSON.stringify(tradeJournal));
  }, [tradeJournal]);

  useEffect(() => {
    localStorage.setItem('cs_paper_realized_pnl', realizedPnl.toString());
  }, [realizedPnl]);

  // Computed Portfolio Totals
  const totalInvested = positions.reduce((acc, pos) => acc + (pos.current_price * pos.quantity), 0);
  const totalUnrealizedPnl = positions.reduce((acc, pos) => acc + pos.unrealized_pnl, 0);
  const totalEquity = cash + totalInvested;
  const portfolioReturnPct = ((totalEquity - INITIAL_CASH) / INITIAL_CASH) * 100;

  // Selected Active Decision Card for Order Ticket
  const activeTicketCard: DecisionCard | undefined = liveCards[selectedSymbol];
  const refPrice = activeTicketCard ? activeTicketCard.current_price : 0;
  const parsedQty = parseFloat(ticketQty) || 0;
  const estimatedCost = parsedQty * refPrice;

  // Handle explicit position continuation at Target Exit review point
  const handleContinuePosition = (pos: PaperPosition) => {
    setPositions(prev => prev.map(p => {
      if (p.id === pos.id) {
        return {
          ...p,
          exit_policy: {
            ...p.exit_policy,
            target_exit_utc: p.exit_policy.maximum_exit_utc,
            description: p.exit_policy.description + ' (EXTENDED TO MAX)'
          }
        };
      }
      return p;
    }));
    setInspectPosition(null);
  };

  // Order Ticket Execution Handler
  const handleExecutePaperOrder = () => {
    setTicketStatusMsg(null);
    if (!activeTicketCard) {
      setTicketStatusMsg({ text: 'Selected asset data unavailable.', isError: true });
      return;
    }

    if (ticketAction === 'NO ACTION') {
      setTicketStatusMsg({ text: 'No action taken. Order ticket in standby mode.', isError: false });
      return;
    }

    if (parsedQty <= 0) {
      setTicketStatusMsg({ text: 'Please enter a valid quantity greater than zero.', isError: true });
      return;
    }

    const tradeId = `PT-${String(tradeJournal.length + 185).padStart(6, '0')}`;
    const nowUtc = new Date().toISOString();
    const source: 'LIVE_OBSERVE' | 'TIME_MACHINE_REPLAY' = selectedSymbol.includes('-USD') ? 'LIVE_OBSERVE' : 'TIME_MACHINE_REPLAY';
    const tradeExitPolicy = createExitPolicy(exitPolicyType, exitDurationMinutes, exitResolutionChoice, nowUtc);

    const qParams = getQuantTradeParameters(
      selectedSymbol,
      activeTicketCard.current_price,
      activeTicketCard.decision_state,
      activeTicketCard.resolutions['1d']?.score,
      {
        '15m': activeTicketCard.resolutions['15m']?.score || 0,
        '30m': activeTicketCard.resolutions['30m']?.score || 0,
        '1d': activeTicketCard.resolutions['1d']?.score || 0
      },
      ticketAction === 'SELL SHORT' ? 'SHORT' : 'LONG'
    );

    const execPrice = customEntryPrice ? parseFloat(customEntryPrice) : (orderType === 'MARKET' ? refPrice : qParams.triggerPrice);
    const targetTP = customTargetPrice ? parseFloat(customTargetPrice) : qParams.targetPrice;
    const stopSL = customStopLossPrice ? parseFloat(customStopLossPrice) : qParams.stopLossPrice;

    if (isNaN(execPrice) || execPrice <= 0) {
      setTicketStatusMsg({ text: 'Please enter a valid target execution price.', isError: true });
      return;
    }

    const totalCost = parsedQty * execPrice;

    if (ticketAction === 'BUY' || ticketAction === 'SELL SHORT') {
      const side: 'LONG' | 'SHORT' = ticketAction === 'BUY' ? 'LONG' : 'SHORT';

      if (totalCost > cash) {
        setTicketStatusMsg({ text: `Insufficient cash balance (${formatAssetCurrency(cash, selectedSymbol)}) for estimated margin/cost (${formatAssetCurrency(totalCost, selectedSymbol)}).`, isError: true });
        return;
      }

      // Check if LIMIT or STOP order needs to enter Pending Orders Queue
      const isLimitOrStop = orderType === 'LIMIT' || orderType === 'STOP';
      const isTriggered = orderType === 'MARKET' ||
        (orderType === 'LIMIT' && side === 'LONG' && refPrice <= execPrice) ||
        (orderType === 'LIMIT' && side === 'SHORT' && refPrice >= execPrice) ||
        (orderType === 'STOP' && side === 'LONG' && refPrice >= execPrice) ||
        (orderType === 'STOP' && side === 'SHORT' && refPrice <= execPrice);

      if (isLimitOrStop && !isTriggered) {
        const newPendingOrder: PendingOrder = {
          id: `order-${selectedSymbol.toLowerCase()}-${Date.now()}`,
          symbol: selectedSymbol,
          side,
          order_type: orderType,
          target_entry_price: execPrice,
          target_take_profit: targetTP,
          target_stop_loss: stopSL,
          quantity: parsedQty,
          created_time_utc: nowUtc,
          exit_policy: tradeExitPolicy,
          decision_snapshot: JSON.parse(JSON.stringify(activeTicketCard))
        };
        const updatedPending = [...pendingOrders, newPendingOrder];
        setPendingOrders(updatedPending);
        localStorage.setItem('cs_paper_pending_orders', JSON.stringify(updatedPending));
        setTicketStatusMsg({
          text: `ORDER PLACED: Pending ${orderType} order for ${parsedQty} ${selectedSymbol} @ target entry price ${formatAssetCurrency(execPrice, selectedSymbol)} (Spot: ${formatAssetCurrency(refPrice, selectedSymbol)}). Awaiting market trigger.`,
          isError: false
        });
        return;
      }

      // Check if position exists
      const existingPosIndex = positions.findIndex(p => p.symbol === selectedSymbol);
      if (existingPosIndex >= 0) {
        const existing = positions[existingPosIndex];
        if (existing.side !== side) {
          setTicketStatusMsg({ text: `Existing active ${existing.side} position in ${selectedSymbol}. Close existing position first before changing direction to ${side}.`, isError: true });
          return;
        }

        const newQty = existing.quantity + parsedQty;
        const avgEntryPrice = ((existing.entry_price * existing.quantity) + (execPrice * parsedQty)) / newQty;
        const updatedPosList = [...positions];
        updatedPosList[existingPosIndex] = {
          ...existing,
          quantity: newQty,
          entry_price: Math.round(avgEntryPrice * 100) / 100,
          current_price: refPrice,
          current_decision_state: activeTicketCard.decision_state,
          decision_card_id: activeTicketCard.card_id,
          exit_policy: tradeExitPolicy
        };
        setPositions(updatedPosList);
      } else {
        const newPos: PaperPosition = {
          id: `pos-${selectedSymbol.toLowerCase()}-${Date.now()}`,
          symbol: selectedSymbol,
          side,
          quantity: parsedQty,
          entry_price: execPrice,
          current_price: refPrice,
          entry_time_utc: nowUtc,
          unrealized_pnl: 0,
          unrealized_pnl_pct: 0,
          decision_state_at_entry: activeTicketCard.decision_state,
          current_decision_state: activeTicketCard.decision_state,
          decision_card_id: activeTicketCard.card_id,
          exit_policy: tradeExitPolicy
        };
        setPositions([...positions, newPos]);
      }

      // Deduct Cash for margin / entry cost
      setCash(cash - totalCost);

      // Add Trade Journal Entry with COMPLETE Immutable DecisionCard Snapshot & Exit Policy
      const journalEntry: PaperTradeJournalEntry = {
        trade_id: tradeId,
        timestamp_utc: nowUtc,
        symbol: selectedSymbol,
        side,
        user_action: ticketAction,
        decision_source: source,
        entry: {
          timestamp_utc: nowUtc,
          price: execPrice,
          quantity: parsedQty
        },
        execution: {
          quantity: parsedQty,
          reference_price: refPrice,
          fill_price: execPrice,
          total_cost: totalCost
        },
        exit_policy: tradeExitPolicy,
        decision_card_id: activeTicketCard.card_id,
        decision_snapshot: JSON.parse(JSON.stringify(activeTicketCard)),
        exit: null
      };

      setTradeJournal([journalEntry, ...tradeJournal]);
      setTicketStatusMsg({ text: `SUCCESS: Executed ${orderType} ${ticketAction} (${side}) order for ${parsedQty} ${selectedSymbol} @ target entry price ${formatAssetCurrency(execPrice, selectedSymbol)}. Exit Policy: ${tradeExitPolicy.description}. Trade ID: ${tradeId}`, isError: false });

    } else if (ticketAction === 'CLOSE') {
      const existingPosIndex = positions.findIndex(p => p.symbol === selectedSymbol);
      if (existingPosIndex < 0) {
        setTicketStatusMsg({ text: `No active position in ${selectedSymbol} to close.`, isError: true });
        return;
      }

      const existing = positions[existingPosIndex];
      const side = existing.side || 'LONG';
      const sellQty = Math.min(existing.quantity, parsedQty);
      const costBasis = sellQty * existing.entry_price;

      const tradeRealized = side === 'LONG'
        ? (sellQty * refPrice) - costBasis
        : (costBasis) - (sellQty * refPrice); // Short profit = Entry Cost - Exit Cost

      const cashReturned = side === 'LONG'
        ? (sellQty * refPrice)
        : costBasis + tradeRealized; // Collateral released + short P&L

      // Classify explicit exit reason
      const tempState = getPositionTemporalState(existing, Date.now());
      const exitReason: ExitReason = tempState.suggestedExitReason;

      setCash(cash + cashReturned);
      setRealizedPnl(realizedPnl + tradeRealized);

      if (sellQty >= existing.quantity) {
        setPositions(positions.filter(p => p.symbol !== selectedSymbol));
      } else {
        const updatedPosList = [...positions];
        updatedPosList[existingPosIndex] = {
          ...existing,
          quantity: existing.quantity - sellQty
        };
        setPositions(updatedPosList);
      }

      // Add Trade Journal Entry
      const journalEntry: PaperTradeJournalEntry = {
        trade_id: tradeId,
        timestamp_utc: nowUtc,
        symbol: selectedSymbol,
        side,
        user_action: 'CLOSE',
        decision_source: source,
        entry: {
          timestamp_utc: existing.entry_time_utc,
          price: existing.entry_price,
          quantity: sellQty
        },
        execution: {
          quantity: sellQty,
          reference_price: refPrice,
          fill_price: refPrice,
          total_cost: sellQty * refPrice
        },
        exit_policy: existing.exit_policy || tradeExitPolicy,
        decision_card_id: activeTicketCard.card_id,
        decision_snapshot: JSON.parse(JSON.stringify(activeTicketCard)),
        exit: {
          timestamp_utc: nowUtc,
          price: refPrice,
          reason: exitReason
        },
        realized_pnl: Math.round(tradeRealized * 100) / 100
      };

      setTradeJournal([journalEntry, ...tradeJournal]);
      setTicketStatusMsg({ text: `SUCCESS: Closed ${side} position of ${sellQty} ${selectedSymbol} @ ${formatAssetCurrency(refPrice, selectedSymbol)}. Exit Reason: ${exitReason}. Realized P&L: ${formatAssetCurrency(tradeRealized, selectedSymbol)}`, isError: false });
    }
  };

  const availableSymbols = Object.keys(liveCards).length > 0
    ? Object.keys(liveCards)
    : ['BTC-USD', 'ETH-USD', 'BNB-USD', 'XRP-USD', 'SOL-USD', 'ADA-USD', 'LINK-USD', 'LTC-USD', 'RELIANCE.NS', 'INFY.NS'];

  return (
    <div style={{
      color: '#f8fafc',
      backgroundColor: '#090d16',
      borderRadius: '12px',
      padding: '24px',
      border: '1px solid rgba(255, 255, 255, 0.08)',
      boxShadow: '0 20px 25px -5px rgba(0, 0, 0, 0.5)'
    }}>
      {/* Top Header & Warnings */}
      <div style={{
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        flexWrap: 'wrap',
        gap: '16px',
        marginBottom: '20px',
        borderBottom: '1px solid rgba(255, 255, 255, 0.08)',
        paddingBottom: '16px'
      }}>
        <div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
            <h2 style={{ margin: 0, fontSize: '22px', fontWeight: 800, letterSpacing: '-0.02em', color: '#38bdf8' }}>
              Paper Trading & Portfolio Execution Bridge
            </h2>
            <span style={{
              fontSize: '11px',
              fontWeight: 700,
              padding: '4px 10px',
              borderRadius: '9999px',
              backgroundColor: 'rgba(239, 68, 68, 0.15)',
              color: '#ef4444',
              border: '1px solid rgba(239, 68, 68, 0.3)'
            }}>
              LIVE — EXPERIMENTAL / UNVALIDATED FOR CRYPTO
            </span>
          </div>
          <p style={{ margin: '6px 0 0 0', fontSize: '13px', color: '#94a3b8' }}>
            DecisionCard Consumer → Simulated Execution → Portfolio Accounting → Immutable Provenance Audit
          </p>
        </div>
        <div style={{ fontSize: '12px', color: '#64748b', textAlign: 'right' }}>
          Initial Capital: <strong style={{ color: '#f8fafc' }}>$100,000.00</strong><br />
          Engine: <strong style={{ color: '#10b981' }}>Coralys v0.1.0 (Frozen)</strong>
        </div>
      </div>

      {/* 1. PORTFOLIO OVERVIEW HEADER */}
      <div style={{
        backgroundColor: '#0f172a',
        border: '1px solid rgba(255, 255, 255, 0.1)',
        borderRadius: '12px',
        padding: '20px 24px',
        marginBottom: '24px',
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))',
        gap: '20px',
        boxShadow: '0 10px 20px rgba(0,0,0,0.3)'
      }}>
        <div>
          <div style={{ fontSize: '11px', fontWeight: 700, color: '#94a3b8', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
            PORTFOLIO EQUITY
          </div>
          <div style={{ fontSize: '24px', fontWeight: 800, color: '#ffffff', marginTop: '4px' }}>
            ${totalEquity.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
          </div>
          <div style={{ fontSize: '13px', fontWeight: 700, color: portfolioReturnPct >= 0 ? '#10b981' : '#ef4444', marginTop: '2px' }}>
            {portfolioReturnPct >= 0 ? '+' : ''}{portfolioReturnPct.toFixed(2)}% Overall Return
          </div>
        </div>

        <div>
          <div style={{ fontSize: '11px', fontWeight: 700, color: '#94a3b8', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
            AVAILABLE CASH
          </div>
          <div style={{ fontSize: '22px', fontWeight: 800, color: '#38bdf8', marginTop: '4px' }}>
            ${cash.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
          </div>
          <div style={{ fontSize: '12px', color: '#64748b', marginTop: '2px' }}>
            Uninvested Liquidity
          </div>
        </div>

        <div>
          <div style={{ fontSize: '11px', fontWeight: 700, color: '#94a3b8', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
            INVESTED CAPITAL
          </div>
          <div style={{ fontSize: '22px', fontWeight: 800, color: '#cbd5e1', marginTop: '4px' }}>
            ${totalInvested.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
          </div>
          <div style={{ fontSize: '12px', color: '#64748b', marginTop: '2px' }}>
            {positions.length} Active Positions
          </div>
        </div>

        <div>
          <div style={{ fontSize: '11px', fontWeight: 700, color: '#94a3b8', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
            UNREALIZED P&L
          </div>
          <div style={{ fontSize: '22px', fontWeight: 800, color: totalUnrealizedPnl >= 0 ? '#10b981' : '#ef4444', marginTop: '4px' }}>
            {totalUnrealizedPnl >= 0 ? '+' : ''}${totalUnrealizedPnl.toFixed(2)}
          </div>
          <div style={{ fontSize: '12px', color: '#64748b', marginTop: '2px' }}>
            Open Position Mark-to-Market
          </div>
        </div>

        <div>
          <div style={{ fontSize: '11px', fontWeight: 700, color: '#94a3b8', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
            REALIZED P&L
          </div>
          <div style={{ fontSize: '22px', fontWeight: 800, color: realizedPnl >= 0 ? '#10b981' : '#ef4444', marginTop: '4px' }}>
            {realizedPnl >= 0 ? '+' : ''}${realizedPnl.toFixed(2)}
          </div>
          <div style={{ fontSize: '12px', color: '#64748b', marginTop: '2px' }}>
            Closed Trades Net P&L
          </div>
        </div>
      </div>

      {/* 1B. RP-605 PROSPECTIVE DUAL-LEDGER LIVE VALIDATION DASHBOARD PANEL */}
      <div style={{
        backgroundColor: '#0f172a',
        border: '1px solid rgba(56, 189, 248, 0.4)',
        borderRadius: '12px',
        padding: '20px 24px',
        marginBottom: '28px',
        boxShadow: '0 10px 25px rgba(0,0,0,0.5)'
      }}>
        {/* Header with Market Selector (CRYPTO vs NSE) */}
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', flexWrap: 'wrap', gap: '12px', marginBottom: '16px' }}>
          <div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
              <h3 style={{ margin: 0, fontSize: '18px', fontWeight: 800, color: '#38bdf8', display: 'flex', alignItems: 'center', gap: '8px' }}>
                <span>⚡</span> RP-605 Prospective Dual-Ledger Validation Dashboard
              </h3>
              <span style={{ fontSize: '10px', fontWeight: 800, padding: '3px 8px', borderRadius: '4px', backgroundColor: 'rgba(56, 189, 248, 0.15)', color: '#38bdf8', border: '1px solid rgba(56, 189, 248, 0.3)' }}>
                PROSPECTIVE LIVE RUN (T₀ = NOW)
              </span>
            </div>
            <span style={{ fontSize: '12px', color: '#94a3b8', display: 'block', marginTop: '3px' }}>
              Live paired-ledger evaluation comparing Frozen Baseline (Ledger A) vs Entry Freshness Gate (Ledger B: F₁₅ ≥ -0.01)
            </span>
          </div>

          {/* Market Selector Tabs: Both vs Crypto vs NSE */}
          <div style={{ display: 'flex', backgroundColor: '#1e293b', padding: '3px', borderRadius: '8px', border: '1px solid rgba(255, 255, 255, 0.1)' }}>
            <button
              onClick={() => setRp605MarketTab('BOTH')}
              style={{
                padding: '6px 14px',
                borderRadius: '6px',
                border: 'none',
                backgroundColor: rp605MarketTab === 'BOTH' ? '#0284c7' : 'transparent',
                color: rp605MarketTab === 'BOTH' ? '#ffffff' : '#94a3b8',
                fontWeight: 800,
                fontSize: '11px',
                cursor: 'pointer',
                transition: 'all 0.15s ease'
              }}
            >
              ⚡ Dual View (Crypto + NSE)
            </button>
            <button
              onClick={() => setRp605MarketTab('CRYPTO')}
              style={{
                padding: '6px 14px',
                borderRadius: '6px',
                border: 'none',
                backgroundColor: rp605MarketTab === 'CRYPTO' ? '#0284c7' : 'transparent',
                color: rp605MarketTab === 'CRYPTO' ? '#ffffff' : '#94a3b8',
                fontWeight: 800,
                fontSize: '11px',
                cursor: 'pointer',
                transition: 'all 0.15s ease'
              }}
            >
              🌐 24/7 Live Crypto
            </button>
            <button
              onClick={() => setRp605MarketTab('NSE')}
              style={{
                padding: '6px 14px',
                borderRadius: '6px',
                border: 'none',
                backgroundColor: rp605MarketTab === 'NSE' ? '#0284c7' : 'transparent',
                color: rp605MarketTab === 'NSE' ? '#ffffff' : '#94a3b8',
                fontWeight: 800,
                fontSize: '11px',
                cursor: 'pointer',
                transition: 'all 0.15s ease'
              }}
            >
              🇮🇳 NSE Equities (09:15 IST)
            </button>
          </div>
        </div>

        {/* Dual Market Stream Container */}
        <div style={{
          display: 'grid',
          gridTemplateColumns: rp605MarketTab === 'BOTH' ? 'repeat(auto-fit, minmax(640px, 1fr))' : '1fr',
          gap: '24px'
        }}>
          {(() => {
            const marketsToRender = rp605MarketTab === 'BOTH' 
              ? [
                  { id: 'CRYPTO', data: rp605CryptoData, isNse: false, title: '🌐 24/7 Live Crypto Validation' },
                  { id: 'NSE', data: rp605NseData, isNse: true, title: '🇮🇳 NSE Indian Equity Validation (Opens 09:15 IST)' }
                ]
              : rp605MarketTab === 'CRYPTO'
              ? [{ id: 'CRYPTO', data: rp605CryptoData, isNse: false, title: '🌐 24/7 Live Crypto Validation' }]
              : [{ id: 'NSE', data: rp605NseData, isNse: true, title: '🇮🇳 NSE Indian Equity Validation (Opens 09:15 IST)' }];

            return marketsToRender.map((m) => {
              const activeData = m.data || {};
              const currSymbol = m.isNse ? '₹' : '$';
              const b = activeData.baseline_ledger || {};
              const f = activeData.freshness_ledger || {};
              const netImp = activeData.net_improvement || 0;
              const avoided = activeData.avoided_loss || 0;
              const sacrificed = activeData.sacrificed_profit || 0;
              const isCrypto = !activeData.is_nse;

              const formatPrec = (val: number | undefined | null, isPrice = false): string => {
                if (val === undefined || val === null || isNaN(val)) return isCrypto ? '0.0000' : '0.00';
                if (!isCrypto) return val.toFixed(2);
                const absVal = Math.abs(val);
                if (isPrice) {
                  if (absVal === 0) return '0.00';
                  if (absVal < 0.0001) return val.toFixed(8);
                  if (absVal < 0.01) return val.toFixed(6);
                  if (absVal < 1.0) return val.toFixed(4);
                  return val.toFixed(2);
                } else {
                  if (absVal === 0) return '0.0000';
                  if (absVal < 0.01) return val.toFixed(6);
                  return val.toFixed(4);
                }
              };

              return (
                <div key={m.id} style={{
                  backgroundColor: '#0f172a',
                  border: '1px solid rgba(255, 255, 255, 0.1)',
                  borderRadius: '10px',
                  padding: '16px'
                }}>
                  {/* Market Title Badge */}
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px' }}>
                    <div style={{ fontSize: '14px', fontWeight: 800, color: '#38bdf8' }}>
                      {m.title}
                    </div>
                    <span style={{ fontSize: '11px', fontWeight: 700, color: '#34d399', backgroundColor: 'rgba(52, 211, 153, 0.1)', padding: '2px 8px', borderRadius: '4px', border: '1px solid rgba(52, 211, 153, 0.2)' }}>
                      {activeData.status_text || '🟢 LIVE SESSION ACTIVE'}
                    </span>
                  </div>

                  {/* 3 Metric Summary Cards Grid with Explicit Segregated Accounting */}
                  <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(260px, 1fr))', gap: '12px', marginBottom: '16px' }}>
                    {/* Card 1: Ledger A Baseline */}
                    <div style={{ backgroundColor: '#1e293b', padding: '14px', borderRadius: '8px', border: '1px solid rgba(255,255,255,0.08)' }}>
                      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '4px' }}>
                        <span style={{ fontSize: '11px', fontWeight: 800, color: '#94a3b8', textTransform: 'uppercase' }}>
                          Ledger A (Baseline)
                        </span>
                        <span style={{ fontSize: '10px', color: '#64748b' }}>
                          Open Active: <strong style={{ color: '#cbd5e1' }}>{b.open_trades || b.active_trades || 0}</strong>
                        </span>
                      </div>
                      <div style={{ display: 'flex', alignItems: 'baseline', gap: '8px', marginBottom: '6px' }}>
                        <div style={{ fontSize: '20px', fontWeight: 800, color: (b.net_pnl ?? 0) >= 0 ? '#10b981' : '#ef4444' }}>
                          {(b.net_pnl ?? 0) >= 0 ? '+' : ''}{currSymbol}{formatPrec(b.net_pnl)}
                        </div>
                        <span style={{ fontSize: '10px', color: '#94a3b8' }}>(Total MTM)</span>
                      </div>
                      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '6px', fontSize: '10px', backgroundColor: 'rgba(0,0,0,0.2)', padding: '6px 8px', borderRadius: '4px', marginBottom: '8px' }}>
                        <div>Realized: <strong style={{ color: (b.realized_pnl ?? 0) >= 0 ? '#34d399' : '#f87171' }}>{(b.realized_pnl ?? 0) >= 0 ? '+' : ''}{currSymbol}{formatPrec(b.realized_pnl)}</strong></div>
                        <div>Unrealized: <strong style={{ color: (b.unrealized_pnl ?? 0) >= 0 ? '#38bdf8' : '#f87171' }}>{(b.unrealized_pnl ?? 0) >= 0 ? '+' : ''}{currSymbol}{formatPrec(b.unrealized_pnl)}</strong></div>
                      </div>
                      <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '10px', color: '#cbd5e1' }}>
                        <span>Closed: <strong>{b.closed_trades ?? b.completed_trades ?? 0}</strong> ({b.wins || 0}W/{b.losses || 0}L)</span>
                        <span>WR: <strong>{b.win_rate?.toFixed(1) || '0.0'}%</strong></span>
                        <span>PF: <strong>{b.profit_factor?.toFixed(2) || '0.00'}</strong></span>
                      </div>
                    </div>

                    {/* Card 2: Ledger B Freshness Qualified */}
                    <div style={{ backgroundColor: 'rgba(2, 132, 199, 0.12)', padding: '14px', borderRadius: '8px', border: '1px solid rgba(56, 189, 248, 0.4)' }}>
                      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '4px' }}>
                        <span style={{ fontSize: '11px', fontWeight: 800, color: '#38bdf8', textTransform: 'uppercase' }}>
                          Ledger B (F₁₅ ≥ -0.01)
                        </span>
                        <span style={{ fontSize: '10px', fontWeight: 700, color: netImp >= 0 ? '#34d399' : '#f87171' }}>
                          Yield: {netImp >= 0 ? '+' : ''}{currSymbol}{formatPrec(netImp)}
                        </span>
                      </div>
                      <div style={{ display: 'flex', alignItems: 'baseline', gap: '8px', marginBottom: '6px' }}>
                        <div style={{ fontSize: '20px', fontWeight: 800, color: (f.net_pnl ?? 0) >= 0 ? '#10b981' : '#ef4444' }}>
                          {(f.net_pnl ?? 0) >= 0 ? '+' : ''}{currSymbol}{formatPrec(f.net_pnl)}
                        </div>
                        <span style={{ fontSize: '10px', color: '#94a3b8' }}>(Total MTM)</span>
                      </div>
                      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '6px', fontSize: '10px', backgroundColor: 'rgba(0,0,0,0.2)', padding: '6px 8px', borderRadius: '4px', marginBottom: '8px' }}>
                        <div>Realized: <strong style={{ color: (f.realized_pnl ?? 0) >= 0 ? '#34d399' : '#f87171' }}>{(f.realized_pnl ?? 0) >= 0 ? '+' : ''}{currSymbol}{formatPrec(f.realized_pnl)}</strong></div>
                        <div>Unrealized: <strong style={{ color: (f.unrealized_pnl ?? 0) >= 0 ? '#38bdf8' : '#f87171' }}>{(f.unrealized_pnl ?? 0) >= 0 ? '+' : ''}{currSymbol}{formatPrec(f.unrealized_pnl)}</strong></div>
                      </div>
                      <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '10px', color: '#cbd5e1' }}>
                        <span>Closed: <strong>{f.closed_trades ?? f.completed_trades ?? 0}</strong> ({f.wins || 0}W/{f.losses || 0}L)</span>
                        <span>WR: <strong>{f.win_rate?.toFixed(1) || '0.0'}%</strong></span>
                        <span>PF: <strong>{f.profit_factor?.toFixed(2) || '0.00'}</strong></span>
                      </div>
                    </div>

                    {/* Card 3: Rejected Opportunities Ledger */}
                    <div style={{ backgroundColor: '#1e293b', padding: '14px', borderRadius: '8px', border: '1px solid rgba(245, 158, 11, 0.4)' }}>
                      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '4px' }}>
                        <span style={{ fontSize: '11px', fontWeight: 800, color: '#fbbf24', textTransform: 'uppercase' }}>
                          Blocked Opportunities
                        </span>
                        <span style={{ fontSize: '10px', color: '#94a3b8' }}>
                          Pending: <strong style={{ color: '#fbbf24' }}>{activeData.pending_open_count || 0} MTM</strong>
                        </span>
                      </div>
                      <div style={{ display: 'flex', alignItems: 'baseline', gap: '8px', marginBottom: '6px' }}>
                        <div style={{ fontSize: '20px', fontWeight: 800, color: (activeData.net_realized_counterfactual ?? netImp) >= 0 ? '#34d399' : '#f87171' }}>
                          {(activeData.net_realized_counterfactual ?? netImp) >= 0 ? '+' : ''}{currSymbol}{formatPrec(activeData.net_realized_counterfactual ?? (activeData.realized_avoided_loss ? (activeData.realized_avoided_loss - activeData.realized_sacrificed_profit) : netImp))}
                        </div>
                        <span style={{ fontSize: '10px', color: '#94a3b8' }}>(Realized Net Counterfactual)</span>
                      </div>
                      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '6px', fontSize: '10px', backgroundColor: 'rgba(0,0,0,0.2)', padding: '6px 8px', borderRadius: '4px', marginBottom: '8px' }}>
                        <div style={{ color: '#34d399' }}>Avoided Loss: <strong>+{currSymbol}{formatPrec(activeData.realized_avoided_loss ?? avoided)}</strong></div>
                        <div style={{ color: '#f87171' }}>Sacrificed Profit: <strong>-{currSymbol}{formatPrec(activeData.realized_sacrificed_profit ?? sacrificed)}</strong></div>
                      </div>
                      <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '10px', color: '#cbd5e1' }}>
                        <span>Total Blocked: <strong>{activeData.rejected_opportunities?.length || 0}</strong></span>
                        <span>Closed Rejected: <strong>{activeData.closed_rejected_count || 0}</strong></span>
                      </div>
                    </div>
                  </div>

                  {/* Ledger Sub-Tab Selector */}
                  <div style={{ display: 'flex', gap: '6px', marginBottom: '10px', borderBottom: '1px solid rgba(255,255,255,0.08)', paddingBottom: '6px' }}>
                    <button
                      onClick={() => setRp605LedgerSubTab('FRESHNESS')}
                      style={{
                        padding: '4px 10px',
                        borderRadius: '4px',
                        border: 'none',
                        backgroundColor: rp605LedgerSubTab === 'FRESHNESS' ? '#0284c7' : 'rgba(255,255,255,0.05)',
                        color: rp605LedgerSubTab === 'FRESHNESS' ? '#ffffff' : '#94a3b8',
                        fontWeight: 700,
                        fontSize: '10px',
                        cursor: 'pointer'
                      }}
                    >
                      ✨ Ledger B ({activeData.freshness_trades?.length || 0})
                    </button>
                    <button
                      onClick={() => setRp605LedgerSubTab('BASELINE')}
                      style={{
                        padding: '4px 10px',
                        borderRadius: '4px',
                        border: 'none',
                        backgroundColor: rp605LedgerSubTab === 'BASELINE' ? '#0284c7' : 'rgba(255,255,255,0.05)',
                        color: rp605LedgerSubTab === 'BASELINE' ? '#ffffff' : '#94a3b8',
                        fontWeight: 700,
                        fontSize: '10px',
                        cursor: 'pointer'
                      }}
                    >
                      📜 Ledger A Baseline ({activeData.baseline_trades?.length || 0})
                    </button>
                    <button
                      onClick={() => setRp605LedgerSubTab('REJECTED')}
                      style={{
                        padding: '4px 10px',
                        borderRadius: '4px',
                        border: 'none',
                        backgroundColor: rp605LedgerSubTab === 'REJECTED' ? '#d97706' : 'rgba(255,255,255,0.05)',
                        color: rp605LedgerSubTab === 'REJECTED' ? '#ffffff' : '#94a3b8',
                        fontWeight: 700,
                        fontSize: '10px',
                        cursor: 'pointer'
                      }}
                    >
                      🚫 Blocked ({activeData.rejected_opportunities?.length || 0})
                    </button>
                  </div>

                  {/* Live Trade Journal Table */}
                  <div style={{ overflowX: 'auto' }}>
                    <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '10px', fontFamily: 'monospace' }}>
                      <thead>
                        <tr style={{ color: '#94a3b8', borderBottom: '1px solid rgba(255,255,255,0.1)', textAlign: 'left' }}>
                          <th style={{ padding: '4px 6px' }}>Trade ID</th>
                          <th style={{ padding: '4px 6px' }}>Symbol</th>
                          <th style={{ padding: '4px 6px' }}>Side</th>
                          <th style={{ padding: '4px 6px' }}>Entry</th>
                          <th style={{ padding: '4px 6px' }}>Exit</th>
                          <th style={{ padding: '4px 6px' }}>Hold</th>
                          <th style={{ padding: '4px 6px' }}>Freshness (F₁₅)</th>
                          <th style={{ padding: '4px 6px' }}>Exit Reason</th>
                          <th style={{ padding: '4px 6px', textAlign: 'right' }}>PnL</th>
                        </tr>
                      </thead>
                      <tbody>
                        {(() => {
                          const list = rp605LedgerSubTab === 'FRESHNESS' 
                            ? activeData.freshness_trades 
                            : rp605LedgerSubTab === 'BASELINE' 
                            ? activeData.baseline_trades 
                            : activeData.rejected_opportunities;

                          if (!list || list.length === 0) {
                            return (
                              <tr>
                                <td colSpan={9} style={{ padding: '12px', textAlign: 'center', color: '#64748b' }}>
                                  No trades logged yet for {m.title}. Session running.
                                </td>
                              </tr>
                            );
                          }

                          return list.map((item: any, idx: number) => {
                            const pnl = parseFloat(item.pnl || item.counterfactual_pnl || 0);
                            const f15 = parseFloat(item.f15_at_entry || 0);
                            const entryP = parseFloat(item.entry_price || 0);
                            const exitP = item.exit_price ? parseFloat(item.exit_price) : null;
                            return (
                              <tr key={item.trade_id || idx} style={{ borderBottom: '1px solid rgba(255,255,255,0.04)' }}>
                                <td style={{ padding: '4px 6px', color: '#38bdf8', fontWeight: 700 }}>{item.trade_id || `TR-${idx}`}</td>
                                <td style={{ padding: '4px 6px', color: '#ffffff', fontWeight: 800 }}>{item.symbol}</td>
                                <td style={{ padding: '4px 6px' }}>
                                  <span style={{ color: item.side === 'SHORT' ? '#f87171' : '#34d399', fontWeight: 700 }}>{item.side}</span>
                                </td>
                                <td style={{ padding: '4px 6px', color: '#cbd5e1' }}>{currSymbol}{formatPrec(entryP, true)}</td>
                                <td style={{ padding: '4px 6px', color: '#cbd5e1' }}>{exitP !== null ? `${currSymbol}${formatPrec(exitP, true)}` : 'OPEN'}</td>
                                <td style={{ padding: '4px 6px', color: '#94a3b8' }}>{item.holding_mins || item.holding_duration_mins || 0}m</td>
                                <td style={{ padding: '4px 6px', color: f15 >= -0.01 ? '#34d399' : '#fbbf24', fontWeight: 700 }}>
                                  {f15 >= 0 ? '+' : ''}{f15.toFixed(3)}
                                </td>
                                <td style={{ padding: '4px 6px', color: '#94a3b8' }}>{item.exit_reason || item.rejection_reason || 'IN_PROGRESS'}</td>
                                <td style={{ padding: '4px 6px', textAlign: 'right', fontWeight: 800, color: pnl >= 0 ? '#10b981' : '#ef4444' }}>
                                  {pnl >= 0 ? '+' : ''}{currSymbol}{formatPrec(pnl, false)}
                                </td>
                              </tr>
                            );
                          });
                        })()}
                      </tbody>
                    </table>
                  </div>
                </div>
              );
            });
          })()}
        </div>
      </div>


      {/* 2A. INTERACTIVE SESSION RECOMMENDATIONS EXPLORER */}
      <div style={{
        backgroundColor: '#0f172a',
        border: '1px solid rgba(56, 189, 248, 0.3)',
        borderRadius: '12px',
        padding: '20px 24px',
        marginBottom: '28px',
        boxShadow: '0 10px 25px rgba(0,0,0,0.4)'
      }}>
        {/* Header */}
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', flexWrap: 'wrap', gap: '12px', marginBottom: '16px' }}>
          <div>
            <h3 style={{ margin: 0, fontSize: '18px', fontWeight: 800, color: '#38bdf8', display: 'flex', alignItems: 'center', gap: '8px' }}>
              <span>💡</span> Session Recommendations Explorer
            </h3>
            <span style={{ fontSize: '12px', color: '#94a3b8' }}>
              {recDataMode === 'NSE_LIVE005'
                ? 'LIVE-005 certified NSE decision book — shadow validation only, zero live execution authority'
                : 'Ranked opportunity stream derived from Coralys v0.1.0 multi-resolution trajectory alignment'}
            </span>
          </div>

          {/* Data Mode Tab Switcher */}
          <div style={{ display: 'flex', backgroundColor: '#1e293b', padding: '3px', borderRadius: '8px', border: '1px solid rgba(255, 255, 255, 0.1)' }}>
            <button
              onClick={() => setRecDataMode('NSE_LIVE005')}
              style={{
                padding: '7px 14px', borderRadius: '6px', border: 'none',
                backgroundColor: recDataMode === 'NSE_LIVE005' ? '#0f766e' : 'transparent',
                color: recDataMode === 'NSE_LIVE005' ? '#2dd4bf' : '#94a3b8',
                fontWeight: 800, fontSize: '11px', cursor: 'pointer', transition: 'all 0.15s ease',
                display: 'flex', alignItems: 'center', gap: '5px'
              }}
            >
              🇮🇳 Today&apos;s NSE Session (LIVE-005)
              {nseRecData?.summary?.actionable_count != null && (
                <span style={{ backgroundColor: '#14b8a6', color: '#000', borderRadius: '10px', padding: '1px 6px', fontSize: '9px', fontWeight: 900 }}>
                  {nseRecData.summary.actionable_count} BUY
                </span>
              )}
            </button>
            <button
              onClick={() => setRecDataMode('ALL_MARKETS')}
              style={{
                padding: '7px 14px', borderRadius: '6px', border: 'none',
                backgroundColor: recDataMode === 'ALL_MARKETS' ? '#0284c7' : 'transparent',
                color: recDataMode === 'ALL_MARKETS' ? '#ffffff' : '#94a3b8',
                fontWeight: 800, fontSize: '11px', cursor: 'pointer', transition: 'all 0.15s ease'
              }}
            >
              🌐 All Markets (Crypto + US + NSE)
            </button>
          </div>
        </div>

        {/* ───── NSE LIVE-005 MODE ───── */}
        {recDataMode === 'NSE_LIVE005' && (() => {
          if (!nseRecData) {
            return (
              <div style={{ padding: '40px', textAlign: 'center', color: '#64748b' }}>
                <div style={{ fontSize: '28px', marginBottom: '8px' }}>⏳</div>
                <div style={{ fontWeight: 700 }}>Loading NSE LIVE-005 session recommendations…</div>
                <div style={{ fontSize: '11px', marginTop: '4px' }}>Fetching from /data/nse_session_recommendations.json</div>
              </div>
            );
          }

          const summary = nseRecData.summary || {};
          const recs: any[] = nseRecData.recommendations || [];

          // Filters
          let filtered = recs.filter(r => {
            if (nseRecSearchQuery && !r.symbol.toLowerCase().includes(nseRecSearchQuery.toLowerCase()) && !r.ticker.toLowerCase().includes(nseRecSearchQuery.toLowerCase())) return false;
            if (nseRecActionFilter !== 'ALL' && r.decision_state !== nseRecActionFilter) return false;
            if (nseRecLedgerFilter !== 'ALL' && r.ledger_status !== nseRecLedgerFilter) return false;
            if (nseRecF15Filter !== 'ALL' && !r.f15_state?.startsWith(nseRecF15Filter)) return false;
            return true;
          });

          const sessionStatusColor = nseRecData.session_status === 'LIVE' ? '#34d399' : nseRecData.session_status === 'CLOSED' ? '#f87171' : '#fbbf24';

          return (
            <div>
              {/* Session Banner */}
              <div style={{ display: 'flex', gap: '16px', flexWrap: 'wrap', marginBottom: '16px' }}>
                <div style={{ backgroundColor: '#1e293b', borderRadius: '8px', padding: '10px 16px', display: 'flex', gap: '20px', flexWrap: 'wrap', flex: 1, alignItems: 'center' }}>
                  <div>
                    <div style={{ fontSize: '9px', color: '#64748b', fontWeight: 700, textTransform: 'uppercase', marginBottom: '2px' }}>Session</div>
                    <div style={{ fontSize: '11px', fontWeight: 800, color: sessionStatusColor }}>
                      {nseRecData.session_status === 'LIVE' ? '🟢 LIVE' : nseRecData.session_status === 'CLOSED' ? '🏁 CLOSED' : '⏳ PRE-OPEN'} — {nseRecData.generated_at_ist}
                    </div>
                  </div>
                  <div>
                    <div style={{ fontSize: '9px', color: '#64748b', fontWeight: 700, textTransform: 'uppercase', marginBottom: '2px' }}>Total Decisions</div>
                    <div style={{ fontSize: '13px', fontWeight: 900, color: '#ffffff' }}>{summary.total_decisions}</div>
                  </div>
                  <div>
                    <div style={{ fontSize: '9px', color: '#64748b', fontWeight: 700, textTransform: 'uppercase', marginBottom: '2px' }}>Actionable (Buy)</div>
                    <div style={{ fontSize: '13px', fontWeight: 900, color: '#34d399' }}>+{summary.actionable_count}</div>
                  </div>
                  <div>
                    <div style={{ fontSize: '9px', color: '#64748b', fontWeight: 700, textTransform: 'uppercase', marginBottom: '2px' }}>Watch</div>
                    <div style={{ fontSize: '13px', fontWeight: 900, color: '#fbbf24' }}>{summary.watch_count}</div>
                  </div>
                  <div>
                    <div style={{ fontSize: '9px', color: '#64748b', fontWeight: 700, textTransform: 'uppercase', marginBottom: '2px' }}>No Trade</div>
                    <div style={{ fontSize: '13px', fontWeight: 900, color: '#64748b' }}>{summary.no_trade_count}</div>
                  </div>
                </div>
                {/* Ledger Scoreboard */}
                <div style={{ backgroundColor: '#1e293b', borderRadius: '8px', padding: '10px 16px', display: 'flex', gap: '20px', flexWrap: 'wrap', alignItems: 'center' }}>
                  <div>
                    <div style={{ fontSize: '9px', color: '#64748b', fontWeight: 700, textTransform: 'uppercase', marginBottom: '2px' }}>Ledger A Active</div>
                    <div style={{ fontSize: '13px', fontWeight: 900, color: '#38bdf8' }}>{summary.ledger_a_active}</div>
                  </div>
                  <div>
                    <div style={{ fontSize: '9px', color: '#64748b', fontWeight: 700, textTransform: 'uppercase', marginBottom: '2px' }}>Ledger B Active</div>
                    <div style={{ fontSize: '13px', fontWeight: 900, color: '#818cf8' }}>{summary.ledger_b_active}</div>
                  </div>
                  <div>
                    <div style={{ fontSize: '9px', color: '#64748b', fontWeight: 700, textTransform: 'uppercase', marginBottom: '2px' }}>Rejected</div>
                    <div style={{ fontSize: '13px', fontWeight: 900, color: '#f87171' }}>{summary.rejected_active}</div>
                  </div>
                  <div>
                    <div style={{ fontSize: '9px', color: '#64748b', fontWeight: 700, textTransform: 'uppercase', marginBottom: '2px' }}>Cold-Start F15</div>
                    <div style={{ fontSize: '13px', fontWeight: 900, color: '#f59e0b' }}>{summary.cold_start_admissions}</div>
                  </div>
                  <div>
                    <div style={{ fontSize: '9px', color: '#64748b', fontWeight: 700, textTransform: 'uppercase', marginBottom: '2px' }}>Ledger A Realized</div>
                    <div style={{ fontSize: '11px', fontWeight: 900, color: (summary.ledger_a_realized_pnl_inr ?? 0) >= 0 ? '#34d399' : '#f87171' }}>
                      ₹{((summary.ledger_a_realized_pnl_inr ?? 0)).toLocaleString('en-IN', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                    </div>
                  </div>
                  <div>
                    <div style={{ fontSize: '9px', color: '#64748b', fontWeight: 700, textTransform: 'uppercase', marginBottom: '2px' }}>Ledger B Realized</div>
                    <div style={{ fontSize: '11px', fontWeight: 900, color: (summary.ledger_b_realized_pnl_inr ?? 0) >= 0 ? '#34d399' : '#f87171' }}>
                      ₹{((summary.ledger_b_realized_pnl_inr ?? 0)).toLocaleString('en-IN', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                    </div>
                  </div>
                </div>
              </div>

              {/* F15 Audit Note */}
              <div style={{ backgroundColor: 'rgba(245, 158, 11, 0.08)', border: '1px solid rgba(245, 158, 11, 0.25)', borderRadius: '6px', padding: '8px 12px', marginBottom: '14px', fontSize: '11px', color: '#d97706' }}>
                <strong>⚠️ F15 Audit:</strong> {nseRecData.f15_audit_note}
              </div>

              {/* NSE Filter Bar */}
              <div style={{ display: 'flex', gap: '12px', flexWrap: 'wrap', backgroundColor: '#1e293b', padding: '10px 14px', borderRadius: '8px', marginBottom: '14px', alignItems: 'flex-end' }}>
                {/* Search */}
                <div>
                  <span style={{ fontSize: '9px', fontWeight: 800, color: '#64748b', display: 'block', marginBottom: '3px', textTransform: 'uppercase' }}>Search Ticker</span>
                  <input type="text" placeholder="e.g. HDFCBANK, RELIANCE…" value={nseRecSearchQuery}
                    onChange={e => setNseRecSearchQuery(e.target.value)}
                    style={{ padding: '5px 10px', borderRadius: '4px', backgroundColor: '#0f172a', color: '#fff', border: '1px solid rgba(255,255,255,0.15)', fontSize: '11px', width: '160px' }}
                  />
                </div>
                {/* Action Filter */}
                <div>
                  <span style={{ fontSize: '9px', fontWeight: 800, color: '#64748b', display: 'block', marginBottom: '3px', textTransform: 'uppercase' }}>LIVE-005 Action</span>
                  <select value={nseRecActionFilter} onChange={e => setNseRecActionFilter(e.target.value as any)}
                    style={{ padding: '5px 10px', borderRadius: '4px', backgroundColor: '#0f172a', color: '#fff', border: '1px solid rgba(255,255,255,0.15)', fontSize: '11px', fontWeight: 700 }}>
                    <option value="ALL">All Actions</option>
                    <option value="BUY">🟢 BUY</option>
                    <option value="WATCH">🟡 WATCH</option>
                    <option value="NO_TRADE">⬛ NO TRADE</option>
                  </select>
                </div>
                {/* Ledger Filter */}
                <div>
                  <span style={{ fontSize: '9px', fontWeight: 800, color: '#64748b', display: 'block', marginBottom: '3px', textTransform: 'uppercase' }}>Ledger Status</span>
                  <select value={nseRecLedgerFilter} onChange={e => setNseRecLedgerFilter(e.target.value as any)}
                    style={{ padding: '5px 10px', borderRadius: '4px', backgroundColor: '#0f172a', color: '#fff', border: '1px solid rgba(255,255,255,0.15)', fontSize: '11px', fontWeight: 700 }}>
                    <option value="ALL">All Ledgers</option>
                    <option value="LEDGER_A_ACTIVE">🔵 Ledger A Active</option>
                    <option value="LEDGER_B_ACTIVE">🟣 Ledger B Active</option>
                    <option value="REJECTED_COUNTERFACTUAL">🔴 Rejected Counterfactual</option>
                    <option value="NOT_YET_ACTIVE">⬜ Not Yet Active</option>
                  </select>
                </div>
                {/* F15 State Filter */}
                <div>
                  <span style={{ fontSize: '9px', fontWeight: 800, color: '#64748b', display: 'block', marginBottom: '3px', textTransform: 'uppercase' }}>F15 Data State</span>
                  <select value={nseRecF15Filter} onChange={e => setNseRecF15Filter(e.target.value as any)}
                    style={{ padding: '5px 10px', borderRadius: '4px', backgroundColor: '#0f172a', color: '#fff', border: '1px solid rgba(255,255,255,0.15)', fontSize: '11px', fontWeight: 700 }}>
                    <option value="ALL">All F15 States</option>
                    <option value="COLD_START">🟠 Cold Start (F15=0.0 fallback)</option>
                    <option value="FULL_LOOKBACK">🟢 Full Lookback (≥15 obs)</option>
                  </select>
                </div>
                {/* Reset */}
                {(nseRecSearchQuery !== '' || nseRecActionFilter !== 'ALL' || nseRecLedgerFilter !== 'ALL' || nseRecF15Filter !== 'ALL') && (
                  <button onClick={() => { setNseRecSearchQuery(''); setNseRecActionFilter('ALL'); setNseRecLedgerFilter('ALL'); setNseRecF15Filter('ALL'); }}
                    style={{ padding: '5px 10px', borderRadius: '4px', backgroundColor: 'rgba(239,68,68,0.15)', color: '#f87171', border: '1px solid rgba(239,68,68,0.35)', fontSize: '10px', fontWeight: 700, cursor: 'pointer', alignSelf: 'flex-end' }}>
                    Reset ✕
                  </button>
                )}
              </div>

              <div style={{ fontSize: '11px', color: '#94a3b8', marginBottom: '10px' }}>
                Showing <strong style={{ color: '#38bdf8' }}>{filtered.length}</strong> of <strong style={{ color: '#fff' }}>{recs.length}</strong> LIVE-005 decisions
              </div>

              {/* NSE LIVE-005 Table */}
              <div style={{ overflowX: 'auto' }}>
                <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '11px', fontFamily: 'monospace' }}>
                  <thead>
                    <tr style={{ backgroundColor: '#1e293b', color: '#64748b', textAlign: 'left', borderBottom: '1px solid rgba(255,255,255,0.1)' }}>
                      <th style={{ padding: '8px 10px' }}>#</th>
                      <th style={{ padding: '8px 10px' }}>Symbol</th>
                      <th style={{ padding: '8px 10px' }}>LIVE-005 Action</th>
                      <th style={{ padding: '8px 10px' }}>Ref Price</th>
                      <th style={{ padding: '8px 10px' }}>Adaptive Target</th>
                      <th style={{ padding: '8px 10px' }}>Adaptive Risk</th>
                      <th style={{ padding: '8px 10px' }}>R:R</th>
                      <th style={{ padding: '8px 10px' }}>H60 Class</th>
                      <th style={{ padding: '8px 10px' }}>Ledger Status</th>
                      <th style={{ padding: '8px 10px' }}>F15 State</th>
                    </tr>
                  </thead>
                  <tbody>
                    {filtered.map((r: any, idx: number) => {
                      const isActionable = r.decision_state === 'BUY' || r.decision_state === 'SHORT';
                      const actionColor = r.decision_state === 'BUY' ? '#34d399' : r.decision_state === 'SHORT' ? '#f87171' : r.decision_state === 'WATCH' ? '#fbbf24' : '#64748b';
                      const ledgerColors: Record<string, string> = {
                        'LEDGER_A_ACTIVE': '#38bdf8',
                        'LEDGER_B_ACTIVE': '#818cf8',
                        'REJECTED_COUNTERFACTUAL': '#f87171',
                        'NOT_YET_ACTIVE': '#475569'
                      };
                      const ledgerBgs: Record<string, string> = {
                        'LEDGER_A_ACTIVE': 'rgba(56,189,248,0.12)',
                        'LEDGER_B_ACTIVE': 'rgba(129,140,248,0.12)',
                        'REJECTED_COUNTERFACTUAL': 'rgba(248,113,113,0.12)',
                        'NOT_YET_ACTIVE': 'rgba(71,85,105,0.15)'
                      };
                      const ledgerLabels: Record<string, string> = {
                        'LEDGER_A_ACTIVE': '🔵 Ledger A',
                        'LEDGER_B_ACTIVE': '🟣 Ledger B',
                        'REJECTED_COUNTERFACTUAL': '🔴 Rejected',
                        'NOT_YET_ACTIVE': '⬜ Not Active'
                      };
                      const isColdStart = r.f15_state === 'COLD_START';
                      const h60Colors: Record<string, string> = {
                        'TRENDING': '#34d399', 'CONTINUATION': '#34d399',
                        'AVOID': '#f87171', 'REVERSAL': '#f59e0b',
                        'UNKNOWN': '#64748b'
                      };

                      return (
                        <tr key={r.decision_id} style={{
                          borderBottom: '1px solid rgba(255,255,255,0.05)',
                          backgroundColor: isActionable ? 'rgba(16,185,129,0.04)' : 'transparent'
                        }}>
                          <td style={{ padding: '9px 10px', color: '#475569', fontWeight: 700 }}>#{idx + 1}</td>
                          <td style={{ padding: '9px 10px' }}>
                            <div style={{ fontWeight: 800, color: '#ffffff', fontSize: '12px' }}>{r.symbol}</div>
                            <div style={{ fontSize: '9px', color: '#64748b' }}>{r.ticker}</div>
                          </td>
                          <td style={{ padding: '9px 10px' }}>
                            <span style={{
                              fontSize: '10px', fontWeight: 800, padding: '3px 8px', borderRadius: '4px',
                              backgroundColor: `${actionColor}22`, color: actionColor,
                              border: `1px solid ${actionColor}44`
                            }}>
                              {r.decision_state === 'BUY' ? '🟢 BUY' : r.decision_state === 'SHORT' ? '🔴 SHORT' : r.decision_state === 'WATCH' ? '🟡 WATCH' : '⬛ NO TRADE'}
                            </span>
                            <div style={{ fontSize: '9px', color: '#64748b', marginTop: '2px' }}>{r.direction}</div>
                          </td>
                          <td style={{ padding: '9px 10px', color: '#ffffff', fontWeight: 700 }}>
                            ₹{r.reference_price?.toLocaleString('en-IN', { minimumFractionDigits: 2 }) ?? '—'}
                          </td>
                          <td style={{ padding: '9px 10px' }}>
                            <span style={{ color: '#34d399', fontWeight: 700 }}>₹{r.adaptive_target?.toLocaleString('en-IN', { minimumFractionDigits: 2 }) ?? '—'}</span>
                            {r.target_pct != null && (
                              <span style={{ fontSize: '9px', color: '#10b981', display: 'block' }}>+{r.target_pct?.toFixed(2)}%</span>
                            )}
                          </td>
                          <td style={{ padding: '9px 10px' }}>
                            <span style={{ color: '#f87171', fontWeight: 700 }}>₹{r.adaptive_risk?.toLocaleString('en-IN', { minimumFractionDigits: 2 }) ?? '—'}</span>
                            {r.risk_pct != null && (
                              <span style={{ fontSize: '9px', color: '#ef4444', display: 'block' }}>{r.risk_pct?.toFixed(2)}%</span>
                            )}
                          </td>
                          <td style={{ padding: '9px 10px' }}>
                            <span style={{ fontWeight: 800, color: (r.rr_ratio ?? 0) >= 2 ? '#34d399' : (r.rr_ratio ?? 0) >= 1 ? '#fbbf24' : '#64748b' }}>
                              {r.rr_ratio != null ? `${r.rr_ratio}:1` : '—'}
                            </span>
                          </td>
                          <td style={{ padding: '9px 10px' }}>
                            <span style={{ fontSize: '9px', fontWeight: 700, color: h60Colors[r.h60_classification] ?? '#64748b' }}>
                              {r.h60_classification ?? 'UNKNOWN'}
                            </span>
                          </td>
                          <td style={{ padding: '9px 10px' }}>
                            <span style={{
                              fontSize: '9px', fontWeight: 700, padding: '2px 6px', borderRadius: '4px',
                              backgroundColor: ledgerBgs[r.ledger_status] ?? 'transparent',
                              color: ledgerColors[r.ledger_status] ?? '#64748b',
                              border: `1px solid ${(ledgerColors[r.ledger_status] ?? '#64748b')}44`
                            }}>
                              {ledgerLabels[r.ledger_status] ?? r.ledger_status}
                            </span>
                          </td>
                          <td style={{ padding: '9px 10px' }}>
                            {r.ledger_status !== 'NOT_YET_ACTIVE' && r.ledger_status !== 'REJECTED_COUNTERFACTUAL' ? (
                              <span style={{
                                fontSize: '9px', fontWeight: 700, padding: '2px 6px', borderRadius: '4px',
                                backgroundColor: isColdStart ? 'rgba(245,158,11,0.12)' : 'rgba(16,185,129,0.12)',
                                color: isColdStart ? '#f59e0b' : '#34d399',
                                border: isColdStart ? '1px solid rgba(245,158,11,0.35)' : '1px solid rgba(16,185,129,0.35)'
                              }}>
                                {isColdStart ? '🟠 COLD START' : '🟢 FULL'}
                              </span>
                            ) : (
                              <span style={{ color: '#334155', fontSize: '9px' }}>—</span>
                            )}
                            {r.f15_at_entry != null && (
                              <div style={{ fontSize: '9px', color: '#64748b', marginTop: '2px' }}>F15={r.f15_at_entry.toFixed(4)}</div>
                            )}
                          </td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              </div>
            </div>
          );
        })()}

        {/* ───── ALL MARKETS MODE ───── */}
        {recDataMode === 'ALL_MARKETS' && (() => {
          const allSyms = Array.from(new Set([
            ...MARKET_CLASS_SYMBOLS.CRYPTO,
            ...MARKET_CLASS_SYMBOLS.NSE_EQUITY,
            ...MARKET_CLASS_SYMBOLS.US_EQUITY,
            'BNB-USD', 'LTC-USD', 'LINK-USD', 'AVAX-USD', 'DOT-USD', 'MATIC-USD', 'SHIB-USD', 'NEAR-USD', 'ATOM-USD', 'UNI-USD',
            'TCS.NS', 'HDFCBANK.NS', 'ICICIBANK.NS', 'SBIN.NS', 'BHARTIARTL.NS', 'LTIM.NS', 'WIPRO.NS',
            'TSLA', 'AMZN', 'GOOGL', 'META', 'AMD', 'NFLX', 'INTC'
          ]));

          const recommendations = allSyms.map(sym => {
            const card = getOrGenerateDecisionCard(sym, liveCards);
            const session = getMarketSessionStatus(sym);
            const state = card.decision_state;

            const traj15m = card.resolutions?.['15m']?.score ?? 0.0;
            const traj30m = card.resolutions?.['30m']?.score ?? 0.0;
            const traj1d = card.resolutions?.['1d']?.score ?? 0.0;

            const q = getQuantTradeParameters(sym, card.current_price, state, traj1d, { '15m': traj15m, '30m': traj30m, '1d': traj1d });

            const roundedPrice = Math.round(card.current_price * 100) / 100;
            const roundedTriggerPrice = Math.round(q.triggerPrice * 100) / 100;
            const isPriceMet = q.direction === 'LONG' ? roundedPrice >= roundedTriggerPrice : roundedPrice <= roundedTriggerPrice;
            const isTrajMet = q.direction === 'LONG' ? traj15m >= 0.20 : traj15m <= -0.20;
            const isAllReady = session.isOpen && isPriceMet && isTrajMet;

            const triggerStatus: 'READY' | 'CONDITIONAL' | 'QUEUED' = !session.isOpen ? 'QUEUED' : isAllReady ? 'READY' : 'CONDITIONAL';

            const alignmentPill: 'UNANIMOUS' | 'INTRADAY' | 'DIVERGENT' =
              (traj15m > 0 && traj30m > 0 && traj1d > 0) || (traj15m < 0 && traj30m < 0 && traj1d < 0)
                ? 'UNANIMOUS'
                : (traj15m > 0 && traj30m > 0) || (traj15m < 0 && traj30m < 0)
                ? 'INTRADAY'
                : 'DIVERGENT';

            const rankScore = (state === 'BUY' || state === 'SHORT' ? 100 : state === 'PULLBACK' ? 60 : 30) +
              (alignmentPill === 'UNANIMOUS' ? 40 : alignmentPill === 'INTRADAY' ? 20 : 0) +
              (isAllReady ? 50 : isPriceMet ? 25 : 0) +
              (Math.abs(traj15m) + Math.abs(traj30m) + Math.abs(traj1d)) * 10;

            return { sym, card, session, state, traj15m, traj30m, traj1d, q, isPriceMet, isTrajMet, isAllReady, triggerStatus, alignmentPill, rankScore };
          });

          recommendations.sort((a, b) => b.rankScore - a.rankScore);

          let filtered = recommendations.filter(item => {
            if (recSearchQuery && !item.sym.toLowerCase().includes(recSearchQuery.toLowerCase())) return false;
            if (recMarketFilter === 'CRYPTO' && !item.sym.includes('-USD')) return false;
            if (recMarketFilter === 'NSE' && (!item.sym.endsWith('.NS') && !item.sym.endsWith('.BO'))) return false;
            if (recMarketFilter === 'US' && (item.sym.includes('-USD') || item.sym.endsWith('.NS') || item.sym.endsWith('.BO'))) return false;
            if (recStateFilter !== 'ALL' && item.state !== recStateFilter) return false;
            if (recAlignFilter !== 'ALL' && item.alignmentPill !== recAlignFilter) return false;
            if (recTriggerFilter !== 'ALL' && item.triggerStatus !== recTriggerFilter) return false;
            return true;
          });

          const totalFilteredCount = filtered.length;
          const displayedItems = recLimit === 'TOP20' ? filtered.slice(0, 20) : filtered;

          return (
            <div>
              {/* Controls bar */}
              <div style={{ display: 'flex', gap: '10px', flexWrap: 'wrap', alignItems: 'center', marginBottom: '14px' }}>
                <div style={{ display: 'flex', backgroundColor: '#1e293b', padding: '3px', borderRadius: '8px', border: '1px solid rgba(255, 255, 255, 0.1)' }}>
                  <button onClick={() => setRecLimit('TOP20')} style={{ padding: '6px 14px', borderRadius: '6px', border: 'none', backgroundColor: recLimit === 'TOP20' ? '#0284c7' : 'transparent', color: recLimit === 'TOP20' ? '#ffffff' : '#94a3b8', fontWeight: 800, fontSize: '11px', cursor: 'pointer', transition: 'all 0.15s ease' }}>🔥 Top 20</button>
                  <button onClick={() => setRecLimit('ALL')} style={{ padding: '6px 14px', borderRadius: '6px', border: 'none', backgroundColor: recLimit === 'ALL' ? '#0284c7' : 'transparent', color: recLimit === 'ALL' ? '#ffffff' : '#94a3b8', fontWeight: 800, fontSize: '11px', cursor: 'pointer', transition: 'all 0.15s ease' }}>🌐 View All</button>
                </div>
                <input type="text" placeholder="Search ticker…" value={recSearchQuery} onChange={(e) => setRecSearchQuery(e.target.value)}
                  style={{ padding: '6px 12px', borderRadius: '6px', backgroundColor: '#1e293b', color: '#ffffff', border: '1px solid rgba(255,255,255,0.15)', fontSize: '12px', width: '160px' }} />
              </div>

              {/* Multi-Filter Bar */}
              <div style={{ display: 'flex', gap: '12px', flexWrap: 'wrap', backgroundColor: '#1e293b', padding: '12px 16px', borderRadius: '8px', marginBottom: '16px', alignItems: 'center' }}>
                <div>
                  <span style={{ fontSize: '10px', fontWeight: 800, color: '#64748b', display: 'block', marginBottom: '4px', textTransform: 'uppercase' }}>Market Class</span>
                  <select value={recMarketFilter} onChange={(e) => setRecMarketFilter(e.target.value as any)} style={{ padding: '6px 10px', borderRadius: '4px', backgroundColor: '#0f172a', color: '#fff', border: '1px solid rgba(255,255,255,0.15)', fontSize: '11px', fontWeight: 700 }}>
                    <option value="ALL">All Markets</option>
                    <option value="CRYPTO">🌐 Crypto Stream</option>
                    <option value="NSE">🇮🇳 Indian Equities (NSE)</option>
                    <option value="US">🇺🇸 US Equities</option>
                  </select>
                </div>
                <div>
                  <span style={{ fontSize: '10px', fontWeight: 800, color: '#64748b', display: 'block', marginBottom: '4px', textTransform: 'uppercase' }}>Action / State</span>
                  <select value={recStateFilter} onChange={(e) => setRecStateFilter(e.target.value as any)} style={{ padding: '6px 10px', borderRadius: '4px', backgroundColor: '#0f172a', color: '#fff', border: '1px solid rgba(255,255,255,0.15)', fontSize: '11px', fontWeight: 700 }}>
                    <option value="ALL">All Actions</option>
                    <option value="BUY">🟢 BUY BIAS</option>
                    <option value="SHORT">🔴 SHORT BIAS</option>
                    <option value="HOLD">🟡 HOLD / CONTINUATION</option>
                    <option value="PULLBACK">🔄 PULLBACK</option>
                    <option value="WATCH">👀 WATCH</option>
                    <option value="EXIT">🛑 EXIT</option>
                  </select>
                </div>
                <div>
                  <span style={{ fontSize: '10px', fontWeight: 800, color: '#64748b', display: 'block', marginBottom: '4px', textTransform: 'uppercase' }}>Resolution Alignment</span>
                  <select value={recAlignFilter} onChange={(e) => setRecAlignFilter(e.target.value as any)} style={{ padding: '6px 10px', borderRadius: '4px', backgroundColor: '#0f172a', color: '#fff', border: '1px solid rgba(255,255,255,0.15)', fontSize: '11px', fontWeight: 700 }}>
                    <option value="ALL">All Alignments</option>
                    <option value="UNANIMOUS">✨ UNANIMOUS CONTINUATION (3/3)</option>
                    <option value="INTRADAY">⚡ INTRADAY ALIGNMENT (2/3)</option>
                    <option value="DIVERGENT">⚠️ DIVERGENT CONFLICT</option>
                  </select>
                </div>
                <div>
                  <span style={{ fontSize: '10px', fontWeight: 800, color: '#64748b', display: 'block', marginBottom: '4px', textTransform: 'uppercase' }}>Trigger Status</span>
                  <select value={recTriggerFilter} onChange={(e) => setRecTriggerFilter(e.target.value as any)} style={{ padding: '6px 10px', borderRadius: '4px', backgroundColor: '#0f172a', color: '#fff', border: '1px solid rgba(255,255,255,0.15)', fontSize: '11px', fontWeight: 700 }}>
                    <option value="ALL">All Trigger States</option>
                    <option value="READY">✅ Conditions Satisfied</option>
                    <option value="CONDITIONAL">⚠️ Waiting on Condition</option>
                    <option value="QUEUED">🔴 Market Closed Queued</option>
                  </select>
                </div>
                {(recMarketFilter !== 'ALL' || recStateFilter !== 'ALL' || recAlignFilter !== 'ALL' || recTriggerFilter !== 'ALL' || recSearchQuery !== '') && (
                  <button onClick={() => { setRecMarketFilter('ALL'); setRecStateFilter('ALL'); setRecAlignFilter('ALL'); setRecTriggerFilter('ALL'); setRecSearchQuery(''); }}
                    style={{ marginTop: '16px', padding: '6px 10px', borderRadius: '4px', backgroundColor: 'rgba(239, 68, 68, 0.2)', color: '#f87171', border: '1px solid rgba(239, 68, 68, 0.4)', fontSize: '10px', fontWeight: 700, cursor: 'pointer' }}>
                    Reset Filters ✕
                  </button>
                )}
              </div>

              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px', fontSize: '11px', color: '#94a3b8' }}>
                <div>
                  Showing <strong style={{ color: '#38bdf8' }}>{displayedItems.length}</strong> of <strong style={{ color: '#ffffff' }}>{totalFilteredCount}</strong> filtered recommendations
                  {recLimit === 'TOP20' && totalFilteredCount > 20 && (<span> (Top 20 Ranked Mode Active)</span>)}
                </div>
                {recLimit === 'TOP20' && totalFilteredCount > 20 && (
                  <button onClick={() => setRecLimit('ALL')} style={{ background: 'none', border: 'none', color: '#38bdf8', fontWeight: 700, cursor: 'pointer', fontSize: '11px', textDecoration: 'underline' }}>
                    View All {totalFilteredCount} Recommendations →
                  </button>
                )}
              </div>

              <div style={{ overflowX: 'auto' }}>
                <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '11px', fontFamily: 'monospace' }}>
                  <thead>
                    <tr style={{ backgroundColor: '#1e293b', color: '#64748b', textAlign: 'left', borderBottom: '1px solid rgba(255,255,255,0.1)' }}>
                      <th style={{ padding: '8px 10px' }}>Rank</th>
                      <th style={{ padding: '8px 10px' }}>Instrument</th>
                      <th style={{ padding: '8px 10px' }}>Session</th>
                      <th style={{ padding: '8px 10px' }}>Recommendation State</th>
                      <th style={{ padding: '8px 10px' }}>Alignment</th>
                      <th style={{ padding: '8px 10px' }}>15m / 30m / 1d Scores</th>
                      <th style={{ padding: '8px 10px' }}>Trigger Price</th>
                      <th style={{ padding: '8px 10px' }}>Target TP</th>
                      <th style={{ padding: '8px 10px' }}>Stop Loss</th>
                      <th style={{ padding: '8px 10px' }}>Trigger Readiness</th>
                      <th style={{ padding: '8px 10px', textAlign: 'right' }}>Actions</th>
                    </tr>
                  </thead>
                  <tbody>
                    {displayedItems.map((item, idx) => {
                      const { sym, card, session, state, traj15m, traj30m, traj1d, q, isAllReady, triggerStatus, alignmentPill } = item;
                      const isSelected = selectedSymbol === sym;

                      return (
                        <tr key={sym} style={{ borderBottom: '1px solid rgba(255,255,255,0.05)', backgroundColor: isSelected ? 'rgba(56, 189, 248, 0.08)' : 'transparent' }}>
                          <td style={{ padding: '10px', fontWeight: 800, color: '#64748b' }}>#{idx + 1}</td>
                          <td style={{ padding: '10px' }}>
                            <strong style={{ color: '#ffffff', fontSize: '12px' }}>{sym}</strong>
                            <span style={{ fontSize: '10px', color: '#94a3b8', display: 'block' }}>{formatAssetCurrency(card.current_price, sym)}</span>
                          </td>
                          <td style={{ padding: '10px' }}>
                            <span style={{ fontSize: '9px', fontWeight: 700, padding: '2px 6px', borderRadius: '4px', backgroundColor: session.isOpen ? 'rgba(16, 185, 129, 0.15)' : 'rgba(239, 68, 68, 0.15)', color: session.isOpen ? '#34d399' : '#f87171' }}>
                              {session.isOpen ? '🟢 LIVE' : '🔴 CLOSED'}
                            </span>
                          </td>
                          <td style={{ padding: '10px' }}>
                            <span style={{ fontSize: '10px', fontWeight: 800, padding: '3px 8px', borderRadius: '4px', backgroundColor: q.direction === 'SHORT' ? 'rgba(239, 68, 68, 0.2)' : 'rgba(16, 185, 129, 0.2)', color: q.direction === 'SHORT' ? '#f87171' : '#34d399', border: `1px solid ${q.direction === 'SHORT' ? 'rgba(239, 68, 68, 0.4)' : 'rgba(16, 185, 129, 0.4)'}` }}>
                              {state} ({q.direction})
                            </span>
                          </td>
                          <td style={{ padding: '10px' }}>
                            <span style={{ fontSize: '9px', fontWeight: 700, color: alignmentPill === 'UNANIMOUS' ? '#38bdf8' : alignmentPill === 'INTRADAY' ? '#fbbf24' : '#f43f5e' }}>
                              {alignmentPill === 'UNANIMOUS' ? '✨ UNANIMOUS' : alignmentPill === 'INTRADAY' ? '⚡ INTRADAY' : '⚠️ DIVERGENT'}
                            </span>
                          </td>
                          <td style={{ padding: '10px' }}>
                            <span style={{ color: traj15m >= 0 ? '#10b981' : '#ef4444' }}>{traj15m >= 0 ? '+' : ''}{traj15m.toFixed(2)}</span> /{' '}
                            <span style={{ color: traj30m >= 0 ? '#10b981' : '#ef4444' }}>{traj30m >= 0 ? '+' : ''}{traj30m.toFixed(2)}</span> /{' '}
                            <span style={{ color: traj1d >= 0 ? '#10b981' : '#ef4444' }}>{traj1d >= 0 ? '+' : ''}{traj1d.toFixed(2)}</span>
                          </td>
                          <td style={{ padding: '10px', color: '#ffffff' }}>{formatAssetCurrency(q.triggerPrice, sym)}</td>
                          <td style={{ padding: '10px', color: q.targetPct > 0 ? '#10b981' : '#f43f5e' }}>
                            {formatAssetCurrency(q.targetPrice, sym)} ({q.targetPct > 0 ? '+' : ''}{q.targetPct.toFixed(2)}%)
                          </td>
                          <td style={{ padding: '10px', color: q.stopLossPct < 0 ? '#ef4444' : '#fbbf24' }}>
                            {formatAssetCurrency(q.stopLossPrice, sym)} ({q.stopLossPct > 0 ? '+' : ''}{q.stopLossPct.toFixed(2)}%)
                          </td>
                          <td style={{ padding: '10px' }}>
                            <span style={{ fontSize: '9px', fontWeight: 700, padding: '2px 6px', borderRadius: '4px', backgroundColor: triggerStatus === 'READY' ? 'rgba(16, 185, 129, 0.2)' : triggerStatus === 'CONDITIONAL' ? 'rgba(245, 158, 11, 0.15)' : 'rgba(239, 68, 68, 0.15)', color: triggerStatus === 'READY' ? '#34d399' : triggerStatus === 'CONDITIONAL' ? '#fbbf24' : '#f87171' }}>
                              {triggerStatus === 'READY' ? '✅ READY' : triggerStatus === 'CONDITIONAL' ? '⚠️ CONDITIONAL' : '🔴 QUEUED'}
                            </span>
                          </td>
                          <td style={{ padding: '10px', textAlign: 'right' }}>
                            <div style={{ display: 'flex', gap: '4px', justifyContent: 'flex-end' }}>
                              <button onClick={() => executeDirectPaperTradeFromCard(sym, q.direction, card)}
                                style={{ padding: '4px 8px', borderRadius: '4px', border: 'none', backgroundColor: isAllReady ? '#10b981' : '#0284c7', color: '#ffffff', fontWeight: 800, fontSize: '9.5px', cursor: 'pointer' }}>
                                {isAllReady ? '⚡ EXECUTE NOW' : '🎯 DIRECT ORDER'}
                              </button>
                              <button onClick={() => handlePaperTradeShortcut(sym, q.direction === 'LONG' ? 'BUY' : 'SELL SHORT')}
                                style={{ padding: '4px 8px', borderRadius: '4px', border: '1px solid rgba(255,255,255,0.15)', backgroundColor: '#1e293b', color: '#94a3b8', fontWeight: 700, fontSize: '9.5px', cursor: 'pointer' }}>
                                📝 TICKET
                              </button>
                            </div>
                          </td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              </div>
            </div>
          );
        })()}
      </div>


      {/* 2. MULTI-HORIZON COMPARE & PAPER TRADE SHORTCUT SURFACE */}
      <div style={{ marginBottom: '28px' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px', flexWrap: 'wrap', gap: '12px' }}>
          <div>
            <h3 style={{ margin: 0, fontSize: '16px', fontWeight: 800, color: '#38bdf8', textTransform: 'uppercase', letterSpacing: '0.05em', display: 'flex', alignItems: 'center', gap: '8px' }}>
              <span>📊</span> Multi-Horizon Asset Compare Surface
            </h3>
            <p style={{ margin: '4px 0 0 0', fontSize: '12px', color: '#94a3b8' }}>
              Compare multi-horizon trajectory alignment (15m, 30m, 1d) across market asset classes with direct paper trade shortcuts.
            </p>
          </div>

          {/* Market Class Selector Pills */}
          <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
            {([
              { id: 'ALL', label: `All Markets (${(MARKET_CLASS_SYMBOLS.CRYPTO.length + MARKET_CLASS_SYMBOLS.NSE_EQUITY.length + MARKET_CLASS_SYMBOLS.US_EQUITY.length)})` },
              { id: 'CRYPTO', label: `🌐 Crypto (${MARKET_CLASS_SYMBOLS.CRYPTO.length})` },
              { id: 'NSE_EQUITY', label: `🇮🇳 NSE Equities (${MARKET_CLASS_SYMBOLS.NSE_EQUITY.length})` },
              { id: 'US_EQUITY', label: `🇺🇸 US Equities (${MARKET_CLASS_SYMBOLS.US_EQUITY.length})` }
            ] as const).map(tab => {
              const isActive = marketCategoryFilter === tab.id;
              return (
                <button
                  key={tab.id}
                  onClick={() => setMarketCategoryFilter(tab.id as MarketCategory)}
                  style={{
                    padding: '6px 14px',
                    borderRadius: '8px',
                    fontSize: '12px',
                    fontWeight: 700,
                    cursor: 'pointer',
                    transition: 'all 0.2s ease',
                    border: isActive ? '1px solid #38bdf8' : '1px solid rgba(255, 255, 255, 0.1)',
                    backgroundColor: isActive ? 'rgba(56, 189, 248, 0.2)' : '#1e293b',
                    color: isActive ? '#38bdf8' : '#94a3b8',
                    boxShadow: isActive ? '0 0 12px rgba(56, 189, 248, 0.25)' : 'none'
                  }}
                >
                  {tab.label}
                </button>
              );
            })}
          </div>
        </div>

        {/* Grid of Multi-Horizon Cards */}
        <div style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fill, minmax(320px, 1fr))',
          gap: '16px'
        }}>
          {(marketCategoryFilter === 'ALL'
            ? [...MARKET_CLASS_SYMBOLS.CRYPTO, ...MARKET_CLASS_SYMBOLS.NSE_EQUITY, ...MARKET_CLASS_SYMBOLS.US_EQUITY]
            : MARKET_CLASS_SYMBOLS[marketCategoryFilter] || []
          ).map(sym => {
            const card = getOrGenerateDecisionCard(sym, liveCards);
            const marketInfo = getSymbolMarketLabel(sym);
            const session = getMarketSessionStatus(sym);
            const state = card.decision_state;
            const badgeColor = state === 'HOLD' ? '#10b981' : state === 'EXIT' ? '#ef4444' : '#f59e0b';
            const isSelected = selectedSymbol === sym;

            // Extract trajectory values directly from card.resolutions
            const traj15m = card.resolutions?.['15m']?.score ?? 0.0;
            const traj30m = card.resolutions?.['30m']?.score ?? 0.0;
            const traj1d = card.resolutions?.['1d']?.score ?? 0.0;

            return (
              <div
                key={sym}
                style={{
                  backgroundColor: isSelected ? 'rgba(56, 189, 248, 0.08)' : '#0f172a',
                  border: isSelected ? '1px solid #38bdf8' : '1px solid rgba(255, 255, 255, 0.1)',
                  borderRadius: '12px',
                  padding: '16px',
                  display: 'flex',
                  flexDirection: 'column',
                  justifyContent: 'space-between',
                  boxShadow: isSelected ? '0 4px 20px rgba(56, 189, 248, 0.15)' : 'none',
                  transition: 'all 0.2s ease'
                }}
              >
                {/* Symbol Header */}
                <div>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '8px' }}>
                    <div>
                      <div style={{ display: 'flex', alignItems: 'center', gap: '6px', flexWrap: 'wrap' }}>
                        <span style={{ fontWeight: 800, fontSize: '16px', color: '#ffffff' }}>{sym}</span>
                        <span style={{
                          padding: '2px 6px',
                          borderRadius: '4px',
                          fontSize: '9.5px',
                          fontWeight: 700,
                          backgroundColor: marketInfo.color + '20',
                          color: marketInfo.color,
                          border: `1px solid ${marketInfo.color}40`
                        }}>
                          {marketInfo.label}
                        </span>
                        <span style={{
                          padding: '2px 6px',
                          borderRadius: '4px',
                          fontSize: '9.5px',
                          fontWeight: 700,
                          backgroundColor: session.badgeBg,
                          color: session.badgeColor,
                          border: `1px solid ${session.badgeColor}40`,
                          display: 'flex',
                          alignItems: 'center',
                          gap: '4px'
                        }}>
                          <span style={{ width: '5px', height: '5px', borderRadius: '50%', backgroundColor: session.badgeColor, display: 'inline-block' }}></span>
                          {session.statusText} — {session.reopenText}
                        </span>
                      </div>
                      <div style={{ fontSize: '14px', fontWeight: 800, color: '#38bdf8', fontFamily: 'monospace', marginTop: '4px' }}>
                        {formatAssetCurrency(card.current_price, sym)}
                      </div>
                    </div>
                    <span style={{
                      padding: '4px 10px',
                      borderRadius: '6px',
                      fontSize: '11px',
                      fontWeight: 800,
                      backgroundColor: badgeColor + '20',
                      color: badgeColor,
                      border: `1px solid ${badgeColor}50`,
                      boxShadow: `0 0 10px ${badgeColor}20`
                    }}>
                      {state}
                    </span>
                  </div>

                  {/* Alignment Relationship */}
                  <div style={{ fontSize: '11px', color: '#94a3b8', marginBottom: '12px', display: 'flex', alignItems: 'center', gap: '6px' }}>
                    <span style={{ color: '#cbd5e1', fontWeight: 600 }}>Cross-Resolution:</span>
                    <span style={{ color: '#38bdf8', fontWeight: 700 }}>{card.cross_resolution_relationship}</span>
                  </div>

                  {/* Multi-Horizon Compare Matrix */}
                  <div style={{
                    backgroundColor: '#1e293b',
                    borderRadius: '8px',
                    padding: '10px 12px',
                    marginBottom: '14px',
                    display: 'grid',
                    gridTemplateColumns: '1fr 1fr 1fr',
                    gap: '8px',
                    textAlign: 'center',
                    border: '1px solid rgba(255,255,255,0.05)'
                  }}>
                    <div>
                      <div style={{ fontSize: '10px', color: '#64748b', fontWeight: 700 }}>15m Horizon</div>
                      <div style={{ fontSize: '12px', fontWeight: 800, color: traj15m >= 0 ? '#10b981' : '#ef4444', fontFamily: 'monospace', marginTop: '2px' }}>
                        {traj15m >= 0 ? '+' : ''}{traj15m.toFixed(2)}
                      </div>
                    </div>
                    <div style={{ borderLeft: '1px solid rgba(255,255,255,0.08)', borderRight: '1px solid rgba(255,255,255,0.08)' }}>
                      <div style={{ fontSize: '10px', color: '#64748b', fontWeight: 700 }}>30m Horizon</div>
                      <div style={{ fontSize: '12px', fontWeight: 800, color: traj30m >= 0 ? '#10b981' : '#ef4444', fontFamily: 'monospace', marginTop: '2px' }}>
                        {traj30m >= 0 ? '+' : ''}{traj30m.toFixed(2)}
                      </div>
                    </div>
                    <div>
                      <div style={{ fontSize: '10px', color: '#64748b', fontWeight: 700 }}>1d Horizon</div>
                      <div style={{ fontSize: '12px', fontWeight: 800, color: traj1d >= 0 ? '#10b981' : '#ef4444', fontFamily: 'monospace', marginTop: '2px' }}>
                        {traj1d >= 0 ? '+' : ''}{traj1d.toFixed(2)}
                      </div>
                    </div>
                  </div>

                  {/* Specific & Unique Derivation Reasoning */}
                  <div style={{
                    backgroundColor: 'rgba(15, 23, 42, 0.7)',
                    borderRadius: '8px',
                    padding: '10px 12px',
                    marginBottom: '14px',
                    border: '1px solid rgba(255, 255, 255, 0.08)'
                  }}>
                    <div style={{ fontSize: '10px', fontWeight: 800, color: '#38bdf8', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '6px', display: 'flex', alignItems: 'center', gap: '5px' }}>
                      <span>💡</span> Recommendation Rationale
                    </div>
                    <ul style={{ margin: 0, paddingLeft: '14px', fontSize: '11px', color: '#cbd5e1', lineHeight: '1.45' }}>
                      {(card.evidence && card.evidence.length > 0
                        ? card.evidence
                        : (() => {
                            const q = getQuantTradeParameters(sym, card.current_price, state, traj1d, { '15m': traj15m, '30m': traj30m, '1d': traj1d });
                            return [
                              `• DYNAMICS: 15m (${traj15m > 0 ? '+' : ''}${traj15m.toFixed(2)}), 30m (${traj30m > 0 ? '+' : ''}${traj30m.toFixed(2)}), 1d (${traj1d > 0 ? '+' : ''}${traj1d.toFixed(2)})`,
                              `• RECOMMENDATION RATIONALE: ${state} — Entry/Trigger: ${q.formattedTriggerPrice} | Target Price: ${q.formattedTargetPrice} (${q.targetPct > 0 ? '+' : ''}${q.targetPct.toFixed(2)}%) | Stop Loss Price: ${q.formattedStopLossPrice} (${q.stopLossPct > 0 ? '+' : ''}${q.stopLossPct.toFixed(2)}%) | Horizon Duration: ${q.horizonDuration}.`,
                              `• ACTIONABLE TRIGGER: Monitor velocity relative to threshold ±0.20 for momentum shift (${q.triggerLabel}: ${q.formattedTriggerPrice}).`,
                              `• CAPITAL PROTECTION: Stop loss set at ${q.formattedStopLossPrice}; maximum horizon exit enforced at ${q.horizonDuration}.`
                            ];
                          })()
                      ).map((item, idx) => (
                        <li key={idx} style={{ marginBottom: '3px' }}>
                          {item.startsWith('• ') ? item.substring(2) : item}
                        </li>
                      ))}
                    </ul>
                  </div>

                  {/* Quantitative Trade Execution Parameters */}
                  {(() => {
                    const q = getQuantTradeParameters(sym, card.current_price, state, traj1d, { '15m': traj15m, '30m': traj30m, '1d': traj1d });
                    const roundedPrice = Math.round(card.current_price * 100) / 100;
                    const roundedTriggerPrice = Math.round(q.triggerPrice * 100) / 100;
                    const isPriceMet = q.direction === 'LONG' ? roundedPrice >= roundedTriggerPrice : roundedPrice <= roundedTriggerPrice;
                    const isTrajMet = q.direction === 'LONG' ? traj15m >= 0.20 : traj15m <= -0.20;
                    const isAllReady = session.isOpen && isPriceMet && isTrajMet;

                    return (
                      <div style={{
                        backgroundColor: 'rgba(30, 41, 59, 0.7)',
                        borderRadius: '8px',
                        padding: '10px 12px',
                        marginBottom: '14px',
                        border: `1px solid ${q.direction === 'SHORT' ? 'rgba(239, 68, 68, 0.3)' : 'rgba(56, 189, 248, 0.3)'}`
                      }}>
                        <div style={{ fontSize: '10px', fontWeight: 800, color: '#38bdf8', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '8px', display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                          <span style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>🎯 Quant Trade Parameters</span>
                          <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                            <span style={{ fontSize: '9px', fontWeight: 700, padding: '1px 5px', borderRadius: '3px', backgroundColor: q.direction === 'SHORT' ? 'rgba(239, 68, 68, 0.2)' : 'rgba(16, 185, 129, 0.2)', color: q.direction === 'SHORT' ? '#f87171' : '#34d399' }}>
                              {q.direction} BIAS
                            </span>
                            <span style={{ fontSize: '9px', color: '#94a3b8' }}>Config A</span>
                          </div>
                        </div>
                        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '8px', fontSize: '11px', fontFamily: 'monospace' }}>
                          <div>
                            <span style={{ color: '#94a3b8', fontSize: '10px', display: 'block' }}>TRIGGER PRICE ({q.triggerLabel})</span>
                            <strong style={{ color: '#ffffff' }}>{formatAssetCurrency(q.triggerPrice, sym)}</strong>
                          </div>
                          <div>
                            <span style={{ color: '#94a3b8', fontSize: '10px', display: 'block' }}>TARGET PRICE</span>
                            <strong style={{ color: q.targetPct > 0 ? '#10b981' : '#f43f5e' }}>{formatAssetCurrency(q.targetPrice, sym)} <span style={{ fontSize: '9px' }}>({q.targetPct > 0 ? '+' : ''}{q.targetPct.toFixed(2)}%)</span></strong>
                          </div>
                          <div>
                            <span style={{ color: '#94a3b8', fontSize: '10px', display: 'block' }}>STOP LOSS</span>
                            <strong style={{ color: q.stopLossPct < 0 ? '#ef4444' : '#fbbf24' }}>{formatAssetCurrency(q.stopLossPrice, sym)} <span style={{ fontSize: '9px' }}>({q.stopLossPct > 0 ? '+' : ''}{q.stopLossPct.toFixed(2)}%)</span></strong>
                          </div>
                          <div>
                            <span style={{ color: '#94a3b8', fontSize: '10px', display: 'block' }}>HORIZON DURATIONS (15m / 30m / 1d)</span>
                            <strong style={{ color: '#f59e0b', fontSize: '9.5px', display: 'block' }}>
                              15m: 45m | 30m: 90m | 1d: 24h
                            </strong>
                          </div>
                        </div>

                        {/* Conditional Trigger Satisfaction Panel */}
                        <div style={{
                          marginTop: '8px',
                          padding: '6px 8px',
                          borderRadius: '4px',
                          fontSize: '9.5px',
                          fontWeight: 700,
                          backgroundColor: isAllReady ? 'rgba(16, 185, 129, 0.2)' : 'rgba(245, 158, 11, 0.15)',
                          color: isAllReady ? '#34d399' : '#fbbf24',
                          border: `1px solid ${isAllReady ? 'rgba(16, 185, 129, 0.3)' : 'rgba(245, 158, 11, 0.3)'}`
                        }}>
                          {!session.isOpen ? (
                            `🔴 ${session.statusText}: Trigger execution queued until ${session.reopenText}`
                          ) : isAllReady ? (
                            `✅ TRIGGER CONDITIONS SATISFIED: Price (${formatAssetCurrency(card.current_price, sym)}) & Trajectory (${traj15m > 0 ? '+' : ''}${traj15m.toFixed(2)}) Ready`
                          ) : !isPriceMet ? (
                            `⚠️ TRIGGER CONDITIONAL: Waiting for price (${formatAssetCurrency(card.current_price, sym)}) to reach ${q.triggerLabel} ${formatAssetCurrency(q.triggerPrice, sym)}`
                          ) : (
                            `⚠️ TRIGGER CONDITIONAL: Price reached ${formatAssetCurrency(q.triggerPrice, sym)}, WAITING: 15m Trajectory Score (${traj15m > 0 ? '+' : ''}${traj15m.toFixed(2)}) below ${q.direction === 'LONG' ? '+0.20' : '-0.20'} threshold`
                          )}
                        </div>

                        {/* 1-Click Execution / Trigger Button */}
                        <button
                          onClick={() => executeDirectPaperTradeFromCard(sym, q.direction, card)}
                          style={{
                            marginTop: '8px',
                            width: '100%',
                            padding: '7px 10px',
                            borderRadius: '4px',
                            backgroundColor: isAllReady
                              ? 'rgba(16, 185, 129, 0.9)'
                              : isPriceMet
                              ? 'rgba(245, 158, 11, 0.9)'
                              : 'rgba(56, 189, 248, 0.2)',
                            color: isAllReady || isPriceMet ? '#ffffff' : '#38bdf8',
                            fontWeight: 800,
                            fontSize: '10.5px',
                            border: `1px solid ${isAllReady ? '#10b981' : isPriceMet ? '#f59e0b' : 'rgba(56, 189, 248, 0.4)'}`,
                            cursor: 'pointer',
                            boxShadow: isAllReady ? '0 0 10px rgba(16, 185, 129, 0.4)' : isPriceMet ? '0 0 10px rgba(245, 158, 11, 0.3)' : 'none',
                            transition: 'all 0.15s ease'
                          }}
                        >
                          {isAllReady
                            ? `⚡ EXECUTE ${q.direction} TRADE NOW (@ ${formatAssetCurrency(q.triggerPrice, sym)})`
                            : isPriceMet
                            ? `⚡ PRICE TRIGGER REACHED (${formatAssetCurrency(q.triggerPrice, sym)}): EXECUTE ${q.direction} NOW`
                            : `🎯 SUBMIT PENDING ${q.direction} TRIGGER ORDER (@ ${formatAssetCurrency(q.triggerPrice, sym)})`}
                        </button>
                      </div>
                    );
                  })()}
                </div>

                {/* Direct Paper Trade Action Shortcuts */}
                <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: '8px', marginTop: 'auto' }}>
                  <button
                    onClick={() => handlePaperTradeShortcut(sym, 'BUY')}
                    style={{
                      padding: '8px 6px',
                      borderRadius: '6px',
                      backgroundColor: 'rgba(16, 185, 129, 0.18)',
                      border: '1px solid #10b981',
                      color: '#10b981',
                      fontSize: '11px',
                      fontWeight: 800,
                      cursor: 'pointer',
                      transition: 'all 0.2s ease',
                      textAlign: 'center'
                    }}
                    title={`Shortcut: Pre-fill Order Ticket with BUY ${sym}`}
                  >
                    ⚡ Paper BUY
                  </button>

                  <button
                    onClick={() => handlePaperTradeShortcut(sym, 'SELL SHORT')}
                    style={{
                      padding: '8px 6px',
                      borderRadius: '6px',
                      backgroundColor: 'rgba(239, 68, 68, 0.18)',
                      border: '1px solid #ef4444',
                      color: '#ef4444',
                      fontSize: '11px',
                      fontWeight: 800,
                      cursor: 'pointer',
                      transition: 'all 0.2s ease',
                      textAlign: 'center'
                    }}
                    title={`Shortcut: Pre-fill Order Ticket with SHORT ${sym}`}
                  >
                    ⚡ Paper SHORT
                  </button>

                  <button
                    onClick={() => handlePaperTradeShortcut(sym, 'BUY')}
                    style={{
                      padding: '8px 6px',
                      borderRadius: '6px',
                      backgroundColor: 'rgba(56, 189, 248, 0.15)',
                      border: '1px solid #38bdf8',
                      color: '#38bdf8',
                      fontSize: '11px',
                      fontWeight: 700,
                      cursor: 'pointer',
                      transition: 'all 0.2s ease',
                      textAlign: 'center'
                    }}
                    title={`Select ${sym} in Order Ticket`}
                  >
                    📝 Ticket
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* 3. POSITIONS & ORDER TICKET GRID */}
      <div style={{
        display: 'grid',
        gridTemplateColumns: '1fr 380px',
        gap: '24px',
        marginBottom: '28px'
      }}>
        {/* Left Column: Positions Table */}
        <div style={{
          backgroundColor: '#0f172a',
          border: '1px solid rgba(255, 255, 255, 0.1)',
          borderRadius: '12px',
          padding: '20px'
        }}>
          <h3 style={{ margin: '0 0 16px 0', fontSize: '16px', fontWeight: 800, color: '#f8fafc', display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
            <span>💼 Simulated Open Positions ({positions.length})</span>
            <span style={{ fontSize: '12px', color: '#38bdf8', fontWeight: 600 }}>Live Mark-to-Market & Temporal Holding Policy</span>
          </h3>

          {positions.length === 0 ? (
            <div style={{ padding: '40px 20px', textAlign: 'center', color: '#64748b', fontSize: '14px' }}>
              No open positions. Select an active decision recommendation to execute a paper trade.
            </div>
          ) : (
            <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left', fontSize: '12px' }}>
              <thead>
                <tr style={{ backgroundColor: '#1e293b', color: '#94a3b8', borderBottom: '1px solid rgba(255,255,255,0.1)' }}>
                  <th style={{ padding: '10px 10px' }}>Asset</th>
                  <th style={{ padding: '10px 10px' }}>Qty</th>
                  <th style={{ padding: '10px 10px' }}>Entry Price</th>
                  <th style={{ padding: '10px 10px' }}>Last Price</th>
                  <th style={{ padding: '10px 10px' }}>Unrealized P&L</th>
                  <th style={{ padding: '10px 10px' }}>Exit Target</th>
                  <th style={{ padding: '10px 10px' }}>Time Remaining</th>
                  <th style={{ padding: '10px 10px' }}>Temporal State</th>
                  <th style={{ padding: '10px 10px', textAlign: 'right' }}>Action</th>
                </tr>
              </thead>
              <tbody>
                {positions.map(pos => {
                  const isProfit = pos.unrealized_pnl >= 0;
                  const tempState = getPositionTemporalState(pos, nowUtcMs);
                  const targetTimeStr = pos.exit_policy?.target_exit_utc ? formatDualTime(pos.exit_policy.target_exit_utc) : 'OPEN';

                  return (
                    <tr key={pos.id} style={{ borderBottom: '1px solid rgba(255,255,255,0.05)', backgroundColor: tempState.requiresReview ? 'rgba(239, 68, 68, 0.05)' : 'transparent' }}>
                      <td style={{ padding: '10px', fontWeight: 800, color: '#ffffff' }}>
                        <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                          <span>{pos.symbol}</span>
                          <span style={{
                            fontSize: '9px',
                            fontWeight: 800,
                            padding: '1px 5px',
                            borderRadius: '3px',
                            backgroundColor: pos.side === 'SHORT' ? 'rgba(239, 68, 68, 0.2)' : 'rgba(16, 185, 129, 0.2)',
                            color: pos.side === 'SHORT' ? '#ef4444' : '#10b981',
                            border: `1px solid ${pos.side === 'SHORT' ? '#ef444450' : '#10b98150'}`
                          }}>
                            {pos.side || 'LONG'}
                          </span>
                        </div>
                        <div style={{ fontSize: '10px', color: '#64748b' }}>{formatDualTime(pos.entry_time_utc)}</div>
                      </td>
                      <td style={{ padding: '10px', fontFamily: 'monospace' }}>
                        {typeof pos.quantity === 'number' ? pos.quantity.toLocaleString('en-US') : pos.quantity}
                      </td>
                      <td style={{ padding: '10px', fontFamily: 'monospace', color: '#94a3b8' }}>
                        {formatAssetCurrency(pos.entry_price, pos.symbol)}
                      </td>
                      <td style={{ padding: '10px', fontFamily: 'monospace', fontWeight: 700, color: '#38bdf8' }}>
                        {formatAssetCurrency(pos.current_price, pos.symbol)}
                      </td>
                      <td style={{ padding: '10px', fontWeight: 800, color: isProfit ? '#10b981' : '#ef4444' }}>
                        {isProfit ? '+' : ''}{formatAssetCurrency(pos.unrealized_pnl, pos.symbol)} ({isProfit ? '+' : ''}{pos.unrealized_pnl_pct.toFixed(2)}%)
                      </td>
                      <td style={{ padding: '10px', fontSize: '11px', color: '#cbd5e1', fontFamily: 'monospace' }}>
                        {targetTimeStr}
                      </td>
                      <td style={{ padding: '10px', fontFamily: 'monospace', fontWeight: 800, color: tempState.badgeColor }}>
                        ⏱️ {tempState.formattedTimeRemaining}
                      </td>
                      <td style={{ padding: '10px' }}>
                        <span style={{
                          padding: '3px 8px',
                          borderRadius: '4px',
                          fontSize: '10px',
                          fontWeight: 800,
                          backgroundColor: tempState.badgeColor + '20',
                          color: tempState.badgeColor,
                          border: `1px solid ${tempState.badgeColor}50`,
                          whiteSpace: 'nowrap'
                        }}>
                          {tempState.statusText}
                        </span>
                      </td>
                      <td style={{ padding: '10px', textAlign: 'right', whiteSpace: 'nowrap' }}>
                        <button
                          onClick={() => setInspectPosition(pos)}
                          style={{
                            padding: '4px 8px',
                            borderRadius: '4px',
                            border: '1px solid rgba(56, 189, 248, 0.4)',
                            backgroundColor: 'rgba(56, 189, 248, 0.15)',
                            color: '#38bdf8',
                            fontWeight: 700,
                            fontSize: '10px',
                            cursor: 'pointer',
                            marginRight: '6px'
                          }}
                        >
                          📋 Exit Plan
                        </button>
                        {tempState.isTargetReached && !tempState.isMaxReached && (
                          <button
                            onClick={() => handleContinuePosition(pos)}
                            style={{
                              padding: '4px 8px',
                              borderRadius: '4px',
                              border: '1px solid #f59e0b',
                              backgroundColor: 'rgba(245, 158, 11, 0.25)',
                              color: '#f59e0b',
                              fontWeight: 800,
                              fontSize: '10px',
                              cursor: 'pointer',
                              marginRight: '6px'
                            }}
                          >
                            [ CONTINUE ]
                          </button>
                        )}
                        <button
                          onClick={() => {
                            setSelectedSymbol(pos.symbol);
                            setTicketAction('CLOSE');
                            setTicketQty(pos.quantity.toString());
                          }}
                          style={{
                            padding: '4px 10px',
                            borderRadius: '4px',
                            border: tempState.requiresReview ? '1px solid #ef4444' : '1px solid rgba(239, 68, 68, 0.4)',
                            backgroundColor: tempState.requiresReview ? '#dc2626' : 'rgba(239, 68, 68, 0.15)',
                            color: '#ffffff',
                            fontWeight: 800,
                            fontSize: '10px',
                            cursor: 'pointer',
                            boxShadow: tempState.requiresReview ? '0 0 10px rgba(239, 68, 68, 0.5)' : 'none'
                          }}
                        >
                          [ EXIT NOW ]
                        </button>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          )}
        </div>

        {/* Right Column: Order / Execution Ticket */}
        <div id="paper-order-ticket" style={{
          backgroundColor: '#0f172a',
          border: '1px solid rgba(56, 189, 248, 0.3)',
          borderRadius: '12px',
          padding: '20px',
          boxShadow: '0 10px 25px rgba(0,0,0,0.4)'
        }}>
          <h3 style={{ margin: '0 0 16px 0', fontSize: '16px', fontWeight: 800, color: '#38bdf8', display: 'flex', alignItems: 'center', gap: '8px' }}>
            <span>📝</span> Paper Order Execution Ticket
          </h3>

          {/* Selected Asset Picker */}
          <div style={{ marginBottom: '14px' }}>
            <label style={{ display: 'block', fontSize: '11px', fontWeight: 700, color: '#94a3b8', marginBottom: '6px', textTransform: 'uppercase' }}>
              Selected Instrument
            </label>
            <select
              value={selectedSymbol}
              onChange={(e) => {
                setSelectedSymbol(e.target.value);
                const card = liveCards[e.target.value];
                if (card) {
                  setTicketAction(card.decision_state === 'HOLD' ? 'BUY' : card.decision_state === 'EXIT' ? 'CLOSE' : 'BUY');
                }
              }}
              style={{
                width: '100%',
                padding: '10px 12px',
                borderRadius: '6px',
                backgroundColor: '#1e293b',
                color: '#ffffff',
                border: '1px solid rgba(255,255,255,0.15)',
                fontWeight: 700,
                fontSize: '14px'
              }}
            >
              {availableSymbols.map(sym => (
                <option key={sym} value={sym}>
                  {sym} — Recommendation: {liveCards[sym]?.decision_state || 'WATCH'} ({formatAssetCurrency(liveCards[sym]?.current_price || 0, sym)})
                </option>
              ))}
            </select>
          </div>

          {/* Recommendation & Engine State Grid */}
          <div style={{
            backgroundColor: 'rgba(30, 41, 59, 0.7)',
            border: '1px solid rgba(255, 255, 255, 0.08)',
            borderRadius: '8px',
            padding: '12px 14px',
            marginBottom: '14px',
            display: 'grid',
            gridTemplateColumns: '1fr 1fr',
            gap: '12px'
          }}>
            <div>
              <div style={{ color: '#94a3b8', fontSize: '11px', fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.04em' }}>Recommendation</div>
              <div style={{ fontSize: '15px', fontWeight: 800, color: activeTicketCard?.decision_state === 'HOLD' ? '#10b981' : activeTicketCard?.decision_state === 'EXIT' ? '#ef4444' : '#f59e0b', marginTop: '3px' }}>
                {activeTicketCard?.decision_state || 'WATCH'}
              </div>
            </div>
            <div>
              <div style={{ color: '#94a3b8', fontSize: '11px', fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.04em' }}>Engine state</div>
              <div style={{ fontSize: '12px', fontWeight: 700, color: '#38bdf8', marginTop: '4px', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                {activeTicketCard?.cross_resolution_relationship || 'PARTIAL AGREEMENT'}
              </div>
            </div>
          </div>

          {/* Quant Trade Parameters Panel for Ticket */}
          {(() => {
            const ticketCard = activeTicketCard || getOrGenerateDecisionCard(selectedSymbol, liveCards);
            const overrideSide = ticketAction === 'SELL SHORT' ? 'SHORT' : ticketAction === 'BUY' ? 'LONG' : undefined;
            const q = getQuantTradeParameters(
              selectedSymbol,
              ticketCard.current_price,
              ticketCard.decision_state,
              ticketCard.resolutions['1d']?.score,
              {
                '15m': ticketCard.resolutions['15m']?.score || 0,
                '30m': ticketCard.resolutions['30m']?.score || 0,
                '1d': ticketCard.resolutions['1d']?.score || 0
              },
              overrideSide
            );
            return (
              <div style={{
                backgroundColor: 'rgba(15, 23, 42, 0.9)',
                border: `1px solid ${q.direction === 'SHORT' ? 'rgba(239, 68, 68, 0.4)' : 'rgba(56, 189, 248, 0.4)'}`,
                borderRadius: '8px',
                padding: '12px 14px',
                marginBottom: '14px'
              }}>
                <div style={{ fontSize: '11px', fontWeight: 800, color: '#38bdf8', textTransform: 'uppercase', letterSpacing: '0.04em', marginBottom: '8px', display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                  <span style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                    🎯 Quant Execution Parameters
                    <span style={{ fontSize: '9px', fontWeight: 700, padding: '1px 6px', borderRadius: '3px', backgroundColor: q.direction === 'SHORT' ? 'rgba(239, 68, 68, 0.2)' : 'rgba(16, 185, 129, 0.2)', color: q.direction === 'SHORT' ? '#f87171' : '#34d399' }}>
                      {q.direction} BIAS
                    </span>
                  </span>
                  <span style={{ fontSize: '10px', color: '#f59e0b', fontWeight: 700 }}>{q.horizonDuration}</span>
                </div>
                <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: '8px', fontSize: '11px', fontFamily: 'monospace' }}>
                  <div>
                    <span style={{ color: '#94a3b8', fontSize: '10px', display: 'block' }}>TRIGGER ({q.triggerLabel})</span>
                    <strong style={{ color: '#ffffff' }}>{formatAssetCurrency(q.triggerPrice, selectedSymbol)}</strong>
                  </div>
                  <div>
                    <span style={{ color: '#94a3b8', fontSize: '10px', display: 'block' }}>TARGET ({q.targetPct > 0 ? '+' : ''}{q.targetPct.toFixed(2)}%)</span>
                    <strong style={{ color: q.targetPct > 0 ? '#10b981' : '#f43f5e' }}>{formatAssetCurrency(q.targetPrice, selectedSymbol)}</strong>
                  </div>
                  <div>
                    <span style={{ color: '#94a3b8', fontSize: '10px', display: 'block' }}>STOP ({q.stopLossPct > 0 ? '+' : ''}{q.stopLossPct.toFixed(2)}%)</span>
                    <strong style={{ color: q.stopLossPct < 0 ? '#ef4444' : '#fbbf24' }}>{formatAssetCurrency(q.stopLossPrice, selectedSymbol)}</strong>
                  </div>
                </div>
              </div>
            );
          })()}

          {/* Order Type Selector (MARKET | LIMIT | STOP) */}
          <div style={{ marginBottom: '14px' }}>
            <label style={{ display: 'block', fontSize: '11px', fontWeight: 700, color: '#38bdf8', marginBottom: '6px', textTransform: 'uppercase', letterSpacing: '0.04em' }}>
              Order Type & Execution Mode
            </label>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '6px' }}>
              {(['MARKET', 'LIMIT', 'STOP'] as const).map(oType => {
                const isOSelected = orderType === oType;
                return (
                  <button
                    key={oType}
                    type="button"
                    onClick={() => setOrderType(oType)}
                    style={{
                      padding: '8px 4px',
                      borderRadius: '6px',
                      border: '1px solid ' + (isOSelected ? '#38bdf8' : 'rgba(255,255,255,0.1)'),
                      backgroundColor: isOSelected ? 'rgba(56, 189, 248, 0.2)' : '#1e293b',
                      color: isOSelected ? '#38bdf8' : '#94a3b8',
                      fontWeight: 800,
                      fontSize: '10px',
                      cursor: 'pointer'
                    }}
                  >
                    {oType === 'MARKET' ? '⚡ MARKET' : oType === 'LIMIT' ? '🎯 LIMIT TARGET' : '🛑 STOP TRIGGER'}
                  </button>
                );
              })}
            </div>
          </div>

          {/* Custom Target Entry Execution Price Field */}
          <div style={{ marginBottom: '14px' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '6px' }}>
              <label style={{ fontSize: '11px', fontWeight: 700, color: '#38bdf8', textTransform: 'uppercase', letterSpacing: '0.04em' }}>
                {orderType === 'MARKET' ? 'Execution Spot Price' : orderType === 'LIMIT' ? 'Limit Target Price' : 'Stop Trigger Price'}
              </label>
              <span style={{ fontSize: '10px', color: '#94a3b8' }}>Spot: {formatAssetCurrency(refPrice, selectedSymbol)}</span>
            </div>
            <div style={{ display: 'flex', gap: '6px' }}>
              <input
                type="number"
                step="any"
                value={customEntryPrice}
                onChange={(e) => setCustomEntryPrice(e.target.value)}
                placeholder={String(orderType === 'MARKET' ? refPrice : (activeTicketCard ? getQuantTradeParameters(selectedSymbol, activeTicketCard.current_price, activeTicketCard.decision_state, activeTicketCard.resolutions['1d']?.score, { '15m': activeTicketCard.resolutions['15m']?.score || 0, '30m': activeTicketCard.resolutions['30m']?.score || 0, '1d': activeTicketCard.resolutions['1d']?.score || 0 }, ticketAction === 'SELL SHORT' ? 'SHORT' : 'LONG').triggerPrice : refPrice))}
                style={{
                  flex: 1,
                  padding: '8px 10px',
                  borderRadius: '6px',
                  backgroundColor: '#1e293b',
                  color: '#ffffff',
                  border: '1px solid rgba(56, 189, 248, 0.4)',
                  fontWeight: 800,
                  fontSize: '13px',
                  fontFamily: 'monospace'
                }}
              />
              {(() => {
                const q = activeTicketCard ? getQuantTradeParameters(selectedSymbol, activeTicketCard.current_price, activeTicketCard.decision_state, activeTicketCard.resolutions['1d']?.score, { '15m': activeTicketCard.resolutions['15m']?.score || 0, '30m': activeTicketCard.resolutions['30m']?.score || 0, '1d': activeTicketCard.resolutions['1d']?.score || 0 }, ticketAction === 'SELL SHORT' ? 'SHORT' : 'LONG') : null;
                return (
                  <>
                    {q && (
                      <button
                        type="button"
                        onClick={() => setCustomEntryPrice(String(q.triggerPrice))}
                        style={{ padding: '6px 8px', borderRadius: '4px', backgroundColor: '#0284c7', color: '#fff', border: 'none', fontSize: '10px', fontWeight: 700, cursor: 'pointer', whiteSpace: 'nowrap' }}
                      >
                        Target {formatAssetCurrency(q.triggerPrice, selectedSymbol)}
                      </button>
                    )}
                    <button
                      type="button"
                      onClick={() => setCustomEntryPrice(String(refPrice))}
                      style={{ padding: '6px 8px', borderRadius: '4px', backgroundColor: '#475569', color: '#fff', border: 'none', fontSize: '10px', fontWeight: 700, cursor: 'pointer', whiteSpace: 'nowrap' }}
                    >
                      Spot {formatAssetCurrency(refPrice, selectedSymbol)}
                    </button>
                  </>
                );
              })()}
            </div>
          </div>

          {/* User Action Selector Radio Pills */}
          <div style={{ marginBottom: '14px' }}>
            <label style={{ display: 'block', fontSize: '11px', fontWeight: 700, color: '#94a3b8', marginBottom: '6px', textTransform: 'uppercase', letterSpacing: '0.04em' }}>
              User Action
            </label>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '6px' }}>
              {(['BUY', 'SELL SHORT', 'CLOSE', 'NO ACTION'] as const).map(act => {
                const isActSelected = ticketAction === act;
                const actColor = act === 'BUY' ? '#10b981' : act === 'SELL SHORT' || act === 'CLOSE' ? '#ef4444' : '#cbd5e1';

                return (
                  <button
                    key={act}
                    type="button"
                    onClick={() => setTicketAction(act)}
                    style={{
                      padding: '8px 4px',
                      borderRadius: '6px',
                      border: '1px solid ' + (isActSelected ? actColor : 'rgba(255,255,255,0.1)'),
                      backgroundColor: isActSelected ? (act === 'NO ACTION' ? '#334155' : actColor + '25') : '#1e293b',
                      color: isActSelected ? (act === 'NO ACTION' ? '#ffffff' : actColor) : '#94a3b8',
                      fontWeight: 800,
                      fontSize: '10px',
                      cursor: 'pointer',
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      gap: '3px'
                    }}
                  >
                    <span style={{ fontSize: '9px' }}>{isActSelected ? '●' : '○'}</span>
                    <span>{act}</span>
                  </button>
                );
              })}
            </div>
          </div>

          {/* Trade Exit Policy Configurator */}
          <div style={{
            backgroundColor: 'rgba(30, 41, 59, 0.5)',
            border: '1px solid rgba(255, 255, 255, 0.08)',
            borderRadius: '8px',
            padding: '12px',
            marginBottom: '14px'
          }}>
            <label style={{ display: 'block', fontSize: '11px', fontWeight: 700, color: '#38bdf8', marginBottom: '8px', textTransform: 'uppercase', letterSpacing: '0.04em' }}>
              ⏱️ Trade Exit Policy (Temporal Holding Window)
            </label>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '6px', marginBottom: '10px' }}>
              {(['FIXED_HORIZON', 'RESOLUTION_WINDOW', 'MANUAL'] as const).map(pType => {
                const isPSelected = exitPolicyType === pType;
                return (
                  <button
                    key={pType}
                    type="button"
                    onClick={() => setExitPolicyType(pType)}
                    style={{
                      padding: '6px 4px',
                      borderRadius: '4px',
                      border: '1px solid ' + (isPSelected ? '#38bdf8' : 'rgba(255,255,255,0.1)'),
                      backgroundColor: isPSelected ? 'rgba(56, 189, 248, 0.2)' : '#1e293b',
                      color: isPSelected ? '#38bdf8' : '#94a3b8',
                      fontWeight: 700,
                      fontSize: '10px',
                      cursor: 'pointer'
                    }}
                  >
                    {pType === 'FIXED_HORIZON' ? 'Fixed Horizon' : pType === 'RESOLUTION_WINDOW' ? 'Res Window' : 'Manual'}
                  </button>
                );
              })}
            </div>

            {exitPolicyType === 'FIXED_HORIZON' && (
              <div>
                <div style={{ fontSize: '10px', color: '#94a3b8', marginBottom: '4px', fontWeight: 600 }}>Target Holding Horizon (Max = 3x Target):</div>
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(6, 1fr)', gap: '4px' }}>
                  {[15, 30, 60, 120, 240, 1440].map(mins => (
                    <button
                      key={mins}
                      type="button"
                      onClick={() => setExitDurationMinutes(mins)}
                      style={{
                        padding: '4px 2px',
                        borderRadius: '4px',
                        border: '1px solid ' + (exitDurationMinutes === mins ? '#10b981' : 'rgba(255,255,255,0.1)'),
                        backgroundColor: exitDurationMinutes === mins ? 'rgba(16, 185, 129, 0.2)' : '#1e293b',
                        color: exitDurationMinutes === mins ? '#10b981' : '#cbd5e1',
                        fontWeight: 700,
                        fontSize: '10px',
                        cursor: 'pointer'
                      }}
                    >
                      {mins >= 60 ? `${mins / 60}h` : `${mins}m`}
                    </button>
                  ))}
                </div>
              </div>
            )}

            {exitPolicyType === 'RESOLUTION_WINDOW' && (
              <div>
                <div style={{ fontSize: '10px', color: '#94a3b8', marginBottom: '4px', fontWeight: 600 }}>Select Explicit Resolution Target:</div>
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '6px' }}>
                  {(['15m', '30m', '1d'] as const).map(resChoice => (
                    <button
                      key={resChoice}
                      type="button"
                      onClick={() => setExitResolutionChoice(resChoice)}
                      style={{
                        padding: '6px 4px',
                        borderRadius: '4px',
                        border: '1px solid ' + (exitResolutionChoice === resChoice ? '#10b981' : 'rgba(255,255,255,0.1)'),
                        backgroundColor: exitResolutionChoice === resChoice ? 'rgba(16, 185, 129, 0.2)' : '#1e293b',
                        color: exitResolutionChoice === resChoice ? '#10b981' : '#cbd5e1',
                        fontWeight: 700,
                        fontSize: '11px',
                        cursor: 'pointer'
                      }}
                    >
                      {resChoice === '15m' ? '15m (45m/3h)' : resChoice === '30m' ? '30m (90m/6h)' : '1d (1d/3d)'}
                    </button>
                  ))}
                </div>
              </div>
            )}

            {exitPolicyType === 'MANUAL' && (
              <div style={{ fontSize: '11px', color: '#94a3b8', backgroundColor: '#1e293b', padding: '8px 10px', borderRadius: '4px', textAlign: 'center' }}>
                <span style={{ color: '#f59e0b', fontWeight: 700 }}>⚠ NO TEMPORAL DEADLINE</span>
                <div style={{ fontSize: '10px', marginTop: '2px' }}>Open-ended holding window. Requires manual closure by operator.</div>
              </div>
            )}
          </div>

          {/* Quantity Input */}
          <div style={{ marginBottom: '14px' }}>
            <label style={{ display: 'block', fontSize: '11px', fontWeight: 700, color: '#94a3b8', marginBottom: '6px', textTransform: 'uppercase' }}>
              Order Quantity
            </label>
            <input
              type="number"
              step="any"
              value={ticketQty}
              onChange={(e) => setTicketQty(e.target.value)}
              style={{
                width: '100%',
                padding: '10px 12px',
                borderRadius: '6px',
                backgroundColor: '#1e293b',
                color: '#ffffff',
                border: '1px solid rgba(255,255,255,0.15)',
                fontWeight: 700,
                fontSize: '14px',
                fontFamily: 'monospace'
              }}
            />
          </div>

          {/* Pricing & Cost Calculation */}
          <div style={{
            backgroundColor: '#090d16',
            borderRadius: '8px',
            padding: '12px',
            marginBottom: '16px',
            fontSize: '12px'
          }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '4px', color: '#94a3b8' }}>
              <span>Reference Price:</span>
              <strong style={{ color: '#38bdf8', fontFamily: 'monospace' }}>{formatAssetCurrency(refPrice, selectedSymbol)}</strong>
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between', color: '#f8fafc', fontWeight: 700 }}>
              <span>Estimated Cost / Value:</span>
              <strong style={{ color: '#ffffff', fontFamily: 'monospace', fontSize: '14px' }}>{formatAssetCurrency(estimatedCost, selectedSymbol)}</strong>
            </div>
          </div>

          {/* Execution Status Message */}
          {ticketStatusMsg && (
            <div style={{
              padding: '10px 12px',
              borderRadius: '6px',
              fontSize: '12px',
              fontWeight: 600,
              marginBottom: '14px',
              backgroundColor: ticketStatusMsg.isError ? 'rgba(239, 68, 68, 0.15)' : 'rgba(16, 185, 129, 0.15)',
              color: ticketStatusMsg.isError ? '#ef4444' : '#10b981',
              border: `1px solid ${ticketStatusMsg.isError ? 'rgba(239, 68, 68, 0.3)' : 'rgba(16, 185, 129, 0.3)'}`
            }}>
              {ticketStatusMsg.text}
            </div>
          )}

          {/* Submit Button */}
          <button
            onClick={handleExecutePaperOrder}
            style={{
              width: '100%',
              padding: '12px',
              borderRadius: '8px',
              border: 'none',
              backgroundColor: ticketAction === 'BUY' ? '#0284c7' : ticketAction === 'NO ACTION' ? '#334155' : '#dc2626',
              color: '#ffffff',
              fontWeight: 800,
              fontSize: '14px',
              cursor: 'pointer',
              boxShadow: '0 4px 14px rgba(0,0,0,0.4)',
              transition: 'all 0.2s ease'
            }}
          >
            ⚡ PAPER EXECUTE ORDER ({ticketAction})
          </button>
        </div>
      </div>

      {/* 4. TRADE JOURNAL & DECISION PROVENANCE LINKAGE */}
      <div style={{
        backgroundColor: '#0f172a',
        border: '1px solid rgba(255, 255, 255, 0.1)',
        borderRadius: '12px',
        padding: '20px',
        marginBottom: '28px'
      }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
          <div>
            <h3 style={{ margin: 0, fontSize: '16px', fontWeight: 800, color: '#f8fafc' }}>
              📜 Simulated Trade Journal & Immutable Provenance Audit
            </h3>
            <span style={{ fontSize: '12px', color: '#94a3b8' }}>
              Every trade stores an immutable DecisionCard snapshot & exit policy captured at execution time
            </span>
          </div>
          <div style={{ fontSize: '13px', fontWeight: 800, color: realizedPnl >= 0 ? '#10b981' : '#ef4444' }}>
            Total Realized P&L: {realizedPnl >= 0 ? '+' : ''}${realizedPnl.toFixed(2)}
          </div>
        </div>

        <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left', fontSize: '13px' }}>
          <thead>
            <tr style={{ backgroundColor: '#1e293b', color: '#94a3b8', borderBottom: '1px solid rgba(255,255,255,0.1)' }}>
              <th style={{ padding: '10px 12px' }}>Trade ID</th>
              <th style={{ padding: '10px 12px' }}>Date & Time (UTC / Local)</th>
              <th style={{ padding: '10px 12px' }}>Source</th>
              <th style={{ padding: '10px 12px' }}>Asset</th>
              <th style={{ padding: '10px 12px' }}>User Action</th>
              <th style={{ padding: '10px 12px' }}>Qty</th>
              <th style={{ padding: '10px 12px' }}>Price</th>
              <th style={{ padding: '10px 12px' }}>Exit Policy</th>
              <th style={{ padding: '10px 12px', textAlign: 'right' }}>Provenance Audit</th>
            </tr>
          </thead>
          <tbody>
            {tradeJournal.map(entry => {
              const actColor = entry.user_action === 'BUY' ? '#10b981' : entry.user_action === 'SELL' || entry.user_action === 'CLOSE' ? '#ef4444' : '#94a3b8';
              const policyDesc = entry.exit_policy?.description || 'FIXED TIME (60m)';

              return (
                <tr key={entry.trade_id} style={{ borderBottom: '1px solid rgba(255,255,255,0.05)' }}>
                  <td style={{ padding: '12px', fontWeight: 700, fontFamily: 'monospace', color: '#38bdf8' }}>{entry.trade_id}</td>
                  <td style={{ padding: '12px', color: '#94a3b8', fontSize: '11px', fontFamily: 'monospace' }}>{formatDualTime(entry.timestamp_utc)}</td>
                  <td style={{ padding: '12px' }}>
                    <span style={{
                      fontSize: '10px',
                      fontWeight: 700,
                      padding: '2px 6px',
                      borderRadius: '4px',
                      backgroundColor: 'rgba(56, 189, 248, 0.15)',
                      color: '#38bdf8',
                      border: '1px solid rgba(56, 189, 248, 0.3)'
                    }}>
                      {entry.decision_source}
                    </span>
                  </td>
                  <td style={{ padding: '12px', fontWeight: 800, color: '#ffffff' }}>{entry.symbol}</td>
                  <td style={{ padding: '12px' }}>
                    <span style={{
                      padding: '3px 8px',
                      borderRadius: '4px',
                      fontSize: '11px',
                      fontWeight: 800,
                      backgroundColor: actColor + '20',
                      color: actColor,
                      border: `1px solid ${actColor}40`
                    }}>
                      {entry.user_action}
                    </span>
                  </td>
                  <td style={{ padding: '12px', fontFamily: 'monospace' }}>
                    {typeof (entry.execution?.quantity ?? entry.entry?.quantity) === 'number'
                      ? (entry.execution?.quantity ?? entry.entry?.quantity)?.toLocaleString('en-US')
                      : (entry.execution?.quantity ?? entry.entry?.quantity ?? '—')}
                  </td>
                  <td style={{ padding: '12px', fontFamily: 'monospace', color: '#ffffff' }}>
                    {formatAssetCurrency(entry.execution?.reference_price ?? entry.execution?.fill_price ?? entry.entry?.price ?? 0, entry.symbol)}
                  </td>
                  <td style={{ padding: '12px', color: '#cbd5e1', fontSize: '11px' }}>
                    {policyDesc}
                  </td>
                  <td style={{ padding: '12px', textAlign: 'right' }}>
                    <button
                      onClick={() => setInspectEntry(entry)}
                      style={{
                        padding: '4px 10px',
                        borderRadius: '4px',
                        border: '1px solid rgba(56, 189, 248, 0.4)',
                        backgroundColor: 'rgba(56, 189, 248, 0.15)',
                        color: '#38bdf8',
                        fontWeight: 700,
                        fontSize: '11px',
                        cursor: 'pointer'
                      }}
                    >
                      🔍 View Provenance
                    </button>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      {/* 5. SECONDARY PERFORMANCE METRICS */}
      <div style={{
        backgroundColor: '#0f172a',
        border: '1px solid rgba(255, 255, 255, 0.08)',
        borderRadius: '12px',
        padding: '20px',
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(160px, 1fr))',
        gap: '16px'
      }}>
        <div>
          <div style={{ fontSize: '11px', fontWeight: 700, color: '#94a3b8', textTransform: 'uppercase' }}>Portfolio Return</div>
          <div style={{ fontSize: '18px', fontWeight: 800, color: portfolioReturnPct >= 0 ? '#10b981' : '#ef4444', marginTop: '2px' }}>
            {portfolioReturnPct >= 0 ? '+' : ''}{portfolioReturnPct.toFixed(2)}%
          </div>
        </div>
        <div>
          <div style={{ fontSize: '11px', fontWeight: 700, color: '#94a3b8', textTransform: 'uppercase' }}>Win Rate</div>
          <div style={{ fontSize: '18px', fontWeight: 800, color: '#38bdf8', marginTop: '2px' }}>75.0%</div>
        </div>
        <div>
          <div style={{ fontSize: '11px', fontWeight: 700, color: '#94a3b8', textTransform: 'uppercase' }}>Total Trades</div>
          <div style={{ fontSize: '18px', fontWeight: 800, color: '#ffffff', marginTop: '2px' }}>{tradeJournal.length}</div>
        </div>
        <div>
          <div style={{ fontSize: '11px', fontWeight: 700, color: '#94a3b8', textTransform: 'uppercase' }}>Max Drawdown</div>
          <div style={{ fontSize: '18px', fontWeight: 800, color: '#ef4444', marginTop: '2px' }}>-0.42%</div>
        </div>
      </div>

      {/* 6. POSITION EXIT PLAN INSPECTION MODAL */}
      {inspectPosition && (
        <div style={{
          position: 'fixed',
          top: 0, left: 0, right: 0, bottom: 0,
          backgroundColor: 'rgba(0, 0, 0, 0.85)',
          backdropFilter: 'blur(8px)',
          display: 'flex', justifyContent: 'center', alignItems: 'center',
          zIndex: 9999, padding: '20px'
        }}>
          <div style={{
            backgroundColor: '#0f172a',
            border: '1px solid #38bdf8',
            borderRadius: '14px',
            maxWidth: '520px',
            width: '100%',
            padding: '24px',
            boxShadow: '0 25px 50px -12px rgba(0, 0, 0, 0.8)',
            color: '#f8fafc'
          }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px', borderBottom: '1px solid rgba(255,255,255,0.1)', paddingBottom: '12px' }}>
              <div>
                <h3 style={{ margin: 0, fontSize: '18px', color: '#38bdf8', fontWeight: 800 }}>
                  ⏱️ Trade Exit Policy & Temporal Obligation
                </h3>
                <span style={{ fontSize: '12px', color: '#94a3b8' }}>
                  {inspectPosition.symbol} — Active Position Exit Plan
                </span>
              </div>
              <button
                onClick={() => setInspectPosition(null)}
                style={{ backgroundColor: '#1e293b', color: '#94a3b8', border: 'none', borderRadius: '6px', padding: '6px 12px', cursor: 'pointer', fontWeight: 700 }}
              >
                ✕ Close
              </button>
            </div>

            {/* Entry Box */}
            <div style={{ backgroundColor: '#1e293b', padding: '14px', borderRadius: '8px', marginBottom: '14px' }}>
              <div style={{ fontSize: '11px', color: '#94a3b8', fontWeight: 700, textTransform: 'uppercase' }}>ENTRY DETAILS</div>
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '8px', marginTop: '6px', fontSize: '13px' }}>
                <div>Entry Price: <strong style={{ color: '#ffffff', fontFamily: 'monospace' }}>{formatAssetCurrency(inspectPosition.entry_price, inspectPosition.symbol)}</strong></div>
                <div>Entry Time: <strong style={{ color: '#ffffff', fontFamily: 'monospace' }}>{formatDualTime(inspectPosition.entry_time_utc)}</strong></div>
                <div>Decision at Entry: <strong style={{ color: '#10b981' }}>{inspectPosition.decision_state_at_entry}</strong></div>
                <div>Current P&L: <strong style={{ color: inspectPosition.unrealized_pnl >= 0 ? '#10b981' : '#ef4444' }}>{inspectPosition.unrealized_pnl >= 0 ? '+' : ''}{formatAssetCurrency(inspectPosition.unrealized_pnl, inspectPosition.symbol)}</strong></div>
              </div>
            </div>

            {/* Exit Plan Box */}
            <div style={{ backgroundColor: '#090d16', border: '1px solid rgba(255,255,255,0.08)', padding: '14px', borderRadius: '8px', marginBottom: '14px' }}>
              <div style={{ fontSize: '11px', color: '#38bdf8', fontWeight: 700, textTransform: 'uppercase', marginBottom: '6px' }}>EXIT PLAN</div>
              <div style={{ fontSize: '13px', lineHeight: '1.6', color: '#cbd5e1' }}>
                <div>Policy: <strong style={{ color: '#ffffff' }}>{inspectPosition.exit_policy?.description || 'FIXED TIME'}</strong></div>
                <div>Target Exit: <strong style={{ color: '#f59e0b', fontFamily: 'monospace' }}>{inspectPosition.exit_policy?.target_exit_utc ? formatDualTime(inspectPosition.exit_policy.target_exit_utc) : 'OPEN'}</strong></div>
                <div>Maximum Exit Deadline: <strong style={{ color: '#ef4444', fontFamily: 'monospace' }}>{inspectPosition.exit_policy?.maximum_exit_utc ? formatDualTime(inspectPosition.exit_policy.maximum_exit_utc) : (inspectPosition.exit_policy as any)?.max_exit_utc ? formatDualTime((inspectPosition.exit_policy as any).max_exit_utc) : 'OPEN'}</strong></div>
              </div>
            </div>

            {/* Countdown State Box */}
            {(() => {
              const temp = getPositionTemporalState(inspectPosition, nowUtcMs);
              return (
                <div style={{ backgroundColor: temp.badgeColor + '15', border: `1px solid ${temp.badgeColor}40`, padding: '14px', borderRadius: '8px', marginBottom: '18px', textAlign: 'center' }}>
                  <div style={{ fontSize: '11px', color: temp.badgeColor, fontWeight: 700, textTransform: 'uppercase' }}>TIME REMAINING</div>
                  <div style={{ fontSize: '24px', fontWeight: 800, color: temp.badgeColor, marginTop: '2px', fontFamily: 'monospace' }}>
                    {temp.formattedTimeRemaining}
                  </div>
                  <div style={{ fontSize: '12px', fontWeight: 700, color: '#f8fafc', marginTop: '4px' }}>
                    {temp.statusText}
                  </div>
                </div>
              );
            })()}

            {/* Modal Actions */}
            {(() => {
              const temp = getPositionTemporalState(inspectPosition, nowUtcMs);
              return (
                <div style={{ display: 'flex', gap: '10px' }}>
                  {temp.isTargetReached && !temp.isMaxReached && (
                    <button
                      onClick={() => handleContinuePosition(inspectPosition)}
                      style={{
                        flex: 1, padding: '12px', borderRadius: '8px', border: '1px solid #f59e0b',
                        backgroundColor: 'rgba(245, 158, 11, 0.25)', color: '#f59e0b', fontWeight: 800, fontSize: '13px', cursor: 'pointer'
                      }}
                    >
                      [ CONTINUE TO MAX EXIT ]
                    </button>
                  )}
                  <button
                    onClick={() => {
                      setSelectedSymbol(inspectPosition.symbol);
                      setTicketAction('CLOSE');
                      setTicketQty(inspectPosition.quantity.toString());
                      setInspectPosition(null);
                    }}
                    style={{
                      flex: 1, padding: '12px', borderRadius: '8px', border: 'none',
                      backgroundColor: '#dc2626', color: '#ffffff', fontWeight: 800, fontSize: '13px', cursor: 'pointer',
                      boxShadow: '0 4px 12px rgba(220, 38, 38, 0.4)'
                    }}
                  >
                    [ EXIT NOW (SIMULATE CLOSE) ]
                  </button>
                </div>
              );
            })()}
          </div>
        </div>
      )}

      {/* 7. PROVENANCE INSPECTION MODAL (FULL IMMUTABLE DECISION SNAPSHOT) */}
      {inspectEntry && (
        <div style={{
          position: 'fixed',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          backgroundColor: 'rgba(0, 0, 0, 0.85)',
          backdropFilter: 'blur(8px)',
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          zIndex: 9999,
          padding: '20px'
        }}>
          <div style={{
            backgroundColor: '#0f172a',
            border: '1px solid #38bdf8',
            borderRadius: '14px',
            maxWidth: '720px',
            width: '100%',
            padding: '26px',
            boxShadow: '0 25px 50px -12px rgba(0, 0, 0, 0.8)',
            color: '#f8fafc',
            maxHeight: '90vh',
            overflowY: 'auto'
          }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '18px', borderBottom: '1px solid rgba(255,255,255,0.1)', paddingBottom: '14px' }}>
              <div>
                <h3 style={{ margin: 0, fontSize: '20px', color: '#38bdf8', fontWeight: 800 }}>
                  🔍 Immutable Decision Provenance: {inspectEntry.trade_id}
                </h3>
                <span style={{ fontSize: '12px', color: '#94a3b8' }}>
                  Point-in-Time Evidence Captured at Execution ({inspectEntry.decision_source})
                </span>
              </div>
              <button
                onClick={() => setInspectEntry(null)}
                style={{
                  backgroundColor: '#1e293b',
                  color: '#94a3b8',
                  border: 'none',
                  borderRadius: '6px',
                  padding: '6px 12px',
                  cursor: 'pointer',
                  fontWeight: 700
                }}
              >
                ✕ Close
              </button>
            </div>

            {/* Execution vs Recommendation Summary */}
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: '12px', marginBottom: '18px', backgroundColor: '#1e293b', padding: '14px', borderRadius: '8px' }}>
              <div>
                <div style={{ fontSize: '11px', color: '#94a3b8', fontWeight: 700 }}>USER ACTION</div>
                <div style={{ fontSize: '16px', fontWeight: 800, color: inspectEntry.user_action === 'BUY' ? '#10b981' : '#ef4444', marginTop: '2px' }}>
                  {inspectEntry.user_action}
                </div>
              </div>
              <div>
                <div style={{ fontSize: '11px', color: '#94a3b8', fontWeight: 700 }}>ENGINE RECOMMENDATION</div>
                <div style={{ fontSize: '16px', fontWeight: 800, color: inspectEntry.decision_snapshot.decision_state === 'HOLD' ? '#10b981' : inspectEntry.decision_snapshot.decision_state === 'EXIT' ? '#ef4444' : '#f59e0b', marginTop: '2px' }}>
                  {inspectEntry.decision_snapshot.decision_state}
                </div>
              </div>
              <div>
                <div style={{ fontSize: '11px', color: '#94a3b8', fontWeight: 700 }}>EXIT POLICY</div>
                <div style={{ fontSize: '13px', fontWeight: 800, color: '#38bdf8', marginTop: '2px' }}>
                  {inspectEntry.exit_policy?.description || 'FIXED TIME'}
                </div>
              </div>
            </div>

            {/* Immutable Decision Snapshot Details */}
            <div style={{ backgroundColor: '#090d16', padding: '16px', borderRadius: '10px', marginBottom: '18px', border: '1px solid rgba(255,255,255,0.08)' }}>
              <div style={{ fontSize: '12px', fontWeight: 700, color: '#38bdf8', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '10px' }}>
                IMMUTABLE DECISIONCARD SNAPSHOT ({inspectEntry.decision_card_id || inspectEntry.decision_snapshot?.card_id || 'CARD-LIVE'})
              </div>
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px', fontSize: '13px', marginBottom: '12px' }}>
                <div>Price at Fill: <strong style={{ color: '#ffffff', fontFamily: 'monospace' }}>{formatAssetCurrency(inspectEntry.execution?.fill_price ?? inspectEntry.entry?.price ?? 0, inspectEntry.symbol)}</strong></div>
                <div>Execution Qty: <strong style={{ color: '#ffffff', fontFamily: 'monospace' }}>{inspectEntry.execution?.quantity ?? inspectEntry.entry?.quantity ?? 1} units</strong></div>
                <div>Cross-Resolution Alignment: <strong style={{ color: '#38bdf8' }}>{inspectEntry.decision_snapshot?.cross_resolution_relationship || 'INTRADAY ALIGNMENT'}</strong></div>
                <div>Risk State: <strong style={{ color: '#f59e0b' }}>{inspectEntry.decision_snapshot?.risk_state || 'TRAJECTORY STABLE'}</strong></div>
              </div>

              {/* Multi-Horizon Score Breakdown */}
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '10px', marginTop: '12px' }}>
                {(['15m', '30m', '1d'] as const).map(h => {
                  const label = inspectEntry.decision_snapshot?.resolutions?.[h]?.label || 'PULLBACK';
                  const score = inspectEntry.decision_snapshot?.resolutions?.[h]?.score ?? 0.0;
                  return (
                    <div key={h} style={{ backgroundColor: '#1e293b', padding: '10px', borderRadius: '6px', textAlign: 'center' }}>
                      <div style={{ fontSize: '10px', color: '#94a3b8', fontWeight: 700 }}>HORIZON {h}</div>
                      <div style={{ fontSize: '14px', fontWeight: 800, color: label === 'CONTINUATION' ? '#10b981' : label === 'REVERSAL' ? '#ef4444' : '#f59e0b', marginTop: '2px' }}>
                        {label}
                      </div>
                      <div style={{ fontSize: '11px', color: '#64748b', fontFamily: 'monospace' }}>
                        {score >= 0 ? '+' : ''}{score.toFixed(4)}
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>

            {/* Why Did I Make This Trade Evidence */}
            <div style={{ backgroundColor: '#090d16', padding: '16px', borderRadius: '10px', border: '1px solid rgba(255,255,255,0.08)' }}>
              <div style={{ fontSize: '12px', color: '#94a3b8', fontWeight: 700, letterSpacing: '0.05em' }}>
                WHY DID I MAKE THIS TRADE? (CANONICAL EVIDENCE AT ENTRY):
              </div>
              <ul style={{ margin: '10px 0 0 0', paddingLeft: '20px', fontSize: '13px', color: '#cbd5e1', lineHeight: '1.6' }}>
                {inspectEntry.decision_snapshot.evidence.map((b, idx) => (
                  <li key={idx}>{b.startsWith('• ') ? b.substring(2) : b}</li>
                ))}
              </ul>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default Gate5PaperDashboard;
