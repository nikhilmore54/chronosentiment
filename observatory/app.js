// ChronoSentiment Observatory — Application Logic
let DATA = null;

document.addEventListener('DOMContentLoaded', () => {
    fetch('data.json').then(r => r.json()).then(data => {
        DATA = data;
        initTabs();
        renderEcology(data);
        // Defer hidden tabs — they render when activated
    }).catch(e => console.error('Failed to load data:', e));
});

// === TAB NAVIGATION ===
const rendered = { ecology: true };
function initTabs() {
    document.querySelectorAll('.tab').forEach(tab => {
        tab.addEventListener('click', () => {
            document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
            document.querySelectorAll('.tab-content').forEach(p => p.classList.remove('active'));
            tab.classList.add('active');
            const id = tab.dataset.tab;
            document.getElementById('panel-' + id).classList.add('active');
            // Deferred render on first activation
            if (!rendered[id] && DATA) {
                rendered[id] = true;
                setTimeout(() => {
                    if (id === 'smoothness') renderSmoothness(DATA);
                    if (id === 'genesis') renderGenesis(DATA);
                    if (id === 'atlas') renderAtlas(DATA);
                    if (id === 'replay') renderReplay(DATA);
                }, 50);
            }
        });
    });
}

// === COLOR HELPERS ===
const EXIT_COLORS = {
    TakeProfit: '#10b981', TrailingStop: '#3b82f6',
    Mortality: '#6b7280', StopLoss: '#ef4444'
};

function pnlColor(v) { return v > 0 ? '#10b981' : v < 0 ? '#ef4444' : '#94a3b8'; }
function pnlClass(v) { return v > 0 ? 'pnl-positive' : 'pnl-negative'; }

// === CANVAS SCATTER PLOT ===
function drawScatter(containerId, trades, xKey, yKey, xLabel, yLabel, opts = {}) {
    const container = document.getElementById(containerId);
    if (!container) return;
    const canvas = document.createElement('canvas');
    const dpr = window.devicePixelRatio || 1;
    const w = container.clientWidth, h = container.clientHeight;
    canvas.width = w * dpr; canvas.height = h * dpr;
    canvas.style.width = w + 'px'; canvas.style.height = h + 'px';
    container.innerHTML = '';
    container.appendChild(canvas);
    const ctx = canvas.getContext('2d');
    ctx.scale(dpr, dpr);

    const pad = { top: 20, right: 30, bottom: 40, left: 55 };
    const pw = w - pad.left - pad.right, ph = h - pad.top - pad.bottom;

    const xs = trades.map(t => t[xKey]).filter(v => v !== undefined);
    const ys = trades.map(t => t[yKey]).filter(v => v !== undefined);
    if (!xs.length) return;

    let xMin = opts.xMin ?? Math.min(...xs), xMax = opts.xMax ?? Math.max(...xs);
    let yMin = opts.yMin ?? Math.min(...ys), yMax = opts.yMax ?? Math.max(...ys);
    const xR = xMax - xMin || 1, yR = yMax - yMin || 1;

    // Grid
    ctx.strokeStyle = 'rgba(148,163,184,0.08)';
    ctx.lineWidth = 1;
    for (let i = 0; i <= 4; i++) {
        const y = pad.top + (ph / 4) * i;
        ctx.beginPath(); ctx.moveTo(pad.left, y); ctx.lineTo(pad.left + pw, y); ctx.stroke();
    }

    // Zero line
    if (yMin < 0 && yMax > 0) {
        const zy = pad.top + ph - ((0 - yMin) / yR) * ph;
        ctx.strokeStyle = 'rgba(148,163,184,0.2)'; ctx.setLineDash([4,4]);
        ctx.beginPath(); ctx.moveTo(pad.left, zy); ctx.lineTo(pad.left + pw, zy); ctx.stroke();
        ctx.setLineDash([]);
    }

    // Points
    trades.forEach(t => {
        if (t[xKey] === undefined || t[yKey] === undefined) return;
        const px = pad.left + ((t[xKey] - xMin) / xR) * pw;
        const py = pad.top + ph - ((t[yKey] - yMin) / yR) * ph;
        const color = opts.colorBy === 'exit' ? (EXIT_COLORS[t.exit_type] || '#6b7280')
            : opts.colorBy === 'pnl' ? pnlColor(t.pnl_bps) : '#3b82f6';
        ctx.globalAlpha = 0.7;
        ctx.beginPath(); ctx.arc(px, py, 5, 0, Math.PI * 2);
        ctx.fillStyle = color; ctx.fill();
        ctx.strokeStyle = 'rgba(255,255,255,0.15)'; ctx.lineWidth = 1; ctx.stroke();
    });
    ctx.globalAlpha = 1;

    // Axes labels
    ctx.fillStyle = '#64748b'; ctx.font = '11px Inter, sans-serif';
    ctx.textAlign = 'center';
    ctx.fillText(xLabel, pad.left + pw / 2, h - 8);
    ctx.save(); ctx.translate(14, pad.top + ph / 2);
    ctx.rotate(-Math.PI / 2); ctx.fillText(yLabel, 0, 0); ctx.restore();

    // Tick labels
    ctx.font = '10px JetBrains Mono, monospace'; ctx.fillStyle = '#64748b';
    ctx.textAlign = 'center';
    for (let i = 0; i <= 4; i++) {
        const v = xMin + (xR / 4) * i;
        ctx.fillText(v.toFixed(2), pad.left + (pw / 4) * i, h - 22);
    }
    ctx.textAlign = 'right';
    for (let i = 0; i <= 4; i++) {
        const v = yMin + (yR / 4) * i;
        ctx.fillText(v.toFixed(1), pad.left - 8, pad.top + ph - (ph / 4) * i + 4);
    }

    // Legend
    if (opts.colorBy === 'exit') {
        const legend = document.createElement('div');
        legend.className = 'scatter-legend';
        for (const [name, color] of Object.entries(EXIT_COLORS)) {
            legend.innerHTML += `<div class="legend-item"><div class="legend-dot" style="background:${color}"></div>${name}</div>`;
        }
        container.appendChild(legend);
    }
}

