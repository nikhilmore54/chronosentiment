import React, { useEffect, useMemo, useState } from 'react';

type HorizonClass = 'INTRADAY' | 'MULTI_DAY' | 'MULTI_WEEK' | 'LONG_TERM';
type Market = 'NSE' | 'CRYPTO';

type MhLeg = {
  symbol?: string;
  sector?: string;
  target_weight_pct?: number | null;
  qty_filled?: number | null;
  notional_filled?: number | null;
  avg_price?: number | null;
};

type MhRec = {
  market?: Market | string;
  horizon_class?: HorizonClass;
  horizon_label?: string;
  horizon_term?: string;
  horizon_id?: string;
  symbol?: string;
  symbols?: string[];
  legs?: MhLeg[];
  book_kind?: string;
  observational_only?: boolean;
  leg_count?: number;
  tranches_filled?: number;
  n_tranches?: number;
  tranche_progress?: string;
  total_capital_inr?: number | null;
  side?: string;
  decision?: string;
  entry_price?: number | null;
  stop_loss_price?: number | null;
  target_price?: number | null;
  status?: string;
  thesis?: string;
  source?: string;
  plan_id?: string;
  product_type?: string;
};

type MhUniverseRow = {
  symbol?: string;
  tradingsymbol?: string;
  in_mh_books?: boolean;
  mh_book_count?: number;
  horizon_classes?: string[];
  books?: Array<{
    horizon_id?: string;
    horizon_class?: string;
    decision?: string;
    status?: string;
    plan_id?: string;
    thesis?: string;
  }>;
};

type MhUniverse = {
  total_universe?: number;
  in_mh_books?: number;
  not_in_mh_books?: number;
  universe_members_in_open_books?: number;
  shown_in_open_books?: number;
  note?: string;
  symbols?: MhUniverseRow[];
};

type MhState = {
  live_order_authority?: number;
  updated_utc?: string;
  note?: string;
  counts_by_class?: Record<string, number>;
  universe_member_count?: number;
  recommendations?: MhRec[];
  nse_universe?: MhUniverse;
  nse?: { available?: boolean; catalog?: any[] };
  crypto?: { available?: boolean; catalog?: any[] };
};

const CLASS_META: Record<
  HorizonClass,
  { title: string; meaning: string; color: string; nseOwner: string; cryptoOwner: string }
> = {
  INTRADAY: {
    title: 'Intraday',
    meaning: 'Same-session / ≤60 minutes. Owned by RP-608 live shadows — not opened here.',
    color: '#22d3ee',
    nseOwner: 'RP-608 NSE MIS shadow (link-only)',
    cryptoOwner: 'RP-608 trade-plan overlay (link-only)',
  },
  MULTI_DAY: {
    title: 'Multi-day',
    meaning: 'Swing hold: NSE 2–20 sessions (CNC) · Crypto 2–14 calendar days.',
    color: '#c4b5fd',
    nseOwner: 'RP-609 H1 swing book',
    cryptoOwner: 'RP-610 H1 multi-day book',
  },
  MULTI_WEEK: {
    title: 'Multi-week',
    meaning: 'Longer build: NSE staggered CNC portfolio · Crypto 14–56 day position.',
    color: '#fdba74',
    nseOwner: 'RP-609 H2 portfolio build',
    cryptoOwner: 'RP-610 H2 multi-week book',
  },
  LONG_TERM: {
    title: 'Long-term',
    meaning: 'Thesis hold: NSE ≥60 sessions / ≥90d · Crypto ≥90 calendar days.',
    color: '#fb7185',
    nseOwner: 'RP-609 H3 long-term book',
    cryptoOwner: 'RP-610 H3 long-term book',
  },
};

function formatSymbols(r: MhRec): string {
  const list =
    Array.isArray(r.symbols) && r.symbols.length
      ? r.symbols
      : r.symbol
        ? [r.symbol]
        : [];
  return list.filter(Boolean).join(' · ') || '—';
}

function formatPx(r: MhRec): string {
  return formatLevel(r, 'entry_price');
}

