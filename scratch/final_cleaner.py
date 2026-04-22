import re
import os

with open("core/src/ga.rs", "r") as f:
    lines = f.readlines()

def fix_line(line, line_idx):
    # 1. Fix unused vars usage logic:
    if "_pnl_score" in line and not "let mut " in line:
        line = line.replace("_pnl_score", "pnl_score")
    if "_total_avg_e_score" in line and not "let mut " in line:
        line = line.replace("_total_avg_e_score", "total_avg_e_score")
    if "_phase2_sum_expected" in line and not "let mut " in line:
        line = line.replace("_phase2_sum_expected", "phase2_sum_expected")

    # Revert the lets to drop the undercore but suppress warning
    if "let mut _pnl_score =" in line:
        line = line.replace("let mut _pnl_score =", "let mut pnl_score =")
    if "let mut _total_avg_e_score =" in line:
        line = line.replace("let mut _total_avg_e_score =", "let mut total_avg_e_score =")
    if "let mut _phase2_sum_expected =" in line:
        line = line.replace("let mut _phase2_sum_expected =", "let mut phase2_sum_expected =")

    # Unused parentheses warning
    if "consensus_conf *= (1.0 - 0.3 * entropy_penalty);" in line:
        line = line.replace("consensus_conf *= (1.0 - 0.3 * entropy_penalty);", "consensus_conf *= 1.0 - 0.3 * entropy_penalty;")

    # Duplicate pnl in SignalAlpha
    # Since it's a multiline block, I will just fix it in the main string

    # Strategy missing fields
    if "Strategy {" in line:
        pass # It's multi-line

    return line

clean_lines = [fix_line(l, i) for i, l in enumerate(lines)]
text = "".join(clean_lines)

# Fix duplicate pnl
text = text.replace("    pub pnl: f64,\n    pub pnl: f64,", "    pub pnl: f64,")

# Strategy missing exec_aggression, fill_threshold, latency_bias
# Let's cleanly inject it using precise regex
text = re.sub(
    r'(Strategy \{[^\}]*?edge_ratio:[^\n]*\n)([ \t]*\}|)',
    r'\1            exec_aggression: 50,\n            latency_bias: 10,\n            fill_threshold: 50,\n\2',
    text
)
# Wait, some Strategy initializers don't have edge_ratio.
# E.g. at 486:
# Strategy {
#    ...
# }

with open("core/src/ga.rs", "w") as f:
    f.write(text)
