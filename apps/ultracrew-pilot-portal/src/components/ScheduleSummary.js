import React from 'react';
import { S } from '../styles';

export default function ScheduleSummary({ submitted }) {
  if (!submitted) return null;
  
  const presented = submitted.recommendations_presented;
  const overrideRate = presented > 0 ? ((submitted.recommendations_rejected || 0) / presented * 100).toFixed(1) : '0.0';
  
  return (
    <div style={S.card}>
      <div style={S.cardTitle}>Session summary</div>
      <div style={S.cardSub}>Thank you — your session is complete.</div>
      <table style={S.summaryTable}><tbody>
        <tr><td style={S.summaryTd}>Evidence ID</td><td style={S.summaryTdVal}>{submitted.id}</td></tr>
        <tr><td style={S.summaryTd}>Timestamp</td><td style={S.summaryTdVal}>{submitted.timestamp}</td></tr>
        <tr><td style={S.summaryTd}>Dispatcher</td><td style={S.summaryTdVal}>{submitted.dispatcher_id} · {submitted.dispatcher_role}</td></tr>
        <tr><td style={S.summaryTd}>Scenario</td><td style={S.summaryTdVal}>{submitted.scenario_id}</td></tr>
        <tr><td style={S.summaryTd}>Coverage</td><td style={S.summaryTdVal}>{submitted.coverage_pct.toFixed(1)}%</td></tr>
        <tr><td style={S.summaryTd}>Hard violations</td><td style={S.summaryTdVal}>{submitted.hard_violations}</td></tr>
        <tr><td style={S.summaryTd}>Rest violations</td><td style={S.summaryTdVal}>{submitted.rest_violations}</td></tr>
        <tr><td style={S.summaryTd}>Recommendations presented</td><td style={S.summaryTdVal}>{presented}</td></tr>
        <tr><td style={S.summaryTd}>Accepted</td><td style={S.summaryTdVal}>{submitted.recommendations_accepted}</td></tr>
        <tr><td style={S.summaryTd}>Rejected</td><td style={S.summaryTdVal}>{submitted.recommendations_rejected}</td></tr>
        <tr><td style={S.summaryTd}>Override rate</td><td style={S.summaryTdVal}>{overrideRate}%</td></tr>
        <tr><td style={S.summaryTd}>Manual edits</td><td style={S.summaryTdVal}>{submitted.manual_edits}</td></tr>
        <tr><td style={S.summaryTd}>Avg explanation rating</td><td style={S.summaryTdVal}>{submitted.explanation_usefulness}/5</td></tr>
        {submitted.disruption_recovery_secs != null && <tr><td style={S.summaryTd}>Disruption recovery time</td><td style={S.summaryTdVal}>{submitted.disruption_recovery_secs.toFixed(0)}s</td></tr>}
        <tr><td style={S.summaryTd}>Runtime</td><td style={S.summaryTdVal}>{submitted.runtime_secs.toFixed(2)}s</td></tr>
      </tbody></table>
      <div style={{ marginTop: '20px' }}>
        <button style={S.btn('secondary')} onClick={() => window.location.reload()}>Start New Session</button>
      </div>
    </div>
  );
}
