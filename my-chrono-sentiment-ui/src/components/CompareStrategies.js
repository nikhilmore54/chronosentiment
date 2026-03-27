import React, { useState } from 'react';

const CompareStrategies = () => {
  const [strategyIdsInput, setStrategyIdsInput] = useState('');
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

    try {
      const response = await fetch('http://localhost:8000/compare_strategies', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          strategy_ids: strategy_ids,
        }),
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.message || 'Failed to compare strategies');
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
          placeholder="e.g., abc123, xyz789"
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
                  <tr key={row.strategy_id}>
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
