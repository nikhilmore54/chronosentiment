// DeviceSessionService.ts — Client service for server-side device session persistence
// Relies primarily on HttpOnly browser cookies with credentials: 'include'.
// Retains X-Device-Token strictly for explicit programmatic/non-browser callers.
// Includes monotonic sequence revisions and schema version protection.

export interface DeviceSessionResponse<T = any> {
  device_token: string;
  schema_version?: number;
  revision?: number;
  updated_at: string | null;
  state: T | null;
}

export interface SaveSessionResult {
  status: string;
  device_token: string;
  schema_version?: number;
  revision?: number;
  updated_at: string;
}

let clientRevision = 0;
let saveTimeout: ReturnType<typeof setTimeout> | null = null;
let currentSyncListeners: Array<(status: 'idle' | 'saving' | 'saved' | 'error') => void> = [];

export class DeviceSessionService {
  /** Returns the current monotonic revision number for this client tab */
  static getRevision(): number {
    return clientRevision;
  }

  /** Reset the client revision (e.g. upon starting a new assessment or clearing session) */
  static resetRevision(): void {
    clientRevision = 0;
  }

  /** Subscribe to sync status changes ('idle' | 'saving' | 'saved' | 'error') */
  static onSyncStatusChange(callback: (status: 'idle' | 'saving' | 'saved' | 'error') => void): () => void {
    currentSyncListeners.push(callback);
    return () => {
      currentSyncListeners = currentSyncListeners.filter(l => l !== callback);
    };
  }

  private static notifyStatus(status: 'idle' | 'saving' | 'saved' | 'error'): void {
    for (const listener of currentSyncListeners) {
      listener(status);
    }
  }

  /**
   * Load device workflow session from backend.
   * Relies on the HttpOnly cookie transmitted automatically with credentials: 'include'.
   * An optional explicitToken can be provided for programmatic/non-browser clients.
   */
  static async loadSession<T = any>(explicitToken?: string): Promise<DeviceSessionResponse<T> | null> {
    try {
      const headers: Record<string, string> = {};
      if (explicitToken) {
        headers['x-device-token'] = explicitToken;
      }

      const res = await fetch('/api/session/state', {
        method: 'GET',
        headers,
        credentials: 'include',
      });

      if (!res.ok) {
        return null;
      }

      const data: DeviceSessionResponse<T> = await res.json();
      if (typeof data.revision === 'number') {
        clientRevision = Math.max(clientRevision, data.revision);
      }
      return data;
    } catch (err) {
      console.warn('Failed to load device session from server:', err);
      return null;
    }
  }

  /**
   * Immediately save device workflow state to backend with monotonic revision tracking.
   */
  static async saveSessionNow<T = any>(state: T, explicitToken?: string): Promise<SaveSessionResult | null> {
    try {
      this.notifyStatus('saving');
      clientRevision += 1;

      const headers: Record<string, string> = {
        'Content-Type': 'application/json',
      };
      if (explicitToken) {
        headers['x-device-token'] = explicitToken;
      }

      const res = await fetch('/api/session/state', {
        method: 'POST',
        headers,
        credentials: 'include',
        body: JSON.stringify({
          device_token: explicitToken || undefined,
          schema_version: 1,
          revision: clientRevision,
          state,
        }),
      });

      if (!res.ok) {
        this.notifyStatus('error');
        return null;
      }

      const result: SaveSessionResult = await res.json();
      if (typeof result.revision === 'number') {
        clientRevision = Math.max(clientRevision, result.revision);
      }
      this.notifyStatus('saved');
      return result;
    } catch (err) {
      console.warn('Failed to persist device session to server:', err);
      this.notifyStatus('error');
      return null;
    }
  }

  /**
   * Debounced save (default: 1000ms delay) to prevent hammering the server
   * on rapid keystrokes or grid interactions.
   */
  static saveSessionDebounced<T = any>(state: T, delayMs: number = 1000, explicitToken?: string): void {
    if (saveTimeout) {
      clearTimeout(saveTimeout);
    }
    saveTimeout = setTimeout(() => {
      saveTimeout = null;
      void this.saveSessionNow(state, explicitToken);
    }, delayMs);
  }

  /**
   * Clear device session from server.
   */
  static async clearSession(explicitToken?: string): Promise<boolean> {
    try {
      if (saveTimeout) {
        clearTimeout(saveTimeout);
        saveTimeout = null;
      }
      this.resetRevision();

      const headers: Record<string, string> = {};
      if (explicitToken) {
        headers['x-device-token'] = explicitToken;
      }

      const res = await fetch('/api/session/state', {
        method: 'DELETE',
        headers,
        credentials: 'include',
      });

      this.notifyStatus('idle');
      return res.ok;
    } catch (err) {
      console.warn('Failed to clear device session:', err);
      this.notifyStatus('idle');
      return false;
    }
  }
}
