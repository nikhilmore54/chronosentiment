import React, { useState } from 'react';
import type { RuleSet } from './WorkflowTypes';
import { RULE_PRESETS } from './WorkflowTypes';
import { primaryBtnStyle, ghostBtnStyle, accentGhostBtnStyle, textareaStyle } from './WorkflowComponents';

export const SelectRules: React.FC<{
  selectedRuleId: string;
  onSelect: (id: string, payload: object) => void;
  onNext: () => void;
  onBack: () => void;
}> = ({ selectedRuleId, onSelect, onNext, onBack }) => {
  const [customJson, setCustomJson] = useState('');
  const [customError, setCustomError] = useState('');

  const handleCustomApply = () => {
    try {
      const parsed = JSON.parse(customJson);
      onSelect('custom', parsed);
      setCustomError('');
    } catch {
      setCustomError('Invalid JSON. Please check your syntax.');
    }
  };

  const RadioDot: React.FC<{ active: boolean }> = ({ active }) => (
    <div style={{
      width: '18px', height: '18px', borderRadius: '50%', flexShrink: 0,
      border: `2px solid ${active ? 'var(--accent-color)' : 'var(--border-color)'}`,
      backgroundColor: active ? 'var(--accent-color)' : 'transparent',
    }} />
  );

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
      <div>
        <h3 style={{ margin: '0 0 0.5rem 0', color: 'var(--text-main)' }}>Select Rule Set</h3>
        <p style={{ margin: 0, color: 'var(--text-muted)', fontSize: '0.9rem' }}>
          Choose the scheduling rules that apply to your organisation.
        </p>
      </div>

      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
        {RULE_PRESETS.map((preset: RuleSet) => (
          <div
            key={preset.id}
            onClick={() => onSelect(preset.id, preset.payload)}
            style={{
              border: `2px solid ${selectedRuleId === preset.id ? 'var(--accent-color)' : 'var(--border-color)'}`,
              borderRadius: '8px',
              padding: '1rem 1.25rem',
              cursor: 'pointer',
              backgroundColor: selectedRuleId === preset.id ? 'rgba(56, 189, 248, 0.06)' : 'var(--panel-bg)',
              transition: 'all 0.15s',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
              <RadioDot active={selectedRuleId === preset.id} />
              <div>
                <div style={{ fontWeight: 700, color: 'var(--text-main)', marginBottom: '0.2rem' }}>{preset.label}</div>
                <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>{preset.description}</div>
              </div>
            </div>
          </div>
        ))}

        <div style={{
          border: `2px solid ${selectedRuleId === 'custom' ? 'var(--accent-color)' : 'var(--border-color)'}`,
          borderRadius: '8px',
          padding: '1rem 1.25rem',
          backgroundColor: selectedRuleId === 'custom' ? 'rgba(56, 189, 248, 0.06)' : 'var(--panel-bg)',
        }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem', marginBottom: '0.75rem' }}>
            <RadioDot active={selectedRuleId === 'custom'} />
            <div style={{ fontWeight: 700, color: 'var(--text-main)' }}>Custom JSON</div>
          </div>
          <textarea
            value={customJson}
            onChange={e => setCustomJson(e.target.value)}
            placeholder={'{\n  "max_consecutive_working_days": 5,\n  "min_consecutive_days_off": 2\n}'}
            style={{ ...textareaStyle, height: '100px', marginBottom: '0.5rem' }}
          />
          {customError && (
            <div style={{ color: 'var(--danger-color)', fontSize: '0.8rem', marginBottom: '0.5rem' }}>⚠ {customError}</div>
          )}
          <button onClick={handleCustomApply} style={accentGhostBtnStyle}>Apply Custom Rules</button>
        </div>
      </div>

      <div style={{ display: 'flex', justifyContent: 'space-between' }}>
        <button onClick={onBack} style={ghostBtnStyle}>← Back</button>
        <button
          onClick={onNext}
          disabled={!selectedRuleId}
          style={{ ...primaryBtnStyle, opacity: !selectedRuleId ? 0.4 : 1, cursor: !selectedRuleId ? 'not-allowed' : 'pointer' }}
        >
          Next: Generate Schedule →
        </button>
      </div>
    </div>
  );
};