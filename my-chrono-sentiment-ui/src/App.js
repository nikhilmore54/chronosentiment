import React, { useState, useEffect } from 'react';
import RunGA from './components/RunGA';
import StrategyInspector from './components/StrategyInspector';
import CompareStrategies from './components/CompareStrategies';
import GlobalRanking from './components/GlobalRanking';

const TABS = [
  { id: 'run-ga',            label: 'Run GA' },
  { id: 'inspect-strategy',  label: 'Inspect Strategy' },
  { id: 'compare-strategies',label: 'Compare Strategies' },
  { id: 'global-ranking',    label: 'Global Ranking' },
];

function useClock() {
  const [ts, setTs] = useState('');
  useEffect(() => {
    const fmt = () => {
      const p = {};
      new Intl.DateTimeFormat('en-IN', {
        timeZone: 'Asia/Kolkata', hour12: false,
        year: 'numeric', month: '2-digit', day: '2-digit',
        hour: '2-digit', minute: '2-digit', second: '2-digit',
      }).formatToParts(new Date()).forEach(x => { p[x.type] = x.value; });
      setTs(`${p.year}-${p.month}-${p.day} ${p.hour}:${p.minute}:${p.second} IST`);
    };
    fmt();
    const id = setInterval(fmt, 1000);
    return () => clearInterval(id);
  }, []);
  return ts;
}

export default function App() {
  const [activeTab, setActiveTab] = useState('run-ga');
  const [selectedStrategyId, setSelectedStrategyId] = useState(null);
  const [selectedSeed, setSelectedSeed] = useState(null);
  const [selectedStrategyId2, setSelectedStrategyId2] = useState(null);
  const [selectedSeed2, setSelectedSeed2] = useState(null);
  const clock = useClock();

  const handleSetSelectedStrategyForInspection = (strategyId, seed, isSecondStrategy = false) => {
    if (isSecondStrategy) {
      setSelectedStrategyId2(strategyId);
      setSelectedSeed2(seed);
    } else {
      setSelectedStrategyId(strategyId);
      setSelectedSeed(seed);
      setSelectedStrategyId2(null);
      setSelectedSeed2(null);
    }
    setActiveTab('inspect-strategy');
  };

  const renderContent = () => {
    switch (activeTab) {
      case 'run-ga':
        return <RunGA setSelectedStrategyForInspection={handleSetSelectedStrategyForInspection} />;
      case 'inspect-strategy':
        return (
          <StrategyInspector
            strategyId={selectedStrategyId}
            seed={selectedSeed}
            strategyId2={selectedStrategyId2}
            seed2={selectedSeed2}
            onReset={() => {
              setSelectedStrategyId(null);
              setSelectedSeed(null);
              setSelectedStrategyId2(null);
              setSelectedSeed2(null);
            }}
          />
        );
      case 'compare-strategies':
        return <CompareStrategies setSelectedStrategyForInspection={handleSetSelectedStrategyForInspection} />;
      case 'global-ranking':
        return <GlobalRanking />;
      default:
        return <RunGA setSelectedStrategyForInspection={handleSetSelectedStrategyForInspection} />;
    }
  };

  return (
    <div className="cs-app">
      {/* Header */}
      <header className="cs-header">
        <div className="cs-logo">
          <div className="cs-logo-mark">C</div>
          <div className="cs-logo-text">
            <h1>ChronoSentiment</h1>
            <p>Execution Intelligence Platform</p>
          </div>
        </div>
        <div className="cs-clock">{clock}</div>
      </header>

      {/* Tab bar */}
      <nav className="cs-tabs">
        {TABS.map(tab => (
          <button
            key={tab.id}
            className={`cs-tab${activeTab === tab.id ? ' active' : ''}`}
            onClick={() => setActiveTab(tab.id)}
          >
            {tab.label}
          </button>
        ))}
      </nav>

      {/* Content */}
      <main className="cs-main">
        {renderContent()}
      </main>

      {/* Footer */}
      <footer className="cs-footer">
        <span>ChronoSentiment · NSE Execution Intelligence</span>
        <span>{clock}</span>
      </footer>
    </div>
  );
}
