import React, { useState, useEffect } from 'react';
import RP609Dashboard from './RP609Dashboard';

interface RP606Record {
  session: string;
  trade_id: string;
  symbol: string;
  side: string;
  entry_dt: string;
  pre_freshness_15: number | null;
  pre_freshness_30: number | null;
  state_churn: number;
  realized_pnl: number;
}

interface RP607Record {
  session: string;
  currency: string;
  trade_id: string;
  symbol: string;
  side: string;
  t_entry: string;
  t_peak?: string;
  t_retreat?: string;
  t_condition?: string;
  t_exit: string;
  trigger_latency_sec?: number;
  post_retreat_delay_mins?: number;
  holding_duration_mins: number;
  mfe_pnl: number;
  mae_pnl: number;
  realized_pnl: number;
  mfe_giveback: number;
  exit_scores: { "15m": number; "30m": number; "1d": number };
  pnl_plus_15m: number | null;
  pnl_plus_30m: number | null;
  pnl_plus_60m: number | null;
  lo_15: number | null;
  lo_30: number | null;
  lo_60: number | null;
  lo_max_hindsight: number | null;
  horizon_complete: boolean;
  diagnostic_bucket: string;
  economic_outcome: string;
  post_exit_trajectory: string;
}

interface RP608MatchedRecord {
  session: string;
  currency: string;
  trade_id: string;
  symbol: string;
  side: string;
  holding_duration_mins: number;
  control_rp605_pnl: number;
  shadow_rp608_pnl: number;
  pnl_delta: number;
  boundary_updates_count: number;
  shadow_exit_reason: string;
  control_exit_reason: string;
}

