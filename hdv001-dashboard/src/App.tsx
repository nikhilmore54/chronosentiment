import './index.css'

// ── Frozen evidence data (hardcoded from certified artifacts) ─────────────────
const DATA = {
  total: 1144,
  complete: 728,
  maturing: 416,
  instruments: 52,

  coralys_rate: 34.9,
  coralys_hits: 260,

  random_rate: 28.8,
  random_hits: 210,
  margin_random: 6.0,

  inverse_rate: 22.5,
  inverse_hits: 164,
  margin_inverse: 12.4,

  momentum_rate: 34.1,
  momentum_hits: 248,
  margin_momentum: 0.8,

  segments: [
    { name: 'Bullish + Positive', n: 130, coralys: 40.8, random: 29.1, inverse: 17.7, pass: true },
    { name: 'Bullish + Negative', n: 113, coralys: 25.7, random: 34.5, inverse: 39.8, pass: false },
    { name: 'Bearish + Positive', n: 190, coralys: 37.9, random: 36.8, inverse: 20.0, pass: false },
    { name: 'Bearish + Negative', n: 295, coralys: 30.3, random: 22.2, inverse: 21.3, pass: true },
  ],

  mfe: [
    { session: 'Session 1',  mfe: 0.138 },
    { session: 'Session 2',  mfe: 0.658 },
    { session: 'Session 3',  mfe: 1.037 },
    { session: 'Session 5',  mfe: 1.723 },
    { session: 'Session 10', mfe: 2.803 },
  ],

  commit: 'a7919ce06',
  frozen_date: '2026-08-17',
}

const MAX_MFE = 3.0

