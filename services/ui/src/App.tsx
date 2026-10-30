import ErrorBoundary from './ErrorBoundary';
import { useState, useEffect } from 'react';
import TimelineViewer from './components/TimelineViewer';
import EventExplorer from './components/EventExplorer';
import TradeInspector from './TradeInspector';
import ReplayStepper from './ReplayStepper';
import StateViewerPanel from './StateViewerPanel';
import Gate5PaperDashboard from './Gate5PaperDashboard';
import DecisionSurfacePanel from './components/DecisionSurfacePanel';
import ResearchForensicsWorkspace from './components/ResearchForensicsWorkspace';
import MultiHorizonBooksPanel from './components/MultiHorizonBooksPanel';
import { SystemState } from './types';
import { API_BASE_URL } from './config';

interface EventWrapper {
  sequence_id: number;
  timestamp: number;
  type: string;
  parent_sequence_id: number | null;
  payload: any;
}

function App() {
  const [simulationEvents, setSimulationEvents] = useState<EventWrapper[]>([]);
  const [timelineEvents, setTimelineEvents] = useState<EventWrapper[]>([]);
  const [currentSequenceId, setCurrentSequenceId] = useState<number>(0);
  const [systemState, setSystemState] = useState<SystemState | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchTimeline();
    fetchSystemState(0);
  }, []);

  const fetchTimeline = async () => {
    try {
      const response = await fetch(`${API_BASE_URL}/timeline`);
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const data = await response.json();
      const events: EventWrapper[] = data.events || [];
      setTimelineEvents(events);
      setSimulationEvents(events);
    } catch (e: any) {
      console.warn("API server not active (running static Decision Surface mode):", e.message);
    }
  };

  const fetchSystemState = async (seqId: number) => {
    try {
      const response = await fetch(`${API_BASE_URL}/replay/${seqId}`);
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const data: SystemState = await response.json();
      setSystemState(data);
      setCurrentSequenceId(seqId);
    } catch (e: any) {
      console.warn("API server state unavailable (running static Decision Surface mode):", e.message);
    }
  };

  const handleNextEvent = () => {
    if (currentSequenceId < simulationEvents.length - 1) {
      fetchSystemState(currentSequenceId + 1);
    }
  };

  const handlePreviousEvent = () => {
    if (currentSequenceId > 0) {
      fetchSystemState(currentSequenceId - 1);
    }
  };

  const handleReset = () => {
    fetchSystemState(0);
  };

  const [activeTab, setActiveTab] = useState<'surface' | 'gate5' | 'horizons' | 'forensics' | 'observatory' | 'replay' | 'inspector' | 'research'>('horizons');

  return (
    <div style={{ padding: '20px', fontFamily: 'Inter, sans-serif', backgroundColor: '#0a0a0a', color: '#e5e5e5', minHeight: '100vh' }}>
      <header style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '30px', borderBottom: '1px solid #333', paddingBottom: '20px', flexWrap: 'wrap', gap: '15px' }}>
        <div>
          <h1 style={{ margin: 0, fontSize: '24px', fontWeight: 700, letterSpacing: '-0.02em', color: '#38bdf8' }}>ChronoSentiment Single Web Surface</h1>
          <p style={{ margin: '5px 0 0 0', color: '#888', fontSize: '14px' }}>Consolidated Provider Chronology Observatory, Decision Surface & Protection System</p>
        </div>
        <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
          <button 
            id="tab-forensics"
            style={{ padding: '10px 16px', backgroundColor: activeTab === 'forensics' ? '#0ea5e9' : '#1f2937', color: 'white', border: 'none', borderRadius: '6px', cursor: 'pointer', fontWeight: 700, boxShadow: activeTab === 'forensics' ? '0 4px 12px rgba(14,165,233,0.3)' : 'none' }}
            onClick={() => setActiveTab('forensics')}>🔬 Research & Forensics (RP-606-609)</button>
          <button 
            id="tab-surface"
            style={{ padding: '10px 16px', backgroundColor: activeTab === 'surface' ? '#0ea5e9' : '#1f2937', color: 'white', border: 'none', borderRadius: '6px', cursor: 'pointer', fontWeight: 700 }}
            onClick={() => setActiveTab('surface')}>📊 Decision Surface (RP-600/601)</button>
          <button 
            id="tab-gate5"
            style={{ padding: '10px 16px', backgroundColor: activeTab === 'gate5' ? '#0284c7' : '#1f2937', color: 'white', border: 'none', borderRadius: '6px', cursor: 'pointer', fontWeight: 700 }}
            onClick={() => setActiveTab('gate5')}>📄 Paper Trading & Protection</button>
          <button
            id="tab-horizons"
            style={{ padding: '10px 16px', backgroundColor: activeTab === 'horizons' ? '#7c3aed' : '#1f2937', color: 'white', border: 'none', borderRadius: '6px', cursor: 'pointer', fontWeight: 700, boxShadow: activeTab === 'horizons' ? '0 4px 12px rgba(124,58,237,0.35)' : 'none' }}
            onClick={() => setActiveTab('horizons')}>Multi-Horizon Books</button>
          <button 
            id="tab-observatory"
            style={{ padding: '10px 16px', backgroundColor: activeTab === 'observatory' ? '#2563eb' : '#1f2937', color: 'white', border: 'none', borderRadius: '6px', cursor: 'pointer' }}
            onClick={() => setActiveTab('observatory')}>Observatory</button>
          <button 
            id="tab-replay"
            style={{ padding: '10px 16px', backgroundColor: activeTab === 'replay' ? '#2563eb' : '#1f2937', color: 'white', border: 'none', borderRadius: '6px', cursor: 'pointer' }}
            onClick={() => setActiveTab('replay')}>Replay Timeline</button>
          <button 
            id="tab-inspector"
            style={{ padding: '10px 16px', backgroundColor: activeTab === 'inspector' ? '#2563eb' : '#1f2937', color: 'white', border: 'none', borderRadius: '6px', cursor: 'pointer' }}
            onClick={() => setActiveTab('inspector')}>Trade Inspector</button>
          <button 
            id="tab-research"
            style={{ padding: '10px 16px', backgroundColor: activeTab === 'research' ? '#2563eb' : '#1f2937', color: 'white', border: 'none', borderRadius: '6px', cursor: 'pointer' }}
            onClick={() => setActiveTab('research')}>Research Console</button>
        </div>
      </header>

      {error && <div style={{ color: '#ef4444', backgroundColor: '#ef444420', padding: '10px', borderRadius: '4px', marginBottom: '20px' }}>{error}</div>}

      <main style={{ backgroundColor: '#111', padding: '20px', borderRadius: '8px', border: '1px solid #222' }}>
        {activeTab === 'forensics' && (
          <ResearchForensicsWorkspace />
        )}

        {activeTab === 'surface' && (
          <DecisionSurfacePanel />
        )}

        {activeTab === 'gate5' && (
          <Gate5PaperDashboard />
        )}

        {activeTab === 'horizons' && (
          <MultiHorizonBooksPanel />
        )}

        {activeTab === 'observatory' && (
          <div>
            <h2 style={{ margin: '0 0 15px 0', color: '#38bdf8' }}>Provider Synchronization & Chronology Integrity</h2>
            {systemState ? <StateViewerPanel systemState={systemState} /> : <p style={{ color: '#94a3b8' }}>API Server offline. Displaying static chronology Observatory data...</p>}
          </div>
        )}

        {activeTab === 'replay' && (
          <div>
            <h2 style={{ margin: '0 0 15px 0', color: '#38bdf8' }}>Causal Reconstruction Engine</h2>
            <div style={{ marginBottom: '20px', padding: '20px', backgroundColor: '#1a1a1a', borderRadius: '8px' }}>
              <ReplayStepper
                currentSequenceId={currentSequenceId}
                totalEvents={simulationEvents.length}
                onNext={handleNextEvent}
                onPrevious={handlePreviousEvent}
                onReset={handleReset}
              />
            </div>
            <TimelineViewer events={timelineEvents} apiBaseUrl={API_BASE_URL} />
          </div>
        )}

        {activeTab === 'inspector' && (
          <div>
            <h2 style={{ margin: '0 0 15px 0', color: '#38bdf8' }}>Single-Trade Forensic Analysis</h2>
            <ErrorBoundary>
              <TradeInspector apiBaseUrl={API_BASE_URL} />
            </ErrorBoundary>
          </div>
        )}

        {activeTab === 'research' && (
          <div>
            <h2 style={{ margin: '0 0 15px 0', color: '#38bdf8' }}>Long-Horizon Analytics</h2>
            <EventExplorer apiBaseUrl={API_BASE_URL} />
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