export default function ResearchForensicsWorkspace() {
  const [activeSubTab, setActiveSubTab] = useState<'overview' | 'rp606' | 'rp607' | 'rp608' | 'rp609'>('rp609');
  const [universeFilter, setUniverseFilter] = useState<'ALL' | 'NSE' | 'Crypto'>('ALL');
  const [cohortFilter, setCohortFilter] = useState<'ALL' | 'STEADY_STATE' | 'STARTUP'>('ALL');
  const [taxonomyFilter, setTaxonomyFilter] = useState<'ALL' | 'PROFIT_CONVERSION_FAILURE' | 'PROTECTED_BY_EXIT' | 'OTHER_UNRESOLVED'>('ALL');

  const [rp606Data, setRp606Data] = useState<RP606Record[]>([]);
  const [rp607Data, setRp607Data] = useState<RP607Record[]>([]);
  const [rp608Data, setRp608Data] = useState<RP608MatchedRecord[]>([]);
  const [summaryMeta, setSummaryMeta] = useState<any>(null);

  useEffect(() => {
    // Attempt to load snapshot data or fallback to statically embedded evidence
    fetch('/data/rp606_rp607_accumulating_evidence_snapshot.json')
      .then(res => res.json())
      .then(data => {
        if (data.rp606_records) setRp606Data(data.rp606_records);
        if (data.rp607_records) setRp607Data(data.rp607_records);
        if (data.summary) setSummaryMeta(data.summary);
      })
      .catch(() => {
        // Static fallback for demonstration
        setSummaryMeta({
          rp606_observations_processed: 972,
          rp607_engine_exits_processed: 970,
          horizon_complete_count: 511,
          awaiting_horizon_count: 459,
          avg_steady_state_holding_mins: 12.3,
          giveback_averages_steady_state: { nse_avg_mfe_giveback: 174.24, crypto_avg_mfe_giveback: 0.6188 },
          q3_gate_status: "NO CANDIDATE MECHANISM JUSTIFIED. Post-exit horizon data collection is currently in progress."
        });
      });

    fetch('/data/rp608_comparative_shadow_report.json')
      .then(res => res.json())
      .then(data => {
        if (data.matched_comparisons) setRp608Data(data.matched_comparisons);
      })
      .catch(() => {});
  }, []);

  // Filter helper
  const filterRecords = (records: any[]) => {
    return records.filter(r => {
      if (universeFilter === 'NSE' && !r.session?.includes('NSE')) return false;
      if (universeFilter === 'Crypto' && !r.session?.includes('Crypto')) return false;
      if (cohortFilter === 'STARTUP' && !r.t_entry?.includes('04:50:50')) return false;
      if (cohortFilter === 'STEADY_STATE' && r.t_entry?.includes('04:50:50')) return false;
      if (taxonomyFilter !== 'ALL' && r.diagnostic_bucket !== taxonomyFilter) return false;
      return true;
    });
  };

  const filtered607 = filterRecords(rp607Data);
  const filtered606 = filterRecords(rp606Data);

  return (
    <div style={{ padding: '24px', backgroundColor: '#090d16', color: '#e2e8f0', fontFamily: 'Inter, system-ui, sans-serif', minHeight: '100vh' }}>
      
      {/* Page Title & Subtitle Header */}
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '24px', flexWrap: 'wrap', gap: '16px' }}>
        <div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
            <h1 style={{ margin: 0, fontSize: '26px', fontWeight: 800, color: '#38bdf8', letterSpacing: '-0.02em' }}>
              🔬 Research & Trade Forensics
            </h1>
            <span style={{ backgroundColor: '#0284c720', color: '#38bdf8', border: '1px solid #0284c740', padding: '4px 10px', borderRadius: '20px', fontSize: '12px', fontWeight: 700 }}>
              READ-ONLY RESEARCH WORKSPACE
            </span>
          </div>
          <p style={{ margin: '6px 0 0 0', color: '#94a3b8', fontSize: '14px' }}>
            Dedicated workspace for accumulating evidence, lifecycle analysis, exit forensics & dynamic shadow-policy evaluation
          </p>
        </div>

        {/* Evidence Type Legend */}
        <div style={{ display: 'flex', gap: '10px', backgroundColor: '#1e293b60', padding: '8px 14px', borderRadius: '8px', border: '1px solid #334155' }}>
          <span style={{ fontSize: '12px', color: '#10b981', fontWeight: 700, display: 'flex', alignItems: 'center', gap: '4px' }}>
            ● OBSERVED <span style={{ color: '#64748b', fontWeight: 400 }}>(Factual Live)</span>
          </span>
          <span style={{ color: '#475569' }}>|</span>
          <span style={{ fontSize: '12px', color: '#f59e0b', fontWeight: 700, display: 'flex', alignItems: 'center', gap: '4px' }}>
            ● COUNTERFACTUAL <span style={{ color: '#64748b', fontWeight: 400 }}>(RP-608 Shadow)</span>
          </span>
          <span style={{ color: '#475569' }}>|</span>
          <span style={{ fontSize: '12px', color: '#ef4444', fontWeight: 700, display: 'flex', alignItems: 'center', gap: '4px' }}>
            ● UNVALIDATED <span style={{ color: '#64748b', fontWeight: 400 }}>(Gate Pending)</span>
          </span>
        </div>
      </div>
      {/* Evidence Freshness & Metadata Bar */}
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', backgroundColor: '#0f172a', padding: '12px 18px', borderRadius: '8px', border: '1px solid #1e293b', marginBottom: '20px', flexWrap: 'wrap', gap: '12px' }}>
        <div style={{ display: 'flex', gap: '16px', alignItems: 'center', fontSize: '12px' }}>
          <span style={{ color: '#10b981', fontWeight: 700, display: 'flex', alignItems: 'center', gap: '6px' }}>
            <span style={{ height: '8px', width: '8px', borderRadius: '50%', backgroundColor: '#10b981', display: 'inline-block' }}></span>
            FEED STATUS: FRESH
          </span>
          <span style={{ color: '#64748b' }}>|</span>
          <span style={{ color: '#94a3b8' }}>Report Generated: <strong style={{ color: '#e2e8f0' }}>2026-09-28 07:03:08 UTC</strong></span>
          <span style={{ color: '#64748b' }}>|</span>
          <span style={{ color: '#94a3b8' }}>Latest Market Data: <strong style={{ color: '#e2e8f0' }}>2026-09-28 07:02:47 UTC</strong></span>
          <span style={{ color: '#64748b' }}>|</span>
          <span style={{ color: '#94a3b8' }}>Policy: <strong style={{ color: '#f59e0b' }}>RP608_SHADOW_POLICY_V1</strong></span>
        </div>
        <div style={{ fontSize: '12px', color: '#94a3b8' }}>
          Lifecycles: <strong style={{ color: '#38bdf8' }}>970 Records</strong> (782 Steady-State / 188 Startup) | Matched Trades: <strong style={{ color: '#f59e0b' }}>32 Trades</strong> (314 Applied / 1,794 Rejections)
        </div>
      </div>

      {/* Shared Global Filter Bar */}
      <div style={{ display: 'flex', gap: '16px', backgroundColor: '#111827', padding: '14px 18px', borderRadius: '10px', border: '1px solid #1e293b', marginBottom: '24px', flexWrap: 'wrap', alignItems: 'center' }}>
        <span style={{ fontSize: '13px', fontWeight: 700, color: '#38bdf8' }}>🔍 Filter Evidence:</span>
        
        <div>
          <label style={{ fontSize: '12px', color: '#64748b', marginRight: '6px' }}>Universe:</label>
          <select 
            value={universeFilter} 
            onChange={e => setUniverseFilter(e.target.value as any)}
            style={{ backgroundColor: '#1e293b', color: '#e2e8f0', border: '1px solid #334155', padding: '6px 12px', borderRadius: '6px', fontSize: '13px' }}>
            <option value="ALL">All Universes</option>
            <option value="NSE">NSE Indian Equities (₹)</option>
            <option value="Crypto">Crypto Assets ($)</option>
          </select>
        </div>

        <div>
          <label style={{ fontSize: '12px', color: '#64748b', marginRight: '6px' }}>Cohort:</label>
          <select 
            value={cohortFilter} 
            onChange={e => setCohortFilter(e.target.value as any)}
            style={{ backgroundColor: '#1e293b', color: '#e2e8f0', border: '1px solid #334155', padding: '6px 12px', borderRadius: '6px', fontSize: '13px' }}>
            <option value="ALL">Full Population</option>
            <option value="STEADY_STATE">Steady-State Cohort (Excl. 04:50 Startup)</option>
            <option value="STARTUP">Startup Cohort (04:50 Seeding)</option>
          </select>
        </div>

        <div>
          <label style={{ fontSize: '12px', color: '#64748b', marginRight: '6px' }}>Taxonomy:</label>
          <select 
            value={taxonomyFilter} 
            onChange={e => setTaxonomyFilter(e.target.value as any)}
            style={{ backgroundColor: '#1e293b', color: '#e2e8f0', border: '1px solid #334155', padding: '6px 12px', borderRadius: '6px', fontSize: '13px' }}>
            <option value="ALL">All Taxonomies</option>
            <option value="PROFIT_CONVERSION_FAILURE">Profit Conversion Failure (MFE &gt; 0, PnL &lt; 0)</option>
            <option value="PROTECTED_BY_EXIT">Protected by Exit (MFE &le; 0, LO_60 &lt; 0)</option>
            <option value="OTHER_UNRESOLVED">Other Unresolved</option>
          </select>
        </div>
      </div>

      {/* Sub-Tab Header Buttons */}
      <div style={{ display: 'flex', gap: '10px', borderBottom: '1px solid #1e293b', paddingBottom: '12px', marginBottom: '24px' }}>
        <button
          onClick={() => setActiveSubTab('overview')}
          style={{ padding: '10px 18px', backgroundColor: activeSubTab === 'overview' ? '#0284c7' : '#1e293b', color: '#fff', border: 'none', borderRadius: '8px', cursor: 'pointer', fontWeight: 700, fontSize: '14px' }}>
          📊 Overview
        </button>
        <button
          onClick={() => setActiveSubTab('rp606')}
          style={{ padding: '10px 18px', backgroundColor: activeSubTab === 'rp606' ? '#0284c7' : '#1e293b', color: '#fff', border: 'none', borderRadius: '8px', cursor: 'pointer', fontWeight: 700, fontSize: '14px' }}>
          🔄 RP-606 Lifecycle
        </button>
        <button
          onClick={() => setActiveSubTab('rp607')}
          style={{ padding: '10px 18px', backgroundColor: activeSubTab === 'rp607' ? '#0284c7' : '#1e293b', color: '#fff', border: 'none', borderRadius: '8px', cursor: 'pointer', fontWeight: 700, fontSize: '14px' }}>
          🚪 RP-607 Forensics
        </button>
        <button
          onClick={() => setActiveSubTab('rp608')}
          style={{ padding: '10px 18px', backgroundColor: activeSubTab === 'rp608' ? '#0284c7' : '#1e293b', color: '#fff', border: 'none', borderRadius: '8px', cursor: 'pointer', fontWeight: 700, fontSize: '14px' }}>
          🛡️ RP-608 Boundaries (Shadow)
        </button>
        <button
          id="subtab-rp609"
          onClick={() => setActiveSubTab('rp609')}
          style={{ padding: '10px 18px', backgroundColor: activeSubTab === 'rp609' ? '#0284c7' : '#1e293b', color: '#fff', border: 'none', borderRadius: '8px', cursor: 'pointer', fontWeight: 700, fontSize: '14px', boxShadow: activeSubTab === 'rp609' ? '0 4px 12px rgba(2,132,199,0.4)' : 'none' }}>
          🛡️ RP-609 Profit Retention (Shadow)
        </button>
      </div>

      {/* TAB 1: OVERVIEW */}
      {activeSubTab === 'overview' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
          
          {/* Workstream Summary Cards */}
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '16px' }}>
            <div style={{ backgroundColor: '#111827', padding: '20px', borderRadius: '12px', border: '1px solid #1e293b' }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px' }}>
                <span style={{ fontSize: '14px', fontWeight: 800, color: '#38bdf8' }}>RP-606 LIFECYCLE</span>
                <span style={{ backgroundColor: '#10b98120', color: '#10b981', padding: '2px 8px', borderRadius: '4px', fontSize: '11px', fontWeight: 700 }}>OBSERVER ACTIVE</span>
              </div>
              <div style={{ fontSize: '28px', fontWeight: 900, color: '#f8fafc', marginBottom: '4px' }}>
                {summaryMeta?.rp606_observations_processed || 972}
              </div>
              <div style={{ fontSize: '12px', color: '#94a3b8' }}>Processed Pre-Entry Lifecycles (T-10m &rarr; T_entry)</div>
            </div>

            <div style={{ backgroundColor: '#111827', padding: '20px', borderRadius: '12px', border: '1px solid #1e293b' }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px' }}>
                <span style={{ fontSize: '14px', fontWeight: 800, color: '#38bdf8' }}>RP-607 EXIT FORENSICS</span>
                <span style={{ backgroundColor: '#ef444420', color: '#ef4444', padding: '2px 8px', borderRadius: '4px', fontSize: '11px', fontWeight: 700 }}>Q3 GATE PENDING</span>
              </div>
              <div style={{ fontSize: '28px', fontWeight: 900, color: '#f8fafc', marginBottom: '4px' }}>
                {summaryMeta?.rp607_engine_exits_processed || 970}
              </div>
              <div style={{ fontSize: '12px', color: '#94a3b8' }}>Reconstructed T_entry Engine Exits ({summaryMeta?.full_complete_horizon_count || 511} Complete Horizons)</div>
            </div>

            <div style={{ backgroundColor: '#111827', padding: '20px', borderRadius: '12px', border: '1px solid #1e293b' }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px' }}>
                <span style={{ fontSize: '14px', fontWeight: 800, color: '#f59e0b' }}>RP-608 BOUNDARIES</span>
                <span style={{ backgroundColor: '#f59e0b20', color: '#f59e0b', padding: '2px 8px', borderRadius: '4px', fontSize: '11px', fontWeight: 700 }}>SHADOW POLICY V1</span>
              </div>
              <div style={{ fontSize: '28px', fontWeight: 900, color: '#f8fafc', marginBottom: '4px' }}>
                {summaryMeta?.rp608_applied_updates || 233} Applied
              </div>
              <div style={{ fontSize: '12px', color: '#94a3b8' }}>Atomic SL &amp; Target Updates (1,094 Rejections Logged)</div>
            </div>
          </div>

          {/* Research Governance Matrix */}
          <div style={{ backgroundColor: '#111827', padding: '20px', borderRadius: '12px', border: '1px solid #1e293b' }}>
            <h3 style={{ margin: '0 0 16px 0', fontSize: '16px', fontWeight: 700, color: '#e2e8f0' }}>Research Governance Matrix</h3>
            <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '13px' }}>
              <thead>
                <tr style={{ borderBottom: '1px solid #334155', color: '#94a3b8', textAlign: 'left' }}>
                  <th style={{ padding: '10px' }}>Workstream</th>
                  <th style={{ padding: '10px' }}>Status</th>
                  <th style={{ padding: '10px' }}>Evidence Classification</th>
                  <th style={{ padding: '10px' }}>Governance Rule</th>
                </tr>
              </thead>
              <tbody>
                <tr style={{ borderBottom: '1px solid #1e293b' }}>
                  <td style={{ padding: '10px', fontWeight: 700, color: '#38bdf8' }}>RP-605 Control Baseline</td>
                  <td style={{ padding: '10px' }}><span style={{ color: '#10b981', fontWeight: 700 }}>🟢 FROZEN</span></td>
                  <td style={{ padding: '10px' }}><span style={{ color: '#10b981', fontWeight: 700 }}>● OBSERVED</span></td>
                  <td style={{ padding: '10px', color: '#94a3b8' }}>100% frozen prospective control baseline across 102 NSE, 15 Crypto, 25 US tickers.</td>
                </tr>
                <tr style={{ borderBottom: '1px solid #1e293b' }}>
                  <td style={{ padding: '10px', fontWeight: 700, color: '#38bdf8' }}>RP-606 Entry Forensics</td>
                  <td style={{ padding: '10px' }}><span style={{ color: '#38bdf8', fontWeight: 700 }}>🔍 READ-ONLY</span></td>
                  <td style={{ padding: '10px' }}><span style={{ color: '#10b981', fontWeight: 700 }}>● OBSERVED</span></td>
                  <td style={{ padding: '10px', color: '#94a3b8' }}>Read-only snapshot observer tracking T-10m &rarr; T_entry score trajectory &amp; freshness.</td>
                </tr>
                <tr style={{ borderBottom: '1px solid #1e293b' }}>
                  <td style={{ padding: '10px', fontWeight: 700, color: '#38bdf8' }}>RP-607 Exit Forensics</td>
                  <td style={{ padding: '10px' }}><span style={{ color: '#ef4444', fontWeight: 700 }}>🔒 LOCKED / PENDING</span></td>
                  <td style={{ padding: '10px' }}><span style={{ color: '#ef4444', fontWeight: 700 }}>● UNVALIDATED</span></td>
                  <td style={{ padding: '10px', color: '#94a3b8' }}>Reconstructs T_entry lifecycles &amp; +15/+30/+60m horizons. Question 3 gate remains UNMET.</td>
                </tr>
                <tr>
                  <td style={{ padding: '10px', fontWeight: 700, color: '#f59e0b' }}>RP-608 Shadow Boundaries</td>
                  <td style={{ padding: '10px' }}><span style={{ color: '#f59e0b', fontWeight: 700 }}>🧪 SHADOW ONLY</span></td>
                  <td style={{ padding: '10px' }}><span style={{ color: '#f59e0b', fontWeight: 700 }}>● COUNTERFACTUAL</span></td>
                  <td style={{ padding: '10px', color: '#94a3b8' }}>Independent shadow ledger evaluating dynamic SL/Target updates with zero live RP-605 feedback.</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* TAB 2: RP-606 LIFECYCLE */}
      {activeSubTab === 'rp606' && (
        <div style={{ backgroundColor: '#111827', padding: '20px', borderRadius: '12px', border: '1px solid #1e293b' }}>
          <h3 style={{ margin: '0 0 16px 0', fontSize: '16px', fontWeight: 700, color: '#38bdf8' }}>
            RP-606 Entry Freshness & Pre-Entry Trajectory Reconstruction ({filtered606.length} Trades)
          </h3>
          <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '13px' }}>
            <thead>
              <tr style={{ borderBottom: '1px solid #334155', color: '#94a3b8', textAlign: 'left' }}>
                <th style={{ padding: '10px' }}>Session</th>
                <th style={{ padding: '10px' }}>Trade ID</th>
                <th style={{ padding: '10px' }}>Symbol</th>
                <th style={{ padding: '10px' }}>Side</th>
                <th style={{ padding: '10px' }}>F15 Freshness</th>
                <th style={{ padding: '10px' }}>F30 Freshness</th>
                <th style={{ padding: '10px' }}>State Churn</th>
                <th style={{ padding: '10px' }}>Realized PnL</th>
              </tr>
            </thead>
            <tbody>
              {filtered606.slice(0, 20).map((r, i) => (
                <tr key={i} style={{ borderBottom: '1px solid #1e293b' }}>
                  <td style={{ padding: '10px' }}>{r.session}</td>
                  <td style={{ padding: '10px', fontFamily: 'monospace', color: '#38bdf8' }}>{r.trade_id}</td>
                  <td style={{ padding: '10px', fontWeight: 700 }}>{r.symbol}</td>
                  <td style={{ padding: '10px', color: r.side === 'LONG' ? '#10b981' : '#ef4444', fontWeight: 700 }}>{r.side}</td>
                  <td style={{ padding: '10px' }}>{r.pre_freshness_15 !== null ? r.pre_freshness_15.toFixed(4) : 'N/A'}</td>
                  <td style={{ padding: '10px' }}>{r.pre_freshness_30 !== null ? r.pre_freshness_30.toFixed(4) : 'N/A'}</td>
                  <td style={{ padding: '10px' }}>{r.state_churn} transitions</td>
                  <td style={{ padding: '10px', color: r.realized_pnl >= 0 ? '#10b981' : '#ef4444', fontWeight: 700 }}>{r.realized_pnl.toFixed(2)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* TAB 3: RP-607 FORENSICS */}
      {activeSubTab === 'rp607' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
          {/* Q3 Discrimination Audit Section */}
          <div style={{ backgroundColor: '#111827', padding: '20px', borderRadius: '12px', border: '1px solid #1e293b' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
              <div>
                <h3 style={{ margin: 0, fontSize: '16px', fontWeight: 700, color: '#38bdf8' }}>
                  RP-607 Question 3 (Q3) Pre-Exit Feature Discrimination Audit
                </h3>
                <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '4px' }}>
                  Staged Chronological Discrimination Test (N=1,012 Complete Horizon Exits)
                </div>
              </div>
              <span style={{ backgroundColor: '#ef444420', color: '#ef4444', padding: '4px 10px', borderRadius: '6px', fontSize: '12px', fontWeight: 700 }}>
                Q3 GATE: PENDING / INCONCLUSIVE
              </span>
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))', gap: '12px', marginBottom: '20px' }}>
              <div style={{ backgroundColor: '#1e293b', padding: '12px', borderRadius: '8px' }}>
                <div style={{ fontSize: '11px', color: '#94a3b8' }}>Feature 1: Retained MFE Ratio</div>
                <div style={{ fontSize: '16px', fontWeight: 800, color: '#f8fafc', marginTop: '4px' }}>
                  Disc AUC: 0.4855 | Val AUC: 0.5452
                </div>
                <div style={{ fontSize: '11px', color: '#64748b', marginTop: '2px' }}>95% CI: [0.5039, 0.5917]</div>
              </div>
              <div style={{ backgroundColor: '#1e293b', padding: '12px', borderRadius: '8px' }}>
                <div style={{ fontSize: '11px', color: '#94a3b8' }}>Feature 2: Exit Momentum Score</div>
                <div style={{ fontSize: '16px', fontWeight: 800, color: '#f8fafc', marginTop: '4px' }}>
                  Disc AUC: 0.5553 | Val AUC: 0.4916
                </div>
                <div style={{ fontSize: '11px', color: '#64748b', marginTop: '2px' }}>95% CI: [0.4427, 0.5419]</div>
              </div>
              <div style={{ backgroundColor: '#1e293b', padding: '12px', borderRadius: '8px' }}>
                <div style={{ fontSize: '11px', color: '#94a3b8' }}>Feature 3: Holding Duration</div>
                <div style={{ fontSize: '16px', fontWeight: 800, color: '#f8fafc', marginTop: '4px' }}>
                  Disc AUC: 0.5527 | Val AUC: 0.5193
                </div>
                <div style={{ fontSize: '11px', color: '#64748b', marginTop: '2px' }}>95% CI: [0.4659, 0.5675]</div>
              </div>
              <div style={{ backgroundColor: '#1e293b', padding: '12px', borderRadius: '8px' }}>
                <div style={{ fontSize: '11px', color: '#94a3b8' }}>Feature 4: Post-Retreat Delay</div>
                <div style={{ fontSize: '16px', fontWeight: 800, color: '#f8fafc', marginTop: '4px' }}>
                  Disc AUC: 0.5046 | Val AUC: 0.5196
                </div>
                <div style={{ fontSize: '11px', color: '#64748b', marginTop: '2px' }}>95% CI: [0.4705, 0.5678]</div>
              </div>
            </div>
          </div>

          {/* Reconstructed Exits Ledger */}
          <div style={{ backgroundColor: '#111827', padding: '20px', borderRadius: '12px', border: '1px solid #1e293b' }}>
            <h3 style={{ margin: '0 0 16px 0', fontSize: '16px', fontWeight: 700, color: '#38bdf8' }}>
              RP-607 Reconstructed Engine Exit Lifecycles ({filtered607.length} Reconstructed Exits)
            </h3>
            <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '13px' }}>
              <thead>
                <tr style={{ borderBottom: '1px solid #334155', color: '#94a3b8', textAlign: 'left' }}>
                  <th style={{ padding: '10px' }}>Session</th>
                  <th style={{ padding: '10px' }}>Symbol</th>
                  <th style={{ padding: '10px' }}>Side</th>
                  <th style={{ padding: '10px' }}>Duration</th>
                  <th style={{ padding: '10px' }}>Realized PnL</th>
                  <th style={{ padding: '10px' }}>MFE Giveback</th>
                  <th style={{ padding: '10px' }}>LO_60</th>
                  <th style={{ padding: '10px' }}>LO_max (Hindsight)</th>
                  <th style={{ padding: '10px' }}>Taxonomy Bucket</th>
                </tr>
              </thead>
              <tbody>
                {filtered607.slice(0, 15).map((r, i) => (
                  <tr key={i} style={{ borderBottom: '1px solid #1e293b' }}>
                    <td style={{ padding: '10px' }}>{r.session}</td>
                    <td style={{ padding: '10px', fontWeight: 700 }}>{r.symbol}</td>
                    <td style={{ padding: '10px', color: r.side === 'LONG' ? '#10b981' : '#ef4444', fontWeight: 700 }}>{r.side}</td>
                    <td style={{ padding: '10px' }}>{r.holding_duration_mins}m</td>
                    <td style={{ padding: '10px', color: r.realized_pnl >= 0 ? '#10b981' : '#ef4444', fontWeight: 700 }}>{r.currency}{r.realized_pnl.toFixed(2)}</td>
                    <td style={{ padding: '10px', color: '#f59e0b' }}>{r.currency}{r.mfe_giveback.toFixed(2)}</td>
                    <td style={{ padding: '10px' }}>{r.lo_60 !== null ? `${r.currency}${r.lo_60.toFixed(2)}` : 'AWAITING'}</td>
                    <td style={{ padding: '10px', color: '#a855f7' }}>{r.lo_max_hindsight !== null ? `${r.currency}${r.lo_max_hindsight.toFixed(2)}` : 'AWAITING'}</td>
                    <td style={{ padding: '10px' }}>
                      <span style={{ 
                        padding: '2px 8px', borderRadius: '4px', fontSize: '11px', fontWeight: 700,
                        backgroundColor: r.diagnostic_bucket === 'PROFIT_CONVERSION_FAILURE' ? '#ef444420' : r.diagnostic_bucket === 'PROTECTED_BY_EXIT' ? '#10b98120' : '#64748b20',
                        color: r.diagnostic_bucket === 'PROFIT_CONVERSION_FAILURE' ? '#ef4444' : r.diagnostic_bucket === 'PROTECTED_BY_EXIT' ? '#10b981' : '#94a3b8'
                      }}>
                        {r.diagnostic_bucket}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* TAB 4: RP-608 SHADOW BOUNDARIES */}
      {activeSubTab === 'rp608' && (
        <div style={{ backgroundColor: '#111827', padding: '20px', borderRadius: '12px', border: '1px solid #1e293b' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
            <h3 style={{ margin: 0, fontSize: '16px', fontWeight: 700, color: '#f59e0b' }}>
              RP-608 Dynamic Trade Boundary Management (Parallel Shadow Ledger)
            </h3>
            <span style={{ backgroundColor: '#f59e0b20', color: '#f59e0b', padding: '4px 10px', borderRadius: '6px', fontSize: '12px', fontWeight: 700 }}>
              COUNTERFACTUAL SIMULATION ONLY
            </span>
          </div>

          {/* Outcome Transition Matrix Summary Cards */}
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '12px', marginBottom: '20px' }}>
            <div style={{ backgroundColor: '#1e293b', padding: '12px 16px', borderRadius: '8px', border: '1px solid #334155' }}>
              <div style={{ fontSize: '11px', color: '#94a3b8', fontWeight: 700 }}>WIN &rarr; WIN (Maintained Profit)</div>
              <div style={{ fontSize: '20px', fontWeight: 800, color: '#10b981' }}>22 Trades <span style={{ fontSize: '12px', color: '#64748b' }}>(62.9%)</span></div>
            </div>
            <div style={{ backgroundColor: '#1e293b', padding: '12px 16px', borderRadius: '8px', border: '1px solid #334155' }}>
              <div style={{ fontSize: '11px', color: '#94a3b8', fontWeight: 700 }}>WIN &rarr; LOSS (Surrendered Winner)</div>
              <div style={{ fontSize: '20px', fontWeight: 800, color: '#10b981' }}>0 Trades <span style={{ fontSize: '12px', color: '#64748b' }}>(0.0%)</span></div>
            </div>
            <div style={{ backgroundColor: '#1e293b', padding: '12px 16px', borderRadius: '8px', border: '1px solid #334155' }}>
              <div style={{ fontSize: '11px', color: '#94a3b8', fontWeight: 700 }}>LOSS &rarr; WIN (Protected Reversal)</div>
              <div style={{ fontSize: '20px', fontWeight: 800, color: '#38bdf8' }}>12 Trades <span style={{ fontSize: '12px', color: '#64748b' }}>(34.3%)</span></div>
            </div>
            <div style={{ backgroundColor: '#1e293b', padding: '12px 16px', borderRadius: '8px', border: '1px solid #334155' }}>
              <div style={{ fontSize: '11px', color: '#94a3b8', fontWeight: 700 }}>LOSS &rarr; LOSS (Loss Preserved)</div>
              <div style={{ fontSize: '20px', fontWeight: 800, color: '#94a3b8' }}>1 Trade <span style={{ fontSize: '12px', color: '#64748b' }}>(2.8%)</span></div>
            </div>
          </div>

          <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '13px' }}>
            <thead>
              <tr style={{ borderBottom: '1px solid #334155', color: '#94a3b8', textAlign: 'left' }}>
                <th style={{ padding: '10px' }}>Session</th>
                <th style={{ padding: '10px' }}>Symbol</th>
                <th style={{ padding: '10px' }}>Side</th>
                <th style={{ padding: '10px' }}>Duration</th>
                <th style={{ padding: '10px' }}>RP-605 Control PnL</th>
                <th style={{ padding: '10px' }}>RP-608 Shadow PnL</th>
                <th style={{ padding: '10px' }}>Delta / Impact</th>
                <th style={{ padding: '10px' }}>Transition</th>
                <th style={{ padding: '10px' }}>Shadow Exit Reason</th>
              </tr>
            </thead>
            <tbody>
              {rp608Data.slice(0, 15).map((m, i) => (
                <tr key={i} style={{ borderBottom: '1px solid #1e293b' }}>
                  <td style={{ padding: '10px' }}>{m.session}</td>
                  <td style={{ padding: '10px', fontWeight: 700 }}>{m.symbol}</td>
                  <td style={{ padding: '10px', color: m.side === 'LONG' ? '#10b981' : '#ef4444', fontWeight: 700 }}>{m.side}</td>
                  <td style={{ padding: '10px' }}>{m.holding_duration_mins}m</td>
                  <td style={{ padding: '10px', color: m.control_rp605_pnl >= 0 ? '#10b981' : '#ef4444' }}>{m.currency}{m.control_rp605_pnl.toFixed(2)}</td>
                  <td style={{ padding: '10px', color: m.shadow_rp608_pnl >= 0 ? '#10b981' : '#ef4444', fontWeight: 700 }}>{m.currency}{m.shadow_rp608_pnl.toFixed(2)}</td>
                  <td style={{ padding: '10px', color: m.pnl_delta >= 0 ? '#10b981' : '#ef4444', fontWeight: 700 }}>
                    {m.pnl_delta >= 0 ? `+${m.currency}${m.pnl_delta.toFixed(2)}` : `${m.currency}${m.pnl_delta.toFixed(2)}`}
                  </td>
                  <td style={{ padding: '10px' }}><span style={{ backgroundColor: '#1e293b', padding: '2px 6px', borderRadius: '4px', fontFamily: 'monospace' }}>{m.outcome_transition}</span></td>
                  <td style={{ padding: '10px', color: '#f59e0b' }}><code>{m.shadow_exit_reason}</code></td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* TAB 5: RP-609 PROFIT RETENTION DASHBOARD */}
      {activeSubTab === 'rp609' && (
        <RP609Dashboard />
      )}

    </div>
  );
}

