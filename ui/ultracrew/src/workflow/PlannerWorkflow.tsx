import React, { useState, useMemo } from 'react';
import type { StaffMember, ScheduleResult, RosterAlternative, SchedulerDecision, RedistributionLog } from './WorkflowTypes';
import { RULE_PRESETS } from './WorkflowTypes';
import { Stepper } from './WorkflowComponents';
import { ImportStaff } from './ImportStaff';
import { SelectRules } from './SelectRules';
import { GenerateSchedule } from './GenerateSchedule';
import { SelectDecision } from './SelectDecision';
import { ReviewSchedule } from './ReviewSchedule';
import { ExportRoster } from './ExportRoster';
import { DecisionRepository } from '../services/DecisionRepository';
import { accumulatePatterns } from '../services/PatternAccumulator';

const decisionRepository = new DecisionRepository();

export const PlannerWorkflow: React.FC = () => {
  const [step, setStep] = useState(1);
  const [maxReached, setMaxReached] = useState(1);

  const [staff, setStaff] = useState<StaffMember[]>([]);
  const [selectedRuleId, setSelectedRuleId] = useState('');
  const [rulePayload, setRulePayload] = useState<object>({});
  const [scheduleResult, setScheduleResult] = useState<ScheduleResult | null>(null);
  const [editableSchedule, setEditableSchedule] = useState<Record<string, string[]>>({});
  const [manualEditCount, setManualEditCount] = useState(0);
  const [editDistribution, setEditDistribution] = useState<Record<string, number>>({});
  const [originalAssignmentCount, setOriginalAssignmentCount] = useState(0);

  // P3: alternatives + decision
  const [alternatives, setAlternatives] = useState<RosterAlternative[]>([]);
  const [recommendedId, setRecommendedId] = useState('');
  const [schedulerDecision, setSchedulerDecision] = useState<SchedulerDecision | null>(null);

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

  const handleStartOver = () => {
    setStep(1);
    setMaxReached(1);
    setStaff([]);
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
  };

  return (
    <div className="card">
      <div style={{ marginBottom: '0.25rem' }}>
        <h2 style={{ margin: '0 0 0.25rem 0' }}>Import &amp; Schedule</h2>
        <p style={{ margin: 0, color: 'var(--text-muted)', fontSize: '0.9rem' }}>
          Bring your own data, generate a roster, explore the decision, edit it, and export.
        </p>
      </div>

      <div style={{ marginTop: '1.75rem' }}>
        <Stepper currentStep={step} maxReached={maxReached} onJump={goTo} />

        {step === 1 && (
          <ImportStaff
            staff={staff}
            onStaffChange={setStaff}
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

        {/* Step 4 — Alternatives unavailable: backend returned no candidates — workflow stops here */}
        {step === 4 && scheduleResult && alternatives.length === 0 && (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
            <div>
              <h3 style={{ margin: '0 0 0.5rem 0', color: 'var(--text-main)' }}>Explore the Decision</h3>
              <div style={{
                backgroundColor: 'rgba(245, 158, 11, 0.08)',
                border: '1px solid rgba(245, 158, 11, 0.3)',
                borderRadius: '8px',
                padding: '1rem 1.25rem',
                color: '#fcd34d',
                fontSize: '0.9rem',
              }}>
                <strong>Decision alternatives unavailable.</strong> The optimizer did not return candidate alternatives
                required for the decision step. The workflow cannot proceed until the backend provides this data.
                Please regenerate the schedule or contact support.
              </div>
            </div>
            <div style={{ display: 'flex', justifyContent: 'flex-start' }}>
              <button onClick={() => goTo(3)} style={{ padding: '0.5rem 1.25rem', background: 'transparent', border: '1px solid var(--border-color)', borderRadius: 6, color: 'var(--text-muted)', cursor: 'pointer' }}>
                ← Back to Generate
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
            onBack={() => goTo(5)}
            onStartOver={handleStartOver}
          />
        )}
      </div>
    </div>
  );
};