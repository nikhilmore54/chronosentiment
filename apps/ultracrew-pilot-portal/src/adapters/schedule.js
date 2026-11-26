import { GERAD_INSTANCE1_WORKERS, GERAD_INSTANCE1_SHIFTS, GERAD_INSTANCE1_META } from '../geradInstance1';
import { GERAD_INSTANCE2_WORKERS, GERAD_INSTANCE2_SHIFTS, GERAD_INSTANCE2_META } from '../geradInstance2';
import { GERAD_INSTANCE3_WORKERS, GERAD_INSTANCE3_SHIFTS, GERAD_INSTANCE3_META } from '../geradInstance3';
import { GERAD_INSTANCE4_WORKERS, GERAD_INSTANCE4_SHIFTS, GERAD_INSTANCE4_META } from '../geradInstance4';
import { GERAD_INSTANCE5_WORKERS, GERAD_INSTANCE5_SHIFTS, GERAD_INSTANCE5_META } from '../geradInstance5';
import { GERAD_INSTANCE6_WORKERS, GERAD_INSTANCE6_SHIFTS, GERAD_INSTANCE6_META } from '../geradInstance6';
import { GERAD_INSTANCE7_WORKERS, GERAD_INSTANCE7_SHIFTS, GERAD_INSTANCE7_META } from '../geradInstance7';

export const GERAD_INSTANCES = {
  'gerad-instance1': { workers: GERAD_INSTANCE1_WORKERS, shifts: GERAD_INSTANCE1_SHIFTS, meta: GERAD_INSTANCE1_META },
  'gerad-instance2': { workers: GERAD_INSTANCE2_WORKERS, shifts: GERAD_INSTANCE2_SHIFTS, meta: GERAD_INSTANCE2_META },
  'gerad-instance3': { workers: GERAD_INSTANCE3_WORKERS, shifts: GERAD_INSTANCE3_SHIFTS, meta: GERAD_INSTANCE3_META },
  'gerad-instance4': { workers: GERAD_INSTANCE4_WORKERS, shifts: GERAD_INSTANCE4_SHIFTS, meta: GERAD_INSTANCE4_META },
  'gerad-instance5': { workers: GERAD_INSTANCE5_WORKERS, shifts: GERAD_INSTANCE5_SHIFTS, meta: GERAD_INSTANCE5_META },
  'gerad-instance6': { workers: GERAD_INSTANCE6_WORKERS, shifts: GERAD_INSTANCE6_SHIFTS, meta: GERAD_INSTANCE6_META },
  'gerad-instance7': { workers: GERAD_INSTANCE7_WORKERS, shifts: GERAD_INSTANCE7_SHIFTS, meta: GERAD_INSTANCE7_META }
};

export function buildGeradBenchmarkScenario(scenarioId) {
  const instance = GERAD_INSTANCES[scenarioId];
  if (!instance) throw new Error(`Unknown scenario: ${scenarioId}`);
  
  const workers = instance.workers.map((w, index) => {
    const roleStr = index % 2 === 0 ? '-CPT' : '-FO';
    const roleName = index % 2 === 0 ? 'Captain' : 'First Officer';
    return {
      id: w.id,
      skills: w.skills.map(s => s + roleStr),
      role: roleName,
      name: w.name,
      base: w.base,
      gerad_id: w.gerad_id,
      contract_type: w.contract_type,
    };
  });

  const shifts = instance.shifts.flatMap((s) => {
    const start_hour = Math.round(s.start_hour);
    const duration_hours = Math.max(1, Math.round(s.duration_hours));
    const base = {
      start_hour,
      duration_hours,
      flight_id: s.gerad_duty_id,
      gerad_duty_id: s.gerad_duty_id,
      gerad_crew_id: s.gerad_crew_id,
      flight_ids: s.flight_ids,
    };
    return [
      { id: s.id * 10 + 1, ...base, required_skill: `${s.required_skill}-CPT`, crew_role: 'Captain' },
      { id: s.id * 10 + 2, ...base, required_skill: `${s.required_skill}-FO`,  crew_role: 'First Officer' }
    ];
  });

  const layoverMarkers = [];

  return {
    workers,
    shifts,
    layoverMarkers,
    horizonHours: instance.meta.horizon_hours,
    maxHoursPerWorker: instance.meta.max_hours_per_worker,
  };
}
