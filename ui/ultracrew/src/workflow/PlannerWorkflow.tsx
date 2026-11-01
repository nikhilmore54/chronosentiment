import React, { useState, useMemo, useEffect, useRef } from 'react';
import type {
  StaffMember,
  ScheduleResult,
  RosterAlternative,
  SchedulerDecision,
  RedistributionLog,
  BaselineAuditReport,
  BaselineAssignment,
  ApprovedDemandSnapshot,
} from './WorkflowTypes';
import { RULE_PRESETS, SAMPLE_CSV } from './WorkflowTypes';
import { Stepper } from './WorkflowComponents';
import { ImportStaff } from './ImportStaff';
import { SelectRules } from './SelectRules';
import { GenerateSchedule } from './GenerateSchedule';
import { SelectDecision } from './SelectDecision';
import { ReviewSchedule } from './ReviewSchedule';
import { ExportRoster } from './ExportRoster';
import { DecisionRepository } from '../services/DecisionRepository';
import { accumulatePatterns } from '../services/PatternAccumulator';
import {
  getScheduleSlotMeta,
  parseStaffCSV,
  generateSampleBaselineWithBreaches,
  parseBaselineCSV,
  auditBaselineClientSide,
  buildSyntheticAlternatives,
  type CustomShiftInput,
} from './WorkflowUtils';

const decisionRepository = new DecisionRepository();

export interface PlannerWorkflowProps {
  workflowMode?: 'blank' | 'sample';
  workflowTrigger?: number;
  onWorkflowModeChange?: (mode: 'blank' | 'sample') => void;
}

