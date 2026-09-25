import React from 'react';
import { S } from '../styles';
import StarRating from './StarRating';
import CommercialAssessmentPanel from './CommercialAssessmentPanel';

export default function DecisionWorkspace({
  recDecisions,
  updateRec,
  toggleRejectionReason,
  REJECTION_REASONS,
  MANUAL_EDIT_REASONS,
  manualEdits,
  addManualEdit,
  updateManualEdit,
  removeManualEdit,
  ADOPTION_OPTIONS,
  WILLING_TO_PILOT_OPTIONS,
  NEXT_STEPS_OPTIONS,
  overallRating,
  setOverallRating,
  adoptionSignal,
  setAdoptionSignal,
  adoptionBarrier,
  setAdoptionBarrier,
  dispatcherComments,
  setDispatcherComments,
  orgName,
  setOrgName,
  baselineSchedulingMins,
  setBaselineSchedulingMins,
  baselineDisruptionMins,
  setBaselineDisruptionMins,
  productGaps,
  setProductGaps,
  willingToPilot,
  setWillingToPilot,
  nextSteps,
  setNextSteps,
  submitting,
  submitError,
  handleSubmit
}) {
  return (
    <div style={{ marginTop: '20px' }}>
      <div style={S.card}>
        <div style={S.cardTitle}>Planner Feedback & Manual Edits</div>
        <div style={S.cardSub}>
          Would you change anything in this schedule? Record manual edits here.
        </div>
        
        {manualEdits.map((edit, idx) => (
          <div key={idx} style={S.editCard}>
            <label style={S.label}>Reason for edit</label>
            <select style={{ ...S.select, marginBottom: '8px' }} value={edit.reason} onChange={e => updateManualEdit(idx, { reason: e.target.value })}>
              <option value="">Select reason…</option>
              {MANUAL_EDIT_REASONS.map(r => <option key={r} value={r}>{r}</option>)}
            </select>
            <label style={S.label}>Comment (optional)</label>
            <input style={{ ...S.input, marginBottom: '8px' }} value={edit.comment} onChange={e => updateManualEdit(idx, { comment: e.target.value })} placeholder="What did you change?" />
            <button style={{ ...S.btn('danger'), fontSize: '12px', padding: '6px 12px' }} onClick={() => removeManualEdit(idx)}>Remove</button>
          </div>
        ))}
        <button style={S.btn('secondary')} onClick={addManualEdit}>+ Record Manual Edit</button>
      </div>

      {recDecisions && recDecisions.length > 0 && (
        <div style={S.card}>
          <div style={S.cardTitle}>Recommendations</div>
          <div style={S.cardSub}>
            For each recommendation, decide whether to accept, reject, or modify it.
          </div>
          {recDecisions.map((d, i) => (
            <div key={i} style={S.recCard(d.action)}>
              <div style={S.recText}>{d.recommendation_text}</div>
              <div style={S.recActions}>
                <button style={{ ...S.btn(d.action === 'accepted' ? 'success' : 'secondary'), opacity: d.action === 'accepted' ? 1 : 0.65 }} onClick={() => updateRec(i, { action: 'accepted' })}>✓ Accept</button>
                <button style={{ ...S.btn(d.action === 'rejected' ? 'danger' : 'secondary'), opacity: d.action === 'rejected' ? 1 : 0.65 }} onClick={() => updateRec(i, { action: 'rejected' })}>✗ Reject</button>
                <button style={{ ...S.btn(d.action === 'modified' ? 'primary' : 'secondary'), opacity: d.action === 'modified' ? 1 : 0.65 }} onClick={() => updateRec(i, { action: 'modified' })}>✎ Modify</button>
              </div>
              {(d.action === 'rejected' || d.action === 'modified') && (
                <div style={S.recSubCard}>
                  <div style={S.recSubLabel}>Why?</div>
                  {REJECTION_REASONS.map(r => (
                    <label key={r} style={S.checkRow}>
                      <input type="checkbox" checked={d.rejection_reasons.includes(r)} onChange={e => toggleRejectionReason(i, r, e.target.checked)} />
                      <span style={S.checkLabel}>{r}</span>
                    </label>
                  ))}
                  <input style={{ ...S.input, marginTop: '8px', marginBottom: '0' }} value={d.rejection_comment} onChange={e => updateRec(i, { rejection_comment: e.target.value })} placeholder="Additional comments…" />
                </div>
              )}
              <div style={{ ...S.recSubCard, marginTop: '10px' }}>
                <StarRating label="Was this explanation useful?" value={d.explanation_rating} onChange={v => updateRec(i, { explanation_rating: v })} />
              </div>
            </div>
          ))}
        </div>
      )}

      <div style={S.card}>
        <div style={S.cardTitle}>Session Debrief</div>
        <div style={S.cardSub}>
          A few final questions about your experience with the scheduling tool today.
        </div>

        <label style={S.label}>Overall satisfaction with UltraCrew</label>
        <StarRating value={overallRating} onChange={setOverallRating} />

        <div style={S.divider} />

        <label style={S.label}>Would you use this for tomorrow's roster?</label>
        <div style={{ display: 'flex', gap: '10px', flexWrap: 'wrap', marginBottom: '16px' }}>
          {ADOPTION_OPTIONS.map(o => (
            <button key={o.value} style={S.radioBtn(adoptionSignal === o.value)} onClick={() => setAdoptionSignal(o.value)}>{o.label}</button>
          ))}
        </div>

        {adoptionSignal && adoptionSignal !== 'yes' && (
          <div>
            <label style={S.label}>What prevented a stronger answer?</label>
            <textarea style={S.textarea} value={adoptionBarrier} onChange={e => setAdoptionBarrier(e.target.value)} placeholder="What would need to change for you to use this tomorrow?" />
          </div>
        )}

        <div style={S.divider} />

        <label style={S.label}>Any other comments?</label>
        <textarea style={S.textarea} value={dispatcherComments} onChange={e => setDispatcherComments(e.target.value)} placeholder="What surprised you? What was most useful? What was missing?" />

        <div style={S.divider} />

        <CommercialAssessmentPanel
          orgName={orgName} setOrgName={setOrgName}
          baselineSchedulingMins={baselineSchedulingMins} setBaselineSchedulingMins={setBaselineSchedulingMins}
          baselineDisruptionMins={baselineDisruptionMins} setBaselineDisruptionMins={setBaselineDisruptionMins}
          productGaps={productGaps} setProductGaps={setProductGaps}
          willingToPilot={willingToPilot} setWillingToPilot={setWillingToPilot} WILLING_TO_PILOT_OPTIONS={WILLING_TO_PILOT_OPTIONS}
          nextSteps={nextSteps} setNextSteps={setNextSteps} NEXT_STEPS_OPTIONS={NEXT_STEPS_OPTIONS}
        />

        {submitError && <div style={S.alert('error')}>{submitError}</div>}
        
        <div style={{ display: 'flex', gap: '8px', marginTop: '20px' }}>
          <button
            style={submitting || overallRating === 0 || !adoptionSignal ? S.btnDisabled : S.btn('success')}
            disabled={submitting || overallRating === 0 || !adoptionSignal}
            onClick={handleSubmit}
          >
            {submitting ? '⏳ Submitting…' : '✓ Submit'}
          </button>
        </div>
        {(overallRating === 0 || !adoptionSignal) && (
          <div style={{ fontSize: '12px', color: '#64748b', marginTop: '8px' }}>
            Please rate your overall satisfaction and answer the adoption question to submit.
          </div>
        )}
      </div>
    </div>
  );
}
