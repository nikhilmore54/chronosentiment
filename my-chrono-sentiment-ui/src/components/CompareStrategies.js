import React, { useState } from 'react';
import { parseStrategyParamsFromId } from '../utils/strategyId';

const CompareStrategies = ({ setSelectedStrategyForInspection }) => {
  const [strategyIdsInput, setStrategyIdsInput] = useState('');
  const [seed, setSeed] = useState(42);
  const [comparisonResult, setComparisonResult] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  const resolveExecutionFitness = (row) => {
    if (!row) return 0;
    if (typeof row.execution_fitness === 'number') return row.execution_fitness;
    if (typeof row.fitness === 'number') return row.fitness;
    if (typeof row.score === 'number') return row.score;
    if (typeof row.final_fitness === 'number') return row.final_fitness;
    return 0;
  };

  const resolveGaFitness = (row) => {
    if (!row) return null;
    if (typeof row.ga_fitness === 'number') return row.ga_fitness;
    return null;
  };

  const handleCompareStrategies = async () => {
    setLoading(true);
    setError(null);
    setComparisonResult(null);

    const strategy_ids = strategyIdsInput.split(',').map(id => id.trim()).filter(id => id !== '');

    if (strategy_ids.length < 2) {
      setError('Enter at least two strategy IDs separated by commas.');
      setLoading(false);
      return;
    }

    let strategiesPayload;
    try {
      strategiesPayload = strategy_ids.map((id) => ({
        strategy_config: parseStrategyParamsFromId(id),
      }));
    } catch (parseErr) {
      setError(parseErr.message);
      setLoading(false);
      return;
    }

    try {
      const response = await fetch('http://localhost:8000/compare_strategies', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ strategies: strategiesPayload, scenarios: [], seed: Number(seed) }),
      });

      if (!response.ok) {
        const text = await response.text();
        let msg = 'Failed to compare strategies';
        try { const d = JSON.parse(text); msg = d.message || d.error || msg; } catch { if (text) msg = text; }
        throw new Error(msg);
      }

      const data = await response.json();
      setComparisonResult(data);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="cs-gap-20">
      {/* Header */}
      <div>
        <div className="cs-section-sub">Strategy Evaluation</div>
        <div className="cs-section-title">Compare Strategies</div>
      </div>

      {/* Input form */}
      <div className="cs-card">
        <div className="cs-card-title">Parameters</div>
        <div className="cs-form-grid">
          <div className="cs-field" style={{ gridColumn: '1 / -1' }}>
            <label className="cs-label" htmlFor="strategy_ids">Strategy IDs (comma-separated)</label>
            <input
              type="text"
              id="strategy_ids"
              className="cs-input"
              value={strategyIdsInput}
              onChange={(e) => setStrategyIdsInput(e.target.value)}
              placeholder="strat_200_300_400_500_42, strat_100_200_300_400_42"
            />
          </div>
          <div className="cs-field">
            <label className="cs-label" htmlFor="compare_seed">Seed</label>
            <input
              type="number"
              id="compare_seed"
              className="cs-input"
              value={seed}
              onChange={(e) => setSeed(Number(e.target.value))}
            />
          </div>
        </div>
        <button
          className="cs-btn cs-btn-primary"
          onClick={handleCompareStrategies}
          disabled={loading}
        >
          {loading ? 'Comparing…' : 'Compare Strategies'}
        </button>
      </div>

      {error && (
        <div className="cs-alert red">
          <div className="cs-alert-title">Error</div>
          <div className="cs-alert-body">{error}</div>
        </div>
      )}

      {comparisonResult && (
        <div className="cs-gap-16">
          {/* Ranking table */}
          <div className="cs-card" style={{ padding: 0 }}>
            <div style={{ padding: '14px 20px 0', borderBottom: '1px solid var(--b)' }}>
              <div className="cs-card-title" style={{ marginBottom: 12 }}>Ranking</div>
            </div>
            <div className="cs-table-wrap" style={{ border: 'none', borderRadius: '0 0 var(--r10) var(--r10)' }}>
              <table className="cs-table">
                <thead>
                  <tr>
                    <th style={{ width: 36 }}>#</th>
                    <th>Strategy ID</th>
                    <th className="right">Exec Fitness</th>
                    <th className="right">GA Fitness</th>
                    {setSelectedStrategyForInspection && <th></th>}
                  </tr>
                </thead>
                <tbody>
                  {comparisonResult.ranking.map((row, i) => {
                    const gaFit = resolveGaFitness(row);
                    return (
                      <tr
                        key={row.strategy_id}
                        className={setSelectedStrategyForInspection ? 'clickable' : ''}
                        onClick={() => setSelectedStrategyForInspection && setSelectedStrategyForInspection(row.strategy_id, seed)}
                        title={setSelectedStrategyForInspection ? 'Open in Inspect Strategy' : undefined}
                      >
                        <td style={{ color: 'var(--tm)', fontSize: 11 }}>{i + 1}</td>
                        <td className="bold" style={{ maxWidth: 280, overflow: 'hidden', textOverflow: 'ellipsis' }}>
                          {row.strategy_id}
                        </td>
                        <td className="right blu bold">{resolveExecutionFitness(row).toFixed(6)}</td>
                        <td className="right">
                          {gaFit === null
                            ? <span style={{ color: 'var(--tm)' }}>—</span>
                            : gaFit.toFixed(6)}
                        </td>
                        {setSelectedStrategyForInspection && (
                          <td style={{ color: 'var(--tm)', fontSize: 11 }}>Inspect →</td>
                        )}
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          </div>

          {/* Comparison insights */}
          {comparisonResult.comparison_summary?.reason && (
            <div className="cs-alert blu">
              <div className="cs-alert-title">Comparison Insights</div>
              <div className="cs-alert-body">{comparisonResult.comparison_summary.reason}</div>
            </div>
          )}
        </div>
      )}
    </div>
  );
};

export default CompareStrategies;
