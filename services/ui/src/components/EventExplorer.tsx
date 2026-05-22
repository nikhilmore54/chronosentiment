import React, { useState, useEffect } from 'react';

interface SimEvent {
  sequence_id: number;
  timestamp: number;
  type: string;
  payload: any;
}

interface EventExplorerProps {
  apiBaseUrl?: string;
}

const EventExplorer: React.FC<EventExplorerProps> = ({ apiBaseUrl = 'http://localhost:8000' }) => {
  const [events, setEvents] = useState<SimEvent[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filterType, setFilterType] = useState<string>('');

  useEffect(() => {
    fetch(`${apiBaseUrl}/events`)
      .then(res => {
        if (!res.ok) throw new Error('Failed to fetch events');
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

  const filteredEvents = filterType 
    ? events.filter(e => e.type.toLowerCase().includes(filterType.toLowerCase())) 
    : events;

  if (loading) return <div>Loading events...</div>;
  if (error) return <div style={{ color: 'red' }}>Error: {error}</div>;

  return (
    <div style={{ border: '1px solid #222', padding: '20px', marginTop: '20px', borderRadius: '8px', backgroundColor: '#111' }}>
      <h2 style={{ margin: '0 0 15px 0', fontSize: '18px', fontWeight: 500, color: '#ededed' }}>Event Explorer</h2>
      <div style={{ marginBottom: '15px' }}>
        <input 
          type="text" 
          placeholder="Filter by event type..." 
          value={filterType}
          onChange={(e) => setFilterType(e.target.value)}
          style={{ width: '100%', padding: '8px 12px', borderRadius: '4px', border: '1px solid #333', backgroundColor: '#000', color: '#ededed' }}
        />
      </div>
      <div style={{ maxHeight: '400px', overflowY: 'scroll', background: '#000', color: '#60a5fa', padding: '15px', border: '1px solid #222', borderRadius: '4px', fontFamily: 'monospace', fontSize: '13px' }}>
        <pre style={{ margin: 0 }}>{JSON.stringify(filteredEvents, null, 2)}</pre>
      </div>
    </div>
  );
};

export default EventExplorer;