function formatLevel(r: MhRec, field: 'entry_price' | 'stop_loss_price' | 'target_price'): string {
  const raw = r[field];
  if (raw == null || Number.isNaN(Number(raw))) return '—';
  const n = Number(raw);
  if ((r.market || '') === 'CRYPTO' && Math.abs(n) > 0 && Math.abs(n) < 1 && field !== 'entry_price') {
    return `${(n * 100).toFixed(1)}%`;
  }
  if ((r.market || '') === 'NSE') return `₹${n.toFixed(2)}`;
  return `$${n.toFixed(2)}`;
}

export default function MultiHorizonBooksPanel() {
  const [data, setData] = useState<MhState | null>(null);
  const [market, setMarket] = useState<'ALL' | Market>('ALL');
  const [cls, setCls] = useState<'ALL' | HorizonClass>('ALL');
  const [bookKind, setBookKind] = useState<'ALL' | 'ACTIONABLE' | 'UNIVERSE'>('ALL');
  const [symSearch, setSymSearch] = useState('');
  const [innerTab, setInnerTab] = useState<'OPEN' | 'GUIDE' | 'UNIVERSE'>('OPEN');
  const [uniFilter, setUniFilter] = useState<'ALL' | 'IN' | 'OUT'>('ALL');
  const [uniSearch, setUniSearch] = useState('');
  const [err, setErr] = useState<string | null>(null);

  useEffect(() => {
    let alive = true;
    const load = async () => {
      const urls = [
        '/pi-dashboard/api/multi_horizon',
        '/data/multi_horizon_snapshot.json?t=' + Date.now(),
      ];
      for (const url of urls) {
        try {
          const res = await fetch(url);
          if (!res.ok) continue;
          const json = await res.json();
          if (!alive) return;
          if (json?.recommendations || json?.counts_by_class) {
            setData(json);
            setErr(null);
            return;
          }
        } catch {
          // try next
        }
      }
      if (alive) setErr('Could not load multi-horizon books from Pi or local snapshot.');
    };
    load();
    const t = setInterval(load, 15000);
    return () => {
      alive = false;
      clearInterval(t);
    };
  }, []);

  const isUniverseMember = (r: MhRec) =>
    r.book_kind === 'UNIVERSE_MEMBER' || r.decision === 'UNIVERSE_MEMBER' || r.status === 'COVERED';

  const recs = useMemo(() => {
    return (data?.recommendations ?? []).filter((r) => {
      if (market !== 'ALL' && (r.market || '') !== market) return false;
      if (cls !== 'ALL' && (r.horizon_class || '') !== cls) return false;
      if (bookKind === 'ACTIONABLE' && isUniverseMember(r)) return false;
      if (bookKind === 'UNIVERSE' && !isUniverseMember(r)) return false;
      const q = symSearch.trim().toUpperCase();
      if (q) {
        const blob = [r.symbol, ...(r.symbols || []), ...((r.legs || []).map((l) => l.symbol)), r.plan_id, r.thesis]
          .filter(Boolean)
          .join(' ')
          .toUpperCase();
        if (!blob.includes(q)) return false;
      }
      return true;
    });
  }, [data, market, cls, bookKind, symSearch]);

  const counts = data?.counts_by_class ?? {
    INTRADAY: 0,
    MULTI_DAY: 0,
    MULTI_WEEK: 0,
    LONG_TERM: 0,
  };

  const universe = data?.nse_universe;
  const uniRows = useMemo(() => {
    let rows = universe?.symbols ?? [];
    if (uniFilter === 'IN') rows = rows.filter((r) => r.in_mh_books);
    if (uniFilter === 'OUT') rows = rows.filter((r) => !r.in_mh_books);
    const q = uniSearch.trim().toUpperCase();
    if (q) {
      rows = rows.filter(
        (r) =>
          String(r.symbol || '').toUpperCase().includes(q) ||
          String(r.tradingsymbol || '').toUpperCase().includes(q),
      );
    }
    return rows;
  }, [universe, uniFilter, uniSearch]);

  return (
    <div style={{ fontFamily: 'IBM Plex Sans, Segoe UI, sans-serif' }}>
      <div style={{ marginBottom: '20px' }}>
        <h2 style={{ margin: 0, fontSize: '22px', fontWeight: 800, color: '#c4b5fd', letterSpacing: '-0.02em' }}>
          Multi-Horizon Observation Books
        </h2>
        <p style={{ margin: '8px 0 0 0', color: '#94a3b8', fontSize: '14px', maxWidth: '820px', lineHeight: 1.5 }}>
          Separate holding-term paper books for NSE (RP-609 CNC) and Crypto (RP-610 spot-sim).
          This is <strong style={{ color: '#e2e8f0' }}>not</strong> the intraday decision feed —
          each row is an open observation position with a named symbol and a holding class.
        </p>
      </div>

      <div style={{ display: 'flex', gap: '10px', flexWrap: 'wrap', marginBottom: '16px', alignItems: 'center' }}>
        <span style={{ fontSize: '11px', fontWeight: 800, padding: '4px 10px', borderRadius: '999px', background: 'rgba(245,158,11,0.12)', color: '#fbbf24', border: '1px solid rgba(245,158,11,0.35)' }}>
          {(data?.live_order_authority ?? 0).toFixed(1)}% order authority · gates CLOSED
        </span>
        <span style={{ fontSize: '11px', color: '#64748b' }}>
          Updated {data?.updated_utc ?? '—'}
        </span>
        {err ? <span style={{ fontSize: '12px', color: '#f87171' }}>{err}</span> : null}
      </div>

      <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap', marginBottom: '14px', alignItems: 'center' }}>
        <span style={{ fontSize: '11px', fontWeight: 800, padding: '4px 10px', borderRadius: '999px', background: 'rgba(56,189,248,0.12)', color: '#38bdf8', border: '1px solid rgba(56,189,248,0.35)' }}>
          NSE in Open books: {universe?.shown_in_open_books ?? universe?.total_universe ?? 119}
        </span>
        <span style={{ fontSize: '11px', fontWeight: 800, padding: '4px 10px', borderRadius: '999px', background: 'rgba(16,185,129,0.12)', color: '#34d399', border: '1px solid rgba(16,185,129,0.35)' }}>
          Actionable H1–H3: {universe?.in_mh_books ?? 0}
        </span>
        <span style={{ fontSize: '11px', fontWeight: 800, padding: '4px 10px', borderRadius: '999px', background: 'rgba(148,163,184,0.12)', color: '#94a3b8', border: '1px solid rgba(148,163,184,0.3)' }}>
          COVERED members: {universe?.universe_members_in_open_books ?? data?.universe_member_count ?? '—'}
        </span>
      </div>

      <div style={{ display: 'flex', gap: '8px', marginBottom: '18px', flexWrap: 'wrap' }}>
        {([
          { id: 'OPEN' as const, label: '1. Open books (actionable)' },
          { id: 'UNIVERSE' as const, label: '2. NSE universe (119)' },
          { id: 'GUIDE' as const, label: '3. How to read classes' },
        ]).map((t) => (
          <button
            key={t.id}
            type="button"
            onClick={() => setInnerTab(t.id)}
            style={{
              padding: '10px 16px',
              borderRadius: '8px',
              border: 'none',
              cursor: 'pointer',
              fontWeight: 800,
              fontSize: '13px',
              backgroundColor: innerTab === t.id ? '#7c3aed' : '#1e293b',
              color: '#fff',
              boxShadow: innerTab === t.id ? '0 4px 14px rgba(124,58,237,0.35)' : 'none',
            }}
          >
            {t.label}
          </button>
        ))}
      </div>

      {innerTab === 'UNIVERSE' && (
        <div style={{ marginBottom: '8px' }}>
          <p style={{ margin: '0 0 12px 0', color: '#94a3b8', fontSize: '13px', maxWidth: '820px', lineHeight: 1.5 }}>
            All 119 RP-608 NSE shadow scrips. <strong style={{ color: '#34d399' }}>IN MH</strong> means an open H1/H2/H3 observation book.
            Others are market-data coverage only — not auto-opened (0.0% order authority).
          </p>
          <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap', marginBottom: '12px', alignItems: 'center' }}>
            {(['ALL', 'IN', 'OUT'] as const).map((f) => (
              <button
                key={f}
                type="button"
                onClick={() => setUniFilter(f)}
                style={{
                  padding: '6px 12px',
                  borderRadius: '6px',
                  border: uniFilter === f ? '1px solid #38bdf8' : '1px solid rgba(255,255,255,0.1)',
                  background: uniFilter === f ? 'rgba(56,189,248,0.15)' : '#1e293b',
                  color: uniFilter === f ? '#38bdf8' : '#94a3b8',
                  fontWeight: 800,
                  fontSize: '12px',
                  cursor: 'pointer',
                }}
              >
                {f === 'ALL' ? 'All 119' : f === 'IN' ? 'In MH books' : 'Not in MH books'}
              </button>
            ))}
            <input
              value={uniSearch}
              onChange={(e) => setUniSearch(e.target.value)}
              placeholder="Search symbol…"
              style={{
                padding: '6px 10px',
                borderRadius: '6px',
                border: '1px solid rgba(255,255,255,0.12)',
                background: '#0f172a',
                color: '#e2e8f0',
                fontFamily: 'IBM Plex Mono, monospace',
                fontSize: '12px',
              }}
            />
          </div>
          <div style={{ overflowX: 'auto', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '10px', maxHeight: '520px', overflowY: 'auto' }}>
            <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '13px' }}>
              <thead>
                <tr style={{ background: '#1e293b', color: '#94a3b8', textAlign: 'left' }}>
                  <th style={{ padding: '12px 14px' }}>Symbol</th>
                  <th style={{ padding: '12px 14px' }}>Tradingsymbol</th>
                  <th style={{ padding: '12px 14px' }}>MH status</th>
                  <th style={{ padding: '12px 14px' }}>Classes</th>
                  <th style={{ padding: '12px 14px' }}>Open book detail</th>
                </tr>
              </thead>
              <tbody>
                {uniRows.length === 0 ? (
                  <tr>
                    <td colSpan={5} style={{ padding: '24px 14px', color: '#64748b' }}>
                      No symbols for this filter (or universe not loaded from Pi yet).
                    </td>
                  </tr>
                ) : (
                  uniRows.map((r) => (
                    <tr key={r.symbol} style={{ borderTop: '1px solid rgba(255,255,255,0.06)' }}>
                      <td style={{ padding: '10px 14px', fontFamily: 'IBM Plex Mono, monospace', fontWeight: 800, color: '#38bdf8' }}>
                        {r.symbol}
                      </td>
                      <td style={{ padding: '10px 14px', color: '#e2e8f0' }}>{r.tradingsymbol}</td>
                      <td style={{ padding: '10px 14px', fontWeight: 800, color: r.in_mh_books ? '#34d399' : '#64748b' }}>
                        {r.in_mh_books ? 'IN MH' : 'data only'}
                      </td>
                      <td style={{ padding: '10px 14px', color: '#fdba74', fontSize: '11px', fontWeight: 700 }}>
                        {(r.horizon_classes || []).join(' · ') || '—'}
                      </td>
                      <td style={{ padding: '10px 14px', color: '#94a3b8', fontSize: '11px' }}>
                        {(r.books || [])
                          .map((b) => `${b.horizon_id || ''} ${b.status || ''} ${b.plan_id || b.thesis || ''}`.trim())
                          .filter(Boolean)
                          .join(' · ') || '—'}
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {innerTab === 'GUIDE' && (
        <div style={{ display: 'grid', gap: '12px', marginBottom: '8px' }}>
          {(Object.keys(CLASS_META) as HorizonClass[]).map((k) => {
            const m = CLASS_META[k];
            return (
              <div
                key={k}
                style={{
                  background: '#0f172a',
                  border: `1px solid ${m.color}44`,
                  borderLeft: `4px solid ${m.color}`,
                  borderRadius: '10px',
                  padding: '14px 16px',
                }}
              >
                <div style={{ display: 'flex', gap: '10px', alignItems: 'center', marginBottom: '6px', flexWrap: 'wrap' }}>
                  <span style={{ fontSize: '12px', fontWeight: 900, color: m.color }}>{k}</span>
                  <span style={{ fontSize: '14px', fontWeight: 700, color: '#f8fafc' }}>{m.title}</span>
                  <span style={{ fontSize: '11px', color: '#94a3b8' }}>open now: {counts[k] ?? 0}</span>
                </div>
                <p style={{ margin: '0 0 8px 0', color: '#cbd5e1', fontSize: '13px' }}>{m.meaning}</p>
                <div style={{ fontSize: '12px', color: '#64748b' }}>
                  NSE: {m.nseOwner} · Crypto: {m.cryptoOwner}
                </div>
              </div>
            );
          })}
          <p style={{ margin: '4px 0 0 0', fontSize: '12px', color: '#64748b' }}>
            {data?.note ??
              'H0/INTRADAY stays on RP-608 daemons. H1–H3 here are observation-only until promotion gates open.'}
          </p>
        </div>
      )}

      {innerTab === 'OPEN' && (
        <>
          <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap', marginBottom: '14px', alignItems: 'center' }}>
            <span style={{ fontSize: '11px', fontWeight: 700, color: '#64748b', textTransform: 'uppercase' }}>Market</span>
            {(['ALL', 'NSE', 'CRYPTO'] as const).map((m) => (
              <button
                key={m}
                type="button"
                onClick={() => setMarket(m)}
                style={{
                  padding: '6px 12px',
                  borderRadius: '6px',
                  border: market === m ? '1px solid #38bdf8' : '1px solid rgba(255,255,255,0.1)',
                  background: market === m ? 'rgba(56,189,248,0.15)' : '#1e293b',
                  color: market === m ? '#38bdf8' : '#94a3b8',
                  fontWeight: 800,
                  fontSize: '12px',
                  cursor: 'pointer',
                }}
              >
                {m === 'ALL' ? 'NSE + CRYPTO' : m}
              </button>
            ))}
            <span style={{ width: '12px' }} />
            <span style={{ fontSize: '11px', fontWeight: 700, color: '#64748b', textTransform: 'uppercase' }}>Class</span>
            {(['ALL', 'INTRADAY', 'MULTI_DAY', 'MULTI_WEEK', 'LONG_TERM'] as const).map((c) => {
              const color = c === 'ALL' ? '#94a3b8' : CLASS_META[c].color;
              const active = cls === c;
              return (
                <button
                  key={c}
                  type="button"
                  onClick={() => setCls(c)}
                  style={{
                    padding: '6px 10px',
                    borderRadius: '6px',
                    border: `1px solid ${active ? color : 'rgba(255,255,255,0.1)'}`,
                    background: active ? `${color}22` : '#1e293b',
                    color: active ? color : '#94a3b8',
                    fontWeight: 800,
                    fontSize: '11px',
                    cursor: 'pointer',
                  }}
                >
                  {c === 'ALL' ? 'All' : `${c} (${counts[c] ?? 0})`}
                </button>
              );
            })}
            <span style={{ width: '12px' }} />
            <span style={{ fontSize: '11px', fontWeight: 700, color: '#64748b', textTransform: 'uppercase' }}>Books</span>
            {([
              { id: 'ALL' as const, label: 'Actionable + 119 COVERED' },
              { id: 'ACTIONABLE' as const, label: 'Actionable only' },
              { id: 'UNIVERSE' as const, label: 'COVERED only' },
            ]).map((b) => (
              <button
                key={b.id}
                type="button"
                onClick={() => setBookKind(b.id)}
                style={{
                  padding: '6px 10px',
                  borderRadius: '6px',
                  border: bookKind === b.id ? '1px solid #38bdf8' : '1px solid rgba(255,255,255,0.1)',
                  background: bookKind === b.id ? 'rgba(56,189,248,0.15)' : '#1e293b',
                  color: bookKind === b.id ? '#38bdf8' : '#94a3b8',
                  fontWeight: 800,
                  fontSize: '11px',
                  cursor: 'pointer',
                }}
              >
                {b.label}
              </button>
            ))}
            <input
              value={symSearch}
              onChange={(e) => setSymSearch(e.target.value)}
              placeholder="Search symbol…"
              style={{
                padding: '6px 10px',
                borderRadius: '6px',
                border: '1px solid rgba(255,255,255,0.12)',
                background: '#0f172a',
                color: '#e2e8f0',
                fontFamily: 'IBM Plex Mono, monospace',
                fontSize: '12px',
              }}
            />
          </div>

          <div style={{ overflowX: 'auto', border: '1px solid rgba(255,255,255,0.08)', borderRadius: '10px' }}>
            <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '13px' }}>
              <thead>
                <tr style={{ background: '#1e293b', color: '#94a3b8', textAlign: 'left' }}>
                  <th style={{ padding: '12px 14px' }}>Symbol(s)</th>
                  <th style={{ padding: '12px 14px' }}>Market</th>
                  <th style={{ padding: '12px 14px' }}>Holding class</th>
                  <th style={{ padding: '12px 14px' }}>What it means</th>
                  <th style={{ padding: '12px 14px' }}>Decision</th>
                  <th style={{ padding: '12px 14px' }}>Side</th>
                  <th style={{ padding: '12px 14px' }}>Status</th>
                  <th style={{ padding: '12px 14px' }}>Entry</th>
                  <th style={{ padding: '12px 14px' }}>Stop</th>
                  <th style={{ padding: '12px 14px' }}>Target</th>
                  <th style={{ padding: '12px 14px' }}>Thesis / plan</th>
                </tr>
              </thead>
              <tbody>
                {recs.length === 0 ? (
                  <tr>
                    <td colSpan={11} style={{ padding: '24px 14px', color: '#64748b' }}>
                      No open observation books for this filter. Intraday (H0) never lists symbols here —
                      check the Guide tab, or open H1–H3 books on the Pi.
                    </td>
                  </tr>
                ) : (
                  recs.flatMap((r, i) => {
                    const hc = (r.horizon_class || 'INTRADAY') as HorizonClass;
                    const meta = CLASS_META[hc];
                    if (isUniverseMember(r)) {
                      return [
                        <tr key={`uni-${r.symbol}-${i}`} style={{ borderTop: '1px solid rgba(255,255,255,0.04)', background: 'rgba(148,163,184,0.05)' }}>
                          <td style={{ padding: '10px 14px', fontFamily: 'IBM Plex Mono, monospace', fontWeight: 800, color: '#7dd3fc', whiteSpace: 'nowrap' }}>
                            {formatSymbols(r)}
                          </td>
                          <td style={{ padding: '10px 14px', color: '#e2e8f0', fontWeight: 700 }}>NSE</td>
                          <td style={{ padding: '10px 14px' }}>
                            <span style={{ fontSize: '11px', fontWeight: 900, color: '#94a3b8' }}>MULTI_DAY</span>
                            <div style={{ fontSize: '10px', color: '#64748b', marginTop: '2px' }}>H1_SWING · universe</div>
                          </td>
                          <td style={{ padding: '10px 14px', color: '#94a3b8', fontSize: '12px' }}>
                            119-universe member · COVERED (not an opened trade)
                          </td>
                          <td style={{ padding: '10px 14px', color: '#94a3b8', fontWeight: 700, fontSize: '12px' }}>UNIVERSE_MEMBER</td>
                          <td style={{ padding: '10px 14px', color: '#94a3b8' }}>—</td>
                          <td style={{ padding: '10px 14px', color: '#94a3b8', fontWeight: 700 }}>COVERED</td>
                          <td style={{ padding: '10px 14px', fontFamily: 'IBM Plex Mono, monospace', color: '#cbd5e1' }}>{formatPx(r)}</td>
                          <td style={{ padding: '10px 14px', color: '#64748b', fontSize: '11px' }}>n/a</td>
                          <td style={{ padding: '10px 14px', color: '#64748b', fontSize: '11px' }}>n/a</td>
                          <td style={{ padding: '10px 14px', color: '#94a3b8', fontSize: '12px' }}>
                            rp608_nse_119_universe
                            <div style={{ fontSize: '10px', color: '#64748b' }}>RP-608/609 · CNC · 0.0% authority</div>
                          </td>
                        </tr>,
                      ];
                    }
                    const isPortfolio =
                      r.book_kind === 'PORTFOLIO_BUILD' ||
                      r.decision === 'PORTFOLIO_BUILD' ||
                      (Array.isArray(r.legs) && r.legs.length > 1);
                    if (isPortfolio) {
                      const legs =
                        Array.isArray(r.legs) && r.legs.length
                          ? r.legs
                          : (r.symbols || []).map((s) => ({ symbol: s }));
                      const progress =
                        r.tranche_progress ||
                        (r.n_tranches != null ? `${r.tranches_filled ?? 0}/${r.n_tranches}` : '—');
                      const capital =
                        r.total_capital_inr != null && !Number.isNaN(Number(r.total_capital_inr))
                          ? `₹${Number(r.total_capital_inr).toLocaleString('en-IN')}`
                          : '—';
                      const planId = r.plan_id || r.thesis || r.symbol || 'PORTFOLIO';
                      const header = (
                        <tr
                          key={`${r.market}-PORTFOLIO-${planId}-${i}`}
                          style={{ borderTop: '1px solid rgba(251,146,60,0.25)', background: 'rgba(251,146,60,0.08)' }}
                        >
                          <td style={{ padding: '12px 14px', fontFamily: 'IBM Plex Mono, monospace', fontWeight: 800, color: '#fdba74' }}>
                            PORTFOLIO · {legs.length} names
                            <div style={{ fontSize: '10px', color: '#94a3b8', fontWeight: 600, marginTop: '2px' }}>
                              one CNC build plan — not a single ticker
                            </div>
                          </td>
                          <td style={{ padding: '12px 14px', color: '#e2e8f0', fontWeight: 700 }}>{r.market || 'NSE'}</td>
                          <td style={{ padding: '12px 14px' }}>
                            <span style={{ fontSize: '11px', fontWeight: 900, color: meta.color }}>{hc}</span>
                            <div style={{ fontSize: '10px', color: '#64748b', marginTop: '2px' }}>{r.horizon_id}</div>
                          </td>
                          <td style={{ padding: '12px 14px', color: '#94a3b8', fontSize: '12px', maxWidth: '220px' }}>
                            {r.horizon_term || meta.meaning}
                            <div style={{ fontSize: '10px', color: '#fdba74', marginTop: '2px' }}>
                              Staggered build · tranche {progress} · capital {capital}
                            </div>
                          </td>
                          <td style={{ padding: '12px 14px', color: '#c4b5fd', fontWeight: 700, fontSize: '12px' }}>
                            {r.decision || 'PORTFOLIO_BUILD'}
                          </td>
                          <td style={{ padding: '12px 14px', color: '#e2e8f0' }}>{r.side || 'LONG'}</td>
                          <td style={{ padding: '12px 14px', color: '#34d399', fontWeight: 700 }}>{r.status || 'ACTIVE'}</td>
                          <td style={{ padding: '12px 14px', fontFamily: 'IBM Plex Mono, monospace', color: '#f8fafc' }}>—</td>
                          <td style={{ padding: '12px 14px', color: '#64748b', fontSize: '11px' }} title="H2 uses weights/caps, not swing stops">n/a</td>
                          <td style={{ padding: '12px 14px', color: '#64748b', fontSize: '11px' }} title="H2 uses weights/caps, not swing targets">n/a</td>
                          <td style={{ padding: '12px 14px', color: '#94a3b8', fontSize: '12px' }}>
                            {planId}
                            <div style={{ fontSize: '10px', color: '#64748b' }}>{r.source} · {r.product_type || 'CNC'}</div>
                          </td>
                        </tr>
                      );
                      const legRows = legs.map((leg, li) => (
                        <tr key={`${planId}-leg-${leg.symbol}-${li}`} style={{ background: 'rgba(0,0,0,0.18)', borderTop: '1px solid rgba(255,255,255,0.04)' }}>
                          <td style={{ padding: '10px 14px 10px 28px', fontFamily: 'IBM Plex Mono, monospace', fontWeight: 800, color: '#38bdf8', whiteSpace: 'nowrap' }}>
                            {leg.symbol || '—'}
                          </td>
                          <td style={{ padding: '10px 14px', color: '#e2e8f0', fontWeight: 700 }}>{r.market || 'NSE'}</td>
                          <td style={{ padding: '10px 14px', color: '#94a3b8', fontSize: '11px' }}>
                            {leg.sector || '—'}
                            <div style={{ fontSize: '10px', color: '#64748b' }}>
                              leg · weight {leg.target_weight_pct != null ? `${Number(leg.target_weight_pct).toFixed(1)}%` : '—'}
                            </div>
                          </td>
                          <td style={{ padding: '10px 14px', color: '#94a3b8', fontSize: '12px' }}>
                            CNC tranche leg · qty {leg.qty_filled ?? 0}
                          </td>
                          <td style={{ padding: '10px 14px', color: '#c4b5fd', fontSize: '11px' }}>—</td>
                          <td style={{ padding: '10px 14px', color: '#e2e8f0' }}>{r.side || 'LONG'}</td>
                          <td style={{ padding: '10px 14px', color: '#94a3b8', fontSize: '11px' }}>building</td>
                          <td style={{ padding: '10px 14px', fontFamily: 'IBM Plex Mono, monospace', color: '#f8fafc' }}>
                            {leg.avg_price != null ? `₹${Number(leg.avg_price).toFixed(2)}` : '—'}
                          </td>
                          <td style={{ padding: '10px 14px', color: '#64748b', fontSize: '11px' }}>n/a</td>
                          <td style={{ padding: '10px 14px', color: '#64748b', fontSize: '11px' }}>n/a</td>
                          <td style={{ padding: '10px 14px', color: '#64748b', fontSize: '11px' }}>plan {planId}</td>
                        </tr>
                      ));
                      return [header, ...legRows];
                    }
                    const stopTxt = formatLevel(r, 'stop_loss_price');
                    const tgtTxt = formatLevel(r, 'target_price');
                    return [
                      <tr key={`${r.market}-${hc}-${formatSymbols(r)}-${i}`} style={{ borderTop: '1px solid rgba(255,255,255,0.06)' }}>
                        <td style={{ padding: '12px 14px', fontFamily: 'IBM Plex Mono, monospace', fontWeight: 800, color: '#38bdf8', whiteSpace: 'nowrap' }}>
                          {formatSymbols(r)}
                        </td>
                        <td style={{ padding: '12px 14px', color: '#e2e8f0', fontWeight: 700 }}>{r.market || '—'}</td>
                        <td style={{ padding: '12px 14px' }}>
                          <span style={{ fontSize: '11px', fontWeight: 900, color: meta.color }}>{hc}</span>
                          <div style={{ fontSize: '10px', color: '#64748b', marginTop: '2px' }}>{r.horizon_id}</div>
                        </td>
                        <td style={{ padding: '12px 14px', color: '#94a3b8', fontSize: '12px', maxWidth: '220px' }}>
                          {r.horizon_term || meta.meaning}
                        </td>
                        <td style={{ padding: '12px 14px', color: '#c4b5fd', fontWeight: 700, fontSize: '12px' }}>
                          {r.decision || '—'}
                        </td>
                        <td style={{ padding: '12px 14px', color: '#e2e8f0' }}>{r.side || 'LONG'}</td>
                        <td style={{ padding: '12px 14px', color: '#34d399', fontWeight: 700 }}>{r.status || 'OPEN'}</td>
                        <td style={{ padding: '12px 14px', fontFamily: 'IBM Plex Mono, monospace', color: '#f8fafc' }}>
                          {formatPx(r)}
                        </td>
                        <td style={{ padding: '12px 14px', fontFamily: 'IBM Plex Mono, monospace', color: '#f87171' }}>
                          {hc === 'LONG_TERM' && stopTxt === '—' ? 'n/a' : stopTxt}
                        </td>
                        <td style={{ padding: '12px 14px', fontFamily: 'IBM Plex Mono, monospace', color: '#34d399' }}>
                          {hc === 'LONG_TERM' && tgtTxt === '—' ? 'n/a' : tgtTxt}
                        </td>
                        <td style={{ padding: '12px 14px', color: '#94a3b8', fontSize: '12px' }}>
                          {r.thesis || r.plan_id || '—'}
                          <div style={{ fontSize: '10px', color: '#64748b' }}>{r.source} · {r.product_type || '—'}</div>
                        </td>
                      </tr>,
                    ];
                  })
                )}
              </tbody>
            </table>
          </div>
        </>
      )}
    </div>
  );
}
