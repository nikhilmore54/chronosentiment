#!/usr/bin/env bash
# G-GATE v1.1 on B4 only. Does not modify B3 dump or B3 G-GATE artifacts.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

B4_DUMP="${B4_DUMP:-r3_evidence/20260814T023457Z_B4/db/full_dump.dump}"
B4_EXPECT="${B4_EXPECT:-f74e576e8e98b24058cc913b14a567d9ff4b3eabc75662ab9a96901b102f8cd6}"
METH="G_Extension_Methodology_v1.1.md"
SPLIT="G_Extension_Methodology_v1.1_TrainTestSplit.md"
MANIFEST="G_Extension_Methodology_v1.1_manifest.txt"
METH_EXPECT="e129d7add66d7f4c12aab14811a3d552abf6b603f012eeb75c99c484e0065e66"
SPLIT_EXPECT="6e9b3405a21b21f6c59cf99c05822c0d20007d335ef38a7bb5a21cf8f79d5691"
MANIFEST_EXPECT="1604563a0a4516cbe983ef398ad36b6e1daacc8842b7a8daa28812e8ffee958e"
OUT_DIR="r3_evidence/20260814T023457Z_B4/G_GATE"
DBNAME="${G_GATE_DBNAME:-chrono_g_gate_b4_readonly}"
PGHOST="${PGHOST:-localhost}"

hash_of() { shasum -a 256 "$1" | awk '{print $1}'; }

echo "=== 0. B3 freeze untouched ==="
B3_GOT="$(hash_of r3_evidence/20260813T035739Z_B3/db/full_dump.dump)"
if [[ "$B3_GOT" != "af11d318b03fb171207f96348fcf210e1b9149b1ab6e699c06c363faec518788" ]]; then
  echo "STOP: B3 dump hash changed" >&2
  exit 2
fi

echo "=== 1. Verify B4 identity ==="
B4_GOT="$(hash_of "$B4_DUMP")"
if [[ "$B4_GOT" != "$B4_EXPECT" ]]; then
  echo "STOP: B4 dump hash mismatch" >&2
  echo " expected $B4_EXPECT" >&2
  echo " got      $B4_GOT" >&2
  exit 2
fi
echo "B4 dump hash OK"

echo "=== 2. Verify v1.1 methodology hashes (unchanged) ==="
METH_GOT="$(hash_of "$METH")"
SPLIT_GOT="$(hash_of "$SPLIT")"
MANIFEST_GOT="$(hash_of "$MANIFEST")"
if [[ "$METH_GOT" != "$METH_EXPECT" || "$SPLIT_GOT" != "$SPLIT_EXPECT" || "$MANIFEST_GOT" != "$MANIFEST_EXPECT" ]]; then
  echo "STOP: v1.1 methodology hash mismatch" >&2
  exit 2
fi
echo "v1.1 hashes OK"

if [[ "$DBNAME" == "chrono_b3_test" || "$DBNAME" == "chrono_b4_test" ]]; then
  echo "STOP: refusing to use canonical $DBNAME as working database" >&2
  exit 2
fi

echo "=== 3. Restore B4 dump into separate working database ==="
dropdb --if-exists -h "$PGHOST" "$DBNAME"
createdb -h "$PGHOST" "$DBNAME"
pg_restore --no-owner --no-acl -h "$PGHOST" --dbname="$DBNAME" "$B4_DUMP"

echo "=== 4. Build experiment binary ==="
cargo build --release -p chronosentiment_adapter --bin m6_predictive_value_experiment
TARGET_DIR="$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
BIN="$TARGET_DIR/release/m6_predictive_value_experiment"
BIN_SHA="$(hash_of "$BIN")"
echo "binary sha256 $BIN_SHA"

echo "=== 5. Execute G-GATE v1.1 once on B4 ==="
rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"
export DATABASE_URL="postgresql://${PGUSER:-$(whoami)}@${PGHOST}:5432/${DBNAME}"
export G_GATE_OUT_DIR="$ROOT/$OUT_DIR"
export G_GATE_DATASET="B4"
export DATASET_SHA256="$B4_GOT"
export METH_MANIFEST_SHA256="$MANIFEST_GOT"
export EXPERIMENT_BINARY_SHA256="$BIN_SHA"
export REPO_ROOT="$ROOT"
"$BIN"

echo "=== 6–7. Verify witness hashes ==="
WITNESS="$OUT_DIR/G_GATE_WITNESS.json"
OUTPUT="$OUT_DIR/g_gate_output.txt"
OUT_SHA="$(hash_of "$OUTPUT")"
WITNESS_OUT_SHA="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["output_sha256"])' "$WITNESS")"
if [[ "$OUT_SHA" != "$WITNESS_OUT_SHA" ]]; then
  echo "STOP: witness output_sha256 mismatch" >&2
  exit 2
fi
WITNESS_DS="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["dataset_sha256"])' "$WITNESS")"
WITNESS_M="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["methodology_manifest_sha256"])' "$WITNESS")"
WITNESS_B="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["experiment_binary_sha256"])' "$WITNESS")"
if [[ "$WITNESS_DS" != "$B4_GOT" || "$WITNESS_M" != "$MANIFEST_GOT" || "$WITNESS_B" != "$BIN_SHA" ]]; then
  echo "STOP: witness identity hashes inconsistent" >&2
  exit 2
fi
echo "Witness hashes OK"
echo "G-GATE v1.1 on B4 complete. Artifacts in $OUT_DIR"
exit 0
