
#!/usr/bin/env python3
"""
Gate all non-critical println! calls in ga.rs behind GA_DEBUG.

ALWAYS-ON (never gated) — generation-level / system-level events:
  These fire once per generation or less, and are essential for operators.

GA_DEBUG-ONLY — everything else:
  Per-signal, per-trade, per-window prints that flood on every tick.
"""

import re, shutil

TARGET = "core/src/ga.rs"
shutil.copy(TARGET, TARGET + ".bak2")

# Lines/substrings that should ALWAYS print (not gated).
# Match is done against the full println! block text.
ALWAYS_ON = [
    "FINAL_EVAL →",
    "ALPHA →",
    "FIRST_ALPHA_DISCOVERY",
    "NUCLEAR COLLAPSE",
    "ANTI-COLLAPSE GUARD",
    "SHOCK THERAPY",
    "SOFT_EXPANSION_TRIGGER",
    "SOFT_EXPANSION_ROLLBACK",
    "GLOBAL_EXPANSION_TRIGGERED",
    "GLOBAL_FRONTIER",
    "Top 5 strategies",
    "Starting Multi-Asset",
    "Evolving Bucket",
    "WARNING: Massive evaluation drop",
    "FINAL_EVAL_COUNT",
    "⏭️  SKIP",          # scenario skip lines
    "SCENARIO_SKIP",
    "AQG_SKIP_ENSEMBLE",
    "EDGE STARVATION",
    "NO EMISSION",
    "--- Starting Multi",
    ">> Evolving",
]

with open(TARGET, "r") as f:
    src = f.read()

lines = src.split("\n")
out = []
i = 0
gated = 0
skipped_always_on = 0
already_gated = 0

def collect_println_block(lines, start):
    """
    From lines[start] which begins with optional indent + println!(
    collect until the matching ); is found.
    Returns (block_lines, end_index_exclusive).
    """
    block = []
    depth = 0
    j = start
    found_open = False
    while j < len(lines):
        line = lines[j]
        block.append(line)
        for ch in line:
            if ch == '(':
                depth += 1
                found_open = True
            elif ch == ')':
                depth -= 1
        j += 1
        if found_open and depth == 0:
            break
    return block, j

def is_inside_ga_debug_block(out_lines):
    """
    Look back through emitted output to detect if we're already inside
    an `if std::env::var("GA_DEBUG").is_ok() {` block.
    Simple heuristic: scan last 15 lines for the pattern and unclosed {.
    """
    tail = "\n".join(out_lines[-20:]) if len(out_lines) >= 20 else "\n".join(out_lines)
    return 'GA_DEBUG' in tail and 'is_ok()' in tail and '{' in tail

while i < len(lines):
    line = lines[i]
    stripped = line.lstrip()

    # Detect the start of a println! statement (not a comment, not macro args)
    if stripped.startswith('println!(') and not stripped.startswith('//'):
        # Collect the full block
        block, next_i = collect_println_block(lines, i)
        block_text = "\n".join(block)

        # Already gated?
        if is_inside_ga_debug_block(out):
            already_gated += 1
            out.extend(block)
            i = next_i
            continue

        # Should it always print?
        if any(tag in block_text for tag in ALWAYS_ON):
            skipped_always_on += 1
            out.extend(block)
            i = next_i
            continue

        # Gate it
        indent = len(line) - len(line.lstrip())
        pad = " " * indent

        out.append(f"{pad}if std::env::var(\"GA_DEBUG\").is_ok() {{")
        for bl in block:
            out.append("    " + bl)  # indent block one level further
        out.append(f"{pad}}}")
        gated += 1
        i = next_i
        continue

    out.append(line)
    i += 1

result = "\n".join(out)
with open(TARGET, "w") as f:
    f.write(result)

total_after = result.count("println!")
print(f"✅ Done.")
print(f"   Gated (wrapped in GA_DEBUG):  {gated}")
print(f"   Already gated (skipped):      {already_gated}")
print(f"   Always-on (kept bare):        {skipped_always_on}")
print(f"   println! calls after:         {total_after}")
