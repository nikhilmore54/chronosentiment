import React from 'react';
import { S, theme } from '../styles';

export default function ConvergenceChart({ history = [] }) {
  if (!history || history.length === 0) return null;

  const width = 640;
  const height = 180;
  const padding = 36;

  const maxFit = Math.max(...history.map((h) => h.fitness), 1);
  const minFit = Math.min(...history.map((h) => h.fitness), 0);
  const fitRange = maxFit - minFit || 1;

  const points = history.map((pt, idx) => {
    const x = padding + (idx / (history.length - 1)) * (width - padding * 2);
    const y = height - padding - ((pt.fitness - minFit) / fitRange) * (height - padding * 2);
    return `${x},${y}`;
  }).join(' ');

  return (
    <div style={S.card}>
      <div style={S.cardTitle}>
        <span>Evolutionary Convergence Telemetry</span>
        <span style={S.badge('success')}>MOGA PARETO DYNAMICS</span>
      </div>
      <div style={S.cardSubtitle}>
        Fitness objective descent across generational iterations. Smooth asymptotic convergence toward the global Pareto frontier.
      </div>

      <div style={{ marginTop: '16px', display: 'flex', justifyContent: 'center' }}>
        <svg viewBox={`0 0 ${width} ${height}`} style={{ width: '100%', maxWidth: '700px', height: 'auto', background: 'rgba(15, 23, 42, 0.8)', borderRadius: '8px' }}>
          {/* Grid lines */}
          <line x1={padding} y1={padding} x2={width - padding} y2={padding} stroke="rgba(255,255,255,0.08)" />
          <line x1={padding} y1={height / 2} x2={width - padding} y2={height / 2} stroke="rgba(255,255,255,0.08)" />
          <line x1={padding} y1={height - padding} x2={width - padding} y2={height - padding} stroke="rgba(255,255,255,0.15)" />

          {/* Area under curve */}
          <polygon
            points={`${padding},${height - padding} ${points} ${width - padding},${height - padding}`}
            fill="url(#grad)"
            opacity="0.3"
          />

          {/* Line */}
          <polyline
            fill="none"
            stroke={theme.primaryHover}
            strokeWidth="3"
            points={points}
          />

          {/* Points */}
          {history.map((pt, idx) => {
            const x = padding + (idx / (history.length - 1)) * (width - padding * 2);
            const y = height - padding - ((pt.fitness - minFit) / fitRange) * (height - padding * 2);
            return (
              <circle
                key={idx}
                cx={x}
                cy={y}
                r="4"
                fill={theme.primary}
                stroke="#fff"
                strokeWidth="1.5"
              />
            );
          })}

          <defs>
            <linearGradient id="grad" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stopColor="#0ea5e9" stopOpacity="0.8" />
              <stop offset="100%" stopColor="#0ea5e9" stopOpacity="0.0" />
            </linearGradient>
          </defs>
        </svg>
      </div>

      <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: '10px', fontSize: '11px', color: theme.textMuted }}>
        <span>Gen 0 (Greedy Construction Floor: {Math.round(maxFit)})</span>
        <span>Asymptotic Optimum Reached (Final Fitness: {Math.round(minFit)})</span>
      </div>
    </div>
  );
}
