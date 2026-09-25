import React, { useState } from 'react';
import { S } from '../styles';

const EXPLANATION_LABELS = ['', 'Not useful', 'Slightly useful', 'Moderately useful', 'Very useful', 'Extremely useful'];

export default function StarRating({ value, onChange, label }) {
  const [hover, setHover] = useState(0);
  return (
    <div>
      {label && <div style={S.recSubLabel}>{label}</div>}
      <div style={S.starRow}>
        {[1, 2, 3, 4, 5].map(n => (
          <span key={n} style={S.star(n <= (hover || value))}
            onClick={() => onChange(n)} onMouseEnter={() => setHover(n)} onMouseLeave={() => setHover(0)}>★</span>
        ))}
        {(hover || value) > 0 && <span style={{ fontSize: '12px', color: '#64748b', marginLeft: '6px' }}>{EXPLANATION_LABELS[hover || value]}</span>}
      </div>
    </div>
  );
}
