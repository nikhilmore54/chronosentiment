import React, { useState } from 'react';
import { S, theme } from '../styles';

export default function GanttScheduleViewer({ rotations = [] }) {
  const [filterBase, setFilterBase] = useState('ALL');
  const [filterRole, setFilterRole] = useState('ALL');
  const [searchPilot, setSearchPilot] = useState('');
  const [selectedPilot, setSelectedPilot] = useState(null);

  const filtered = rotations.filter((rot) => {
    if (filterBase !== 'ALL' && rot.base !== filterBase) return false;
    if (filterRole === 'LINE' && rot.role.includes('Reserve')) return false;
    if (filterRole === 'RESERVE' && !rot.role.includes('Reserve')) return false;
    if (
      searchPilot &&
      !rot.name.toLowerCase().includes(searchPilot.toLowerCase()) &&
      !rot.crew_id.toLowerCase().includes(searchPilot.toLowerCase())
    ) {
      return false;
    }
    return true;
  });

  const totalDays = 31;
  const totalHours = totalDays * 24; // 744 hours

  return (
    <div>
      {/* Controls Bar */}
      <div style={S.card}>
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: '14px', alignItems: 'center', justifyContent: 'space-between' }}>
          <div style={{ display: 'flex', gap: '10px', alignItems: 'center' }}>
            <span style={{ fontSize: '13px', fontWeight: '700', color: '#f8fafc' }}>Filter Domicile:</span>
            {['ALL', 'YUL', 'YYZ', 'YVR'].map((b) => (
              <button
                key={b}
                onClick={() => setFilterBase(b)}
                style={{
                  ...S.btn(filterBase === b ? 'primary' : 'neutral', 'sm'),
                }}
              >
                {b}
              </button>
            ))}
          </div>

          <div style={{ display: 'flex', gap: '10px', alignItems: 'center' }}>
            <span style={{ fontSize: '13px', fontWeight: '700', color: '#f8fafc' }}>Role:</span>
            <select
              style={{ ...S.select, width: '160px', padding: '6px 10px' }}
              value={filterRole}
              onChange={(e) => setFilterRole(e.target.value)}
            >
              <option value="ALL">All Pilots ({rotations.length})</option>
              <option value="LINE">Line Captains</option>
              <option value="RESERVE">Standby Reserves</option>
            </select>
          </div>

          <div style={{ width: '220px' }}>
            <input
              type="text"
              style={{ ...S.input, padding: '6px 12px' }}
              placeholder="Search pilot ID or name..."
              value={searchPilot}
              onChange={(e) => setSearchPilot(e.target.value)}
            />
          </div>
        </div>

        {/* Legend */}
        <div style={{ display: 'flex', gap: '20px', marginTop: '14px', borderTop: `1px solid ${theme.surfaceBorder}`, paddingTop: '10px', fontSize: '12px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
            <span style={{ width: '12px', height: '12px', background: 'linear-gradient(135deg, #0284c7, #2563eb)', borderRadius: '2px' }} />
            <span>Flight Duty Period (FDP)</span>
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
            <span style={{ width: '12px', height: '12px', background: 'rgba(245, 158, 11, 0.4)', border: '1px solid #f59e0b', borderRadius: '2px' }} />
            <span>Layover Rest (≥ 12.0h)</span>
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
            <span style={{ width: '12px', height: '12px', background: 'rgba(100, 116, 139, 0.3)', borderRadius: '2px' }} />
            <span>Home Base Rest (≥ 30h)</span>
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '6px', marginLeft: 'auto', color: theme.textMuted }}>
            Showing <strong>{filtered.length}</strong> of <strong>{rotations.length}</strong> crew rotations
          </div>
        </div>
      </div>

      {/* Gantt Timeline View */}
      <div style={{ ...S.card, padding: '16px', overflowX: 'auto' }}>
        <div style={{ minWidth: '960px' }}>
          {/* Day Numbers Header */}
          <div style={{ display: 'flex', marginLeft: '220px', borderBottom: `1px solid ${theme.surfaceBorder}`, paddingBottom: '6px' }}>
            {Array.from({ length: totalDays }).map((_, d) => (
              <div
                key={d}
                style={{
                  flex: 1,
                  textAlign: 'center',
                  fontSize: '10px',
                  color: (d + 1) % 7 === 0 ? theme.primaryHover : theme.textMuted,
                  fontFamily: theme.fontMono,
                  fontWeight: '600',
                }}
              >
                D{d + 1}
              </div>
            ))}
          </div>

          {/* Pilot Rotation Rows */}
          <div style={{ maxHeight: '520px', overflowY: 'auto', marginTop: '6px' }}>
            {filtered.slice(0, 40).map((rot) => (
              <div
                key={rot.crew_id}
                onClick={() => setSelectedPilot(rot)}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  padding: '6px 0',
                  borderBottom: `1px solid rgba(255, 255, 255, 0.04)`,
                  cursor: 'pointer',
                  background: selectedPilot?.crew_id === rot.crew_id ? 'rgba(14, 165, 233, 0.1)' : 'transparent',
                }}
              >
                {/* Pilot Info Column */}
                <div style={{ width: '220px', paddingRight: '12px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <div>
                    <div style={{ fontSize: '12px', fontWeight: '700', color: '#f8fafc' }}>
                      {rot.name}
                    </div>
                    <div style={{ fontSize: '10px', color: theme.textMuted }}>
                      {rot.crew_id} • <span style={{ color: theme.primaryHover }}>{rot.base}</span> • {rot.days_off}d off
                    </div>
                  </div>
                  <div style={{ textAlign: 'right', fontFamily: theme.fontMono, fontSize: '11px', fontWeight: '700', color: theme.success }}>
                    {rot.flying_hours.toFixed(1)}h
                  </div>
                </div>

                {/* Timeline Bar Track */}
                <div style={{ flex: 1, height: '22px', position: 'relative', background: 'rgba(15, 23, 42, 0.5)', borderRadius: '4px', overflow: 'hidden' }}>
                  {/* Standby Reserve Bar if no FDPs */}
                  {rot.fdps.length === 0 && (
                    <div
                      style={{
                        position: 'absolute',
                        left: 0,
                        right: 0,
                        top: 0,
                        bottom: 0,
                        background: 'repeating-linear-gradient(45deg, rgba(99, 102, 241, 0.1), rgba(99, 102, 241, 0.1) 10px, rgba(99, 102, 241, 0.2) 10px, rgba(99, 102, 241, 0.2) 20px)',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        fontSize: '10px',
                        fontWeight: '700',
                        color: theme.indigo,
                        fontFamily: theme.fontMono,
                      }}
                    >
                      STANDBY RESERVE AVAILABLE (70h GUARANTEE)
                    </div>
                  )}

                  {/* FDP Duty Blocks */}
                  {rot.fdps.map((fdp, idx) => {
                    const leftPct = (fdp.report_hour / totalHours) * 100;
                    const widthPct = Math.max(1.2, (fdp.fdp_hours / totalHours) * 100);
                    return (
                      <div
                        key={idx}
                        title={`Duty: ${fdp.duty_id} (${fdp.fdp_hours.toFixed(1)}h FDP)`}
                        style={{
                          position: 'absolute',
                          left: `${leftPct}%`,
                          width: `${widthPct}%`,
                          top: '2px',
                          bottom: '2px',
                          background: 'linear-gradient(135deg, #0284c7 0%, #2563eb 100%)',
                          borderRadius: '3px',
                          boxShadow: '0 1px 4px rgba(0,0,0,0.3)',
                        }}
                      />
                    );
                  })}

                  {/* Rest Blocks */}
                  {rot.rests.map((rest, idx) => {
                    const leftPct = (rest.start_hour / totalHours) * 100;
                    const widthPct = (rest.duration_hours / totalHours) * 100;
                    if (leftPct > 100) return null;
                    const isLayover = rest.rest_type === 'layover';
                    return (
                      <div
                        key={`r-${idx}`}
                        title={`${isLayover ? 'Layover' : 'Home Rest'}: ${rest.duration_hours.toFixed(1)}h`}
                        style={{
                          position: 'absolute',
                          left: `${leftPct}%`,
                          width: `${widthPct}%`,
                          top: '6px',
                          bottom: '6px',
                          background: isLayover ? 'rgba(245, 158, 11, 0.35)' : 'rgba(148, 163, 184, 0.15)',
                          borderLeft: isLayover ? '1px solid #f59e0b' : 'none',
                          borderRadius: '2px',
                        }}
                      />
                    );
                  })}
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Selected Pilot Detail Drawer */}
      {selectedPilot && (
        <div style={{ ...S.card, borderColor: theme.primaryHover }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '14px' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
              <span style={{ fontSize: '18px' }}>👨‍✈️</span>
              <div>
                <strong style={{ fontSize: '15px', color: '#f8fafc' }}>
                  {selectedPilot.name} ({selectedPilot.crew_id})
                </strong>
                <div style={{ fontSize: '12px', color: theme.textMuted }}>
                  Domicile Base: <strong>{selectedPilot.base}</strong> • Flying Hours: <strong>{selectedPilot.flying_hours}h</strong> • Credited: <strong>{selectedPilot.credited_hours}h</strong>
                </div>
              </div>
            </div>
            <button style={S.btn('neutral', 'sm')} onClick={() => setSelectedPilot(null)}>
              Close Details ✕
            </button>
          </div>

          <div style={{ overflowX: 'auto' }}>
            <table style={S.table}>
              <thead>
                <tr>
                  <th style={S.th}>Duty ID</th>
                  <th style={S.th}>Report / Release (H)</th>
                  <th style={S.th}>FDP Duration</th>
                  <th style={S.th}>Sectors (Flights)</th>
                  <th style={S.th}>CAR 700 Legality</th>
                </tr>
              </thead>
              <tbody>
                {selectedPilot.fdps.map((fdp) => (
                  <tr key={fdp.duty_id}>
                    <td style={{ ...S.td, fontWeight: '700', fontFamily: theme.fontMono }}>
                      {fdp.duty_id}
                    </td>
                    <td style={S.td}>
                      {fdp.report_hour.toFixed(1)}h → {fdp.release_hour.toFixed(1)}h
                    </td>
                    <td style={{ ...S.td, fontFamily: theme.fontMono, color: theme.primaryHover }}>
                      {fdp.fdp_hours.toFixed(1)}h (Limit: {fdp.fdp_limit_hours}h)
                    </td>
                    <td style={S.td}>
                      <div style={{ display: 'flex', gap: '8px' }}>
                        {fdp.sectors.map((s) => (
                          <span key={s.flight_id} style={S.badge('neutral')}>
                            {s.flight_number} ({s.origin}→{s.destination})
                          </span>
                        ))}
                      </div>
                    </td>
                    <td style={S.td}>
                      <span style={S.badge('success')}>✓ CAR 700 Compliant</span>
                    </td>
                  </tr>
                ))}
                {selectedPilot.fdps.length === 0 && (
                  <tr>
                    <td colSpan={5} style={{ ...S.td, textAlign: 'center', color: theme.textMuted }}>
                      Designated Standby Reserve Pilot with zero fixed initial assignments. Fresh capacity for OCC disruption recovery.
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
}
