import React, { useState, useEffect } from 'react';



function safeDisplay(value, digits = 2) {
  if (value === undefined || value === null || Number.isNaN(value)) {
    return "N/A";
  }
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
  const ga = resolveGaFitness(entry);
  const exec = resolveExecutionFitness(entry);
  if (ga === undefined || exec === undefined) return "— (Search not performed)";
  const normalizedGa = Math.max(0, Math.min(1, ga / 100.0));
  const divergence = exec - normalizedGa;
  if (divergence < -0.2) return "⚠ Overfit";
  if (divergence > 0.2) return "💎 Hidden Gem";
  return "✅ Aligned";
}

function findBestByGaFitness(list) {
  if (!Array.isArray(list) || list.length === 0) return { best: undefined, index: null };
  let best = list[0];
  let bestIndex = 0;
  let bestFitness = resolveGaFitness(best) ?? Number.NEGATIVE_INFINITY;
  for (let i = 1; i < list.length; i += 1) {
    const f = resolveGaFitness(list[i]) ?? Number.NEGATIVE_INFINITY;
    if (f > bestFitness) {
      best = list[i];
      bestIndex = i;
      bestFitness = f;
    }
  }
  return { best, index: bestIndex };
}

/** Strong path sets composite_score from the gate; weak surrogate uses 0. See core pipeline / SDS-style signal gating. */
function signalStrength(s) {
  if (!s || s.action === 'HOLD') return null;
  const cs = typeof s.composite_score === 'number' ? s.composite_score : 0;
  return cs > 1e-9 ? 'STRONG' : 'WEAK';
}

/**
 * Per-asset rollup for "Top Assets" when API does not send `asset_rankings` (fallback only).
 */
function buildAssetRollups(signals) {
  const byAsset = new Map();
  for (const s of signals ?? []) {
    const a = s.asset ?? 'UNKNOWN';
    if (!byAsset.has(a)) {
      byAsset.set(a, {
        asset: a,
        n: 0,
        traded: 0,
        confSum: 0,
        maxConf: Number.NEGATIVE_INFINITY,
        pnlSum: 0,
        pnlN: 0,
      });
    }
    const row = byAsset.get(a);
    row.n += 1;
    if (s.action !== 'HOLD') {
      row.traded += 1;
      const c = typeof s.confidence === 'number' ? s.confidence : 0;
      row.confSum += c;
      row.maxConf = Math.max(row.maxConf, c);
      if (typeof s.scenario_pnl === 'number') {
        row.pnlSum += s.scenario_pnl;
        row.pnlN += 1;
      }
    }
  }
  const out = [];
  for (const row of byAsset.values()) {
    const participation = row.n > 0 ? row.traded / row.n : 0;
    const avgConf = row.traded > 0 ? row.confSum / row.traded : 0;
    const avgPnl = row.pnlN > 0 ? row.pnlSum / row.pnlN : 0;
    const maxConf = row.maxConf === Number.NEGATIVE_INFINITY ? 0 : row.maxConf;
    const score = 0.5 * maxConf + 0.3 * participation + 0.2 * Math.max(0, Math.min(1, avgPnl + 0.5));
    out.push({
      asset: row.asset,
      score,
      participation,
      avgPnl,
      maxConf,
      avgConf,
      traded: row.traded,
      scenarios: row.n,
    });
  }
  out.sort((a, b) => b.score - a.score || a.asset.localeCompare(b.asset));
  return out;
}

/**
 * Best K non-HOLD signals per asset by confidence (dedupes duplicate template rows across scenarios).
 */
