import re
import numpy as np

log_file = "archive/replay_1m_gen11.log"

pnls = []
drawdowns = []
equity = 1.0
peak = 1.0

print("Calculating full metrics...")

# We can estimate PnL from [PAPER_EXCURSION] worst_bps vs best_bps based on exit reason?
# Actually, we can just look at the [PAPER_SUMMARY] for the macro stats.
# But let's just parse PAPER_SUMMARY directly for the response.

with open(log_file, "r") as f:
    for line in f:
        if "[PAPER_SUMMARY]" in line:
            m = re.search(r"closed=(\d+) pnl=([-\d\.]+) win_rate=([-\d\.]+) .* avg_win=([-\d\.]+) avg_loss=([-\d\.]+) expectancy=([-\d\.]+)", line)
            if m:
                print(f"Closed: {m.group(1)}")
                print(f"PnL: {float(m.group(2))*100:.2f}%")
                print(f"Win Rate: {float(m.group(3))*100:.2f}%")
                print(f"Avg Win: {float(m.group(4))*10000:.2f} bps")
                print(f"Avg Loss: {float(m.group(5))*10000:.2f} bps")
                print(f"Expectancy: {float(m.group(6))*10000:.2f} bps")
