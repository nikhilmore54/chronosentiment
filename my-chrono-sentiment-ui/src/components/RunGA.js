import React, { useState, useEffect } from 'react';

function safeDisplay(value, digits = 2) {
  if (value === undefined || value === null || Number.isNaN(value)) return 'N/A';
  return typeof value === 'number' ? value.toFixed(digits) : value;
}

function resolveExecutionFitness(entry) {
  if (!entry) return undefined;
  if (typeof entry.execution_fitness === 'number') return entry.execution_fitness;
  if (typeof entry.fitness === 'number') return entry.fitness;
  if (typeof entry.score === 'number') return entry.score;
  if (typeof entry.final_fitness === 'number') return entry.final_fitness;
  return undefined;
}

function resolveGaFitness(entry) {
  if (!entry) return undefined;
  if (typeof entry.ga_fitness === 'number') return entry.ga_fitness;
  return undefined;
}

function divergenceBadge(entry) {
  const ga   = resolveGaFitness(entry);
  const exec = resolveExecutionFitness(entry);
  if (ga === undefined || exec === undefined) return { label: '—', cls: 'gray' };
  const normalizedGa = Math.max(0, Math.min(1, ga / 100.0));
  const divergence   = exec - normalizedGa;
  if (divergence < -0.2) return { label: 'Overfit',    cls: 'red' };
  if (divergence >  0.2) return { label: 'Hidden Gem', cls: 'grn' };
  return { label: 'Aligned', cls: 'blu' };
}

function findBestByGaFitness(list) {
  if (!Array.isArray(list) || list.length === 0) return { best: undefined, index: null };
  let best = list[0], bestIndex = 0, bestFitness = resolveGaFitness(best) ?? Number.NEGATIVE_INFINITY;
  for (let i = 1; i < list.length; i++) {
    const f = resolveGaFitness(list[i]) ?? Number.NEGATIVE_INFINITY;
    if (f > bestFitness) { best = list[i]; bestIndex = i; bestFitness = f; }
  }
  return { best, index: bestIndex };
}

function signalStrength(s) {
  if (!s || s.action === 'HOLD') return null;
  const cs = typeof s.composite_score === 'number' ? s.composite_score : 0;
  return cs > 1e-9 ? 'STRONG' : 'WEAK';
}

function buildAssetRollups(signals) {
  const byAsset = new Map();
  for (const s of signals ?? []) {
    const a = s.asset ?? 'UNKNOWN';
    if (!byAsset.has(a)) byAsset.set(a, { asset: a, n: 0, traded: 0, confSum: 0, maxConf: Number.NEGATIVE_INFINITY, pnlSum: 0, pnlN: 0 });
    const row = byAsset.get(a);
    row.n += 1;
    if (s.action !== 'HOLD') {
      row.traded += 1;
      const c = typeof s.confidence === 'number' ? s.confidence : 0;
      row.confSum += c;
      row.maxConf = Math.max(row.maxConf, c);
      if (typeof s.scenario_pnl === 'number') { row.pnlSum += s.scenario_pnl; row.pnlN += 1; }
    }
  }
  const out = [];
  for (const row of byAsset.values()) {
    const participation = row.n > 0 ? row.traded / row.n : 0;
    const avgConf = row.traded > 0 ? row.confSum / row.traded : 0;
    const avgPnl  = row.pnlN > 0 ? row.pnlSum / row.pnlN : 0;
    const maxConf = row.maxConf === Number.NEGATIVE_INFINITY ? 0 : row.maxConf;
    const score   = 0.5 * maxConf + 0.3 * participation + 0.2 * Math.max(0, Math.min(1, avgPnl + 0.5));
    out.push({ asset: row.asset, score, participation, avgPnl, maxConf, avgConf, traded: row.traded, scenarios: row.n });
  }
  out.sort((a, b) => b.score - a.score || a.asset.localeCompare(b.asset));
  return out;
}

