import React, { useState } from 'react';
import type { StaffMember, ScheduleResult } from './WorkflowTypes';
import { RULE_PRESETS } from './WorkflowTypes';
import { Stepper } from './WorkflowComponents';
import { ImportStaff } from './ImportStaff';
import { SelectRules } from './SelectRules';
import { GenerateSchedule } from './GenerateSchedule';
import { ReviewSchedule } from './ReviewSchedule';
import { ExportRoster } from './ExportRoster';

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
  };

  return (
    <div className="card">
      <div style={{ marginBottom: '0.25rem' }}>
        <h2 style={{ margin: '0 0 0.25rem 0' }}>Import &amp; Schedule</h2>
        <p style={{ margin: 0, color: 'var(--text-muted)', fontSize: '0.9rem' }}>
          Bring your own data, generate a roster, edit it, and export.
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
            onResult={(result, sched) => { setScheduleResult(result); setEditableSchedule(sched); goTo(4); }}
            onBack={() => goTo(2)}
          />
        )}

        {step === 4 && scheduleResult && (
          <ReviewSchedule
            staff={staff}
            schedule={editableSchedule}
            result={scheduleResult}
            onScheduleChange={setEditableSchedule}
            onNext={(editCount, dist) => { setManualEditCount(editCount); setEditDistribution(dist); goTo(5); }}
            onBack={() => goTo(3)}
          />
        )}

        {step === 5 && scheduleResult && (
          <ExportRoster
            staff={staff}
            schedule={editableSchedule}
            result={scheduleResult}
            manualEditCount={manualEditCount}
            editDistribution={editDistribution}
            onBack={() => goTo(4)}
            onStartOver={handleStartOver}
          />
        )}
      </div>
    </div>
  );
};