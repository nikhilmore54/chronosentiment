import React, { useState, useEffect } from 'react';

interface SimEvent {
  sequence_id: number;
  timestamp: number;
  type: string;
  parent_sequence_id: number | null;
  payload: any;
}

interface ReplayStepperProps {
  apiBaseUrl?: string;
}

const ReplayStepper: React.FC<ReplayStepperProps> = ({ apiBaseUrl = 'http://localhost:8000' }) => {
  const [events, setEvents] = useState<SimEvent[]>([]);
  const [currentIndex, setCurrentIndex] = useState<number>(-1);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetch(`${apiBaseUrl}/timeline`)
      .then(res => {
        if (!res.ok) throw new Error('Failed to fetch timeline');
        return res.json();
      })
      .then(data => {
        setEvents(data);
        setLoading(false);
      })
      .catch(err => {
        setError(err.message);
        setLoading(false);
      });
  }, [apiBaseUrl]);

  const handleNext = () => {
    if (currentIndex < events.length - 1) {
      setCurrentIndex(currentIndex + 1);
    }
  };

  const handlePrev = () => {
    if (currentIndex > 0) {
      setCurrentIndex(currentIndex - 1);
    }
  };

  const handleReset = () => {
    setCurrentIndex(-1);
  };

  if (loading) return <div>Loading replay stepper...</div>;
  if (error) return <div style={{ color: 'red' }}>Error: {error}</div>;

  const currentEvent = currentIndex >= 0 ? events[currentIndex] : null;

  return (
    <div style={{ border: '1px solid #ccc', padding: '10px', marginTop: '20px', background: '#f0f2f5' }}>
      <h2>Replay Stepper</h2>
      <div style={{ display: 'flex', gap: '10px', marginBottom: '15px' }}>
        <button onClick={handlePrev} disabled={currentIndex <= 0} style={{ padding: '5px 15px', cursor: 'pointer' }}>
          ← Previous
        </button>
        <button onClick={handleNext} disabled={currentIndex >= events.length - 1} style={{ padding: '5px 15px', cursor: 'pointer', background: '#1890ff', color: 'white', border: 'none' }}>
          Next Event →
        </button>
        <button onClick={handleReset} style={{ padding: '5px 15px', cursor: 'pointer' }}>
          Reset
        </button>
        <div style={{ alignSelf: 'center', marginLeft: '10px' }}>
          Step {currentIndex + 1} of {events.length}
        </div>
      </div>

      <div style={{ background: 'white', padding: '15px', border: '1px solid #ddd', minHeight: '150px' }}>
        {currentEvent ? (
          <div>
            <div style={{ display: 'flex', justifyContent: 'space-between' }}>
              <strong>Event Type:</strong> <span>{currentEvent.type}</span>
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between', borderBottom: '1px solid #eee', paddingBottom: '5px', marginBottom: '5px' }}>
              <strong>Sequence ID:</strong> <span>{currentEvent.sequence_id}</span>
              <strong>Timestamp:</strong> <span>{currentEvent.timestamp}</span>
            </div>
            {currentEvent.parent_sequence_id !== null && (
              <div style={{ fontSize: '12px', color: '#888' }}>
                Parent Event: {currentEvent.parent_sequence_id}
              </div>
            )}
            <div style={{ marginTop: '10px' }}>
              <strong>Event Payload:</strong>
              <pre style={{ fontSize: '11px', background: '#f4f4f4', padding: '5px', maxHeight: '150px', overflowY: 'auto' }}>
                {JSON.stringify(currentEvent, null, 2)}
              </pre>
            </div>
          </div>
        ) : (
          <div style={{ textAlign: 'center', padding: '40px', color: '#999' }}>
            Click "Next" to begin the sequential replay.
          </div>
        )}
      </div>
    </div>
  );
};

export default ReplayStepper;