function topSignalsPerAsset(signals, { topK = 2, includeWeak = true, strongOnly = false } = {}) {
  const active   = (signals ?? []).filter(s => s.action !== 'HOLD');
  const filtered = active.filter(s => {
    const st = signalStrength(s);
    if (strongOnly) return st === 'STRONG';
    if (!includeWeak) return st === 'STRONG';
    return true;
  });
  const byAsset = new Map();
  for (const s of filtered) {
    const a = s.asset ?? 'UNKNOWN';
    if (!byAsset.has(a)) byAsset.set(a, []);
    byAsset.get(a).push(s);
  }
  const result = [];
  for (const [, arr] of byAsset) {
    arr.sort((a, b) => (b.confidence ?? 0) - (a.confidence ?? 0));
    result.push(...arr.slice(0, topK));
  }
  result.sort((a, b) => (b.confidence ?? 0) - (a.confidence ?? 0));
  return result;
}

function normalizeGaResult(raw) {
  const history = Array.isArray(raw?.generation_history) ? raw.generation_history : [];
  const processedHistory = history.map(entry => ({
    ...entry,
    ga_fitness: resolveGaFitness(entry) ?? (entry.fitness ?? 0),
  }));
  const { best: historyBest, index: historyBestIndex } = findBestByGaFitness(processedHistory);
  const globalBest = raw?.global_best ?? historyBest ?? (Array.isArray(raw?.results) && raw.results.length > 0 ? raw.results[0] : undefined);
  const finalGenerationBest = raw?.final_generation_best ?? raw?.final_gen_best
    ?? (processedHistory.length > 0 ? processedHistory[processedHistory.length - 1] : undefined)
    ?? (Array.isArray(raw?.results) && raw.results.length > 0 ? raw.results[0] : undefined);
  const peakGeneration = raw?.global_best_generation ?? raw?.generation_found ?? historyBestIndex;
  return { ...raw, generation_history: processedHistory, global_best: globalBest, final_generation_best: finalGenerationBest, global_best_generation: peakGeneration };
}

