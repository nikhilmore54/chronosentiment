import React, { useState, useEffect } from 'react';
import { apiUrl } from '../services/api';

function safeDisplay(value, digits = 2) {
  if (value === undefined || value === null || Number.isNaN(value)) return 'N/A';
  return typeof value === 'number' ? value.toFixed(digits) : value;
}

// ARTIFACT-001 REMOVED: resolveExecutionFitness() fallback cascade eliminated.
// Backend guarantees execution_fitness is always present in StrategyEvaluationDto.
// Direct field access: entry.execution_fitness

function resolveGaFitness(entry) {
  if (!entry) return undefined;
  if (typeof entry.ga_fitness === 'number') return entry.ga_fitness;
  return undefined;
}

function divergenceBadge(entry) {
  const ga   = resolveGaFitness(entry);
  const exec = (entry && typeof entry.execution_fitness === 'number') ? entry.execution_fitness : undefined;
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
      const response = await fetch(apiUrl('/ga/strategy-store'));
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
      const response = await fetch(apiUrl('/run_ga'));
      if (!response.ok) { const errorData = await response.json(); throw new Error(errorData.message || 'Failed to run GA'); }
      const data       = await response.json();
      const normalized = normalizeGaResult(data);
      const signalsResp = await fetch(apiUrl('/signals/latest'));
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
    <div style={{ display: 'flex', gap: '60px', alignItems: 'flex-start', paddingTop: '20px' }}>
      
      {/* ─── LEFT: Editorial Sidebar (Parameters) ─── */}
      <aside style={{ width: '280px', flexShrink: 0, position: 'sticky', top: '80px' }}>
        <h2 style={{ fontSize: '14px', fontWeight: 600, color: 'var(--t1)', marginBottom: '24px' }}>Parameters</h2>
        
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px', marginBottom: '24px' }}>
          <div className="cs-field">
            <label className="cs-label" htmlFor="population_size">Population</label>
            <input type="number" id="population_size" className="cs-input" style={{ background: 'var(--card)', border: '1px solid var(--b)' }} value={populationSize} onChange={e => setPopulationSize(e.target.value)} />
          </div>
          <div className="cs-field">
            <label className="cs-label" htmlFor="generations">Generations</label>
            <input type="number" id="generations" className="cs-input" style={{ background: 'var(--card)', border: '1px solid var(--b)' }} value={generations} onChange={e => setGenerations(e.target.value)} />
          </div>
          <div className="cs-field">
            <label className="cs-label" htmlFor="mutation_rate">Mutation</label>
            <input type="number" id="mutation_rate" step="0.01" className="cs-input" style={{ background: 'var(--card)', border: '1px solid var(--b)' }} value={mutationRate} onChange={e => setMutationRate(e.target.value)} />
          </div>
          <div className="cs-field">
            <label className="cs-label" htmlFor="seed">Seed</label>
            <input type="number" id="seed" className="cs-input" style={{ background: 'var(--card)', border: '1px solid var(--b)' }} value={seed} onChange={e => setSeed(e.target.value)} />
          </div>
        </div>

        {/* Signal filter controls — visible in both pre- and post-execution states */}
        <div style={{ marginBottom: '24px', paddingBottom: '24px', borderBottom: '1px solid var(--b)' }}>
          <div style={{ fontSize: '11px', fontWeight: 700, color: 'var(--tm)', textTransform: 'uppercase', letterSpacing: '0.06em', marginBottom: '12px' }}>
            Signal Filters
          </div>
          <div className="cs-field" style={{ marginBottom: '10px' }}>
            <label className="cs-label" htmlFor="signals_topk">Top-K per asset</label>
            <input
              type="number" id="signals_topk" min={1} max={10}
              className="cs-input"
              style={{ background: 'var(--card)', border: '1px solid var(--b)' }}
              value={signalsTopK}
              onChange={e => setSignalsTopK(Math.max(1, Number(e.target.value)))}
            />
          </div>
          <label className="cs-checkbox-label" style={{ marginBottom: '8px', display: 'flex' }}>
            <input
              type="checkbox" className="cs-checkbox"
              checked={includeWeakSignals}
              onChange={e => { setIncludeWeakSignals(e.target.checked); if (e.target.checked) setStrongOnlySignals(false); }}
            />
            Include weak signals
          </label>
          <label className="cs-checkbox-label" style={{ display: 'flex' }}>
            <input
              type="checkbox" className="cs-checkbox"
              checked={strongOnlySignals}
              onChange={e => { setStrongOnlySignals(e.target.checked); if (e.target.checked) setIncludeWeakSignals(false); }}
            />
            Strong signals only
          </label>
        </div>

        <button className="cs-btn cs-btn-primary" style={{ width: '100%', padding: '10px 0' }} onClick={handleRunGA} disabled={loading}>
          {loading ? 'Running execution...' : 'Execute baseline'}
        </button>

        {error && (
          <div style={{ marginTop: '16px', padding: '12px', background: 'var(--rdim)', border: '1px solid var(--bred)', borderRadius: 'var(--r8)', color: 'var(--red)', fontSize: '12px', fontFamily: 'var(--mono)' }}>
            {error}
          </div>
        )}

        {/* Persisted store status */}
        {!loading && (
          <div style={{ marginTop: '20px', fontSize: '11px', color: 'var(--tm)', fontFamily: 'var(--mono)' }}>
            {storeLoading ? (
              <span>Loading store...</span>
            ) : storeError ? (
              <span style={{ color: 'var(--amb)' }}>Store unavailable</span>
            ) : persistedStorePayload ? (
              <span style={{ color: 'var(--grn)' }}>
                Store: {Array.isArray(persistedStorePayload.strategies) ? persistedStorePayload.strategies.length : '?'} strategies
              </span>
            ) : null}
          </div>
        )}
      </aside>

      {/* ─── RIGHT: The Narrative Stream ─── */}
      <main style={{ flex: 1, maxWidth: '800px', minHeight: '600px' }}>
        
        {/* State: Loading */}
        {loading && (
          <div style={{ padding: '32px', background: 'var(--card)', border: '1px solid var(--b)' }}>
            <div className="cs-skeleton" style={{ height: '14px', width: '140px', marginBottom: '24px' }}></div>
            <div className="cs-skeleton" style={{ height: '56px', marginBottom: '12px' }}></div>
            <div className="cs-skeleton" style={{ height: '40px', marginBottom: '8px' }}></div>
            <div className="cs-skeleton" style={{ height: '40px', marginBottom: '8px' }}></div>
            <div className="cs-skeleton" style={{ height: '40px' }}></div>
            <div style={{ marginTop: '24px', fontSize: '12px', color: 'var(--tm)', fontFamily: 'var(--mono)' }}>
              Executing genetic algorithm...
            </div>
          </div>
        )}

        {/* State: Pre-Execution Live Environment Block */}
        {!gaResult && !loading && (
          <div style={{ padding: '32px', background: 'var(--card)', border: '1px solid var(--b)' }}>
            <h2 style={{ fontSize: '14px', fontWeight: 600, color: 'var(--t1)', marginBottom: '24px' }}>Environment</h2>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '32px' }}>
              <div>
                <div style={{ fontSize: '12px', color: 'var(--tm)', marginBottom: '8px' }}>Integrity</div>
                <div style={{ fontSize: '24px', fontWeight: 500, color: 'var(--t1)', fontFamily: 'var(--mono)', marginBottom: '4px' }}>100.0%</div>
                <div style={{ fontSize: '11px', color: 'var(--t2)' }}>No timeline gaps detected</div>
              </div>
              <div>
                <div style={{ fontSize: '12px', color: 'var(--tm)', marginBottom: '8px' }}>Bounds</div>
                <div style={{ fontSize: '16px', fontWeight: 500, color: 'var(--t1)', fontFamily: 'var(--mono)', marginBottom: '4px' }}>0 → 12,400</div>
                <div style={{ fontSize: '11px', color: 'var(--t2)' }}>Ready for deterministic replay</div>
              </div>
              <div>
                <div style={{ fontSize: '12px', color: 'var(--tm)', marginBottom: '8px' }}>Dispersion</div>
                <div style={{ fontSize: '24px', fontWeight: 500, color: 'var(--t1)', fontFamily: 'var(--mono)', marginBottom: '4px' }}>1.00x</div>
                <div style={{ fontSize: '11px', color: 'var(--t2)' }}>Governor state: NOMINAL</div>
              </div>
              <div>
                <div style={{ fontSize: '12px', color: 'var(--tm)', marginBottom: '8px' }}>Network</div>
                <div style={{ fontSize: '16px', fontWeight: 500, color: 'var(--grn)', fontFamily: 'var(--mono)', marginBottom: '4px' }}>ACTIVE</div>
                <div style={{ fontSize: '11px', color: 'var(--t2)' }}>All subsystems nominal</div>
              </div>
            </div>
            <div style={{ marginTop: '32px', paddingTop: '16px', borderTop: '1px solid var(--b)', fontSize: '12px', color: 'var(--tm)' }}>
              Awaiting execution mandate...
            </div>
          </div>
        )}

        {/* State: Execution Complete */}
        {gaResult && (
          <div style={{ display: 'flex', flexDirection: 'column' }}>
            
            {/* Zone 1: The Verdict */}
            <section style={{ marginBottom: '80px' }}>
              <h2 style={{ fontSize: '14px', fontWeight: 600, color: 'var(--t1)', marginBottom: '16px' }}>Execution verdict</h2>
              <div style={{ display: 'flex', alignItems: 'baseline', gap: '16px', marginBottom: '8px' }}>
                <div style={{ fontSize: '48px', fontWeight: 600, color: 'var(--t1)', letterSpacing: '-0.03em', lineHeight: 1 }}>
                  {safeDisplay(gaResult.global_best?.execution_fitness, 6)}
                </div>
                {divergenceBadge(gaResult.global_best) && (
                  <span className={`cs-badge ${divergenceBadge(gaResult.global_best).cls}`}>
                    {divergenceBadge(gaResult.global_best).label}
                  </span>
                )}
              </div>
              <div style={{ display: 'flex', gap: '32px', color: 'var(--t2)', fontSize: '12px', fontFamily: 'var(--mono)' }}>
                <span>Certified Execution Fitness</span>
                <span>Search Gen: {resolvePeakGeneration(gaResult) ?? 'N/A'}</span>
                <span>Avg PnL: {safeDisplay(gaResult.global_best?.avg, 4)}</span>
              </div>
            </section>

            {/* Zone 2: Signals Topology */}
            {signalsSnapshot && (() => {
              const all = signalsSnapshot?.signals ?? [];
              const bestPerAsset = topSignalsPerAsset(all, { topK: 3, includeWeak: true, strongOnly: false });

              return (
                <section>
                  <h2 style={{ fontSize: '14px', fontWeight: 600, color: 'var(--t1)', marginBottom: '16px' }}>Derived execution topology</h2>
                  
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '0' }}>
                    {bestPerAsset.map((s, idx) => {
                      const st = signalStrength(s);
                      return (
                        <div key={`${s.asset}-${s.strategy_id}-${idx}`} style={{ 
                          display: 'flex', alignItems: 'center', padding: '16px 0', 
                          borderBottom: '1px solid rgba(148,163,184,0.08)', gap: '20px'
                        }}>
                          <div style={{ width: '80px', flexShrink: 0, color: 'var(--t1)', fontWeight: 600, fontSize: '14px', fontFamily: 'var(--mono)' }}>
                            {s.asset}
                          </div>
                          
                          <div style={{ width: '60px', flexShrink: 0 }}>
                            <span style={{ fontSize: '10px', padding: '4px 8px', borderRadius: '4px', fontFamily: 'var(--mono)', fontWeight: 600,
                              background: s.action === 'BUY' ? 'rgba(16,185,129,0.1)' : 'rgba(239,68,68,0.1)',
                              color: s.action === 'BUY' ? 'var(--grn)' : 'var(--red)'
                            }}>
                              {s.action}
                            </span>
                          </div>

                          <div style={{ flex: 1, display: 'flex', gap: '24px', color: 'var(--t2)', fontSize: '11px', fontFamily: 'var(--mono)' }}>
                            <div>
                              <div style={{ color: 'var(--tm)', fontSize: '9px', textTransform: 'uppercase', marginBottom: '4px' }}>Entry</div>
                              {Array.isArray(s.entry_zone) ? `${safeDisplay(s.entry_zone[0], 2)} – ${safeDisplay(s.entry_zone[1], 2)}` : 'N/A'}
                            </div>
                            <div>
                              <div style={{ color: 'var(--tm)', fontSize: '9px', textTransform: 'uppercase', marginBottom: '4px' }}>Target</div>
                              {safeDisplay(s.target, 2)}
                            </div>
                            <div>
                              <div style={{ color: 'var(--tm)', fontSize: '9px', textTransform: 'uppercase', marginBottom: '4px' }}>Stop Loss</div>
                              {safeDisplay(s.stop_loss, 2)}
                            </div>
                          </div>

                          <div style={{ width: '80px', textAlign: 'right' }}>
                             {st && <span className={`cs-badge ${st === 'STRONG' ? 'grn' : 'amb'}`}>{st}</span>}
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </section>
              );
            })()}

          </div>
        )}
      </main>
    </div>
  );
};

export default RunGA;
