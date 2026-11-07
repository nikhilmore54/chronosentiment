import { useState } from 'react';
import OperatorOverview, { type OperatorWorkspace } from './OperatorOverview';
import CryptoWorkspace from './CryptoWorkspace';
import NseWorkspace from './NseWorkspace';
import { useOperatorState } from './useOperatorState';
import { op } from './operatorStyles';

type Props = {
  onOpenHorizons?: () => void;
};

export default function OperatorShell({ onOpenHorizons }: Props) {
  const [ws, setWs] = useState<OperatorWorkspace>('overview');
  const { state, loading } = useOperatorState(true);

  const navigate = (next: OperatorWorkspace) => {
    if (next === 'horizons') {
      onOpenHorizons?.();
      return;
    }
    setWs(next);
  };

  return (
    <div
      style={{
        background: `radial-gradient(ellipse at 20% 0%, rgba(34,211,238,0.08), transparent 45%),
          radial-gradient(ellipse at 80% 10%, rgba(167,139,250,0.07), transparent 40%),
          ${op.bg}`,
        margin: '-20px',
        padding: 20,
        minHeight: '70vh',
        borderRadius: 8,
      }}
    >
      {ws === 'overview' && (
        <OperatorOverview state={state} loading={loading} onNavigate={navigate} />
      )}
      {ws === 'crypto' && (
        <CryptoWorkspace state={state} onBack={() => setWs('overview')} />
      )}
      {ws === 'nse' && <NseWorkspace state={state} onBack={() => setWs('overview')} />}
    </div>
  );
}
