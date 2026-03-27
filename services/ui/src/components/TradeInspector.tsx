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
    <div style={{ border: '1px solid #ccc', padding: '10px', marginTop: '20px' }}>
      <h2>Trade Inspector</h2>
      <div style={{ marginBottom: '10px' }}>
        <input 
          type="text" 
          placeholder="Enter Order ID..." 
          value={orderId}
          onChange={(e) => setOrderId(e.target.value)}
          style={{ width: '200px', padding: '5px' }}
        />
        <button 
          onClick={fetchInspection} 
          disabled={loading || !orderId}
          style={{ padding: '5px 10px', marginLeft: '10px', cursor: 'pointer' }}
        >
          Inspect
        </button>
      </div>

      {loading && <div>Loading inspection...</div>}
      {error && <div style={{ color: 'red' }}>Error: {error}</div>}

      {inspection && (
        <>
          <div style={{ marginBottom: '20px', padding: '15px', background: '#f0f2f5', borderRadius: '4px', textAlign: 'center' }}>
            <h3>Latency Visualization</h3>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '10px', marginTop: '10px' }}>
              <div style={{ padding: '10px', border: '1px solid #ddd', borderRadius: '4px', background: 'white' }}>
                <strong>Intent</strong><br/>
                Time: {latencyInfo?.intentTs}
              </div>
              <span style={{ fontSize: '20px' }}>→</span>
              <div style={{ padding: '10px', border: '1px solid #ddd', borderRadius: '4px', background: 'white' }}>
                <strong>Queue Entry</strong><br/>
                Time: {latencyInfo?.queueTs}
              </div>
              <span style={{ fontSize: '20px' }}>→</span>
              <div style={{ padding: '10px', border: '1px solid #ddd', borderRadius: '4px', background: 'white' }}>
                <strong>Final Execution</strong><br/>
                Time: {latencyInfo?.lastFillTs ?? 'N/A'}
              </div>
            </div>
            <div style={{ marginTop: '15px', fontWeight: 'bold', color: '#1890ff' }}>
              Total Latency: {latencyInfo?.latency} units
            </div>
          </div>

          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: '15px' }}>
            {/* Decision Section */}
            <div style={{ border: '1px solid #ddd', padding: '10px' }}>
              <h3>Decision</h3>
              <pre style={{ fontSize: '12px', background: '#f4f4f4', padding: '5px' }}>
                {JSON.stringify(inspection.decision, null, 2)}
              </pre>
            </div>

            {/* Execution Section */}
            <div style={{ border: '1px solid #ddd', padding: '10px' }}>
              <h3>Execution</h3>
              <div style={{ maxHeight: '200px', overflowY: 'scroll', fontSize: '12px', background: '#f4f4f4', padding: '5px' }}>
                <strong>Initial Queue Ahead:</strong> {inspection.execution.queue_ahead_initial}<br/>
                <strong>Latency Applied:</strong> {inspection.execution.latency_applied}<br/>
                <strong>Fills:</strong>
                <ul style={{ listStyle: 'none', padding: 0 }}>
                  {inspection.execution.fills.map((fill, idx) => (
                    <li key={idx} style={{ marginBottom: '5px', borderBottom: '1px dotted #ccc' }}>
                      Time {fill.timestamp}: {fill.qty} @ {fill.price}
                    </li>
                  ))}
                </ul>
              </div>
            </div>

            {/* Outcome Section */}
            <div style={{ border: '1px solid #ddd', padding: '10px' }}>
              <h3>Outcome</h3>
              <pre style={{ fontSize: '12px', background: '#f4f4f4', padding: '5px' }}>
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
