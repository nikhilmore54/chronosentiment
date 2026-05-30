import React, { useState } from 'react';
import { parseStrategyParamsFromId } from '../utils/strategyId';
import { apiUrl } from '../services/api';

const CompareStrategies = ({ setSelectedStrategyForInspection }) => {
  const [strategyIdsInput, setStrategyIdsInput] = useState('');
  const [seed, setSeed] = useState(42);
  const [comparisonResult, setComparisonResult] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  // ARTIFACT-001 REMOVED: resolveExecutionFitness() fallback cascade eliminated.
  // Backend guarantees execution_fitness is always present in StrategyEvaluationDto.
  // Direct field access: row.execution_fitness

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
      const response = await fetch(apiUrl('/compare_strategies'), {
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
          <div style={{ marginTop: '16px', padding: '12px', background: 'var(--rdim)', border: '1px solid var(--bred)', borderRadius: 'var(--r8)', color: 'var(--red)', fontSize: '12px', fontFamily: 'var(--mono)' }}>
            {error}
          </div>
        )}
      </aside>

      {/* ─── RIGHT: The Narrative Stream ─── */}
      <main style={{ flex: 1, maxWidth: '900px', minHeight: '600px' }}>

        {/* State: Pre-Execution idle block */}
        {!comparisonResult && !loading && !error && (
          <div className="cs-empty" style={{ padding: '48px 32px', background: 'var(--card)', border: '1px solid var(--b)', borderRadius: 'var(--r10)', textAlign: 'center' }}>
            <div className="cs-empty-icon">⚖</div>
            <div className="cs-empty-title">No comparison loaded</div>
            <div style={{ fontSize: '12px', color: 'var(--tm)', marginTop: '8px', lineHeight: 1.6 }}>
              Enter two or more strategy IDs separated by commas and press <strong style={{ color: 'var(--t2)' }}>Execute comparison</strong>.
            </div>
          </div>
        )}

        {/* State: Loading */}
        {loading && (
          <div style={{ padding: '32px', background: 'var(--card)', border: '1px solid var(--b)' }}>
            <div style={{ marginBottom: '24px' }}>
              <div className="cs-skeleton" style={{ height: '14px', width: '120px', marginBottom: '16px' }}></div>
              <div className="cs-skeleton" style={{ height: '40px', marginBottom: '8px' }}></div>
              <div className="cs-skeleton" style={{ height: '40px', marginBottom: '8px' }}></div>
              <div className="cs-skeleton" style={{ height: '40px' }}></div>
            </div>
            <div style={{ fontSize: '12px', color: 'var(--tm)', fontFamily: 'var(--mono)' }}>Evaluating strategies...</div>
          </div>
        )}

        {/* State: Comparison Complete */}
        {comparisonResult && !loading && (
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
                    {Array.isArray(comparisonResult.ranking) && comparisonResult.ranking.length > 0 ? (
                      comparisonResult.ranking.map((row, i) => {
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
                            <td className="right blu bold">{(row.execution_fitness ?? 0).toFixed(6)}</td>
                            <td className="right">
                              {gaFit === null ? <span style={{ color: 'var(--tm)' }}>—</span> : gaFit.toFixed(6)}
                            </td>
                            {setSelectedStrategyForInspection && (
                              <td style={{ color: 'var(--tm)', fontSize: 11 }}>Inspect →</td>
                            )}
                          </tr>
                        );
                      })
                    ) : (
                      <tr>
                        <td colSpan={setSelectedStrategyForInspection ? 5 : 4} style={{ textAlign: 'center', padding: '40px 0', color: 'var(--tm)', fontSize: '13px' }}>
                          No ranking data returned.
                        </td>
                      </tr>
                    )}
                  </tbody>
                </table>
              </div>
            </section>

            {/* Zone 2: Structural Comparison */}
            {comparisonResult.comparison_summary && (
              <section>
                <div style={{ padding: '24px', background: 'var(--card)', border: '1px solid var(--b)' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', borderBottom: '1px solid var(--b)', paddingBottom: '16px', marginBottom: '16px' }}>
                    <div style={{ fontSize: '14px', fontWeight: 600, color: 'var(--t1)' }}>Structural comparison</div>
                    {comparisonResult.comparison_summary.replay_certified !== undefined && (
                      <div style={{ fontSize: '11px', fontFamily: 'var(--mono)', color: 'var(--tm)' }}>
                        Replay Cert:{' '}
                        <span style={{ color: comparisonResult.comparison_summary.replay_certified ? 'var(--grn)' : 'var(--red)' }}>
                          {comparisonResult.comparison_summary.replay_certified ? 'VALID' : 'INVALID'}
                        </span>
                      </div>
                    )}
                  </div>

                  {/* Certification rows — driven by API response fields */}
                  {(() => {
                    const s = comparisonResult.comparison_summary;
                    const certRows = [
                      s.replay_integrity   !== undefined && { label: 'Replay Integrity',     value: s.replay_integrity,   color: s.replay_integrity   === 'CERTIFIED' ? 'var(--grn)' : 'var(--amb)' },
                      s.timestamp_cohesion !== undefined && { label: 'Timestamp Cohesion',   value: s.timestamp_cohesion, color: s.timestamp_cohesion === 'VALID'     ? 'var(--grn)' : 'var(--amb)' },
                      s.sync_state         !== undefined && { label: 'Synchronization State',value: s.sync_state,         color: s.sync_state         === 'NOMINAL'   ? 'var(--grn)' : s.sync_state === 'DEGRADED' ? 'var(--amb)' : 'var(--red)' },
                      s.governor_action    !== undefined && { label: 'Governor Action',       value: s.governor_action,    color: 'var(--blu)' },
                    ].filter(Boolean);

                    const metricRows = Array.isArray(s.metrics) ? s.metrics : [];

                    return (
                      <div style={{ display: 'grid', gridTemplateColumns: certRows.length > 0 ? '1fr 1fr' : '1fr', gap: '32px' }}>
                        {certRows.length > 0 && (
                          <div>
                            {certRows.map(row => (
                              <div key={row.label} style={{ display: 'flex', justifyContent: 'space-between', borderBottom: '1px solid var(--b)', padding: '8px 0', fontSize: '12px' }}>
                                <span style={{ color: 'var(--t2)', fontFamily: 'var(--sans)' }}>{row.label}</span>
                                <span style={{ color: row.color, fontFamily: 'var(--mono)', fontWeight: 600 }}>{row.value}</span>
                              </div>
                            ))}
                          </div>
                        )}

                        <div>
                          {metricRows.length > 0 && (
                            <>
                              <div style={{ display: 'flex', justifyContent: 'space-between', borderBottom: '1px solid var(--b)', paddingBottom: '8px', marginBottom: '8px', fontSize: '11px', color: 'var(--tm)', fontWeight: 600, textTransform: 'uppercase' }}>
                                <span>Expected</span>
                                <span>Observed</span>
                              </div>
                              {metricRows.map((m, i) => (
                                <div key={i} style={{ display: 'flex', justifyContent: 'space-between', padding: '4px 0', fontSize: '12px', fontFamily: 'var(--mono)', color: 'var(--t2)' }}>
                                  <span>{m.key}={m.expected}</span>
                                  <span style={{ color: m.diverged ? 'var(--amb)' : 'var(--t2)' }}>{m.key}={m.observed}</span>
                                </div>
                              ))}
                            </>
                          )}

                          {s.reason && (
                            <div style={{ marginTop: metricRows.length > 0 ? '16px' : '0', fontSize: '12px', color: 'var(--t2)', lineHeight: 1.5, padding: '12px', background: 'var(--bg)', borderLeft: '2px solid var(--b)' }}>
                              <div style={{ fontWeight: 600, color: 'var(--t1)', marginBottom: '4px', fontFamily: 'var(--sans)' }}>Analytical Conclusion</div>
                              {s.reason}
                            </div>
                          )}

                          {!s.reason && metricRows.length === 0 && certRows.length === 0 && (
                            <div className="cs-empty">
                              <div className="cs-empty-icon">—</div>
                              <div className="cs-empty-title">No structural data</div>
                              <div>Backend did not return comparison metadata.</div>
                            </div>
                          )}
                        </div>
                      </div>
                    );
                  })()}
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
