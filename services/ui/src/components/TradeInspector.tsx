import React, { useState } from 'react';

interface InspectionResult {
  decision: {
    order_id: string;
    side: string;
    price: number;
    quantity: number;
    timestamp: number;
  };
  execution: {
    arrival_time: number;
    latency_applied: number;
    queue_ahead_initial: number;
    queue_progression: number[];
    fills: { timestamp: number, qty: number, price: number }[];
    causal_chain: any[];
  };
  outcome: {
    filled_quantity: number;
    remaining_quantity: number;
    average_price: number;
  };
}

interface TradeInspectorProps {
  apiBaseUrl?: string;
}

const TradeInspector: React.FC<TradeInspectorProps> = ({ apiBaseUrl = 'http://localhost:8000' }) => {
  const [orderId, setOrderId] = useState<string>('');
  const [inspection, setInspection] = useState<InspectionResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchInspection = () => {
    if (!orderId) return;
    setLoading(true);
    setInspection(null);
    setError(null);

    fetch(`${apiBaseUrl}/trade/${orderId}/inspect`)
      .then(res => {
        if (!res.ok) throw new Error(`Failed to fetch inspection for order ${orderId}`);
        return res.json();
      })
      .then(data => {
        setInspection(data);
        setLoading(false);
      })
      .catch(err => {
        setError(err.message);
        setLoading(false);
      });
  };

  const getLatencyInfo = () => {
    if (!inspection) return null;
    const intentTs = inspection.decision.timestamp;
    const queueTs = inspection.execution.arrival_time;
    const lastFillTs = inspection.execution.fills.length > 0 
      ? Math.max(...inspection.execution.fills.map(f => f.timestamp))
      : null;
    
    return {
      intentTs,
      queueTs,
      lastFillTs,
      latency: lastFillTs !== null ? lastFillTs - intentTs : queueTs - intentTs
    };
  };

  const latencyInfo = getLatencyInfo();

  return (
    <div style={{ border: '1px solid #222', padding: '20px', marginTop: '20px', borderRadius: '8px', backgroundColor: '#111' }}>
      <h2 style={{ margin: '0 0 15px 0', fontSize: '18px', fontWeight: 500, color: '#ededed' }}>Trade Inspector</h2>
      <div style={{ marginBottom: '15px' }}>
        <input 
          type="text" 
          placeholder="Enter Order ID..." 
          value={orderId}
          onChange={(e) => setOrderId(e.target.value)}
          style={{ width: '200px', padding: '8px 12px', borderRadius: '4px', border: '1px solid #333', backgroundColor: '#000', color: '#ededed' }}
        />
        <button 
          onClick={fetchInspection} 
          disabled={loading || !orderId}
          style={{ padding: '8px 16px', marginLeft: '10px', cursor: 'pointer', backgroundColor: '#2563eb', color: 'white', border: 'none', borderRadius: '4px' }}
        >
          Inspect
        </button>
      </div>

      {loading && <div>Loading inspection...</div>}
      {error && <div style={{ color: 'red' }}>Error: {error}</div>}

      {inspection && (
        <>
          <div style={{ marginBottom: '20px', padding: '20px', background: '#0a0a0a', borderRadius: '6px', textAlign: 'center', border: '1px solid #222' }}>
            <h3 style={{ color: '#ededed', fontSize: '16px', fontWeight: 500, marginTop: 0 }}>Latency Visualization</h3>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '15px', marginTop: '15px' }}>
              <div style={{ padding: '15px', border: '1px solid #333', borderRadius: '6px', background: '#111', color: '#ededed' }}>
                <strong style={{ color: '#a1a1aa', display: 'block', marginBottom: '5px' }}>Intent</strong>
                Time: {latencyInfo?.intentTs}
              </div>
              <span style={{ fontSize: '20px', color: '#71717a' }}>→</span>
              <div style={{ padding: '15px', border: '1px solid #333', borderRadius: '6px', background: '#111', color: '#ededed' }}>
                <strong style={{ color: '#a1a1aa', display: 'block', marginBottom: '5px' }}>Queue Entry</strong>
                Time: {latencyInfo?.queueTs}
              </div>
              <span style={{ fontSize: '20px', color: '#71717a' }}>→</span>
              <div style={{ padding: '15px', border: '1px solid #333', borderRadius: '6px', background: '#111', color: '#ededed' }}>
                <strong style={{ color: '#a1a1aa', display: 'block', marginBottom: '5px' }}>Final Execution</strong>
                Time: {latencyInfo?.lastFillTs ?? 'N/A'}
              </div>
            </div>
            <div style={{ marginTop: '20px', fontWeight: 500, color: '#3b82f6' }}>
              Total Latency: {latencyInfo?.latency} units
            </div>
          </div>

          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: '15px' }}>
            {/* Decision Section */}
            <div style={{ border: '1px solid #222', padding: '15px', borderRadius: '6px', background: '#0a0a0a' }}>
              <h3 style={{ color: '#ededed', fontSize: '14px', marginTop: 0 }}>Decision</h3>
              <pre style={{ fontSize: '12px', background: '#000', padding: '10px', border: '1px solid #333', borderRadius: '4px', color: '#60a5fa', overflowX: 'auto' }}>
                {JSON.stringify(inspection.decision, null, 2)}
              </pre>
            </div>

            {/* Execution Section */}
            <div style={{ border: '1px solid #222', padding: '15px', borderRadius: '6px', background: '#0a0a0a' }}>
              <h3 style={{ color: '#ededed', fontSize: '14px', marginTop: 0 }}>Execution</h3>
              <div style={{ maxHeight: '200px', overflowY: 'scroll', fontSize: '12px', background: '#000', padding: '10px', border: '1px solid #333', borderRadius: '4px', color: '#ededed' }}>
                <strong style={{ color: '#a1a1aa' }}>Initial Queue Ahead:</strong> {inspection.execution.queue_ahead_initial}<br/>
                <strong style={{ color: '#a1a1aa' }}>Latency Applied:</strong> {inspection.execution.latency_applied}<br/>
                <strong style={{ color: '#a1a1aa' }}>Fills:</strong>
                <ul style={{ listStyle: 'none', padding: 0, margin: '5px 0 0 0' }}>
                  {inspection.execution.fills.map((fill, idx) => (
                    <li key={idx} style={{ marginBottom: '5px', paddingBottom: '5px', borderBottom: '1px dotted #333', color: '#10b981' }}>
                      Time {fill.timestamp}: {fill.qty} @ {fill.price}
                    </li>
                  ))}
                </ul>
              </div>
            </div>

            {/* Outcome Section */}
            <div style={{ border: '1px solid #222', padding: '15px', borderRadius: '6px', background: '#0a0a0a' }}>
              <h3 style={{ color: '#ededed', fontSize: '14px', marginTop: 0 }}>Outcome</h3>
              <pre style={{ fontSize: '12px', background: '#000', padding: '10px', border: '1px solid #333', borderRadius: '4px', color: '#f59e0b', overflowX: 'auto' }}>
                {JSON.stringify(inspection.outcome, null, 2)}
              </pre>
            </div>
          </div>
        </>
      )}
    </div>
  );
};

export default TradeInspector;
