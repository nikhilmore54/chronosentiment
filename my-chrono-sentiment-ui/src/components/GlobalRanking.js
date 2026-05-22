import React, { useState, useEffect } from 'react';

function safeDisplay(value, digits = 2) {
  if (value === undefined || value === null) return 'N/A';
  return typeof value === 'number' ? value.toFixed(digits) : value;
}

function resolveExecutionFitness(row) {
  if (!row) return undefined;
  if (typeof row.execution_fitness === 'number') return row.execution_fitness;
  if (typeof row.fitness === 'number') return row.fitness;
  if (typeof row.score === 'number') return row.score;
  if (typeof row.final_fitness === 'number') return row.final_fitness;
  return undefined;
}

function resolveGaFitness(row) {
  if (!row) return undefined;
  if (typeof row.ga_fitness === 'number') return row.ga_fitness;
  return undefined;
}

function classificationColor(cls) {
  if (!cls) return 'gray';
  const c = cls.toLowerCase();
  if (c === 'stable')   return 'grn';
  if (c === 'volatile') return 'amb';
  if (c === 'fragile')  return 'red';
  return 'gray';
}

const GlobalRanking = () => {
  const [ranking, setRanking] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  const fetchRanking = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetch('http://localhost:8000/ga/global-ranking');
      if (!response.ok) throw new Error('Failed to fetch global ranking');
      const data = await response.json();
      setRanking(data);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { fetchRanking(); }, []);

  const sorted = [...ranking].sort(
    (a, b) => (resolveExecutionFitness(b) ?? 0) - (resolveExecutionFitness(a) ?? 0)
  );

  return (
    <div className="cs-gap-20">
      {/* Header row */}
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-end' }}>
        <div>
          <div className="cs-section-sub">Strategy Evaluation</div>
          <div className="cs-section-title">Global Strategy Ranking</div>
        </div>
        <button
          className="cs-btn cs-btn-success"
          onClick={fetchRanking}
          disabled={loading}
        >
          {loading ? 'Refreshing…' : 'Refresh'}
        </button>
      </div>

      {error && (
        <div className="cs-alert red">
          <div className="cs-alert-title">Fetch Error</div>
          <div className="cs-alert-body">{error}</div>
        </div>
      )}

      <div className="cs-card" style={{ padding: 0 }}>
        <div className="cs-table-wrap" style={{ border: 'none', borderRadius: 'var(--r10)' }}>
          <table className="cs-table">
            <thead>
              <tr>
                <th style={{ width: 36 }}>#</th>
                <th>Strategy</th>
                <th className="right">Avg PnL</th>
                <th className="right">Std Dev</th>
                <th className="right">Exec Fitness</th>
                <th className="right">GA Fitness</th>
                <th>Classification</th>
              </tr>
            </thead>
            <tbody>
              {sorted.length > 0 ? (
                sorted.map((row, index) => {
                  const execFit = resolveExecutionFitness(row);
                  const gaFit   = resolveGaFitness(row);
                  const cls     = classificationColor(row.classification);
                  return (
                    <tr key={row.strategy_id}>
                      <td style={{ color: 'var(--tm)', fontSize: 11 }}>{index + 1}</td>
                      <td className="bold" style={{ maxWidth: 260, overflow: 'hidden', textOverflow: 'ellipsis' }} title={row.strategy_id}>
                        {row.strategy_id}
                      </td>
                      <td className="right">{safeDisplay(row.avg, 6)}</td>
                      <td className="right">{safeDisplay(row.std, 6)}</td>
                      <td className={`right blu bold`}>{safeDisplay(execFit, 6)}</td>
                      <td className="right">
                        {gaFit === undefined ? <span style={{ color: 'var(--tm)' }}>—</span> : safeDisplay(gaFit, 6)}
                      </td>
                      <td>
                        <span className={`cs-badge ${cls}`}>{row.classification ?? '—'}</span>
                      </td>
                    </tr>
                  );
                })
              ) : (
                <tr>
                  <td colSpan={7} style={{ textAlign: 'center', padding: '32px 0', color: 'var(--tm)' }}>
                    {loading ? 'Loading ranking data…' : 'No ranking data available. Run multi-asset evaluation first.'}
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};

export default GlobalRanking;
