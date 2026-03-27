import React, { useState } from 'react';
import RunGA from './components/RunGA';
import StrategyInspector from './components/StrategyInspector';
import CompareStrategies from './components/CompareStrategies';
import GlobalRanking from './components/GlobalRanking';

function App() {
  const [activeTab, setActiveTab] = useState('run-ga');
  const [selectedStrategyId, setSelectedStrategyId] = useState(null);
  const [selectedSeed, setSelectedSeed] = useState(null);
  const [selectedStrategyId2, setSelectedStrategyId2] = useState(null); // ADDED: For dual strategy mode
  const [selectedSeed2, setSelectedSeed2] = useState(null); // ADDED: For dual strategy mode

  const setSelectedStrategyForInspection = (strategyId, seed, isSecondStrategy = false) => {
    if (isSecondStrategy) {
      setSelectedStrategyId2(strategyId);
      setSelectedSeed2(seed);
    } else {
      setSelectedStrategyId(strategyId);
      setSelectedSeed(seed);
      setSelectedStrategyId2(null); // Clear second strategy if a new first strategy is selected
      setSelectedSeed2(null);
    }
    setActiveTab('inspect-strategy');
  };

  const renderContent = () => {
    switch (activeTab) {
      case 'run-ga':
        return <RunGA setSelectedStrategyForInspection={setSelectedStrategyForInspection} />;
      case 'inspect-strategy':
        return (
          <StrategyInspector
            strategyId={selectedStrategyId}
            seed={selectedSeed}
            strategyId2={selectedStrategyId2} // ADDED: Pass second strategy ID
            seed2={selectedSeed2} // ADDED: Pass second strategy seed
            onReset={() => {
              setSelectedStrategyId(null);
              setSelectedSeed(null);
              setSelectedStrategyId2(null); // ADDED: Reset second strategy as well
              setSelectedSeed2(null);
            }}
          />
        );
      case 'compare-strategies':
        return <CompareStrategies setSelectedStrategyForInspection={setSelectedStrategyForInspection} />; // MODIFIED: Pass handler to CompareStrategies
      case 'global-ranking':
        return <GlobalRanking />;
      default:
        return <RunGA setSelectedStrategyForInspection={setSelectedStrategyForInspection} />;
    }
  };

  return (
    <div className="min-h-screen bg-gray-100 p-4 font-mono">
      <h1 className="text-3xl font-bold text-center mb-6 text-gray-800">ChronoSentiment UI</h1>
      <div className="flex justify-center mb-6">
        <button
          className={`px-4 py-2 mx-2 rounded-md ${activeTab === 'run-ga' ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-700'}`}
          onClick={() => setActiveTab('run-ga')}
        >
          Run GA
        </button>
        <button
          className={`px-4 py-2 mx-2 rounded-md ${activeTab === 'inspect-strategy' ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-700'}`}
          onClick={() => setActiveTab('inspect-strategy')}
        >
          Inspect Strategy
        </button>
        <button
          className={`px-4 py-2 mx-2 rounded-md ${activeTab === 'compare-strategies' ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-700'}`}
          onClick={() => setActiveTab('compare-strategies')}
        >
          Compare Strategies
        </button>
        <button
          className={`px-4 py-2 mx-2 rounded-md ${activeTab === 'global-ranking' ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-700'}`}
          onClick={() => setActiveTab('global-ranking')}
        >
          Global Ranking
        </button>
      </div>
      <div className="bg-white p-6 rounded-lg shadow-md max-w-4xl mx-auto">
        {renderContent()}
      </div>
    </div>
  );
}

export default App;
