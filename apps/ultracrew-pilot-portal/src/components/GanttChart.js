import React from 'react';

export default function GanttChart({
  result,
  horizonHours,
  shiftsMap,
  workersMap,
  ganttFilter,
  setGanttFilter,
  layoverMarkers
}) {
  if (!result.schedule || Object.keys(result.schedule).length === 0) return null;

  const HORIZON_HRS = horizonHours;
  const workerShifts = {}; 
  Object.entries(result.schedule).forEach(([shiftId, workerId]) => {
    const sid = parseInt(shiftId);
    const meta = shiftsMap[sid] || {};
    const day = Math.floor((sid - 1) / 3) + 1;
    const slotNames = ['Morning', 'Afternoon', 'Night'];
    const slot = (sid - 1) % 3;
    const name = `Flt ${String(sid).padStart(2, '0')} · Day ${day} ${slotNames[slot]}`;
    if (!workerShifts[workerId]) workerShifts[workerId] = [];
    workerShifts[workerId].push({ shiftId: sid, start_hour: meta.start_hour || 0, duration_hours: meta.duration_hours || 8, name });
  });

  const workerIds = Object.keys(workerShifts).map(Number).sort((a, b) => a - b);
  const numDays = Math.ceil(HORIZON_HRS / 24) + 1;
  const dayTicks = Array.from({ length: numDays }, (_, i) => i);

  return (
    <div style={{ marginBottom: '20px' }}>
      <div style={{ fontSize: '13px', fontWeight: '600', color: '#94a3b8', marginBottom: '10px', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
        Schedule — {workerIds.length} workers · {Object.keys(result.schedule).length} shifts · {Math.ceil(HORIZON_HRS / 24)}-day horizon
      </div>
      <div style={{ display: 'flex', marginLeft: '72px', marginBottom: '4px', position: 'relative', height: '18px' }}>
        {dayTicks.map(d => (
          <div key={d} style={{ position: 'absolute', left: `${(d * 24 / HORIZON_HRS) * 100}%`, fontSize: '10px', color: '#475569', transform: 'translateX(-50%)' }}>
            {d === 0 ? 'Day 1' : d % 2 === 0 ? `D${d + 1}` : ''}
          </div>
        ))}
      </div>
      <div style={{ maxHeight: '340px', overflowY: 'auto', border: '1px solid #1e293b', borderRadius: '8px', background: '#0f172a' }}>
        {(ganttFilter
          ? workerIds.filter(wid => workerShifts[wid].some(s => (shiftsMap[s.shiftId] || {}).flight_id === ganttFilter))
          : workerIds
        ).map(wid => {
          const wMeta = workersMap[wid] || {};
          const role = wMeta.role || '';
          const typeRating = wMeta.type_rating || '';
          const skill = (!role && !typeRating && wMeta.skills && wMeta.skills.length > 0) ? wMeta.skills[0] : '';
          const skillShort = skill ? skill.replace(/^[^-]+-/, '') : '';
          const skillFull = skill || (typeRating ? `${typeRating}` : '');

          return (
            <div key={wid} style={{ display: 'flex', alignItems: 'center', borderBottom: '1px solid #1e293b', minHeight: '44px' }}>
              <div style={{ width: '80px', flexShrink: 0, paddingLeft: '8px', paddingRight: '4px' }}>
                <div style={{ fontSize: '11px', color: '#94a3b8', fontWeight: '700' }}>W{wid}</div>
                {role && <div style={{ fontSize: '9px', color: '#64748b', marginTop: '1px', lineHeight: '1.2' }}>{role}</div>}
                {skillFull && <div style={{ fontSize: '9px', color: '#38bdf8', marginTop: '1px', lineHeight: '1.2', fontFamily: 'monospace' }}>{skillFull}</div>}
                {skillShort && !skillFull && <div style={{ fontSize: '9px', color: '#38bdf8', marginTop: '1px', lineHeight: '1.2', fontFamily: 'monospace' }}>{skillShort}</div>}
              </div>
              <div style={{ flex: 1, position: 'relative', height: '28px', background: '#1e293b' }}>
                {dayTicks.slice(1).map(d => (
                  <div key={d} style={{ position: 'absolute', left: `${(d * 24 / HORIZON_HRS) * 100}%`, top: 0, bottom: 0, width: '1px', background: '#334155' }} />
                ))}
                {workerShifts[wid].map(s => {
                  const leftPct = (s.start_hour / HORIZON_HRS) * 100;
                  const widthPct = (s.duration_hours / HORIZON_HRS) * 100;
                  const slotColors = { Morning: '#3b82f6', Afternoon: '#8b5cf6', Night: '#06b6d4' };
                  const colorKey = s.name.includes('Morning') ? 'Morning' : s.name.includes('Afternoon') ? 'Afternoon' : 'Night';
                  const shiftMeta = shiftsMap[s.shiftId] || {};
                  const aircraft = shiftMeta.aircraft_type || '';
                  const flightId = shiftMeta.flight_id || '';
                  const route = shiftMeta.route || '';
                  const crewRole = shiftMeta.crew_role || '';
                  const isFiltered = ganttFilter === flightId;
                  const color = isFiltered ? '#f59e0b' : slotColors[colorKey];
                  const reqSkill = shiftMeta.required_skill || s.required_skill || '';
                  const routeDisplay = route || (reqSkill ? `Skill: ${reqSkill}` : '');
                  const workerSkillDisplay = skillFull || typeRating || '';
                  const tooltip = [
                    flightId ? `${flightId}${routeDisplay ? ` · ${routeDisplay}` : ''}` : `Shift ${s.shiftId}${routeDisplay ? ` · ${routeDisplay}` : ''}`,
                    aircraft ? `Aircraft: ${aircraft}` : '',
                    crewRole ? `Position: ${crewRole}` : '',
                    `Worker W${wid}${workerSkillDisplay ? ` · ${workerSkillDisplay}` : ''}`,
                    `Start: h${s.start_hour}  Duration: ${s.duration_hours}h`,
                    isFiltered ? 'Click to clear filter' : 'Click to filter crew for this shift',
                  ].filter(Boolean).join('\n');
                  return (
                    <div key={s.shiftId} title={tooltip}
                      onClick={() => setGanttFilter(prev => prev === flightId ? null : flightId)}
                      style={{
                        position: 'absolute', left: `${leftPct}%`, width: `${widthPct}%`,
                        top: '3px', bottom: '3px', background: color, borderRadius: '3px',
                        display: 'flex', alignItems: 'center', justifyContent: 'center',
                        overflow: 'hidden', cursor: 'pointer',
                        outline: isFiltered ? '2px solid #fbbf24' : 'none',
                        boxShadow: isFiltered ? '0 0 0 2px #78350f' : 'none',
                      }}>
                      <span style={{ fontSize: '9px', color: '#fff', fontWeight: '700', whiteSpace: 'nowrap', padding: '0 3px', textOverflow: 'ellipsis', overflow: 'hidden' }}>
                        {flightId || s.name.split(' · ')[0]}
                      </span>
                    </div>
                  );
                })}
                {layoverMarkers.filter(m => {
                  if (m.type === 'flight_id') {
                    return workerShifts[wid].some(s => (shiftsMap[s.shiftId] || {}).flight_id === m.flight_id);
                  }
                  return m.worker_id === wid;
                }).map((m, i) => {
                  const leftPct = (m.start_hour / HORIZON_HRS) * 100;
                  const widthPct = (m.duration_hours / HORIZON_HRS) * 100;
                  const isViolation = m.label && m.label.includes('VIOLATION');
                  const isDeadhead = m.label && m.label.includes('Deadhead');
                  const bg = isViolation ? 'repeating-linear-gradient(45deg, #7f1d1d 0, #7f1d1d 3px, #991b1b 3px, #991b1b 6px)'
                    : isDeadhead ? 'repeating-linear-gradient(-45deg, #1e3a5f 0, #1e3a5f 3px, #1e40af 3px, #1e40af 6px)'
                    : 'repeating-linear-gradient(45deg, #78350f 0, #78350f 3px, #92400e 3px, #92400e 6px)';
                  const border = isViolation ? '1px solid #ef4444' : isDeadhead ? '1px solid #3b82f6' : '1px solid #f59e0b';
                  return (
                    <div key={`lo-${i}`} title={`${m.label} (Start: h${m.start_hour}, Dur: ${m.duration_hours}h)`}
                      style={{
                        position: 'absolute', left: `${leftPct}%`, width: `${widthPct}%`,
                        top: '10px', bottom: '10px', background: bg, border: border, borderRadius: '2px',
                        pointerEvents: 'none', opacity: 0.8
                      }} />
                  );
                })}
              </div>
            </div>
          );
        })}
      </div>
      {ganttFilter && (
        <div style={{ marginTop: '8px', fontSize: '11px', color: '#94a3b8', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <span>Showing only crew assigned to flight <strong style={{ color: '#f59e0b' }}>{ganttFilter}</strong></span>
          <button onClick={() => setGanttFilter(null)} style={{ background: 'none', border: '1px solid #78350f', borderRadius: '4px', color: '#f59e0b', fontSize: '10px', padding: '1px 6px', cursor: 'pointer' }}>Clear filter</button>
        </div>
      )}
      <div style={{ display: 'flex', gap: '12px', marginTop: '8px', fontSize: '11px', color: '#64748b', flexWrap: 'wrap' }}>
        <span><span style={{ display: 'inline-block', width: '10px', height: '10px', background: '#3b82f6', borderRadius: '2px', marginRight: '4px' }} />Duty (Morning)</span>
        <span><span style={{ display: 'inline-block', width: '10px', height: '10px', background: '#8b5cf6', borderRadius: '2px', marginRight: '4px' }} />Duty (Afternoon)</span>
        <span><span style={{ display: 'inline-block', width: '10px', height: '10px', background: '#06b6d4', borderRadius: '2px', marginRight: '4px' }} />Duty (Night)</span>
        <span><span style={{ display: 'inline-block', width: '10px', height: '10px', background: 'repeating-linear-gradient(45deg,#78350f 0,#78350f 3px,#92400e 3px,#92400e 6px)', border: '1px solid #f59e0b', borderRadius: '2px', marginRight: '4px' }} />🛏 Layover (rest ≥10h)</span>
        <span><span style={{ display: 'inline-block', width: '10px', height: '10px', background: 'repeating-linear-gradient(-45deg,#1e3a5f 0,#1e3a5f 3px,#1e40af 3px,#1e40af 6px)', border: '1px solid #3b82f6', borderRadius: '2px', marginRight: '4px' }} />✈ Deadhead (short rest)</span>
        <span><span style={{ display: 'inline-block', width: '10px', height: '10px', background: 'repeating-linear-gradient(45deg,#7f1d1d 0,#7f1d1d 3px,#991b1b 3px,#991b1b 6px)', border: '1px solid #ef4444', borderRadius: '2px', marginRight: '4px' }} />⚠ FDP violation</span>
      </div>
    </div>
  );
}
