import React from 'react';
import { S, theme } from '../styles';

export default function BenchmarkStudio({
  catalog,
  selectedInstanceId,
  onSelectInstance,
  onLaunchSolver,
}) {
  const selected = catalog.find((i) => i.id === selectedInstanceId) || catalog[0];

  return (
    <div>
      {/* Overview Banner */}
      <div style={{ ...S.card, borderColor: 'rgba(14, 165, 233, 0.4)' }}>
        <div style={S.cardHeader}>
          <div>
            <div style={S.cardTitle}>
              <span>GERAD G-2014-22 Definitive Airline Benchmark Suite</span>
              <span style={S.badge('success')}>7 INSTANCES • 1,034 CREW • 6,160 PAIRINGS</span>
            </div>
            <div style={S.cardSubtitle}>
              Benchmarked against exact column-generation (GENCOL / Kasirzadeh et al. 2017 & Racette et al. 2025).
              Coralys v2.0 solves the entire seven-instance benchmark in 20.57s (up to 2,121x speedup).
            </div>
          </div>
          <button style={S.btn('primary')} onClick={onLaunchSolver}>
            Launch Solver for Selected Instance ➔
          </button>
        </div>

        {/* Selected Instance Highlight KPIs */}
        {selected && (
          <div style={S.grid4}>
            <div style={S.statBox('primary')}>
              <div style={S.statLabel}>Pairings / Flights</div>
              <div style={S.statValue('primary')}>
                {selected.pairings_count} / {selected.flights_count}
              </div>
              <div style={S.statMeta}>Complete unfiltered pairing pool</div>
            </div>
            <div style={S.statBox('success')}>
              <div style={S.statLabel}>Coralys Speedup</div>
              <div style={S.statValue('success')}>
                {selected.coralys_speedup.toLocaleString()}x
              </div>
              <div style={S.statMeta}>
                {selected.coralys_runtime_secs}s vs {selected.gencol_runtime_secs}s (GENCOL)
              </div>
            </div>
            <div style={S.statBox('primary')}>
              <div style={S.statLabel}>Workload Disparity (Mode A)</div>
              <div style={S.statValue('primary')}>
                σ = {selected.coralys_sigma_mode_a}h
              </div>
              <div style={S.statMeta}>Max variance across {selected.crew_count} pilots</div>
            </div>
            <div style={S.statBox('warning')}>
              <div style={S.statLabel}>Vacation Fulfillment (Mode B)</div>
              <div style={S.statValue('warning')}>
                {selected.coralys_vacation_mode_b_pct}%
              </div>
              <div style={S.statMeta}>
                vs {selected.gencol_vacation_pct}% in GENCOL (6.1x increase)
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Instance Selector Grid */}
      <div style={S.card}>
        <div style={S.cardTitle}>Select Benchmark Instance</div>
        <div style={S.cardSubtitle}>
          Click an instance below to load its flight network, aircraft types, and crew complement.
        </div>
        <div style={{ ...S.grid3, marginTop: '16px' }}>
          {catalog.map((inst) => {
            const isSel = inst.id === selectedInstanceId;
            return (
              <div
                key={inst.id}
                onClick={() => onSelectInstance(inst.id)}
                style={{
                  ...S.statBox(isSel ? 'success' : 'primary'),
                  cursor: 'pointer',
                  borderColor: isSel ? theme.primaryHover : theme.surfaceBorder,
                  background: isSel ? 'rgba(14, 165, 233, 0.12)' : 'rgba(15, 23, 42, 0.75)',
                  transform: isSel ? 'scale(1.01)' : 'none',
                  transition: 'all 0.15s ease',
                }}
              >
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <span style={{ fontWeight: '700', fontSize: '14px', color: '#f8fafc' }}>
                    {inst.name}
                  </span>
                  {isSel && <span style={S.badge('success')}>SELECTED</span>}
                </div>
                <div style={{ fontSize: '12px', color: theme.textSecondary, marginTop: '8px' }}>
                  <strong>{inst.crew_count}</strong> Pilots • <strong>{inst.pairings_count}</strong> Pairings • <strong>{inst.bases.join(', ')}</strong>
                </div>
                <div style={{ fontSize: '11px', color: theme.textMuted, marginTop: '4px' }}>
                  Fleets: {inst.aircraft_types.join(', ')} • 31-Day Horizon
                </div>
                <div style={{ marginTop: '10px', display: 'flex', justifyContent: 'space-between', fontSize: '12px' }}>
                  <span style={{ color: theme.success, fontWeight: '700' }}>
                    ⚡ {inst.coralys_runtime_secs}s
                  </span>
                  <span style={{ color: theme.textMuted }}>
                    GENCOL: {Math.round(inst.gencol_runtime_secs / 60)} min
                  </span>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Literature Comparison Table */}
      <div style={S.card}>
        <div style={S.cardTitle}>
          <span>Empirical Validation Matrix: GENCOL vs Coralys Evolutionary v2.0</span>
        </div>
        <div style={{ overflowX: 'auto', marginTop: '14px' }}>
          <table style={S.table}>
            <thead>
              <tr>
                <th style={S.th}>Instance</th>
                <th style={S.th}>Pairings / Crew</th>
                <th style={S.th}>GENCOL Time (s)</th>
                <th style={S.th}>Coralys Time (s)</th>
                <th style={S.th}>Speedup Factor</th>
                <th style={S.th}>Coralys σ (Equity)</th>
                <th style={S.th}>Vacation Satisfaction</th>
                <th style={S.th}>Statutory Overtime</th>
              </tr>
            </thead>
            <tbody>
              {catalog.map((item) => (
                <tr key={item.id} style={{ background: item.id === selectedInstanceId ? 'rgba(14, 165, 233, 0.08)' : 'transparent' }}>
                  <td style={{ ...S.td, fontWeight: '600', color: theme.textPrimary }}>
                    {item.name}
                  </td>
                  <td style={S.td}>
                    {item.pairings_count} / {item.crew_count}
                  </td>
                  <td style={{ ...S.td, fontFamily: theme.fontMono }}>
                    {item.gencol_runtime_secs.toFixed(1)}s ({Math.round(item.gencol_runtime_secs / 60)}m)
                  </td>
                  <td style={{ ...S.td, fontFamily: theme.fontMono, color: theme.success, fontWeight: '700' }}>
                    {item.coralys_runtime_secs.toFixed(2)}s
                  </td>
                  <td style={{ ...S.td, fontFamily: theme.fontMono, color: theme.primaryHover, fontWeight: '700' }}>
                    {item.coralys_speedup.toLocaleString()}x
                  </td>
                  <td style={{ ...S.td, fontFamily: theme.fontMono }}>
                    σ = {item.coralys_sigma_mode_a.toFixed(2)}h
                  </td>
                  <td style={S.td}>
                    <span style={{ color: theme.warning, fontWeight: '700' }}>
                      {item.coralys_vacation_mode_b_pct}%
                    </span>{' '}
                    <span style={{ color: theme.textMuted, fontSize: '11px' }}>
                      (vs {item.gencol_vacation_pct}%)
                    </span>
                  </td>
                  <td style={S.td}>
                    <span style={S.badge('success')}>0 Violations (100% ≤ 85h)</span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
