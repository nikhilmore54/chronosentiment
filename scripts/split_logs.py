import sys
import os
import re

LOG_DIR = "analysis/live_multi"
os.makedirs(LOG_DIR, exist_ok=True)

# Regex to find symbol in lines
# [SYMBOL_TS] AXISBANK.NS:1777399350,...
# [MOMENTUM_BOOTSTRAP] sym=BTC-USD ...
# [DIAG] sym=BTC-USD ...
SYM_RE = re.compile(r"sym=([A-Z0-9.\-_]+)")
SYM_TS_RE = re.compile(r"\[SYMBOL_TS\]\s+(.+)")

files = {}

def get_file(sym):
    sym_clean = sym.replace("-", "_").replace(".", "_")
    if sym_clean not in files:
        files[sym_clean] = open(os.path.join(LOG_DIR, f"live_{sym_clean}.log"), "w")
    return files[sym_clean]

for line in sys.stdin:
    sys.stdout.write(line)
    sys.stdout.flush()
    
    # Try to find symbol
    m = SYM_RE.search(line)
    if m:
        sym = m.group(1)
        get_file(sym).write(line)
        get_file(sym).flush()
        continue
    
    m = SYM_TS_RE.search(line)
    if m:
        parts = m.group(1).split(",")
        for p in parts:
            sym = p.split(":")[0].strip()
            get_file(sym).write(line)
            get_file(sym).flush()
        continue
    
    # Generic lines to all?
    for f in files.values():
        f.write(line)
        f.flush()