// === BAR CHART ===
function drawBars(containerId, items) {
    const container = document.getElementById(containerId);
    if (!container) return;
    container.innerHTML = '';
    const maxVal = Math.max(...items.map(i => Math.abs(i.value)), 1);
    items.forEach(item => {
        const pct = (Math.abs(item.value) / maxVal) * 100;
        const row = document.createElement('div');
        row.className = 'bar-row';
        row.innerHTML = `
            <div class="bar-label">${item.label}</div>
            <div class="bar-track">
                <div class="bar-fill" style="width:${pct}%;background:${item.color || '#3b82f6'}">
                    <span>${item.text || item.value.toFixed(1)}</span>
                </div>
            </div>`;
        container.appendChild(row);
    });
    // Animate
    setTimeout(() => container.querySelectorAll('.bar-fill').forEach(b => b.style.width = b.style.width), 50);
}

// === RENDER: ECOLOGY TAB ===
function renderEcology(data) {
    const s = data.summary;
    if (!s) return;

    // Header chips
    document.getElementById('header-trades').textContent = s.total_trades;
    document.getElementById('header-exp').textContent = s.expectancy_bps + ' bps';
    document.getElementById('header-exp').style.color = pnlColor(s.expectancy_bps);
    document.getElementById('header-err').textContent = s.elastic_recovery_ratio;

    // Metric cards
    document.getElementById('metric-trades').textContent = s.total_trades;
    document.getElementById('metric-winloss').textContent = `${s.winners} W / ${s.losers} L`;
    document.getElementById('metric-winrate').textContent = s.win_rate + '%';
    document.getElementById('metric-winrate').style.color = s.win_rate > 35 ? '#10b981' : '#f59e0b';
    document.getElementById('metric-exp').textContent = s.expectancy_bps + ' bps';
    document.getElementById('metric-exp').style.color = pnlColor(s.expectancy_bps);
    document.getElementById('metric-payoff').textContent = `Avg W/L: ${s.avg_win_bps} / ${s.avg_loss_bps}`;
    document.getElementById('metric-err').textContent = s.elastic_recovery_ratio;
    document.getElementById('metric-err').style.color = s.elastic_recovery_ratio > 1 ? '#10b981' : '#f59e0b';

    // Exit ecology bars
    const exitItems = [
        { label: 'Mortality', value: s.exit_distribution.Mortality || 0, color: EXIT_COLORS.Mortality },
        { label: 'TrailingStop', value: s.exit_distribution.TrailingStop || 0, color: EXIT_COLORS.TrailingStop },
        { label: 'TakeProfit', value: s.exit_distribution.TakeProfit || 0, color: EXIT_COLORS.TakeProfit },
        { label: 'StopLoss', value: s.exit_distribution.StopLoss || 0, color: EXIT_COLORS.StopLoss },
    ];
    exitItems.forEach(i => i.text = `${i.value} (${(i.value / s.total_trades * 100).toFixed(1)}%)`);
    drawBars('exit-ecology-chart', exitItems);

    // Asset bars
    const assetItems = Object.entries(s.asset_stats).map(([sym, st]) => ({
        label: sym, value: st.trades,
        color: sym === 'BTC-USD' ? '#f59e0b' : sym === 'ETH-USD' ? '#8b5cf6' : '#06b6d4',
        text: `${st.trades} trades · ${st.win_rate}% WR · ${st.avg_pnl} bps`
    }));
    drawBars('asset-chart', assetItems);

    // PnL distribution
    drawScatter('pnl-distribution-chart', data.trades, 'rec_id', 'pnl_bps', 'Trade #', 'PnL (bps)', { colorBy: 'exit' });
}

