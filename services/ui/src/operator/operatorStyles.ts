/** Shared presentation tokens for Gate5 Operator workspaces. */

import type { CSSProperties } from 'react';

export const op = {
  bg: '#0b1220',
  panel: 'rgba(15, 23, 42, 0.85)',
  border: 'rgba(148, 163, 184, 0.18)',
  text: '#e2e8f0',
  muted: '#94a3b8',
  dim: '#64748b',
  cyan: '#22d3ee',
  emerald: '#34d399',
  rose: '#fb7185',
  amber: '#fbbf24',
  violet: '#a78bfa',
  mono: "'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, monospace",
  sans: "'IBM Plex Sans', 'Segoe UI', system-ui, sans-serif",
} as const;

export const panelStyle: CSSProperties = {
  background: op.panel,
  border: `1px solid ${op.border}`,
  borderRadius: 10,
  padding: '14px 16px',
};

export const labelStyle: CSSProperties = {
  fontSize: 11,
  letterSpacing: '0.08em',
  textTransform: 'uppercase',
  color: op.dim,
  fontFamily: op.sans,
  marginBottom: 6,
};

export const valueStyle: CSSProperties = {
  fontFamily: op.mono,
  fontSize: 15,
  color: op.text,
  fontWeight: 600,
};

export const chip = (bg: string, fg = '#0b1220'): CSSProperties => ({
  display: 'inline-flex',
  alignItems: 'center',
  gap: 6,
  padding: '3px 8px',
  borderRadius: 4,
  fontSize: 11,
  fontWeight: 700,
  fontFamily: op.mono,
  background: bg,
  color: fg,
  letterSpacing: '0.04em',
});
