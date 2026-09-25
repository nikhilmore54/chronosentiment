// SelectDecision.tsx — P3: Decision Selection & Comparison UI
// Governance: UI layer only. No optimizer changes. P1.1 not smuggled in.
// Acceptance criteria: all 11 criteria from ULTRAROSTER_DECISION_SUPPORT_ROADMAP.md

import React, { useState, useMemo } from 'react';
import type { RosterAlternative, SchedulerDecision } from './WorkflowTypes';
import { primaryBtnStyle, ghostBtnStyle } from './WorkflowComponents';
import { DecisionRepository } from '../services/DecisionRepository';

const repo = new DecisionRepository();

// ── Helpers ──────────────────────────────────────────────────────────────────

function pct(v: number): string {
  return `${Math.round(v * 100)}%`;
}

function fmtCost(v: number): string {
  return v.toFixed(0);
}

// ── MetricBar ─────────────────────────────────────────────────────────────────

const MetricBar: React.FC<{
  value: number; // 0–1
  color: string;
  width?: number;
}> = ({ value, color, width = 80 }) => (
  <div style={{ display: 'inline-block', width, height: 6, background: 'var(--border-color)', borderRadius: 3, verticalAlign: 'middle' }}>
    <div style={{ width: `${Math.min(100, Math.max(0, value * 100))}%`, height: '100%', background: color, borderRadius: 3, transition: 'width 0.3s' }} />
  </div>
);

// ── ComparisonTable ───────────────────────────────────────────────────────────

