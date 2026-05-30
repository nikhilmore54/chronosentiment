import React from 'react';

const ComparisonPanels = ({
  isDualMode,
  allNarrativeBlocks1,
  allNarrativeBlocks2,
  strategyId,
  strategyId2,
  executionSummary1,
  executionSummary2,
  finalVerdict,
  confidenceLevel,
  confidenceColorClass,
  confidenceReason,
  divergenceStatements,
}) => {
  const hasData = isDualMode && (allNarrativeBlocks1?.length > 0 || allNarrativeBlocks2?.length > 0);
  if (!hasData) return null;

  // confidenceColorClass is already a cs- modifier: 'grn' | 'amb' | 'red'
  const verdictMod = confidenceColorClass || 'red';

  // Build insights list imperatively to avoid nested conditionals in JSX
  const insights = [];
  if (executionSummary1.totalSteps < executionSummary2.totalSteps)
    insights.push('Strategy 1 executed faster (fewer steps).');
  else if (executionSummary2.totalSteps < executionSummary1.totalSteps)
    insights.push('Strategy 2 executed faster (fewer steps).');
  else if (executionSummary1.totalSteps > 0)
    insights.push('Both strategies executed in the same number of steps.');

  if (executionSummary1.hasQueueProgression && !executionSummary2.hasQueueProgression)
    insights.push('Strategy 1 experienced queue delay; Strategy 2 did not.');
  else if (!executionSummary1.hasQueueProgression && executionSummary2.hasQueueProgression)
    insights.push('Strategy 2 experienced queue delay; Strategy 1 did not.');
  else if (executionSummary1.hasQueueProgression && executionSummary2.hasQueueProgression)
    insights.push('Both strategies experienced queue delays.');
  else
    insights.push('Neither strategy experienced significant queue delays.');

  if (executionSummary1.partialFills > executionSummary2.partialFills)
    insights.push('Strategy 1 had more fragmented execution (more partial fills).');
  else if (executionSummary2.partialFills > executionSummary1.partialFills)
    insights.push('Strategy 2 had more fragmented execution (more partial fills).');
  else if (executionSummary1.partialFills === 0)
    insights.push('Neither strategy experienced partial fills.');
  else
    insights.push('Both strategies experienced the same number of partial fills.');

  return (
    <div className="cs-gap-16">

      {/* ── Execution Summary Comparison ─────────────────────────────────── */}
      <div className="cs-card">
        <div className="cs-card-title">Execution Summary Comparison</div>
        <div className="cs-dual-grid">
          <div>
            <div className="cs-section-sub">Strategy 1 — {strategyId || 'N/A'}</div>
            <div className="cs-row">
              <span className="cs-row-key">Steps</span>
              <span className="cs-row-val">{executionSummary1.totalSteps}</span>
            </div>
            <div className="cs-row">
              <span className="cs-row-key">Partial Fills</span>
              <span className="cs-row-val">{executionSummary1.partialFills}</span>
            </div>
            <div className="cs-row">
              <span className="cs-row-key">Queue Progression</span>
              <span className={`cs-row-val ${executionSummary1.hasQueueProgression ? 'amb' : 'grn'}`}>
                {executionSummary1.hasQueueProgression ? 'Yes' : 'No'}
              </span>
            </div>
            <div className="cs-row">
              <span className="cs-row-key">Full Fills</span>
              <span className="cs-row-val">{executionSummary1.totalFills}</span>
            </div>
          </div>
          <div>
            <div className="cs-section-sub">Strategy 2 — {strategyId2 || 'N/A'}</div>
            <div className="cs-row">
              <span className="cs-row-key">Steps</span>
              <span className="cs-row-val">{executionSummary2.totalSteps}</span>
            </div>
            <div className="cs-row">
              <span className="cs-row-key">Partial Fills</span>
              <span className="cs-row-val">{executionSummary2.partialFills}</span>
            </div>
            <div className="cs-row">
              <span className="cs-row-key">Queue Progression</span>
              <span className={`cs-row-val ${executionSummary2.hasQueueProgression ? 'amb' : 'grn'}`}>
                {executionSummary2.hasQueueProgression ? 'Yes' : 'No'}
              </span>
            </div>
            <div className="cs-row">
              <span className="cs-row-key">Full Fills</span>
              <span className="cs-row-val">{executionSummary2.totalFills}</span>
            </div>
          </div>
        </div>
      </div>

      {/* ── Execution Insights ────────────────────────────────────────────── */}
      <div className="cs-card">
        <div className="cs-card-title">Execution Insights</div>
        <div className="cs-gap-4">
          {insights.map((insight, i) => (
            <div key={i} className="cs-row">
              <span className="cs-row-key">{insight}</span>
            </div>
          ))}
        </div>
      </div>

      {/* ── Final Execution Verdict ───────────────────────────────────────── */}
      <div className={`cs-alert ${verdictMod}`}>
        <div className="cs-alert-title">{finalVerdict}</div>
        <div className={`cs-row-val ${verdictMod}`} style={{ fontSize: '11px', marginBottom: '4px' }}>
          Confidence: {confidenceLevel}
        </div>
        <div className="cs-alert-body">{confidenceReason}</div>
      </div>

      {/* ── ARTIFACT-010: Observational trace comparison (see AUTHORITY_MAP.md) ─ */}
      <div className="cs-card">
        <div className="cs-card-title">Observational Trace Comparison</div>
        <div style={{ fontSize: '10px', color: 'var(--tm)', marginBottom: '12px', lineHeight: 1.5 }}>
          Client-derived from two certified traces. Comparison aid — not certified divergence authority.
        </div>
        <div className="cs-gap-8">
          {divergenceStatements.length > 0 ? (
            divergenceStatements.map((d, i) => (
              <div key={i} className="cs-alert red" style={{ marginBottom: 0 }}>
                <div className="cs-alert-body">{d.message}</div>
              </div>
            ))
          ) : (
            <div className="cs-alert grn" style={{ marginBottom: 0 }}>
              <div className="cs-alert-body">No observational divergences detected in visible events.</div>
            </div>
          )}
        </div>
      </div>

    </div>
  );
};

export default ComparisonPanels;