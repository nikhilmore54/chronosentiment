import React, { useState, useEffect } from 'react';

function safeDisplay(value, digits = 2) {
  if (value === undefined || value === null) return 'N/A';
  return typeof value === 'number' ? value.toFixed(digits) : value;
}

// ARTIFACT-001 REMOVED: resolveExecutionFitness() fallback cascade eliminated.
// Backend guarantees execution_fitness is always present in StrategyEvaluationDto.
// Direct field access: row.execution_fitness

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
    (a, b) => (b.execution_fitness ?? 0) - (a.execution_fitness ?? 0)
  );

  return (
    <div style={{ display: 'flex', gap: '60px', alignItems: 'flex-start', paddingTop: '20px' }}>
      
      {/* ─── LEFT: Editorial Sidebar (Controls) ─── */}
      <aside style={{ width: '280px', flexShrink: 0, position: 'sticky', top: '80px' }}>
        <h2 style={{ fontSize: '14px', fontWeight: 600, color: 'var(--t1)', marginBottom: '24px' }}>Global ranking</h2>
        
        <div style={{ marginBottom: '32px', fontSize: '13px', color: 'var(--t2)', lineHeight: 1.5 }}>
          The multi-asset execution table continuously aggregates and standardizes PnL, execution fitness, and survivability variance across all generated strategies.
        </div>

        <button className="cs-btn cs-btn-primary" style={{ width: '100%', padding: '10px 0' }} onClick={fetchRanking} disabled={loading}>
          {loading ? 'Refreshing sequence...' : 'Fetch execution ranking'}
        </button>

        {error && (
          <div style={{ marginTop: '20px', color: 'var(--red)', fontSize: '13px' }}>
            Fetch Error: {error}
          </div>
        )}
      </aside>

      {/* ─── RIGHT: The Structural Space ─── */}
      <main style={{ flex: 1, maxWidth: '900px', minHeight: '600px' }}>
        
        {/* Table UI */}
        <div className="cs-table-wrap" style={{ border: '1px solid var(--b)', borderRadius: 'var(--r8)', background: 'var(--card)' }}>
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
                  const execFit = row.execution_fitness;
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
                  <td colSpan={7} style={{ textAlign: 'center', padding: '64px 0', color: 'var(--tm)', fontSize: '13px' }}>
                    {loading ? 'Synchronizing global state...' : 'No ranking data materialized. Run multi-asset evaluation first.'}
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </main>
    </div>
  );
};

export default GlobalRanking;
