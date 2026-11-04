// @vitest-environment jsdom
import { describe, test, expect, beforeEach, vi, afterEach } from 'vitest';
import { DeviceSessionService } from './DeviceSessionService';

describe('DeviceSessionService', () => {
  beforeEach(() => {
    DeviceSessionService.resetRevision();
    vi.restoreAllMocks();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  test('loadSession sends credentials: include and tracks revision', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        device_token: 'returned-token-abc',
        schema_version: 1,
        revision: 5,
        updated_at: '2026-11-03T12:00:00Z',
        state: { step: 3, workflow_mode: 'sample' },
      }),
    });
    global.fetch = mockFetch;

    const res = await DeviceSessionService.loadSession();

    expect(mockFetch).toHaveBeenCalledWith(
      '/api/session/state',
      expect.objectContaining({
        method: 'GET',
        credentials: 'include',
      }),
    );
    expect(res).not.toBeNull();
    expect(res?.device_token).toBe('returned-token-abc');
    expect(res?.state?.step).toBe(3);
    expect(DeviceSessionService.getRevision()).toBe(5);
  });

  test('loadSession supports programmatic explicitToken in header', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        device_token: 'cli-token-123',
        schema_version: 1,
        revision: 2,
        updated_at: '2026-11-03T12:00:00Z',
        state: null,
      }),
    });
    global.fetch = mockFetch;

    await DeviceSessionService.loadSession('cli-token-123');

    expect(mockFetch).toHaveBeenCalledWith(
      '/api/session/state',
      expect.objectContaining({
        method: 'GET',
        headers: { 'x-device-token': 'cli-token-123' },
        credentials: 'include',
      }),
    );
  });

  test('saveSessionNow increments revision, includes schema_version, and notifies listeners', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        status: 'saved',
        device_token: 'cookie-token',
        schema_version: 1,
        revision: 1,
        updated_at: '2026-11-03T12:01:00Z',
      }),
    });
    global.fetch = mockFetch;

    const statusUpdates: string[] = [];
    const unsub = DeviceSessionService.onSyncStatusChange(status => {
      statusUpdates.push(status);
    });

    const result = await DeviceSessionService.saveSessionNow({ step: 4 });

    expect(mockFetch).toHaveBeenCalledWith(
      '/api/session/state',
      expect.objectContaining({
        method: 'POST',
        credentials: 'include',
        headers: expect.objectContaining({
          'Content-Type': 'application/json',
        }),
        body: JSON.stringify({
          schema_version: 1,
          revision: 1,
          state: { step: 4 },
        }),
      }),
    );
    expect(result?.status).toBe('saved');
    expect(statusUpdates).toContain('saving');
    expect(statusUpdates).toContain('saved');
    expect(DeviceSessionService.getRevision()).toBe(1);
    unsub();
  });

  test('saveSessionDebounced batches multiple rapid calls', async () => {
    vi.useFakeTimers();
    const saveSpy = vi.spyOn(DeviceSessionService, 'saveSessionNow').mockResolvedValue({
      status: 'saved',
      device_token: 'tok',
      updated_at: '2026-11-03T12:00:00Z',
    });

    DeviceSessionService.saveSessionDebounced({ count: 1 }, 500);
    DeviceSessionService.saveSessionDebounced({ count: 2 }, 500);
    DeviceSessionService.saveSessionDebounced({ count: 3 }, 500);

    expect(saveSpy).not.toHaveBeenCalled();

    vi.advanceTimersByTime(500);

    expect(saveSpy).toHaveBeenCalledTimes(1);
    expect(saveSpy).toHaveBeenCalledWith({ count: 3 }, undefined);
  });

  test('clearSession invokes DELETE with credentials: include and resets revision', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ status: 'cleared' }),
    });
    global.fetch = mockFetch;

    // Simulate prior revision
    await DeviceSessionService.saveSessionNow({ dummy: true });

    const ok = await DeviceSessionService.clearSession();

    expect(mockFetch).toHaveBeenCalledWith(
      '/api/session/state',
      expect.objectContaining({
        method: 'DELETE',
        credentials: 'include',
      }),
    );
    expect(ok).toBe(true);
    expect(DeviceSessionService.getRevision()).toBe(0);
  });
});
