#!/usr/bin/env python3
"""Insert Section 13 (Principal Research Contributions) before Version History."""

with open('docs/roadef/RC001_AB_REPORT.md', 'r', encoding='utf-8') as f:
    content = f.read()

section13_block = """\
---

## 13. Principal Research Contributions

The following contributions are derived directly from the RC-001 A/B benchmark campaign
(rc001_ab_v2.3, 20 setA instances, 6,000 evaluations per arm).

**C-1 \u2014 Load-aware greedy initialization consistently produces superior feasible solutions.**
Arm B (RP-401C greedy constructor) achieved a mean IFR of 0.587 versus 0.124 for Arm A
(random constructor), a +0.463 absolute improvement across 20 instances. When both arms
produced a valid final solution, Arm B won in 100% of cases (9/9). The greedy constructor
never degraded solution quality relative to random initialization on any instance where both
arms succeeded.

**C-2 \u2014 Initialization behaviour separates naturally into three distinct operating classes.**
The campaign data does not support a simple feasible/infeasible binary. Three classes emerge:
Class A (IFR \u2248 1, max_sat < 1.0 \u2014 constructor succeeds, evolution optimizes normally);
Class B (mild overload \u2264 6%, repairable \u2014 constructor produces a near-feasible genome that
targeted repair could recover); Class C (catastrophic bridge bottleneck 300\u2013600%, structural
routing failure \u2014 the same arcs recur across independent runs, indicating a topological
deficiency rather than a stochastic failure). This taxonomy directly informs the design of
RC-001B (binary feasibility classifier) and RP-410 (bridge-aware initialization).

**C-3 \u2014 Evaluation cost scales with both instance size and genome state.**
Per-evaluation runtime grew from \u22482 ms on setA-01 (200 demands) to \u224819,200 ms on setA-16
Greedy (6,000 demands) \u2014 a \u22489,600\u00d7 increase over a 30\u00d7 demand-count increase, implying
super-linear scaling. More critically, on the same instance (setA-16) the Greedy arm cost
19,200 ms/eval versus 4,333 ms/eval for the Random arm \u2014 a 4.4\u00d7 difference attributable
solely to genome state. This state-dependent evaluation cost is a previously uncharacterized
property of the Coralys evaluator.

**C-4 \u2014 The dominant scalability bottleneck has shifted from construction to evaluation.**
The RP-401C O(D\u00b2)\u2192O(D) constructor fix (v2.3) resolved construction-time scaling. The
remaining throughput collapse is located inside the evaluator. At setA-20 scale (6,000
demands), evaluation throughput is so low that the EA cannot complete within the time budget,
rendering constructor quality irrelevant. Future work must prioritize evaluator architecture
investigation (RC-004A baseline, RC-004B profiling) before introducing additional evolutionary
operators or crossover strategies.

**C-5 \u2014 Remaining constructor failures are caused primarily by recurring structural bottlenecks
rather than inadequate greedy heuristics.**
Class C failures (setA-17 and analogues) exhibit the same overloaded arc IDs (66, 67, 163,
606, 968) across all 50 independent initialization attempts. A heuristic that cannot see
network topology cannot avoid these bottlenecks regardless of load-awareness. This motivates
RP-410 (bridge-aware initialization): edge-betweenness detection at startup, adaptive path
scoring incorporating a bridge penalty term, and demand-ordering by structural risk. The
greedy heuristic is not the limiting factor; the routing model's blindness to bridge arcs is.

---

## 14. Version History"""

old_header = "---\n\n## 13. Version History"

if old_header not in content:
    print("ERROR: target string not found in file.")
    # Show context around line 619
    lines = content.splitlines()
    for i, line in enumerate(lines[615:625], start=616):
        print(f"{i}: {repr(line)}")
    exit(1)

content = content.replace(old_header, section13_block, 1)

with open('docs/roadef/RC001_AB_REPORT.md', 'w', encoding='utf-8') as f:
    f.write(content)

print("Done. Verifying...")
with open('docs/roadef/RC001_AB_REPORT.md', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i, line in enumerate(lines, start=1):
    if '## 13. Principal Research' in line or '## 14. Version History' in line:
        start = max(0, i - 2)
        end = min(len(lines), i + 4)
        for j in range(start, end):
            print(f"{j+1}: {lines[j]}", end='')
        print("---")