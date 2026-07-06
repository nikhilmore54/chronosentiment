import type { Recommendation, Decision } from '../types';

/**
 * Simple repository abstraction over browser localStorage.
 * Persists recommendations and decision log so that state survives a page refresh.
 */
export class DecisionRepository {
  private readonly RECS_KEY = 'ultracrew_recommendations';
  private readonly LOG_KEY = 'ultracrew_decision_log';

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
  }
}
