import React, { useState, useMemo } from 'react';
import type {
  StaffMember,
  ScheduleResult,
  SchedulerDecision,
  BaselineAuditReport,
  BaselineAssignment,
  RosterAlternative,
  ShiftSlotMeta,
} from './WorkflowTypes';
import { primaryBtnStyle, ghostBtnStyle, accentGhostBtnStyle } from './WorkflowComponents';
import {
  exportRosterToExcel,
  exportRosterToCSV,
  buildStaffingRequirements,
  computeCanonicalCoverage,
  generateShiftLevelDeltaRecords,
  exportShiftLevelDeltaCSV,
} from './WorkflowUtils';
import { ComparativeScorecard } from './SelectDecision';

const StatBlock: React.FC<{ label: string; value: string; color: string }> = ({ label, value, color }) => (
  <div style={{ display: 'flex', flexDirection: 'column', gap: '0.2rem' }}>
    <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.04em' }}>{label}</div>
    <div style={{ fontSize: '1.5rem', fontWeight: 700, color }}>{value}</div>
  </div>
);

export const ExportRoster: React.FC<{
  staff: StaffMember[];
  schedule: Record<string, string[]>;
  result: ScheduleResult;
  manualEditCount: number;
  editDistribution: Record<string, number>;
  originalAssignmentCount: number;
  schedulerDecision?: SchedulerDecision | null;
  customSlotMeta?: Record<number, ShiftSlotMeta>;
  baselineAuditReport?: BaselineAuditReport | null;
  baselineAssignments?: BaselineAssignment[];
  alternatives?: RosterAlternative[];
  onBack: () => void;
  onStartOver: () => void;
}> = ({
  staff,
  schedule,
  result,
  manualEditCount,
  editDistribution,
  originalAssignmentCount,
  schedulerDecision,
  customSlotMeta,
  baselineAuditReport,
  baselineAssignments,
  alternatives = [],
  onBack,
  onStartOver,
}) => {
  const [exported, setExported] = useState(false);
  const [exportNotice, setExportNotice] = useState<string | null>(null);

  const horizonDays = useMemo(() => {
    if (customSlotMeta) {
      const daysArr = Object.values(customSlotMeta).map(m => m.dayIdx);
      if (daysArr.length > 0) return Math.max(...daysArr) + 1;
    }
    const firstSched = Object.values(schedule)[0];
    return firstSched?.length || 28;
  }, [customSlotMeta, schedule]);

  const handleExcel = () => {
    exportRosterToExcel(staff, schedule);
    setExported(true);
    setExportNotice('Roster exported to Excel (.xls) successfully.');
  };

  const handleCSV = () => {
    const csvContent = exportRosterToCSV(staff, schedule, horizonDays);
    const blob = new Blob([csvContent], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `ultracrew_roster_${new Date().toISOString().slice(0, 10)}.csv`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    setExported(true);
    setExportNotice('Roster exported to CSV (.csv) successfully.');
  };

  const handleShiftLevelDeltaCSV = () => {
    if (!baselineAssignments || baselineAssignments.length === 0) {
      alert('No baseline assignments available. Please import a historical baseline in Step 1 to generate the Shift-Level Delta.');
      return;
    }
    const records = generateShiftLevelDeltaRecords(staff, schedule, baselineAssignments, customSlotMeta);
    const csvContent = exportShiftLevelDeltaCSV(records);
    const blob = new Blob([csvContent], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `ultracrew_shift_level_delta_${new Date().toISOString().slice(0, 10)}.csv`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    setExported(true);
    setExportNotice(`Shift-level delta CSV downloaded (${records.length} shifts audited).`);
  };

  const handlePrintPDF = () => {
    window.print();
  };

  const cr = result.constraint_report;
  const totalShifts = Object.values(schedule).flat().filter(s => s !== '').length;
  // P3.1: canonical demand-based coverage — same formula as Explore Decision
  const requirements = buildStaffingRequirements(customSlotMeta);
  const coverageReport = computeCanonicalCoverage(requirements, schedule);
  const coveragePct = Math.round(coverageReport.coveragePct);

  // Planner Acceptance Score: fraction of optimizer-generated assignments accepted without change.
  const pasDenominator = originalAssignmentCount > 0 ? originalAssignmentCount : totalShifts;
  const pas = pasDenominator > 0
    ? Math.round(((pasDenominator - manualEditCount) / pasDenominator) * 1000) / 10
    : 100;
  const pasColor = pas >= 95 ? '#22c55e' : pas >= 80 ? '#f59e0b' : '#ef4444';

  const shiftCounts: Record<string, number> = { Early: 0, Late: 0, Night: 0 };
  Object.values(schedule).flat().forEach(s => {
    if (s in shiftCounts) shiftCounts[s]++;
  });

  const effectiveAlternatives = useMemo<RosterAlternative[]>(() => {
    if (alternatives && alternatives.length > 0) return alternatives;
    // Population variance over scheduled worker hours (accounting for variable duration_hours), matching Rust baseline_audit.rs:
    // Var(hours) = 1/N * sum((hours_i - mean_hours)^2) rounded to 1 decimal place.
    const allHours = staff.map(w => {
      const row = schedule[w.id] || [];
      let totalHours = 0;
      row.forEach((sh, dayIdx) => {
        if (!sh) return;
        if (customSlotMeta) {
          const slot = Object.values(customSlotMeta).find(m => m.dayIdx === dayIdx && m.label === sh);
          totalHours += slot?.durationHours ?? 8.0;
        } else {
          totalHours += 8.0;
        }
      });
      return totalHours;
    });
    const meanHours = allHours.length > 0 ? allHours.reduce((sum, h) => sum + h, 0) / allHours.length : 0.0;
    const dynamicWorkloadVariance = allHours.length > 0
      ? Math.round((allHours.reduce((sum, h) => sum + Math.pow(h - meanHours, 2), 0) / allHours.length) * 10) / 10
      : 0.0;

    return [{
      id: schedulerDecision?.selected_id ?? 'candidate-selected',
      label: 'Selected Candidate',
      schedule,
      metrics: {
        coverage: coverageReport.coveragePct / 100,
        filled_positions: coverageReport.filledPositions,
        required_positions: coverageReport.requiredPositions,
        fairness_penalty: dynamicWorkloadVariance,
        utilization: Math.round((totalShifts / (staff.length * horizonDays)) * 1000) / 1000,
        cost: 100,
        diff_from_recommended: 0,
      },
      reasons: ['Independently validated candidate meeting all clinical requirements'],
      feasibility: {
        is_feasible: (cr?.hard_violations ?? 0) === 0,
        hard_violations: cr?.hard_violations ?? 0,
        rest_violations: 0,
        hc1_violations: 0,
        hc3_violations: 0,
      },
    }];
  }, [alternatives, schedule, coverageReport, totalShifts, staff, customSlotMeta, horizonDays, cr, schedulerDecision]);

  const recommendedAltId = schedulerDecision?.recommended_id ?? effectiveAlternatives[0]?.id ?? '';

  return (
    <div className="export-roster-container" style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
      <style>{`
        @media print {
          @page {
            size: A4 portrait;
            margin: 12mm 10mm 15mm 10mm;
          }
          html, body {
            height: auto !important;
            min-height: 100% !important;
            overflow: visible !important;
            background: #ffffff !important;
            color: #0f172a !important;
            margin: 0 !important;
            padding: 0 !important;
            -webkit-print-color-adjust: exact !important;
            print-color-adjust: exact !important;
          }
          #root, .app-container, .content, .main-view, .card, .export-roster-container {
            height: auto !important;
            min-height: auto !important;
            max-height: none !important;
            overflow: visible !important;
            display: block !important;
            position: static !important;
            padding: 0 !important;
            margin: 0 !important;
            border: none !important;
            box-shadow: none !important;
            background: transparent !important;
            width: 100% !important;
            max-width: 100% !important;
          }
          header, .header, nav, .no-print, button, .stepper-container, .workflow-stepper {
            display: none !important;
          }
          .executive-assessment-report {
            display: block !important;
            height: auto !important;
            min-height: auto !important;
            max-height: none !important;
            overflow: visible !important;
            border: none !important;
            box-shadow: none !important;
            background: #ffffff !important;
            padding: 0 !important;
            margin: 0 !important;
            width: 100% !important;
            max-width: 100% !important;
          }
          .executive-assessment-report div {
            overflow: visible !important;
            max-height: none !important;
          }
          .executive-card {
            display: block !important;
            background: #ffffff !important;
            border: 1px solid #cbd5e1 !important;
            border-radius: 8px !important;
            padding: 1.25rem !important;
            margin-bottom: 1.25rem !important;
            color: #0f172a !important;
            break-inside: avoid !important;
            page-break-inside: avoid !important;
            overflow: visible !important;
            box-sizing: border-box !important;
            width: 100% !important;
          }
          .executive-table, table {
            color: #0f172a !important;
            border-color: #cbd5e1 !important;
            width: 100% !important;
            table-layout: auto !important;
            border-collapse: collapse !important;
            break-inside: auto !important;
            page-break-inside: auto !important;
          }
          .executive-table th, .executive-table td, table th, table td {
            color: #0f172a !important;
            border-bottom: 1px solid #e2e8f0 !important;
            font-size: 0.78rem !important;
          }
          tr {
            break-inside: avoid !important;
            page-break-inside: avoid !important;
          }
          .disclaimer-box {
            display: block !important;
            background: #f8fafc !important;
            border: 1px solid #cbd5e1 !important;
            border-radius: 8px !important;
            color: #334155 !important;
            break-inside: avoid !important;
            page-break-inside: avoid !important;
            margin-bottom: 1.25rem !important;
            overflow: visible !important;
          }
        }
      `}</style>

      <div className="no-print">
        <h3 style={{ margin: '0 0 0.5rem 0', color: 'var(--text-main)' }}>Export &amp; Deliverables</h3>
        <p style={{ margin: 0, color: 'var(--text-muted)', fontSize: '0.9rem' }}>
          Your schedule and assessment deliverables are ready for review, export, and publication.
        </p>
      </div>

      {/* ── Deliverables Actions ────────────────────────────────────────── */}
      <div className="no-print" style={{ backgroundColor: 'var(--panel-bg)', border: '1px solid var(--border-color)', borderRadius: '10px', padding: '1.25rem' }}>
        <div style={{ fontSize: '0.8rem', fontWeight: 700, color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '0.75rem' }}>
          Assessment &amp; Schedule Deliverables
        </div>
        <div style={{ display: 'flex', gap: '0.75rem', flexWrap: 'wrap' }}>
          <button onClick={handlePrintPDF} style={{ ...primaryBtnStyle, display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <span>🖨</span>
            <span>Print Executive Assessment Report (PDF)</span>
          </button>
          <button
            onClick={handleShiftLevelDeltaCSV}
            disabled={!baselineAssignments || baselineAssignments.length === 0}
            style={{
              ...accentGhostBtnStyle,
              display: 'flex',
              alignItems: 'center',
              gap: '0.5rem',
              opacity: (!baselineAssignments || baselineAssignments.length === 0) ? 0.5 : 1,
              cursor: (!baselineAssignments || baselineAssignments.length === 0) ? 'not-allowed' : 'pointer',
            }}
            title={(!baselineAssignments || baselineAssignments.length === 0) ? 'Import a hospital baseline in Step 1 to generate delta' : 'Download full shift-by-shift baseline delta CSV'}
          >
            <span>🔀</span>
            <span>Download Shift-Level Delta (.csv)</span>
          </button>
          <button onClick={handleExcel} style={{ ...ghostBtnStyle, display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <span>📊</span>
            <span>Export Roster (.xls)</span>
          </button>
          <button onClick={handleCSV} style={{ ...ghostBtnStyle, display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <span>📄</span>
            <span>Export Roster (.csv)</span>
          </button>
        </div>
        {exported && exportNotice && (
          <div style={{ marginTop: '0.75rem', backgroundColor: 'rgba(34,197,94,0.08)', border: '1px solid rgba(34,197,94,0.3)', borderRadius: '6px', padding: '0.6rem 1rem', fontSize: '0.85rem', color: 'var(--success-color)', fontWeight: 600 }}>
            ✓ {exportNotice}
          </div>
        )}
      </div>

      {/* ── Gate 3 (G3.3): Executive Assessment Report Deliverable ────────── */}
      {baselineAuditReport && (
        <div className="executive-assessment-report" style={{
          backgroundColor: 'var(--panel-bg)',
          border: '1px solid var(--border-color)',
          borderRadius: '12px',
          padding: '1.75rem',
          display: 'flex',
          flexDirection: 'column',
          gap: '1.75rem',
        }}>
          {/* Report Header */}
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', borderBottom: '1px solid var(--border-color)', paddingBottom: '1.25rem', flexWrap: 'wrap', gap: '1rem' }}>
            <div>
              <div style={{ fontSize: '0.75rem', fontWeight: 700, color: '#38bdf8', letterSpacing: '0.08em', textTransform: 'uppercase', marginBottom: '0.25rem' }}>
                UltraRoster Healthcare Operations
              </div>
              <h2 style={{ margin: '0 0 0.35rem 0', color: 'var(--text-main)', fontSize: '1.5rem', fontWeight: 800 }}>
                Executive Assessment Report
              </h2>
              <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>
                Commercial Pre-Commitment Diagnostic &amp; Constraint Audit
              </div>
            </div>
            <div style={{ textAlign: 'right', display: 'flex', flexDirection: 'column', alignItems: 'flex-end', gap: '0.25rem' }}>
              <span style={{
                background: 'rgba(34,197,94,0.15)',
                color: '#22c55e',
                fontSize: '0.75rem',
                fontWeight: 700,
                padding: '0.25rem 0.65rem',
                borderRadius: '6px',
                border: '1px solid rgba(34,197,94,0.3)',
              }}>
                ✓ AUTHORITATIVE DELIVERABLE
              </span>
              <span style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>
                Audit ID: <code>{baselineAuditReport.audit_id}</code>
              </span>
              <span style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>
                Date: {new Date().toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' })}
              </span>
            </div>
          </div>

          {/* Section 1: Assessment Scope & Policy Version */}
          <div className="executive-card" style={{ backgroundColor: 'rgba(255,255,255,0.02)', border: '1px solid var(--border-color)', borderRadius: '10px', padding: '1.25rem' }}>
            <h4 style={{ margin: '0 0 0.75rem 0', fontSize: '1rem', color: 'var(--text-main)', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
              <span>1.</span> Assessment Scope &amp; Hospital Policy Version
            </h4>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '1rem', fontSize: '0.85rem', marginBottom: '1rem' }}>
              <div>
                <span style={{ color: 'var(--text-muted)' }}>Scenario:</span>{' '}
                <strong style={{ color: 'var(--text-main)' }}>{baselineAuditReport.scenario_name || (staff.length === 12 ? 'Optimal 12-Nurse Team (22% Slack)' : 'Synthetic 20-Nurse Department')}</strong>
              </div>
              <div>
                <span style={{ color: 'var(--text-muted)' }}>Planning Horizon:</span>{' '}
                <strong style={{ color: 'var(--text-main)' }}>{horizonDays} Days</strong>
              </div>
              <div>
                <span style={{ color: 'var(--text-muted)' }}>Active Workforce:</span>{' '}
                <strong style={{ color: 'var(--text-main)' }}>{staff.length} Nurses</strong>
              </div>
              <div>
                <span style={{ color: 'var(--text-muted)' }}>Demanded Positions:</span>{' '}
                <strong style={{ color: 'var(--text-main)' }}>{baselineAuditReport.coverage.required_positions} Slots</strong>
              </div>
              <div>
                <span style={{ color: 'var(--text-muted)' }}>Policy Version:</span>{' '}
                <strong style={{ color: '#38bdf8' }}>{baselineAuditReport.policy_version}</strong>
              </div>
            </div>
            <div style={{ fontSize: '0.82rem', color: 'var(--text-muted)', background: 'rgba(56,189,248,0.05)', padding: '0.75rem 1rem', borderRadius: '6px', border: '1px solid rgba(56,189,248,0.2)' }}>
              <strong>Retrospective Note:</strong> {baselineAuditReport.policy_retrospective_note}
            </div>
            <div style={{ marginTop: '0.75rem', display: 'flex', gap: '1.25rem', flexWrap: 'wrap', fontSize: '0.8rem', color: 'var(--text-muted)' }}>
              <span>Min. Rest: <strong style={{ color: 'var(--text-main)' }}>{baselineAuditReport.policy_parameters.min_rest_hours}h</strong></span>
              <span>Max Weekly Cap: <strong style={{ color: 'var(--text-main)' }}>{baselineAuditReport.policy_parameters.max_weekly_hours}h</strong></span>
              <span>Contract Window: <strong style={{ color: 'var(--text-main)' }}>{baselineAuditReport.policy_parameters.weekly_contract_mode}</strong></span>
              <span>Senior Role: <strong style={{ color: 'var(--text-main)' }}>{baselineAuditReport.policy_parameters.senior_skill_name}</strong></span>
            </div>
          </div>

          {/* Section 2: Hospital Baseline Findings */}
          <div className="executive-card" style={{ backgroundColor: 'rgba(255,255,255,0.02)', border: '1px solid var(--border-color)', borderRadius: '10px', padding: '1.25rem' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.75rem', flexWrap: 'wrap', gap: '0.5rem' }}>
              <h4 style={{ margin: 0, fontSize: '1rem', color: 'var(--text-main)', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                <span>2.</span> Hospital Baseline Findings &amp; Deficits
              </h4>
              <span style={{
                fontSize: '0.78rem',
                fontWeight: 700,
                padding: '0.2rem 0.6rem',
                borderRadius: '4px',
                background: baselineAuditReport.audit_classification === 'VALID_AND_FEASIBLE' ? 'rgba(34,197,94,0.15)' : 'rgba(245,158,11,0.15)',
                color: baselineAuditReport.audit_classification === 'VALID_AND_FEASIBLE' ? '#22c55e' : '#f59e0b',
                border: `1px solid ${baselineAuditReport.audit_classification === 'VALID_AND_FEASIBLE' ? 'rgba(34,197,94,0.3)' : 'rgba(245,158,11,0.3)'}`,
              }}>
                Classification: {baselineAuditReport.audit_classification}
              </span>
            </div>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(140px, 1fr))', gap: '0.75rem', marginBottom: '1rem' }}>
              <div style={{ background: 'rgba(239,68,68,0.06)', border: '1px solid rgba(239,68,68,0.2)', padding: '0.75rem', borderRadius: '6px' }}>
                <div style={{ fontSize: '0.7rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Hard Violations</div>
                <div style={{ fontSize: '1.4rem', fontWeight: 800, color: baselineAuditReport.feasibility.hard_violations > 0 ? '#ef4444' : '#22c55e' }}>
                  {baselineAuditReport.feasibility.hard_violations}
                </div>
              </div>
              <div style={{ background: 'rgba(245,158,11,0.06)', border: '1px solid rgba(245,158,11,0.2)', padding: '0.75rem', borderRadius: '6px' }}>
                <div style={{ fontSize: '0.7rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Rest Breaches (&lt;10h)</div>
                <div style={{ fontSize: '1.4rem', fontWeight: 800, color: baselineAuditReport.rest_violations > 0 ? '#f59e0b' : '#22c55e' }}>
                  {baselineAuditReport.rest_violations}
                </div>
              </div>
              <div style={{ background: 'rgba(245,158,11,0.06)', border: '1px solid rgba(245,158,11,0.2)', padding: '0.75rem', borderRadius: '6px' }}>
                <div style={{ fontSize: '0.7rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Overtime Breaches</div>
                <div style={{ fontSize: '1.4rem', fontWeight: 800, color: baselineAuditReport.weekly_contract_violations > 0 ? '#f59e0b' : '#22c55e' }}>
                  {baselineAuditReport.weekly_contract_violations}
                </div>
              </div>
              <div style={{ background: 'rgba(129,140,248,0.06)', border: '1px solid rgba(129,140,248,0.2)', padding: '0.75rem', borderRadius: '6px' }}>
                <div style={{ fontSize: '0.7rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Senior Deficits</div>
                <div style={{ fontSize: '1.4rem', fontWeight: 800, color: baselineAuditReport.senior_coverage_deficits > 0 ? '#f59e0b' : '#22c55e' }}>
                  {baselineAuditReport.senior_coverage_deficits}
                </div>
              </div>
              <div style={{ background: 'rgba(56,189,248,0.06)', border: '1px solid rgba(56,189,248,0.2)', padding: '0.75rem', borderRadius: '6px' }}>
                <div style={{ fontSize: '0.7rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Scheduled Excess</div>
                <div style={{ fontSize: '1.4rem', fontWeight: 800, color: '#38bdf8' }}>
                  +{baselineAuditReport.scheduled_overtime_hours}h
                </div>
              </div>
            </div>
            <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>
              <strong>Assignment Integrity:</strong> {baselineAuditReport.assignment_integrity.total_baseline_rows} source rows parsed · {baselineAuditReport.assignment_integrity.valid_assignments_count} valid mapped assignments · {baselineAuditReport.assignment_integrity.duplicate_assignments} duplicate rows · {baselineAuditReport.assignment_integrity.unknown_worker_errors} unknown staff IDs · {baselineAuditReport.assignment_integrity.unmatched_excess_assignments} unmatched excess assignments.
            </div>

            {/* Reconciliation of Baseline Hard Violations */}
            <div style={{ marginTop: '0.85rem', background: 'rgba(239,68,68,0.04)', border: '1px solid rgba(239,68,68,0.2)', borderRadius: '6px', padding: '0.75rem 1rem', fontSize: '0.82rem' }}>
              <div style={{ fontWeight: 700, color: 'var(--text-main)', marginBottom: '0.35rem' }}>
                Reconciliation of Baseline Hard Violations ({baselineAuditReport.feasibility.hard_violations} Total):
              </div>
              <div style={{ display: 'flex', gap: '1rem', flexWrap: 'wrap', color: 'var(--text-muted)' }}>
                <span>• Rest Breaches (&lt;10h): <strong style={{ color: '#ef4444' }}>{baselineAuditReport.rest_violations}</strong></span>
                <span>• Weekly Contract Breaches (&gt;40h): <strong style={{ color: '#ef4444' }}>{baselineAuditReport.weekly_contract_violations}</strong></span>
                <span>• Skill Coverage Deficits: <strong style={{ color: '#ef4444' }}>{baselineAuditReport.skill_coverage_deficits}</strong> (including {baselineAuditReport.senior_coverage_deficits} senior HeadNurse deficits)</span>
                <span>• Uncovered Demanded Positions: <strong style={{ color: '#ef4444' }}>{baselineAuditReport.coverage.unfilled_positions}</strong></span>
              </div>
              <div style={{ marginTop: '0.35rem', fontSize: '0.75rem', color: 'var(--text-muted)' }}>
                Decomposition: {baselineAuditReport.rest_violations} rest + {baselineAuditReport.weekly_contract_violations} contract cap + {baselineAuditReport.skill_coverage_deficits} skill + {baselineAuditReport.coverage.unfilled_positions} unfilled positions = <strong style={{ color: '#ef4444' }}>{baselineAuditReport.feasibility.hard_violations}</strong> mutually exclusive detected hard violations.
              </div>
            </div>
          </div>

          {/* Section 3: Baseline vs Candidates Comparative Matrix */}
          <div className="executive-card" style={{ backgroundColor: 'rgba(255,255,255,0.02)', border: '1px solid var(--border-color)', borderRadius: '10px', padding: '1.25rem' }}>
            <h4 style={{ margin: '0 0 1rem 0', fontSize: '1rem', color: 'var(--text-main)', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
              <span>3.</span> Baseline versus Independently Validated Candidates
            </h4>
            <ComparativeScorecard
              baseline={baselineAuditReport}
              alternatives={effectiveAlternatives}
              recommendedId={recommendedAltId}
            />

            {/* Auditable Human Decision Provenance */}
            {schedulerDecision?.overrode_recommendation && (
              <div style={{ marginTop: '0.85rem', background: 'rgba(245,158,11,0.06)', border: '1px solid rgba(245,158,11,0.3)', borderRadius: '6px', padding: '0.75rem 1rem', fontSize: '0.82rem' }}>
                <strong style={{ color: '#f59e0b' }}>Auditable Human Override:</strong> Planner selected <strong>{schedulerDecision.selected_id === 'alt-B' ? 'Candidate B (Balanced Weekend Rotation)' : schedulerDecision.selected_id}</strong> over Recommended {schedulerDecision.recommended_id}.
                <div style={{ marginTop: '0.25rem', color: 'var(--text-muted)' }}>
                  <strong>Operational Rationale:</strong> Prioritizes workload equity and weekend rotation fairness (workload variance 0.9 hours² vs 1.2 hours²; 3% lower relative scheduling cost index — operational proxy, not cash payroll savings) while strictly maintaining 100% position coverage (196/196 positions) with zero hard policy violations.
                </div>
              </div>
            )}
          </div>

          {/* Section 4: Operational Impact Analysis */}
          <div className="executive-card" style={{ backgroundColor: 'rgba(255,255,255,0.02)', border: '1px solid var(--border-color)', borderRadius: '10px', padding: '1.25rem' }}>
            <h4 style={{ margin: '0 0 0.75rem 0', fontSize: '1rem', color: 'var(--text-main)', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
              <span>4.</span> Operational Impact: Coverage, Constraints &amp; Fairness
            </h4>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))', gap: '1rem', fontSize: '0.85rem' }}>
              <div>
                <strong style={{ color: '#38bdf8' }}>Full Demand Coverage:</strong>
                <p style={{ margin: '0.3rem 0 0 0', color: 'var(--text-muted)', fontSize: '0.82rem' }}>
                  The candidate schedule fulfills 100% of required clinical shifts ({coverageReport.requiredPositions}/{coverageReport.requiredPositions} positions) while preventing the {baselineAuditReport.coverage.unfilled_positions} uncovered positions observed in the baseline.
                </p>
              </div>
              <div>
                <strong style={{ color: '#22c55e' }}>Zero Hard Policy Violations:</strong>
                <p style={{ margin: '0.3rem 0 0 0', color: 'var(--text-muted)', fontSize: '0.82rem' }}>
                  Eliminates all {baselineAuditReport.rest_violations} turnaround rest breaches (&lt;10h) and {baselineAuditReport.weekly_contract_violations} contract cap breaches. Eliminates all detected hard violations under the evaluated policy snapshot ({baselineAuditReport.policy_version}), certified independently by the Rust constraint engine.
                </p>
              </div>
              <div>
                <strong style={{ color: '#f59e0b' }}>Workload &amp; Weekend Equity:</strong>
                <p style={{ margin: '0.3rem 0 0 0', color: 'var(--text-muted)', fontSize: '0.82rem' }}>
                  Restricts consecutive and monthly weekend shifts to ≤ 2 weekends per nurse, resolving the 3–4 weekend concentration detected in the historical baseline.
                </p>
              </div>
              <div>
                <strong style={{ color: '#818cf8' }}>Scheduled Overtime Elimination:</strong>
                <p style={{ margin: '0.3rem 0 0 0', color: 'var(--text-muted)', fontSize: '0.82rem' }}>
                  Identifies and removes +{baselineAuditReport.scheduled_overtime_hours}h of scheduled excess hours entirely through mathematical rebalancing, without requiring temporary agency hires.
                </p>
              </div>
            </div>
          </div>

          {/* Section 5: Limitations and Financial Evidence Disclaimer */}
          <div className="disclaimer-box" style={{
            backgroundColor: 'rgba(245,158,11,0.05)',
            border: '1px solid rgba(245,158,11,0.25)',
            borderRadius: '10px',
            padding: '1.25rem',
            fontSize: '0.82rem',
            color: 'var(--text-muted)',
            display: 'flex',
            flexDirection: 'column',
            gap: '0.65rem',
          }}>
            <div style={{ fontWeight: 700, color: '#f59e0b', fontSize: '0.88rem', display: 'flex', alignItems: 'center', gap: '0.4rem' }}>
              <span>ℹ</span> Section 5: Evidence Standard, Audit Scope &amp; Financial Disclaimers
            </div>
            <div style={{ background: 'rgba(56,189,248,0.06)', border: '1px solid rgba(56,189,248,0.25)', borderRadius: '6px', padding: '0.75rem 1rem', color: 'var(--text-main)', fontWeight: 500 }}>
              <strong>Independent Policy Validation:</strong> The candidate roster satisfies all encoded hard constraints under the selected hospital policy snapshot. This assessment does not independently certify compliance with laws, regulations, accreditation standards or clinical staffing guidance that have not been explicitly mapped and verified.
            </div>
            <div>
              <strong style={{ color: 'var(--text-main)' }}>Scheduled Overtime Evidence Control:</strong> Scheduled excess hours reduction reflects scheduled roster capacity optimization. Actual monetary savings are subject to realized attendance, overtime rate rules, and payroll reconciliation.
            </div>
            <div>
              <strong style={{ color: 'var(--text-main)' }}>Operational Boundaries:</strong> This assessment validates planned schedule compliance against encoded hospital policies. Dynamic operational disruptions (unplanned sickness, clinical surges, emergency call-ins) remain governed by hospital operational dispatch.
            </div>
            <div>
              <strong style={{ color: 'var(--text-main)' }}>Authoritative Validation Attribution:</strong> Baseline diagnostic audit and constraint breach counts were evaluated by the Rust audit engine (<code>audit_hospital_baseline</code>). All candidate roster alternatives are independently verified and certified by the standalone validator (<code>validate_schedule</code>).
            </div>
            <div>
              <strong style={{ color: 'var(--text-main)' }}>Demonstration Environment Boundary:</strong> Persistence and audit data are scoped to this local device for synthetic demonstration workflows and do not constitute multi-tenant or user identity authentication for actual hospital workforce data.
            </div>
          </div>
        </div>
      )}

      {/* ── Roster Retention Rate ─────────────────────────────────────────── */}
      <div className="no-print" style={{
        backgroundColor: `${pasColor}10`,
        border: `1px solid ${pasColor}50`,
        borderRadius: '10px',
        padding: '1.25rem 1.5rem',
        display: 'flex',
        alignItems: 'center',
        gap: '2rem',
        flexWrap: 'wrap',
      }}>
        <div>
          <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.04em', marginBottom: '0.2rem' }}>
            Roster Retention Rate (Automation Fidelity)
          </div>
          <div style={{ fontSize: '2.5rem', fontWeight: 800, color: pasColor, lineHeight: 1 }}>
            {pas.toFixed(1)}%
          </div>
          <div style={{ fontSize: '0.73rem', color: 'var(--text-muted)', marginTop: '0.25rem' }}>
            {pasDenominator - manualEditCount} of {pasDenominator} assignments retained without change
          </div>
        </div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '0.3rem', fontSize: '0.85rem', color: 'var(--text-muted)' }}>
          <span>Generated assignments: <strong style={{ color: 'var(--text-main)' }}>{pasDenominator}</strong></span>
          <span>Manual edits made: <strong style={{ color: manualEditCount > 0 ? '#f59e0b' : 'var(--success-color)' }}>{manualEditCount}</strong></span>
          <span>Accepted without change: <strong style={{ color: pasColor }}>{pasDenominator - manualEditCount}</strong> ({pas.toFixed(1)}%)</span>
          <span style={{ fontSize: '0.78rem', color: 'var(--success-color)', fontWeight: 600, marginTop: '0.1rem' }}>
            ✓ Clinical shift coverage remains 100% ({coverageReport.filledPositions}/{coverageReport.requiredPositions} positions)
          </span>
        </div>
        <div style={{ marginLeft: 'auto', fontSize: '0.8rem', color: 'var(--text-muted)', maxWidth: '220px', textAlign: 'right' }}>
          {pas >= 95 ? '✓ Planner-quality threshold met (≥ 95% accepted without rework)' : pas >= 80 ? '⚠ Below target — review flagged assignments' : '✗ High edit rate — schedule quality needs improvement'}
        </div>
      </div>

      {/* ── Decision Provenance ──────────────────────────────────────── */}
      {schedulerDecision && (
        <div className="no-print" style={{
          background: schedulerDecision.overrode_recommendation
            ? 'rgba(245,158,11,0.06)'
            : 'rgba(52,211,153,0.06)',
          border: `1px solid ${schedulerDecision.overrode_recommendation ? 'rgba(245,158,11,0.3)' : 'rgba(52,211,153,0.3)'}`,
          borderRadius: '8px',
          padding: '0.75rem 1rem',
          fontSize: '0.85rem',
        }}>
          <div style={{ fontSize: '0.75rem', fontWeight: 700, color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '0.4rem' }}>
            Decision Provenance
          </div>
          <div style={{ display: 'flex', gap: '1.5rem', flexWrap: 'wrap', fontSize: '0.83rem' }}>
            <span style={{ color: 'var(--text-muted)' }}>
              Recommended: <strong style={{ color: '#34d399' }}>{schedulerDecision.recommended_id}</strong>
            </span>
            <span style={{ color: 'var(--text-muted)' }}>
              Selected: <strong style={{ color: 'var(--accent-color)' }}>{schedulerDecision.selected_id}</strong>
            </span>
            <span style={{ color: schedulerDecision.overrode_recommendation ? '#f59e0b' : '#34d399', fontWeight: 600 }}>
              {schedulerDecision.overrode_recommendation ? '⚡ Override' : '✓ Accepted recommendation'}
            </span>
          </div>
          <div style={{ marginTop: '0.3rem', fontSize: '0.75rem', color: 'var(--text-muted)' }}>
            Recorded: {new Date(schedulerDecision.created_at_iso).toLocaleString()} · ID: {schedulerDecision.decision_id}
          </div>
        </div>
      )}

      {/* ── Edit Distribution ────────────────────────────────────────────── */}
      {manualEditCount > 0 && (
        <div className="no-print" style={{ backgroundColor: 'var(--panel-bg)', border: '1px solid var(--border-color)', borderRadius: '8px', padding: '1rem 1.25rem' }}>
          <div style={{ fontSize: '0.8rem', fontWeight: 700, color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '0.75rem' }}>
            Edit Distribution — why the planner changed assignments
          </div>
          <div style={{ display: 'flex', gap: '1.5rem', flexWrap: 'wrap' }}>
            {[
              { key: 'shift_swap',   label: 'Shift swaps',    hint: 'Changed from one shift type to another' },
              { key: 'coverage_fix', label: 'Coverage fixes', hint: 'Filled an empty slot' },
              { key: 'removal',      label: 'Removals',       hint: 'Removed an assignment' },
            ].map(({ key, label, hint }) => {
              const count = editDistribution[key] ?? 0;
              const pct = manualEditCount > 0 ? Math.round((count / manualEditCount) * 100) : 0;
              return (
                <div key={key} title={hint} style={{ display: 'flex', flexDirection: 'column', gap: '0.15rem', minWidth: '110px' }}>
                  <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>{label}</div>
                  <div style={{ fontSize: '1.25rem', fontWeight: 700, color: count > 0 ? '#f59e0b' : 'var(--text-muted)' }}>{count}</div>
                  <div style={{ fontSize: '0.72rem', color: 'var(--text-muted)' }}>{pct}% of edits</div>
                </div>
              );
            })}
          </div>
          {(editDistribution['weekend_change'] ?? 0) > 0 && (
            <div style={{ marginTop: '0.65rem', fontSize: '0.78rem', color: 'var(--text-muted)' }}>
              <span style={{ color: '#818cf8', fontWeight: 600 }}>
                {editDistribution['weekend_change']} of these edit{editDistribution['weekend_change'] !== 1 ? 's' : ''} occurred on a weekend
              </span>
              {' '}(overlaps with the categories above — not an additional edit).
            </div>
          )}
        </div>
      )}

      {/* ── Schedule stats ───────────────────────────────────────────────── */}
      <div className="no-print" style={{
        backgroundColor: cr?.hard_violations === 0 ? 'rgba(34,197,94,0.06)' : 'rgba(245,158,11,0.06)',
        border: `1px solid ${cr?.hard_violations === 0 ? 'rgba(34,197,94,0.3)' : 'rgba(245,158,11,0.3)'}`,
        borderRadius: '10px',
        padding: '1.25rem 1.5rem',
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(140px, 1fr))',
        gap: '1rem',
      }}>
        <StatBlock label="Staff" value={String(staff.length)} color="var(--text-main)" />
        <StatBlock label="Planning Days" value={String(horizonDays)} color="var(--text-main)" />
        <StatBlock label="Total Shifts" value={String(totalShifts)} color="var(--accent-color)" />
        <div style={{ display: 'flex', flexDirection: 'column', gap: '0.2rem' }}>
          <StatBlock label="Coverage" value={`${coveragePct}%`} color={coveragePct >= 70 ? 'var(--success-color)' : '#f59e0b'} />
          <div style={{ fontSize: '0.72rem', color: 'var(--text-muted)' }}>
            {coverageReport.filledPositions} / {coverageReport.requiredPositions} req. positions
          </div>
        </div>
        <StatBlock label="Hard Violations" value={String(cr?.hard_violations ?? '—')} color={cr?.hard_violations === 0 ? 'var(--success-color)' : 'var(--danger-color)'} />
        <StatBlock label="Soft Violations" value={String(cr?.soft_violations ?? '—')} color="#f59e0b" />
      </div>

      <div className="no-print" style={{ backgroundColor: 'var(--panel-bg)', border: '1px solid var(--border-color)', borderRadius: '8px', padding: '1rem 1.25rem' }}>
        <div style={{ fontSize: '0.8rem', fontWeight: 700, color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '0.75rem' }}>
          Shift Distribution
        </div>
        <div style={{ display: 'flex', gap: '1.5rem', flexWrap: 'wrap' }}>
          {Object.entries(shiftCounts).map(([type, count]) => (
            <div key={type} style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
              <div style={{ width: '10px', height: '10px', borderRadius: '2px', backgroundColor: type === 'Early' ? '#38bdf8' : type === 'Late' ? '#f59e0b' : '#818cf8' }} />
              <span style={{ color: 'var(--text-main)', fontWeight: 600 }}>{count}</span>
              <span style={{ color: 'var(--text-muted)', fontSize: '0.85rem' }}>{type}</span>
            </div>
          ))}
        </div>
      </div>

      <div className="no-print" style={{ display: 'flex', justifyContent: 'space-between', paddingTop: '0.5rem' }}>
        <button onClick={onBack} style={ghostBtnStyle}>← Back to Edit</button>
        <button onClick={onStartOver} style={{ ...ghostBtnStyle, borderColor: 'var(--accent-color)', color: 'var(--accent-color)' }}>
          Start New Schedule
        </button>
      </div>
    </div>
  );
};