const RunGA = ({ setSelectedStrategyForInspection }) => {
  const [populationSize, setPopulationSize] = useState(50);
  const [generations, setGenerations]       = useState(20);
  const [mutationRate, setMutationRate]     = useState(0.1);
  const [seed, setSeed]                     = useState(42);
  const [gaResult, setGaResult]             = useState(null);
  const [loading, setLoading]               = useState(false);
  const [error, setError]                   = useState(null);
  const [signalsSnapshot, setSignalsSnapshot]           = useState(null);
  const [persistedStorePayload, setPersistedStorePayload] = useState(null);
  const [storeLoading, setStoreLoading]     = useState(false);
  const [storeError, setStoreError]         = useState(null);
  const [signalsTopK, setSignalsTopK]       = useState(2);
  const [includeWeakSignals, setIncludeWeakSignals] = useState(true);
  const [strongOnlySignals, setStrongOnlySignals]   = useState(false);

  const resolvePeakGeneration = (result) => {
    const direct = result?.global_best_generation ?? result?.generation_found;
    if (direct !== undefined && direct !== null) return direct;
    const history = result?.generation_history;
    if (!Array.isArray(history) || history.length === 0) return null;
    let bestIdx = 0, bestFitness = Number.NEGATIVE_INFINITY;
    history.forEach((g, idx) => { const s = resolveGaFitness(g) ?? 0; if (s > bestFitness) { bestFitness = s; bestIdx = idx; } });
    return bestIdx;
  };

  const fetchPersistedStrategyStore = async () => {
    setStoreLoading(true);
    setStoreError(null);
    try {
      const response = await fetch('http://localhost:8000/ga/strategy-store');
      if (!response.ok) { const text = await response.text(); throw new Error(text || 'Failed to load persisted strategy store'); }
      const data = await response.json();
      setPersistedStorePayload(data);
    } catch (err) {
      setStoreError(err.message);
      setPersistedStorePayload(null);
    } finally {
      setStoreLoading(false);
    }
  };

  useEffect(() => { fetchPersistedStrategyStore(); }, []);

  const handleRunGA = async () => {
    setLoading(true);
    setError(null);
    setGaResult(null);
    setSignalsSnapshot(null);
    try {
      const response = await fetch('http://localhost:8000/run_ga');
      if (!response.ok) { const errorData = await response.json(); throw new Error(errorData.message || 'Failed to run GA'); }
      const data       = await response.json();
      const normalized = normalizeGaResult(data);
      const signalsResp = await fetch('http://localhost:8000/signals/latest');
      if (!signalsResp.ok) { const errorData = await signalsResp.json(); throw new Error(errorData.message || 'Failed to fetch latest signals'); }
      const signalsData = await signalsResp.json();
      setGaResult(normalized);
      setSignalsSnapshot(signalsData);
      await fetchPersistedStrategyStore();
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
        <div className="cs-section-sub">Genetic Algorithm</div>
        <div className="cs-section-title">Run GA</div>
      </div>

      {/* Strategy store */}
      <details className="cs-details">
        <summary>Persisted Strategy Store</summary>
        <div className="cs-details-body" style={{ paddingTop: 14 }}>
          <p style={{ fontSize: 11, color: 'var(--tm)', marginBottom: 10, lineHeight: 1.6 }}>
            On-disk JSON loaded by the API for signals. The interactive{' '}
            <code className="cs-code">/run_ga</code> endpoint does not overwrite this file.
          </p>
          <div className="cs-row-gap-8" style={{ marginBottom: 10 }}>
            <button className="cs-btn" onClick={fetchPersistedStrategyStore} disabled={storeLoading}>
              {storeLoading ? 'Loading…' : 'Refresh'}
            </button>
            {persistedStorePayload?.path && (
              <span style={{ fontSize: 11, color: 'var(--tm)', fontFamily: 'var(--mono)' }}>
                {persistedStorePayload.path}
              </span>
            )}
          </div>
          {storeError && (
            <div className="cs-alert red" style={{ marginBottom: 0 }}>
              <div className="cs-alert-body">{storeError}</div>
            </div>
          )}
          {persistedStorePayload?.store != null && (
            <pre className="cs-pre">{JSON.stringify(persistedStorePayload.store, null, 2)}</pre>
          )}
          {persistedStorePayload?.store == null && !storeLoading && !storeError && (
            <p style={{ fontSize: 12, color: 'var(--tm)' }}>
              No store on disk. Run the full training pipeline to generate{' '}
              <code className="cs-code">strategy_store.json</code>.
            </p>
          )}
        </div>
      </details>

      {/* Parameters */}
      <div className="cs-card">
        <div className="cs-card-title">Parameters</div>
        <div className="cs-form-grid">
          <div className="cs-field">
            <label className="cs-label" htmlFor="population_size">Population Size</label>
            <input type="number" id="population_size" className="cs-input" value={populationSize} onChange={e => setPopulationSize(e.target.value)} />
          </div>
          <div className="cs-field">
            <label className="cs-label" htmlFor="generations">Generations</label>
            <input type="number" id="generations" className="cs-input" value={generations} onChange={e => setGenerations(e.target.value)} />
          </div>
          <div className="cs-field">
            <label className="cs-label" htmlFor="mutation_rate">Mutation Rate</label>
            <input type="number" id="mutation_rate" step="0.01" className="cs-input" value={mutationRate} onChange={e => setMutationRate(e.target.value)} />
          </div>
          <div className="cs-field">
            <label className="cs-label" htmlFor="seed">Seed</label>
            <input type="number" id="seed" className="cs-input" value={seed} onChange={e => setSeed(e.target.value)} />
          </div>
        </div>
        <button className="cs-btn cs-btn-primary" onClick={handleRunGA} disabled={loading}>
          {loading ? 'Running GA…' : 'Run GA'}
        </button>
      </div>

      {error && (
        <div className="cs-alert red">
          <div className="cs-alert-title">Error</div>
          <div className="cs-alert-body">{error}</div>
        </div>
      )}

      {gaResult && (
        <div className="cs-gap-20">
          {/* Best strategy metrics */}
          <div className="cs-metric-grid">
            <div className="cs-metric info">
              <div className="cs-metric-label">Exec Fitness (Best)</div>
              <div className="cs-metric-value blu">{safeDisplay(resolveExecutionFitness(gaResult.global_best), 6)}</div>
              <div className="cs-metric-sub">Global best</div>
            </div>
            <div className="cs-metric">
              <div className="cs-metric-label">GA Fitness</div>
              <div className="cs-metric-value">{resolveGaFitness(gaResult.global_best) === undefined ? '—' : safeDisplay(resolveGaFitness(gaResult.global_best), 6)}</div>
              <div className="cs-metric-sub">Search score</div>
            </div>
            <div className="cs-metric">
              <div className="cs-metric-label">Avg PnL</div>
              <div className="cs-metric-value">{safeDisplay(gaResult.global_best?.avg, 4)}</div>
              <div className="cs-metric-sub">Best genome</div>
            </div>
            <div className="cs-metric">
              <div className="cs-metric-label">Peak Generation</div>
              <div className="cs-metric-value">{resolvePeakGeneration(gaResult) ?? 'N/A'}</div>
              <div className="cs-metric-sub">Best found at</div>
            </div>
          </div>

          {/* Final generation best */}
          <div className="cs-card">
            <div className="cs-card-title">Final Generation Best</div>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))', gap: 0 }}>
              <div className="cs-row">
                <span className="cs-row-key">Exec Fitness</span>
                <span className="cs-row-val blu">{safeDisplay(resolveExecutionFitness(gaResult.final_generation_best ?? gaResult.final_gen_best), 6)}</span>
              </div>
              <div className="cs-row">
                <span className="cs-row-key">GA Fitness</span>
                <span className="cs-row-val">{resolveGaFitness(gaResult.final_generation_best ?? gaResult.final_gen_best) === undefined ? '—' : safeDisplay(resolveGaFitness(gaResult.final_generation_best ?? gaResult.final_gen_best), 6)}</span>
              </div>
              <div className="cs-row">
                <span className="cs-row-key">Avg PnL</span>
                <span className="cs-row-val">{safeDisplay((gaResult.final_generation_best ?? gaResult.final_gen_best)?.avg, 4)}</span>
              </div>
              <div className="cs-row">
                <span className="cs-row-key">Std Dev</span>
                <span className="cs-row-val">{safeDisplay((gaResult.final_generation_best ?? gaResult.final_gen_best)?.std, 6)}</span>
              </div>
            </div>
          </div>

          {/* Best per regime */}
          {gaResult.best_per_regime && Object.keys(gaResult.best_per_regime).length > 0 && (
            <div className="cs-card" style={{ padding: 0 }}>
              <div style={{ padding: '14px 20px 0' }}>
                <div className="cs-card-title" style={{ marginBottom: 12 }}>Best Per Regime</div>
              </div>
              <div className="cs-table-wrap" style={{ border: 'none', borderRadius: '0 0 var(--r10) var(--r10)' }}>
                <table className="cs-table">
                  <thead>
                    <tr>
                      <th>Regime</th>
                      <th>Strategy ID</th>
                      <th className="right">Exec Fitness</th>
                      <th className="right">GA Fitness</th>
                      <th>Class</th>
                    </tr>
                  </thead>
                  <tbody>
                    {Object.entries(gaResult.best_per_regime).map(([regimeKey, row]) => (
                      <tr key={regimeKey} className="clickable" onClick={() => setSelectedStrategyForInspection(row.strategy_id, seed)}>
                        <td className="bold">{regimeKey}</td>
                        <td>{row.strategy_id}</td>
                        <td className="right blu">{safeDisplay(resolveExecutionFitness(row), 6)}</td>
                        <td className="right">{resolveGaFitness(row) === undefined ? <span style={{ color: 'var(--tm)' }}>—</span> : safeDisplay(resolveGaFitness(row), 6)}</td>
                        <td>{row.classification ?? '—'}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {/* Best strategies table */}
          <div className="cs-card" style={{ padding: 0 }}>
            <div style={{ padding: '14px 20px 0' }}>
              <div className="cs-card-title" style={{ marginBottom: 12 }}>Best Strategies</div>
            </div>
            <div className="cs-table-wrap" style={{ border: 'none', borderRadius: '0 0 var(--r10) var(--r10)' }}>
              <table className="cs-table">
                <thead>
                  <tr>
                    <th>Strategy ID</th>
                    <th className="right">Exec Fitness</th>
                    <th className="right">GA Fitness</th>
                    <th>Divergence</th>
                    <th className="right">Avg PnL</th>
                    <th className="right">Std Dev</th>
                    <th>Class</th>
                  </tr>
                </thead>
                <tbody>
                  {gaResult.results && gaResult.results.map(row => {
                    const div = divergenceBadge(row);
                    return (
                      <tr key={row.strategy_id} className="clickable" onClick={() => setSelectedStrategyForInspection(row.strategy_id, seed)}>
                        <td className="bold" style={{ maxWidth: 220, overflow: 'hidden', textOverflow: 'ellipsis' }}>{row.strategy_id}</td>
                        <td className="right blu">{safeDisplay(resolveExecutionFitness(row), 6)}</td>
                        <td className="right">{resolveGaFitness(row) === undefined ? <span style={{ color: 'var(--tm)' }}>—</span> : safeDisplay(resolveGaFitness(row), 6)}</td>
                        <td><span className={`cs-badge ${div.cls}`}>{div.label}</span></td>
                        <td className="right">{safeDisplay(row.avg, 4)}</td>
                        <td className="right">{safeDisplay(row.std, 6)}</td>
                        <td>{row.classification ?? '—'}</td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          </div>

          {/* Generation history */}
          <div className="cs-card">
            <div className="cs-card-title">Generation History</div>
            <p style={{ fontSize: 11, color: 'var(--amb)', fontFamily: 'var(--mono)', marginBottom: 12 }}>
              Peak at Gen {resolvePeakGeneration(gaResult) ?? 'N/A'}
            </p>
            {gaResult.generation_history && gaResult.generation_history.length > 0 ? (
              <div style={{ maxHeight: 240, overflowY: 'auto' }}>
                {gaResult.generation_history.map((gen, index) => {
                  const val = resolveGaFitness(gen) ?? 0;
                  const pct = Math.max(0, Math.min(100, val * 100));
                  return (
                    <div key={index} className="cs-gen-row">
                      <span className="cs-gen-label">Gen {index}</span>
                      <div className="cs-gen-bar-track">
                        <div className="cs-gen-bar-fill" style={{ width: `${pct}%` }} />
                      </div>
                      <span className="cs-gen-val">{safeDisplay(val, 6)}</span>
                    </div>
                  );
                })}
              </div>
            ) : (
              <p style={{ fontSize: 12, color: 'var(--tm)' }}>No generation history available.</p>
            )}
          </div>

          {/* Signals snapshot */}
          {signalsSnapshot && (() => {
            const all          = signalsSnapshot?.signals ?? [];
            const meta         = signalsSnapshot?.meta;
            const elb          = meta?.edge_loss_breakdown;
            const apiRankings  = signalsSnapshot?.asset_rankings;
            const rollups      = Array.isArray(apiRankings) && apiRankings.length > 0
              ? apiRankings.map(r => ({ asset: r.asset, score: r.score, participation: r.participation, avgPnl: r.avg_pnl, weakExecutedCount: r.weak_executed_count }))
              : buildAssetRollups(all);
            const strongOnly   = strongOnlySignals || !includeWeakSignals;
            const bestPerAsset = topSignalsPerAsset(all, { topK: signalsTopK, includeWeak: includeWeakSignals, strongOnly });
            const trades           = meta?.trades ?? 0;
            const totalScenarios   = meta?.total_scenarios ?? 0;
            const weakEx           = elb?.weak_executed_count ?? 0;
            const weakPctOfScenarios   = totalScenarios > 0 ? (weakEx / totalScenarios) * 100 : 0;
            const fundedPctOfScenarios = totalScenarios > 0 ? (trades / totalScenarios) * 100 : 0;

            return (
              <div className="cs-gap-16">
                {/* Top assets */}
                <div className="cs-card" style={{ padding: 0 }}>
                  <div style={{ padding: '14px 20px 0' }}>
                    <div className="cs-card-title" style={{ marginBottom: 4 }}>Top Assets</div>
                    <p style={{ fontSize: 11, color: 'var(--tm)', marginBottom: 12 }}>
                      {Array.isArray(apiRankings) && apiRankings.length > 0
                        ? 'Server-ranked via asset_rankings from pipeline.'
                        : 'Client rollup from raw signals (fallback).'}
                    </p>
                  </div>
                  <div className="cs-table-wrap" style={{ border: 'none', borderRadius: '0 0 var(--r10) var(--r10)' }}>
                    <table className="cs-table">
                      <thead>
                        <tr>
                          <th>#</th>
                          <th>Asset</th>
                          <th className="right">Score</th>
                          <th className="right">Participation</th>
                          <th className="right">Avg PnL</th>
                          {rollups[0]?.weakExecutedCount !== undefined && <th className="right">Low-Conf Trades</th>}
                        </tr>
                      </thead>
                      <tbody>
                        {rollups.slice(0, 8).map((r, i) => (
                          <tr key={r.asset}>
                            <td style={{ color: 'var(--tm)', fontSize: 11 }}>{i + 1}</td>
                            <td className="bold">{r.asset}</td>
                            <td className="right blu">{safeDisplay(r.score, 4)}</td>
                            <td className="right">{safeDisplay(r.participation, 4)}</td>
                            <td className="right">{safeDisplay(r.avgPnl, 6)}</td>
                            {r.weakExecutedCount !== undefined && <td className="right">{r.weakExecutedCount}</td>}
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>

                {/* Best signals */}
                <div className="cs-card">
                  <div className="cs-card-title">Best Signals</div>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 14, flexWrap: 'wrap', gap: 10 }}>
                    <p style={{ fontSize: 12, color: 'var(--t2)', fontFamily: 'var(--mono)' }}>
                      Trades: {meta?.trades ?? 0} / {meta?.total_scenarios ?? 0}
                      {' · '}
                      Participation: {safeDisplay(meta?.participation, 4)}
                    </p>
                    <div className="cs-row-gap-12">
                      <label className="cs-checkbox-label">
                        <span style={{ color: 'var(--tm)' }}>Top per asset</span>
                        <select className="cs-select" value={signalsTopK} onChange={e => setSignalsTopK(Number(e.target.value))}>
                          <option value={1}>1</option>
                          <option value={2}>2</option>
                          <option value={3}>3</option>
                        </select>
                      </label>
                      <label className="cs-checkbox-label">
                        <input type="checkbox" className="cs-checkbox" checked={strongOnlySignals} onChange={e => setStrongOnlySignals(e.target.checked)} />
                        Strong only
                      </label>
                      <label className="cs-checkbox-label">
                        <input type="checkbox" className="cs-checkbox" checked={includeWeakSignals} onChange={e => setIncludeWeakSignals(e.target.checked)} />
                        Include weak
                      </label>
                    </div>
                  </div>
                  <p style={{ fontSize: 11, color: 'var(--tm)', marginBottom: 14 }}>
                    STRONG = gate composite score active · WEAK = surrogate path
                  </p>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                    {bestPerAsset.map((s, idx) => {
                      const st = signalStrength(s);
                      return (
                        <div key={`${s.asset}-${s.strategy_id}-${idx}`} className={`cs-signal-card ${st === 'STRONG' ? 'strong' : st === 'WEAK' ? 'weak' : ''}`}>
                          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                            <span className="cs-signal-asset">{s.asset}</span>
                            {st && <span className={`cs-badge ${st === 'STRONG' ? 'grn' : 'amb'}`}>{st}</span>}
                          </div>
                          <div className="cs-signal-row">
                            <span>Action: <span>{s.action}</span></span>
                            <span>Confidence: <span>{safeDisplay(s.confidence, 4)}</span></span>
                            <span>Entry: <span>{Array.isArray(s.entry_zone) ? `${safeDisplay(s.entry_zone[0], 2)} – ${safeDisplay(s.entry_zone[1], 2)}` : 'N/A'}</span></span>
                            <span>Target: <span>{safeDisplay(s.target, 2)}</span></span>
                            <span>SL: <span>{safeDisplay(s.stop_loss, 2)}</span></span>
                            <span>Holding: <span>{s.expected_holding_time ?? 'N/A'}</span></span>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </div>

                {/* Participation & edge */}
                <div className="cs-card">
                  <div className="cs-card-title">Participation & Edge</div>

<p style={{ fontSize: 11, color: 'var(--tm)', marginBottom: 12, lineHeight: 1.6 }}>
                    Participation = funded positions ÷ scenarios. Weak count = weak-eval surrogate BUY/SELL rows (pre-capital).
                  </p>
                  <div className="cs-gap-4">
                    <div className="cs-row">
                      <span className="cs-row-key">Participation</span>
                      <span className="cs-row-val">{safeDisplay(meta?.participation, 4)} ({trades} funded / {totalScenarios} scenarios)</span>
                    </div>
                    <div className="cs-row">
                      <span className="cs-row-key">Funded share of scenarios</span>
                      <span className="cs-row-val">{safeDisplay(fundedPctOfScenarios, 1)}%</span>
                    </div>
                    <div className="cs-row">
                      <span className="cs-row-key">Weak surrogate signals</span>
                      <span className="cs-row-val">{safeDisplay(weakPctOfScenarios, 1)}% (count {weakEx})</span>
                    </div>
                    <div className="cs-row">
                      <span className="cs-row-key">Edge retention (signal vs eval)</span>
                      <span className="cs-row-val">{safeDisplay(elb?.edge_retention_ratio, 4)}</span>
                    </div>
                    <div className="cs-row">
                      <span className="cs-row-key">True edge retention (inclusive)</span>
                      <span className="cs-row-val">{safeDisplay(elb?.true_edge_retention, 4)}</span>
                    </div>
                  </div>
                </div>

                {/* Debug weak path */}
                <div className="cs-card">
                  <div className="cs-card-title">Weak Path Debug</div>
                  <div className="cs-gap-4">
                    <div className="cs-row">
                      <span className="cs-row-key">LOW_VOL_REJECT</span>
                      <span className="cs-row-val">{elb?.weak_rejected_low_vol ?? '—'}</span>
                    </div>
                    <div className="cs-row">
                      <span className="cs-row-key">LOW_CONF_REJECT</span>
                      <span className="cs-row-val">{elb?.weak_rejected_low_conf ?? '—'}</span>
                    </div>
                    <div className="cs-row">
                      <span className="cs-row-key">WEAK_EXECUTED</span>
                      <span className="cs-row-val">{elb?.weak_executed_count ?? '—'}</span>
                    </div>
                  </div>
                </div>

                {/* All signals raw */}
                <div className="cs-card" style={{ padding: 0 }}>
                  <div style={{ padding: '14px 20px 0' }}>
                    <div className="cs-card-title" style={{ marginBottom: 4 }}>All Signals (Raw)</div>
                    <p style={{ fontSize: 11, color: 'var(--tm)', marginBottom: 12 }}>Full scenario-level list, no dedupe.</p>
                  </div>
                  <div className="cs-table-wrap" style={{ border: 'none', borderRadius: '0 0 var(--r10) var(--r10)' }}>
                    <table className="cs-table">
                      <thead>
                        <tr>
                          <th>Asset</th>
                          <th>Action</th>
                          <th>Strength</th>
                          <th className="right">Confidence</th>
                          <th>Entry Zone</th>
                          <th className="right">SL</th>
                          <th className="right">Target</th>
                          <th>Holding</th>
                        </tr>
                      </thead>
                      <tbody>
                        {all.filter(s => s.action !== 'HOLD').map((s, idx) => {
                          const st = signalStrength(s);
                          return (
                            <tr key={`${s.asset}-${s.strategy_id}-${idx}`}>
                              <td className="bold">{s.asset}</td>
                              <td>{s.action}</td>
                              <td>
                                {st ? <span className={`cs-badge ${st === 'STRONG' ? 'grn' : 'amb'}`}>{st}</span> : '—'}
                              </td>
                              <td className="right">{safeDisplay(s.confidence, 4)}</td>
                              <td>{Array.isArray(s.entry_zone) ? `${safeDisplay(s.entry_zone[0], 2)} – ${safeDisplay(s.entry_zone[1], 2)}` : 'N/A'}</td>
                              <td className="right">{safeDisplay(s.stop_loss, 2)}</td>
                              <td className="right">{safeDisplay(s.target, 2)}</td>
                              <td>{s.expected_holding_time ?? 'N/A'}</td>
                            </tr>
                          );
                        })}
                      </tbody>
                    </table>
                  </div>
                </div>
              </div>
            );
          })()}
        </div>
      )}
    </div>
  );
};

export default RunGA;
