import type { Recommendation, Decision } from '../types';
import type { SchedulerDecision, RedistributionLog } from '../workflow/WorkflowTypes';

/**
 * Simple repository abstraction over browser localStorage.
 * Persists recommendations and decision log so that state survives a page refresh.
 * P3:   persists SchedulerDecision records (recommended vs selected option).
 * P3.3: persists RedistributionLog alongside each SchedulerDecision (hard gate 5).
 */
export class DecisionRepository {
  private readonly RECS_KEY = 'ultracrew_recommendations';
  private readonly LOG_KEY = 'ultracrew_decision_log';
  private readonly P3_KEY = 'ultracrew_scheduler_decisions';
  // P3.3: redistribution logs keyed by decision_id — satisfies hard gate 5
  private readonly P3_3_KEY = 'ultracrew_redistribution_logs';

  /** Load persisted recommendations. Returns empty array if none. */
  loadRecommendations(): Recommendation[] {
    const data = window.localStorage.getItem(this.RECS_KEY);
    if (!data) return [];
    try {
      return JSON.parse(data) as Recommendation[];
    } catch {
      return [];
    }
  }

  /** Persist the current recommendations array. */
  saveRecommendations(recs: Recommendation[]): void {
    window.localStorage.setItem(this.RECS_KEY, JSON.stringify(recs));
  }

  /** Load persisted decision log. Returns empty array if none. */
  loadDecisionLog(): Decision[] {
    const data = window.localStorage.getItem(this.LOG_KEY);
    if (!data) return [];
    try {
      return JSON.parse(data) as Decision[];
    } catch {
      return [];
    }
  }

  /** Persist the decision log. */
  saveDecisionLog(log: Decision[]): void {
    window.localStorage.setItem(this.LOG_KEY, JSON.stringify(log));
  }

  /** Clear all persisted data. */
  clear(): void {
    window.localStorage.removeItem(this.RECS_KEY);
    window.localStorage.removeItem(this.LOG_KEY);
    window.localStorage.removeItem(this.P3_KEY);
    window.localStorage.removeItem(this.P3_3_KEY);
  }

  // ── P3: Scheduler decisions (recommended vs selected option) ────────────────

  /** Load all persisted P3 scheduler decisions. */
  loadSchedulerDecisions(): SchedulerDecision[] {
    const data = window.localStorage.getItem(this.P3_KEY);
    if (!data) return [];
    try {
      return JSON.parse(data) as SchedulerDecision[];
    } catch {
      return [];
    }
  }

  /** Append a new P3 scheduler decision record. */
  appendSchedulerDecision(decision: SchedulerDecision): void {
    const existing = this.loadSchedulerDecisions();
    existing.push(decision);
    window.localStorage.setItem(this.P3_KEY, JSON.stringify(existing));
  }

  /** Build and persist a new P3 decision record. Returns the created record. */
  recordSchedulerDecision(recommendedId: string, selectedId: string): SchedulerDecision {
    const record: SchedulerDecision = {
      decision_id: `p3_${Date.now()}_${Math.random().toString(36).slice(2, 7)}`,
      created_at_iso: new Date().toISOString(),
      recommended_id: recommendedId,
      selected_id: selectedId,
      overrode_recommendation: selectedId !== recommendedId,
    };
    this.appendSchedulerDecision(record);
    return record;
  }

  // ── P3.3: Redistribution logs (hard gate 5) ─────────────────────────────────

  /**
   * Persist a redistribution log for the given decision_id.
   * Overwrites any existing log for that decision (idempotent on re-save).
   */
  saveRedistributionLog(decisionId: string, log: RedistributionLog): void {
    const all = this.loadAllRedistributionLogs();
    all[decisionId] = log;
    window.localStorage.setItem(this.P3_3_KEY, JSON.stringify(all));
  }

  /**
   * Load the redistribution log for a specific decision_id.
   * Returns null if no log has been saved for that decision.
   */
  loadRedistributionLog(decisionId: string): RedistributionLog | null {
    const all = this.loadAllRedistributionLogs();
    return all[decisionId] ?? null;
  }

  private loadAllRedistributionLogs(): Record<string, RedistributionLog> {
    const data = window.localStorage.getItem(this.P3_3_KEY);
    if (!data) return {};
    try {
      return JSON.parse(data) as Record<string, RedistributionLog>;
    } catch {
      return {};
    }
  }
}