// === RENDER: SMOOTHNESS TRAP ===
function renderSmoothness(data) {
    const trades = data.trades.filter(t => t.eff !== undefined);

    // Scatter
    drawScatter('smoothness-scatter', trades, 'eff', 'pnl_bps', 'Directional Efficiency', 'PnL (bps)', { colorBy: 'exit' });

    // Efficiency by exit type bars
    const groups = {};
    trades.forEach(t => {
        if (!groups[t.exit_type]) groups[t.exit_type] = [];
        groups[t.exit_type].push(t.eff);
    });
    const items = Object.entries(groups).map(([exit, vals]) => ({
        label: exit, value: vals.reduce((a, b) => a + b, 0) / vals.length,
        color: EXIT_COLORS[exit] || '#6b7280',
        text: (vals.reduce((a, b) => a + b, 0) / vals.length).toFixed(3)
    })).sort((a, b) => b.value - a.value);
    drawBars('efficiency-bars', items);

    // Inversion table
    const table = document.getElementById('inversion-table');
    let html = '<table class="data-table"><thead><tr><th>Exit Type</th><th>Trades</th><th>Avg Efficiency</th><th>Avg PnL (bps)</th><th>Interpretation</th></tr></thead><tbody>';
    const exitOrder = ['StopLoss', 'Mortality', 'TrailingStop', 'TakeProfit'];
    const interp = { StopLoss: 'Terminal — Smoothness Trap', Mortality: 'Exhausted continuation', TrailingStop: 'Durable elastic harvesting', TakeProfit: 'Convex asymmetry capture' };
    exitOrder.forEach(exit => {
        const g = trades.filter(t => t.exit_type === exit);
        if (!g.length) return;
        const avgEff = (g.reduce((s, t) => s + t.eff, 0) / g.length).toFixed(4);
        const avgPnl = (g.reduce((s, t) => s + t.pnl_bps, 0) / g.length).toFixed(1);
        html += `<tr><td><span class="exit-badge ${exit}">${exit}</span></td><td>${g.length}</td><td>${avgEff}</td><td class="${pnlClass(+avgPnl)}">${avgPnl}</td><td>${interp[exit] || ''}</td></tr>`;
    });
    html += '</tbody></table>';
    table.innerHTML = html;
}

