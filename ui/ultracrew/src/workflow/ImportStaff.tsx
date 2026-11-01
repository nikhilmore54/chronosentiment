import React, { useRef, useState, useMemo } from 'react';
import type {
  StaffMember,
  BaselineAssignment,
  BaselineAuditReport,
  ApprovedDemandSnapshot,
  HospitalWardConfiguration,
  ShiftModifierCensusInput,
} from './WorkflowTypes';
import { SAMPLE_CSV, OPTIMAL_12_STAFF_CSV } from './WorkflowTypes';
import {
  parseStaffCSV,
  parseShiftsCSV,
  parseBaselineCSV,
  generateSampleBaselineWithBreaches,
  runBaselineAudit,
  getScheduleSlotMeta,
  buildImportSummary,
  calculateClinicalDemand,
  createApprovedDemandSnapshot,
  computeDemandCapacityDiagnostic,
  DEFAULT_HOSPITAL_CONFIG,
  type CustomShiftInput,
  type ShiftSlotMeta,
} from './WorkflowUtils';
import {
  primaryBtnStyle, ghostBtnStyle, accentGhostBtnStyle,
  codeBoxStyle, textareaStyle, errorBoxStyle,
  SummaryRow, SkillBadge,
} from './WorkflowComponents';

function generateVariableWeekendFixture(): ShiftSlotMeta[] {
  const shifts: ShiftSlotMeta[] = [];
  let slotId = 1;
  for (let day = 0; day < 28; day++) {
    const isWeekend = (day % 7 === 5) || (day % 7 === 6); // Sat (5), Sun (6)
    if (isWeekend) {
      // Reduced weekend demand: 4 positions (2 Early, 1 Late, 1 Night)
      shifts.push({ shiftId: slotId++, dayIdx: day, label: 'Early', skill: 'Nurse', startHour: day * 24 + 7, durationHours: 8 });
      shifts.push({ shiftId: slotId++, dayIdx: day, label: 'Early', skill: 'HeadNurse', startHour: day * 24 + 7, durationHours: 8 });
      shifts.push({ shiftId: slotId++, dayIdx: day, label: 'Late', skill: 'Nurse', startHour: day * 24 + 14, durationHours: 8 });
      shifts.push({ shiftId: slotId++, dayIdx: day, label: 'Night', skill: 'Nurse', startHour: day * 24 + 22, durationHours: 8 });
    } else {
      // Standard weekday demand: 7 positions (3 Early, 2 Late, 2 Night)
      shifts.push({ shiftId: slotId++, dayIdx: day, label: 'Early', skill: 'Nurse', startHour: day * 24 + 7, durationHours: 8 });
      shifts.push({ shiftId: slotId++, dayIdx: day, label: 'Early', skill: 'Nurse', startHour: day * 24 + 7, durationHours: 8 });
      shifts.push({ shiftId: slotId++, dayIdx: day, label: 'Early', skill: 'HeadNurse', startHour: day * 24 + 7, durationHours: 8 });
      shifts.push({ shiftId: slotId++, dayIdx: day, label: 'Late', skill: 'Nurse', startHour: day * 24 + 14, durationHours: 8 });
      shifts.push({ shiftId: slotId++, dayIdx: day, label: 'Late', skill: 'Nurse', startHour: day * 24 + 14, durationHours: 8 });
      shifts.push({ shiftId: slotId++, dayIdx: day, label: 'Night', skill: 'Nurse', startHour: day * 24 + 22, durationHours: 8 });
      shifts.push({ shiftId: slotId++, dayIdx: day, label: 'Night', skill: 'Nurse', startHour: day * 24 + 22, durationHours: 8 });
    }
  }
  return shifts;
}

