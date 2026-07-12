// ─── Domain types for the UltraCrew planner workflow ─────────────────────────

export interface StaffMember {
  id: string;
  contract: string;
  skills: string[];
}

export interface ImportSummary {
  staffCount: number;
  contracts: string[];
  skills: string[];
  warnings: string[];
}

export interface RuleSet {
  id: string;
  label: string;
  description: string;
  payload: object;
}

export interface ScheduleResult {
  schedule: Record<string, number>;
  metrics: Record<string, number>;
  constraint_report: {
    fitness: number;
    is_valid: boolean;
    hard_violations: number;
    soft_violations: number;
    violated_constraints: string[];
    satisfied_constraints: string[];
    warnings: string[];
  } | null;
  recommendations: Array<{
    constraint_id: string;
    severity: string;
    explanation: string;
    recommended_action: string;
  }>;
}

// ─── Workflow step descriptor ─────────────────────────────────────────────────

export interface WorkflowStep {
  num: number;
  label: string;
}

export const WORKFLOW_STEPS: WorkflowStep[] = [
  { num: 1, label: 'Import Staff' },
  { num: 2, label: 'Select Rules' },
  { num: 3, label: 'Generate' },
  { num: 4, label: 'Review & Edit' },
  { num: 5, label: 'Export' },
];

// ─── Rule presets ─────────────────────────────────────────────────────────────

export const RULE_PRESETS: RuleSet[] = [
  {
    id: 'hospital_standard',
    label: 'Hospital Standard',
    description: 'Max 5 consecutive days, min 2 days off, no Night→Early succession, max 2 weekends/month.',
    payload: { max_consecutive_working_days: 5, min_consecutive_days_off: 2, max_working_weekends: 2 },
  },
  {
    id: 'inrc_demo',
    label: 'INRC Demo',
    description: 'INRC-II benchmark rules: 3–7 consecutive days, 1–4 days off, complete weekends.',
    payload: { min_consecutive_working_days: 3, max_consecutive_working_days: 7, min_consecutive_days_off: 1, max_consecutive_days_off: 4, complete_weekends: true, max_working_weekends: 4 },
  },
  {
    id: 'light',
    label: 'Light Rules',
    description: 'Minimal constraints for quick demos: max 6 consecutive days, 1 day off minimum.',
    payload: { max_consecutive_working_days: 6, min_consecutive_days_off: 1, max_working_weekends: 4 },
  },
];

// ─── Shift display constants ──────────────────────────────────────────────────

export const SHIFT_CYCLE = ['Early', 'Late', 'Night', '', 'Early', 'Late', ''];

export const SHIFT_COLORS: Record<string, string> = {
  Early: '#38bdf8',
  Late: '#f59e0b',
  Night: '#818cf8',
  '': 'var(--text-muted)',
};

// ─── Sample CSV for onboarding ────────────────────────────────────────────────

export const SAMPLE_CSV = `id,contract,skills
Alice,FullTime,HeadNurse
Bob,FullTime,Nurse
Carol,PartTime,Nurse
Dave,FullTime,HeadNurse
Eve,PartTime,Nurse
Frank,FullTime,Nurse
Grace,PartTime,HeadNurse
Henry,FullTime,Nurse`;