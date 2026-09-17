/**
 * Today's Decision Board — Cockpit home screen.
 *
 * Presentation only. Projects GET /deferred-live + DecisionBrief facts.
 * Does not classify, fill, exit, or reassess.
 */
import { useEffect, useMemo, useState, type ReactNode } from 'react';
import type { DecisionBrief, DecisionPortfolioContext, DeferredLiveResponse } from '../../types/decisionBrief';
import {
  buildDecisionJournal,
  buildLifecycleStory,
  buildLiveActivityFeed,
  buildNowActionRows,
  buildOutcomeAnalytics,
  buildPaperPortfolio,
  buildTodayRecommendations,
  conditionCaption,
  conditionFromMark,
  isOpenBookRow,
  type DecisionJournalEntry,
  type LifecycleStoryItem,
  type LiveActivityItem,
  type NowActionRow,
  type NowCondition,
  type OutcomeAnalytics,
  type OutcomeClosedRow,
  type PaperPortfolio,
  type TodayRecommendations,
} from './nowAction';
import { classifyLiveFreshness } from './liveFreshness';

function tickerLabel(t: string | null | undefined): string {
  return (t ?? '').replace('_NS', '') || '—';
}

function fmtPct(v: number | null | undefined): string {
  if (v == null) return '—';
  return `${v >= 0 ? '+' : ''}${(v * 100).toFixed(2)}%`;
}

