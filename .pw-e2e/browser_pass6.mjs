#!/usr/bin/env node
/**
 * Pass 6 browser certification — headless collection of artifacts A/B/C.
 * Requires: UI at :3000, API at :8000, playwright chromium installed.
 */
import { chromium } from 'playwright';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const OUT = path.join(ROOT, 'artifacts', 'browser_pass6');
const UI_URL = process.env.UI_E2E_URL || 'http://localhost:3000';

fs.mkdirSync(OUT, { recursive: true });

const consoleAll = [];
const consoleWarnings = [];

function setSlider(page, value) {
  return page.locator('input.cs-range').evaluate((el, val) => {
    const setter = Object.getOwnPropertyDescriptor(
      window.HTMLInputElement.prototype,
      'value',
    )?.set;
    if (setter) setter.call(el, String(val));
    else el.value = String(val);
    el.dispatchEvent(new Event('input', { bubbles: true }));
    el.dispatchEvent(new Event('change', { bubbles: true }));
  }, value);
}

async function readCurrentSeq(page) {
  const seqLabel = page.locator('span', { hasText: /^Seq \d+$/ }).nth(1);
  const text = await seqLabel.innerText();
  const m = text.match(/Seq (\d+)/);
  return m ? Number(m[1]) : null;
}

async function divergenceStripText(page) {
  const strip = page.locator('span', { hasText: 'Divergences at Seq' }).first();
  if ((await strip.count()) === 0) return 'Divergence strip: not rendered (single mode or not loaded)';
  const container = strip.locator('xpath=..');
  return (await container.innerText()).replace(/\s+/g, ' ').trim();
}

async function extractComparisonPanels(page) {
  const card = page.locator('.cs-card').filter({ hasText: 'Execution Summary Comparison' }).first();
  await card.waitFor({ state: 'visible', timeout: 5000 });
  const summary = await card.innerText();

  const verdict = page.locator('.cs-alert').filter({ hasText: 'Confidence:' }).first();
  const verdictText = (await verdict.count()) ? await verdict.innerText() : 'Verdict: not found';

  const divergenceCard = page.locator('.cs-card').filter({ hasText: 'Execution Divergence Analysis' }).first();
  const divergenceText = (await divergenceCard.count()) ? await divergenceCard.innerText() : 'Divergence analysis: not found';

  const strip = await divergenceStripText(page);

  return { summary, verdictText, divergenceText, strip };
}

const browser = await chromium.launch({ headless: true });
const page = await browser.newPage({ viewport: { width: 1440, height: 1200 } });

page.on('console', (msg) => {
  const line = `[${msg.type()}] ${msg.text()}`;
  consoleAll.push(line);
  if (msg.type() === 'warning' || msg.type() === 'error') consoleWarnings.push(line);
});
page.on('pageerror', (err) => consoleWarnings.push(`[pageerror] ${err.message}`));