// === RENDER: EDGE GENESIS ===
function renderGenesis(data) {
    const trades = data.trades.filter(t => t.comp !== undefined);

    drawScatter('compression-scatter', trades, 'comp', 'pnl_bps', 'Compression Ratio (exec/pre)', 'PnL (bps)', { colorBy: 'exit' });
    drawScatter('bias-scatter', trades, 'bias', 'pnl_bps', 'Pre-Entry Directional Bias', 'PnL (bps)', { colorBy: 'pnl' });

    // TP genesis stats
    const tp = trades.filter(t => t.exit_type === 'TakeProfit');
    if (tp.length) {
        const avg = (arr, k) => arr.reduce((s, t) => s + t[k], 0) / arr.length;
        document.getElementById('gen-tp-comp').textContent = avg(tp, 'comp').toFixed(3);
        document.getElementById('gen-tp-bias').textContent = (avg(tp, 'bias') >= 0 ? '+' : '') + avg(tp, 'bias').toFixed(3);
        document.getElementById('gen-tp-age').textContent = avg(tp, 'age').toFixed(1) + ' bars';
    }

    // Genesis table
    const table = document.getElementById('genesis-table');
    let html = '<table class="data-table"><thead><tr><th>Exit</th><th>N</th><th>Avg PnL</th><th>Compression</th><th>Pre-Bias</th><th>Age</th></tr></thead><tbody>';
    ['TakeProfit', 'TrailingStop', 'Mortality', 'StopLoss'].forEach(exit => {
        const g = trades.filter(t => t.exit_type === exit);
        if (!g.length) return;
        const avg = (k) => (g.reduce((s, t) => s + (t[k] || 0), 0) / g.length);
        html += `<tr><td><span class="exit-badge ${exit}">${exit}</span></td><td>${g.length}</td><td class="${pnlClass(avg('pnl_bps'))}">${avg('pnl_bps').toFixed(1)}</td><td>${avg('comp').toFixed(3)}</td><td>${avg('bias').toFixed(4)}</td><td>${avg('age').toFixed(1)}</td></tr>`;
    });
    html += '</tbody></table>';
    table.innerHTML = html;
}

