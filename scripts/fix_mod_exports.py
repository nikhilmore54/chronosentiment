#!/usr/bin/env python3
"""Add ViolationExplanation to the compliance mod.rs pub use traits block."""
import sys

path = 'adapters/ultracrew/src/compliance/mod.rs'
with open(path, 'rb') as f:
    data = f.read()

# The exact bytes to find (real < characters in the file)
old = b'pub use traits::{\n    ConstraintRule,\n    CompliancePack,\n    RuleId,\n    RuleOutcome,\n    RuleContext,\n    Severity,\n};'
new = b'pub use traits::{\n    ConstraintRule,\n    CompliancePack,\n    RuleId,\n    RuleOutcome,\n    RuleContext,\n    Severity,\n    ViolationExplanation,\n};'

if old in data:
    data2 = data.replace(old, new, 1)
    with open(path, 'wb') as f:
        f.write(data2)
    print('OK: ViolationExplanation added to mod.rs re-exports')
else:
    print('NOT FOUND — showing lines 90-105:')
    lines = data.split(b'\n')
    for i, ln in enumerate(lines[89:105], start=90):
        print(f'{i}: {ln!r}')
    sys.exit(1)