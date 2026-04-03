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
    if (!row) return 0;
    if (typeof row.ga_fitness === 'number') return row.ga_fitness;
    return null;
  };

  const handleCompareStrategies = async () => {
    setLoading(true);
    setError(null);
    setComparisonResult(null);

    const strategy_ids = strategyIdsInput.split(',').map(id => id.trim()).filter(id => id !== '');

    if (strategy_ids.length < 2) {
      setError('Please enter at least two strategy IDs separated by commas.');
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
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          strategies: strategiesPayload,
          scenarios: [],
          seed: Number(seed),
        }),
      });

      if (!response.ok) {
        const text = await response.text();
        let msg = 'Failed to compare strategies';
        try {
          const errorData = JSON.parse(text);
          msg = errorData.message || errorData.error || msg;
        } catch {
          if (text) msg = text;
        }
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
    <div className="p-4">
      <h2 className="text-2xl font-semibold mb-4">Compare Strategies</h2>

      <div className="mb-6">
        <label className="block text-gray-700 text-sm font-bold mb-2" htmlFor="strategy_ids">
          Strategy IDs (comma-separated):
        </label>
        <input
          type="text"
          id="strategy_ids"
          className="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
          value={strategyIdsInput}
          onChange={(e) => setStrategyIdsInput(e.target.value)}
          placeholder="e.g., strat_200_300_400_500_42 (same format as GA / inspect)"
        />
      </div>

      <div className="mb-6">
        <label className="block text-gray-700 text-sm font-bold mb-2" htmlFor="compare_seed">
          Seed (must match inspect / evaluation):
        </label>
        <input
          type="number"
          id="compare_seed"
          className="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
          value={seed}
          onChange={(e) => setSeed(Number(e.target.value))}
        />
      </div>

      <button
        className="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline"
        onClick={handleCompareStrategies}
        disabled={loading}
      >
        {loading ? 'Comparing...' : 'Compare Strategies'}
      </button>

      {error && <p className="text-red-500 mt-4">Error: {error}</p>}

      {comparisonResult && (
        <div className="mt-6">
          <h3 className="text-xl font-semibold mb-3">Comparison Result</h3>
          <div className="overflow-x-auto">
            <table className="min-w-full bg-white border border-gray-300">
              <thead>
                <tr>
                  <th className="py-2 px-4 border-b text-left">Strategy ID</th>
                  <th className="py-2 px-4 border-b text-left">Execution Fitness</th>
                  <th className="py-2 px-4 border-b text-left">GA Fitness</th>
                </tr>
              </thead>
              <tbody>
                {comparisonResult.ranking.map((row) => (
                  <tr
                    key={row.strategy_id}
                    className={setSelectedStrategyForInspection ? 'hover:bg-gray-50 cursor-pointer' : ''}
                    onClick={() => {
                      if (setSelectedStrategyForInspection) {
                        setSelectedStrategyForInspection(row.strategy_id, seed);
                      }
                    }}
                    title={setSelectedStrategyForInspection ? 'Open in Inspect Strategy' : undefined}
                  >
                    <td className="py-2 px-4 border-b">{row.strategy_id}</td>
                    <td className="py-2 px-4 border-b">{resolveExecutionFitness(row).toFixed(6)}</td>
                    <td className="py-2 px-4 border-b">{resolveGaFitness(row) === null ? "— (Search not performed)" : resolveGaFitness(row).toFixed(6)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          {comparisonResult.comparison_summary && comparisonResult.comparison_summary.reason && (
            <div className="mt-6 p-4 bg-blue-50 border border-blue-200 rounded-md text-blue-800">
              <h3 className="text-xl font-semibold mb-2">Comparison Insights</h3>
              <p>{comparisonResult.comparison_summary.reason}</p>
            </div>
          )}
        </div>
      )}
    </div>
  );
};

export default CompareStrategies;
