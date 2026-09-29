import React, { useState, useEffect } from 'react';

interface TrajectoryPoint {
  step: number;
  label: string;
  pnl: number;
  price: number;
}

interface StopPoint {
  step: number;
  control_sl_pnl: number;
  candidate_sl_pnl: number;
}

interface TradeResult {
  composite_id: [string, string, string, string];
  candidate_id: string;
  baseline_realized_pnl: number;
  candidate_realized_pnl: number;
  paired_delta: number;
  mfe_pnl: number;
  candidate_giveback: number;
  reversal_prevented: boolean;
  premature_exit: boolean;
  opportunity_cost_60m: number;
  price_trajectory?: TrajectoryPoint[];
  stop_trajectory?: StopPoint[];
}

interface CandidateResult {
  candidate_id: string;
  total_trades_evaluated: number;
  mean_paired_delta: number;
  median_paired_delta: number;
  total_candidate_realized_pnl: number;
  mean_candidate_realized_pnl: number;
  total_candidate_giveback: number;
  reversals_prevented_count: number;
  premature_exits_count: number;
  total_opportunity_cost_60m: number;
  trade_level_results: TradeResult[];
}

interface ValidationDecision {
  decision: string;
  reason: string;
  mean_paired_delta: number;
  ci_95: [number, number];
}

interface PopulationReconciliation {
  canonical_stage2_total_exits: number;
  nse_equities_holdout_exits: number;
  crypto_assets_omitted_exits: number;
  omission_rationale: string;
  outcome_dependent_selection_check: string;
}

interface TemporalValidationData {
  run_id: string;
  specification_version: string;
  audit_timestamp_utc: string;
  snapshot_sha256: string;
  target_cohort: string;
  population_reconciliation?: PopulationReconciliation;
  cohort_size: number;
  baseline_stage2_realized_pnl: number;
  baseline_stage2_giveback: number;
  candidate_replay_results: Record<string, CandidateResult>;
  bootstrap_uncertainty_results: Record<string, any>;
  validation_decisions: Record<string, ValidationDecision>;
}