// === RENDER: TOXICITY ATLAS ===
function renderAtlas(data) {
    const trades = data.trades.filter(t => t.age !== undefined);

    drawScatter('age-scatter', trades, 'age', 'pnl_bps', 'Elasticity Age (bars)', 'PnL (bps)', { colorBy: 'exit' });

    // Freshness decay curve
    const container = document.getElementById('decay-curve');
    if (container) {
        const canvas = document.createElement('canvas');
        const dpr = window.devicePixelRatio || 1;
        const w = container.clientWidth, h = container.clientHeight;
        canvas.width = w * dpr; canvas.height = h * dpr;
        canvas.style.width = w + 'px'; canvas.style.height = h + 'px';
        container.innerHTML = '';
        container.appendChild(canvas);
        const ctx = canvas.getContext('2d');
        ctx.scale(dpr, dpr);

        const pad = { top: 20, right: 30, bottom: 40, left: 55 };
        const pw = w - pad.left - pad.right, ph = h - pad.top - pad.bottom;

        // Draw decay curve
        ctx.beginPath();
        ctx.strokeStyle = '#06b6d4'; ctx.lineWidth = 2.5;
        for (let age = 0; age <= 20; age += 0.5) {
            const decay = Math.max(0.25, 1.0 / (1.0 + Math.exp((age - 10) / 2.5)));
            const px = pad.left + (age / 20) * pw;
            const py = pad.top + ph - (decay * ph);
            age === 0 ? ctx.moveTo(px, py) : ctx.lineTo(px, py);
        }
        ctx.stroke();

        // Fill under curve
        ctx.lineTo(pad.left + pw, pad.top + ph);
        ctx.lineTo(pad.left, pad.top + ph);
        ctx.closePath();
        ctx.fillStyle = 'rgba(6, 182, 212, 0.08)';
        ctx.fill();

        // Center line at age=10
        const cx = pad.left + (10 / 20) * pw;
        ctx.strokeStyle = 'rgba(245, 158, 11, 0.4)'; ctx.setLineDash([4, 4]); ctx.lineWidth = 1;
        ctx.beginPath(); ctx.moveTo(cx, pad.top); ctx.lineTo(cx, pad.top + ph); ctx.stroke();
        ctx.setLineDash([]);
        ctx.fillStyle = '#f59e0b'; ctx.font = '10px Inter'; ctx.textAlign = 'center';
        ctx.fillText('center = 10', cx, pad.top - 6);

        // Axes
        ctx.fillStyle = '#64748b'; ctx.font = '11px Inter';
        ctx.textAlign = 'center';
        ctx.fillText('Elasticity Age (bars)', pad.left + pw / 2, h - 8);
        ctx.save(); ctx.translate(14, pad.top + ph / 2);
        ctx.rotate(-Math.PI / 2); ctx.fillText('Freshness Multiplier', 0, 0); ctx.restore();

        ctx.font = '10px JetBrains Mono'; ctx.textAlign = 'center';
        for (let i = 0; i <= 4; i++) ctx.fillText((i * 5).toString(), pad.left + (pw / 4) * i, h - 22);
        ctx.textAlign = 'right';
        for (let i = 0; i <= 4; i++) ctx.fillText((i * 0.25).toFixed(2), pad.left - 8, pad.top + ph - (ph / 4) * i + 4);
    }

    // Toxicity table
    const table = document.getElementById('toxicity-table');
    const bins = {};
    trades.forEach(t => {
        const ageBin = Math.floor(t.age / 5) * 5;
        const key = ageBin;
        if (!bins[key]) bins[key] = [];
        bins[key].push(t);
    });

    let html = '<table class="data-table"><thead><tr><th>Age Bin</th><th>Trades</th><th>Win Rate</th><th>Avg PnL</th><th>Total PnL</th><th>Classification</th></tr></thead><tbody>';
    Object.keys(bins).sort((a, b) => +a - +b).forEach(key => {
        const g = bins[key];
        const wr = (g.filter(t => t.pnl_bps > 0).length / g.length * 100).toFixed(1);
        const avgPnl = (g.reduce((s, t) => s + t.pnl_bps, 0) / g.length).toFixed(1);
        const totalPnl = g.reduce((s, t) => s + t.pnl_bps, 0).toFixed(1);
        const cls = +key < 10 ? 'Fresh Elasticity' : +key < 15 ? 'Transitional' : 'Stale Toxicity';
        const clsColor = +key < 10 ? 'pnl-positive' : +key >= 15 ? 'pnl-negative' : '';
        html += `<tr><td>${key}–${+key + 4} bars</td><td>${g.length}</td><td>${wr}%</td><td class="${pnlClass(+avgPnl)}">${avgPnl}</td><td class="${pnlClass(+totalPnl)}">${totalPnl}</td><td class="${clsColor}">${cls}</td></tr>`;
    });
    html += '</tbody></table>';
    table.innerHTML = html;
}

// === RENDER: TRADE REPLAY ===
let replayIndex = 0;
let replayFilter = 'all';
let replayTrades = [];

function renderReplay(data) {
    replayTrades = data.trades.filter(t => t.eff !== undefined);
    document.querySelectorAll('.replay-filter').forEach(btn => {
        btn.addEventListener('click', () => {
            document.querySelectorAll('.replay-filter').forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
            replayFilter = btn.dataset.filter;
            replayIndex = 0;
            updateReplayView();
        });
    });
    document.getElementById('replay-prev').addEventListener('click', () => { replayIndex = Math.max(0, replayIndex - 1); updateReplayView(); });
    document.getElementById('replay-next').addEventListener('click', () => { const f = getFilteredTrades(); replayIndex = Math.min(f.length - 1, replayIndex + 1); updateReplayView(); });
    updateReplayView();
}

