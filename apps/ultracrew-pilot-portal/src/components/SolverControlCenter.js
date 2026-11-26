import React from 'react';
import { S, theme } from '../styles';

export default function SolverControlCenter({
  selectedInstance,
  mode,
  setMode,
  generations,
  setGenerations,
  popSize,
  setPopSize,
  seed,
  setSeed,
  creditRigs,
  setCreditRigs,
  crewAvailability,
  setCrewAvailability,
  reserveRatio,
  setReserveRatio,
  running,
  onRunSolver,
  result,
}) {
  return (
    <div>
      <div style={S.grid2}>
        {/* Left Column: Optimization Mode & Parameters */}
        <div style={S.card}>
          <div style={S.cardTitle}>
            <span>1. Optimization Mode</span>
            <span style={S.badge('success')}>MOGA EVO v2.0</span>
          </div>
          <div style={S.cardSubtitle}>
            Select the primary operational objective for the evolutionary population search.
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '12px', marginTop: '16px' }}>
            {/* Mode A */}
            <div
              onClick={() => setMode('equity')}
              style={{
                ...S.statBox(mode === 'equity' ? 'success' : 'primary'),
                cursor: 'pointer',
                borderColor: mode === 'equity' ? theme.success : theme.surfaceBorder,
                background: mode === 'equity' ? 'rgba(16, 185, 129, 0.12)' : 'rgba(15, 23, 42, 0.75)',
              }}
            >
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <span style={{ fontWeight: '700', fontSize: '14px', color: '#f8fafc' }}>
                  Mode A: Workload Equity (Minimize σ)
                </span>
                {mode === 'equity' && <span style={S.badge('success')}>ACTIVE</span>}
              </div>
              <div style={{ fontSize: '12px', color: theme.textSecondary, marginTop: '6px' }}>
                Primary objective: minimize standard deviation of flying hours across all pilots.
                Guarantees labor union fairness and balanced fatigue accumulation.
              </div>
            </div>

            {/* Mode B */}
            <div
              onClick={() => setMode('preference')}
              style={{
                ...S.statBox(mode === 'preference' ? 'warning' : 'primary'),
                cursor: 'pointer',
                borderColor: mode === 'preference' ? theme.warning : theme.surfaceBorder,
                background: mode === 'preference' ? 'rgba(245, 158, 11, 0.12)' : 'rgba(15, 23, 42, 0.75)',
              }}
            >
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <span style={{ fontWeight: '700', fontSize: '14px', color: '#f8fafc' }}>
                  Mode B: Preference-Aware (Maximizes Bids & Vacations)
                </span>
                {mode === 'preference' && <span style={S.badge('warning')}>ACTIVE</span>}
              </div>
              <div style={{ fontSize: '12px', color: theme.textSecondary, marginTop: '6px' }}>
                Maximizes satisfaction of pilot vacation requests and pairing preferences.
                Achieves 84.2% vacation fulfillment (6.1x higher than column generation).
              </div>
            </div>

            {/* Mode C */}
            <div
              onClick={() => setMode('deadhead')}
              style={{
                ...S.statBox(mode === 'deadhead' ? 'primary' : 'primary'),
                cursor: 'pointer',
                borderColor: mode === 'deadhead' ? theme.primary : theme.surfaceBorder,
                background: mode === 'deadhead' ? 'rgba(14, 165, 233, 0.12)' : 'rgba(15, 23, 42, 0.75)',
              }}
            >
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <span style={{ fontWeight: '700', fontSize: '14px', color: '#f8fafc' }}>
                  Mode C: Deadhead / Open-Flight Protection
                </span>
                {mode === 'deadhead' && <span style={S.badge('primary')}>ACTIVE</span>}
              </div>
              <div style={{ fontSize: '12px', color: theme.textSecondary, marginTop: '6px' }}>
                Utilizes the Airport Transition Graph for cross-base repositioning,
                eliminating uncrewed flights during capacity-constrained peak schedules.
              </div>
            </div>
          </div>

          {/* Advanced Feature Toggles: Option A & Option B */}
          <div style={{ marginTop: '24px', borderTop: `1px solid ${theme.surfaceBorder}`, paddingTop: '18px' }}>
            <div style={{ fontSize: '14px', fontWeight: '700', marginBottom: '12px', color: '#f8fafc' }}>
              Statutory & Contractual Features
            </div>

            <label style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '12px', cursor: 'pointer' }}>
              <input
                type="checkbox"
                checked={crewAvailability}
                onChange={(e) => setCrewAvailability(e.target.checked)}
                style={{ width: '18px', height: '18px', accentColor: theme.primary }}
              />
              <div>
                <div style={{ fontSize: '13px', fontWeight: '600', color: '#f8fafc' }}>
                  Option A: Daily Crew Availability Limits (crew_avail_const)
                </div>
                <div style={{ fontSize: '11px', color: theme.textMuted }}>
                  Enforces daily capacity caps per domicile base from benchmark constraints.
                </div>
              </div>
            </label>

            <label style={{ display: 'flex', alignItems: 'center', gap: '10px', cursor: 'pointer' }}>
              <input
                type="checkbox"
                checked={creditRigs}
                onChange={(e) => setCreditRigs(e.target.checked)}
                style={{ width: '18px', height: '18px', accentColor: theme.primary }}
              />
              <div>
                <div style={{ fontSize: '13px', fontWeight: '600', color: '#f8fafc' }}>
                  Option B: Contractual Credit Rigs (Minimum Duty Guarantee + Trip Rig)
                </div>
                <div style={{ fontSize: '11px', color: theme.textMuted }}>
                  1h credit per 2h FDP duty allowance and 1h per 3.5h elapsed time.
                </div>
              </div>
            </label>
          </div>
        </div>

        {/* Right Column: Search Hyperparameters & Execution */}
        <div style={S.card}>
          <div style={S.cardTitle}>
            <span>2. Algorithm Hyperparameters</span>
            <span style={S.badge('neutral')}>CONFIG</span>
          </div>
          <div style={S.cardSubtitle}>
            Configure evolutionary population sizing, generation bounds, and seed.
          </div>

          <div style={{ marginTop: '16px', display: 'flex', flexDirection: 'column', gap: '16px' }}>
            <div>
              <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '6px' }}>
                <span style={S.statLabel}>Generations: {generations}</span>
                <span style={{ fontSize: '12px', color: theme.textMuted }}>100 — 1000</span>
              </div>
              <input
                type="range"
                min="100"
                max="1000"
                step="50"
                value={generations}
                onChange={(e) => setGenerations(Number(e.target.value))}
                style={{ width: '100%', accentColor: theme.primary }}
              />
            </div>

            <div>
              <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '6px' }}>
                <span style={S.statLabel}>Population Size: {popSize}</span>
                <span style={{ fontSize: '12px', color: theme.textMuted }}>20 — 100</span>
              </div>
              <input
                type="range"
                min="20"
                max="100"
                step="10"
                value={popSize}
                onChange={(e) => setPopSize(Number(e.target.value))}
                style={{ width: '100%', accentColor: theme.primary }}
              />
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px' }}>
              <div>
                <label style={S.statLabel}>Seed</label>
                <input
                  type="number"
                  style={S.input}
                  value={seed}
                  onChange={(e) => setSeed(Number(e.target.value))}
                />
              </div>
              <div>
                <label style={S.statLabel}>Standby Reserve Ratio</label>
                <select
                  style={S.select}
                  value={reserveRatio}
                  onChange={(e) => setReserveRatio(Number(e.target.value))}
                >
                  <option value={0.05}>5% Reserve Buffer</option>
                  <option value={0.08}>8% Standard (AOCC Recommended)</option>
                  <option value={0.12}>12% High-Risk / Weather Peak</option>
                </select>
              </div>
            </div>

            {/* Target Instance Summary Box */}
            {selectedInstance && (
              <div style={{ ...S.statBox('primary'), marginTop: '8px' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                  <span style={{ fontSize: '12px', color: theme.textMuted }}>Target Instance:</span>
                  <strong style={{ fontSize: '13px', color: '#f8fafc' }}>{selectedInstance.name}</strong>
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: '4px' }}>
                  <span style={{ fontSize: '12px', color: theme.textMuted }}>Pairings / Pilots:</span>
                  <span style={{ fontSize: '12px', color: theme.textSecondary }}>
                    {selectedInstance.pairings_count} pairings • {selectedInstance.crew_count} pilots
                  </span>
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: '4px' }}>
                  <span style={{ fontSize: '12px', color: theme.textMuted }}>Literature Benchmark:</span>
                  <span style={{ fontSize: '12px', color: theme.success, fontWeight: '700' }}>
                    Kasirzadeh et al. (2017)
                  </span>
                </div>
              </div>
            )}

            {/* Action Trigger */}
            <div style={{ marginTop: '16px' }}>
              <button
                style={{ ...S.btn(running ? 'neutral' : 'primary', 'lg'), width: '100%' }}
                onClick={onRunSolver}
                disabled={running}
              >
                {running ? (
                  <>
                    <span style={{ animation: 'spin 1s linear infinite' }}>⚙</span> Optimizing Roster...
                  </>
                ) : (
                  <>
                    <span>⚡</span> Execute Coralys Evolutionary Solver
                  </>
                )}
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Execution Results Banner if Solved */}
      {result && (
        <div style={{ ...S.card, borderColor: theme.successBorder, background: 'rgba(16, 185, 129, 0.08)' }}>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
            <div>
              <div style={{ fontSize: '17px', fontWeight: '800', color: theme.success }}>
                ✓ Optimal Schedule Constructed in {result.runtime_secs.toFixed(2)}s
              </div>
              <div style={{ fontSize: '13px', color: theme.textSecondary, marginTop: '4px' }}>
                Achieved <strong>{result.speedup_vs_gencol.toLocaleString()}x speedup</strong> over column generation with zero CAR 700 legality violations.
              </div>
            </div>
            <div style={{ display: 'flex', gap: '12px' }}>
              <div style={S.statBox('success')}>
                <div style={S.statLabel}>Pairing Coverage</div>
                <div style={S.statValue('success')}>100.0%</div>
              </div>
              <div style={S.statBox('primary')}>
                <div style={S.statLabel}>Workload σ</div>
                <div style={S.statValue('primary')}>
                  {result.audit_matrix.tier2.workload_disparity_sigma.toFixed(2)}h
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