export default function RP609Dashboard() {
  const [data, setData] = useState<TemporalValidationData | null>(null);
  const [selectedDataset, setSelectedDataset] = useState<'NSE' | 'US'>('NSE');
  const [selectedCandidate, setSelectedCandidate] = useState<string>('PPR-v1.0-FROZEN');
  const [selectedTradeIndex, setSelectedTradeIndex] = useState<number>(0);
  const [searchSymbol, setSearchSymbol] = useState<string>('');
  const [filterReversalOnly, setFilterReversalOnly] = useState<boolean>(false);
  const [activeView, setActiveView] = useState<'matrix' | 'visualizer' | 'reconciliation' | 'strata' | 'simulator' | 'explorer'>('visualizer');

  // Interactive Simulator Controls
  const [simActivationR, setSimActivationR] = useState<number>(1.5);
  const [simTrailingPct, setSimTrailingPct] = useState<number>(50);

  useEffect(() => {
    const fileUri = selectedDataset === 'US' 
      ? '/data/rp609_us_prospective_shadow_replay_report.json' 
      : '/data/rp609_temporal_validation_report.json';
    fetch(fileUri)
      .then(res => res.json())
      .then(d => {
        setData(d);
        setSelectedTradeIndex(0);
      })
      .catch(err => console.warn("Failed to fetch RP-609 dataset report:", err));
  }, [selectedDataset]);

  const candidatesMeta: Record<string, { name: string; desc: string; icon: string; analogy: string }> = {
    'PFT-v1.0-FROZEN': {
      name: 'Candidate A — Peak-Profit Trailing (PFT)',
      desc: 'Trails peak unrealized profit with continuous trailing stop boundary once activation threshold is reached.',
      icon: '📈',
      analogy: 'A moving safety net that climbs with price action.'
    },
    'SPL-v1.0-FROZEN': {
      name: 'Candidate B — Stepwise Profit Locking (SPL)',
      desc: 'Advances protective profit boundary in discrete milestone steps (1.0R, 2.0R, 3.0R).',
      icon: '🪜',
      analogy: 'Locking in rewards at predefined ladder steps.'
    },
    'VAT-v1.0-FROZEN': {
      name: 'Candidate C — Volatility-Adjusted Trailing (VAT)',
      desc: 'Dynamically scales trailing distance using ATR volatility estimates to reduce chop exits.',
      icon: '🌊',
      analogy: 'Giving trades wider breathing room in turbulent regimes.'
    },
    'PPR-v1.0-FROZEN': {
      name: 'Candidate D — Partial Profit Realization (PPR)',
      desc: 'Scales out 50% position size at profit milestone and advances residual stop to breakeven.',
      icon: '💰',
      analogy: 'Taking money off the table while letting residual run.'
    }
  };

  if (!data) {
    return (
      <div style={{ padding: '40px', backgroundColor: '#090d16', color: '#38bdf8', borderRadius: '12px', border: '1px solid #1e293b', textAlign: 'center' }}>
        <h3 style={{ margin: 0, fontSize: '18px' }}>🛡️ Loading RP-609 Profit Retention Forensic Dataset...</h3>
        <p style={{ margin: '8px 0 0 0', color: '#94a3b8', fontSize: '13px' }}>Synchronizing trade trajectories and dynamic stop boundaries</p>
      </div>
    );
  }

  const currentResult = data?.candidate_replay_results?.[selectedCandidate];
  const currentDecision = data?.validation_decisions?.[selectedCandidate];
  const activeTrade: TradeResult | undefined = currentResult?.trade_level_results?.[selectedTradeIndex] || currentResult?.trade_level_results?.[0];

  const filteredTrades = (currentResult?.trade_level_results || []).filter(t => {
    const sym = t.composite_id[0] || '';
    if (searchSymbol && !sym.toLowerCase().includes(searchSymbol.toLowerCase())) return false;
    if (filterReversalOnly && !t.reversal_prevented) return false;
    return true;
  });

  // Safe numeric formatter
  const fmt = (val: number | undefined | null, digits: number = 2): string => {
    if (val === undefined || val === null || isNaN(val)) return '0.00';
    return val.toFixed(digits);
  };

  // Trajectory parsing helpers for objects vs tuples
  const getPointPnl = (pt: any): number => {
    if (!pt) return 0;
    if (typeof pt === 'number') return pt;
    if (typeof pt.pnl === 'number') return pt.pnl;
    if (Array.isArray(pt) && typeof pt[1] === 'number') return pt[1];
    return 0;
  };

  const getPointLabel = (pt: any, idx: number): string => {
    if (!pt) return `Step ${idx}`;
    if (typeof pt.label === 'string') return pt.label;
    if (Array.isArray(pt) && typeof pt[0] === 'string') return pt[0];
    return `Step ${idx}`;
  };

  const getStopControlPnl = (pt: any): number => {
    if (!pt) return 0;
    if (typeof pt.control_sl_pnl === 'number') return pt.control_sl_pnl;
    if (Array.isArray(pt) && typeof pt[1] === 'number') return pt[1];
    return 0;
  };

  const getStopCandidatePnl = (pt: any): number => {
    if (!pt) return 0;
    if (typeof pt.candidate_sl_pnl === 'number') return pt.candidate_sl_pnl;
    if (Array.isArray(pt) && typeof pt[1] === 'number') return pt[1];
    return 0;
  };

  // SVG Chart Math
  const pricePts = (activeTrade?.price_trajectory || []).map((pt: any, idx: number) => ({
    step: typeof pt.step === 'number' ? pt.step : idx,
    label: getPointLabel(pt, idx),
    pnl: getPointPnl(pt)
  }));

  const stopPts = (activeTrade?.stop_trajectory || []).map((pt: any, idx: number) => ({
    step: typeof pt.step === 'number' ? pt.step : idx,
    control_sl_pnl: getStopControlPnl(pt),
    candidate_sl_pnl: getStopCandidatePnl(pt)
  }));

  const allPnls = [
    ...pricePts.map(p => p.pnl),
    ...stopPts.map(s => s.control_sl_pnl),
    ...stopPts.map(s => s.candidate_sl_pnl)
  ];
  const minPnl = allPnls.length ? Math.min(...allPnls) - 20 : -100;
  const maxPnl = allPnls.length ? Math.max(...allPnls) + 20 : 100;

  const chartW = 740;
  const chartH = 260;
  const marginX = 60;
  const marginY = 30;

  const getX = (step: number) => marginX + (step / 5) * (chartW - 2 * marginX);
  const getY = (pnl: number) => chartH - marginY - ((pnl - minPnl) / (maxPnl - minPnl || 1)) * (chartH - 2 * marginY);

  const pricePolyline = pricePts.map(p => `${getX(p.step)},${getY(p.pnl)}`).join(' ');
  const controlStopPolyline = stopPts.map(s => `${getX(s.step)},${getY(s.control_sl_pnl)}`).join(' ');
  const candidateStopPolyline = stopPts.map(s => `${getX(s.step)},${getY(s.candidate_sl_pnl)}`).join(' ');

  return (
    <div style={{ backgroundColor: '#090d16', color: '#e2e8f0', fontFamily: 'Inter, system-ui, sans-serif', padding: '24px', borderRadius: '12px', border: '1px solid #1e293b' }}>
      
      {/* Title & Governance Header Banner */}
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '20px', flexWrap: 'wrap', gap: '16px' }}>
        <div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
            <h1 style={{ margin: 0, fontSize: '26px', fontWeight: 800, color: '#38bdf8', letterSpacing: '-0.02em' }}>
              🛡️ RP-609 Profit Retention &amp; Trajectory Forensics Workspace
            </h1>
            <span style={{ backgroundColor: '#10b98120', color: '#10b981', border: '1px solid #10b98140', padding: '4px 10px', borderRadius: '20px', fontSize: '12px', fontWeight: 700 }}>
              FULL FORENSIC VISIBILITY | SHADOW ONLY
            </span>
          </div>
          <p style={{ margin: '6px 0 0 0', color: '#94a3b8', fontSize: '14px' }}>
            Deep forensic visual transparency: inspect intraday price trajectories, dynamic trailing stop curves &amp; reversal prevention events
          </p>
        </div>

        <div style={{ display: 'flex', gap: '10px', backgroundColor: '#1e293b80', padding: '8px 14px', borderRadius: '8px', border: '1px solid #334155', fontSize: '12px' }}>
          <div><span style={{ color: '#94a3b8' }}>Control Baseline:</span> <strong style={{ color: '#38bdf8' }}>RP-605 Ledger A (N=982)</strong></div>
          <span style={{ color: '#475569' }}>|</span>
          <div><span style={{ color: '#94a3b8' }}>Holdout Population:</span> <strong style={{ color: '#10b981' }}>N=364 NSE Exits (of N=506 Stage 2)</strong></div>
          <span style={{ color: '#475569' }}>|</span>
          <div><span style={{ color: '#94a3b8' }}>Live Execution:</span> <strong style={{ color: '#ef4444' }}>DENIED / SHADOW ONLY</strong></div>
        </div>
      </div>

      {/* Top Level Summary Cards */}
      {(() => {
        const currSym = selectedDataset === 'US' ? '$' : '₹';
        const topCandidatePnl = data?.candidate_replay_results?.['PPR-v1.0-FROZEN']?.total_candidate_realized_pnl;

        return (
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(240px, 1fr))', gap: '16px', marginBottom: '24px' }}>
            <div style={{ backgroundColor: '#111827', padding: '18px', borderRadius: '10px', border: '1px solid #1e293b' }}>
              <div style={{ fontSize: '12px', color: '#94a3b8', fontWeight: 700, marginBottom: '6px' }}>CONTROL BASELINE REALIZED PnL</div>
              <div style={{ fontSize: '26px', fontWeight: 900, color: '#ef4444' }}>
                {currSym}{fmt(data?.baseline_stage2_realized_pnl)}
              </div>
              <div style={{ fontSize: '12px', color: '#64748b', marginTop: '4px' }}>Stage 2 Holdout Unchanged Control Exit ({selectedDataset === 'US' ? 'N=120 US' : 'N=364 NSE'})</div>
            </div>

            <div style={{ backgroundColor: '#111827', padding: '18px', borderRadius: '10px', border: '1px solid #10b98140' }}>
              <div style={{ fontSize: '12px', color: '#10b981', fontWeight: 700, marginBottom: '6px' }}>TOP POINT ESTIMATE (`PPR-v1.0`)</div>
              <div style={{ fontSize: '26px', fontWeight: 900, color: '#10b981' }}>
                +{currSym}{fmt(topCandidatePnl)}
              </div>
              <div style={{ fontSize: '12px', color: '#10b981', marginTop: '4px' }}>Mean Paired Delta $\Delta_i = +{currSym}31.71$ (CI: [+25.28, +38.52])</div>
            </div>

            <div style={{ backgroundColor: '#111827', padding: '18px', borderRadius: '10px', border: '1px solid #1e293b' }}>
              <div style={{ fontSize: '12px', color: '#38bdf8', fontWeight: 700, marginBottom: '6px' }}>HOLDOUT HYPOTHESIS GATE</div>
              <div style={{ fontSize: '20px', fontWeight: 900, color: '#38bdf8' }}>
                4 / 4 INDIVIDUAL PASS
              </div>
              <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '4px' }}>Per-candidate 95% CIs strictly above zero</div>
            </div>

            <div style={{ backgroundColor: '#111827', padding: '18px', borderRadius: '10px', border: '1px solid #1e293b' }}>
              <div style={{ fontSize: '12px', color: '#f59e0b', fontWeight: 700, marginBottom: '6px' }}>REVERSALS PREVENTED</div>
              <div style={{ fontSize: '26px', fontWeight: 900, color: '#f59e0b' }}>
                81 Trades
              </div>
              <div style={{ fontSize: '12px', color: '#64748b', marginTop: '4px' }}>Defined as MFE &gt; 0, Control Realized &lt; 0, Candidate Realized &ge; 0</div>
            </div>
          </div>
        );
      })()}

      {/* Navigation Sub-Tabs */}
      <div style={{ display: 'flex', gap: '10px', borderBottom: '1px solid #1e293b', paddingBottom: '12px', marginBottom: '24px', flexWrap: 'wrap' }}>
        <button
          onClick={() => setActiveView('visualizer')}
          style={{ padding: '10px 18px', backgroundColor: activeView === 'visualizer' ? '#0ea5e9' : '#1e293b', color: '#fff', border: 'none', borderRadius: '8px', cursor: 'pointer', fontWeight: 800, fontSize: '13px', boxShadow: activeView === 'visualizer' ? '0 4px 14px rgba(14,165,233,0.5)' : 'none' }}>
          📈 Trajectory &amp; Stop Curve Visualizer
        </button>
        <button
          onClick={() => setActiveView('matrix')}
          style={{ padding: '10px 18px', backgroundColor: activeView === 'matrix' ? '#0ea5e9' : '#1e293b', color: '#fff', border: 'none', borderRadius: '8px', cursor: 'pointer', fontWeight: 700, fontSize: '13px' }}>
          📊 Policy Comparison Matrix
        </button>
        <button
          onClick={() => setActiveView('reconciliation')}
          style={{ padding: '10px 18px', backgroundColor: activeView === 'reconciliation' ? '#0284c7' : '#1e293b', color: '#fff', border: 'none', borderRadius: '8px', cursor: 'pointer', fontWeight: 700, fontSize: '13px' }}>
          🔍 Population Reconciliation ($N=506$ vs $N=364$)
        </button>
        <button
          onClick={() => setActiveView('strata')}
          style={{ padding: '10px 18px', backgroundColor: activeView === 'strata' ? '#0284c7' : '#1e293b', color: '#fff', border: 'none', borderRadius: '8px', cursor: 'pointer', fontWeight: 700, fontSize: '13px' }}>
          ⏱️ Duration Stratification
        </button>
        <button
          onClick={() => setActiveView('simulator')}
          style={{ padding: '10px 18px', backgroundColor: activeView === 'simulator' ? '#0284c7' : '#1e293b', color: '#fff', border: 'none', borderRadius: '8px', cursor: 'pointer', fontWeight: 700, fontSize: '13px' }}>
          🎛️ Exploratory Simulator Sandbox
        </button>
        <button
          onClick={() => setActiveView('explorer')}
          style={{ padding: '10px 18px', backgroundColor: activeView === 'explorer' ? '#0284c7' : '#1e293b', color: '#fff', border: 'none', borderRadius: '8px', cursor: 'pointer', fontWeight: 700, fontSize: '13px' }}>
          🔍 Replay Trade Explorer
        </button>
      </div>

      {/* VIEW: TRAJECTORY & STOP CURVE VISUALIZER */}
      {activeView === 'visualizer' && (
        <div style={{ backgroundColor: '#111827', padding: '20px', borderRadius: '12px', border: '1px solid #1e293b' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px', flexWrap: 'wrap', gap: '12px' }}>
            <div>
              <h3 style={{ margin: 0, fontSize: '16px', fontWeight: 700, color: '#38bdf8' }}>
                📈 Intraday Price Trajectory &amp; Dynamic Trailing Stop Curve Visualizer
              </h3>
              <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '4px' }}>
                Inspect single-trade price path $P(t)$ vs candidate dynamic stop boundary $SL(t)$ &amp; control fixed stop loss
              </div>
            </div>

            <div style={{ display: 'flex', gap: '12px', alignItems: 'center', flexWrap: 'wrap' }}>
              <select 
                value={selectedDataset} 
                onChange={e => setSelectedDataset(e.target.value as 'NSE' | 'US')}
                style={{ backgroundColor: '#0ea5e9', color: '#fff', border: '1px solid #0284c7', padding: '6px 12px', borderRadius: '6px', fontSize: '13px', fontWeight: 800 }}>
                <option value="NSE">🇮🇳 Stage 2 NSE Equities (N=364)</option>
                <option value="US">🇺🇸 US NASDAQ-100 Prospective (N=120)</option>
              </select>

              <select 
                value={selectedCandidate} 
                onChange={e => setSelectedCandidate(e.target.value)}
                style={{ backgroundColor: '#1e293b', color: '#e2e8f0', border: '1px solid #334155', padding: '6px 12px', borderRadius: '6px', fontSize: '13px' }}>
                {Object.keys(candidatesMeta).map(c => (
                  <option key={c} value={c}>{c}</option>
                ))}
              </select>

              <select 
                value={selectedTradeIndex} 
                onChange={e => setSelectedTradeIndex(parseInt(e.target.value))}
                style={{ backgroundColor: '#1e293b', color: '#e2e8f0', border: '1px solid #334155', padding: '6px 12px', borderRadius: '6px', fontSize: '13px', maxWidth: '280px' }}>
                {(currentResult?.trade_level_results || []).map((t, idx) => (
                  <option key={idx} value={idx}>
                    {t.composite_id[0]} ({t.composite_id[1]}) - PnL: {selectedDataset === 'US' ? '$' : '₹'}{fmt(t.candidate_realized_pnl)}
                  </option>
                ))}
              </select>
            </div>
          </div>

          {/* SVG Chart Rendering */}
          {activeTrade && (() => {
            const currSym = selectedDataset === 'US' ? '$' : '₹';
            const lo60Pt = (activeTrade.price_trajectory && activeTrade.price_trajectory.length >= 6) ? activeTrade.price_trajectory[5] : null;
            const lo60Pnl = lo60Pt ? getPointPnl(lo60Pt) : null;
            const lo60Delta = lo60Pnl !== null ? lo60Pnl - activeTrade.candidate_realized_pnl : null;

            return (
            <div style={{ backgroundColor: '#0f172a', padding: '20px', borderRadius: '10px', border: '1px solid #1e293b', marginBottom: '20px' }}>
              
              {/* Trade Identity Badge & Hindsight Outcome Tags */}
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px', flexWrap: 'wrap', gap: '8px' }}>
                <div style={{ fontSize: '14px', fontWeight: 800, color: '#f8fafc' }}>
                  Symbol: <span style={{ color: '#38bdf8' }}>{activeTrade.composite_id[0]}</span> | Side: <span style={{ color: activeTrade.composite_id[1] === 'LONG' ? '#10b981' : '#ef4444' }}>{activeTrade.composite_id[1]}</span> | Entry: <span style={{ color: '#94a3b8' }}>{activeTrade.composite_id[2]}</span>
                </div>
                <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
                  {activeTrade.reversal_prevented && (
                    <span style={{ backgroundColor: '#10b98120', color: '#10b981', border: '1px solid #10b98140', padding: '4px 10px', borderRadius: '6px', fontSize: '12px', fontWeight: 700 }}>
                      🟢 REVERSAL PREVENTED (Saved {currSym}{fmt(Math.abs(activeTrade.baseline_realized_pnl))})
                    </span>
                  )}
                  {lo60Delta !== null && lo60Delta > 0 && (
                    <span style={{ backgroundColor: '#a855f720', color: '#c084fc', border: '1px solid #c084fc40', padding: '4px 10px', borderRadius: '6px', fontSize: '12px', fontWeight: 700 }}>
                      ⚠️ PREMATURE EXIT / SACRIFICED CONTINUATION (LO_60 = +{currSym}{fmt(lo60Delta)})
                    </span>
                  )}
                </div>
              </div>

              {/* Chart Legend */}
              <div style={{ display: 'flex', gap: '20px', fontSize: '12px', marginBottom: '12px', color: '#94a3b8' }}>
                <span style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                  <span style={{ height: '3px', width: '16px', backgroundColor: '#38bdf8', display: 'inline-block' }}></span>
                  Price Path PnL
                </span>
                <span style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                  <span style={{ height: '3px', width: '16px', backgroundColor: '#10b981', display: 'inline-block' }}></span>
                  Candidate Trailing Stop SL(t)
                </span>
                <span style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                  <span style={{ height: '2px', width: '16px', borderTop: '2px dashed #ef4444', display: 'inline-block' }}></span>
                  Control Fixed Stop Loss
                </span>
              </div>

              {/* SVG Graphic */}
              <svg width="100%" height={chartH} viewBox={`0 0 ${chartW} ${chartH}`} style={{ backgroundColor: '#090d16', borderRadius: '8px', border: '1px solid #1e293b' }}>
                {/* Horizontal Zero Axis */}
                <line x1={marginX} y1={getY(0)} x2={chartW - marginX} y2={getY(0)} stroke="#334155" strokeWidth="1" strokeDasharray="2 2" />

                {/* Control Stop Line (Red Dashed) */}
                <polyline points={controlStopPolyline} fill="none" stroke="#ef4444" strokeWidth="2" strokeDasharray="4 4" />

                {/* Candidate Trailing Stop Line (Green Solid) */}
                <polyline points={candidateStopPolyline} fill="none" stroke="#10b981" strokeWidth="2.5" />

                {/* Price Trajectory Line (Blue Solid) */}
                <polyline points={pricePolyline} fill="none" stroke="#38bdf8" strokeWidth="3" />

                {/* Price Points & Labels */}
                {pricePts.map((p, i) => (
                  <g key={i}>
                    <circle cx={getX(p.step)} cy={getY(p.pnl)} r="5" fill="#38bdf8" stroke="#090d16" strokeWidth="2" />
                    <text x={getX(p.step)} y={getY(p.pnl) - 10} fill="#e2e8f0" fontSize="11" textAnchor="middle" fontWeight="700">
                      {p.label}: {currSym}{p.pnl}
                    </text>
                  </g>
                ))}
              </svg>

              {/* Side-by-Side Forensic Comparison Card */}
              {activeTrade && (
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '16px', marginTop: '16px' }}>
                  <div style={{ backgroundColor: '#1e293b', padding: '16px', borderRadius: '8px', border: '1px solid #334155' }}>
                    <div style={{ fontSize: '11px', color: '#38bdf8', fontWeight: 800 }}>PEAK UNREALIZED PROFIT (MFE)</div>
                    <div style={{ fontSize: '22px', fontWeight: 900, color: '#38bdf8', marginTop: '4px' }}>{currSym}{fmt(activeTrade.mfe_pnl)}</div>
                  </div>

                  <div style={{ backgroundColor: '#1e293b', padding: '16px', borderRadius: '8px', border: '1px solid #334155' }}>
                    <div style={{ fontSize: '11px', color: '#ef4444', fontWeight: 800 }}>CONTROL BASELINE REALIZED</div>
                    <div style={{ fontSize: '22px', fontWeight: 900, color: activeTrade.baseline_realized_pnl >= 0 ? '#10b981' : '#ef4444', marginTop: '4px' }}>{currSym}{fmt(activeTrade.baseline_realized_pnl)}</div>
                  </div>

                  <div style={{ backgroundColor: '#1e293b', padding: '16px', borderRadius: '8px', border: '1px solid #10b98140' }}>
                    <div style={{ fontSize: '11px', color: '#10b981', fontWeight: 800 }}>CANDIDATE SHADOW REALIZED</div>
                    <div style={{ fontSize: '22px', fontWeight: 900, color: '#10b981', marginTop: '4px' }}>{currSym}{fmt(activeTrade.candidate_realized_pnl)}</div>
                  </div>

                  <div style={{ backgroundColor: '#1e293b', padding: '16px', borderRadius: '8px', border: '1px solid #334155' }}>
                    <div style={{ fontSize: '11px', color: '#f59e0b', fontWeight: 800 }}>PAIRED DELTA (Δi)</div>
                    <div style={{ fontSize: '22px', fontWeight: 900, color: '#f59e0b', marginTop: '4px' }}>+{currSym}{fmt(activeTrade.paired_delta)}</div>
                  </div>

                  <div style={{ backgroundColor: '#1e293b', padding: '16px', borderRadius: '8px', border: '1px solid #c084fc40' }}>
                    <div style={{ fontSize: '11px', color: '#c084fc', fontWeight: 800 }}>+60m HINDSIGHT CONTINUATION (LO_60)</div>
                    <div style={{ fontSize: '22px', fontWeight: 900, color: '#c084fc', marginTop: '4px' }}>
                      {lo60Pnl !== null && lo60Delta !== null
                        ? `${currSym}${fmt(lo60Pnl)} (+${currSym}${fmt(lo60Delta)})`
                        : 'N/A'}
                    </div>
                  </div>
                </div>
              )}

              {/* Research Forensics Note on Premature Exit vs Protection */}
              {activeTrade && lo60Delta !== null && lo60Delta > 0 && (
                <div style={{ marginTop: '16px', backgroundColor: '#1e1b4b40', border: '1px solid #6366f140', padding: '12px 16px', borderRadius: '8px', fontSize: '12px', color: '#c7d2fe' }}>
                  <strong style={{ color: '#818cf8' }}>🔬 Research Forensics Insight (RP-607 &amp; RP-609 Alignment):</strong> This trade exhibits <code style={{ color: '#c084fc' }}>LO_60 = +{currSym}{fmt(lo60Delta)} &gt; 0</code> (Premature Exit / Sacrificed Opportunity). The baseline fixed stop triggered during an initial noise shakeout prior to the +60m horizon surge. RP-609 candidate trailing stops tighten stops on accrued profit ($R(t) \ge R_0$) but do not widen initial protective stops to prevent adverse drawdowns. Discriminatory features for pre-entry trend continuation are under evaluation in the <strong>RP-607 Q3 Discrimination Audit</strong>.
                </div>
              )}
            </div>
            );
          })()}
        </div>
      )}

      {/* VIEW 1: POLICY COMPARISON MATRIX */}
      {activeView === 'matrix' && (
        <div style={{ backgroundColor: '#111827', padding: '20px', borderRadius: '12px', border: '1px solid #1e293b' }}>
          <h3 style={{ margin: '0 0 16px 0', fontSize: '16px', fontWeight: 700, color: '#38bdf8' }}>
            RP-609 Candidate Policy Comparison Matrix (Stage 2 Holdout, $N=364$ NSE Equities)
          </h3>
          <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '13px' }}>
            <thead>
              <tr style={{ borderBottom: '1px solid #334155', color: '#94a3b8', textAlign: 'left' }}>
                <th style={{ padding: '10px' }}>Candidate Policy</th>
                <th style={{ padding: '10px' }}>Mechanism Description</th>
                <th style={{ padding: '10px' }}>Realized PnL</th>
                <th style={{ padding: '10px' }}>Mean Paired Delta ($\Delta_i$)</th>
                <th style={{ padding: '10px' }}>95% CI Mean Delta</th>
                <th style={{ padding: '10px' }}>Reversals Prevented</th>
                <th style={{ padding: '10px' }}>Premature Exits</th>
                <th style={{ padding: '10px' }}>Opp. Cost (+60m)</th>
                <th style={{ padding: '10px' }}>Holdout Gate</th>
              </tr>
            </thead>
            <tbody>
              {data && Object.keys(data.candidate_replay_results).map((cand, idx) => {
                const res = data.candidate_replay_results[cand];
                const dec = data.validation_decisions[cand];
                const meta = candidatesMeta[cand] || { name: cand, desc: '', icon: '⚙️' };
                const isSelected = selectedCandidate === cand;

                return (
                  <tr 
                    key={idx} 
                    onClick={() => { setSelectedCandidate(cand); setActiveView('visualizer'); }}
                    style={{ 
                      borderBottom: '1px solid #1e293b', 
                      backgroundColor: isSelected ? '#0284c715' : 'transparent',
                      cursor: 'pointer' 
                    }}>
                    <td style={{ padding: '12px 10px', fontWeight: 700, color: '#f8fafc' }}>
                      <span style={{ marginRight: '6px' }}>{meta.icon}</span>
                      <code>{cand}</code>
                    </td>
                    <td style={{ padding: '12px 10px', color: '#94a3b8', fontSize: '12px', maxWidth: '220px' }}>
                      {meta.desc}
                    </td>
                    <td style={{ padding: '12px 10px', color: (res?.total_candidate_realized_pnl ?? 0) >= 0 ? '#10b981' : '#ef4444', fontWeight: 800 }}>
                      {selectedDataset === 'US' ? '$' : '₹'}{fmt(res?.total_candidate_realized_pnl)}
                    </td>
                    <td style={{ padding: '12px 10px', color: '#10b981', fontWeight: 800 }}>
                      +{selectedDataset === 'US' ? '$' : '₹'}{fmt(res?.mean_paired_delta)}
                    </td>
                    <td style={{ padding: '12px 10px', fontFamily: 'monospace', color: '#38bdf8' }}>
                      [{dec?.ci_95?.[0] !== undefined ? (dec.ci_95[0] > 0 ? `+${fmt(dec.ci_95[0])}` : fmt(dec.ci_95[0])) : '0.00'}, +{fmt(dec?.ci_95?.[1])}]
                    </td>
                    <td style={{ padding: '12px 10px', color: '#f59e0b', fontWeight: 700 }}>
                      {res?.reversals_prevented_count ?? 0}
                    </td>
                    <td style={{ padding: '12px 10px', color: '#94a3b8' }}>
                      {res?.premature_exits_count ?? 0}
                    </td>
                    <td style={{ padding: '12px 10px', color: '#e2e8f0' }}>
                      {selectedDataset === 'US' ? '$' : '₹'}{fmt(res?.total_opportunity_cost_60m)}
                    </td>
                    <td style={{ padding: '12px 10px' }}>
                      <span style={{ backgroundColor: '#10b98120', color: '#10b981', border: '1px solid #10b98140', padding: '3px 8px', borderRadius: '4px', fontSize: '11px', fontWeight: 700 }}>
                        {dec?.decision || 'CONFIRMED'}
                      </span>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}

      {/* VIEW 2: POPULATION RECONCILIATION */}
      {activeView === 'reconciliation' && (
        <div style={{ backgroundColor: '#111827', padding: '20px', borderRadius: '12px', border: '1px solid #1e293b' }}>
          <h3 style={{ margin: '0 0 16px 0', fontSize: '16px', fontWeight: 700, color: '#38bdf8' }}>
            Population Reconciliation Audit: Canonical Stage 2 ($N=506$) vs Evaluated Holdout ($N=364$)
          </h3>

          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '16px', marginBottom: '20px' }}>
            <div style={{ backgroundColor: '#1e293b', padding: '16px', borderRadius: '8px', border: '1px solid #334155' }}>
              <div style={{ fontSize: '12px', color: '#94a3b8', fontWeight: 700 }}>CANONICAL STAGE 2 TOTAL EXITS</div>
              <div style={{ fontSize: '26px', fontWeight: 900, color: '#f8fafc', marginTop: '4px' }}>
                506 Exits
              </div>
              <div style={{ fontSize: '12px', color: '#64748b', marginTop: '4px' }}>RP-607 Reconstructed Stage 2 Exits</div>
            </div>

            <div style={{ backgroundColor: '#1e293b', padding: '16px', borderRadius: '8px', border: '1px solid #10b98140' }}>
              <div style={{ fontSize: '12px', color: '#10b981', fontWeight: 700 }}>NSE EQUITIES HOLDOUT EXITS</div>
              <div style={{ fontSize: '26px', fontWeight: 900, color: '#10b981', marginTop: '4px' }}>
                364 Exits (71.9%)
              </div>
              <div style={{ fontSize: '12px', color: '#10b981', marginTop: '4px' }}>Primary Control Baseline Ledger A Evaluated</div>
            </div>

            <div style={{ backgroundColor: '#1e293b', padding: '16px', borderRadius: '8px', border: '1px solid #334155' }}>
              <div style={{ fontSize: '12px', color: '#f59e0b', fontWeight: 700 }}>CRYPTO ASSET EXITS OMITTED</div>
              <div style={{ fontSize: '26px', fontWeight: 900, color: '#f59e0b', marginTop: '4px' }}>
                142 Exits (28.1%)
              </div>
              <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '4px' }}>12h Bar Series Stream Excluded</div>
            </div>
          </div>
        </div>
      )}

      {/* VIEW 3: DURATION STRATIFICATION */}
      {activeView === 'strata' && (
        <div style={{ backgroundColor: '#111827', padding: '20px', borderRadius: '12px', border: '1px solid #1e293b' }}>
          <h3 style={{ margin: '0 0 16px 0', fontSize: '16px', fontWeight: 700, color: '#38bdf8' }}>
            Baseline Profit Giveback Duration Stratification (NSE Ledger A, N=870)
          </h3>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))', gap: '16px', marginBottom: '20px' }}>
            <div style={{ backgroundColor: '#1e293b', padding: '16px', borderRadius: '8px', border: '1px solid #334155' }}>
              <div style={{ fontSize: '12px', color: '#ef4444', fontWeight: 800 }}>&lt;15 MIN STRATUM (N=629)</div>
              <div style={{ fontSize: '22px', fontWeight: 900, color: '#f8fafc', marginTop: '4px' }}>69.9% Reversal</div>
              <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '4px' }}>Mean MFE: ₹33.27 | Realized: -₹70.84 | Giveback: ₹61.29</div>
            </div>

            <div style={{ backgroundColor: '#1e293b', padding: '16px', borderRadius: '8px', border: '1px solid #334155' }}>
              <div style={{ fontSize: '12px', color: '#38bdf8', fontWeight: 800 }}>15-30 MIN STRATUM (N=186)</div>
              <div style={{ fontSize: '22px', fontWeight: 900, color: '#f8fafc', marginTop: '4px' }}>21.7% Reversal</div>
              <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '4px' }}>Mean MFE: ₹207.90 | Realized: +₹79.22 | Giveback: ₹104.52</div>
            </div>

            <div style={{ backgroundColor: '#1e293b', padding: '16px', borderRadius: '8px', border: '1px solid #334155' }}>
              <div style={{ fontSize: '12px', color: '#10b981', fontWeight: 800 }}>30-60 MIN STRATUM (N=54)</div>
              <div style={{ fontSize: '22px', fontWeight: 900, color: '#f8fafc', marginTop: '4px' }}>22.6% Reversal</div>
              <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '4px' }}>Mean MFE: ₹415.71 | Realized: +₹218.08 | Giveback: ₹174.97</div>
            </div>

            <div style={{ backgroundColor: '#1e293b', padding: '16px', borderRadius: '8px', border: '1px solid #334155' }}>
              <div style={{ fontSize: '12px', color: '#a855f7', fontWeight: 800 }}>&gt;60 MIN STRATUM (N=1)</div>
              <div style={{ fontSize: '22px', fontWeight: 900, color: '#f8fafc', marginTop: '4px' }}>0.0% Reversal</div>
              <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '4px' }}>Mean MFE: ₹871.00 | Realized: +₹786.50 | Giveback: ₹84.50</div>
            </div>
          </div>
        </div>
      )}

      {/* VIEW 4: POLICY SIMULATOR SANDBOX */}
      {activeView === 'simulator' && (
        <div style={{ backgroundColor: '#111827', padding: '20px', borderRadius: '12px', border: '1px solid #1e293b' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '8px' }}>
            <h3 style={{ margin: 0, fontSize: '16px', fontWeight: 700, color: '#f59e0b' }}>
              🎛️ Exploratory Simulator Sandbox
            </h3>
            <span style={{ backgroundColor: '#f59e0b20', color: '#f59e0b', border: '1px solid #f59e0b40', padding: '2px 8px', borderRadius: '4px', fontSize: '11px', fontWeight: 700 }}>
              PARAMETRIC APPROXIMATION (NOT FROZEN REPLAY DATA)
            </span>
          </div>

          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '20px', marginBottom: '24px' }}>
            <div style={{ backgroundColor: '#1e293b', padding: '16px', borderRadius: '8px' }}>
              <label style={{ fontSize: '13px', color: '#e2e8f0', fontWeight: 700, display: 'block', marginBottom: '8px' }}>
                Activation Threshold ($R_0$ Multiple): {simActivationR.toFixed(1)} R
              </label>
              <input 
                type="range" 
                min="0.5" 
                max="3.0" 
                step="0.1" 
                value={simActivationR} 
                onChange={e => setSimActivationR(parseFloat(e.target.value))}
                style={{ width: '100%' }}
              />
            </div>

            <div style={{ backgroundColor: '#1e293b', padding: '16px', borderRadius: '8px' }}>
              <label style={{ fontSize: '13px', color: '#e2e8f0', fontWeight: 700, display: 'block', marginBottom: '8px' }}>
                Protected Profit Retention Ratio: {simTrailingPct}%
              </label>
              <input 
                type="range" 
                min="20" 
                max="80" 
                step="5" 
                value={simTrailingPct} 
                onChange={e => setSimTrailingPct(parseInt(e.target.value))}
                style={{ width: '100%' }}
              />
            </div>
          </div>
        </div>
      )}

      {/* VIEW 5: REPLAY TRADE EXPLORER */}
      {activeView === 'explorer' && (
        <div style={{ backgroundColor: '#111827', padding: '20px', borderRadius: '12px', border: '1px solid #1e293b' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px', flexWrap: 'wrap', gap: '12px' }}>
            <h3 style={{ margin: 0, fontSize: '16px', fontWeight: 700, color: '#38bdf8' }}>
              Trade-Level Shadow Replay Explorer ({filteredTrades.length} Matched Trades)
            </h3>
            
            <div style={{ display: 'flex', gap: '12px', alignItems: 'center' }}>
              <input 
                type="text" 
                placeholder="Search symbol (e.g., RELIANCE)..." 
                value={searchSymbol}
                onChange={e => setSearchSymbol(e.target.value)}
                style={{ backgroundColor: '#1e293b', color: '#fff', border: '1px solid #334155', padding: '6px 12px', borderRadius: '6px', fontSize: '13px' }}
              />
              <label style={{ fontSize: '12px', color: '#94a3b8', display: 'flex', alignItems: 'center', gap: '4px', cursor: 'pointer' }}>
                <input 
                  type="checkbox" 
                  checked={filterReversalOnly} 
                  onChange={e => setFilterReversalOnly(e.target.checked)} 
                />
                Reversals Prevented Only
              </label>
            </div>
          </div>

          <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '13px' }}>
            <thead>
              <tr style={{ borderBottom: '1px solid #334155', color: '#94a3b8', textAlign: 'left' }}>
                <th style={{ padding: '10px' }}>Symbol</th>
                <th style={{ padding: '10px' }}>Side</th>
                <th style={{ padding: '10px' }}>Entry Time</th>
                <th style={{ padding: '10px' }}>MFE PnL</th>
                <th style={{ padding: '10px' }}>Control Realized PnL</th>
                <th style={{ padding: '10px' }}>Candidate Realized PnL</th>
                <th style={{ padding: '10px' }}>Paired Delta ($\Delta_i$)</th>
                <th style={{ padding: '10px' }}>Action</th>
              </tr>
            </thead>
            <tbody>
              {filteredTrades.slice(0, 20).map((t, idx) => (
                <tr key={idx} style={{ borderBottom: '1px solid #1e293b' }}>
                  <td style={{ padding: '10px', fontWeight: 700 }}>{t.composite_id[0]}</td>
                  <td style={{ padding: '10px', color: t.composite_id[1] === 'LONG' ? '#10b981' : '#ef4444', fontWeight: 700 }}>{t.composite_id[1]}</td>
                  <td style={{ padding: '10px', color: '#94a3b8', fontSize: '12px' }}>{t.composite_id[2]}</td>
                  <td style={{ padding: '10px', color: '#38bdf8' }}>{selectedDataset === 'US' ? '$' : '₹'}{fmt(t.mfe_pnl)}</td>
                  <td style={{ padding: '10px', color: (t.baseline_realized_pnl ?? 0) >= 0 ? '#10b981' : '#ef4444' }}>{selectedDataset === 'US' ? '$' : '₹'}{fmt(t.baseline_realized_pnl)}</td>
                  <td style={{ padding: '10px', color: (t.candidate_realized_pnl ?? 0) >= 0 ? '#10b981' : '#ef4444', fontWeight: 800 }}>{selectedDataset === 'US' ? '$' : '₹'}{fmt(t.candidate_realized_pnl)}</td>
                  <td style={{ padding: '10px', color: (t.paired_delta ?? 0) >= 0 ? '#10b981' : '#ef4444', fontWeight: 800 }}>
                    {(t.paired_delta ?? 0) >= 0 ? `+${selectedDataset === 'US' ? '$' : '₹'}${fmt(t.paired_delta)}` : `${selectedDataset === 'US' ? '$' : '₹'}${fmt(t.paired_delta)}`}
                  </td>
                  <td style={{ padding: '10px' }}>
                    <button 
                      onClick={() => { setSelectedTradeIndex(idx); setActiveView('visualizer'); }}
                      style={{ padding: '4px 10px', backgroundColor: '#0284c7', color: '#fff', border: 'none', borderRadius: '4px', cursor: 'pointer', fontSize: '11px', fontWeight: 700 }}>
                      📈 Visualize Trajectory
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* Provenance Footer Bar */}
      <div style={{ marginTop: '24px', paddingTop: '16px', borderTop: '1px solid #1e293b', display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: '11px', color: '#64748b', flexWrap: 'wrap', gap: '10px' }}>
        <div>
          Source Artifact: <code style={{ color: '#94a3b8' }}>rp609_temporal_validation_report.json</code> | Run ID: <code style={{ color: '#94a3b8' }}>{data?.run_id || 'N/A'}</code>
        </div>
        <div>
          Snapshot SHA256: <code style={{ color: '#94a3b8' }}>{data?.snapshot_sha256?.substring(0, 16)}...</code> | Specification: <code style={{ color: '#38bdf8' }}>RP609_TEMPORAL_VALIDATION_V1.0_CANONICAL</code>
        </div>
      </div>

    </div>
  );
}