function getFilteredTrades() {
    if (replayFilter === 'all') return replayTrades;
    return replayTrades.filter(t => t.exit_type === replayFilter);
}

function updateReplayView() {
    const filtered = getFilteredTrades();
    if (!filtered.length) return;
    const t = filtered[replayIndex];
    document.getElementById('replay-counter').textContent = `Trade ${replayIndex + 1} / ${filtered.length}`;

    const exitColor = EXIT_COLORS[t.exit_type] || '#6b7280';
    document.getElementById('replay-header').innerHTML = `
        <h2 style="color:${exitColor}">#${t.rec_id} · ${t.sym} · ${t.dir}</h2>
        <div class="replay-sub">Exit: <span class="exit-badge ${t.exit_type}">${t.exit_type}</span> · Duration: ${t.duration} bars · PnL: <span style="color:${pnlColor(t.pnl_bps)}">${t.pnl_bps > 0 ? '+' : ''}${t.pnl_bps.toFixed(1)} bps</span></div>`;

    const events = buildLifecycleEvents(t);
    document.getElementById('replay-timeline').innerHTML = `<div class="timeline-events">${events.map(e =>
        `<div class="timeline-event ${e.cls}"><div class="event-icon">${e.icon}</div><div class="event-body"><div class="event-title">${e.title}</div><div class="event-detail">${e.detail}</div></div></div>`
    ).join('')}</div>`;

    const m = (label, val, color) => `<div class="replay-metric"><div class="replay-metric-label">${label}</div><div class="replay-metric-value" style="color:${color || 'inherit'}">${val}</div></div>`;
    document.getElementById('replay-metrics').innerHTML = [
        m('Efficiency', (t.eff||0).toFixed(3), t.eff > 0.4 ? '#ef4444' : '#10b981'),
        m('Resilience', (t.res||0).toFixed(3)),
        m('Compression', (t.comp||1).toFixed(3), (t.comp||1) < 0.8 ? '#10b981' : ''),
        m('Pre-Bias', (t.bias||0).toFixed(3), Math.abs(t.bias||0) > 0.35 ? '#ef4444' : '#10b981'),
        m('Elasticity Age', (t.age||0) + ' bars', (t.age||0) > 10 ? '#ef4444' : '#10b981'),
        m('Fertility', (t.fert||1).toFixed(3)),
    ].join('');

    document.getElementById('replay-interpretation').innerHTML = `<h4>🧠 Ecological Interpretation</h4><p>${buildInterpretation(t)}</p>`;
}