function App() {
  return (
    <>
      {/* ── Header ── */}
      <div className="header">
        <span className="header-id">HDV-001</span>
        <span className="header-sep">|</span>
        <span className="header-sub">Development Evidence Dashboard</span>
        <span className="badge badge-frozen">Frozen</span>
        <span className="badge badge-pass">Official Determination: PASS</span>
      </div>

      <div className="dashboard">

        {/* ── KPI row ── */}
        <div className="kpi-row">
          <div className="kpi">
            <div className="kpi-label">Complete Decisions</div>
            <div className="kpi-value white">{DATA.complete}</div>
          </div>
          <div className="kpi">
            <div className="kpi-label">Coralys Target Rate</div>
            <div className="kpi-value blue">{DATA.coralys_rate}%</div>
          </div>
          <div className="kpi">
            <div className="kpi-label">Random Baseline</div>
            <div className="kpi-value white">{DATA.random_rate}%</div>
          </div>
          <div className="kpi">
            <div className="kpi-label">vs Random</div>
            <div className="kpi-value green">+{DATA.margin_random} pp</div>
          </div>
          <div className="kpi">
            <div className="kpi-label">Inverse Baseline</div>
            <div className="kpi-value white">{DATA.inverse_rate}%</div>
          </div>
          <div className="kpi">
            <div className="kpi-label">vs Inverse</div>
            <div className="kpi-value green">+{DATA.margin_inverse} pp</div>
          </div>
        </div>

        {/* ── Gate cards ── */}
        <div className="section">
          <div className="section-label">Official Criterion — HDV-001-G Gate 6</div>
          <div className="gate-row">
            <div className="gate-card">
              <div className="gate-name">Coralys vs Random</div>
              <div className="gate-margin">+{DATA.margin_random} pp</div>
              <div className="gate-status">✓ PASS (≥ 5 pp)</div>
            </div>
            <div className="gate-card">
              <div className="gate-name">Coralys vs Inverse</div>
              <div className="gate-margin">+{DATA.margin_inverse} pp</div>
              <div className="gate-status">✓ PASS (≥ 5 pp)</div>
            </div>
            <div className="gate-card">
              <div className="gate-name">State Segments</div>
              <div className="gate-margin">2 / 4</div>
              <div className="gate-status">✓ PASS (≥ 2 of 4)</div>
            </div>
          </div>
          <div className="determination">
            HDV-001-F — OFFICIAL DETERMINATION: PASS
          </div>
        </div>

        {/* ── State segmentation chart ── */}
        <div className="section">
          <div className="section-label">State Segmentation — TARGET_BEFORE_RISK %</div>
          <div className="card">
            {DATA.segments.map(seg => (
              <div className="state-group" key={seg.name}>
                <div className="state-name">
                  {seg.name}
                  {seg.pass && <span className="state-pass-tag">PASS</span>}
                  <span style={{ fontSize: '0.6rem', color: 'var(--text-muted)', marginLeft: 'auto' }}>N={seg.n}</span>
                </div>
                {[
                  { label: 'Coralys', pct: seg.coralys, cls: 'bar-coralys' },
                  { label: 'Random',  pct: seg.random,  cls: 'bar-random'  },
                  { label: 'Inverse', pct: seg.inverse, cls: 'bar-inverse' },
                ].map(b => (
                  <div className="bar-row" key={b.label}>
                    <span className="bar-label">{b.label}</span>
                    <div className="bar-track">
                      <div
                        className={`bar-fill ${b.cls}`}
                        style={{ width: `${b.pct}%` }}
                      >
                        {b.pct.toFixed(1)}%
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            ))}
          </div>
        </div>

        {/* ── Momentum contextual ── */}
        <div className="section">
          <div className="section-label">Momentum Baseline — Contextual Only</div>
          <div className="momentum-card">
            <div className="momentum-stat">
              <div className="momentum-value">{DATA.momentum_rate}%</div>
              <div className="momentum-sublabel">Momentum (MA-20)</div>
            </div>
            <div className="momentum-stat">
              <div className="momentum-value">{DATA.coralys_rate}%</div>
              <div className="momentum-sublabel">Coralys</div>
            </div>
            <div className="momentum-stat">
              <div className="momentum-value" style={{ color: 'var(--text-muted)' }}>+{DATA.margin_momentum} pp</div>
              <div className="momentum-sublabel">Advantage</div>
            </div>
            <div className="momentum-note">
              Baseline C (20-session MA crossover) is not part of the frozen HDV-001-G success criterion.
              The +{DATA.margin_momentum} pp margin is a diagnostic for HDV-002 risk-boundary research.
              It must not be used as an optimisation target.
            </div>
          </div>
        </div>

        {/* ── Frozen evidence ── */}
        <div className="section">
          <div className="section-label">Frozen Evidence</div>
          <div style={{ display: 'flex', gap: '0.75rem', flexWrap: 'wrap' }}>
            <div className="evidence-grid" style={{ flex: '1', minWidth: '260px' }}>
              <div className="evidence-stat-card">
                <div className="evidence-num">{DATA.total.toLocaleString()}</div>
                <div className="evidence-lbl">Total Decisions</div>
              </div>
              <div className="evidence-stat-card">
                <div className="evidence-num">{DATA.complete}</div>
                <div className="evidence-lbl">Complete</div>
              </div>
              <div className="evidence-stat-card">
                <div className="evidence-num">{DATA.maturing}</div>
                <div className="evidence-lbl">Maturing</div>
              </div>
              <div className="evidence-stat-card">
                <div className="evidence-num">{DATA.instruments}</div>
                <div className="evidence-lbl">Instruments</div>
              </div>
            </div>
            <div className="card" style={{ flex: '1', minWidth: '260px' }}>
              <ul className="check-list">
                {[
                  'Price cache certified',
                  'Temporal integrity verified',
                  'Metric integrity verified',
                  'Outcome integrity verified',
                  'Reproducibility verified',
                  'Freeze gate passed (6/6)',
                  'Baseline geometry corrected',
                  'Same-bar rule applied (6 cases)',
                ].map(item => (
                  <li key={item}>
                    <span className="check-icon">✓</span>
                    {item}
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </div>

        {/* ── MFE chart ── */}
        <div className="section">
          <div className="section-label">Median MFE by Session (Direction-Normalised)</div>
          <div className="card">
            {DATA.mfe.map(row => (
              <div className="mfe-row" key={row.session}>
                <span className="mfe-session">{row.session}</span>
                <div className="mfe-track">
                  <div
                    className="mfe-fill"
                    style={{ width: `${(row.mfe / MAX_MFE) * 100}%` }}
                  >
                    +{row.mfe.toFixed(3)}%
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* ── Footer ── */}
        <div className="footer">
          <div className="footer-text">
            HDV-001 is frozen. C3-002 and reference-risk boundaries remain unchanged.<br />
            Official determination commit: <span className="footer-mono">{DATA.commit}</span><br />
            Frozen: {DATA.frozen_date} &nbsp;·&nbsp; HDV-002-A methodology frozen: <span className="footer-mono">901ee439c</span><br />
            HDV-002-B pending ≥200 COMPLETE validation decisions (validation period opens 2026-08-18).
          </div>
        </div>

      </div>
    </>
  )
}

export default App