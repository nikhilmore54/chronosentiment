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

  return (
    <div style={{ padding: '20px', fontFamily: 'Arial, sans-serif' }}>
      <h1>ChronoSentiment Visualization UI</h1>
      {error && <div style={{ color: 'red', marginBottom: '20px' }}>{error}</div>}

      <div style={{ display: 'flex', gap: '20px', marginBottom: '20px' }}>
        <div style={{ flex: 1 }}>
          <h2>Replay Stepper</h2>
          <ReplayStepper
            currentSequenceId={currentSequenceId}
            totalEvents={simulationEvents.length}
            onNext={handleNextEvent}
            onPrevious={handlePreviousEvent}
            onReset={handleReset}
          />
          {systemState && <StateViewerPanel systemState={systemState} />}
        </div>

        <div style={{ flex: 1 }}>
          <ErrorBoundary>
            <TradeInspector apiBaseUrl={API_BASE_URL} />
          </ErrorBoundary>
        </div>
      </div>

      <div style={{ display: 'flex', gap: '20px' }}>
        <div style={{ flex: 1 }}>
          <TimelineViewer events={timelineEvents} apiBaseUrl={API_BASE_URL} />
        </div>
        <div style={{ flex: 1 }}>
          <EventExplorer apiBaseUrl={API_BASE_URL} />
        </div>
      </div>
    </div>
  );
}

export default App;
