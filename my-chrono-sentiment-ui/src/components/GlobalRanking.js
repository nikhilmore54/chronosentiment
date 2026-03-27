import React, { useState, useEffect } from 'react';

function safeDisplay(value, digits = 2) {
  if (value === undefined || value === null) {
    return "N/A";
  }
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

const GlobalRanking = () => {
  const [ranking, setRanking] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  const fetchRanking = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetch('http://localhost:8000/ga/global-ranking');
      if (!response.ok) {
        throw new Error('Failed to fetch global ranking');
      }
      const data = await response.json();
      console.log("Global Ranking API response:", data);
      if (data && data.length > 0) {
          console.log("ROW 0:", data[0]);
      }
      setRanking(data);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchRanking();
  }, []);

  return (
    <div className="p-4">
      <div className="flex justify-between items-center mb-6">
        <h2 className="text-2xl font-semibold">Global Strategy Ranking</h2>
        <button
          className="bg-green-500 hover:bg-green-700 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline"
          onClick={fetchRanking}
          disabled={loading}
        >
          {loading ? 'Refreshing...' : 'Refresh Ranking'}
        </button>
      </div>

      {error && <p className="text-red-500 mb-4">Error: {error}</p>}

      <div className="overflow-x-auto">
        <table className="min-w-full bg-white border border-gray-300">
          <thead>
            <tr className="bg-gray-100">
              <th className="py-2 px-4 border-b text-left">Strategy</th>
              <th className="py-2 px-4 border-b text-right">Avg</th>
              <th className="py-2 px-4 border-b text-right">Std</th>
              <th className="py-2 px-4 border-b text-right">Execution Fitness</th>
              <th className="py-2 px-4 border-b text-right">GA Fitness</th>
              <th className="py-2 px-4 border-b text-left">Type</th>
            </tr>
          </thead>
          <tbody>
            {ranking.length > 0 ? (
              ranking.sort((a, b) => (resolveExecutionFitness(b) ?? 0) - (resolveExecutionFitness(a) ?? 0)).map((row, index) => (
                <tr key={row.strategy_id} className={index % 2 === 0 ? 'bg-white' : 'bg-gray-50'}>
                  <td className="py-2 px-4 border-b text-sm font-mono truncate max-w-xs" title={row.strategy_id}>
                    {row.strategy_id}
                  </td>
                  <td className="py-2 px-4 border-b text-right font-mono">
                    {safeDisplay(row.avg, 6)}
                  </td>
                  <td className="py-2 px-4 border-b text-right font-mono">
                    {safeDisplay(row.std, 6)}
                  </td>
                  <td className="py-2 px-4 border-b text-right font-mono font-bold text-blue-600">
                    {safeDisplay(resolveExecutionFitness(row), 6)}
                  </td>
                  <td className="py-2 px-4 border-b text-right font-mono">
                    {resolveGaFitness(row) === undefined ? "— (Search not performed)" : safeDisplay(resolveGaFitness(row), 6)}
                  </td>
                  <td className="py-2 px-4 border-b">
                    <span className={`px-2 py-1 rounded text-xs font-semibold ${
                      row.classification === 'Stable' ? 'bg-green-100 text-green-800' :
                      row.classification === 'Volatile' ? 'bg-yellow-100 text-yellow-800' :
                      row.classification === 'Fragile' ? 'bg-red-100 text-red-800' :
                      'bg-gray-100 text-gray-800'
                    }`}>
                      {row.classification}
                    </span>
                  </td>
                </tr>
              ))
            ) : (
              <tr>
                <td colSpan="6" className="py-4 text-center text-gray-500">
                  {loading ? 'Loading ranking data...' : 'No ranking data available. Run multi-asset evaluation.'}
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
};

export default GlobalRanking;