function buildLifecycleEvents(t) {
    const events = [];
    const compLabel = (t.comp||1) < 0.8 ? 'compressed (favorable)' : (t.comp||1) > 1.3 ? 'expanding (late)' : 'stable';
    events.push({ icon: '📦', cls: 'genesis', title: 'Pre-Entry Environment: ' + compLabel, detail: `Compression ratio: ${(t.comp||1).toFixed(3)} · Pre-range: ${((t.range||0)*100).toFixed(2)}% · Bias: ${(t.bias||0).toFixed(3)}` });

    const biasAbs = Math.abs(t.bias || 0);
    if (biasAbs < 0.15) events.push({ icon: '🎯', cls: 'genesis', title: 'Directional Ambiguity Preserved', detail: `Pre-bias magnitude: ${biasAbs.toFixed(3)} — asymmetry NOT yet consumed` });
    else if (biasAbs > 0.35) events.push({ icon: '⚠️', cls: 'warning', title: 'Direction Already Established', detail: `Pre-bias magnitude: ${biasAbs.toFixed(3)} — asymmetry partially consumed before entry` });
    else events.push({ icon: '📐', cls: 'topology', title: 'Mild Directional Trend Forming', detail: `Pre-bias magnitude: ${biasAbs.toFixed(3)} — transitional zone` });

    const eff = t.eff || 0;
    const topLabel = eff > 0.5 ? 'Terminal Smoothness (TRAP)' : eff > 0.38 ? 'Transitional' : eff > 0.25 ? 'Elastic (Fertile Zone)' : 'Chaotic';
    events.push({ icon: '🗺️', cls: 'topology', title: 'Topology: ' + topLabel, detail: `Efficiency: ${eff.toFixed(3)} · Density: ${(t.den||0).toFixed(3)} · Resilience: ${(t.res||0).toFixed(3)}` });

    const age = t.age || 0;
    const decay = 1.0 / (1.0 + Math.exp((age - 10) / 2.5));
    if (age <= 8) events.push({ icon: '🌱', cls: 'entry', title: `Fresh Elasticity (${age} bars since reload)`, detail: `Freshness decay: ${decay.toFixed(3)} — reload energy still active` });
    else if (age <= 13) events.push({ icon: '⏳', cls: 'topology', title: `Transitional Age (${age} bars since reload)`, detail: `Freshness decay: ${decay.toFixed(3)} — approaching exhaustion boundary` });
    else events.push({ icon: '💀', cls: 'warning', title: `Stale Elasticity (${age} bars since reload)`, detail: `Freshness decay: ${decay.toFixed(3)} — reload energy likely exhausted` });

    events.push({ icon: '▶️', cls: 'entry', title: `Entry at ${t.entry_price.toFixed(2)}`, detail: `${t.dir} · TP: ${t.tp.toFixed(2)} · SL: ${t.sl.toFixed(2)} · Fertility: ${(t.fert||1).toFixed(3)}` });

    const outcomeClass = t.pnl_bps > 0 ? 'outcome-win' : 'outcome-loss';
    events.push({ icon: t.pnl_bps > 0 ? '✅' : '❌', cls: outcomeClass, title: `${t.exit_type} at ${t.exit_price.toFixed(2)} after ${t.duration} bars`, detail: `PnL: ${t.pnl_bps > 0 ? '+' : ''}${t.pnl_bps.toFixed(1)} bps` });
    return events;
}

function buildInterpretation(t) {
    const parts = [];
    const eff = t.eff || 0, age = t.age || 0, comp = t.comp || 1, bias = Math.abs(t.bias || 0);
    if (t.exit_type === 'TakeProfit') {
        parts.push('This trade captured convex asymmetry successfully.');
        if (comp < 0.9) parts.push('The pre-entry environment was compressed — the entry caught a volatility expansion at its origin.');
        if (bias < 0.2) parts.push('No prior directional trend existed, so the full asymmetry was available for capture.');
        if (age < 12) parts.push('Entry timing was fresh relative to the liquidity reload — elastic continuation energy was still active.');
    } else if (t.exit_type === 'TrailingStop') {
        parts.push('The trailing stop captured partial continuation before the propagation reversed.');
        if (eff > 0.4) parts.push('The topology was borderline smooth — some continuation was real but ultimately limited.');
        if (age > 12) parts.push('Entry may have been slightly late in the elasticity lifecycle, limiting upside capture.');
    } else if (t.exit_type === 'Mortality') {
        parts.push('The trade expired without reaching either profit target or stop loss.');
        if (age > 13) parts.push(`Stale elasticity: entered ${age} bars after reload. The continuation energy was likely already consumed.`);
        if (bias > 0.35) parts.push('A moderate-to-strong directional trend already existed pre-entry — the executable asymmetry had partially dissipated.');
        if (comp > 1.2) parts.push('The pre-entry environment was already expanding — the system entered mid-expansion rather than catching the compression release.');
        if (eff > 0.4) parts.push('The Smoothness Trap may apply: high directional efficiency signaled terminal exhaustion, not durable continuation.');
    } else if (t.exit_type === 'StopLoss') {
        parts.push('Catastrophic reversal — the propagation was a structural fakeout.');
        if (eff > 0.5) parts.push('Extreme smoothness before entry strongly suggests this was a terminal propagation that reversed violently.');
    }
    return parts.join(' ') || 'Insufficient genesis data for detailed interpretation.';
}
