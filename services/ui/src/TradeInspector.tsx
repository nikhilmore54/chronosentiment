import React, { useState, useRef } from 'react';
import { TradeInspectorResponse, ExecutionStep } from './types';
import { formatInr } from './money';

interface TradeInspectorProps {
  apiBaseUrl: string;
}

const TradeInspector: React.FC<TradeInspectorProps> = ({ apiBaseUrl }) => {
  const [orderId, setOrderId] = useState<string>('');
  const [inspection, setInspection] = useState<TradeInspectorResponse | null>(null);
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const [includeChain, setIncludeChain] = useState<boolean>(false); // New state for causal chain toggle
  const currentRequest = useRef(0); // For managing fetch race conditions

  // Helper function to normalize backend enum shape for execution steps
  const normalizeExecution = (execution: any[]): ExecutionStep[] => {
    if (!Array.isArray(execution)) return [];

    return execution
      .map((step) => {
        if (!step || typeof step !== "object") return null;

        const keys = Object.keys(step);
        if (keys.length !== 1) return null;

        const key = keys[0];
        const value = step[key];

        if (!value || typeof value !== "object") return null;

        // Minimal schema validation for specific types
        if (key === "OrderEnteredQueue" && typeof value.queue_ahead !== "number") return null;
        // Add other type-specific validations here if needed, e.g.:
        // if (key === "PartialFillExecution" && (typeof value.filled_qty !== "number" || typeof value.price !== "number")) return null;

        return {
          type: key,
          ...value,
        };
      })
      .filter(Boolean) as ExecutionStep[];
  };

  const handleInspect = async () => {
    if (!orderId) {
      setError('Please enter an Order ID.');
      setInspection(null);
      return;
    }

    setLoading(true);
    setError(null);
    setInspection(null);

    const requestId = Date.now();
    currentRequest.current = requestId; // Update current request ID

    try {
      const response = await fetch(`${apiBaseUrl}/order/${orderId}?include_chain=${includeChain}`);
      
      if (requestId !== currentRequest.current) {
        // A newer request has been made, ignore this one
        return;
      }

      if (!response.ok) {
        const text = await response.text();
        throw new Error(`HTTP error! status: ${response.status} - ${text}`);
      }
      const data: TradeInspectorResponse = await response.json();
      console.log("Trade Inspector Response:", data); // Debug logging

      // Normalize execution steps if needed (backend enum serialization)
      const normalizedExecution = normalizeExecution(data.execution);

      setInspection({ ...data, execution: normalizedExecution });
    } catch (e: any) {
      if (requestId === currentRequest.current) { // Only set error if this is the latest request
        setError("Failed to fetch trade inspection: " + e.message);
        console.error("Failed to fetch trade inspection:", e);
      }
    } finally {
      if (requestId === currentRequest.current) { // Only stop loading if this is the latest request
        setLoading(false);
      }
    }
  };

  return (
    <div style={{ border: '1px solid #ccc', padding: '15px', borderRadius: '5px', overflowY: 'auto', maxHeight: '800px' }}>
      <h2>Trade Inspector</h2>
      <div style={{ marginBottom: '15px' }}>
        <input
          type="text"
          value={orderId}
          onChange={(e) => setOrderId(e.target.value)}
          placeholder="Enter Order ID (e.g., O1)"
          style={{ padding: '8px', marginRight: '10px', width: '180px' }}
        />
        <button onClick={handleInspect} disabled={loading} style={{ padding: '8px 15px' }}>
          {loading ? 'Inspecting...' : 'Inspect'}
        </button>
        <label style={{ marginLeft: '15px' }}>
          <input
            type="checkbox"
            checked={includeChain}
            onChange={(e) => setIncludeChain(e.target.checked)}
            style={{ marginRight: '5px' }}
          />
          Show Full Causal Chain
        </label>
      </div>

      {error && <div style={{ color: 'red', marginBottom: '10px' }}>Error: {error}</div>}

      {!inspection && !loading && !error && <div>Enter an Order ID to inspect a trade.</div>}
      {!inspection && loading && <div>Loading trade inspection...</div>}

      {inspection && (
        <div>
          <h3>Order: {inspection.order_id}</h3>

          <h4>🧠 Decision Layer</h4>
          {inspection.decision ? (
            <p>
              {inspection.decision.side} {inspection.decision.quantity} @ {formatInr(inspection.decision.price)} (Timestamp: {inspection.decision.timestamp})
            </p>
          ) : (
            <p>No decision data available.</p>
          )}


          <h4>⚙️ Execution Timeline</h4>
          <div style={{ marginLeft: '15px', borderLeft: '2px solid #ddd', paddingLeft: '10px' }}>
            {Array.isArray(inspection.execution) ? (
              inspection.execution.map((step, index) => {
                if (!step || typeof step !== 'object' || !('type' in step)) {
                  return <p key={index}>Invalid execution step data</p>;
                }
                const s = step as ExecutionStep;
                switch (s.type) {
                  case 'OrderEnteredQueue':
                    return <p key={s.sequence_id ?? index}>Entered queue ({s.queue_ahead} ahead) (Seq: {s.sequence_id}, TS: {s.timestamp})</p>;
                  case 'QueueProgression':
                    return <p key={s.sequence_id ?? index}>Queue → {s.queue_ahead} (Seq: {s.sequence_id}, TS: {s.timestamp})</p>;
                  case 'PartialFillExecution':
                    return <p key={s.sequence_id ?? index}>Partial Fill: {s.filled_qty} @ {formatInr(s.price)} (Seq: {s.sequence_id}, TS: {s.timestamp})</p>;
                  case 'OrderFilledExecution':
                    return <p key={s.sequence_id ?? index}>Order Filled (Seq: {s.sequence_id}, TS: {s.timestamp})</p>;
                  case 'MarketEventExecution':
                      return (
                          <p key={s.sequence_id ?? index}>Market Event: {s.event_type} {s.quantity} @ {formatInr(s.price)} (Side: {s.side || 'N/A'}) (Seq: {s.sequence_id}, TS: {s.timestamp})</p>
                      );
                }
              })
            ) : (
              <p>No execution steps available or data is malformed.</p>
            )}
          </div>

          <h4>📊 Outcome Layer</h4>
          {inspection.outcome ? (
            <>
              <p>Status: {inspection.outcome.status}</p>
              <p>Filled: {inspection.outcome.filled_qty}</p>
              <p>Remaining: {inspection.outcome.remaining_qty}</p>
              <p>Average Price: {formatInr(inspection.outcome.avg_price)}</p>
            </>
          ) : (
            <p>No outcome data available.</p>
          )}


          {includeChain && inspection.causal_chain && Array.isArray(inspection.causal_chain) && inspection.causal_chain.length > 0 && (
            <>
              <h4>Full Causal Chain</h4>
              <div style={{ marginLeft: '15px', borderLeft: '2px solid #ddd', paddingLeft: '10px' }}>
                {inspection.causal_chain.map((event, index) => {
                  if (!event || typeof event !== 'object') {
                    return <p key={index}>Invalid causal chain event data</p>;
                  }
                  const ev = event as { sequence_id?: number; timestamp?: number; type?: string; payload?: unknown };
                  return (
                  <div key={ev.sequence_id ?? index} style={{ marginBottom: '5px', padding: '5px', border: '1px dotted #ccc', borderRadius: '3px' }}>
                    <strong>Seq {ev.sequence_id ?? "?"}</strong> (t={ev.timestamp ?? "?"}) - {ev.type ?? "Unknown Event"}
                    <pre style={{ margin: '5px 0 0 0', fontSize: '0.8em' }}>{JSON.stringify(ev.payload ?? {}, null, 2)}</pre>
                  </div>
                )})}
              </div>
            </>
          )}
        </div>
      )}
    </div>
  );
};

export default TradeInspector;