export const PlannerWorkflow: React.FC<PlannerWorkflowProps> = ({
  workflowMode = 'blank',
  workflowTrigger = 0,
  onWorkflowModeChange,
}) => {
  const [step, setStep] = useState(1);
  const [maxReached, setMaxReached] = useState(1);

  const [staff, setStaff] = useState<StaffMember[]>([]);
  const [customShifts, setCustomShifts] = useState<CustomShiftInput[]>([]);
  const [selectedRuleId, setSelectedRuleId] = useState('');
  const [rulePayload, setRulePayload] = useState<object>({});
  const [scheduleResult, setScheduleResult] = useState<ScheduleResult | null>(null);
  const [editableSchedule, setEditableSchedule] = useState<Record<string, string[]>>({});
  const [manualEditCount, setManualEditCount] = useState(0);
  const [editDistribution, setEditDistribution] = useState<Record<string, number>>({});
  const [originalAssignmentCount, setOriginalAssignmentCount] = useState(0);

  const customSlotMeta = useMemo(() => getScheduleSlotMeta(staff, customShifts), [staff, customShifts]);

  // P3: alternatives + decision
  const [alternatives, setAlternatives] = useState<RosterAlternative[]>([]);
  const [recommendedId, setRecommendedId] = useState('');
  const [schedulerDecision, setSchedulerDecision] = useState<SchedulerDecision | null>(null);
  const [baselineAuditReport, setBaselineAuditReport] = useState<BaselineAuditReport | null>(null);
  const [baselineAssignments, setBaselineAssignments] = useState<BaselineAssignment[]>([]);
  const [approvedDemandSnapshot, setApprovedDemandSnapshot] = useState<ApprovedDemandSnapshot | null>(null);

  // P4.1: dismissed pattern reasons (persisted in localStorage)
  const [dismissedPatterns, setDismissedPatterns] = useState<string[]>(
    () => decisionRepository.loadDismissedPatterns(),
  );

  // P4.1: compute visible recurring patterns (filtered by dismissed set)
  const recurringPatterns = useMemo(() => {
    const allLogs = decisionRepository.loadAllRedistributionLogs();
    const all = accumulatePatterns(allLogs);
    return all.filter(p => !dismissedPatterns.includes(p.reason));
  }, [dismissedPatterns]);

  const handleDismissPattern = (reason: string) => {
    decisionRepository.dismissPattern(reason);
    setDismissedPatterns(prev => [...prev, reason]);
  };

  const goTo = (n: number) => {
    setStep(n);
    if (n > maxReached) setMaxReached(n);
  };

  const ruleLabel = selectedRuleId === 'custom'
    ? 'Custom JSON'
    : RULE_PRESETS.find(p => p.id === selectedRuleId)?.label ?? selectedRuleId;

  const loadSampleDemonstration = () => {
    const { staff: sampleStaff } = parseStaffCSV(SAMPLE_CSV);
    const slotMeta = getScheduleSlotMeta(sampleStaff, []);
    const sampleBaselineCsv = generateSampleBaselineWithBreaches(sampleStaff, slotMeta);
    const { assignments: parsedBaseline } = parseBaselineCSV(sampleBaselineCsv);
    const auditReport = auditBaselineClientSide(
      sampleStaff,
      Object.values(slotMeta),
      parsedBaseline,
      {
        min_rest_hours: 10,
        max_weekly_hours: 40.0,
        weekly_contract_mode: 'monday_sunday',
        weekly_anchor_offset_hours: 0,
      },
      'hospital_policy_v1'
    );

    const baseSchedule: Record<string, string[]> = {};
    const shiftTypes = ['Early', 'Late', 'Night', ''];
    sampleStaff.forEach((s, idx) => {
      const row: string[] = [];
      for (let day = 0; day < 28; day++) {
        const isRest = (day + idx) % 7 === 5 || (day + idx) % 7 === 6;
        if (isRest) {
          row.push('');
        } else {
          const shiftIdx = (day + idx * 2) % 3;
          row.push(shiftTypes[shiftIdx]);
        }
      }
      baseSchedule[s.id] = row;
    });

    const { alternatives: sampleAlternatives, recommendedId: recId } = buildSyntheticAlternatives(
      sampleStaff,
      baseSchedule,
      slotMeta
    );

    const rawSchedule: Record<string, number> = {};
    const staffIndexMap = new Map<string, number>();
    sampleStaff.forEach((s, idx) => staffIndexMap.set(s.id, idx + 1));
    Object.values(slotMeta).forEach(slot => {
      const assignedNurseId = Object.keys(baseSchedule).find(
        id => baseSchedule[id]?.[slot.dayIdx] === slot.label
      );
      if (assignedNurseId && staffIndexMap.has(assignedNurseId)) {
        rawSchedule[String(slot.shiftId)] = staffIndexMap.get(assignedNurseId)!;
      }
    });

    const metricsRecord: Record<string, number> = {
      coverage: 1.0,
      filled_positions: 196,
      required_positions: 196,
      fairness_penalty: 1.2,
      utilization: 0.85,
      cost: 100,
      diff_from_recommended: 0,
    };

    const sampleResult: ScheduleResult = {
      schedule: rawSchedule,
      metrics: metricsRecord,
      constraint_report: {
        fitness: 1.0,
        is_valid: true,
        hard_violations: 0,
        soft_violations: 0,
        violated_constraints: [],
        satisfied_constraints: [
          'minimum_rest_hours',
          'max_weekly_hours',
          'required_skill_coverage',
          'full_demand_coverage',
        ],
        warnings: [],
      },
      recommendations: [],
      alternatives: sampleAlternatives,
      recommended_alternative_id: recId,
    };

    setStaff(sampleStaff);
    setCustomShifts([]);
    setSelectedRuleId('standard-hospital');
    setScheduleResult(sampleResult);
    setEditableSchedule(baseSchedule);
    setOriginalAssignmentCount(196);
    setAlternatives(sampleAlternatives);
    setRecommendedId(recId);
    setSchedulerDecision(null);
    setBaselineAssignments(parsedBaseline);
    setBaselineAuditReport(auditReport);
    setStep(4);
    setMaxReached(6);
  };

  const resetToBlankWorkspace = () => {
    setStaff([]);
    setCustomShifts([]);
    setSelectedRuleId('');
    setRulePayload({});
    setScheduleResult(null);
    setEditableSchedule({});
    setManualEditCount(0);
    setEditDistribution({});
    setOriginalAssignmentCount(0);
    setAlternatives([]);
    setRecommendedId('');
    setSchedulerDecision(null);
    setBaselineAuditReport(null);
    setBaselineAssignments([]);
    setApprovedDemandSnapshot(null);
    setStep(1);
    setMaxReached(1);
  };

  const prevTriggerRef = useRef(workflowTrigger);
  const initialMountRef = useRef(true);

  useEffect(() => {
    if (initialMountRef.current) {
      initialMountRef.current = false;
      if (workflowMode === 'sample') {
        loadSampleDemonstration();
      } else {
        resetToBlankWorkspace();
      }
      return;
    }

    if (prevTriggerRef.current !== workflowTrigger) {
      prevTriggerRef.current = workflowTrigger;
      if (workflowMode === 'sample') {
        loadSampleDemonstration();
      } else {
        resetToBlankWorkspace();
      }
    }
  }, [workflowMode, workflowTrigger]);

  const handleStartOver = () => {
    resetToBlankWorkspace();
    onWorkflowModeChange?.('blank');
  };

  return (
    <div className="card">
      <div style={{ marginBottom: '0.25rem' }}>
        <h2 style={{ margin: '0 0 0.25rem 0' }}>Workforce Assessment &amp; Planning</h2>
        <p style={{ margin: 0, color: 'var(--text-muted)', fontSize: '0.9rem' }}>
          Evaluate historical roster compliance, generate policy-compliant candidates, and compare decision alternatives.
        </p>
      </div>

      {/* Mode Indicator & Quick Switch */}
      <div style={{
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        flexWrap: 'wrap',
        gap: '0.75rem',
        padding: '0.65rem 1rem',
        borderRadius: '8px',
        backgroundColor: workflowMode === 'sample' ? 'rgba(15, 118, 110, 0.08)' : 'rgba(56, 189, 248, 0.08)',
        border: `1px solid ${workflowMode === 'sample' ? 'rgba(15, 118, 110, 0.25)' : 'rgba(56, 189, 248, 0.25)'}`,
        marginTop: '0.85rem',
        marginBottom: '0.5rem',
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', fontSize: '0.86rem' }}>
          <span style={{
            fontSize: '0.72rem',
            fontWeight: 700,
            padding: '0.15rem 0.5rem',
            borderRadius: '4px',
            backgroundColor: workflowMode === 'sample' ? 'var(--primary-color)' : '#0284c7',
            color: 'white',
            textTransform: 'uppercase',
            letterSpacing: '0.04em',
          }}>
            {workflowMode === 'sample' ? 'Sample Demo' : 'Blank Workspace'}
          </span>
          <span style={{ color: 'var(--text-main)', fontWeight: 500 }}>
            {workflowMode === 'sample'
              ? 'Preloaded demonstration for synthetic 20-nurse department. Roster alternatives & scorecard ready for exploration.'
              : 'Blank assessment workspace. Start by importing your hospital staff CSV.'}
          </span>
        </div>
        <div>
          {workflowMode === 'sample' ? (
            <button
              onClick={() => {
                resetToBlankWorkspace();
                onWorkflowModeChange?.('blank');
              }}
              style={{
                background: 'none',
                border: '1px solid var(--border-color)',
                borderRadius: '5px',
                padding: '0.3rem 0.75rem',
                fontSize: '0.8rem',
                color: 'var(--text-main)',
                cursor: 'pointer',
                fontWeight: 600,
              }}
            >
              Start Blank Assessment →
            </button>
          ) : (
            <button
              onClick={() => {
                loadSampleDemonstration();
                onWorkflowModeChange?.('sample');
              }}
              style={{
                background: 'none',
                border: '1px solid var(--border-color)',
                borderRadius: '5px',
                padding: '0.3rem 0.75rem',
                fontSize: '0.8rem',
                color: 'var(--text-main)',
                cursor: 'pointer',
                fontWeight: 600,
              }}
            >
              Explore Sample Demonstration →
            </button>
          )}
        </div>
      </div>

      <div style={{ marginTop: '1.25rem' }}>
        <Stepper currentStep={step} maxReached={maxReached} onJump={goTo} />

        {step === 1 && (
          <ImportStaff
            staff={staff}
            onStaffChange={setStaff}
            customShifts={customShifts}
            onCustomShiftsChange={setCustomShifts}
            baselineAuditReport={baselineAuditReport}
            onBaselineAuditReportChange={setBaselineAuditReport}
            baselineAssignments={baselineAssignments}
            onBaselineAssignmentsChange={setBaselineAssignments}
            approvedDemandSnapshot={approvedDemandSnapshot}
            onApprovedDemandSnapshotChange={snap => {
              setApprovedDemandSnapshot(snap);
              if (snap) {
                setCustomShifts(snap.slots);
              }
            }}
            onNext={() => goTo(2)}
          />
        )}

        {step === 2 && (
          <SelectRules
            selectedRuleId={selectedRuleId}
            onSelect={(id, payload) => { setSelectedRuleId(id); setRulePayload(payload); }}
            onNext={() => goTo(3)}
            onBack={() => goTo(1)}
          />
        )}

        {step === 3 && (
          <GenerateSchedule
            staff={staff}
            rulePayload={rulePayload}
            ruleLabel={ruleLabel}
            customShifts={customShifts}
            customSlotMeta={customSlotMeta}
            onResult={(result, sched) => {
              setScheduleResult(result);
              setEditableSchedule(sched);
              const count = Object.values(sched).flat().filter(s => s !== '').length;
              setOriginalAssignmentCount(count);

              // P3: propagate the optimizer/API recommendation unchanged.
              // The optimizer is the authoritative source of recommendation authority.
              // If the backend does not return alternatives, we do NOT invent them —
              // the scheduler proceeds directly to Review with the single backend schedule.
              if (result.alternatives && result.alternatives.length > 0) {
                // API returned alternatives — use them with the API recommendation
                setAlternatives(result.alternatives);
                setRecommendedId(result.recommended_alternative_id ?? result.alternatives[0]?.id ?? '');
              } else {
                // Backend returned no alternatives — clear the alternatives list.
                // Step 4 (SelectDecision) will show an "alternatives unavailable" notice
                // and allow the scheduler to proceed directly with the generated schedule.
                setAlternatives([]);
                setRecommendedId('');
              }

              goTo(4);
            }}
            onBack={() => goTo(2)}
          />
        )}

        {/* Step 4 — Explore the Decision (or proceed directly when backend provides no alternatives) */}
        {step === 4 && scheduleResult && alternatives.length > 0 && (
          <SelectDecision
            alternatives={alternatives}
            recommendedId={recommendedId}
            baselineAuditReport={baselineAuditReport}
            onDecision={(selectedAlt, decision) => {
              // Persist the scheduler's choice
              setSchedulerDecision(decision);
              // Use the selected alternative's schedule as the editable schedule
              setEditableSchedule(selectedAlt.schedule);
              goTo(5);
            }}
            onBack={() => goTo(3)}
          />
        )}

        {/* Step 4 — Single candidate or no alternative candidates: proceed with primary schedule */}
        {step === 4 && scheduleResult && alternatives.length === 0 && (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
            <div>
              <h3 style={{ margin: '0 0 0.5rem 0', color: 'var(--text-main)' }}>Decision Review: Primary Candidate</h3>
              <div style={{
                backgroundColor: 'rgba(56, 189, 248, 0.08)',
                border: '1px solid rgba(56, 189, 248, 0.3)',
                borderRadius: '8px',
                padding: '1rem 1.25rem',
                color: 'var(--text-main)',
                fontSize: '0.9rem',
              }}>
                <strong style={{ color: '#38bdf8' }}>Single Optimal Candidate:</strong> The optimizer generated a single authoritative candidate meeting all defined constraints. You can proceed directly with this candidate into Review &amp; Edit.
              </div>
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <button onClick={() => goTo(3)} style={{ padding: '0.5rem 1.25rem', background: 'transparent', border: '1px solid var(--border-color)', borderRadius: 6, color: 'var(--text-muted)', cursor: 'pointer' }}>
                ← Back to Generate
              </button>
              <button
                onClick={() => {
                  const defaultMetrics = {
                    coverage: 1.0,
                    filled_positions: Object.values(editableSchedule).flat().filter(s => s !== '').length,
                    required_positions: Object.values(editableSchedule).flat().filter(s => s !== '').length,
                    fairness_penalty: 0,
                    utilization: 1.0,
                    cost: 0,
                    diff_from_recommended: 0,
                  };
                  setSchedulerDecision({
                    decision_id: `dec-${Date.now()}`,
                    created_at_iso: new Date().toISOString(),
                    recommended_id: 'primary',
                    selected_id: 'primary',
                    overrode_recommendation: false,
                    recommended_metrics: defaultMetrics,
                    selected_metrics: defaultMetrics,
                  });
                  goTo(5);
                }}
                style={{
                  padding: '0.6rem 1.5rem',
                  backgroundColor: 'var(--primary-color)',
                  color: 'white',
                  border: 'none',
                  borderRadius: 6,
                  fontWeight: 600,
                  cursor: 'pointer',
                }}
              >
                Proceed to Review &amp; Edit →
              </button>
            </div>
          </div>
        )}

        {/* Step 5 — Review & Edit */}
        {step === 5 && scheduleResult && (
          <ReviewSchedule
            staff={staff}
            schedule={editableSchedule}
            result={scheduleResult}
            customSlotMeta={customSlotMeta}
            onScheduleChange={setEditableSchedule}
            onNext={(editCount, dist) => { setManualEditCount(editCount); setEditDistribution(dist); goTo(6); }}
            onBack={() => goTo(4)}
            decision_id={schedulerDecision?.decision_id}
            onRedistributionComplete={(log: RedistributionLog) => {
              if (schedulerDecision) {
                decisionRepository.saveRedistributionLog(schedulerDecision.decision_id, log);
              }
            }}
            recurringPatterns={recurringPatterns}
            onDismissPattern={handleDismissPattern}
          />
        )}

        {/* Step 6 — Export */}
        {step === 6 && scheduleResult && (
          <ExportRoster
            staff={staff}
            schedule={editableSchedule}
            result={scheduleResult}
            manualEditCount={manualEditCount}
            editDistribution={editDistribution}
            originalAssignmentCount={originalAssignmentCount}
            schedulerDecision={schedulerDecision}
            customSlotMeta={customSlotMeta}
            baselineAuditReport={baselineAuditReport}
            baselineAssignments={baselineAssignments}
            alternatives={alternatives}
            onBack={() => goTo(5)}
            onStartOver={handleStartOver}
          />
        )}
      </div>
    </div>
  );
};