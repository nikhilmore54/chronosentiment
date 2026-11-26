import { useEffect, useState } from 'react';
import { fetchOperatorState } from './piClient';
import type { OperatorState } from './types';

const POLL_MS = 5000;

export function useOperatorState(enabled = true) {
  const [state, setState] = useState<OperatorState | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!enabled) return;
    let cancelled = false;
    const ac = new AbortController();

    const tick = async () => {
      const next = await fetchOperatorState(ac.signal);
      if (!cancelled) {
        setState(next);
        setLoading(false);
      }
    };

    tick();
    const id = window.setInterval(tick, POLL_MS);
    return () => {
      cancelled = true;
      ac.abort();
      window.clearInterval(id);
    };
  }, [enabled]);

  return { state, loading };
}
