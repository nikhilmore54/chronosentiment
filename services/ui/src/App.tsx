import ErrorBoundary from './ErrorBoundary';
import { useState, useEffect } from 'react';
import TimelineViewer from './components/TimelineViewer';
import EventExplorer from './components/EventExplorer';
import TradeInspector from './TradeInspector';
import ReplayStepper from './ReplayStepper'; // New component
import StateViewerPanel from './StateViewerPanel'; // New component
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
    fetchSystemState(0); // Fetch initial state for sequence 0
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
      setSimulationEvents(events); // For ReplayStepper
    } catch (e: any) {
      setError("Failed to fetch timeline: " + e.message);
      console.error("Failed to fetch timeline:", e);
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
      setError("Failed to fetch system state: " + e.message);
      console.error("Failed to fetch system state:", e);
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

  const [activeTab, setActiveTab] = useState<'observatory' | 'replay' | 'inspector' | 'research'>('observatory');

  return (
    <div style={{ padding: '20px', fontFamily: 'Inter, sans-serif', backgroundColor: '#0a0a0a', color: '#e5e5e5', minHeight: '100vh' }}>
      <header style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '30px', borderBottom: '1px solid #333', paddingBottom: '20px' }}>
        <div>
          <h1 style={{ margin: 0, fontSize: '24px', fontWeight: 600 }}>ChronoSentiment</h1>
          <p style={{ margin: '5px 0 0 0', color: '#888', fontSize: '14px' }}>Provider Chronology Observatory</p>
        </div>
        <div style={{ display: 'flex', gap: '10px' }}>
          <button 
            style={{ padding: '10px 20px', backgroundColor: activeTab === 'observatory' ? '#2563eb' : '#1f2937', color: 'white', border: 'none', borderRadius: '6px', cursor: 'pointer' }}
            onClick={() => setActiveTab('observatory')}>Observatory</button>
          <button 
            style={{ padding: '10px 20px', backgroundColor: activeTab === 'replay' ? '#2563eb' : '#1f2937', color: 'white', border: 'none', borderRadius: '6px', cursor: 'pointer' }}
            onClick={() => setActiveTab('replay')}>Replay Timeline</button>
          <button 
            style={{ padding: '10px 20px', backgroundColor: activeTab === 'inspector' ? '#2563eb' : '#1f2937', color: 'white', border: 'none', borderRadius: '6px', cursor: 'pointer' }}
            onClick={() => setActiveTab('inspector')}>Trade Inspector</button>
          <button 
            style={{ padding: '10px 20px', backgroundColor: activeTab === 'research' ? '#2563eb' : '#1f2937', color: 'white', border: 'none', borderRadius: '6px', cursor: 'pointer' }}
            onClick={() => setActiveTab('research')}>Research Console</button>
        </div>
      </header>

      {error && <div style={{ color: '#ef4444', backgroundColor: '#ef444420', padding: '10px', borderRadius: '4px', marginBottom: '20px' }}>{error}</div>}

      <main style={{ backgroundColor: '#111', padding: '20px', borderRadius: '8px', border: '1px solid #222' }}>
        {activeTab === 'observatory' && (
          <div>
            <h2>Provider Synchronization & Chronology Integrity</h2>
            {systemState ? <StateViewerPanel systemState={systemState} /> : <p>Loading state...</p>}
          </div>
        )}

        {activeTab === 'replay' && (
          <div>
            <h2>Causal Reconstruction Engine</h2>
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
            <h2>Single-Trade Forensic Analysis</h2>
            <ErrorBoundary>
              <TradeInspector apiBaseUrl={API_BASE_URL} />
            </ErrorBoundary>
          </div>
        )}

        {activeTab === 'research' && (
          <div>
            <h2>Long-Horizon Analytics</h2>
            <EventExplorer apiBaseUrl={API_BASE_URL} />
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
