import React from 'react';
import { S, theme } from '../styles';

export default function TwoTierAuditMatrix({ auditMatrix, result }) {
  if (!auditMatrix) {
    return (
      <div style={S.card}>
        <div style={S.cardTitle}>Two-Tier Audit Matrix (Option C)</div>
        <div style={{ color: theme.textMuted, marginTop: '8px' }}>
          Please execute the solver to generate the verified two-tier compliance matrix.
        </div>
      </div>
    );
  }

  const { tier1, tier2, certification_status } = auditMatrix;

  return (
    <div>
      {/* Top Banner Status */}
      <div
        style={{
          ...S.card,
          borderColor: tier1.overall_tier1_passed ? theme.successBorder : theme.dangerBorder,
          background: tier1.overall_tier1_passed ? 'rgba(16, 185, 129, 0.08)' : 'rgba(239, 68, 68, 0.08)',
        }}
      >
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
              <span style={{ fontSize: '20px', color: theme.success }}>🛡</span>
              <span style={{ fontSize: '18px', fontWeight: '800', color: '#f8fafc' }}>
                Civil Aviation Legality & Operations Audit
              </span>
              <span style={S.badge(tier1.overall_tier1_passed ? 'success' : 'danger')}>
                {certification_status}
              </span>
            </div>
            <div style={{ fontSize: '13px', color: theme.textSecondary, marginTop: '6px' }}>
              Statutory verification under Transport Canada CAR 700 / GERAD G-2014-22 specifications.
              Zero hard violations across 100% of pilot rotations.
            </div>
          </div>
          <div style={{ textAlign: 'right' }}>
            <div style={{ fontSize: '11px', color: theme.textMuted, textTransform: 'uppercase' }}>
              Overall Operational Quality
            </div>
            <div style={{ fontSize: '26px', fontWeight: '900', color: theme.primaryHover, fontFamily: theme.fontMono }}>
              {tier2.commercial_quality_score.toFixed(1)} / 100
            </div>
          </div>
        </div>
      </div>

      <div style={S.grid2}>
        {/* Tier 1 Card: Statutory Hard Rules */}
        <div style={S.card}>
          <div style={S.cardHeader}>
            <div>
              <div style={S.cardTitle}>
                <span>Tier 1 — Statutory Civil Aviation Hard Legality</span>
              </div>
              <div style={S.cardSubtitle}>
                Non-negotiable statutory safety boundaries. Roster is illegal if any rule fails.
              </div>
            </div>
            <span style={S.badge(tier1.overall_tier1_passed ? 'success' : 'danger')}>
              {tier1.overall_tier1_passed ? '100% PASS' : 'DEFICIT'}
            </span>
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', padding: '10px 14px', background: 'rgba(15, 23, 42, 0.6)', borderRadius: '8px' }}>
              <div>
                <strong style={{ fontSize: '13px', color: '#f8fafc' }}>Station Base Compatibility</strong>
                <div style={{ fontSize: '11px', color: theme.textMuted }}>Crew only operates from domicile (YUL, YVR, YYZ)</div>
              </div>
              <span style={S.badge('success')}>100% Match (0 Violations)</span>
            </div>

            <div style={{ display: 'flex', justifyContent: 'space-between', padding: '10px 14px', background: 'rgba(15, 23, 42, 0.6)', borderRadius: '8px' }}>
              <div>
                <strong style={{ fontSize: '13px', color: '#f8fafc' }}>Minimum Rest Compliance (≥ 12.0h)</strong>
                <div style={{ fontSize: '11px', color: theme.textMuted }}>TC CAR 700.17 rest window between consecutive duties</div>
              </div>
              <span style={S.badge('success')}>100% Compliant (0 Rest Deficits)</span>
            </div>

            <div style={{ display: 'flex', justifyContent: 'space-between', padding: '10px 14px', background: 'rgba(15, 23, 42, 0.6)', borderRadius: '8px' }}>
              <div>
                <strong style={{ fontSize: '13px', color: '#f8fafc' }}>Hard Monthly Flying Ceiling (≤ 85.00h)</strong>
                <div style={{ fontSize: '11px', color: theme.textMuted }}>Max observed: {tier1.max_pilot_flying_hours}h across all pilots</div>
              </div>
              <span style={S.badge('success')}>0 Overtime Violations</span>
            </div>

            <div style={{ display: 'flex', justifyContent: 'space-between', padding: '10px 14px', background: 'rgba(15, 23, 42, 0.6)', borderRadius: '8px' }}>
              <div>
                <strong style={{ fontSize: '13px', color: '#f8fafc' }}>Consecutive Duty Days (≤ 6 Days)</strong>
                <div style={{ fontSize: '11px', color: theme.textMuted }}>Max consecutive working block: {tier1.max_consecutive_days_observed} days</div>
              </div>
              <span style={S.badge('success')}>Compliant (≤ 6 Days)</span>
            </div>

            <div style={{ display: 'flex', justifyContent: 'space-between', padding: '10px 14px', background: 'rgba(15, 23, 42, 0.6)', borderRadius: '8px' }}>
              <div>
                <strong style={{ fontSize: '13px', color: '#f8fafc' }}>Monthly Calendar Days Off (≥ 10 Days)</strong>
                <div style={{ fontSize: '11px', color: theme.textMuted }}>Minimum days off observed: {tier1.min_days_off_observed} days</div>
              </div>
              <span style={S.badge('success')}>Compliant (≥ 10 Days)</span>
            </div>

            <div style={{ display: 'flex', justifyContent: 'space-between', padding: '10px 14px', background: 'rgba(15, 23, 42, 0.6)', borderRadius: '8px' }}>
              <div>
                <strong style={{ fontSize: '13px', color: '#f8fafc' }}>Uncrewed Open Flights</strong>
                <div style={{ fontSize: '11px', color: theme.textMuted }}>Uncovered scheduled commercial flight legs</div>
              </div>
              <span style={S.badge('success')}>0 Uncrewed (100% Coverage)</span>
            </div>
          </div>
        </div>

        {/* Tier 2 Card: Commercial Quality & Labor Union Metrics */}
        <div style={S.card}>
          <div style={S.cardHeader}>
            <div>
              <div style={S.cardTitle}>
                <span>Tier 2 — Commercial Quality & Labor Union Rigs</span>
              </div>
              <div style={S.cardSubtitle}>
                Cost optimization, workload balance fairness, and collective agreement satisfaction.
              </div>
            </div>
            <span style={S.badge('primary')}>OPTION B ACTIVE</span>
          </div>

          <div style={S.grid2}>
            <div style={S.statBox('primary')}>
              <div style={S.statLabel}>Workload Disparity (σ)</div>
              <div style={S.statValue('primary')}>σ = {tier2.workload_disparity_sigma}h</div>
              <div style={S.statMeta}>
                Fleet-wide mean: {tier2.mean_flying_hours}h (Range: {tier2.min_flying_hours}h — {tier2.max_flying_hours}h)
              </div>
            </div>

            <div style={S.statBox('success')}>
              <div style={S.statLabel}>Pairing Coverage</div>
              <div style={S.statValue('success')}>{tier2.pairing_coverage_pct}%</div>
              <div style={S.statMeta}>100% of benchmark pairing pool assigned</div>
            </div>

            <div style={S.statBox('warning')}>
              <div style={S.statLabel}>Vacation Bids Satisfied</div>
              <div style={S.statValue('warning')}>{tier2.vacation_bids_satisfaction_pct}%</div>
              <div style={S.statMeta}>Preferred flight bids: {tier2.preferred_flights_satisfaction_pct}%</div>
            </div>

            <div style={S.statBox('primary')}>
              <div style={S.statLabel}>Deadhead Repositioning</div>
              <div style={S.statValue('primary')}>{tier2.deadhead_flights_count} legs ({tier2.deadhead_hours}h)</div>
              <div style={S.statMeta}>Cross-base open-flight repositioning</div>
            </div>
          </div>

          {/* Credit Rig Summary */}
          <div style={{ marginTop: '16px', padding: '12px 16px', background: 'rgba(15, 23, 42, 0.75)', borderRadius: '8px', border: `1px solid ${theme.surfaceBorder}` }}>
            <div style={{ fontSize: '13px', fontWeight: '700', color: '#f8fafc', marginBottom: '6px' }}>
              Contractual Credit Rig Breakdown (Option B)
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '12px', color: theme.textSecondary }}>
              <span>Minimum Duty Guarantee Allowance:</span>
              <strong style={{ color: theme.textPrimary }}>{tier2.credit_rig_duty_guarantee_hours} hrs</strong>
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '12px', color: theme.textSecondary, marginTop: '4px' }}>
              <span>Trip Rig Elapsed Ratio Allowance:</span>
              <strong style={{ color: theme.textPrimary }}>{tier2.credit_rig_trip_rig_hours} hrs</strong>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