function fmtPrice(v: number | null | undefined): string {
  if (v == null) return '—';
  return `₹${v.toLocaleString('en-IN', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

function fmtIstHm(unix: number | null | undefined): string {
  if (unix == null) return '—';
  return new Date(unix * 1000).toLocaleString('en-GB', {
    timeZone: 'Asia/Kolkata',
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  }) + ' IST';
}

function fmtIstHms(unix: number | null | undefined): string {
  if (unix == null) return '—';
  return new Date(unix * 1000).toLocaleString('en-GB', {
    timeZone: 'Asia/Kolkata',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  });
}

function fmtIstFull(unix: number | null | undefined): string {
  if (unix == null) return '—';
  return new Date(unix * 1000).toLocaleString('en-GB', {
    timeZone: 'Asia/Kolkata',
    day: '2-digit',
    month: 'short',
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  }) + ' IST';
}

function formatSessionDay(date: string | null | undefined): string {
  if (!date) return 'TODAY';
  const [y, m, d] = date.split('-').map(Number);
  if (!y || !m || !d) return date;
  const months = ['JAN', 'FEB', 'MAR', 'APR', 'MAY', 'JUN', 'JUL', 'AUG', 'SEP', 'OCT', 'NOV', 'DEC'];
  return `${d} ${months[m - 1]} ${y}`;
}

function fmtRemaining(v: number | null | undefined): string {
  if (v == null) return '—';
  return `${(Math.abs(v) * 100).toFixed(2)}%`;
}

function conditionColor(condition: NowCondition): string {
  if (condition === 'ADVERSE') return '#b45309';
  if (condition === 'FAVOURABLE') return '#16a34a';
  return 'var(--text-secondary)';
}

function markColor(v: number | null | undefined): string {
  if (v == null) return 'var(--text-secondary)';
  if (v > 0) return '#16a34a';
  if (v < 0) return '#dc2626';
  return 'var(--text-secondary)';
}

function dirColor(dir: string | undefined): string {
  if (dir === 'LONG') return '#16a34a';
  if (dir === 'SHORT') return '#dc2626';
  return 'var(--text-secondary)';
}

export function DecisionBoard({
  live,
  decisions,
  ctxById,
  lastIngestSeenAt,
  error,
  children,
}: {
  live: DeferredLiveResponse | null;
  decisions: DecisionBrief[];
  ctxById?: Map<string, DecisionPortfolioContext>;
  lastIngestSeenAt: number | null;
  error: string | null;
  children?: ReactNode;
}) {
  const [nowMs, setNowMs] = useState(() => Date.now());
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [showNotAct, setShowNotAct] = useState(false);
  const [journalFilter, setJournalFilter] = useState<'ACT' | 'NOT_ACT' | 'ALL'>('ACT');

  const rows = useMemo(
    () => buildNowActionRows(live, decisions, ctxById),
    [live, decisions, ctxById],
  );
  const openRows = useMemo(() => rows.filter(isOpenBookRow), [rows]);
  const noTradeRows = useMemo(() => rows.filter(r => r.action === 'NO TRADE'), [rows]);
  const recs = useMemo(
    () => buildTodayRecommendations(live, openRows, noTradeRows),
    [live, openRows, noTradeRows],
  );
  const portfolio = useMemo(
    () => buildPaperPortfolio(live, openRows),
    [live, openRows],
  );
  const lifecycle = useMemo(() => buildLifecycleStory(live), [live]);
  const journal = useMemo(() => buildDecisionJournal(live, rows, decisions), [live, rows, decisions]);
  const outcomes = useMemo(() => buildOutcomeAnalytics(journal), [journal]);
  const activity = useMemo(() => buildLiveActivityFeed(live, recs), [live, recs]);
  const freshness = classifyLiveFreshness(live?.observation_feed, lastIngestSeenAt, nowMs);

  useEffect(() => {
    if (freshness.kind !== 'LIVE' && freshness.kind !== 'STALE') return;
    const id = window.setInterval(() => setNowMs(Date.now()), 250);
    return () => window.clearInterval(id);
  }, [freshness.kind]);

  const selected = selectedId == null
    ? null
    : rows.find(r =>
      (r.decisionId || r.ticker) === selectedId
      || r.asofDecisionId === selectedId
      || r.ticker === selectedId
    ) ?? null;
  const briefFor = (row: NowActionRow) =>
    decisions.find(d => d.id === row.decisionId)
    ?? decisions.find(d => d.ticker === row.ticker && (!recs?.sessionDate || d.date === recs.sessionDate))
    ?? null;

  const selectedJournal = selected
    ? journal.find(j =>
      j.selectId === selectedId
      || j.decisionId === selected.decisionId
      || j.decisionId === selected.asofDecisionId
      || j.ticker === selected.ticker,
    ) ?? null
    : null;

  if (selected) {
    return (
      <JournalEntryDetail
        row={selected}
        entry={selectedJournal}
        brief={briefFor(selected)}
        onBack={() => setSelectedId(null)}
      />
    );
  }

  const select = (row: NowActionRow) => setSelectedId(row.decisionId || row.ticker);
  const selectActivity = (item: LiveActivityItem) => {
    if (!item.selectId && !item.ticker) return;
    const id = item.selectId;
    const row = rows.find(r =>
      (id != null && (r.decisionId === id || r.asofDecisionId === id || r.ticker === id))
      || (item.ticker != null && r.ticker === item.ticker),
    );
    if (row) setSelectedId(row.decisionId || row.ticker);
  };
  const selectJournal = (entry: DecisionJournalEntry) => {
    const row = rows.find(r =>
      r.ticker === entry.ticker
      || r.asofDecisionId === entry.decisionId
      || r.decisionId === entry.decisionId
      || r.decisionId === entry.selectId,
    );
    setSelectedId(row?.decisionId || row?.ticker || entry.selectId || entry.ticker);
  };
  const selectClosed = (item: OutcomeClosedRow) => {
    const row = rows.find(r =>
      r.ticker === item.ticker
      || r.asofDecisionId === item.selectId
      || r.decisionId === item.selectId,
    );
    setSelectedId(row?.decisionId || row?.ticker || item.selectId || item.ticker);
  };

  return (
    <div style={{ maxWidth: '720px', margin: '0 auto', padding: '0.25rem 0 2rem', fontVariantNumeric: 'tabular-nums' }}>
      <div style={{ marginBottom: '1.1rem' }}>
        <div style={{ fontSize: '0.62rem', fontWeight: 800, letterSpacing: '0.08em', textTransform: 'uppercase', color: '#0369a1' }}>
          ChronoSentiment
        </div>
        <h2 style={{ margin: '0.2rem 0 0', fontSize: '1.35rem', fontWeight: 800, color: 'var(--text-primary)' }}>
          Today · {formatSessionDay(recs?.sessionDate ?? live?.session?.date)}
        </h2>
        <div style={{ fontSize: '0.68rem', color: 'var(--text-secondary)', marginTop: '0.2rem' }}>
          Decision cockpit · paper recommendations · no broker
          {freshness.lastObsUnix != null ? ` · last observation ${fmtIstHm(freshness.lastObsUnix)}` : ''}
        </div>
      </div>

      {error && (
        <div style={{ fontSize: '0.68rem', color: '#ef4444', marginBottom: '0.75rem' }}>{error}</div>
      )}

      {!recs ? (
        <div style={{ fontSize: '0.78rem', color: 'var(--text-secondary)', lineHeight: 1.5 }}>
          No live paper session on this API. Arm a session and supply observations — this board does not invent recommendations.
        </div>
      ) : (
        <>
          <PortfolioHero recs={recs} portfolio={portfolio} />
          <Section title="Lifecycle" note="Ledger counts · PAPER_ENTER / TARGET / STOP / HORIZON only">
            <LifecycleStrip portfolio={portfolio} />
          </Section>
          <Section title="Performance" note="Open mark is mark-to-last · not realized P&L">
            <PerformanceBlock portfolio={portfolio} />
          </Section>
          <Section title="Position mix" note="Open book only · informational">
            <PositionMix portfolio={portfolio} />
          </Section>
          <Section title="Today's lifecycle" note="What the ledger did · not marks or CAUTION">
            <LifecycleStory items={lifecycle} onSelect={selectActivity} />
          </Section>
          <Section title="Decision journal" note="As-of decisions · evidence at decision time · ledger events · realized only when closed">
            <DecisionJournal
              entries={journal}
              filter={journalFilter}
              onFilter={setJournalFilter}
              onSelect={selectJournal}
            />
          </Section>
          <Section title="Outcome analytics" note="Realized TARGET / STOP / HORIZON only · not open marks">
            <OutcomeAnalyticsPanel
              analytics={outcomes}
              nOpen={portfolio?.nOpen ?? recs.nActOpen}
              onSelect={selectClosed}
            />
          </Section>
          <Section title="Attention" note="Nearest frozen STOP · HOLD is not an exit · condition is not a recommendation">
            <NameList rows={recs.attention} kind="stop" onSelect={select} />
          </Section>
          <Section title="Opportunity" note="Nearest frozen TARGET · HOLD is not an exit">
            <NameList rows={recs.opportunity} kind="target" onSelect={select} />
          </Section>
          <Section title="Live activity" note="What just happened · existing events only">
            <LiveActivityFeed items={activity} onSelect={selectActivity} />
          </Section>
          <Section title="Not ACT" note="Withheld · not in the paper book">
            <button
              type="button"
              onClick={() => setShowNotAct(v => !v)}
              style={{
                display: 'block', width: '100%', textAlign: 'left', background: 'transparent',
                border: 'none', padding: '0.15rem 0', cursor: 'pointer', color: 'inherit',
              }}
            >
              <div style={{ fontSize: '0.92rem', fontWeight: 800, color: 'var(--text-primary)' }}>
                {recs.nNotAct} opportunities withheld
              </div>
              <div style={{ fontSize: '0.65rem', color: 'var(--text-secondary)', marginTop: '0.12rem' }}>
                {showNotAct ? 'Hide names' : 'Show names · NOT RECOMMENDED'}
              </div>
            </button>
            {showNotAct && (
              <div style={{ marginTop: '0.35rem' }}>
                {recs.notAct.map(r => (
                  <button
                    key={r.decisionId || r.ticker}
                    type="button"
                    onClick={() => select(r)}
                    style={{
                      display: 'flex', justifyContent: 'space-between', gap: '0.75rem',
                      width: '100%', textAlign: 'left', background: 'transparent',
                      border: 'none', padding: '0.28rem 0', cursor: 'pointer', color: 'inherit',
                      borderTop: '1px solid var(--border-color)',
                    }}
                  >
                    <span style={{ fontSize: '0.78rem', fontWeight: 700 }}>{tickerLabel(r.ticker)}</span>
                    <span style={{ fontSize: '0.68rem', color: '#6b7280', fontWeight: 700 }}>
                      {r.entryState || 'NOT_ACT'}
                    </span>
                  </button>
                ))}
              </div>
            )}
          </Section>
          <div style={{ fontSize: '0.62rem', color: 'var(--text-secondary)', marginTop: '1rem', lineHeight: 1.45 }}>
            Recommendation ≠ condition ≠ lifecycle ≠ outcome.
            Portfolio counts are ledger-driven. Outcome analytics stay empty until TARGET / STOP / HORIZON.
          </div>
        </>
      )}
      {children}
    </div>
  );
}

function PortfolioHero({
  recs,
  portfolio,
}: {
  recs: TodayRecommendations;
  portfolio: PaperPortfolio | null;
}) {
  const hold = (portfolio?.nOpen ?? 0) > 0 && (portfolio?.nClosed ?? 0) === 0;
  const nOpen = portfolio?.nOpen ?? recs.nActOpen;
  return (
    <div style={{
      padding: '1rem 1.05rem',
      borderRadius: '10px',
      border: '1px solid #0369a166',
      backgroundColor: '#0369a114',
      marginBottom: '1.15rem',
    }}>
      <div style={{ fontSize: '0.62rem', fontWeight: 800, letterSpacing: '0.06em', textTransform: 'uppercase', color: '#0369a1' }}>
        Today&apos;s Paper Portfolio
      </div>
      <div style={{ fontSize: '1.55rem', fontWeight: 800, color: 'var(--text-primary)', marginTop: '0.35rem' }}>
        {nOpen}
      </div>
      <div style={{ fontSize: '0.82rem', fontWeight: 800, letterSpacing: '0.04em', color: '#0369a1' }}>
        OPEN
      </div>
      <div style={{ fontSize: '0.82rem', fontWeight: 700, color: 'var(--text-primary)', marginTop: '0.2rem' }}>
        {portfolio?.nLong ?? recs.nLong} LONG · {portfolio?.nShort ?? recs.nShort} SHORT
      </div>
      <div style={{ fontSize: '0.68rem', color: 'var(--text-secondary)', marginTop: '0.35rem' }}>
        {portfolio?.nTarget ?? recs.nTarget} TARGET
        {' · '}{portfolio?.nStop ?? recs.nStop} STOP
        {' · '}{portfolio?.nHorizon ?? recs.nHorizon} HORIZON
      </div>

      <HeroBlock label="Open mark">
        <div style={{ fontSize: '1.05rem', fontWeight: 800, color: markColor(portfolio?.meanOpenMark ?? recs.meanOpenMark) }}>
          {fmtPct(portfolio?.meanOpenMark ?? recs.meanOpenMark)}
        </div>
        <div style={{ fontSize: '0.62rem', color: 'var(--text-secondary)', marginTop: '0.1rem' }}>
          Mark-to-last · equal-weight · not realized P&L
        </div>
      </HeroBlock>

      <HeroBlock label="Realized">
        <div style={{ fontSize: '1.05rem', fontWeight: 800, color: 'var(--text-primary)' }}>
          {portfolio?.meanClosedReturn == null ? '—' : fmtPct(portfolio.meanClosedReturn)}
        </div>
        <div style={{ fontSize: '0.62rem', color: 'var(--text-secondary)', marginTop: '0.1rem' }}>
          {(portfolio?.nClosed ?? 0) === 0
            ? 'No TARGET / STOP / HORIZON yet · nothing realized'
            : `${portfolio?.nClosed} closed · equal-weight mean`}
        </div>
      </HeroBlock>

      <HeroBlock label="Recommendation">
        <div style={{ fontSize: '1.15rem', fontWeight: 800, letterSpacing: '0.04em', color: '#0369a1' }}>
          {hold ? 'HOLD' : nOpen > 0 ? 'HOLD · see lifecycle' : 'NO OPEN PAPER'}
        </div>
        <div style={{ fontSize: '0.65rem', color: 'var(--text-secondary)', marginTop: '0.12rem' }}>
          Frozen paper remains active. HOLD is not a health claim.
        </div>
      </HeroBlock>

      <HeroBlock label="Condition" last>
        <div style={{ fontSize: '0.78rem', fontWeight: 700, color: 'var(--text-primary)' }}>
          {recs.nAdverse} currently adverse · {recs.nFavourable} currently favourable
        </div>
        <div style={{ fontSize: '0.62rem', color: 'var(--text-secondary)', marginTop: '0.1rem' }}>
          Informational attention · not an invalidation
        </div>
      </HeroBlock>
    </div>
  );
}

function LifecycleStrip({ portfolio }: { portfolio: PaperPortfolio | null }) {
  const cells = [
    ['OPEN', portfolio?.nOpen ?? 0],
    ['TARGET', portfolio?.nTarget ?? 0],
    ['STOP', portfolio?.nStop ?? 0],
    ['HORIZON', portfolio?.nHorizon ?? 0],
  ] as const;
  return (
    <div style={{
      display: 'grid',
      gridTemplateColumns: 'repeat(4, minmax(0, 1fr))',
      gap: '0.5rem',
    }}>
      {cells.map(([label, n]) => (
        <div key={label} style={{ padding: '0.45rem 0.2rem 0.15rem' }}>
          <div style={{ fontSize: '0.58rem', fontWeight: 800, letterSpacing: '0.05em', color: 'var(--text-secondary)' }}>
            {label}
          </div>
          <div style={{ fontSize: '1.25rem', fontWeight: 800, color: 'var(--text-primary)', marginTop: '0.12rem' }}>
            {n}
          </div>
        </div>
      ))}
    </div>
  );
}

function PerformanceBlock({ portfolio }: { portfolio: PaperPortfolio | null }) {
  const closed = portfolio?.nClosed ?? 0;
  const realized = closed === 0 ? null : portfolio?.meanClosedReturn ?? null;
  return (
    <div>
      <PerfRow
        label="Realized"
        value={realized == null ? '—' : fmtPct(realized)}
        note={closed === 0 ? '0 closed trades' : `${closed} closed · ${portfolio?.nWinClosed ?? 0} with positive realized`}
        color={realized == null ? undefined : markColor(realized)}
      />
      <PerfRow
        label="Open mark"
        value={fmtPct(portfolio?.meanOpenMark)}
        note={`${portfolio?.nOpen ?? 0} positions · mark-to-last`}
        color={markColor(portfolio?.meanOpenMark)}
      />
      <PerfRow
        label="Total"
        value="—"
        note="Not summed. Realized and open mark stay separate until the book has closed trades."
      />
    </div>
  );
}

function PerfRow({
  label,
  value,
  note,
  color,
}: {
  label: string;
  value: string;
  note: string;
  color?: string;
}) {
  return (
    <div style={{
      display: 'flex',
      justifyContent: 'space-between',
      gap: '1rem',
      padding: '0.45rem 0',
      borderBottom: '1px solid var(--border-color)',
    }}>
      <div>
        <div style={{ fontSize: '0.62rem', fontWeight: 800, letterSpacing: '0.05em', textTransform: 'uppercase', color: 'var(--text-secondary)' }}>
          {label}
        </div>
        <div style={{ fontSize: '0.62rem', color: 'var(--text-secondary)', marginTop: '0.12rem' }}>{note}</div>
      </div>
      <div style={{ fontSize: '1.05rem', fontWeight: 800, color: color ?? 'var(--text-primary)' }}>{value}</div>
    </div>
  );
}

function PositionMix({ portfolio }: { portfolio: PaperPortfolio | null }) {
  return (
    <div>
      <MixRow label="LONG" n={portfolio?.nLong ?? 0} mark={portfolio?.meanLongMark ?? null} />
      <MixRow label="SHORT" n={portfolio?.nShort ?? 0} mark={portfolio?.meanShortMark ?? null} />
    </div>
  );
}

function MixRow({ label, n, mark }: { label: string; n: number; mark: number | null }) {
  return (
    <div style={{
      display: 'flex',
      justifyContent: 'space-between',
      gap: '1rem',
      padding: '0.4rem 0',
      borderBottom: '1px solid var(--border-color)',
      alignItems: 'baseline',
    }}>
      <div style={{ fontSize: '0.82rem', fontWeight: 800, color: dirColor(label) }}>{label}</div>
      <div style={{ fontSize: '0.78rem', fontWeight: 700 }}>
        {n}
        <span style={{ color: markColor(mark), marginLeft: '0.75rem' }}>{fmtPct(mark)}</span>
      </div>
    </div>
  );
}

function LifecycleStory({
  items,
  onSelect,
}: {
  items: LifecycleStoryItem[];
  onSelect: (item: LiveActivityItem) => void;
}) {
  if (items.length === 0) {
    return (
      <div style={{ fontSize: '0.72rem', color: 'var(--text-secondary)', padding: '0.35rem 0' }}>
        No ledger lifecycle yet. This section copies PAPER_ENTER / TARGET / STOP / HORIZON only.
      </div>
    );
  }
  return (
    <div>
      {items.map((item, i) => {
        const clickable = !!(item.selectId || item.ticker);
        const Tag = clickable ? 'button' : 'div';
        return (
          <Tag
            key={`${item.kind}-${item.unix}-${item.ticker ?? 'x'}-${i}`}
            type={clickable ? 'button' : undefined}
            onClick={clickable ? () => onSelect({
              unix: item.unix,
              kind: item.kind,
              ticker: item.ticker,
              direction: null,
              headline: item.headline,
              detail: item.detail,
              selectId: item.selectId,
            }) : undefined}
            style={{
              display: 'block',
              width: '100%',
              textAlign: 'left',
              background: 'transparent',
              border: 'none',
              padding: '0.5rem 0',
              cursor: clickable ? 'pointer' : 'default',
              color: 'inherit',
              borderBottom: '1px solid var(--border-color)',
            }}
          >
            <div style={{ fontSize: '0.62rem', fontWeight: 700, color: 'var(--text-secondary)' }}>
              {fmtIstHm(item.unix)}
            </div>
            <div style={{ fontSize: '0.9rem', fontWeight: 800, marginTop: '0.1rem' }}>{item.headline}</div>
            <div style={{ fontSize: '0.7rem', fontWeight: 700, color: item.kind === 'PAPER_ENTER' ? 'var(--text-secondary)' : markColor(item.realizedReturn), marginTop: '0.08rem' }}>
              {item.detail}
            </div>
          </Tag>
        );
      })}
    </div>
  );
}

function HeroBlock({ label, last, children }: { label: string; last?: boolean; children: ReactNode }) {
  return (
    <div style={{
      marginTop: '0.75rem',
      paddingTop: '0.7rem',
      borderTop: '1px solid #0369a133',
      paddingBottom: last ? 0 : undefined,
    }}>
      <div style={{ fontSize: '0.58rem', fontWeight: 800, letterSpacing: '0.06em', textTransform: 'uppercase', color: 'var(--text-secondary)' }}>
        {label}
      </div>
      <div style={{ marginTop: '0.2rem' }}>{children}</div>
    </div>
  );
}

function Section({ title, note, children }: { title: string; note: string; children: ReactNode }) {
  return (
    <div style={{ marginBottom: '1.05rem' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', gap: '0.75rem', flexWrap: 'wrap', alignItems: 'baseline' }}>
        <div style={{ fontSize: '0.62rem', fontWeight: 800, letterSpacing: '0.06em', textTransform: 'uppercase', color: 'var(--text-secondary)' }}>
          {title}
        </div>
        <div style={{ fontSize: '0.58rem', color: 'var(--text-secondary)' }}>{note}</div>
      </div>
      <div style={{
        marginTop: '0.35rem',
        padding: '0.15rem 0.15rem 0.35rem',
        borderTop: '1px solid var(--border-color)',
      }}>
        {children}
      </div>
    </div>
  );
}

function NameList({
  rows,
  kind,
  onSelect,
}: {
  rows: NowActionRow[];
  kind: 'stop' | 'target';
  onSelect: (row: NowActionRow) => void;
}) {
  if (rows.length === 0) {
    return <div style={{ fontSize: '0.72rem', color: 'var(--text-secondary)', padding: '0.35rem 0' }}>None</div>;
  }
  return (
    <div>
      {rows.map(r => {
        const dist = kind === 'stop' ? r.toStopPct : r.toTargetPct;
        const barrier = kind === 'stop' ? 'STOP' : 'TARGET';
        const condition = conditionFromMark(r.mark);
        const rec = r.recommendation;
        return (
          <button
            key={r.decisionId || r.ticker}
            type="button"
            onClick={() => onSelect(r)}
            style={{
              display: 'block',
              width: '100%',
              textAlign: 'left',
              background: 'transparent',
              border: 'none',
              padding: '0.55rem 0',
              cursor: 'pointer',
              color: 'inherit',
              borderBottom: '1px solid var(--border-color)',
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', gap: '0.75rem', alignItems: 'baseline' }}>
              <span style={{ fontSize: '0.95rem', fontWeight: 800 }}>
                {tickerLabel(r.ticker)}
                {r.direction ? (
                  <span style={{ color: dirColor(r.direction), fontWeight: 800 }}>{` · ${r.direction}`}</span>
                ) : null}
              </span>
              <span style={{ fontSize: '0.68rem', fontWeight: 800, color: '#0369a1' }}>{rec}</span>
            </div>
            <div style={{ fontSize: '0.68rem', fontWeight: 700, color: conditionColor(condition), marginTop: '0.12rem' }}>
              {conditionCaption(condition)}
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between', gap: '0.75rem', marginTop: '0.16rem' }}>
              <span style={{ fontSize: '0.78rem', fontWeight: 700, color: markColor(r.mark) }}>{fmtPct(r.mark)} mark</span>
              <span style={{ fontSize: '0.75rem', fontWeight: 800 }}>
                {fmtRemaining(dist)} to {barrier}
              </span>
            </div>
            <div style={{ fontSize: '0.62rem', color: 'var(--text-secondary)', marginTop: '0.12rem' }}>
              No lifecycle event · not an invalidation
            </div>
          </button>
        );
      })}
    </div>
  );
}

function LiveActivityFeed({
  items,
  onSelect,
}: {
  items: LiveActivityItem[];
  onSelect: (item: LiveActivityItem) => void;
}) {
  if (items.length === 0) {
    return (
      <div style={{ fontSize: '0.72rem', color: 'var(--text-secondary)', padding: '0.35rem 0' }}>
        No session events yet. The feed copies as-of, ledger, and last mark — it does not invent activity.
      </div>
    );
  }
  return (
    <div>
      {items.map((item, i) => {
        const clickable = !!(item.selectId || item.ticker);
        const Tag = clickable ? 'button' : 'div';
        return (
          <Tag
            key={`${item.kind}-${item.unix}-${item.ticker ?? 'x'}-${i}`}
            type={clickable ? 'button' : undefined}
            onClick={clickable ? () => onSelect(item) : undefined}
            style={{
              display: 'block',
              width: '100%',
              textAlign: 'left',
              background: 'transparent',
              border: 'none',
              padding: '0.55rem 0',
              cursor: clickable ? 'pointer' : 'default',
              color: 'inherit',
              borderBottom: '1px solid var(--border-color)',
            }}
          >
            <div style={{ fontSize: '0.62rem', fontWeight: 700, letterSpacing: '0.04em', color: 'var(--text-secondary)' }}>
              {fmtIstHms(item.unix)}
            </div>
            <div style={{ fontSize: '0.92rem', fontWeight: 800, color: 'var(--text-primary)', marginTop: '0.12rem' }}>
              {item.headline}
            </div>
            <div style={{ fontSize: '0.72rem', fontWeight: 700, color: dirColor(item.direction ?? undefined), marginTop: '0.08rem' }}>
              {item.detail}
            </div>
          </Tag>
        );
      })}
    </div>
  );
}

function OutcomeAnalyticsPanel({
  analytics,
  nOpen,
  onSelect,
}: {
  analytics: OutcomeAnalytics;
  nOpen: number;
  onSelect: (item: OutcomeClosedRow) => void;
}) {
  if (analytics.nClosed === 0) {
    return (
      <div style={{ padding: '0.35rem 0 0.2rem' }}>
        <div style={{ fontSize: '1.05rem', fontWeight: 800, color: 'var(--text-primary)' }}>
          No realized outcomes yet.
        </div>
        <div style={{ fontSize: '0.78rem', color: 'var(--text-secondary)', marginTop: '0.28rem', lineHeight: 1.5 }}>
          Analytics will appear after TARGET, STOP, or HORIZON events are recorded.
        </div>
        <div style={{ fontSize: '0.65rem', color: 'var(--text-secondary)', marginTop: '0.4rem', lineHeight: 1.45 }}>
          {nOpen} OPEN {nOpen === 1 ? 'is' : 'are'} not an outcome.
          Open marks, CAUTION, and elapsed time are not measured here.
        </div>
      </div>
    );
  }

  return (
    <div>
      <div style={{ fontSize: '0.78rem', fontWeight: 700, color: 'var(--text-primary)' }}>
        {analytics.nClosed} realized
        {' · '}{analytics.nTarget} TARGET
        {' · '}{analytics.nStop} STOP
        {' · '}{analytics.nHorizon} HORIZON
      </div>
      <div style={{ marginTop: '0.55rem' }}>
        <div style={{ fontSize: '0.58rem', fontWeight: 800, letterSpacing: '0.05em', textTransform: 'uppercase', color: 'var(--text-secondary)' }}>
          Mean realized
        </div>
        <div style={{ fontSize: '1.15rem', fontWeight: 800, color: markColor(analytics.meanRealized), marginTop: '0.1rem' }}>
          {fmtPct(analytics.meanRealized)}
        </div>
        <div style={{ fontSize: '0.62rem', color: 'var(--text-secondary)', marginTop: '0.08rem' }}>
          {analytics.nWithRealized} closed with a ledger realized return · equal-weight · not open mark
        </div>
      </div>
      <div style={{ marginTop: '0.55rem' }}>
        {analytics.closed.map(item => (
          <button
            key={item.selectId || item.ticker}
            type="button"
            onClick={() => onSelect(item)}
            style={{
              display: 'block', width: '100%', textAlign: 'left', background: 'transparent',
              border: 'none', padding: '0.45rem 0', cursor: 'pointer', color: 'inherit',
              borderBottom: '1px solid var(--border-color)',
            }}
          >
            <div style={{ fontSize: '0.62rem', fontWeight: 700, color: 'var(--text-secondary)' }}>
              {fmtIstHm(item.unix)}
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between', gap: '0.75rem', marginTop: '0.08rem' }}>
              <span style={{ fontSize: '0.9rem', fontWeight: 800 }}>
                {tickerLabel(item.ticker)}
                {item.direction ? (
                  <span style={{ color: dirColor(item.direction) }}>{` · ${item.direction}`}</span>
                ) : null}
              </span>
              <span style={{ fontSize: '0.72rem', fontWeight: 800 }}>{item.outcome}</span>
            </div>
            <div style={{ fontSize: '0.72rem', fontWeight: 700, color: markColor(item.realizedReturn), marginTop: '0.08rem' }}>
              {item.realizedReturn == null ? 'realized —' : fmtPct(item.realizedReturn)}
            </div>
          </button>
        ))}
      </div>
    </div>
  );
}

function DecisionJournal({
  entries,
  filter,
  onFilter,
  onSelect,
}: {
  entries: DecisionJournalEntry[];
  filter: 'ACT' | 'NOT_ACT' | 'ALL';
  onFilter: (f: 'ACT' | 'NOT_ACT' | 'ALL') => void;
  onSelect: (entry: DecisionJournalEntry) => void;
}) {
  const nAct = entries.filter(e => e.decisionKind === 'ACT').length;
  const nNot = entries.filter(e => e.decisionKind === 'NOT_ACT').length;
  const nRealized = entries.filter(e =>
    e.paperState === 'TARGET' || e.paperState === 'STOP' || e.paperState === 'HORIZON',
  ).length;
  const visible = entries.filter(e =>
    filter === 'ALL' ? true : e.decisionKind === filter,
  );
  return (
    <div>
      <div style={{ fontSize: '0.78rem', fontWeight: 700, color: 'var(--text-primary)' }}>
        {entries.length} decisions · {nAct} ACT · {nNot} NOT_ACT · {nRealized} realized
      </div>
      <div style={{ fontSize: '0.62rem', color: 'var(--text-secondary)', marginTop: '0.12rem' }}>
        Outcome stays — until TARGET / STOP / HORIZON. T0 Watch AVOID is not the as-of decision.
      </div>
      <div style={{ display: 'flex', gap: '0.4rem', marginTop: '0.55rem', flexWrap: 'wrap' }}>
        {(['ACT', 'NOT_ACT', 'ALL'] as const).map(f => (
          <button
            key={f}
            type="button"
            onClick={() => onFilter(f)}
            style={{
              background: filter === f ? '#0369a114' : 'transparent',
              border: `1px solid ${filter === f ? '#0369a166' : 'var(--border-color)'}`,
              borderRadius: '999px',
              padding: '0.18rem 0.6rem',
              cursor: 'pointer',
              fontSize: '0.62rem',
              fontWeight: 800,
              letterSpacing: '0.04em',
              color: filter === f ? '#0369a1' : 'var(--text-secondary)',
            }}
          >
            {f === 'ALL' ? `ALL · ${entries.length}` : f === 'ACT' ? `ACT · ${nAct}` : `NOT_ACT · ${nNot}`}
          </button>
        ))}
      </div>
      <div style={{ marginTop: '0.35rem', maxHeight: '28rem', overflowY: 'auto' }}>
        {visible.length === 0 ? (
          <div style={{ fontSize: '0.72rem', color: 'var(--text-secondary)', padding: '0.45rem 0' }}>
            No journal rows in this filter. The journal copies as-of events; it does not invent decisions.
          </div>
        ) : visible.map(e => (
          <button
            key={e.decisionId || e.ticker}
            type="button"
            onClick={() => onSelect(e)}
            style={{
              display: 'block', width: '100%', textAlign: 'left', background: 'transparent',
              border: 'none', padding: '0.5rem 0', cursor: 'pointer', color: 'inherit',
              borderBottom: '1px solid var(--border-color)',
            }}
          >
            <div style={{ fontSize: '0.62rem', fontWeight: 700, color: 'var(--text-secondary)' }}>
              {fmtIstHm(e.decisionUnix)}
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between', gap: '0.75rem', marginTop: '0.1rem' }}>
              <span style={{ fontSize: '0.9rem', fontWeight: 800 }}>
                {tickerLabel(e.ticker)}
                {e.direction ? (
                  <span style={{ color: dirColor(e.direction) }}>{` · ${e.direction}`}</span>
                ) : null}
              </span>
              <span style={{ fontSize: '0.68rem', fontWeight: 800, color: e.decisionKind === 'ACT' ? '#0369a1' : '#6b7280' }}>
                {e.decisionKind === 'ACT' ? e.entryAction || 'ACT' : e.entryAction || 'NOT_ACT'}
              </span>
            </div>
            <div style={{ fontSize: '0.68rem', color: 'var(--text-secondary)', marginTop: '0.1rem' }}>
              {e.decisionKind === 'ACT'
                ? `${e.paperState}${e.oqs != null && e.oqs > 0 ? ` · OQS ${e.oqs}` : ''} · outcome ${journalOutcome(e)}`
                : `not in the paper book · outcome —`}
            </div>
          </button>
        ))}
      </div>
    </div>
  );
}

function journalOutcome(entry: DecisionJournalEntry): string {
  if (entry.paperState === 'TARGET' || entry.paperState === 'STOP' || entry.paperState === 'HORIZON') {
    return entry.realizedReturn == null ? entry.paperState : `${entry.paperState} · ${fmtPct(entry.realizedReturn)}`;
  }
  return '—';
}

function JournalEntryDetail({
  row,
  entry,
  brief,
  onBack,
}: {
  row: NowActionRow;
  entry: DecisionJournalEntry | null;
  brief: DecisionBrief | null;
  onBack: () => void;
}) {
  const rec = row.recommendation;
  const condition = conditionFromMark(row.mark);
  const caption = conditionCaption(condition);
  const openHold = rec === 'HOLD' && row.paperState === 'OPEN';
  const decision = entry;
  const events = decision?.lifecycleEvents ?? [];
  const closed = row.paperState === 'TARGET' || row.paperState === 'STOP' || row.paperState === 'HORIZON';
  const evidence: [string, string][] = [];
  if (decision?.offer) evidence.push(['As-of offer', decision.offer]);
  if (decision?.entryAction) evidence.push(['Entry action', decision.entryAction]);
  if (decision?.oqs != null && decision.oqs > 0) evidence.push(['OQS at as-of', String(decision.oqs)]);
  if (decision?.direction) evidence.push(['Direction', decision.direction]);
  if (decision?.watchId) evidence.push(['Watch admitted', decision.watchId]);
  if (decision?.fill != null) evidence.push(['Paper fill', fmtPrice(decision.fill)]);
  if (decision?.stop != null) evidence.push(['Frozen STOP', fmtPrice(decision.stop)]);
  if (decision?.target != null) evidence.push(['Frozen TARGET', fmtPrice(decision.target)]);
  if (evidence.length === 0) {
    evidence.push(['Evidence', 'No as-of or paper facts on this row']);
  }

  return (
    <div style={{ maxWidth: '640px', margin: '0 auto', padding: '0.25rem 0 2rem', fontVariantNumeric: 'tabular-nums' }}>
      <button
        type="button"
        onClick={onBack}
        style={{
          background: 'transparent', border: 'none', padding: 0, cursor: 'pointer',
          color: '#0369a1', fontSize: '0.72rem', fontWeight: 700, letterSpacing: '0.04em',
          textTransform: 'uppercase', marginBottom: '0.85rem',
        }}
      >
        ← Today
      </button>

      <div style={{ fontSize: '1.35rem', fontWeight: 800 }}>
        {tickerLabel(row.ticker)}
      </div>
      {row.direction || decision?.direction ? (
        <div style={{ fontSize: '0.82rem', fontWeight: 800, color: dirColor(row.direction || decision?.direction), marginTop: '0.12rem' }}>
          {row.direction || decision?.direction}
        </div>
      ) : null}

      <div style={{
        marginTop: '0.85rem',
        padding: '0.85rem 0.95rem',
        borderRadius: '10px',
        border: '1px solid #0369a155',
        backgroundColor: '#0369a114',
      }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', gap: '0.75rem', alignItems: 'baseline' }}>
          <div>
            <div style={{ fontSize: '0.58rem', fontWeight: 800, letterSpacing: '0.06em', textTransform: 'uppercase', color: '#0369a1' }}>
              Recommendation
            </div>
            <div style={{ fontSize: '1.45rem', fontWeight: 800, color: '#0369a1', marginTop: '0.15rem' }}>{rec}</div>
          </div>
          <div style={{ textAlign: 'right' }}>
            <div style={{ fontSize: '0.58rem', fontWeight: 800, letterSpacing: '0.06em', textTransform: 'uppercase', color: 'var(--text-secondary)' }}>
              Lifecycle
            </div>
            <div style={{ fontSize: '1.05rem', fontWeight: 800, color: 'var(--text-primary)', marginTop: '0.15rem' }}>
              {row.paperState}
            </div>
          </div>
        </div>
        {openHold || rec === 'HOLD' ? (
          <>
            <div style={{ fontSize: '0.78rem', fontWeight: 700, color: conditionColor(condition), marginTop: '0.45rem' }}>
              {caption}
            </div>
            <div style={{ fontSize: '0.65rem', color: 'var(--text-secondary)', marginTop: '0.12rem' }}>
              {openHold ? `${row.direction || 'Position'} remains within frozen trade geometry` : row.paperState}
              {row.lastTickAt != null ? ` · last ${fmtIstHm(row.lastTickAt)}` : ''}
            </div>
          </>
        ) : closed ? (
          <div style={{ fontSize: '0.65rem', color: 'var(--text-secondary)', marginTop: '0.45rem' }}>
            <div style={{ fontSize: '1.45rem', fontWeight: 800, color: '#0369a1', marginTop: '0.15rem' }}>
              {entry?.entryAction || entry?.decisionKind || row.action}
            </div>
          </div>
        ) : (
          <div style={{ fontSize: '0.65rem', color: 'var(--text-secondary)', marginTop: '0.45rem' }}>
            Current recommendation is copied from the ledger. It is not a new decision.
          </div>
        )}
      </div>

      <JournalStage title="1 · Decision" note="As-of at decision time · not T0 Watch AVOID">
        <div style={{ fontSize: '0.95rem', fontWeight: 800 }}>
          {decision?.decisionKind === 'NOT_ACT' ? 'NOT_ACT' : (decision?.entryAction || row.action)}
          {decision?.direction || row.direction ? ` · ${decision?.direction || row.direction}` : ''}
        </div>
        <div style={{ fontSize: '0.72rem', color: 'var(--text-secondary)', marginTop: '0.18rem' }}>
          {fmtIstFull(decision?.decisionUnix ?? row.enteredAt ?? row.snapUnix)}
        </div>
        <div style={{ fontSize: '0.72rem', fontWeight: 700, marginTop: '0.18rem', wordBreak: 'break-all' }}>
          {decision?.decisionId || row.asofDecisionId || row.decisionId || '—'}
        </div>
      </JournalStage>

      <JournalStage title="2 · Evidence at decision time" note="Copied facts only · OQS is as-of, not T0">
        <FactTable rows={evidence} />
      </JournalStage>

      <JournalStage title="3 · Paper lifecycle" note="PAPER_ENTER / TARGET / STOP / HORIZON only">
        {events.length === 0 ? (
          <div style={{ fontSize: '0.78rem', color: 'var(--text-secondary)' }}>
            {row.paperState === 'NONE' ? 'Not in the paper book.' : 'No ledger lifecycle events yet.'}
          </div>
        ) : (
          events.map((ev, i) => (
            <div
              key={`${ev.kind}-${ev.unix}-${i}`}
              style={{
                padding: '0.4rem 0',
                borderBottom: i === events.length - 1 ? 'none' : '1px solid var(--border-color)',
              }}
            >
              <div style={{ fontSize: '0.62rem', fontWeight: 700, color: 'var(--text-secondary)' }}>
                {fmtIstHm(ev.unix)}
              </div>
              <div style={{ fontSize: '0.85rem', fontWeight: 800, marginTop: '0.08rem' }}>{ev.kind}</div>
              <div style={{ fontSize: '0.72rem', color: 'var(--text-secondary)', marginTop: '0.06rem' }}>
                {ev.price != null ? fmtPrice(ev.price) : '—'}
                {ev.note ? ` · ${ev.note}` : ''}
              </div>
            </div>
          ))
        )}
        <div style={{ fontSize: '0.68rem', color: 'var(--text-secondary)', marginTop: '0.35rem' }}>
          Current state {row.paperState}
        </div>
      </JournalStage>

      <JournalStage title="4 · Realized outcome" note="Empty until TARGET / STOP / HORIZON">
        <div style={{ fontSize: '1.15rem', fontWeight: 800, color: closed ? markColor(decision?.realizedReturn) : 'var(--text-primary)' }}>
          {closed
            ? (decision?.realizedReturn == null
              ? row.paperState
              : `${row.paperState} · ${fmtPct(decision.realizedReturn)}`)
            : '—'}
        </div>
        <div style={{ fontSize: '0.68rem', color: 'var(--text-secondary)', marginTop: '0.12rem' }}>
          {closed
            ? 'Copied from the ledger. Not inferred from the open mark.'
            : row.paperState === 'OPEN'
              ? 'Still OPEN. Mark-to-fill is not an outcome.'
              : 'No realized outcome. This journal does not invent one.'}
        </div>
      </JournalStage>

      {row.paperState === 'OPEN' && (
        <div style={{ marginTop: '0.9rem' }}>
          <div style={{ fontSize: '0.62rem', fontWeight: 800, letterSpacing: '0.06em', textTransform: 'uppercase', color: 'var(--text-secondary)' }}>
            Now · informational
          </div>
          <div style={{ marginTop: '0.35rem', fontSize: '0.92rem', fontWeight: 700 }}>
            {row.paperEntry != null && row.currentPrice != null
              ? `${fmtPrice(row.paperEntry)} → ${fmtPrice(row.currentPrice)}`
              : fmtPrice(row.currentPrice ?? row.paperEntry)}
          </div>
          <div style={{ fontSize: '0.82rem', fontWeight: 800, color: markColor(row.mark), marginTop: '0.12rem' }}>
            {fmtPct(row.mark)} mark-to-fill · not realized
          </div>
          <FactTable rows={[
            ['STOP remaining', row.toStopPct != null ? `${fmtRemaining(row.toStopPct)} · ${fmtPrice(row.risk)}` : fmtPrice(row.risk)],
            ['TARGET remaining', row.toTargetPct != null ? `${fmtRemaining(row.toTargetPct)} · ${fmtPrice(row.target)}` : fmtPrice(row.target)],
          ]} />
          <div style={{
            marginTop: '0.75rem',
            padding: '0.7rem 0.8rem',
            borderRadius: '8px',
            border: '1px solid var(--border-color)',
            fontSize: '0.72rem',
            lineHeight: 1.5,
          }}>
            {condition === 'ADVERSE' && (
              <div style={{ fontWeight: 700, color: '#b45309' }}>Adverse mark-to-fill observed</div>
            )}
            {condition === 'FAVOURABLE' && (
              <div style={{ fontWeight: 700, color: '#16a34a' }}>Mark-to-fill moving with thesis</div>
            )}
            {condition === 'NEUTRAL' && (
              <div style={{ fontWeight: 700 }}>Mark-to-fill is flat</div>
            )}
            <div style={{ color: 'var(--text-secondary)', marginTop: '0.2rem' }}>
              Not an invalidation · No lifecycle event · CS-P-001-C remains off
            </div>
          </div>
        </div>
      )}

      <div style={{ fontSize: '0.62rem', color: 'var(--text-secondary)', marginTop: '1.1rem', lineHeight: 1.45 }}>
        Journal is observational. Frozen LIVE-005 barriers remain authoritative.
        {brief?.id ? ` Watch id ${brief.id} is lineage, not a T0 ACT.` : ''}
        {' '}HOLD is not a health claim. Adverse mark-to-fill is not an exit.
      </div>
    </div>
  );
}

function JournalStage({ title, note, children }: { title: string; note: string; children: ReactNode }) {
  return (
    <div style={{ marginTop: '1.05rem' }}>
      <div style={{ fontSize: '0.62rem', fontWeight: 800, letterSpacing: '0.06em', textTransform: 'uppercase', color: 'var(--text-secondary)' }}>
        {title}
      </div>
      <div style={{ fontSize: '0.58rem', color: 'var(--text-secondary)', marginTop: '0.08rem' }}>{note}</div>
      <div style={{ marginTop: '0.4rem' }}>{children}</div>
    </div>
  );
}

function FactTable({ rows }: { rows: [string, string][] }) {
  return (
    <div style={{ marginTop: '0.9rem' }}>
      {rows.map(([k, v]) => (
        <div
          key={k}
          style={{
            display: 'flex', justifyContent: 'space-between', gap: '1rem',
            padding: '0.32rem 0',
            borderBottom: '1px solid var(--border-color)',
            fontSize: '0.78rem',
          }}
        >
          <span style={{ color: 'var(--text-secondary)' }}>{k}</span>
          <span style={{ fontWeight: 700, textAlign: 'right', color: k === 'P&L' ? markColor(parsePct(v)) : 'var(--text-primary)' }}>{v}</span>
        </div>
      ))}
    </div>
  );
}

function parsePct(s: string): number | null {
  if (!s || s === '—') return null;
  const n = Number(s.replace('%', '').replace('+', ''));
  return Number.isFinite(n) ? n / 100 : null;
}
