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
    <div style={{ border: '1px solid #ccc', padding: '15px', borderRadius: '5px', overflowY: 'auto', maxHeight: '400px' }}>
      <h3>System State (Sequence: {systemState.last_sequence_id})</h3>
      
      <h4>Portfolio</h4>
      <pre>{JSON.stringify(systemState.portfolio, null, 2)}</pre>

      <h4>Orders</h4>
      {Object.keys(systemState.orders).length === 0 ? (
        <p>No active orders.</p>
      ) : (
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))', gap: '10px' }}>
          {Object.values(systemState.orders).map((order: OrderState) => (
            <div key={order.order_id} style={{ border: '1px solid #eee', padding: '10px', borderRadius: '3px' }}>
              <strong>Order ID: {order.order_id}</strong><br/>
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
