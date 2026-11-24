import React, { useState } from 'react';
import { S, theme } from '../styles';

export default function IropsConsole({
  rotations = [],
  onRunIropsRecovery,
  iropsResult,
  recovering,
}) {
  const [frozenDay, setFrozenDay] = useState(10);
  const [sickCrewId, setSickCrewId] = useState('CRW-002');
  const [cancelledFlight, setCancelledFlight] = useState('');
  const [delayedFlight, setDelayedFlight] = useState('FL-001-00-1');
  const [delayMinutes, setDelayMinutes] = useState(60);

  const handleTriggerRecovery = () => {
    onRunIropsRecovery({
      frozen_until_day: frozenDay,
      sick_crew_ids: sickCrewId ? [sickCrewId] : [],
      cancelled_flight_ids: cancelledFlight ? [cancelledFlight] : [],
      delayed_flights: delayedFlight
        ? [{ flight_id: delayedFlight, delay_minutes: Number(delayMinutes) }]
        : [],
    });
  };

  return (
    <div>
      <div style={S.grid2}>
        {/* Left Column: Disruption Injector */}
        <div style={S.card}>
          <div style={S.cardTitle}>
            <span>1. Day-of-Operations Disruption Injector</span>
            <span style={S.badge('danger')}>OCC DISRUPTION</span>
          </div>
          <div style={S.cardSubtitle}>
            Simulate unscheduled crew illness, weather ground stops, or flight delays in the Airline Operations Control Center.
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '16px', marginTop: '16px' }}>
            <div>
              <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '6px' }}>
                <span style={S.statLabel}>Frozen Operational Window: Day {frozenDay}</span>
                <span style={{ fontSize: '11px', color: theme.textMuted }}>Days 1..{frozenDay} locked</span>
              </div>
              <input
                type="range"
                min="1"
                max="28"
                value={frozenDay}
                onChange={(e) => setFrozenDay(Number(e.target.value))}
                style={{ width: '100%', accentColor: theme.primary }}
              />
              <div style={{ fontSize: '11px', color: theme.textMuted, marginTop: '4px' }}>
                Flights prior to Day {frozenDay} are already operated or in progress and cannot be modified.
              </div>
            </div>

            <div>
              <label style={S.statLabel}>Ground Crew Member (Sick Call)</label>
              <select
                style={S.select}
                value={sickCrewId}
                onChange={(e) => setSickCrewId(e.target.value)}
              >
                <option value="">None</option>
                {rotations.slice(0, 15).map((r) => (
                  <option key={r.crew_id} value={r.crew_id}>
                    {r.name} ({r.crew_id} - Base {r.base}) - {r.flying_hours.toFixed(1)}h
                  </option>
                ))}
              </select>
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px' }}>
              <div>
                <label style={S.statLabel}>Delayed Flight ID</label>
                <input
                  type="text"
                  style={S.input}
                  placeholder="e.g. AC101"
                  value={delayedFlight}
                  onChange={(e) => setDelayedFlight(e.target.value)}
                />
              </div>
              <div>
                <label style={S.statLabel}>Delay Duration</label>
                <select
                  style={S.select}
                  value={delayMinutes}
                  onChange={(e) => setDelayMinutes(Number(e.target.value))}
                >
                  <option value={30}>30 Minutes</option>
                  <option value={60}>60 Minutes (ATC Ground Stop)</option>
                  <option value={120}>120 Minutes (Mechanical Delay)</option>
                </select>
              </div>
            </div>

            <div>
              <label style={S.statLabel}>Cancelled Flight Leg ID</label>
              <input
                type="text"
                style={S.input}
                placeholder="Optional flight cancellation (e.g. FL-002-01-1)"
                value={cancelledFlight}
                onChange={(e) => setCancelledFlight(e.target.value)}
              />
            </div>

            <button
              style={{ ...S.btn(recovering ? 'neutral' : 'danger', 'lg'), marginTop: '8px' }}
              onClick={handleTriggerRecovery}
              disabled={recovering}
            >
              {recovering ? 'Resolving OCC IROPS...' : '⚡ Trigger Coralys IROPS Recovery'}
            </button>
          </div>
        </div>

        {/* Right Column: Recovery Engine Capabilities */}
        <div style={S.card}>
          <div style={S.cardTitle}>
            <span>2. Coralys IROPS Architecture</span>
            <span style={S.badge('success')}>ZERO DELAY VIOLATIONS</span>
          </div>
          <div style={S.cardSubtitle}>
            How the Coralys Decision Engine preserves 100% statutory legality during real-time recovery.
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '12px', marginTop: '16px' }}>
            <div style={{ padding: '12px', background: 'rgba(15, 23, 42, 0.6)', borderRadius: '8px', borderLeft: `3px solid ${theme.primary}` }}>
              <strong style={{ fontSize: '13px', color: '#f8fafc' }}>Frozen Prefix Integrity</strong>
              <div style={{ fontSize: '12px', color: theme.textSecondary, marginTop: '4px' }}>
                All operated flight legs up to Day {frozenDay} remain untouched. Only downstream assignments are repaired.
              </div>
            </div>

            <div style={{ padding: '12px', background: 'rgba(15, 23, 42, 0.6)', borderRadius: '8px', borderLeft: `3px solid ${theme.indigo}` }}>
              <strong style={{ fontSize: '13px', color: '#f8fafc' }}>Standby Reserve Sizing & Dispatch</strong>
              <div style={{ fontSize: '12px', color: theme.textSecondary, marginTop: '4px' }}>
                Pre-positioned reserve pilots are seamlessly substituted into disrupted pairings without exceeding CAR 700 duty limits.
              </div>
            </div>

            <div style={{ padding: '12px', background: 'rgba(15, 23, 42, 0.6)', borderRadius: '8px', borderLeft: `3px solid ${theme.success}` }}>
              <strong style={{ fontSize: '13px', color: '#f8fafc' }}>Sub-Second Turnaround</strong>
              <div style={{ fontSize: '12px', color: theme.textSecondary, marginTop: '4px' }}>
                Re-optimization solves in under 500ms, enabling real-time dispatch decisions while passengers and aircraft wait.
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Recovery Result Banner */}
      {iropsResult && (
        <div style={{ ...S.card, borderColor: theme.successBorder, background: 'rgba(16, 185, 129, 0.08)' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <div>
              <div style={{ fontSize: '17px', fontWeight: '800', color: theme.success }}>
                ✓ OCC IROPS Disruption Successfully Resolved
              </div>
              <div style={{ fontSize: '13px', color: theme.textSecondary, marginTop: '4px' }}>
                {iropsResult.summary_message}
              </div>
            </div>
            <div style={{ display: 'flex', gap: '12px' }}>
              <div style={S.statBox('success')}>
                <div style={S.statLabel}>Pairings Rescued</div>
                <div style={S.statValue('success')}>{iropsResult.rescued_pairings_count}</div>
              </div>
              <div style={S.statBox('primary')}>
                <div style={S.statLabel}>Recovery Time</div>
                <div style={S.statValue('primary')}>{iropsResult.recovery_runtime_secs.toFixed(2)}s</div>
              </div>
              <div style={S.statBox('indigo')}>
                <div style={S.statLabel}>Reserve Callouts</div>
                <div style={S.statValue('indigo')}>{iropsResult.reserve_callouts_count}</div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