try {
  await page.goto(UI_URL, { waitUntil: 'networkidle', timeout: 60000 });
  await page.getByRole('button', { name: /Inspect Strategy/i }).click();

  await page.locator('#strategy_id').fill('strat_200_5_4_2');
  await page.locator('#seed_inspect').fill('42');
  await page.locator('#strategy_id2').fill('strat_150_3_3_1');
  await page.locator('#seed_inspect2').fill('42');

  await page.getByRole('button', { name: /Reconstruct trace/i }).click();
  await page.locator('text=Execution Summary Comparison').waitFor({ timeout: 60000 });

  // ── Artifact A ──────────────────────────────────────────────────────────
  const artifactA = await extractComparisonPanels(page);
  fs.writeFileSync(path.join(OUT, 'artifact_a_summary.txt'), [
    '=== Execution Summary Comparison ===',
    artifactA.summary,
    '',
    '=== Divergence Strip ===',
    artifactA.strip,
    '',
    '=== Verdict / Confidence ===',
    artifactA.verdictText,
    '',
    '=== Execution Divergence Analysis ===',
    artifactA.divergenceText,
  ].join('\n'));

  await page.locator('.cs-card').filter({ hasText: 'Execution Summary Comparison' }).first().scrollIntoViewIfNeeded();
  await page.screenshot({
    path: path.join(OUT, 'artifact_a_comparison_panels.png'),
    fullPage: true,
  });

  // ── Artifact B ──────────────────────────────────────────────────────────
  const slider = page.locator('input.cs-range');
  await slider.waitFor({ state: 'visible', timeout: 5000 });
  const minN = Number(await slider.getAttribute('min'));
  const maxN = Number(await slider.getAttribute('max'));
  const midN = Math.floor((minN + maxN) / 2);

  const replayObservations = [];
  for (const [position, seq] of [['start', minN], ['middle', midN], ['end', maxN]]) {
    await setSlider(page, seq);
    await page.waitForTimeout(500);
    const uiSeq = await readCurrentSeq(page);
    replayObservations.push({
      position,
      requestedSeq: seq,
      uiSeq,
      divergenceStrip: await divergenceStripText(page),
    });
  }
  fs.writeFileSync(
    path.join(OUT, 'artifact_b_replay_slider.json'),
    JSON.stringify({ min: minN, max: maxN, observations: replayObservations }, null, 2),
  );

  // Raw toggle exercise (part of console capture scope)
  const rawToggle = page.getByRole('checkbox', { name: /Show raw events/i });
  await rawToggle.check();
  await page.waitForTimeout(400);
  await rawToggle.uncheck();
  await page.waitForTimeout(400);

  // ── Artifact C ──────────────────────────────────────────────────────────
  const reactWarnings = consoleWarnings.filter(
    (l) =>
      (/react/i.test(l) && !/react devtools/i.test(l)) ||
      /prop type/i.test(l) ||
      /Each child in a list should have a unique/i.test(l) ||
      /undefined/i.test(l) ||
      /missing key/i.test(l),
  );

  const artifactC = {
    totalConsoleLines: consoleAll.length,
    warningOrErrorCount: consoleWarnings.length,
    reactOrPropWarnings: reactWarnings,
    allWarningsAndErrors: consoleWarnings,
    pass: reactWarnings.length === 0,
  };
  fs.writeFileSync(path.join(OUT, 'artifact_c_console.json'), JSON.stringify(artifactC, null, 2));
  fs.writeFileSync(path.join(OUT, 'artifact_c_console.log'), consoleAll.join('\n'));

  // Parse key metrics from rendered summary for automated gate
  const summary = artifactA.summary;
  const queueYes = (summary.match(/Queue Progression\s*\n\s*Yes/gi) || []).length;
  const fullFillsZero = !summary.match(/Full Fills\s*\n\s*[1-9]/);
  const stepsNonZero = /Steps\s*\n\s*[1-9]/i.test(summary);

  const gate = {
    queueProgressionVisible: queueYes >= 1,
    fullFillsZero,
    stepsNonZero,
    replaySliderDynamic:
      replayObservations.length >= 2 &&
      replayObservations.every((o) => o.uiSeq === o.requestedSeq) &&
      new Set(replayObservations.map((o) => o.divergenceStrip)).size > 1,
    consoleClean: artifactC.pass,
  };

  fs.writeFileSync(path.join(OUT, 'pass6_gate.json'), JSON.stringify(gate, null, 2));

  console.log(JSON.stringify({ outDir: OUT, gate, artifactC: { pass: artifactC.pass, reactWarnings: reactWarnings.length } }, null, 2));
  process.exit(gate.queueProgressionVisible && gate.consoleClean ? 0 : 1);
} catch (err) {
  console.error('Browser pass failed:', err);
  fs.writeFileSync(path.join(OUT, 'failure.txt'), String(err.stack || err));
  process.exit(2);
} finally {
  await browser.close();
}
