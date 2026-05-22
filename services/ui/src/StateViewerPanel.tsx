import React from 'react';
import { SystemState, OrderState } from './types';
import { formatInr } from './money';

interface StateViewerPanelProps {
  systemState: SystemState;
}

const StateViewerPanel: React.FC<StateViewerPanelProps> = ({ systemState }) => {
  if (!systemState) {
    return <div>Loading System State...</div>;
  }

  return (
    <div style={{ border: '1px solid #222', padding: '20px', borderRadius: '8px', overflowY: 'auto', maxHeight: '400px', backgroundColor: '#111' }}>
      <h3 style={{ margin: '0 0 15px 0', fontSize: '18px', fontWeight: 500 }}>System State (Sequence: {systemState.last_sequence_id})</h3>
      
      <h4 style={{ color: '#a1a1aa', fontSize: '14px', textTransform: 'uppercase', letterSpacing: '0.05em' }}>Portfolio</h4>
      <pre style={{ backgroundColor: '#000', padding: '15px', borderRadius: '6px', border: '1px solid #222', color: '#60a5fa', fontSize: '13px', overflowX: 'auto' }}>{JSON.stringify(systemState.portfolio, null, 2)}</pre>

      <h4 style={{ color: '#a1a1aa', fontSize: '14px', textTransform: 'uppercase', letterSpacing: '0.05em', marginTop: '20px' }}>Orders</h4>
      {Object.keys(systemState.orders).length === 0 ? (
        <p style={{ color: '#71717a', fontStyle: 'italic', fontSize: '14px' }}>No active orders.</p>
      ) : (
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))', gap: '15px' }}>
          {Object.values(systemState.orders).map((order: OrderState) => (
            <div key={order.order_id} style={{ border: '1px solid #333', padding: '15px', borderRadius: '6px', backgroundColor: '#0a0a0a', fontSize: '13px', lineHeight: '1.6' }}>
              <strong style={{ color: '#f59e0b', display: 'block', marginBottom: '8px' }}>Order ID: {order.order_id}</strong>
              Status: {order.status}<br/>
              Side: {order.side}<br/>
              Price: {formatInr(order.price)}<br/>
              Total Qty: {order.quantity_total}<br/>
              Filled Qty: {order.quantity_filled}<br/>
              Remaining Qty: {order.quantity_remaining}<br/>
              Queue Ahead: {order.queue_ahead}
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

export default StateViewerPanel;