function topSignalsPerAsset(signals, { topK = 2, includeWeak = true, strongOnly = false } = {}) {
  const active = (signals ?? []).filter((s) => s.action !== 'HOLD');
  const filtered = active.filter((s) => {
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
  // Ensure each history entry has a ga_fitness for proper resolution
  const processedHistory = history.map(entry => ({ 
    ...entry, 
    ga_fitness: resolveGaFitness(entry) ?? (entry.fitness ?? 0) 
  }));

  const { best: historyBest, index: historyBestIndex } = findBestByGaFitness(processedHistory);
  const globalBest =
    raw?.global_best ??
    historyBest ??
    (Array.isArray(raw?.results) && raw.results.length > 0 ? raw.results[0] : undefined);
  const finalGenerationBest =
    raw?.final_generation_best ??
    raw?.final_gen_best ??
    (processedHistory.length > 0 ? processedHistory[processedHistory.length - 1] : undefined) ??
    (Array.isArray(raw?.results) && raw.results.length > 0 ? raw.results[0] : undefined);
  const peakGeneration =
    raw?.global_best_generation ??
    raw?.generation_found ??
    historyBestIndex;
  return {
    ...raw,
    generation_history: processedHistory, // Use processed history
    global_best: globalBest,
    final_generation_best: finalGenerationBest,
    global_best_generation: peakGeneration,
  };
}

const RunGA = ({ setSelectedStrategyForInspection }) => {
  const [populationSize, setPopulationSize] = useState(50);
  const [generations, setGenerations] = useState(20);
  const [mutationRate, setMutationRate] = useState(0.1);
  const [seed, setSeed] = useState(42);
  const [gaResult, setGaResult] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [signalsSnapshot, setSignalsSnapshot] = useState(null);
  const [persistedStorePayload, setPersistedStorePayload] = useState(null);
  const [storeLoading, setStoreLoading] = useState(false);
  const [storeError, setStoreError] = useState(null);
  const [signalsTopK, setSignalsTopK] = useState(2);
  const [includeWeakSignals, setIncludeWeakSignals] = useState(true);
  const [strongOnlySignals, setStrongOnlySignals] = useState(false);

  const resolvePeakGeneration = (result) => {
    const direct = result?.global_best_generation ?? result?.generation_found;
    if (direct !== undefined && direct !== null) return direct;
    const history = result?.generation_history;
    if (!Array.isArray(history) || history.length === 0) return null;
    let bestIdx = 0;
    let bestFitness = Number.NEGATIVE_INFINITY;
    history.forEach((g, idx) => {
      const s = resolveGaFitness(g) ?? 0;
      if (s > bestFitness) {
        bestFitness = s;
        bestIdx = idx;
      }
    });
    return bestIdx;
  };

  const fetchPersistedStrategyStore = async () => {
    setStoreLoading(true);
    setStoreError(null);
    try {
      const response = await fetch('http://localhost:8000/ga/strategy-store');
      if (!response.ok) {
        const text = await response.text();
        throw new Error(text || 'Failed to load persisted strategy store');
      }
      const data = await response.json();
      setPersistedStorePayload(data);
    } catch (err) {
      setStoreError(err.message);
      setPersistedStorePayload(null);
    } finally {
      setStoreLoading(false);
    }
  };

  useEffect(() => {
    fetchPersistedStrategyStore();
  }, []);

  const handleRunGA = async () => {
    setLoading(true);
    setError(null);
    setGaResult(null);
    setSignalsSnapshot(null);

    try {
      const response = await fetch('http://localhost:8000/run_ga');

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.message || 'Failed to run GA');
      }

      const data = await response.json();
      const normalized = normalizeGaResult(data);
      const signalsResp = await fetch('http://localhost:8000/signals/latest');
      if (!signalsResp.ok) {
        const errorData = await signalsResp.json();
        throw new Error(errorData.message || 'Failed to fetch latest signals');
      }
      const signalsData = await signalsResp.json();
      console.log("FULL RESPONSE:", data);
      console.log("NORMALIZED_GA_RESPONSE:", normalized);
      console.log("SIGNALS_SNAPSHOT:", signalsData);
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
    <div className="p-4">
      <h2 className="text-2xl font-semibold mb-4">Run Genetic Algorithm</h2>

      <details className="mb-6 border border-gray-200 rounded-lg bg-gray-50 p-3" open>
        <summary className="cursor-pointer font-semibold text-gray-800">
          Persisted strategy store (on-disk JSON)
        </summary>
        <p className="text-xs text-gray-600 mt-2 mb-2">
          Same file the API loads for signals: full per-asset GA results written by the training pipeline
          (interactive <code className="bg-gray-200 px-1 rounded">/run_ga</code> does not overwrite this file).
        </p>
        <div className="flex flex-wrap items-center gap-2">
          <button
            type="button"
            className="text-sm bg-gray-200 hover:bg-gray-300 px-3 py-1 rounded"
            onClick={fetchPersistedStrategyStore}
            disabled={storeLoading}
          >
            {storeLoading ? 'Loading…' : 'Refresh'}
          </button>
          {persistedStorePayload?.path && (
            <span className="text-xs text-gray-600 font-mono break-all">
              Path: {persistedStorePayload.path}
            </span>
          )}
        </div>
        {storeError && <p className="text-red-600 text-sm mt-2">{storeError}</p>}
        {persistedStorePayload !== null && persistedStorePayload.store != null && (
          <pre className="mt-3 text-xs overflow-x-auto max-h-96 overflow-y-auto bg-gray-900 text-gray-100 p-3 rounded">
            {JSON.stringify(persistedStorePayload.store, null, 2)}
          </pre>
        )}
        {persistedStorePayload !== null &&
          persistedStorePayload.store == null &&
          !storeLoading &&
          !storeError && (
            <p className="text-sm text-gray-600 mt-2">
              No store on disk at that path (or parse failed). Run the full training pipeline to generate{' '}
              <code className="bg-gray-200 px-1 rounded">strategy_store.json</code>.
            </p>
          )}
      </details>

      <div className="grid grid-cols-2 gap-4 mb-6">
        <div>
          <label className="block text-gray-700 text-sm font-bold mb-2" htmlFor="population_size">
            Population Size:
          </label>
          <input
            type="number"
            id="population_size"
            className="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
            value={populationSize}
            onChange={(e) => setPopulationSize(e.target.value)}
          />
        </div>
        <div>
          <label className="block text-gray-700 text-sm font-bold mb-2" htmlFor="generations">
            Generations:
          </label>
          <input
            type="number"
            id="generations"
            className="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
            value={generations}
            onChange={(e) => setGenerations(e.target.value)}
          />
        </div>
        <div>
          <label className="block text-gray-700 text-sm font-bold mb-2" htmlFor="mutation_rate">
            Mutation Rate:
          </label>
          <input
            type="number"
            id="mutation_rate"
            step="0.01"
            className="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
            value={mutationRate}
            onChange={(e) => setMutationRate(e.target.value)}
          />
        </div>
        <div>
          <label className="block text-gray-700 text-sm font-bold mb-2" htmlFor="seed">
            Seed:
          </label>
          <input
            type="number"
            id="seed"
            className="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
            value={seed}
            onChange={(e) => setSeed(e.target.value)}
          />
        </div>
      </div>

      <button
        className="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline"
        onClick={handleRunGA}
        disabled={loading}
      >
        {loading ? 'Running GA...' : 'Run GA'}
      </button>

      {error && <p className="text-red-500 mt-4">Error: {error}</p>}

      {gaResult && (
        <div className="mt-6">
          <div className="bg-yellow-50 border border-yellow-200 rounded-md p-4 mb-4">
            <h3 className="text-xl font-semibold mb-2">🏆 Best Strategy (Execution Verified)</h3>
            <p><span className="font-semibold">Execution Fitness:</span> {safeDisplay(resolveExecutionFitness(gaResult.global_best), 6)}</p>
            <p><span className="font-semibold">GA Fitness:</span> {resolveGaFitness(gaResult.global_best) === undefined ? "— (Search not performed)" : safeDisplay(resolveGaFitness(gaResult.global_best), 6)}</p>
            <p><span className="font-semibold">Avg PnL:</span> {safeDisplay(gaResult.global_best?.avg, 2)}</p>
            <p><span className="font-semibold">Std Dev:</span> {safeDisplay(gaResult.global_best?.std, 4)}</p>
            <p><span className="font-semibold">Found at Generation:</span> {resolvePeakGeneration(gaResult) ?? 'N/A'}</p>
          </div>

          <div className="bg-blue-50 border border-blue-200 rounded-md p-4 mb-4">
            <h3 className="text-xl font-semibold mb-2">📍 Final Generation Best (Search Result)</h3>
            <p><span className="font-semibold">Execution Fitness:</span> {safeDisplay(resolveExecutionFitness(gaResult.final_generation_best ?? gaResult.final_gen_best), 6)}</p>
            <p><span className="font-semibold">GA Fitness:</span> {resolveGaFitness(gaResult.final_generation_best ?? gaResult.final_gen_best) === undefined ? "— (Search not performed)" : safeDisplay(resolveGaFitness(gaResult.final_generation_best ?? gaResult.final_gen_best), 6)}</p>
            <p><span className="font-semibold">Avg PnL:</span> {safeDisplay((gaResult.final_generation_best ?? gaResult.final_gen_best)?.avg, 2)}</p>
            <p><span className="font-semibold">Std Dev:</span> {safeDisplay((gaResult.final_generation_best ?? gaResult.final_gen_best)?.std, 4)}</p>
          </div>

          {gaResult.best_per_regime && Object.keys(gaResult.best_per_regime).length > 0 && (
            <div className="bg-green-50 border border-green-200 rounded-md p-4 mb-4">
              <h3 className="text-xl font-semibold mb-2">🧭 Best Per Regime (Execution Verified)</h3>
              <div className="overflow-x-auto shadow-md rounded-lg">
                <table className="min-w-full divide-y divide-gray-200">
                  <thead className="bg-gray-50">
                    <tr>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Regime Key</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Strategy ID</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Execution Fitness</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">GA Fitness</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Class</th>
                    </tr>
                  </thead>
                  <tbody className="bg-white divide-y divide-gray-200">
                    {Object.entries(gaResult.best_per_regime).map(([regimeKey, row]) => (
                      <tr key={regimeKey} className="hover:bg-gray-50 transition duration-150 ease-in-out" onClick={() => setSelectedStrategyForInspection(row.strategy_id, seed)}>
                        <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">{regimeKey}</td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{row.strategy_id}</td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{safeDisplay(resolveExecutionFitness(row), 6)}</td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{resolveGaFitness(row) === undefined ? "— (Search not performed)" : safeDisplay(resolveGaFitness(row), 6)}</td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{row.classification}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          <h3 className="text-xl font-semibold mb-3">Best Strategies</h3>
          <div className="overflow-x-auto shadow-md rounded-lg">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Strategy ID</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Execution Fitness</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">GA Fitness</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Divergence</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Avg PNL</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Std Dev</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Class</th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {gaResult.results && gaResult.results.map((row) => (
                  <tr
                    key={row.strategy_id}
                    className="hover:bg-gray-50 transition duration-150 ease-in-out cursor-pointer"
                    onClick={() => setSelectedStrategyForInspection(row.strategy_id, seed)}
                  >
                    <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">{row.strategy_id}</td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{safeDisplay(resolveExecutionFitness(row), 6)}</td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{resolveGaFitness(row) === undefined ? "— (Search not performed)" : safeDisplay(resolveGaFitness(row), 6)}</td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{divergenceBadge(row)}</td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{safeDisplay(row.avg, 2)}</td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{safeDisplay(row.std, 4)}</td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{row.classification}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          <h3 className="text-xl font-semibold mt-6 mb-3">Generation History (GA Search Progress)</h3>
          <div className="bg-gray-50 p-4 rounded-md">
            <p className="mb-2 text-sm font-semibold text-amber-700">
              ⭐ Peak at Gen {resolvePeakGeneration(gaResult) ?? 'N/A'}
            </p>
            {gaResult.generation_history && gaResult.generation_history.length > 0 ? (
              <div className="text-sm">
                {/* Simple text-based line chart representation */}
                {gaResult.generation_history.map((gen, index) => (
                  <div key={index} className="flex items-center mb-1">
                    <span className="w-16 text-right mr-2">Gen {index}:</span>
                    <span className="w-24 text-blue-600">{'█'.repeat(Math.max(0, Math.min(50, Math.floor((resolveGaFitness(gen) ?? 0) * 50))))}</span> {/* Scale to 50 blocks max */}
                    <span className="ml-2">{safeDisplay(resolveGaFitness(gen), 6)}</span>
                  </div>
                ))}
              </div>
            ) : (
              <p>No generation history available.</p>
            )}
          </div>

          {signalsSnapshot && (() => {
            const all = signalsSnapshot?.signals ?? [];
            const meta = signalsSnapshot?.meta;
            const elb = meta?.edge_loss_breakdown;
            const apiRankings = signalsSnapshot?.asset_rankings;
            const rollups =
              Array.isArray(apiRankings) && apiRankings.length > 0
                ? apiRankings.map((r) => ({
                    asset: r.asset,
                    score: r.score,
                    participation: r.participation,
                    avgPnl: r.avg_pnl,
                    weakExecutedCount: r.weak_executed_count,
                  }))
                : buildAssetRollups(all);
            const strongOnly = strongOnlySignals || !includeWeakSignals;
            const bestPerAsset = topSignalsPerAsset(all, {
              topK: signalsTopK,
              includeWeak: includeWeakSignals,
              strongOnly,
            });
            const trades = meta?.trades ?? 0;
            const totalScenarios = meta?.total_scenarios ?? 0;
            const weakEx = elb?.weak_executed_count ?? 0;
            // `trades` = funded positions (qty>0) after allocator; `weakEx` = weak-path BUY/SELL scenario
            // signals (pre-alloc). Do NOT mix into (trades - weakEx) / trades — that yields nonsense (e.g. -900%).
            const weakPctOfScenarios =
              totalScenarios > 0 ? (weakEx / totalScenarios) * 100 : 0;
            const fundedPctOfScenarios =
              totalScenarios > 0 ? (trades / totalScenarios) * 100 : 0;

            return (
              <div className="mt-6 space-y-6">
                <div className="bg-amber-50 border border-amber-200 rounded-md p-4">
                  <h3 className="text-xl font-semibold mb-2">🏆 Top Assets Today</h3>
                  <p className="text-xs text-gray-600 mb-2">
                    {Array.isArray(apiRankings) && apiRankings.length > 0
                      ? 'Server-ranked (`asset_rankings` from pipeline; same composite score as folder evaluation).'
                      : 'Client rollup from raw signals (fallback when `asset_rankings` is empty).'}
                  </p>
                  <ol className="list-decimal list-inside space-y-1 text-sm">
                    {rollups.slice(0, 8).map((r) => (
                      <li key={r.asset}>
                        <span className="font-medium">{r.asset}</span>
                        {' → '}
                        score {safeDisplay(r.score, 2)}
                        {' → '}
                        participation {safeDisplay(r.participation, 2)}
                        {' → '}
                        avg pnl {safeDisplay(r.avgPnl, 4)}
                        {typeof r.weakExecutedCount === 'number' && (
                          <>
                            {' → '}
                            low-conf trades (rank metric) {r.weakExecutedCount}
                          </>
                        )}
                      </li>
                    ))}
                  </ol>
                </div>

                <div className="bg-purple-50 border border-purple-200 rounded-md p-4">
                  <h3 className="text-xl font-semibold mb-2">🔥 Best Signals (filtered)</h3>
                  <p className="text-sm mb-3">
                    Trades: {meta?.trades ?? 0} / {meta?.total_scenarios ?? 0}
                    {' | '}
                    Participation: {safeDisplay(meta?.participation, 2)}
                  </p>
                  <div className="flex flex-wrap gap-4 mb-4 text-sm items-center">
                    <label className="inline-flex items-center gap-2 cursor-pointer">
                      <span>Top per asset</span>
                      <select
                        className="border rounded px-2 py-1"
                        value={signalsTopK}
                        onChange={(e) => setSignalsTopK(Number(e.target.value))}
                      >
                        <option value={1}>1</option>
                        <option value={2}>2</option>
                        <option value={3}>3</option>
                      </select>
                    </label>
                    <label className="inline-flex items-center gap-2 cursor-pointer">
                      <input
                        type="checkbox"
                        checked={strongOnlySignals}
                        onChange={(e) => setStrongOnlySignals(e.target.checked)}
                      />
                      Show strong only
                    </label>
                    <label className="inline-flex items-center gap-2 cursor-pointer">
                      <input
                        type="checkbox"
                        checked={includeWeakSignals}
                        onChange={(e) => setIncludeWeakSignals(e.target.checked)}
                      />
                      Include weak
                    </label>
                  </div>
                  <p className="text-xs text-gray-600 mb-3">
                    STRONG = gate composite score (green); WEAK = surrogate path (yellow). Aligns with core pipeline semantics.
                  </p>
                  <div className="flex flex-col gap-4">
                    {bestPerAsset.map((s, idx) => {
                      const st = signalStrength(s);
                      const rowBg = st === 'STRONG' ? 'bg-green-50 border-green-200' : st === 'WEAK' ? 'bg-yellow-50 border-yellow-200' : 'bg-white border-gray-200';
                      return (
                        <div
                          key={`${s.asset}-${s.strategy_id}-${idx}`}
                          className={`border rounded-md p-3 ${rowBg}`}
                        >
                          <div className="font-semibold text-gray-800">{s.asset}</div>
                          <div className="text-sm mt-1">
                            <span className="font-medium">{s.action}</span>
                            {' | '}
                            Confidence: {safeDisplay(s.confidence, 3)}
                            {st && (
                              <span className="ml-2 text-xs font-semibold uppercase text-gray-700">
                                {st === 'STRONG' ? '🟢 STRONG' : '🟡 WEAK'}
                              </span>
                            )}
                          </div>
                          <div className="text-sm mt-1 text-gray-700">
                            Entry:{' '}
                            {Array.isArray(s.entry_zone)
                              ? `${safeDisplay(s.entry_zone[0], 2)} - ${safeDisplay(s.entry_zone[1], 2)}`
                              : 'N/A'}
                          </div>
                          <div className="text-sm text-gray-700">
                            Target: {safeDisplay(s.target, 2)} | SL: {safeDisplay(s.stop_loss, 2)} | Holding:{' '}
                            {s.expected_holding_time ?? 'N/A'}
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </div>

                <div className="bg-slate-50 border border-slate-200 rounded-md p-4">
                  <h3 className="text-xl font-semibold mb-2">📉 Participation &amp; edge</h3>
                  <p className="text-xs text-gray-600 mb-2">
                    Participation = funded positions ÷ scenarios. Weak count = weak-eval surrogate BUY/SELL rows
                    (pre-capital); funded count = rows with qty &gt; 0 after allocator — different denominators than
                    a single &quot;strong %&quot;.
                  </p>
                  <ul className="text-sm space-y-1">
                    <li>
                      Participation: {safeDisplay(meta?.participation, 2)} ({trades} funded / {totalScenarios}{' '}
                      scenarios)
                    </li>
                    <li>
                      Funded share of scenarios: {safeDisplay(fundedPctOfScenarios, 1)}% / Weak surrogate signals
                      (share of scenarios): {safeDisplay(weakPctOfScenarios, 1)}% (count {weakEx})
                    </li>
                    <li>
                      Edge retention (signal vs Σ eval edge): {safeDisplay(elb?.edge_retention_ratio, 2)} — inclusive
                      (signal vs eval + weak missing): {safeDisplay(elb?.true_edge_retention, 2)}
                    </li>
                  </ul>
                </div>

                <div className="bg-gray-100 border border-gray-300 rounded-md p-4">
                  <h3 className="text-xl font-semibold mb-2">🔬 Debug (weak path)</h3>
                  <ul className="text-sm font-mono space-y-1">
                    <li>LOW_VOL_REJECT: {elb?.weak_rejected_low_vol ?? '—'}</li>
                    <li>LOW_CONF_REJECT: {elb?.weak_rejected_low_conf ?? '—'}</li>
                    <li>WEAK_EXECUTED: {elb?.weak_executed_count ?? '—'}</li>
                  </ul>
                </div>

                <div className="bg-white border border-gray-200 rounded-md p-4">
                  <h3 className="text-xl font-semibold mb-2">All signals (raw)</h3>
                  <p className="text-xs text-gray-500 mb-2">Full scenario-level list (no dedupe).</p>
                  <div className="overflow-x-auto">
                    <table className="min-w-full bg-white border border-gray-300">
                      <thead>
                        <tr>
                          <th className="py-2 px-4 border-b text-left">Asset</th>
                          <th className="py-2 px-4 border-b text-left">Action</th>
                          <th className="py-2 px-4 border-b text-left">Strength</th>
                          <th className="py-2 px-4 border-b text-left">Confidence</th>
                          <th className="py-2 px-4 border-b text-left">Entry Zone</th>
                          <th className="py-2 px-4 border-b text-left">Stop Loss</th>
                          <th className="py-2 px-4 border-b text-left">Target</th>
                          <th className="py-2 px-4 border-b text-left">Holding</th>
                        </tr>
                      </thead>
                      <tbody>
                        {all
                          .filter((s) => s.action !== 'HOLD')
                          .map((s, idx) => (
                            <tr key={`${s.asset}-${s.strategy_id}-${idx}`} className="hover:bg-gray-100">
                              <td className="py-2 px-4 border-b">{s.asset}</td>
                              <td className="py-2 px-4 border-b">{s.action}</td>
                              <td className="py-2 px-4 border-b">{signalStrength(s) ?? '—'}</td>
                              <td className="py-2 px-4 border-b">{safeDisplay(s.confidence, 3)}</td>
                              <td className="py-2 px-4 border-b">
                                {Array.isArray(s.entry_zone)
                                  ? `${safeDisplay(s.entry_zone[0], 2)} - ${safeDisplay(s.entry_zone[1], 2)}`
                                  : 'N/A'}
                              </td>
                              <td className="py-2 px-4 border-b">{safeDisplay(s.stop_loss, 2)}</td>
                              <td className="py-2 px-4 border-b">{safeDisplay(s.target, 2)}</td>
                              <td className="py-2 px-4 border-b">{s.expected_holding_time ?? 'N/A'}</td>
                            </tr>
                          ))}
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
