/** Shared presentation tokens for Gate5 Operator. */

import type { CSSProperties } from 'react';

export const op = {
  bg: '#0a0c0f',
  panel: '#12161c',
  panelElevated: '#171c24',
  border: 'rgba(232, 236, 241, 0.1)',
  borderStrong: 'rgba(232, 236, 241, 0.18)',
  text: '#e8ecf1',
  muted: '#9aa5b1',
  dim: '#6b7682',
  signal: '#8fbf9f',
  loss: '#d4888a',
  warn: '#c4a574',
  ink: '#1a222c',
  // Aliases for NSE / legacy call sites
  cyan: '#8fbf9f',
  emerald: '#8fbf9f',
  rose: '#d4888a',
  amber: '#c4a574',
  mono: "'IBM Plex Mono', ui-monospace, Menlo, monospace",
  sans: "'IBM Plex Sans', 'Segoe UI', system-ui, sans-serif",
  display: "'Fraunces', 'IBM Plex Sans', Georgia, serif",
} as const;

export const panelStyle: CSSProperties = {
  background: op.panel,
  border: `1px solid ${op.border}`,
  borderRadius: 2,
  padding: '16px 18px',
};

export const labelStyle: CSSProperties = {
  fontSize: 11,
  letterSpacing: '0.12em',
  textTransform: 'uppercase',
  color: op.dim,
  fontFamily: op.sans,
  marginBottom: 8,
  fontWeight: 500,
};

export const valueStyle: CSSProperties = {
  fontFamily: op.mono,
  fontSize: 15,
  color: op.text,
  fontWeight: 500,
};

export const chip = (bg: string, fg = op.text): CSSProperties => ({
  display: 'inline-flex',
  alignItems: 'center',
  gap: 6,
  padding: '4px 9px',
  borderRadius: 2,
  fontSize: 11,
  fontWeight: 600,
  fontFamily: op.mono,
  background: bg,
  color: fg,
  letterSpacing: '0.03em',
  border: `1px solid ${op.border}`,
});

export const thStyle: CSSProperties = {
  padding: '8px 10px',
  fontWeight: 500,
  borderBottom: `1px solid ${op.borderStrong}`,
  color: op.dim,
  fontSize: 10,
  letterSpacing: '0.06em',
  textTransform: 'uppercase',
  whiteSpace: 'nowrap',
};

export const tdStyle: CSSProperties = {
  padding: '8px 10px',
  borderBottom: `1px solid ${op.border}`,
  color: op.text,
  whiteSpace: 'nowrap',
};

export function pnlColor(v: number | null | undefined): string {
  if (v == null || Number.isNaN(v)) return op.dim;
  if (v > 0) return op.signal;
  if (v < 0) return op.loss;
  return op.muted;
}
