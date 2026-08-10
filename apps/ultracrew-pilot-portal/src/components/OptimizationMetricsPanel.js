import React from 'react';
import { S } from '../styles';

export default function OptimizationMetricsPanel({ result, runtimeSecs }) {
  if (!result) return null;
  
  return (
    <div style={S.kpiGrid}>
      <div style={S.kpiCard}>
        <div style={S.kpiValue}>{result.schedule ? Object.keys(result.schedule).length : '—'}</div>
        <div style={S.kpiLabel}>Shifts assigned</div>
      </div>
      <div style={S.kpiCard}>
        <div style={S.kpiValue}>{result.constraint_report ? result.constraint_report.hard_violations : '—'}</div>
        <div style={S.kpiLabel}>Rule violations</div>
      </div>
      <div style={S.kpiCard}>
        <div style={S.kpiValue}>{result.constraint_report ? result.constraint_report.rest_violations : '—'}</div>
        <div style={S.kpiLabel}>Rest violations</div>
      </div>
      <div style={S.kpiCard}>
        <div style={S.kpiValue}>{runtimeSecs ? runtimeSecs.toFixed(2) : '—'}s</div>
        <div style={S.kpiLabel}>Generated in</div>
      </div>
    </div>
  );
}
