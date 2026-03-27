import React, { useState, useEffect, useRef } from 'react';

interface SimEvent {
  sequence_id: number;
  timestamp: number;
  type: string;
  parent_sequence_id: number | null;
  payload: any;
}

interface TimelineViewerProps {
  events?: SimEvent[];
  apiBaseUrl?: string;
}

const TimelineViewer: React.FC<TimelineViewerProps> = ({ events: initialEvents, apiBaseUrl = 'http://localhost:8000' }) => {
  const [events, setEvents] = useState<SimEvent[]>(initialEvents || []);
  const [loading, setLoading] = useState(!initialEvents);
  const [error, setError] = useState<string | null>(null);
  const [highlightedChain, setHighlightedChain] = useState<Set<number>>(new Set());
  const [navSeqId, setNavSeqId] = useState<string>('');
  const eventRefs = useRef<Map<number, HTMLTableRowElement>>(new Map());

  useEffect(() => {
    if (initialEvents) {
      setEvents(initialEvents);
      setLoading(false);
      return;
    }

    fetch(`${apiBaseUrl}/timeline`)
      .then(res => {
        if (!res.ok) throw new Error('Failed to fetch timeline');
        return res.json();
      })
      .then(data => {
        setEvents(data.events || []);
        setLoading(false);
      })
      .catch(err => {
        setError(err.message);
        setLoading(false);
      });
  }, [initialEvents, apiBaseUrl]);

  const getEventColor = (type: string) => {
    if (type.includes('OrderIntent')) return '#f0f0f0'; // Gray
    if (type.includes('Queue') || type.includes('QueueProgression')) return '#fffbe6'; // Yellow
    if (type.includes('PartialFill')) return '#f6ffed'; // Green
    if (type.includes('Reject')) return '#fff1f0'; // Red
    return 'transparent';
  };

  const getEventBorderColor = (type: string) => {
    if (type.includes('OrderIntent')) return '#d9d9d9'; 
    if (type.includes('Queue') || type.includes('QueueProgression')) return '#ffe58f'; 
    if (type.includes('PartialFill')) return '#b7eb8f'; 
    if (type.includes('Reject')) return '#ffa39e'; 
    return '#eee';
  };

  const handleEventClick = (clickedEvent: SimEvent) => {
    const chain = new Set<number>();
    let current: number | null = clickedEvent.sequence_id;
    
    // Trace up the chain
    while (current !== null) {
      chain.add(current);
      const ev = events.find(e => e.sequence_id === current);
      current = ev?.parent_sequence_id || null;
    }

    // Trace down the chain (optional but helpful for full causal visibility)
    const findChildren = (parentId: number) => {
      events.forEach(e => {
        if (e.parent_sequence_id === parentId && !chain.has(e.sequence_id)) {
          chain.add(e.sequence_id);
          findChildren(e.sequence_id);
        }
      });
    };
    findChildren(clickedEvent.sequence_id);

    setHighlightedChain(chain);
    console.log('Causal Chain:', Array.from(chain).sort((a, b) => a - b));
  };

  const handleNavigate = () => {
    const seqId = parseInt(navSeqId);
    if (!isNaN(seqId)) {
      const element = eventRefs.current.get(seqId);
      if (element) {
        element.scrollIntoView({ behavior: 'smooth', block: 'center' });
        const ev = events.find(e => e.sequence_id === seqId);
        if (ev) handleEventClick(ev);
      }
    }
  };

  if (loading) return <div>Loading timeline...</div>;
  if (error) return <div style={{ color: 'red' }}>Error: {error}</div>;

  return (
    <div style={{ border: '1px solid #ccc', padding: '10px', marginBottom: '20px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <h2>Timeline Viewer</h2>
        <div>
          <input 
            type="number" 
            placeholder="Go to sequence_id..." 
            value={navSeqId}
            onChange={(e) => setNavSeqId(e.target.value)}
            style={{ padding: '5px' }}
          />
          <button onClick={handleNavigate} style={{ padding: '5px 10px', marginLeft: '5px' }}>Go</button>
        </div>
      </div>
      <div style={{ maxHeight: '400px', overflowY: 'scroll', background: '#f9f9f9', position: 'relative' }}>
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead style={{ position: 'sticky', top: 0, background: '#eee', zIndex: 1 }}>
            <tr style={{ textAlign: 'left', borderBottom: '2px solid #ddd' }}>
              <th style={{ padding: '8px' }}>Seq ID</th>
              <th style={{ padding: '8px' }}>Timestamp</th>
              <th style={{ padding: '8px' }}>Event Type</th>
            </tr>
          </thead>
          <tbody>
            {events.map(event => (
              <tr 
                key={event.sequence_id} 
                ref={el => { if (el) eventRefs.current.set(event.sequence_id, el); }}
                style={{ 
                  borderBottom: `1px solid ${getEventBorderColor(event.type)}`,
                  cursor: 'pointer',
                  backgroundColor: highlightedChain.has(event.sequence_id) 
                    ? '#bae7ff' 
                    : getEventColor(event.type),
                  transition: 'background-color 0.2s'
                }}
                onClick={() => handleEventClick(event)}
              >
                <td style={{ padding: '8px' }}>{event.sequence_id}</td>
                <td style={{ padding: '8px' }}>{event.timestamp}</td>
                <td style={{ padding: '8px' }}>
                  {event.type}
                  {highlightedChain.has(event.sequence_id) && event.parent_sequence_id !== null && 
                    <span style={{ fontSize: '10px', color: '#888', marginLeft: '10px' }}>
                      (Parent: {event.parent_sequence_id})
                    </span>
                  }
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <div style={{ marginTop: '10px', fontSize: '12px', color: '#666' }}>
        Tip: Click an event to highlight its causal chain.
      </div>
    </div>
  );
};

export default TimelineViewer;
