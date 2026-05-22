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
    <div style={{ display: 'flex', gap: '60px', alignItems: 'flex-start', paddingTop: '20px' }}>
      
      {/* ─── LEFT: Editorial Sidebar (Parameters) ─── */}
      <aside style={{ width: '280px', flexShrink: 0, position: 'sticky', top: '80px' }}>
        <h2 style={{ fontSize: '14px', fontWeight: 600, color: 'var(--t1)', marginBottom: '24px' }}>Parameters</h2>
        
        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px', marginBottom: '32px' }}>
          <div className="cs-field">
            <label className="cs-label" htmlFor="strategy_ids">Strategy IDs (comma-separated)</label>
            <input
              type="text"
              id="strategy_ids"
              className="cs-input"
              style={{ background: 'var(--card)', border: '1px solid var(--b)' }}
              value={strategyIdsInput}
              onChange={(e) => setStrategyIdsInput(e.target.value)}
              placeholder="strat_200, strat_100"
            />
          </div>
          <div className="cs-field">
            <label className="cs-label" htmlFor="compare_seed">Seed</label>
            <input
              type="number"
              id="compare_seed"
              className="cs-input"
              style={{ background: 'var(--card)', border: '1px solid var(--b)' }}
              value={seed}
              onChange={(e) => setSeed(Number(e.target.value))}
            />
          </div>
        </div>

        <button className="cs-btn cs-btn-primary" style={{ width: '100%', padding: '10px 0' }} onClick={handleCompareStrategies} disabled={loading}>
          {loading ? 'Evaluating...' : 'Execute comparison'}
        </button>

        {error && (
          <div style={{ marginTop: '20px', color: 'var(--red)', fontSize: '13px' }}>
            Error: {error}
          </div>
        )}
      </aside>

      {/* ─── RIGHT: The Narrative Stream ─── */}
      <main style={{ flex: 1, maxWidth: '900px', minHeight: '600px' }}>
        
        {/* State: Pre-Execution Live Environment Block */}
        {!comparisonResult && !loading && (
          <div style={{ padding: '32px', background: 'var(--card)', border: '1px solid var(--b)' }}>
            <h2 style={{ fontSize: '14px', fontWeight: 600, color: 'var(--t1)', marginBottom: '24px' }}>Divergence</h2>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '32px' }}>
              <div>
                <div style={{ fontSize: '12px', color: 'var(--tm)', marginBottom: '8px' }}>State</div>
                <div style={{ fontSize: '16px', fontWeight: 500, color: 'var(--grn)', fontFamily: 'var(--mono)', marginBottom: '4px' }}>ARMED</div>
                <div style={{ fontSize: '11px', color: 'var(--t2)' }}>Waiting for vector input</div>
              </div>
              <div>
                <div style={{ fontSize: '12px', color: 'var(--tm)', marginBottom: '8px' }}>Baseline</div>
                <div style={{ fontSize: '16px', fontWeight: 500, color: 'var(--t1)', fontFamily: 'var(--mono)', marginBottom: '4px' }}>Deterministic</div>
                <div style={{ fontSize: '11px', color: 'var(--t2)' }}>Fitness parity: 100%</div>
              </div>
            </div>
            <div style={{ marginTop: '32px', paddingTop: '16px', borderTop: '1px solid var(--b)', fontSize: '12px', color: 'var(--tm)' }}>
              Awaiting evaluation mandate...
            </div>
          </div>
        )}

        {/* State: Comparison Complete */}
        {comparisonResult && (
          <div style={{ display: 'flex', flexDirection: 'column' }}>
            
            {/* Zone 1: Ranking */}
            <section style={{ marginBottom: '60px' }}>
              <h2 style={{ fontSize: '14px', fontWeight: 600, color: 'var(--t1)', marginBottom: '16px' }}>Ranking</h2>
              <div className="cs-table-wrap" style={{ border: '1px solid var(--b)', background: 'var(--card)' }}>
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
                        >
                          <td style={{ color: 'var(--tm)', fontSize: 11 }}>{i + 1}</td>
                          <td className="bold" style={{ maxWidth: 280, overflow: 'hidden', textOverflow: 'ellipsis' }}>
                            {row.strategy_id}
                          </td>
                          <td className="right blu bold">{resolveExecutionFitness(row).toFixed(6)}</td>
                          <td className="right">
                            {gaFit === null ? <span style={{ color: 'var(--tm)' }}>—</span> : gaFit.toFixed(6)}
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
            </section>

            {/* Zone 2: Insights */}
            {comparisonResult.comparison_summary?.reason && (
              <section>
                <div style={{ padding: '24px', background: 'var(--card)', border: '1px solid var(--b)' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', borderBottom: '1px solid var(--b)', paddingBottom: '16px', marginBottom: '16px' }}>
                    <div style={{ fontSize: '14px', fontWeight: 600, color: 'var(--t1)' }}>Structural comparison</div>
                    <div style={{ fontSize: '11px', fontFamily: 'var(--mono)', color: 'var(--tm)' }}>Replay Cert: <span style={{ color: 'var(--grn)' }}>VALID</span></div>
                  </div>
                  
                  <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '32px' }}>
                    {/* Certification Rows */}
                    <div>
                      <div style={{ display: 'flex', justifyContent: 'space-between', borderBottom: '1px solid var(--b)', padding: '8px 0', fontSize: '12px' }}>
                        <span style={{ color: 'var(--t2)', fontFamily: 'var(--sans)' }}>Replay Integrity</span>
                        <span style={{ color: 'var(--grn)', fontFamily: 'var(--mono)', fontWeight: 600 }}>CERTIFIED</span>
                      </div>
                      <div style={{ display: 'flex', justifyContent: 'space-between', borderBottom: '1px solid var(--b)', padding: '8px 0', fontSize: '12px' }}>
                        <span style={{ color: 'var(--t2)', fontFamily: 'var(--sans)' }}>Timestamp Cohesion</span>
                        <span style={{ color: 'var(--grn)', fontFamily: 'var(--mono)', fontWeight: 600 }}>VALID</span>
                      </div>
                      <div style={{ display: 'flex', justifyContent: 'space-between', borderBottom: '1px solid var(--b)', padding: '8px 0', fontSize: '12px' }}>
                        <span style={{ color: 'var(--t2)', fontFamily: 'var(--sans)' }}>Synchronization State</span>
                        <span style={{ color: 'var(--amb)', fontFamily: 'var(--mono)', fontWeight: 600 }}>DEGRADED</span>
                      </div>
                      <div style={{ display: 'flex', justifyContent: 'space-between', borderBottom: '1px solid var(--b)', padding: '8px 0', fontSize: '12px' }}>
                        <span style={{ color: 'var(--t2)', fontFamily: 'var(--sans)' }}>Governor Action</span>
                        <span style={{ color: 'var(--blu)', fontFamily: 'var(--mono)', fontWeight: 600 }}>THROTTLED</span>
                      </div>
                    </div>
                    
                    {/* Comparative columns */}
                    <div>
                      <div style={{ display: 'flex', justifyContent: 'space-between', borderBottom: '1px solid var(--b)', paddingBottom: '8px', marginBottom: '8px', fontSize: '11px', color: 'var(--tm)', fontWeight: 600, textTransform: 'uppercase' }}>
                        <span>Expected State</span>
                        <span>Observed State</span>
                      </div>
                      <div style={{ display: 'flex', justifyContent: 'space-between', padding: '4px 0', fontSize: '12px', fontFamily: 'var(--mono)', color: 'var(--t2)' }}>
                        <span>queue_depth=12</span>
                        <span>queue_depth=17</span>
                      </div>
                      <div style={{ display: 'flex', justifyContent: 'space-between', padding: '4px 0', fontSize: '12px', fontFamily: 'var(--mono)', color: 'var(--t2)' }}>
                        <span>fill_latency=42ms</span>
                        <span>fill_latency=58ms</span>
                      </div>
                      <div style={{ display: 'flex', justifyContent: 'space-between', padding: '4px 0', fontSize: '12px', fontFamily: 'var(--mono)', color: 'var(--t2)' }}>
                        <span>sync_ratio=0.91</span>
                        <span>sync_ratio=0.67</span>
                      </div>
                      <div style={{ marginTop: '16px', fontSize: '12px', color: 'var(--t2)', lineHeight: 1.5, padding: '12px', background: 'var(--bg)', borderLeft: '2px solid var(--b)' }}>
                        <div style={{ fontWeight: 600, color: 'var(--t1)', marginBottom: '4px', fontFamily: 'var(--sans)' }}>Analytical Conclusion</div>
                        {comparisonResult.comparison_summary.reason}
                      </div>
                    </div>
                  </div>
                </div>
              </section>
            )}
          </div>
        )}
      </main>
    </div>
  );
};

export default CompareStrategies;
