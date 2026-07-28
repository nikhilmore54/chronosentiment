#!/usr/bin/env python3
"""
Two targeted replacements in App.js:
1. Add `scenario` state variable next to `generationLimit` and `seed`
2. Replace the Step 2 hardcoded scenario display with a radio selector
3. Update handleRunOptimizer call to pass scenarioId
4. Update Gantt HORIZON_HRS to be scenario-aware
"""
import sys

path = 'apps/ultracrew-pilot-portal/src/App.js'

with open(path, 'r', encoding='utf-8') as f:
    content = f.read()

changes = 0

# ── 1. Add scenario state variable ───────────────────────────────────────────
old1 = "  const [generationLimit, setGenerationLimit] = useState(500);\n  const [seed, setSeed] = useState(42);"
new1 = "  const [scenario, setScenario] = useState('sunair');\n  const [generationLimit, setGenerationLimit] = useState(500);\n  const [seed, setSeed] = useState(42);"
if old1 in content:
    content = content.replace(old1, new1, 1)
    changes += 1
    print('✓ Added scenario state variable')
else:
    print('✗ Could not find generationLimit/seed state declarations', file=sys.stderr)

# ── 2. Update handleRunOptimizer to pass scenario ────────────────────────────
old2 = "    const { data, shifts: s, workers: w, layoverMarkers: lm } = await runOptimizer(generationLimit, seed);"
new2 = "    const { data, shifts: s, workers: w, layoverMarkers: lm } = await runOptimizer(scenario, generationLimit, seed);"
if old2 in content:
    content = content.replace(old2, new2, 1)
    changes += 1
    print('✓ Updated handleRunOptimizer call to pass scenarioId')
else:
    # Try alternate form
    old2b = "    const { data, shifts: s, workers: w, layoverMarkers: lm } = await runOptimizer(generationLimit, seed);"
    print(f'✗ Could not find handleRunOptimizer call site (tried: {repr(old2[:60])})', file=sys.stderr)

# ── 3. Replace Step 2 hardcoded scenario display with radio selector ──────────
old3 = """            <div style={S.cardTitle}>Step 2 — Run Optimizer</div>
            <div style={S.cardSub}>Configure and run the UltraCrew optimizer on the SunAir scenario.</div>
            <label style={S.label}>Scenario</label>
            <div style={{ ...S.input, color: '#64748b', cursor: 'default', marginBottom: '16px' }}>SunAir Demo — 20 workers, 42 shifts, 7-day horizon</div>"""

new3 = """            <div style={S.cardTitle}>Step 2 — Run Optimizer</div>
            <div style={S.cardSub}>Select a scenario and run the UltraCrew optimizer.</div>
            <label style={S.label}>Scenario</label>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', marginBottom: '16px' }}>
              {[
                { id: 'sunair',          label: 'SunAir Demo',          desc: '20 workers · 42 shifts · 7-day horizon · Indian airline (BOM/DEL/BLR)', available: true },
                { id: 'gerad-fixture',   label: 'GERAD Fixture',         desc: '8 crew · 10 legs · 5 duties · 5-day horizon · ORD hub (adapter test fixture, NOT the research benchmark)', available: true },
                { id: 'gerad-benchmark', label: 'GERAD Benchmark',       desc: 'G1422-DataSets.zip required — see benchmarks/gerad-g2014-22/README.md', available: false },
              ].map(sc => (
                <label key={sc.id} style={{ display: 'flex', alignItems: 'flex-start', gap: '10px', padding: '10px 12px', borderRadius: '6px', border: `1px solid ${scenario === sc.id ? '#3b82f6' : '#e2e8f0'}`, background: sc.available ? (scenario === sc.id ? '#eff6ff' : '#fff') : '#f8fafc', cursor: sc.available ? 'pointer' : 'not-allowed', opacity: sc.available ? 1 : 0.55 }}>
                  <input type="radio" name="scenario" value={sc.id} checked={scenario === sc.id} disabled={!sc.available} onChange={() => sc.available && setScenario(sc.id)} style={{ marginTop: '3px', accentColor: '#3b82f6' }} />
                  <div>
                    <div style={{ fontWeight: 600, fontSize: '13px', color: sc.available ? '#1e293b' : '#94a3b8' }}>{sc.label}{!sc.available && <span style={{ marginLeft: '8px', fontSize: '11px', color: '#f59e0b', fontWeight: 500 }}>⚠ Not yet available</span>}</div>
                    <div style={{ fontSize: '12px', color: '#64748b', marginTop: '2px' }}>{sc.desc}</div>
                  </div>
                </label>
              ))}
            </div>"""

if old3 in content:
    content = content.replace(old3, new3, 1)
    changes += 1
    print('✓ Replaced Step 2 scenario display with radio selector')
else:
    print(f'✗ Could not find Step 2 scenario display block', file=sys.stderr)
    # Show what's around line 407
    lines = content.split('\n')
    for i, line in enumerate(lines[400:420], start=401):
        print(f'  {i}: {repr(line)}')

# ── 4. Make Gantt HORIZON_HRS scenario-aware ──────────────────────────────────
old4 = "                const HORIZON_HRS = 336;"
new4 = "                const HORIZON_HRS = scenario === 'gerad-fixture' ? 120 : 336;"
if old4 in content:
    content = content.replace(old4, new4, 1)
    changes += 1
    print('✓ Made Gantt HORIZON_HRS scenario-aware')
else:
    print('✗ Could not find HORIZON_HRS constant', file=sys.stderr)

with open(path, 'w', encoding='utf-8') as f:
    f.write(content)

print(f'\nDone: {changes}/4 changes applied. Lines: {content.count(chr(10))}')
if changes < 4:
    sys.exit(1)