export const ImportStaff: React.FC<{
  staff: StaffMember[];
  onStaffChange: (s: StaffMember[]) => void;
  customShifts?: CustomShiftInput[];
  onCustomShiftsChange?: (shifts: CustomShiftInput[]) => void;
  baselineAuditReport?: BaselineAuditReport | null;
  onBaselineAuditReportChange?: (report: BaselineAuditReport | null) => void;
  baselineAssignments?: BaselineAssignment[];
  onBaselineAssignmentsChange?: (assignments: BaselineAssignment[]) => void;
  approvedDemandSnapshot?: ApprovedDemandSnapshot | null;
  onApprovedDemandSnapshotChange?: (snapshot: ApprovedDemandSnapshot | null) => void;
  onNext: () => void;
}> = ({
  staff,
  onStaffChange,
  customShifts,
  onCustomShiftsChange,
  baselineAuditReport: externalAuditReport,
  onBaselineAuditReportChange,
  baselineAssignments: externalBaselineAssignments,
  onBaselineAssignmentsChange,
  approvedDemandSnapshot,
  onApprovedDemandSnapshotChange,
  onNext,
}) => {
  const fileRef = useRef<HTMLInputElement>(null);
  const shiftFileRef = useRef<HTMLInputElement>(null);
  const baselineFileRef = useRef<HTMLInputElement>(null);
  const [errors, setErrors] = useState<string[]>([]);
  const [shiftErrors, setShiftErrors] = useState<string[]>([]);
  const [baselineErrors, setBaselineErrors] = useState<string[]>([]);
  const [internalBaselineAssignments, setInternalBaselineAssignments] = useState<BaselineAssignment[]>([]);
  const baselineAssignments = externalBaselineAssignments !== undefined ? externalBaselineAssignments : internalBaselineAssignments;

  const setBaselineAssignments = (assignments: BaselineAssignment[]) => {
    setInternalBaselineAssignments(assignments);
    onBaselineAssignmentsChange?.(assignments);
  };

  const [internalAuditReport, setInternalAuditReport] = useState<BaselineAuditReport | null>(null);
  const baselineAuditReport = externalAuditReport !== undefined ? externalAuditReport : internalAuditReport;

  const setAuditReport = (report: BaselineAuditReport | null) => {
    setInternalAuditReport(report);
    onBaselineAuditReportChange?.(report);
  };

  const [isAuditing, setIsAuditing] = useState(false);
  const [showFindings, setShowFindings] = useState(false);
  const [showFormat, setShowFormat] = useState(false);

  // ── Coralys Demand Recommendation & Approval State (Level A) ─────────────
  const [wardConfig] = useState<HospitalWardConfiguration>(DEFAULT_HOSPITAL_CONFIG);
  const [activeAcuteBeds, setActiveAcuteBeds] = useState<number>(20);
  const [activeHduBeds, setActiveHduBeds] = useState<number>(6);
  const [acuteOccupancy, setAcuteOccupancy] = useState<number>(90);
  const [hduOccupancy, setHduOccupancy] = useState<number>(100);
  const [isolationCount, setIsolationCount] = useState<number>(0);
  const [telemetryCount, setTelemetryCount] = useState<number>(0);
  const [approverName, setApproverName] = useState<string>('Dr. Aris Thorne, Director of Nursing');
  const [approvalNotes, setApprovalNotes] = useState<string>('Approved staffing demand baseline for Ward 4B acute rotation');
  const [showDerivationDetails, setShowDerivationDetails] = useState<boolean>(false);
  const [showManualDemand, setShowManualDemand] = useState<boolean>(false);

  const effectiveConfig = useMemo<HospitalWardConfiguration>(() => {
    return {
      ...wardConfig,
      bedCategories: wardConfig.bedCategories.map(c => {
        if (c.id === 'general_acute') {
          return { ...c, activeStaffedBeds: activeAcuteBeds, defaultOccupancyPct: acuteOccupancy };
        }
        if (c.id === 'step_down_hdu') {
          return { ...c, activeStaffedBeds: activeHduBeds, defaultOccupancyPct: hduOccupancy };
        }
        return c;
      }),
    };
  }, [wardConfig, activeAcuteBeds, activeHduBeds, acuteOccupancy, hduOccupancy]);

  const modifierCensusInputs = useMemo<ShiftModifierCensusInput[]>(() => {
    if (isolationCount === 0 && telemetryCount === 0) return [];
    const inputs: ShiftModifierCensusInput[] = [];
    for (let d = 0; d < 28; d++) {
      for (const st of ['Early', 'Late', 'Night']) {
        inputs.push({
          dayIdx: d,
          shiftType: st,
          affectedPatientCounts: {
            infectious_isolation: isolationCount,
            cardiac_telemetry: telemetryCount,
          },
        });
      }
    }
    return inputs;
  }, [isolationCount, telemetryCount]);

  const calculatedDemand = useMemo(() => {
    try {
      return calculateClinicalDemand(effectiveConfig, {
        horizonDays: 28,
        modifierCensusInputs,
      });
    } catch {
      return null;
    }
  }, [effectiveConfig, modifierCensusInputs]);

  const capacityDiagnostic = useMemo(() => {
    if (!calculatedDemand) return null;
    return computeDemandCapacityDiagnostic(staff, calculatedDemand.slots, 28);
  }, [staff, calculatedDemand]);

  const handleApproveDemand = () => {
    if (!calculatedDemand || !approverName.trim()) return;
    const snapshot = createApprovedDemandSnapshot(
      `scenario-ward-4b-${Date.now()}`,
      effectiveConfig,
      approverName.trim(),
      approvalNotes.trim(),
      calculatedDemand,
    );
    onApprovedDemandSnapshotChange?.(snapshot);
    if (onCustomShiftsChange) {
      onCustomShiftsChange(snapshot.slots);
    }
  };

  const processText = (text: string) => {
    const { staff: parsed, errors: errs } = parseStaffCSV(text);
    setErrors(errs);
    if (parsed.length > 0) onStaffChange(parsed);
  };

  const processShiftsText = (text: string) => {
    const { shifts: parsed, errors: errs } = parseShiftsCSV(text);
    setShiftErrors(errs);
    if (parsed.length > 0 && onCustomShiftsChange) {
      onCustomShiftsChange(parsed);
    }
  };

  const processBaselineText = (text: string) => {
    const { assignments: parsed, errors: errs } = parseBaselineCSV(text);
    setBaselineErrors(errs);
    setBaselineAssignments(parsed);
    setAuditReport(null);
  };

  const handleRunAudit = async () => {
    if (baselineAssignments.length === 0 || staff.length === 0) return;
    setIsAuditing(true);
    try {
      const slotMeta = getScheduleSlotMeta(staff, customShifts);
      const shifts = Object.values(slotMeta);
      const report = await runBaselineAudit(staff, shifts, baselineAssignments);
      setAuditReport(report);
      setShowFindings(true);
    } catch (err) {
      setBaselineErrors([err instanceof Error ? err.message : 'Audit failed']);
    } finally {
      setIsAuditing(false);
    }
  };

  const summary = staff.length > 0 ? buildImportSummary(staff) : null;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
      <div>
        <h3 style={{ margin: '0 0 0.5rem 0', color: 'var(--text-main)' }}>Import Staff &amp; Demand</h3>
        <p style={{ margin: 0, color: 'var(--text-muted)', fontSize: '0.9rem' }}>
          Upload your hospital staff roster and optional department shift demand.
        </p>
      </div>

      <div style={{ display: 'flex', gap: '0.75rem', flexWrap: 'wrap', alignItems: 'center' }}>
        <label style={primaryBtnStyle}>
          Upload Staff CSV
          <input ref={fileRef} type="file" accept=".csv,.txt" onChange={e => {
            const file = e.target.files?.[0];
            if (!file) return;
            const reader = new FileReader();
            reader.onload = ev => processText(ev.target?.result as string);
            reader.readAsText(file);
            e.target.value = '';
          }} style={{ display: 'none' }} />
        </label>

        <button onClick={() => setShowFormat(s => !s)} style={ghostBtnStyle}>
          {showFormat ? 'Hide Format' : 'Show CSV Format'}
        </button>

        <button onClick={() => {
          const { staff: parsed } = parseStaffCSV(SAMPLE_CSV);
          setErrors([]);
          onStaffChange(parsed);
        }} style={accentGhostBtnStyle}>
          Load Sample Staff (20 staff)
        </button>

        <button onClick={() => {
          const { staff: parsed } = parseStaffCSV(OPTIMAL_12_STAFF_CSV);
          setErrors([]);
          onStaffChange(parsed);
        }} style={{ ...accentGhostBtnStyle, borderColor: '#10b981', color: '#10b981' }}>
          Load Optimal Staff (12 staff • 22% slack)
        </button>

        <button onClick={() => {
          const template = `id,contract,skills\nAlice_Smith,FullTime,Nurse\nBob_Jones,PartTime,HeadNurse;Nurse\nCarla_Night,Night,Nurse`;
          const blob = new Blob([template], { type: 'text/csv' });
          const url = URL.createObjectURL(blob);
          const a = document.createElement('a');
          a.href = url;
          a.download = 'ultracrew_staff_template.csv';
          document.body.appendChild(a);
          a.click();
          document.body.removeChild(a);
          URL.revokeObjectURL(url);
        }} style={ghostBtnStyle}>
          ⬇ Staff Template
        </button>

        {staff.length > 0 && (
          <span style={{ color: 'var(--success-color)', fontSize: '0.9rem', fontWeight: 600 }}>
            ✓ {staff.length} staff members loaded
          </span>
        )}
      </div>

      {showFormat && (
        <div style={codeBoxStyle}>
          <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)', marginBottom: '0.5rem' }}>
            Staff CSV format — paste below or upload file:
          </div>
          <pre style={{ margin: 0, fontSize: '0.8rem', color: 'var(--accent-color)', fontFamily: 'monospace' }}>
            {SAMPLE_CSV}
          </pre>
          <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)', marginTop: '0.5rem' }}>
            Skills: semicolon-separated (e.g. <code>HeadNurse;Nurse</code>). Contracts: FullTime / PartTime / Night.
          </div>
          <textarea
            placeholder="Paste staff CSV here..."
            onPaste={e => processText(e.clipboardData.getData('text'))}
            style={{ ...textareaStyle, marginTop: '0.75rem' }}
          />
        </div>
      )}

      {errors.length > 0 && (
        <div style={errorBoxStyle}>
          {errors.map((e, i) => <div key={i} style={{ fontSize: '0.85rem' }}>⚠ {e}</div>)}
        </div>
      )}

      {/* ── Coralys Clinical Demand Recommendation & Approval (Level A) ── */}
      <div style={{
        backgroundColor: 'var(--panel-bg)',
        border: approvedDemandSnapshot ? '1px solid #10b981' : '1px solid var(--border-color)',
        borderRadius: '8px',
        padding: '1.25rem',
        display: 'flex',
        flexDirection: 'column',
        gap: '1.25rem',
        boxShadow: approvedDemandSnapshot ? '0 0 16px rgba(16, 185, 129, 0.08)' : 'none',
      }}>
        {/* Card Header & Status */}
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', flexWrap: 'wrap', gap: '0.75rem' }}>
          <div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
              <span style={{ fontSize: '1.1rem' }}>🏥</span>
              <div style={{ fontWeight: 700, color: 'var(--text-main)', fontSize: '1rem' }}>
                Coralys Clinical Demand Recommendation &amp; Approval (Level A)
              </div>
            </div>
            <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)', marginTop: '0.2rem' }}>
              Hospital-governed clinical demand derivation for inpatient wards. Recommends required staffing levels by date, shift, and competency.
            </div>
          </div>
          {approvedDemandSnapshot ? (
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', flexWrap: 'wrap' }}>
              <span style={{
                backgroundColor: 'rgba(16, 185, 129, 0.15)',
                color: '#10b981',
                padding: '0.35rem 0.75rem',
                borderRadius: '6px',
                fontSize: '0.85rem',
                fontWeight: 600,
                display: 'flex',
                alignItems: 'center',
                gap: '0.35rem',
              }}>
                🔒 Approved Demand Snapshot Active ({approvedDemandSnapshot.totalPositions} shifts)
              </span>
              <button
                onClick={() => {
                  onApprovedDemandSnapshotChange?.(null);
                  if (onCustomShiftsChange) onCustomShiftsChange([]);
                }}
                style={{ ...ghostBtnStyle, fontSize: '0.8rem', padding: '0.3rem 0.6rem', color: '#f59e0b' }}
              >
                Revert to Edit / Draft
              </button>
            </div>
          ) : (
            <span style={{
              backgroundColor: 'rgba(245, 158, 11, 0.12)',
              color: '#f59e0b',
              padding: '0.35rem 0.75rem',
              borderRadius: '6px',
              fontSize: '0.85rem',
              fontWeight: 600,
            }}>
              Draft / Unapproved Demand Preview
            </span>
          )}
        </div>

        {/* 1. Hospital-Defined Ward & Bed Configuration */}
        <div style={{
          backgroundColor: 'var(--bg-color)',
          border: '1px solid var(--border-color)',
          borderRadius: '6px',
          padding: '1rem',
          display: 'flex',
          flexDirection: 'column',
          gap: '0.75rem',
        }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', flexWrap: 'wrap', gap: '0.5rem' }}>
            <div style={{ fontWeight: 600, color: 'var(--text-main)', fontSize: '0.88rem' }}>
              1. Hospital Ward &amp; Bed Configuration
            </div>
            <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>
              Standard: <code>{effectiveConfig.governance.facilityStandardReference}</code> (v{effectiveConfig.configVersion})
            </div>
          </div>

          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))', gap: '0.75rem' }}>
            <div style={{ backgroundColor: 'var(--panel-bg)', padding: '0.75rem', borderRadius: '6px', border: '1px solid var(--border-color)' }}>
              <div style={{ fontWeight: 600, fontSize: '0.85rem', color: 'var(--accent-color)' }}>General Acute Care Beds</div>
              <div style={{ fontSize: '0.78rem', color: 'var(--text-muted)', margin: '0.25rem 0 0.5rem 0' }}>24 physical ward beds</div>
              <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
                <label style={{ fontSize: '0.78rem', color: 'var(--text-muted)' }}>Active:</label>
                <input
                  type="number"
                  min="1"
                  max="24"
                  disabled={!!approvedDemandSnapshot}
                  value={activeAcuteBeds}
                  onChange={e => setActiveAcuteBeds(Math.max(1, Math.min(24, Number(e.target.value) || 1)))}
                  style={{ width: '55px', padding: '0.2rem 0.4rem', backgroundColor: 'var(--bg-color)', border: '1px solid var(--border-color)', color: 'var(--text-main)', borderRadius: '4px' }}
                />
                <label style={{ fontSize: '0.78rem', color: 'var(--text-muted)', marginLeft: '0.4rem' }}>Occupancy:</label>
                <input
                  type="number"
                  min="0"
                  max="100"
                  disabled={!!approvedDemandSnapshot}
                  value={acuteOccupancy}
                  onChange={e => setAcuteOccupancy(Math.max(0, Math.min(100, Number(e.target.value) || 0)))}
                  style={{ width: '55px', padding: '0.2rem 0.4rem', backgroundColor: 'var(--bg-color)', border: '1px solid var(--border-color)', color: 'var(--text-main)', borderRadius: '4px' }}
                />
                <span style={{ fontSize: '0.78rem', color: 'var(--text-muted)' }}>%</span>
              </div>
            </div>

            <div style={{ backgroundColor: 'var(--panel-bg)', padding: '0.75rem', borderRadius: '6px', border: '1px solid var(--border-color)' }}>
              <div style={{ fontWeight: 600, fontSize: '0.85rem', color: 'var(--accent-color)' }}>Step-Down HDU Beds</div>
              <div style={{ fontSize: '0.78rem', color: 'var(--text-muted)', margin: '0.25rem 0 0.5rem 0' }}>6 physical HDU beds</div>
              <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
                <label style={{ fontSize: '0.78rem', color: 'var(--text-muted)' }}>Active:</label>
                <input
                  type="number"
                  min="0"
                  max="6"
                  disabled={!!approvedDemandSnapshot}
                  value={activeHduBeds}
                  onChange={e => setActiveHduBeds(Math.max(0, Math.min(6, Number(e.target.value) || 0)))}
                  style={{ width: '55px', padding: '0.2rem 0.4rem', backgroundColor: 'var(--bg-color)', border: '1px solid var(--border-color)', color: 'var(--text-main)', borderRadius: '4px' }}
                />
                <label style={{ fontSize: '0.78rem', color: 'var(--text-muted)', marginLeft: '0.4rem' }}>Occupancy:</label>
                <input
                  type="number"
                  min="0"
                  max="100"
                  disabled={!!approvedDemandSnapshot}
                  value={hduOccupancy}
                  onChange={e => setHduOccupancy(Math.max(0, Math.min(100, Number(e.target.value) || 0)))}
                  style={{ width: '55px', padding: '0.2rem 0.4rem', backgroundColor: 'var(--bg-color)', border: '1px solid var(--border-color)', color: 'var(--text-main)', borderRadius: '4px' }}
                />
                <span style={{ fontSize: '0.78rem', color: 'var(--text-muted)' }}>%</span>
              </div>
            </div>

            <div style={{ backgroundColor: 'var(--panel-bg)', padding: '0.75rem', borderRadius: '6px', border: '1px solid var(--border-color)' }}>
              <div style={{ fontWeight: 600, fontSize: '0.85rem', color: 'var(--accent-color)' }}>Clinical Modifiers (Acuity)</div>
              <div style={{ fontSize: '0.78rem', color: 'var(--text-muted)', margin: '0.25rem 0 0.5rem 0' }}>Additive care increments</div>
              <div style={{ display: 'flex', flexDirection: 'column', gap: '0.35rem' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <span style={{ fontSize: '0.78rem', color: 'var(--text-muted)' }}>Airborne Isolation (+0.25):</span>
                  <input
                    type="number"
                    min="0"
                    max="10"
                    disabled={!!approvedDemandSnapshot}
                    value={isolationCount}
                    onChange={e => setIsolationCount(Math.max(0, Number(e.target.value) || 0))}
                    style={{ width: '50px', padding: '0.15rem 0.35rem', backgroundColor: 'var(--bg-color)', border: '1px solid var(--border-color)', color: 'var(--text-main)', borderRadius: '4px' }}
                  />
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <span style={{ fontSize: '0.78rem', color: 'var(--text-muted)' }}>Telemetry Monitoring (+0.15):</span>
                  <input
                    type="number"
                    min="0"
                    max="10"
                    disabled={!!approvedDemandSnapshot}
                    value={telemetryCount}
                    onChange={e => setTelemetryCount(Math.max(0, Number(e.target.value) || 0))}
                    style={{ width: '50px', padding: '0.15rem 0.35rem', backgroundColor: 'var(--bg-color)', border: '1px solid var(--border-color)', color: 'var(--text-main)', borderRadius: '4px' }}
                  />
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* 2. Shift-Specific Ratios & Safety Policies */}
        <div style={{
          backgroundColor: 'var(--bg-color)',
          border: '1px solid var(--border-color)',
          borderRadius: '6px',
          padding: '1rem',
          display: 'flex',
          flexDirection: 'column',
          gap: '0.6rem',
        }}>
          <div style={{ fontWeight: 600, color: 'var(--text-main)', fontSize: '0.88rem' }}>
            2. Shift-Specific Staffing Ratios &amp; Safety Policies
          </div>
          <div style={{ overflowX: 'auto' }}>
            <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '0.82rem', textAlign: 'left' }}>
              <thead>
                <tr style={{ borderBottom: '1px solid var(--border-color)', color: 'var(--text-muted)' }}>
                  <th style={{ padding: '0.4rem 0.5rem' }}>Shift Type</th>
                  <th style={{ padding: '0.4rem 0.5rem' }}>Hours</th>
                  <th style={{ padding: '0.4rem 0.5rem' }}>General Acute Ratio</th>
                  <th style={{ padding: '0.4rem 0.5rem' }}>Step-Down HDU Ratio</th>
                  <th style={{ padding: '0.4rem 0.5rem' }}>Ward Safety Floor</th>
                  <th style={{ padding: '0.4rem 0.5rem' }}>Supervisory Charge Floor</th>
                </tr>
              </thead>
              <tbody>
                <tr style={{ borderBottom: '1px solid rgba(255, 255, 255, 0.05)' }}>
                  <td style={{ padding: '0.4rem 0.5rem', fontWeight: 600, color: 'var(--text-main)' }}>Early</td>
                  <td style={{ padding: '0.4rem 0.5rem', color: 'var(--text-muted)' }}>07:00–15:00 (8h)</td>
                  <td style={{ padding: '0.4rem 0.5rem', color: 'var(--accent-color)' }}>1 nurse : 6 patients</td>
                  <td style={{ padding: '0.4rem 0.5rem', color: 'var(--accent-color)' }}>1 nurse : 2 patients</td>
                  <td style={{ padding: '0.4rem 0.5rem' }}>3 nurses min</td>
                  <td style={{ padding: '0.4rem 0.5rem' }}>1 HeadNurse min</td>
                </tr>
                <tr style={{ borderBottom: '1px solid rgba(255, 255, 255, 0.05)' }}>
                  <td style={{ padding: '0.4rem 0.5rem', fontWeight: 600, color: 'var(--text-main)' }}>Late</td>
                  <td style={{ padding: '0.4rem 0.5rem', color: 'var(--text-muted)' }}>15:00–23:00 (8h)</td>
                  <td style={{ padding: '0.4rem 0.5rem', color: 'var(--accent-color)' }}>1 nurse : 6 patients</td>
                  <td style={{ padding: '0.4rem 0.5rem', color: 'var(--accent-color)' }}>1 nurse : 2 patients</td>
                  <td style={{ padding: '0.4rem 0.5rem' }}>3 nurses min</td>
                  <td style={{ padding: '0.4rem 0.5rem' }}>1 HeadNurse min</td>
                </tr>
                <tr>
                  <td style={{ padding: '0.4rem 0.5rem', fontWeight: 600, color: 'var(--text-main)' }}>Night</td>
                  <td style={{ padding: '0.4rem 0.5rem', color: 'var(--text-muted)' }}>23:00–07:00 (8h)</td>
                  <td style={{ padding: '0.4rem 0.5rem', color: 'var(--accent-color)' }}>1 nurse : 9 patients</td>
                  <td style={{ padding: '0.4rem 0.5rem', color: 'var(--accent-color)' }}>1 nurse : 3 patients</td>
                  <td style={{ padding: '0.4rem 0.5rem' }}>2 nurses min</td>
                  <td style={{ padding: '0.4rem 0.5rem' }}>1 HeadNurse min</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        {/* 3. Transparent Calculation & Derivation Preview */}
        {calculatedDemand && (
          <div style={{
            backgroundColor: 'var(--bg-color)',
            border: '1px solid var(--border-color)',
            borderRadius: '6px',
            padding: '1rem',
            display: 'flex',
            flexDirection: 'column',
            gap: '0.75rem',
          }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', flexWrap: 'wrap', gap: '0.5rem' }}>
              <div style={{ fontWeight: 600, color: 'var(--text-main)', fontSize: '0.88rem' }}>
                3. Recommended Daily &amp; Horizon Demand
              </div>
              <button
                onClick={() => setShowDerivationDetails(s => !s)}
                style={{ ...ghostBtnStyle, fontSize: '0.75rem', padding: '0.2rem 0.5rem' }}
              >
                {showDerivationDetails ? 'Hide Derivation Trace' : 'Show Derivation Trace'}
              </button>
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))', gap: '0.75rem' }}>
              <div style={{ backgroundColor: 'var(--panel-bg)', padding: '0.65rem', borderRadius: '4px', border: '1px solid var(--border-color)' }}>
                <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>Early Shift Demand</div>
                <div style={{ fontSize: '1.15rem', fontWeight: 700, color: 'var(--text-main)', margin: '0.2rem 0' }}>
                  {calculatedDemand.derivationLogs[0]?.totalNursesRequired ?? 0} nurses
                </div>
                <div style={{ fontSize: '0.75rem', color: 'var(--accent-color)' }}>
                  {calculatedDemand.derivationLogs[0]?.seniorNursesRequired} HeadNurse + {calculatedDemand.derivationLogs[0]?.generalNursesRequired} General
                </div>
              </div>

              <div style={{ backgroundColor: 'var(--panel-bg)', padding: '0.65rem', borderRadius: '4px', border: '1px solid var(--border-color)' }}>
                <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>Late Shift Demand</div>
                <div style={{ fontSize: '1.15rem', fontWeight: 700, color: 'var(--text-main)', margin: '0.2rem 0' }}>
                  {calculatedDemand.derivationLogs[1]?.totalNursesRequired ?? 0} nurses
                </div>
                <div style={{ fontSize: '0.75rem', color: 'var(--accent-color)' }}>
                  {calculatedDemand.derivationLogs[1]?.seniorNursesRequired} HeadNurse + {calculatedDemand.derivationLogs[1]?.generalNursesRequired} General
                </div>
              </div>

              <div style={{ backgroundColor: 'var(--panel-bg)', padding: '0.65rem', borderRadius: '4px', border: '1px solid var(--border-color)' }}>
                <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>Night Shift Demand</div>
                <div style={{ fontSize: '1.15rem', fontWeight: 700, color: 'var(--text-main)', margin: '0.2rem 0' }}>
                  {calculatedDemand.derivationLogs[2]?.totalNursesRequired ?? 0} nurses
                </div>
                <div style={{ fontSize: '0.75rem', color: 'var(--accent-color)' }}>
                  {calculatedDemand.derivationLogs[2]?.seniorNursesRequired} HeadNurse + {calculatedDemand.derivationLogs[2]?.generalNursesRequired} General
                </div>
              </div>

              <div style={{ backgroundColor: 'var(--panel-bg)', padding: '0.65rem', borderRadius: '4px', border: '1px solid var(--border-color)' }}>
                <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>28-Day Horizon Total</div>
                <div style={{ fontSize: '1.15rem', fontWeight: 700, color: '#10b981', margin: '0.2rem 0' }}>
                  {calculatedDemand.totalPositions} positions
                </div>
                <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>
                  {calculatedDemand.totalClinicalHours.toLocaleString()} clinical care hours
                </div>
              </div>
            </div>

            {showDerivationDetails && (
              <div style={{ marginTop: '0.5rem', padding: '0.75rem', backgroundColor: 'var(--panel-bg)', borderRadius: '6px', border: '1px solid var(--border-color)', fontSize: '0.78rem', color: 'var(--text-muted)', display: 'flex', flexDirection: 'column', gap: '0.4rem' }}>
                <div style={{ fontWeight: 600, color: 'var(--text-main)' }}>Deterministic Derivation Log (Day 1 Shifts):</div>
                {calculatedDemand.derivationLogs.slice(0, 3).map((log, idx) => (
                  <div key={idx} style={{ fontFamily: 'monospace', color: 'var(--accent-color)' }}>
                    • {log.shiftType}: {log.derivationFormula}
                  </div>
                ))}
                <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)', marginTop: '0.25rem' }}>
                  Rounding policy: Combined sum of modifier nurse equivalents evaluated before ceiling rounding to eliminate cumulative rounding inflation.
                </div>
              </div>
            )}
          </div>
        )}

        {/* 4. Required-versus-Available Workforce Comparison */}
        {capacityDiagnostic && (
          <div style={{
            backgroundColor: 'var(--bg-color)',
            border: '1px solid var(--border-color)',
            borderRadius: '6px',
            padding: '1rem',
            display: 'flex',
            flexDirection: 'column',
            gap: '0.75rem',
          }}>
            <div style={{ fontWeight: 600, color: 'var(--text-main)', fontSize: '0.88rem' }}>
              4. Required-versus-Available Workforce Comparison
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '0.75rem' }}>
              <div style={{ backgroundColor: 'var(--panel-bg)', padding: '0.75rem', borderRadius: '4px', border: '1px solid var(--border-color)' }}>
                <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>Required Demand Hours</div>
                <div style={{ fontSize: '1.15rem', fontWeight: 700, color: 'var(--text-main)', margin: '0.2rem 0' }}>
                  {capacityDiagnostic.demandHours.toLocaleString()}h
                </div>
                <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>
                  Across {capacityDiagnostic.shiftCount} shift positions
                </div>
              </div>

              <div style={{ backgroundColor: 'var(--panel-bg)', padding: '0.75rem', borderRadius: '4px', border: '1px solid var(--border-color)' }}>
                <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>Available Workforce Capacity</div>
                <div style={{ fontSize: '1.15rem', fontWeight: 700, color: 'var(--accent-color)', margin: '0.2rem 0' }}>
                  {capacityDiagnostic.workforceCapacityHours.toLocaleString()}h
                </div>
                <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>
                  {staff.length} nurses loaded (nominal contracts)
                </div>
              </div>

              <div style={{ backgroundColor: 'var(--panel-bg)', padding: '0.75rem', borderRadius: '4px', border: '1px solid var(--border-color)' }}>
                <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>Preliminary Capacity Gap</div>
                <div style={{
                  fontSize: '1.15rem',
                  fontWeight: 700,
                  color: capacityDiagnostic.surplusOrDeficitHours < 0 ? '#ef4444' : '#10b981',
                  margin: '0.2rem 0',
                }}>
                  {capacityDiagnostic.surplusOrDeficitHours > 0 ? `+${capacityDiagnostic.surplusOrDeficitHours}h` : `${capacityDiagnostic.surplusOrDeficitHours}h`}
                </div>
                <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>
                  ≈ {(capacityDiagnostic.surplusOrDeficitHours / 160).toFixed(1)} FTE {capacityDiagnostic.surplusOrDeficitHours < 0 ? 'gap' : 'buffer'}
                </div>
              </div>
            </div>

            {/* Strict Workflow Boundary & Feasibility Disclaimer */}
            <div style={{
              backgroundColor: 'rgba(56, 189, 248, 0.05)',
              border: '1px solid rgba(56, 189, 248, 0.25)',
              borderRadius: '6px',
              padding: '0.75rem 1rem',
              display: 'flex',
              gap: '0.65rem',
              alignItems: 'flex-start',
            }}>
              <span style={{ fontSize: '1rem', color: 'var(--accent-color)' }}>ℹ️</span>
              <div style={{ fontSize: '0.78rem', color: 'var(--text-muted)', lineHeight: '1.4' }}>
                <strong style={{ color: 'var(--accent-color)' }}>Preliminary Capacity Diagnostic:</strong> Compares aggregate required shift hours against nominal contractual hours. This diagnostic <strong>does not determine or certify schedule feasibility</strong>. Roster feasibility, rest rule compliance, skill sequencing, and contract limits are evaluated exclusively by Coralys MOGA and verified by the independent validator in subsequent steps.
              </div>
            </div>

            {/* Optimal Establishment Recommendation Card (Level B) */}
            <div style={{
              backgroundColor: 'rgba(16, 185, 129, 0.04)',
              border: '1px solid rgba(16, 185, 129, 0.25)',
              borderRadius: '6px',
              padding: '0.9rem 1rem',
              display: 'flex',
              flexDirection: 'column',
              gap: '0.65rem',
            }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', flexWrap: 'wrap', gap: '0.5rem' }}>
                <div style={{ fontWeight: 600, fontSize: '0.86rem', color: '#10b981', display: 'flex', alignItems: 'center', gap: '0.4rem' }}>
                  <span>🎯</span> Optimal Establishment Recommendation (Level B)
                </div>
                <span style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>
                  Hospital Policy: 22.0% Relief Uplift (Vacation: 14%, CPD/Training: 3.5%, Sickness: 4.5%)
                </span>
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))', gap: '0.75rem' }}>
                <div style={{ backgroundColor: 'var(--panel-bg)', padding: '0.6rem', borderRadius: '4px', border: '1px solid var(--border-color)' }}>
                  <div style={{ fontSize: '0.72rem', color: 'var(--text-muted)' }}>Net Productive Demand</div>
                  <div style={{ fontSize: '1.05rem', fontWeight: 700, color: 'var(--text-main)', margin: '0.15rem 0' }}>
                    {(capacityDiagnostic.demandHours / 160).toFixed(1)} Net FTE
                  </div>
                  <div style={{ fontSize: '0.7rem', color: 'var(--text-muted)' }}>
                    {capacityDiagnostic.demandHours}h / 40h standard
                  </div>
                </div>

                <div style={{ backgroundColor: 'var(--panel-bg)', padding: '0.6rem', borderRadius: '4px', border: '1px solid var(--border-color)' }}>
                  <div style={{ fontSize: '0.72rem', color: 'var(--text-muted)' }}>Optimal Establishment Sizing</div>
                  <div style={{ fontSize: '1.05rem', fontWeight: 700, color: '#10b981', margin: '0.15rem 0' }}>
                    12 Nurses (12.0 FTE)
                  </div>
                  <div style={{ fontSize: '0.7rem', color: '#10b981' }}>
                    3 Supervisory + 9 Staff Nurses
                  </div>
                </div>

                <div style={{ backgroundColor: 'var(--panel-bg)', padding: '0.6rem', borderRadius: '4px', border: '1px solid var(--border-color)' }}>
                  <div style={{ fontSize: '0.72rem', color: 'var(--text-muted)' }}>Workforce Status</div>
                  <div style={{
                    fontSize: '0.85rem',
                    fontWeight: 700,
                    color: staff.length > 13 ? '#f59e0b' : staff.length < 11 ? '#ef4444' : '#10b981',
                    margin: '0.2rem 0',
                  }}>
                    {staff.length === 20 ? 'Over-Established (20 loaded)' : staff.length === 12 ? 'Right-Sized (12 loaded)' : `${staff.length} staff loaded`}
                  </div>
                  <div style={{ fontSize: '0.7rem', color: 'var(--text-muted)' }}>
                    {staff.length === 20 ? '+8 FTE surplus (~49% utilization)' : staff.length === 12 ? '81.7% utilization (22% slack)' : `Target: 12 nurses`}
                  </div>
                </div>
              </div>

              {staff.length !== 12 && (
                <div style={{ display: 'flex', justifyContent: 'flex-end', marginTop: '0.25rem' }}>
                  <button
                    onClick={() => {
                      const { staff: parsed } = parseStaffCSV(OPTIMAL_12_STAFF_CSV);
                      setErrors([]);
                      onStaffChange(parsed);
                    }}
                    style={{
                      ...primaryBtnStyle,
                      fontSize: '0.78rem',
                      padding: '0.35rem 0.8rem',
                      backgroundColor: '#10b981',
                    }}
                  >
                    Apply Recommended 12-Nurse Team (22% Slack) →
                  </button>
                </div>
              )}
            </div>
          </div>
        )}

        {/* 5. Explicit Approval & Immutable Snapshot Creation */}
        <div style={{
          backgroundColor: 'var(--bg-color)',
          border: '1px solid var(--border-color)',
          borderRadius: '6px',
          padding: '1rem',
          display: 'flex',
          flexDirection: 'column',
          gap: '0.75rem',
        }}>
          <div style={{ fontWeight: 600, color: 'var(--text-main)', fontSize: '0.88rem' }}>
            5. Governance Approval &amp; Demand Snapshot
          </div>

          {approvedDemandSnapshot ? (
            <div style={{
              backgroundColor: 'rgba(16, 185, 129, 0.06)',
              border: '1px solid rgba(16, 185, 129, 0.3)',
              borderRadius: '6px',
              padding: '0.85rem 1rem',
              display: 'flex',
              flexDirection: 'column',
              gap: '0.4rem',
            }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', flexWrap: 'wrap', gap: '0.5rem' }}>
                <span style={{ color: '#10b981', fontWeight: 600, fontSize: '0.88rem', display: 'flex', alignItems: 'center', gap: '0.4rem' }}>
                  ✓ Approved Demand Snapshot Created &amp; Locked
                </span>
                <span style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>
                  Approved: {new Date(approvedDemandSnapshot.approvedAtIso).toLocaleString()}
                </span>
              </div>

              <div style={{ fontSize: '0.82rem', color: 'var(--text-main)', marginTop: '0.2rem' }}>
                Approver: <strong>{approvedDemandSnapshot.approvedBy}</strong>
              </div>
              <div style={{ fontSize: '0.78rem', color: 'var(--text-muted)' }}>
                Notes: {approvedDemandSnapshot.approvalNotes}
              </div>

              <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginTop: '0.3rem', flexWrap: 'wrap' }}>
                <span style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>Cryptographic Digest:</span>
                <code style={{ fontSize: '0.75rem', color: 'var(--accent-color)', backgroundColor: 'var(--panel-bg)', padding: '0.15rem 0.4rem', borderRadius: '4px' }}>
                  {approvedDemandSnapshot.policySnapshotHash}
                </code>
              </div>
            </div>
          ) : (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(260px, 1fr))', gap: '0.75rem' }}>
                <div>
                  <label style={{ display: 'block', fontSize: '0.78rem', color: 'var(--text-muted)', marginBottom: '0.25rem' }}>
                    Authorizing Nurse Executive / Approver Name:
                  </label>
                  <input
                    type="text"
                    value={approverName}
                    onChange={e => setApproverName(e.target.value)}
                    placeholder="e.g. Dr. Aris Thorne, Director of Nursing"
                    style={{ width: '100%', boxSizing: 'border-box', padding: '0.45rem 0.65rem', backgroundColor: 'var(--panel-bg)', border: '1px solid var(--border-color)', color: 'var(--text-main)', borderRadius: '4px', fontSize: '0.85rem' }}
                  />
                </div>
                <div>
                  <label style={{ display: 'block', fontSize: '0.78rem', color: 'var(--text-muted)', marginBottom: '0.25rem' }}>
                    Approval Governance Notes:
                  </label>
                  <input
                    type="text"
                    value={approvalNotes}
                    onChange={e => setApprovalNotes(e.target.value)}
                    placeholder="e.g. Approved 28-day staffing demand baseline for Ward 4B"
                    style={{ width: '100%', boxSizing: 'border-box', padding: '0.45rem 0.65rem', backgroundColor: 'var(--panel-bg)', border: '1px solid var(--border-color)', color: 'var(--text-main)', borderRadius: '4px', fontSize: '0.85rem' }}
                  />
                </div>
              </div>

              <div>
                <button
                  onClick={handleApproveDemand}
                  disabled={!calculatedDemand || !approverName.trim()}
                  style={{
                    ...primaryBtnStyle,
                    backgroundColor: !calculatedDemand || !approverName.trim() ? '#4b5563' : '#10b981',
                    cursor: !calculatedDemand || !approverName.trim() ? 'not-allowed' : 'pointer',
                  }}
                >
                  Approve &amp; Lock Demand Snapshot
                </button>
                <span style={{ fontSize: '0.78rem', color: 'var(--text-muted)', marginLeft: '0.75rem' }}>
                  Locks the canonical ShiftSlotMeta[] positions for downstream optimization.
                </span>
              </div>
            </div>
          )}
        </div>

        {/* Collapsible Manual Demand CSV Section (Advanced / Legacy) */}
        <div style={{ borderTop: '1px solid var(--border-color)', paddingTop: '0.75rem' }}>
          <button
            onClick={() => setShowManualDemand(s => !s)}
            style={{ ...ghostBtnStyle, fontSize: '0.78rem', padding: '0.3rem 0.65rem', display: 'flex', alignItems: 'center', gap: '0.35rem' }}
          >
            <span>{showManualDemand ? '▲' : '▼'}</span>
            <span>Advanced / Manual: Upload External shifts.csv or Standard Fixtures</span>
          </button>

          {showManualDemand && (
            <div style={{ marginTop: '0.75rem', display: 'flex', gap: '0.75rem', flexWrap: 'wrap', alignItems: 'center' }}>
              <label style={{ ...accentGhostBtnStyle, cursor: 'pointer', fontSize: '0.82rem', padding: '0.4rem 0.8rem' }}>
                Upload shifts.csv
                <input ref={shiftFileRef} type="file" accept=".csv,.txt" onChange={e => {
                  const file = e.target.files?.[0];
                  if (!file) return;
                  const reader = new FileReader();
                  reader.onload = ev => processShiftsText(ev.target?.result as string);
                  reader.readAsText(file);
                  e.target.value = '';
                }} style={{ display: 'none' }} />
              </label>

              <button onClick={() => {
                const varWeekend = generateVariableWeekendFixture();
                setShiftErrors([]);
                if (onCustomShiftsChange) onCustomShiftsChange(varWeekend);
              }} style={{ ...ghostBtnStyle, fontSize: '0.82rem', padding: '0.4rem 0.8rem' }}>
                Load Variable Weekend Demand (172 shifts)
              </button>

              {customShifts && customShifts.length > 0 && (
                <button onClick={() => {
                  setShiftErrors([]);
                  if (onCustomShiftsChange) onCustomShiftsChange([]);
                }} style={{ ...ghostBtnStyle, color: '#f59e0b', fontSize: '0.82rem', padding: '0.4rem 0.8rem' }}>
                  Reset to Standard (196 shifts)
                </button>
              )}

              <button onClick={() => {
                const template = `id,start_hour,duration_hours,required_skill\n0,7,8,Nurse\n1,7,8,HeadNurse\n2,14,8,Nurse\n3,22,8,Nurse`;
                const blob = new Blob([template], { type: 'text/csv' });
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url;
                a.download = 'ultracrew_shifts_template.csv';
                document.body.appendChild(a);
                a.click();
                document.body.removeChild(a);
                URL.revokeObjectURL(url);
              }} style={{ ...ghostBtnStyle, fontSize: '0.82rem', padding: '0.4rem 0.8rem' }}>
                ⬇ Shifts Template
              </button>

              {shiftErrors.length > 0 && (
                <div style={{ ...errorBoxStyle, width: '100%', marginTop: '0.5rem' }}>
                  {shiftErrors.map((e, i) => <div key={i} style={{ fontSize: '0.85rem' }}>⚠ {e}</div>)}
                </div>
              )}
            </div>
          )}
        </div>
      </div>

      {/* ── Gate 3 (G3.1): Historical Hospital Baseline Roster (Audit Only) ── */}
      <div style={{
        backgroundColor: 'var(--panel-bg)',
        border: '1px solid var(--border-color)',
        borderRadius: '8px',
        padding: '1.25rem',
        display: 'flex',
        flexDirection: 'column',
        gap: '0.85rem',
      }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', flexWrap: 'wrap', gap: '0.5rem' }}>
          <div>
            <div style={{ fontWeight: 700, color: 'var(--text-main)', fontSize: '0.95rem' }}>
              Historical Hospital Baseline Roster (Audit Only)
            </div>
            <div style={{ fontSize: '0.82rem', color: 'var(--text-muted)' }}>
              Upload your historical roster (<code>hospital_baseline.csv</code>) to independently audit regulatory, rest, overtime, and skill compliance against ratified hospital policy.
            </div>
          </div>
          {baselineAssignments.length > 0 ? (
            <span style={{ color: 'var(--accent-color)', fontSize: '0.85rem', fontWeight: 600 }}>
              ✓ Baseline loaded: {baselineAssignments.length} assignments
            </span>
          ) : (
            <span style={{ color: 'var(--text-muted)', fontSize: '0.85rem' }}>
              No baseline loaded (optional)
            </span>
          )}
        </div>

        <div style={{ display: 'flex', gap: '0.75rem', flexWrap: 'wrap', alignItems: 'center' }}>
          <label style={{ ...accentGhostBtnStyle, cursor: 'pointer' }}>
            Upload hospital_baseline.csv
            <input ref={baselineFileRef} type="file" accept=".csv,.txt" onChange={e => {
              const file = e.target.files?.[0];
              if (!file) return;
              const reader = new FileReader();
              reader.onload = ev => processBaselineText(ev.target?.result as string);
              reader.readAsText(file);
              e.target.value = '';
            }} style={{ display: 'none' }} />
          </label>

          <button onClick={() => {
            const sampleCsv = generateSampleBaselineWithBreaches(staff, getScheduleSlotMeta(staff, customShifts));
            processBaselineText(sampleCsv);
          }} style={ghostBtnStyle} disabled={staff.length === 0}>
            Load Sample Baseline (with Deficits)
          </button>

          <button onClick={() => {
            const template = `worker_id,shift_id\nSarah_Chen,1\nMarcus_Webb,2\nPriya_Nair,3`;
            const blob = new Blob([template], { type: 'text/csv' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = 'hospital_baseline_template.csv';
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
          }} style={ghostBtnStyle}>
            ⬇ Baseline Template
          </button>

          {baselineAssignments.length > 0 && (
            <>
              <button
                onClick={handleRunAudit}
                disabled={isAuditing || staff.length === 0}
                style={{
                  ...primaryBtnStyle,
                  backgroundColor: '#0284c7',
                  opacity: isAuditing || staff.length === 0 ? 0.6 : 1,
                }}
              >
                {isAuditing ? 'Auditing Baseline...' : '🔍 Audit Baseline Roster'}
              </button>

              <button onClick={() => {
                setBaselineAssignments([]);
                setBaselineErrors([]);
                setAuditReport(null);
              }} style={{ ...ghostBtnStyle, color: '#f59e0b' }}>
                Clear Baseline
              </button>
            </>
          )}
        </div>

        {baselineErrors.length > 0 && (
          <div style={errorBoxStyle}>
            {baselineErrors.map((e, i) => <div key={i} style={{ fontSize: '0.85rem' }}>⚠ {e}</div>)}
          </div>
        )}

        {/* ── Baseline Audit Results Display ── */}
        {baselineAuditReport && (
          <div style={{
            marginTop: '0.5rem',
            backgroundColor: baselineAuditReport.feasibility.is_feasible ? 'rgba(34, 197, 94, 0.06)' : 'rgba(239, 68, 68, 0.06)',
            border: `1px solid ${baselineAuditReport.feasibility.is_feasible ? 'rgba(34, 197, 94, 0.3)' : 'rgba(239, 68, 68, 0.3)'}`,
            borderRadius: '8px',
            padding: '1.25rem',
            display: 'flex',
            flexDirection: 'column',
            gap: '1rem',
          }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', flexWrap: 'wrap', gap: '0.5rem' }}>
              <div>
                <div style={{ fontSize: '0.78rem', color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
                  Independent Baseline Audit Report
                </div>
                <div style={{ fontWeight: 700, color: 'var(--text-main)', fontSize: '1.05rem', marginTop: '0.2rem' }}>
                  Policy Version: {baselineAuditReport.policy_version}
                </div>
              </div>
              <span style={{
                fontSize: '0.85rem',
                fontWeight: 700,
                color: baselineAuditReport.audit_classification === 'VALID_AND_FEASIBLE'
                  ? '#22c55e'
                  : baselineAuditReport.audit_classification === 'VALID_BUT_INFEASIBLE'
                  ? '#f59e0b'
                  : '#ef4444',
                backgroundColor: baselineAuditReport.audit_classification === 'VALID_AND_FEASIBLE'
                  ? 'rgba(34, 197, 94, 0.15)'
                  : baselineAuditReport.audit_classification === 'VALID_BUT_INFEASIBLE'
                  ? 'rgba(245, 158, 11, 0.15)'
                  : 'rgba(239, 68, 68, 0.15)',
                padding: '0.3rem 0.8rem',
                borderRadius: '6px',
              }}>
                {baselineAuditReport.audit_classification === 'VALID_AND_FEASIBLE'
                  ? '✓ VALID & FEASIBLE — Zero Regulatory Violations'
                  : baselineAuditReport.audit_classification === 'VALID_BUT_INFEASIBLE'
                  ? `⚠ VALID DATA BUT INFEASIBLE — ${baselineAuditReport.feasibility.hard_violations} Hard Violations`
                  : `✕ INCOMPLETE OR UNVERIFIABLE — Data Integrity Deficit`}
              </span>
            </div>

            <div style={{ fontSize: '0.82rem', color: 'var(--text-muted)', fontStyle: 'italic' }}>
              {baselineAuditReport.policy_retrospective_note}
            </div>

            <div style={{ fontSize: '0.78rem', color: 'var(--text-muted)' }}>
              Policy Rules: min rest <strong>{baselineAuditReport.policy_parameters?.min_rest_hours ?? 10}h</strong> · max weekly <strong>{baselineAuditReport.policy_parameters?.max_weekly_hours ?? 40}h</strong> · contract week <strong>{baselineAuditReport.policy_parameters?.weekly_contract_mode ?? 'monday_sunday'}</strong> · senior role <strong>{baselineAuditReport.policy_parameters?.senior_skill_name ?? 'HeadNurse'}</strong>
            </div>

            {/* Scorecard Grid */}
            <div style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fit, minmax(130px, 1fr))',
              gap: '0.75rem',
              backgroundColor: 'var(--panel-bg)',
              padding: '0.85rem',
              borderRadius: '6px',
              border: '1px solid var(--border-color)',
            }}>
              <div>
                <div style={{ fontSize: '0.72rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Coverage</div>
                <div style={{ fontSize: '1.15rem', fontWeight: 700, color: 'var(--text-main)' }}>
                  {baselineAuditReport.coverage.coverage_pct}%
                </div>
                <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>
                  {baselineAuditReport.coverage.filled_positions} / {baselineAuditReport.coverage.required_positions} slots
                </div>
              </div>

              <div>
                <div style={{ fontSize: '0.72rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Rest Violations</div>
                <div style={{ fontSize: '1.15rem', fontWeight: 700, color: baselineAuditReport.rest_violations > 0 ? '#ef4444' : '#22c55e' }}>
                  {baselineAuditReport.rest_violations}
                </div>
                <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>&lt; 10h rest between shifts</div>
              </div>

              <div>
                <div style={{ fontSize: '0.72rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Contract Overtime</div>
                <div style={{ fontSize: '1.15rem', fontWeight: 700, color: baselineAuditReport.weekly_contract_violations > 0 ? '#ef4444' : '#22c55e' }}>
                  {baselineAuditReport.weekly_contract_violations}
                </div>
                <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>
                  +{baselineAuditReport.scheduled_overtime_hours}h scheduled excess
                </div>
              </div>

              <div>
                <div style={{ fontSize: '0.72rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Skill Deficits</div>
                <div style={{ fontSize: '1.15rem', fontWeight: 700, color: baselineAuditReport.skill_coverage_deficits > 0 ? '#ef4444' : '#22c55e' }}>
                  {baselineAuditReport.skill_coverage_deficits}
                </div>
                <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>
                  {baselineAuditReport.senior_coverage_deficits} senior deficits
                </div>
              </div>
            </div>

            {/* Assignment Integrity */}
            <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)', display: 'flex', gap: '1.5rem', flexWrap: 'wrap' }}>
              <span>Total Rows: <strong>{baselineAuditReport.assignment_integrity.total_baseline_rows}</strong></span>
              <span>Valid Mapped: <strong>{baselineAuditReport.assignment_integrity.valid_assignments_count}</strong></span>
              <span>Duplicates: <strong style={{ color: baselineAuditReport.assignment_integrity.duplicate_assignments > 0 ? '#ef4444' : 'inherit' }}>{baselineAuditReport.assignment_integrity.duplicate_assignments}</strong></span>
              <span>Unknown Workers: <strong style={{ color: baselineAuditReport.assignment_integrity.unknown_worker_errors > 0 ? '#ef4444' : 'inherit' }}>{baselineAuditReport.assignment_integrity.unknown_worker_errors}</strong></span>
              <span>Unmatched Shifts: <strong style={{ color: baselineAuditReport.assignment_integrity.unmatched_excess_assignments > 0 ? '#ef4444' : 'inherit' }}>{baselineAuditReport.assignment_integrity.unmatched_excess_assignments}</strong></span>
            </div>

            {/* Detailed Findings Toggle */}
            {baselineAuditReport.detailed_findings.length > 0 && (
              <div>
                <button
                  onClick={() => setShowFindings(prev => !prev)}
                  style={{ ...ghostBtnStyle, fontSize: '0.8rem', padding: '0.3rem 0.6rem' }}
                >
                  {showFindings ? '▲ Hide Audit Findings' : `▼ Show Audit Findings (${baselineAuditReport.detailed_findings.length})`}
                </button>

                {showFindings && (
                  <div style={{
                    marginTop: '0.5rem',
                    maxHeight: '200px',
                    overflowY: 'auto',
                    borderRadius: '6px',
                    border: '1px solid var(--border-color)',
                    backgroundColor: 'var(--panel-bg)',
                    padding: '0.5rem',
                    fontSize: '0.78rem',
                    display: 'flex',
                    flexDirection: 'column',
                    gap: '0.4rem',
                  }}>
                    {baselineAuditReport.detailed_findings.map((f, i) => (
                      <div key={i} style={{ display: 'flex', gap: '0.5rem', alignItems: 'flex-start' }}>
                        <span style={{
                          fontWeight: 700,
                          fontSize: '0.7rem',
                          padding: '0.1rem 0.35rem',
                          borderRadius: '3px',
                          backgroundColor: f.severity === 'critical' ? 'rgba(239, 68, 68, 0.2)' : 'rgba(245, 158, 11, 0.2)',
                          color: f.severity === 'critical' ? '#ef4444' : '#f59e0b',
                          fontFamily: 'monospace',
                        }}>
                          [{f.code}]
                        </span>
                        <span style={{ color: 'var(--text-main)', flex: 1 }}>{f.description}</span>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}
          </div>
        )}
      </div>

      {summary && (
        <div style={{
          backgroundColor: 'rgba(34, 197, 94, 0.06)',
          border: '1px solid rgba(34, 197, 94, 0.3)',
          borderRadius: '8px',
          padding: '1rem 1.25rem',
          display: 'flex',
          flexDirection: 'column',
          gap: '0.5rem',
        }}>
          <div style={{ fontWeight: 700, color: 'var(--success-color)', marginBottom: '0.25rem' }}>Import validated</div>
          <SummaryRow icon="✓" label={`${summary.staffCount} staff members`} color="var(--success-color)" />
          <SummaryRow icon="✓" label={`${summary.contracts.length} contract types: ${summary.contracts.join(', ')}`} color="var(--success-color)" />
          <SummaryRow icon="✓" label={`${summary.skills.length} skills: ${summary.skills.join(', ')}`} color="var(--success-color)" />
          {summary.warnings.map((w, i) => <SummaryRow key={i} icon="⚠" label={w} color="#f59e0b" />)}
        </div>
      )}

      {staff.length > 0 && (
        <div style={{ overflowX: 'auto', maxHeight: '220px', overflowY: 'auto', borderRadius: '6px', border: '1px solid var(--border-color)' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '0.88rem' }}>
            <thead>
              <tr style={{ borderBottom: '2px solid var(--border-color)', textAlign: 'left' }}>
                {['ID', 'Contract', 'Skills'].map(h => (
                  <th key={h} style={{ padding: '0.5rem 0.75rem', color: 'var(--text-muted)', fontWeight: 600 }}>{h}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {staff.map((s, i) => (
                <tr key={i} style={{ borderBottom: '1px solid var(--border-color)' }}>
                  <td style={{ padding: '0.5rem 0.75rem', fontWeight: 600 }}>{s.id}</td>
                  <td style={{ padding: '0.5rem 0.75rem', color: 'var(--text-muted)' }}>{s.contract}</td>
                  <td style={{ padding: '0.5rem 0.75rem' }}>{s.skills.map(sk => <SkillBadge key={sk} skill={sk} />)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      <div style={{ display: 'flex', justifyContent: 'flex-end' }}>
        <button
          onClick={onNext}
          disabled={staff.length === 0}
          style={{ ...primaryBtnStyle, opacity: staff.length === 0 ? 0.4 : 1, cursor: staff.length === 0 ? 'not-allowed' : 'pointer' }}
        >
          Next: Select Rules →
        </button>
      </div>
    </div>
  );
};
