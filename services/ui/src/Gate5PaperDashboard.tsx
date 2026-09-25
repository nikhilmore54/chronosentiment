import React, { useState, useEffect } from 'react';

export interface Gate5PositionRecord {
  decision_id: string;
  symbol: string;
  direction: 'LONG' | 'SHORT';
  entry_price: number;
  current_price: number;
  current_return_pct: number;
  coralys_raw_action: string;
  protection_action: 'EXECUTE' | 'PROTECT' | 'DEFER';
  state_age_bars: number;
  mfe_pct: number;
  giveback_pct: number;
  trajectory_summary: string;
  last_transition: string;
  transition_count: number;
  paper_status: 'OPEN' | 'CLOSED';
  paired_delta_bps: number;
}

export interface Gate5Summary {
  coralys_exit_signals: number;
  managed_by_generator: number;
  coverage_pct: number;
  execute_count: number;
  protect_count: number;
  defer_count: number;
  open_count: number;
  closed_count: number;
  paired_delta_bps: number;
  loss_reduction_bps: number;
  new_drag_bps: number;
}

export const Gate5PaperDashboard: React.FC = () => {
  const [summary, setSummary] = useState<Gate5Summary>({
    coralys_exit_signals: 42,
    managed_by_generator: 42,
    coverage_pct: 100.0,
    execute_count: 11,
    protect_count: 19,
    defer_count: 12,
    open_count: 31,
    closed_count: 11,
    paired_delta_bps: 14.87,
    loss_reduction_bps: 48758.4,
    new_drag_bps: -22391.1,
  });

  const [positions, setPositions] = useState<Gate5PositionRecord[]>([
    {
      decision_id: 'DEC-20260925-001',
      symbol: 'RELIANCE',
      direction: 'LONG',
      entry_price: 2980.50,
      current_price: 2984.20,
      current_return_pct: 0.12,
      coralys_raw_action: 'EXIT',
      protection_action: 'DEFER',
      state_age_bars: 4,
      mfe_pct: 0.35,
      giveback_pct: 0.23,
      trajectory_summary: 'M5: -0.08, D5: +0.04, ΔM5: -0.01',
      last_transition: 'PROTECT → DEFER',
      transition_count: 2,
      paper_status: 'OPEN',
      paired_delta_bps: 24.5,
    },
    {
      decision_id: 'DEC-20260925-002',
      symbol: 'INFY',
      direction: 'LONG',
      entry_price: 1845.00,
      current_price: 1832.10,
      current_return_pct: -0.70,
      coralys_raw_action: 'EXIT',
      protection_action: 'EXECUTE',
      state_age_bars: 6,
      mfe_pct: 0.10,
      giveback_pct: 0.80,
      trajectory_summary: 'M5: +0.45, D5: +0.32, ΔM5: +0.12',
      last_transition: 'PROTECT → EXECUTE',
      transition_count: 3,
      paper_status: 'CLOSED',
      paired_delta_bps: 0.0,
    },
    {
      decision_id: 'DEC-20260925-003',
      symbol: 'TCS',
      direction: 'LONG',
      entry_price: 4210.00,
      current_price: 4218.40,
      current_return_pct: 0.20,
      coralys_raw_action: 'EXIT',
      protection_action: 'PROTECT',
      state_age_bars: 3,
      mfe_pct: 0.28,
      giveback_pct: 0.08,
      trajectory_summary: 'M5: -0.02, D5: -0.01, ΔM5: 0.00',
      last_transition: 'INITIAL → PROTECT',
      transition_count: 1,
      paper_status: 'OPEN',
      paired_delta_bps: 12.1,
    },
    {
      decision_id: 'DEC-20260925-004',
      symbol: 'HDFCBANK',
      direction: 'SHORT',
      entry_price: 1650.00,
      current_price: 1642.50,
      current_return_pct: 0.45,
      coralys_raw_action: 'EXIT',
      protection_action: 'DEFER',
      state_age_bars: 5,
      mfe_pct: 0.60,
      giveback_pct: 0.15,
      trajectory_summary: 'M5: -0.15, D5: -0.10, ΔM5: -0.03',
      last_transition: 'PROTECT → DEFER',
      transition_count: 2,
      paper_status: 'OPEN',
      paired_delta_bps: 38.2,
    },
    {
      decision_id: 'DEC-20260925-005',
      symbol: 'ICICIBANK',
      direction: 'LONG',
      entry_price: 1210.00,
      current_price: 1214.80,
      current_return_pct: 0.40,
      coralys_raw_action: 'EXIT',
      protection_action: 'DEFER',
      state_age_bars: 7,
      mfe_pct: 0.55,
      giveback_pct: 0.15,
      trajectory_summary: 'M5: -0.12, D5: -0.08, ΔM5: -0.02',
      last_transition: 'DEFER → DEFER',
      transition_count: 3,
      paper_status: 'OPEN',
      paired_delta_bps: 45.0,
    },
  ]);

  const getActionBadge = (act: 'EXECUTE' | 'PROTECT' | 'DEFER') => {
    switch (act) {
      case 'EXECUTE':
        return (
          <span style={{ backgroundColor: '#991b1b', color: '#fca5a5', padding: '4px 10px', borderRadius: '4px', fontSize: '12px', fontWeight: 700 }}>
            Coralys: EXIT → Protection: EXECUTE NOW
          </span>
        );
      case 'PROTECT':
        return (
          <span style={{ backgroundColor: '#854d0e', color: '#fef08a', padding: '4px 10px', borderRadius: '4px', fontSize: '12px', fontWeight: 700 }}>
            Coralys: EXIT → Protection: PROTECT MONITOR
          </span>
        );
      case 'DEFER':
        return (
          <span style={{ backgroundColor: '#166534', color: '#86efac', padding: '4px 10px', borderRadius: '4px', fontSize: '12px', fontWeight: 700 }}>
            Coralys: EXIT → Protection: DEFER EXECUTION
          </span>
        );
    }
  };

  return (
    <div style={{ padding: '24px', backgroundColor: '#0f172a', borderRadius: '12px', color: '#f8fafc' }}>
      {/* Header Banner */}
      <div style={{ borderBottom: '1px solid #334155', paddingBottom: '16px', marginBottom: '24px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <h2 style={{ margin: 0, fontSize: '22px', fontWeight: 700, color: '#38bdf8' }}>
            GATE 5 — CONTROLLED PAPER ACTIVATION
          </h2>
          <p style={{ margin: '6px 0 0 0', color: '#94a3b8', fontSize: '14px' }}>
            Side-by-Side Controlled Paper Execution & Observability Console
          </p>
        </div>
        <div style={{ display: 'flex', gap: '12px', alignItems: 'center' }}>
          <span style={{ backgroundColor: '#1e293b', padding: '6px 14px', borderRadius: '6px', border: '1px solid #475569', fontSize: '13px', color: '#e2e8f0' }}>
            Coralys Engine: <strong style={{ color: '#22c55e' }}>FROZEN</strong>
          </span>
          <span style={{ backgroundColor: '#1e293b', padding: '6px 14px', borderRadius: '6px', border: '1px solid #475569', fontSize: '13px', color: '#e2e8f0' }}>
            Generator: <strong style={{ color: '#38bdf8' }}>FROZEN RUNTIME</strong>
          </span>
        </div>
      </div>

      {/* Summary KPI Cards */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '16px', marginBottom: '32px' }}>
        <div style={{ backgroundColor: '#1e293b', padding: '16px', borderRadius: '8px', border: '1px solid #334155' }}>
          <div style={{ color: '#94a3b8', fontSize: '13px', fontWeight: 500 }}>Coralys EXIT Signals</div>
          <div style={{ fontSize: '24px', fontWeight: 700, marginTop: '8px', color: '#f8fafc' }}>
            {summary.coralys_exit_signals} <span style={{ fontSize: '14px', color: '#22c55e', fontWeight: 500 }}>(100% Coverage)</span>
          </div>
          <div style={{ fontSize: '12px', color: '#64748b', marginTop: '4px' }}>Managed: {summary.managed_by_generator} / {summary.coralys_exit_signals}</div>
        </div>

        <div style={{ backgroundColor: '#1e293b', padding: '16px', borderRadius: '8px', border: '1px solid #334155' }}>
          <div style={{ color: '#94a3b8', fontSize: '13px', fontWeight: 500 }}>Management Action Split</div>
          <div style={{ display: 'flex', gap: '8px', marginTop: '8px', fontSize: '13px', fontWeight: 600 }}>
            <span style={{ color: '#ef4444' }}>EXECUTE: {summary.execute_count}</span> \|
            <span style={{ color: '#eab308' }}>PROTECT: {summary.protect_count}</span> \|
            <span style={{ color: '#22c55e' }}>DEFER: {summary.defer_count}</span>
          </div>
          <div style={{ fontSize: '12px', color: '#64748b', marginTop: '4px' }}>Open: {summary.open_count} \| Closed: {summary.closed_count}</div>
        </div>

        <div style={{ backgroundColor: '#1e293b', padding: '16px', borderRadius: '8px', border: '1px solid #334155' }}>
          <div style={{ color: '#94a3b8', fontSize: '13px', fontWeight: 500 }}>Paired Net Delta</div>
          <div style={{ fontSize: '24px', fontWeight: 700, marginTop: '8px', color: '#22c55e' }}>
            +{summary.paired_delta_bps.toFixed(2)} <span style={{ fontSize: '14px' }}>bps/trade</span>
          </div>
          <div style={{ fontSize: '12px', color: '#64748b', marginTop: '4px' }}>Side-by-Side Paper Ledger</div>
        </div>

        <div style={{ backgroundColor: '#1e293b', padding: '16px', borderRadius: '8px', border: '1px solid #334155' }}>
          <div style={{ color: '#94a3b8', fontSize: '13px', fontWeight: 500 }}>Loss Reduction vs New Drag</div>
          <div style={{ fontSize: '14px', fontWeight: 600, marginTop: '8px', color: '#22c55e' }}>
            Loss Removed: +{summary.loss_reduction_bps.toLocaleString()} bps
          </div>
          <div style={{ fontSize: '12px', color: '#ef4444', marginTop: '4px' }}>
            New Drag: {summary.new_drag_bps.toLocaleString()} bps
          </div>
        </div>
      </div>

      {/* Position Observability Table */}
      <h3 style={{ fontSize: '18px', fontWeight: 600, margin: '0 0 16px 0', color: '#e2e8f0' }}>
        Active & Managed Paper Positions
      </h3>

      <div style={{ overflowX: 'auto' }}>
        <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left', fontSize: '13px' }}>
          <thead>
            <tr style={{ backgroundColor: '#1e293b', color: '#94a3b8', borderBottom: '2px solid #334155' }}>
              <th style={{ padding: '12px' }}>Symbol</th>
              <th style={{ padding: '12px' }}>Direction</th>
              <th style={{ padding: '12px' }}>Entry / Price</th>
              <th style={{ padding: '12px' }}>Return (%)</th>
              <th style={{ padding: '12px' }}>Raw Coralys</th>
              <th style={{ padding: '12px' }}>Protection Action</th>
              <th style={{ padding: '12px' }}>State Age</th>
              <th style={{ padding: '12px' }}>MFE / Giveback</th>
              <th style={{ padding: '12px' }}>Last Transition</th>
              <th style={{ padding: '12px' }}>Status</th>
              <th style={{ padding: '12px' }}>Paired Δ</th>
            </tr>
          </thead>
          <tbody>
            {positions.map((pos) => (
              <tr key={pos.decision_id} style={{ borderBottom: '1px solid #1e293b', backgroundColor: pos.paper_status === 'OPEN' ? '#0f172a' : '#1e1e2e' }}>
                <td style={{ padding: '12px', fontWeight: 600, color: '#f8fafc' }}>{pos.symbol}</td>
                <td style={{ padding: '12px' }}>
                  <span style={{ color: pos.direction === 'LONG' ? '#38bdf8' : '#f43f5e', fontWeight: 600 }}>{pos.direction}</span>
                </td>
                <td style={{ padding: '12px', color: '#cbd5e1' }}>
                  ₹{pos.entry_price.toFixed(2)} → ₹{pos.current_price.toFixed(2)}
                </td>
                <td style={{ padding: '12px', fontWeight: 600, color: pos.current_return_pct >= 0 ? '#22c55e' : '#ef4444' }}>
                  {pos.current_return_pct >= 0 ? `+${pos.current_return_pct.toFixed(2)}%` : `${pos.current_return_pct.toFixed(2)}%`}
                </td>
                <td style={{ padding: '12px' }}>
                  <span style={{ backgroundColor: '#334155', color: '#f8fafc', padding: '3px 8px', borderRadius: '4px', fontSize: '11px', fontWeight: 700 }}>
                    {pos.coralys_raw_action} (IMMUTABLE)
                  </span>
                </td>
                <td style={{ padding: '12px' }}>{getActionBadge(pos.protection_action)}</td>
                <td style={{ padding: '12px', color: '#cbd5e1' }}>{pos.state_age_bars} bars</td>
                <td style={{ padding: '12px', color: '#cbd5e1' }}>
                  +{pos.mfe_pct.toFixed(2)}% / {pos.giveback_pct.toFixed(2)}%
                </td>
                <td style={{ padding: '12px', color: '#94a3b8', fontSize: '12px' }}>
                  {pos.last_transition} ({pos.transition_count}x)
                </td>
                <td style={{ padding: '12px' }}>
                  <span style={{ color: pos.paper_status === 'OPEN' ? '#22c55e' : '#94a3b8', fontWeight: 600 }}>
                    {pos.paper_status}
                  </span>
                </td>
                <td style={{ padding: '12px', fontWeight: 700, color: pos.paired_delta_bps > 0 ? '#22c55e' : '#94a3b8' }}>
                  +{pos.paired_delta_bps.toFixed(1)} bps
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
};

export default Gate5PaperDashboard;
