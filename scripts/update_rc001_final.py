#!/usr/bin/env python3
"""Update RC001_AB_REPORT.md with setA-19, setA-20 results and final score summary."""

with open('docs/roadef/RC001_AB_REPORT.md', 'r', encoding='utf-8') as f:
    content = f.read()

# 1. Add setA-19 and setA-20 rows after setA-18 row
old_row = '| setA-18  | 2000    | 1500  | 799256.747   | \u2014(invalid)  | \u2014    | 0.00 | 1.00 | A (only valid) \u2021 |\n'
new_rows = (
    '| setA-18  | 2000    | 1500  | 799256.747   | \u2014(invalid)  | \u2014    | 0.00 | 1.00 | A (only valid) \u2021 |\n'
    '| setA-19  | 6000    | 1998  | \u2014(invalid)  | \u2014(invalid)  | \u2014    | 0.00 | 0.98 | \u2020 |\n'
    '| setA-20  | 6000    | 2000  | \u2014(invalid)  | \u2014(invalid)  | \u2014    | 0.00 | 1.00 | \u2021 |\n'
)
if old_row in content:
    content = content.replace(old_row, new_rows, 1)
    print("OK: added setA-19 and setA-20 rows")
else:
    print("MISS: setA-18 row not found")

# 2. Update footnotes
old_fn = ('*\u2020 setA-16: both arms failed to produce a valid final solution (obj=\u221e). '
          'Greedy initialized 44/50 feasible genomes (IFR=0.88); Random initialized 0/50 (IFR=0.00). '
          'Neither arm completed evolution within the time budget. This instance is excluded from the '
          'win-rate count but is included in the IFR analysis.*\n'
          '*\u2021 setA-18: Greedy IFR=1.00 (all 50 genomes feasible) but Arm B produced obj=\u221e final result '
          'with \u26a0INVARIANT flag \u2014 EA violated an invariant during evolution despite perfect initialization. '
          'Random (Arm A) IFR=0.00 but produced a valid final solution (obj=799256.75). This is a new failure mode: '
          'constructor succeeds but EA fails. The obj=799256 scale is anomalous compared to all other instances '
          '(10\u2013260 range) \u2014 may indicate a different objective normalization or a reporting issue.*')
new_fn = ('*\u2020 setA-16 and setA-19: both arms failed to produce a valid final solution (obj=\u221e). '
          'Greedy IFR=0.88 and 0.98 respectively; Random IFR=0.00 on both. Evaluator budget exhausted before '
          'evolution could complete. Excluded from win-rate count.*\n'
          '*\u2021 setA-18 and setA-20: Greedy IFR=1.00 (all 50 genomes feasible) but Arm B produced obj=\u221e '
          'final result with \u26a0INVARIANT flag \u2014 EA violated an invariant during evolution despite perfect '
          'initialization. Random (Arm A) IFR=0.00 but produced a valid final solution. setA-20 shows deterministic '
          'max_sat=0.991 across all 50 genomes \u2014 constructor routing is fully deterministic on this instance. '
          'The obj=799256 scale for setA-18 is anomalous (all other instances: 10\u2013450 range) \u2014 may '
          'indicate a different objective normalization.*')
if old_fn in content:
    content = content.replace(old_fn, new_fn, 1)
    print("OK: updated footnotes")
else:
    print("MISS: footnotes not found")

# 3. Update score summary header
if '### Score Summary (18 instances)' in content:
    content = content.replace('### Score Summary (18 instances)', '### Score Summary (20 instances \u2014 FINAL)', 1)
    print("OK: updated score summary header")
else:
    print("MISS: score summary header not found")

# 4. Update score summary table — find the IFR>0 line which has HTML entity
old_line = '| Greedy EA failures (IFR&gt;0 but final obj=\u221e) | **1** (setA-18 \u2014 IFR=1.00, \u26a0INVARIANT) |'
new_block = ('| Greedy wins | **11** |\n'
             '| Random wins | **6** (setA-02, setA-05, setA-08, setA-14, setA-17, setA-18 \u2014 constructor or EA failures) |\n'
             '| Both arms invalid \u2014 evaluator budget exhausted | **2** (setA-16 IFR=0.88/0.00, setA-19 IFR=0.98/0.00) |\n'
             '| Greedy constructor failures (IFR=0) | **5** (setA-02, setA-05, setA-08, setA-14, setA-17) |\n'
             '| Greedy EA failures (IFR>0, \u26a0INVARIANT) | **2** (setA-18 IFR=1.00, setA-20 IFR=1.00) |\n'
             '| Random failures (IFR=0) | **9** (setA-07, setA-12, setA-13, setA-02, setA-16, setA-17, setA-18, setA-19, setA-20) |\n'
             '| Both valid | **9** |\n'
             '| **Greedy win rate when both valid** | **100% (9/9)** |\n'
             '| **Arm B mean IFR** | **0.587** (vs Arm A 0.124) \u2014 **+0.463 improvement** |\n'
             '| **Arm B better IFR** | **13/20 instances** |')

# Replace the entire old summary block
old_block = ('| Greedy wins | **11** |\n'
             '| Random wins | **6** (setA-02, setA-05, setA-08, setA-14, setA-17, setA-18) |\n'
             '| Both arms invalid (no valid final solution) | **1** (setA-16 \u2014 Greedy IFR=0.88, Random IFR=0.00) |\n'
             '| Greedy failures (IFR=0) | **5** (setA-02, setA-05, setA-08, setA-14, setA-17) |\n'
             '| Greedy EA failures (IFR&gt;0 but final obj=\u221e) | **1** (setA-18 \u2014 IFR=1.00, \u26a0INVARIANT) |\n'
             '| Random failures (IFR=0) | **6** (setA-07, setA-12, setA-13, setA-02, setA-16, setA-17) |\n'
             '| Both valid | **9** |\n'
             '| **Greedy win rate when both valid** | **100% (9/9)** |')
if old_block in content:
    content = content.replace(old_block, new_block, 1)
    print("OK: updated score summary table")
else:
    print("MISS: score summary table not found — trying alternate search")
    # Try with literal > instead of &gt;
    old_block2 = old_block.replace('&gt;', '>')
    if old_block2 in content:
        content = content.replace(old_block2, new_block, 1)
        print("OK: updated score summary table (literal >)")
    else:
        print("MISS: score summary table not found with either encoding")

# 5. Update footer
old_footer = '*Report generated from partial campaign results (18/20 instances). Full results will be appended when setA-19 and setA-20 complete.*'
new_footer = '*Report generated from complete campaign results (20/20 instances). Total runtime: 12,278s (~3.4 hours). Campaign ID: rc001_ab_v2.3.*'
if old_footer in content:
    content = content.replace(old_footer, new_footer, 1)
    print("OK: updated footer")
else:
    print("MISS: footer not found")

with open('docs/roadef/RC001_AB_REPORT.md', 'w', encoding='utf-8') as f:
    f.write(content)

print("\nVerifying lines 119-145:")
with open('docs/roadef/RC001_AB_REPORT.md', 'r', encoding='utf-8') as f:
    lines = f.readlines()
for i, line in enumerate(lines[118:147], start=119):
    print(f"{i}: {line}", end='')