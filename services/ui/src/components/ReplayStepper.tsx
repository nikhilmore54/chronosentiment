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
    <div style={{ border: '1px solid #222', padding: '20px', marginTop: '20px', background: '#111', borderRadius: '8px' }}>
      <h2 style={{ margin: '0 0 15px 0', fontSize: '18px', fontWeight: 500, color: '#ededed' }}>Replay Stepper</h2>
      <div style={{ display: 'flex', gap: '10px', marginBottom: '20px' }}>
        <button onClick={handlePrev} disabled={currentIndex <= 0} style={{ padding: '8px 16px', cursor: 'pointer', background: '#1f2937', color: 'white', border: 'none', borderRadius: '4px' }}>
          ← Previous
        </button>
        <button onClick={handleNext} disabled={currentIndex >= events.length - 1} style={{ padding: '8px 16px', cursor: 'pointer', background: '#2563eb', color: 'white', border: 'none', borderRadius: '4px' }}>
          Next Event →
        </button>
        <button onClick={handleReset} style={{ padding: '8px 16px', cursor: 'pointer', background: '#1f2937', color: 'white', border: 'none', borderRadius: '4px' }}>
          Reset
        </button>
        <div style={{ alignSelf: 'center', marginLeft: '10px', color: '#a1a1aa', fontSize: '14px' }}>
          Step {currentIndex + 1} of {events.length}
        </div>
      </div>

      <div style={{ background: '#0a0a0a', padding: '20px', border: '1px solid #222', minHeight: '150px', borderRadius: '6px' }}>
        {currentEvent ? (
          <div>
            <div style={{ display: 'flex', justifyContent: 'space-between', color: '#ededed' }}>
              <strong style={{ color: '#a1a1aa' }}>Event Type:</strong> <span>{currentEvent.type}</span>
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between', borderBottom: '1px solid #333', paddingBottom: '10px', marginBottom: '10px', color: '#ededed' }}>
              <strong style={{ color: '#a1a1aa' }}>Sequence ID:</strong> <span>{currentEvent.sequence_id}</span>
              <strong style={{ color: '#a1a1aa' }}>Timestamp:</strong> <span>{currentEvent.timestamp}</span>
            </div>
            {currentEvent.parent_sequence_id !== null && (
              <div style={{ fontSize: '12px', color: '#71717a' }}>
                Parent Event: {currentEvent.parent_sequence_id}
              </div>
            )}
            <div style={{ marginTop: '15px' }}>
              <strong style={{ color: '#a1a1aa', fontSize: '14px' }}>Event Payload:</strong>
              <pre style={{ fontSize: '12px', background: '#000', padding: '15px', maxHeight: '150px', overflowY: 'auto', border: '1px solid #222', borderRadius: '4px', color: '#60a5fa' }}>
                {JSON.stringify(currentEvent, null, 2)}
              </pre>
            </div>
          </div>
        ) : (
          <div style={{ textAlign: 'center', padding: '40px', color: '#71717a' }}>
            Click "Next" to begin the sequential replay.
          </div>
        )}
      </div>
    </div>
  );
};

export default ReplayStepper;
