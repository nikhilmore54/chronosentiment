import type { StaffMember, ImportSummary, ScheduleResult } from './WorkflowTypes';
import { SHIFT_CYCLE } from './WorkflowTypes';

// ─── CSV parsing ──────────────────────────────────────────────────────────────

export function parseStaffCSV(text: string): { staff: StaffMember[]; errors: string[] } {
  const lines = text.trim().split('\n').map(l => l.trim()).filter(Boolean);
  if (lines.length < 2) return { staff: [], errors: ['CSV must have a header row and at least one data row.'] };

  const header = lines[0].toLowerCase().split(',').map(h => h.trim());
  const idIdx = header.indexOf('id');
  const contractIdx = header.indexOf('contract');
  const skillsIdx = header.indexOf('skills');

  const errors: string[] = [];
  if (idIdx === -1) errors.push('Missing column: id');
  if (contractIdx === -1) errors.push('Missing column: contract');
  if (skillsIdx === -1) errors.push('Missing column: skills');
  if (errors.length > 0) return { staff: [], errors };

  const staff: StaffMember[] = [];
  const seenIds = new Set<string>();

  for (let i = 1; i < lines.length; i++) {
    const cols = lines[i].split(',').map(c => c.trim());
    const id = cols[idIdx];
    const contract = cols[contractIdx];
    const skillsRaw = cols[skillsIdx];

    if (!id) { errors.push(`Row ${i + 1}: missing id`); continue; }
    if (!contract) { errors.push(`Row ${i + 1}: missing contract for ${id}`); continue; }
    if (!skillsRaw) { errors.push(`Row ${i + 1}: missing skills for ${id}`); continue; }
    if (seenIds.has(id)) { errors.push(`Row ${i + 1}: duplicate id "${id}"`); continue; }

    seenIds.add(id);
    staff.push({ id, contract, skills: skillsRaw.split(';').map(s => s.trim()).filter(Boolean) });
  }

  return { staff, errors };
}

// ─── Import validation summary ────────────────────────────────────────────────

export function buildImportSummary(staff: StaffMember[]): ImportSummary {
  const contracts = [...new Set(staff.map(s => s.contract))];
  const skills = [...new Set(staff.flatMap(s => s.skills))];
  const warnings: string[] = [];
  if (staff.length < 3) warnings.push('Very small team — schedule may have coverage gaps.');
  const unknownContracts = contracts.filter(c => !['FullTime', 'PartTime', 'Night'].includes(c));
  if (unknownContracts.length > 0) warnings.push(`Unknown contract types: ${unknownContracts.join(', ')}`);
  return { staffCount: staff.length, contracts, skills, warnings };
}

// ─── API payload builder ──────────────────────────────────────────────────────

export function buildSchedulePayload(staff: StaffMember[], rulePayload: object) {
  const workers = staff.map((s, i) => ({ id: i + 1, skills: s.skills }));
  const shifts = staff.flatMap((s, wi) =>
    Array.from({ length: 28 }, (_, d) => ({
      id: wi * 28 + d + 1,
      start_hour: 8,
      duration_hours: 8,
      required_skill: s.skills[0] || 'Nurse',
    }))
  );
  return { workers, shifts, ...rulePayload };
}

// ─── Schedule builders ────────────────────────────────────────────────────────

export function buildEditableSchedule(
  staff: StaffMember[],
  apiSchedule: Record<string, number>
): Record<string, string[]> {
  const result: Record<string, string[]> = {};
  staff.forEach((s, i) => {
    result[s.id] = Array.from({ length: 28 }, (_, d) => {
      const shiftId = String(i * 28 + d + 1);
      const assigned = apiSchedule[shiftId];
      if (assigned === undefined || assigned === 0) return '';
      return SHIFT_CYCLE[d % 7];
    });
  });
  return result;
}

export function buildSyntheticSchedule(staff: StaffMember[]): Record<string, string[]> {
  const result: Record<string, string[]> = {};
  staff.forEach((s, i) => {
    result[s.id] = Array.from({ length: 28 }, (_, d) =>
      (i + d) % 4 === 3 ? '' : SHIFT_CYCLE[(i + d) % 7]
    );
  });
  return result;
}

// ─── Excel export ─────────────────────────────────────────────────────────────

export function exportRosterToExcel(staff: StaffMember[], schedule: Record<string, string[]>) {
  const startDate = new Date('2026-07-14');
  const headers = [
    'Staff',
    ...Array.from({ length: 28 }, (_, d) => {
      const dt = new Date(startDate);
      dt.setDate(dt.getDate() + d);
      return `${dt.toLocaleDateString('en-US', { weekday: 'short' })} ${dt.getDate()}/${dt.getMonth() + 1}`;
    }),
  ];

  const rows = staff.map(s => {
    const shifts = schedule[s.id] || Array(28).fill('');
    return [s.id, ...shifts.map(sh => sh || 'Off')];
  });

  const tsv = [headers, ...rows].map(row => row.join('\t')).join('\n');
  const blob = new Blob([tsv], { type: 'text/tab-separated-values' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `ultracrew_roster_${new Date().toISOString().slice(0, 10)}.xls`;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

// ─── Synthetic ScheduleResult for fallback ────────────────────────────────────

export function buildSyntheticResult(): ScheduleResult {
  return {
    schedule: {},
    metrics: { fitness: 9200, hard_violations: 0, soft_violations: 2, fairness_penalty: 1.2, fatigue_penalty: 0.8 },
    constraint_report: {
      fitness: 9200,
      is_valid: true,
      hard_violations: 0,
      soft_violations: 2,
      violated_constraints: [],
      satisfied_constraints: ['max_consecutive_working_days', 'min_consecutive_days_off'],
      warnings: ['Backend unavailable — showing synthetic schedule for demonstration.'],
    },
    recommendations: [],
  };
}