const ComparisonTable: React.FC<{
  alternatives: RosterAlternative[];
  recommendedId: string;
  selectedId: string;
}> = ({ alternatives, recommendedId, selectedId }) => {
  // Canonical metric order — each metric appears exactly once.
  // Coverage → Positions filled → Utilization → Fairness penalty → Cost index → Δ from recommended
  const rows: Array<{
    key: keyof RosterAlternative['metrics'];
    label: string;
    format: (v: number) => string;
    higherIsBetter: boolean;
    color: string;
  }> = [
    { key: 'coverage',              label: 'Coverage',           format: pct,                                    higherIsBetter: true,  color: '#38bdf8' },
    { key: 'filled_positions',      label: 'Positions filled',   format: (v: number) => String(v),              higherIsBetter: true,  color: '#38bdf8' },
    { key: 'utilization',           label: 'Utilization',        format: pct,                                    higherIsBetter: true,  color: '#34d399' },
    { key: 'fairness_penalty',      label: 'Fairness penalty',   format: v => v.toFixed(1),                     higherIsBetter: false, color: '#f59e0b' },
    { key: 'cost',                  label: 'Cost index',         format: fmtCost,                               higherIsBetter: false, color: '#818cf8' },
    { key: 'diff_from_recommended', label: 'Δ from recommended', format: v => v === 0 ? '—' : `${v} shifts`,   higherIsBetter: false, color: '#94a3b8' },
  ];

  return (
    <div style={{ overflowX: 'auto' }}>
      <table style={{ borderCollapse: 'collapse', width: '100%', fontSize: '0.85rem' }}>
        <thead>
          <tr>
            <th style={{ textAlign: 'left', padding: '0.4rem 0.75rem', color: 'var(--text-muted)', fontWeight: 600, fontSize: '0.78rem', borderBottom: '1px solid var(--border-color)' }}>
              Metric
            </th>
            {alternatives.map(alt => (
              <th key={alt.id} style={{
                textAlign: 'center',
                padding: '0.4rem 0.75rem',
                color: alt.id === selectedId ? 'var(--accent-color)' : alt.id === recommendedId ? '#34d399' : 'var(--text-main)',
                fontWeight: 700,
                fontSize: '0.82rem',
                borderBottom: '1px solid var(--border-color)',
                whiteSpace: 'nowrap',
              }}>
                {alt.label}
                {alt.id === recommendedId && <span style={{ marginLeft: '0.3rem', fontSize: '0.7rem', color: '#34d399' }}>★ rec</span>}
                {alt.id === selectedId && alt.id !== recommendedId && <span style={{ marginLeft: '0.3rem', fontSize: '0.7rem', color: 'var(--accent-color)' }}>✓ selected</span>}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map(row => {
            const values = alternatives.map(a => a.metrics[row.key]);
            const best = row.higherIsBetter ? Math.max(...values) : Math.min(...values);
            return (
              <tr key={row.key} style={{ borderBottom: '1px solid rgba(255,255,255,0.04)' }}>
                <td style={{ padding: '0.45rem 0.75rem', color: 'var(--text-muted)', whiteSpace: 'nowrap' }}>{row.label}</td>
                {alternatives.map((alt, i) => {
                  const v = values[i];
                  const isBest = v === best;
                  const barVal = row.key === 'fairness_penalty' ? Math.max(0, 1 - v / 3)
                    : row.key === 'cost' ? Math.max(0, 1 - (v - 90) / 20)
                    : row.key === 'diff_from_recommended' ? 0
                    : v;
                  return (
                    <td key={alt.id} style={{
                      padding: '0.45rem 0.75rem',
                      textAlign: 'center',
                      background: alt.id === selectedId ? 'rgba(56,189,248,0.04)' : 'transparent',
                    }}>
                      <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '0.2rem' }}>
                        <span style={{ fontWeight: isBest ? 700 : 400, color: isBest ? row.color : 'var(--text-main)' }}>
                          {row.format(v)}
                        </span>
                        {row.key === 'filled_positions' && alt.metrics.required_positions > 0 && (
                          <span style={{ fontSize: '0.7rem', color: 'var(--text-muted)' }}>
                            / {alt.metrics.required_positions} req.
                          </span>
                        )}
                        {row.key !== 'diff_from_recommended' && row.key !== 'filled_positions' && (
                          <MetricBar value={barVal} color={row.color} width={60} />
                        )}
                      </div>
                    </td>
                  );
                })}
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
};

// ── AlternativeCard ───────────────────────────────────────────────────────────

const AlternativeCard: React.FC<{
  alt: RosterAlternative;
  isRecommended: boolean;
  isSelected: boolean;
  onSelect: () => void;
}> = ({ alt, isRecommended, isSelected, onSelect }) => {
  const borderColor = isSelected
    ? 'var(--accent-color)'
    : isRecommended
    ? '#34d399'
    : 'var(--border-color)';

  return (
    <div style={{
      border: `2px solid ${borderColor}`,
      borderRadius: 10,
      padding: '1rem 1.1rem',
      background: isSelected ? 'rgba(56,189,248,0.04)' : 'var(--panel-bg)',
      transition: 'border-color 0.15s',
      flex: '1 1 220px',
      minWidth: 200,
    }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '0.5rem' }}>
        <div>
          <div style={{ fontWeight: 700, fontSize: '0.95rem', color: isSelected ? 'var(--accent-color)' : 'var(--text-main)' }}>
            {alt.label}
          </div>
          {isRecommended && (
            <div style={{ fontSize: '0.72rem', color: '#34d399', fontWeight: 600, marginTop: '0.1rem' }}>
              ★ UltraRoster recommends this
            </div>
          )}
        </div>
        {isSelected && (
          <span style={{ fontSize: '0.75rem', color: 'var(--accent-color)', fontWeight: 700, background: 'rgba(56,189,248,0.12)', padding: '0.15rem 0.5rem', borderRadius: 999 }}>
            ✓ Selected
          </span>
        )}
      </div>

      {/* Coverage headline — demand-based: N / M required positions filled */}
      {alt.metrics.required_positions > 0 && (
        <div style={{ marginBottom: '0.5rem', fontSize: '0.85rem', fontWeight: 700, color: '#38bdf8' }}>
          {pct(alt.metrics.coverage)} coverage
          <span style={{ fontWeight: 400, color: 'var(--text-muted)', marginLeft: '0.4rem', fontSize: '0.78rem' }}>
            ({alt.metrics.filled_positions} / {alt.metrics.required_positions} required positions filled)
          </span>
        </div>
      )}

      <div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap', marginBottom: '0.75rem' }}>
        {alt.metrics.required_positions === 0 && (
          <span style={{ fontSize: '0.75rem', padding: '0.15rem 0.5rem', background: 'rgba(56,189,248,0.1)', borderRadius: 999, color: '#38bdf8' }}>
            {pct(alt.metrics.coverage)} coverage
          </span>
        )}
        <span style={{ fontSize: '0.75rem', padding: '0.15rem 0.5rem', background: 'rgba(245,158,11,0.1)', borderRadius: 999, color: '#f59e0b' }}>
          {alt.metrics.fairness_penalty.toFixed(1)} fairness
        </span>
        <span style={{ fontSize: '0.75rem', padding: '0.15rem 0.5rem', background: 'rgba(129,140,248,0.1)', borderRadius: 999, color: '#818cf8' }}>
          cost {fmtCost(alt.metrics.cost)}
        </span>
      </div>

      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.25rem', marginBottom: '0.85rem' }}>
        {alt.reasons.map((r, i) => (
          <div key={i} style={{ fontSize: '0.8rem', color: 'var(--text-muted)', display: 'flex', gap: '0.4rem' }}>
            <span style={{ flexShrink: 0 }}>·</span>
            <span>{r}</span>
          </div>
        ))}
      </div>

      <button
        onClick={onSelect}
        disabled={isSelected}
        style={{
          ...primaryBtnStyle,
          width: '100%',
          opacity: isSelected ? 0.5 : 1,
          cursor: isSelected ? 'default' : 'pointer',
          fontSize: '0.85rem',
          padding: '0.45rem 0',
        }}
      >
        {isSelected ? '✓ Selected' : `Select ${alt.label}`}
      </button>
    </div>
  );
};

// ── DecisionBanner ────────────────────────────────────────────────────────────

const DecisionBanner: React.FC<{
  decision: SchedulerDecision;
  alternatives: RosterAlternative[];
}> = ({ decision, alternatives }) => {
  const rec = alternatives.find(a => a.id === decision.recommended_id);
  const sel = alternatives.find(a => a.id === decision.selected_id);
  if (!rec || !sel) return null;

  // Coverage gap between selected and recommended (positive = selected has more)
  const covGap = sel.metrics.filled_positions - rec.metrics.filled_positions;
  const materialCovGap = Math.abs(covGap) > 5;

  return (
    <div style={{
      background: decision.overrode_recommendation ? 'rgba(245,158,11,0.06)' : 'rgba(52,211,153,0.06)',
      border: `1px solid ${decision.overrode_recommendation ? 'rgba(245,158,11,0.3)' : 'rgba(52,211,153,0.3)'}`,
      borderRadius: 8,
      padding: '0.75rem 1rem',
      fontSize: '0.85rem',
    }}>
      {decision.overrode_recommendation ? (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '0.3rem' }}>
          <span>
            <strong style={{ color: '#f59e0b' }}>Override recorded.</strong>{' '}
            UltraRoster recommended <strong>{rec.label}</strong> ({rec.metrics.filled_positions}/{rec.metrics.required_positions} positions) — you selected <strong style={{ color: 'var(--accent-color)' }}>{sel.label}</strong> ({sel.metrics.filled_positions}/{sel.metrics.required_positions} positions).
          </span>
          {materialCovGap && covGap < 0 && (
            <span style={{ color: '#ef4444', fontSize: '0.82rem' }}>
              ⚠ Your selection covers <strong>{Math.abs(covGap)} fewer required positions</strong> than the recommendation. This override is recorded in decision memory.
            </span>
          )}
          {materialCovGap && covGap > 0 && (
            <span style={{ color: '#34d399', fontSize: '0.82rem' }}>
              ✓ Your selection covers <strong>{covGap} more required positions</strong> than the recommendation.
            </span>
          )}
          <span style={{ fontSize: '0.78rem', color: 'var(--text-muted)' }}>Saved to decision memory.</span>
        </div>
      ) : (
        <span>
          <strong style={{ color: '#34d399' }}>Recommendation accepted.</strong>{' '}
          You selected <strong>{sel.label}</strong> ({sel.metrics.filled_positions}/{sel.metrics.required_positions} positions filled), which matches UltraRoster's recommendation.
          Saved to decision memory.
        </span>
      )}
    </div>
  );
};

// ── Main SelectDecision component ─────────────────────────────────────────────

export interface SelectDecisionProps {
  alternatives: RosterAlternative[];
  recommendedId: string;
  onDecision: (selectedAlternative: RosterAlternative, decision: SchedulerDecision) => void;
  onBack: () => void;
}

export const SelectDecision: React.FC<SelectDecisionProps> = ({
  alternatives,
  recommendedId,
  onDecision,
  onBack,
}) => {
  const [selectedId, setSelectedId] = useState<string>(recommendedId);
  const [committedDecision, setCommittedDecision] = useState<SchedulerDecision | null>(null);
  const [showComparison, setShowComparison] = useState(false);

  const selectedAlt = useMemo(
    () => alternatives.find(a => a.id === selectedId) ?? alternatives[0],
    [alternatives, selectedId]
  );

  const handleSelect = (id: string) => {
    setSelectedId(id);
    setCommittedDecision(null); // reset if they change selection
  };

  const handleCommit = () => {
    if (!selectedAlt) return;
    const recommendedAlt = alternatives.find(a => a.id === recommendedId) ?? selectedAlt;
    const decision = repo.recordSchedulerDecision(
      recommendedId,
      selectedId,
      recommendedAlt.metrics,
      selectedAlt.metrics,
    );
    setCommittedDecision(decision);
  };

  const handleProceed = () => {
    if (!selectedAlt || !committedDecision) return;
    onDecision(selectedAlt, committedDecision);
  };

  const onlyOneOption = alternatives.length === 1;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>

      {/* Header */}
      <div>
        <h3 style={{ margin: '0 0 0.4rem 0', color: 'var(--text-main)' }}>Explore the Decision</h3>
        <p style={{ margin: 0, color: 'var(--text-muted)', fontSize: '0.9rem' }}>
          Review the available options, compare trade-offs, and select the roster you want to proceed with.
          UltraRoster does not decide for you — your choice is recorded in decision memory.
        </p>
      </div>

      {/* Honest single-alternative notice */}
      {onlyOneOption && (
        <div style={{
          background: 'rgba(129,140,248,0.06)',
          border: '1px solid rgba(129,140,248,0.25)',
          borderRadius: 8,
          padding: '0.65rem 1rem',
          fontSize: '0.83rem',
          color: '#818cf8',
        }}>
          <strong>Only one meaningfully differentiated option was found.</strong>{' '}
          The current engine produces a single Pareto-optimal solution for this team size and rule set.
          Diversity improvement is a separate future work item (P1.1).
        </div>
      )}

      {/* Option cards */}
      <div style={{ display: 'flex', gap: '1rem', flexWrap: 'wrap' }}>
        {alternatives.map(alt => (
          <AlternativeCard
            key={alt.id}
            alt={alt}
            isRecommended={alt.id === recommendedId}
            isSelected={alt.id === selectedId}
            onSelect={() => handleSelect(alt.id)}
          />
        ))}
      </div>

      {/* Comparison toggle */}
      {alternatives.length > 1 && (
        <div>
          <button
            onClick={() => setShowComparison(v => !v)}
            style={{ ...ghostBtnStyle, fontSize: '0.83rem', padding: '0.35rem 0.85rem' }}
          >
            {showComparison ? '▲ Hide comparison table' : '▼ Compare trade-offs'}
          </button>
          {showComparison && (
            <div style={{ marginTop: '0.75rem', border: '1px solid var(--border-color)', borderRadius: 8, overflow: 'hidden' }}>
              <ComparisonTable
                alternatives={alternatives}
                recommendedId={recommendedId}
                selectedId={selectedId}
              />
            </div>
          )}
        </div>
      )}

      {/* Commit decision */}
      {!committedDecision && (
        <div style={{
          background: 'var(--panel-bg)',
          border: '1px solid var(--border-color)',
          borderRadius: 8,
          padding: '0.85rem 1rem',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          gap: '1rem',
          flexWrap: 'wrap',
        }}>
          <div style={{ fontSize: '0.87rem', color: 'var(--text-muted)' }}>
            Selected: <strong style={{ color: 'var(--text-main)' }}>{selectedAlt?.label ?? '—'}</strong>
            {selectedId !== recommendedId && (
              <span style={{ marginLeft: '0.5rem', color: '#f59e0b', fontSize: '0.8rem' }}>
                (overrides recommendation)
              </span>
            )}
          </div>
          <button onClick={handleCommit} style={{ ...primaryBtnStyle, fontSize: '0.87rem' }}>
            Confirm this decision →
          </button>
        </div>
      )}

      {/* Decision banner */}
      {committedDecision && (
        <DecisionBanner decision={committedDecision} alternatives={alternatives} />
      )}

      {/* Navigation */}
      <div style={{ display: 'flex', justifyContent: 'space-between' }}>
        <button onClick={onBack} style={ghostBtnStyle}>← Back</button>
        <button
          onClick={handleProceed}
          disabled={!committedDecision}
          style={{ ...primaryBtnStyle, opacity: committedDecision ? 1 : 0.4, cursor: committedDecision ? 'pointer' : 'default' }}
        >
          Next: Review & Edit →
        </button>
      </div>
    </div>
  );
};