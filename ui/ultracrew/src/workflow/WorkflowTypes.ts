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

// ─── P3.1: Staffing demand — explicit requirements layer ─────────────────────
//
// StaffingRequirement is the canonical demand object. Coverage is always
// computed as filledPositions / requiredPositions, never from optimizer fitness.
// The same object is used by Explore Decision, Review/Edit, and Export.

export interface StaffingRequirement {
  dayIdx: number;       // 0–27 (relative to planning horizon start)
  shiftType: string;    // 'Early' | 'Late' | 'Night'
  required: number;     // how many staff are needed for this slot
}

export interface CoverageReport {
  requiredPositions: number;   // sum of all StaffingRequirement.required
  filledPositions: number;     // how many required positions have an assignment
  coveragePct: number;         // filledPositions / requiredPositions × 100, 1 decimal
  gapPositions: number;        // requiredPositions - filledPositions
}

// ─── P3: Roster alternative (one option presented to the scheduler) ───────────

export interface RosterAlternativeMetrics {
  coverage: number;        // 0.0–1.0 (filledPositions / requiredPositions)
  filled_positions: number;
  required_positions: number;
  fairness_penalty: number;
  utilization: number;     // 0.0–1.0 (total assignments / total slots)
  cost: number;
  diff_from_recommended: number; // number of assignments that differ
}

export interface RosterAlternative {
  id: string;
  label: string;
  metrics: RosterAlternativeMetrics;
  schedule: Record<string, string[]>; // staffId → 28-day shift array
  reasons: string[];                  // why this option is notable
}

// ─── P3.3: Assignment provenance — four states, emitted not diffed ────────────
//
// Every assignment cell in the post-redistribution roster carries exactly one
// of these four states. The state is emitted by the redistribution operation
// at the moment it acts on the cell — never derived by post-hoc diff.

export type AssignmentProvenanceState =
  | 'original'           // from the selected alternative, untouched
  | 'scheduler_edit'     // scheduler-authored, locked — redistribution must not touch
  | 'system_reassignment' // UltraRoster changed this during redistribution
  | 'unchanged';         // redistribution ran but left this cell the same as original

export interface ChangeRecord {
  assignmentId: string;          // `${staffId}:${dayIdx}`
  previousValue: string;         // shift value before redistribution ('' = off)
  newValue: string;              // shift value after redistribution
  redistributionOperationId: string;
  reason: string;                // human-readable, e.g. "coverage gap on day 14 Night shift"
  timestamp: string;             // ISO-8601
}

export interface RedistributionLog {
  operationId: string;           // unique per redistribution run (crypto.randomUUID or fallback)
  timestamp: string;             // ISO-8601 when redistribution completed
  schedulerEditsPreserved: number;
  assignmentsReassigned: number;
  lockedAssignmentsChanged: number; // invariant: always 0 — non-zero blocks export
  changeRecords: ChangeRecord[];
  // Provenance map: `${staffId}:${dayIdx}` → AssignmentProvenanceState
  provenanceMap: Record<string, AssignmentProvenanceState>;
}

// ─── P3: Decision record — what the scheduler chose ──────────────────────────

export interface SchedulerDecision {
  decision_id: string;
  created_at_iso: string;
  recommended_id: string;
  selected_id: string;
  overrode_recommendation: boolean;
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
  // P3: alternatives presented to the scheduler (may be empty if engine returns only one)
  alternatives?: RosterAlternative[];
  recommended_alternative_id?: string;
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
  { num: 4, label: 'Explore Decision' },
  { num: 5, label: 'Review & Edit' },
  { num: 6, label: 'Export' },
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
// 20-person realistic hospital roster: mixed contracts, multi-skill workers,
// realistic skill distribution (4 HeadNurse, 12 Nurse, 4 Nurse+HeadNurse).

export const SAMPLE_CSV = `id,contract,skills
Sarah_Chen,FullTime,HeadNurse;Nurse
Marcus_Webb,FullTime,Nurse
Priya_Nair,FullTime,HeadNurse;Nurse
James_Okafor,FullTime,Nurse
Elena_Vasquez,FullTime,Nurse
Tom_Lindqvist,PartTime,Nurse
Aisha_Diallo,FullTime,HeadNurse
Ravi_Sharma,FullTime,Nurse
Fatima_Al-Hassan,PartTime,Nurse
Daniel_Park,FullTime,Nurse
Ingrid_Sorensen,Night,Nurse
Carlos_Mendez,FullTime,Nurse
Yuki_Tanaka,PartTime,HeadNurse;Nurse
Kwame_Asante,FullTime,Nurse
Lena_Hoffmann,FullTime,Nurse
Amara_Osei,Night,Nurse
Patrick_Brennan,FullTime,HeadNurse
Nadia_Petrov,PartTime,Nurse
Soren_Andersen,FullTime,Nurse
Mei_Lin_Zhou,FullTime,Nurse`;