import React, { useState } from 'react';
import type { StaffMember, ScheduleResult, RosterAlternative, SchedulerDecision } from './WorkflowTypes';
import { RULE_PRESETS } from './WorkflowTypes';
import { Stepper } from './WorkflowComponents';
import { ImportStaff } from './ImportStaff';
import { SelectRules } from './SelectRules';
import { GenerateSchedule } from './GenerateSchedule';
import { SelectDecision } from './SelectDecision';
import { ReviewSchedule } from './ReviewSchedule';
import { ExportRoster } from './ExportRoster';
import { buildSyntheticAlternatives, rankAlternatives } from './WorkflowUtils';

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

              // P3: build alternatives, then always run rankAlternatives() as the
              // single authoritative ranking step. The API recommendation is never
              // used directly — it may reflect optimizer-internal scoring that does
              // not match the product-level coverage-priority hierarchy.
              let altsToRank: typeof alternatives;
              if (result.alternatives && result.alternatives.length > 0) {
                // API returned alternatives — use them but re-rank
                altsToRank = result.alternatives;
              } else {
                // Fallback: derive alternatives from the editable schedule
                const { alternatives: synAlts } = buildSyntheticAlternatives(staff, sched);
                altsToRank = synAlts;
              }
              // rankAlternatives() is the authoritative adapter-layer decision
              const ranked = rankAlternatives(altsToRank);
              setAlternatives(altsToRank);
              setRecommendedId(ranked.recommendedId);

              goTo(4);
            }}
            onBack={() => goTo(2)}
          />
        )}

        {/* Step 4 — P3: Explore the Decision */}
        {step === 4 && alternatives.length > 0 && (
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

        {/* Step 5 — Review & Edit */}
        {step === 5 && scheduleResult && (
          <ReviewSchedule
            staff={staff}
            schedule={editableSchedule}
            result={scheduleResult}
            onScheduleChange={setEditableSchedule}
            onNext={(editCount, dist) => { setManualEditCount(editCount); setEditDistribution(dist); goTo(6); }}
            onBack={() => goTo(4)}
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