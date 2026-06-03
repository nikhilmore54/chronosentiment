export interface Metadata {
  generated_at: number;
  compiler_version: string;
}

export interface Environment {
  substrate_file: string;
  total_ticks: number;
  latency_injected_ms: number;
  missed_fill_prob: number;
}

export interface StrategyConfig {
  archetype: string;
}

export interface ArtifactSummary {
  what_happened: string;
  where_divergence_started: number | null;
  primary_cause: string;
  severity: string;
}

import type { TimelineContract } from './timeline';

export interface Analytics {
  trades: number;
  fill_rate: number;
  missed_fills: number;
  average_delay_ticks: number;
  average_slippage_bps: number;
  execution_efficiency: number;
  simple_pnl: number;
}

// Global artifact definition
export interface CertifiedArtifact {
  metadata: Metadata;
  environment: Environment;
  strategy: StrategyConfig;
  baseline: any;
  perturbed: any;
  trade_deltas: any[]; // Defined tightly in tradeInspector.ts
  timeline: TimelineContract;
  rules: any[]; // Defined tightly in tradeInspector.ts
  divergence: any;
  analytics: Analytics;
  artifact_summary: ArtifactSummary;
}
