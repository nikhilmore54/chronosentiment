/** Shared presentation tokens for Gate5 Operator. */

import type { CSSProperties } from 'react';

export const op = {
  bg: '#071018',
  panel: 'rgba(12, 22, 34, 0.92)',
  border: 'rgba(148, 163, 184, 0.16)',
  text: '#e8eef4',
  muted: '#8fa3b5',
  dim: '#5f7386',
  cyan: '#2dd4bf',
  emerald: '#34d399',
  rose: '#fb7185',
  amber: '#fbbf24',
  mono: "'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, monospace",
  sans: "'IBM Plex Sans', 'Segoe UI', system-ui, sans-serif",
  display: "'Fraunces', 'IBM Plex Sans', Georgia, serif",
} as const;

export const panelStyle: CSSProperties = {
  background: op.panel,
  border: `1px solid ${op.border}`,
  borderRadius: 8,
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

export const chip = (bg: string, fg = op.text): CSSProperties => ({
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
