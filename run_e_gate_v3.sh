#!/usr/bin/env bash
# E-GATE v3: B3-style lineage plus the temporal metadata invariant omitted by E-GATE v2.
# Does not modify B3 or G-GATE v1.1 artifacts.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

DBNAME="${1:?usage: run_e_gate_v3.sh <dbname>}"
EVIDENCE_DIR="${2:?usage: run_e_gate_v3.sh <dbname> <evidence_dir>}"

mkdir -p "$EVIDENCE_DIR"

if [[ "$DBNAME" == "chrono_b3_test" ]]; then
  echo "STOP: E-GATE v3 must not target chrono_b3_test" >&2
  exit 2
fi

set +e
"$ROOT/run_b4_invariant_check.sh" "$DBNAME" | tee "$EVIDENCE_DIR/invariant_checks.txt"
STATUS=${PIPESTATUS[0]}
set -e

DUMP="$EVIDENCE_DIR/db/full_dump.dump"
DUMP_HASH="unspecified"
if [[ -f "$DUMP" ]]; then
  DUMP_HASH="$(shasum -a 256 "$DUMP" | awk '{print $1}')"
fi

if [[ "$STATUS" -eq 0 ]]; then
  RESULT="PASS"
else
  RESULT="FAIL"
fi

cat > "$EVIDENCE_DIR/E_GATE_CERTIFICATION_B4.md" <<EOF
# E-GATE v3 Certification (B4)

**Result:** ${RESULT}

This gate extends E-GATE v2 with the temporal metadata invariant:

\`\`\`
assessment.evaluation_timestamp <= decision.evaluation_timestamp
\`\`\`

for every assessment→decision pair.

B3 is not modified. G-GATE v1.1 on B3 remains INCONCLUSIVE / leakage FAIL.

## Checks

See \`invariant_checks.txt\`.

Required:

1. Assessments = 195
2. Decisions = 195
3. Strategies = 110
4. Outcomes = 440
5. Assessment → Decision lineage (no orphan decisions)
6. Decision → Strategy lineage
7. Strategy → Outcome matches = 440
8. Exactly 4 outcomes per strategy
9. Horizons = 5D/10D/20D/60D
10. **assessment.evaluation_timestamp <= decision.evaluation_timestamp**
11. Outcome evaluation timestamps not before the parent decision
12. Outcome evaluation_timestamp < horizon_expiry_timestamp

## Dataset

- Database: \`${DBNAME}\`
- Dump SHA-256: \`${DUMP_HASH}\`

## Methodology

G-GATE, if run after this gate PASSes, must use frozen v1.1 unchanged
(\`Y_h\`, candidate, 55/27/28 split, bootstrap, seed \`20260813\`).
EOF

echo "E-GATE v3 ${RESULT}"
exit "$STATUS"
