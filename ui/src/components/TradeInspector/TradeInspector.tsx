import React from 'react';
import type { TradeInspectorViewModel, ExplanationRule } from '../../types/tradeInspector';
import { TradeHeader } from './TradeHeader';
import { DecisionPanel } from './DecisionPanel';
import { ExecutionPanel } from './ExecutionPanel';
import { OutcomePanel } from './OutcomePanel';
import { ExplanationPanel } from './ExplanationPanel';

interface Props {
  model: TradeInspectorViewModel;
}

export const TradeInspector: React.FC<Props> = ({ model }) => {
  const { trade_delta, rules_map } = model;
  
  const resolvedExplanations: ExplanationRule[] = trade_delta.explanations
    .map(id => rules_map[id])
    .filter(Boolean);

  return (
    <div style={{ maxWidth: '800px', margin: '0 auto', padding: '2rem' }}>
      <TradeHeader trade_id={trade_delta.trade_id} strategy={trade_delta.strategy} />
      
      <div className="flex-col gap-lg">
        <DecisionPanel data={trade_delta.signal} />
        
        <ExecutionPanel 
          baseline={trade_delta.baseline} 
          perturbed={trade_delta.perturbed} 
          delta={trade_delta.delta} 
        />
        
        <OutcomePanel delta={trade_delta.delta} />
        
        <ExplanationPanel explanations={resolvedExplanations} />
      </div>
    </div>
  );
};
