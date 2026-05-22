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
    if (type.includes('OrderIntent')) return 'rgba(255, 255, 255, 0.05)'; // Subtle gray
    if (type.includes('Queue') || type.includes('QueueProgression')) return 'rgba(245, 158, 11, 0.1)'; // Amber
    if (type.includes('PartialFill')) return 'rgba(16, 185, 129, 0.1)'; // Green
    if (type.includes('Reject')) return 'rgba(239, 68, 68, 0.1)'; // Red
    return 'transparent';
  };

  const getEventBorderColor = (type: string) => {
    if (type.includes('OrderIntent')) return '#333'; 
    if (type.includes('Queue') || type.includes('QueueProgression')) return '#f59e0b'; 
    if (type.includes('PartialFill')) return '#10b981'; 
    if (type.includes('Reject')) return '#ef4444'; 
    return '#222';
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
  return (
    <div style={{ border: '1px solid #222', padding: '20px', marginBottom: '20px', borderRadius: '8px', backgroundColor: '#111' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '15px' }}>
        <h2 style={{ fontSize: '18px', fontWeight: 500, margin: 0 }}>Timeline Viewer</h2>
        <div>
          <input 
            type="number" 
            placeholder="Go to sequence_id..." 
            value={navSeqId}
            onChange={(e) => setNavSeqId(e.target.value)}
            style={{ padding: '8px 12px', borderRadius: '4px', border: '1px solid #333', backgroundColor: '#000', color: '#ededed' }}
          />
          <button onClick={handleNavigate} style={{ padding: '8px 16px', marginLeft: '10px', backgroundColor: '#2563eb', color: 'white', border: 'none', borderRadius: '4px', cursor: 'pointer' }}>Go</button>
        </div>
      </div>
      <div style={{ maxHeight: '400px', overflowY: 'scroll', background: '#0a0a0a', position: 'relative', borderRadius: '4px', border: '1px solid #222' }}>
        <table style={{ width: '100%', borderCollapse: 'collapse', color: '#ededed', fontSize: '14px' }}>
          <thead style={{ position: 'sticky', top: 0, background: '#171717', zIndex: 1 }}>
            <tr style={{ textAlign: 'left', borderBottom: '1px solid #333' }}>
              <th style={{ padding: '12px', fontWeight: 500, color: '#a1a1aa' }}>Seq ID</th>
              <th style={{ padding: '12px', fontWeight: 500, color: '#a1a1aa' }}>Timestamp</th>
              <th style={{ padding: '12px', fontWeight: 500, color: '#a1a1aa' }}>Event Type</th>
            </tr>
          </thead>
          <tbody>
            {events.map(event => (
              <tr 
                key={event.sequence_id} 
                ref={el => { if (el) eventRefs.current.set(event.sequence_id, el); }}
                style={{ 
                  borderBottom: `1px solid #222`,
                  borderLeft: `3px solid ${getEventBorderColor(event.type)}`,
                  cursor: 'pointer',
                  backgroundColor: highlightedChain.has(event.sequence_id) 
                    ? 'rgba(37, 99, 235, 0.2)' 
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
                    <span style={{ fontSize: '11px', color: '#71717a', marginLeft: '10px' }}>
                      (Parent: {event.parent_sequence_id})
                    </span>
                  }
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <div style={{ marginTop: '15px', fontSize: '12px', color: '#71717a' }}>
        Tip: Click an event to highlight its causal chain.
      </div>
    </div>
  );
};

export default TimelineViewer;
