import React from 'react';

import type { Recommendation } from '../types';

interface RecommendationCardProps {
  recommendation: Recommendation;
  onAccept: (rec: Recommendation) => void;
  onReject: (rec: Recommendation) => void;
  onModify: (rec: Recommendation) => void;
  onDetails?: (rec: Recommendation) => void;
}

export const RecommendationCard: React.FC<RecommendationCardProps> = ({ recommendation, onAccept, onReject, onModify, onDetails }) => {
  const { constraint_id, severity, explanation, recommended_action, confidence } = recommendation;
  // Defensive handling for missing or unexpected severity values
  const severityStr = severity ? String(severity) : 'low';
  const severityLower = severityStr.toLowerCase();
  const severityColor = severityLower === 'hard' ? 'var(--danger-color)' : '#f59e0b';
  const borderColor = severityLower === 'hard' ? 'rgba(239, 68, 68, 0.15)' : 'rgba(245, 158, 11, 0.15)';
  const bgColor = severityLower === 'hard' ? 'rgba(239, 68, 68, 0.05)' : 'rgba(245, 158, 11, 0.05)';

  return (
    <div style={{
      borderLeft: `4px solid ${severityColor}`,
      backgroundColor: bgColor,
      border: `1px solid ${borderColor}`,
      borderRadius: '8px',
      padding: '1rem',
      marginBottom: '0.75rem',
    }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.5rem' }}>
        <span style={{ fontWeight: 600, fontSize: '0.9rem' }}>{constraint_id}</span>
        <span style={{ color: severityColor, fontWeight: 600 }}>{severityStr.toUpperCase()}</span>
      </div>
      <div style={{ fontSize: '0.85rem', color: 'var(--text-main)', marginBottom: '0.5rem' }}>{explanation}</div>
      <div style={{ fontSize: '0.8rem', color: 'var(--accent-color)', fontStyle: 'italic', marginBottom: '0.75rem' }}>Action: {recommended_action}</div>
      {confidence !== undefined && (
        <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)', marginBottom: '0.5rem' }}>Confidence: {confidence}%</div>
      )}
      <div style={{ display: 'flex', gap: '0.5rem', justifyContent: 'flex-end' }}>
        <button onClick={() => onAccept(recommendation)} style={{ backgroundColor: 'var(--success-color)', color: 'white', border: 'none', padding: '0.4rem 0.8rem', borderRadius: '4px', cursor: 'pointer', fontSize: '0.8rem' }}>Accept</button>
        <button onClick={() => onReject(recommendation)} style={{ backgroundColor: '#f59e0b', color: '#1e1b4b', border: 'none', padding: '0.4rem 0.8rem', borderRadius: '4px', cursor: 'pointer', fontSize: '0.8rem' }}>Reject</button>
        <button onClick={() => onModify(recommendation)} style={{ backgroundColor: 'var(--primary-color)', color: 'white', border: 'none', padding: '0.4rem 0.8rem', borderRadius: '4px', cursor: 'pointer', fontSize: '0.8rem' }}>Modify</button>
        {onDetails && <button onClick={() => onDetails(recommendation)} style={{ backgroundColor: 'var(--accent-color)', color: 'white', border: 'none', padding: '0.4rem 0.8rem', borderRadius: '4px', cursor: 'pointer', fontSize: '0.8rem', marginLeft: '0.5rem' }}>Details</button>}
      </div>
    </div>
  );
};
