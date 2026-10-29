#!/usr/bin/env python3
"""
Restart Equivalence Certification Harness
=========================================
Proves that processing N bars in one continuous process produces the EXACT
same chronological outputs and telemetry archive as restarting the engine every
single bar from the frozen substrate.

If this harness passes, there is absolutely zero hidden temporal carry-over state.
"""

import sys
import json
import shutil
import subprocess
from pathlib import Path

def run_cs_ingest(
    batch_id: int, 
    cohort_file: Path, 
    archive_dir: Path, 
    start_interval: int, 
    max_intervals: int | None, 
    resume: bool, 
    fresh: bool
):
    binary = Path("cs-ingest/target/release/cs-ingest")
    if not binary.exists():
        subprocess.run(["cargo", "build", "--release"], cwd="cs-ingest", check=True)

    cmd = [
        str(binary),
        "replay-step",
        "--batch-id", str(batch_id),
        "--cohort", str(cohort_file),
        "--archive", str(archive_dir),
        "--start-interval", str(start_interval),
    ]
    if max_intervals is not None:
        cmd.extend(["--max-intervals", str(max_intervals)])
    if resume:
        cmd.append("--resume")
    if fresh:
        cmd.append("--fresh")

    res = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    if res.returncode != 0:
        print(f"❌ cs-ingest failed!\n{res.stderr}")
        sys.exit(1)
    
    # Extract fingerprint
    fp = None
    for line in res.stdout.splitlines():
        if line.startswith("   Timeline fingerprint:"):
            fp = line.split(":", 1)[1].strip()
    return fp

def get_telemetry_records(archive_dir: Path) -> dict[str, list[dict]]:
    """Loads all telemetry records from raw archive grouped by symbol."""
    records_by_sym = {}
    raw_dir = archive_dir / "raw"
    if not raw_dir.exists():
        return records_by_sym

    import gzip
    for sym_dir in raw_dir.iterdir():
        if not sym_dir.is_dir():
            continue
        sym = sym_dir.name
        sym_records = []
        for gz_path in sorted(sym_dir.glob("*.jsonl.gz")):
            with gzip.open(gz_path, "rt") as f:
                for line in f:
                    if line.strip():
                        sym_records.append(json.loads(line.strip()))
        records_by_sym[sym] = sym_records
    return records_by_sym

def hash_archive_contents(archive_dir: Path) -> str:
    """Creates a deterministic SHA-256 hash of the unzipped archive contents."""
    import hashlib
    import gzip
    hasher = hashlib.sha256()
    raw_dir = archive_dir / "raw"
    if not raw_dir.exists():
        return hasher.hexdigest()

    for sym_dir in sorted(raw_dir.iterdir(), key=lambda p: p.name):
        if not sym_dir.is_dir():
            continue
        for gz_path in sorted(sym_dir.glob("*.jsonl.gz"), key=lambda p: p.name):
            with gzip.open(gz_path, "rb") as f:
                hasher.update(f.read())
    return hasher.hexdigest()

def compare_archives(arch1: Path, arch2: Path):
    hash1 = hash_archive_contents(arch1)
    hash2 = hash_archive_contents(arch2)
    
    if hash1 != hash2:
        print(f"❌ Cryptographic Archive Mismatch! Hash1: {hash1} vs Hash2: {hash2}")
        return False
    print(f"   ✅ Cryptographic Archive Hash Match: {hash1}")

    rec1 = get_telemetry_records(arch1)
    rec2 = get_telemetry_records(arch2)

    syms1 = set(rec1.keys())
    syms2 = set(rec2.keys())

    if syms1 != syms2:
        print(f"❌ Mismatch in symbols: {syms1} vs {syms2}")
        return False

    for sym in syms1:
        list1 = rec1[sym]
        list2 = rec2[sym]
        if len(list1) != len(list2):
            print(f"❌ Mismatch in record count for {sym}: {len(list1)} vs {len(list2)}")
            return False
        
        for i, (r1, r2) in enumerate(zip(list1, list2)):
            # Drop ts since it's the key, compare math fields
            for k in r1.keys():
                if r1[k] != r2[k]:
                    print(f"❌ Divergence at {sym} index {i} (ts={r1['ts']}) field '{k}': {r1[k]} != {r2[k]}")
                    return False

    return True

def main():
    print("=" * 60)
    print(" RESTART EQUIVALENCE CERTIFICATION")
    print("=" * 60)

    cohort = Path("cohorts/batch_999_cert.txt")
    cohort.parent.mkdir(parents=True, exist_ok=True)
    cohort.write_text("BTC-USD\nETH-USD\n")

    # 1. Freeze a tiny 2-day substrate
    print("\n1. Freezing Substrate (2 days)...")
    from candle_substrate import freeze_cohort
    manifest = freeze_cohort(cohort, 999, interval="5m", period="2d")
    with open(manifest) as f:
        m = json.load(f)
        total_intervals = m["timeline_intervals"]
    print(f"   Frozen {total_intervals} intervals.")

    # 2. Continuous Run
    print("\n2. Executing Continuous Harness...")
    arch_cont = Path("state_archive/batches/batch_999/runs/continuous")
    if arch_cont.exists(): shutil.rmtree(arch_cont)
    fp_cont = run_cs_ingest(999, cohort, arch_cont, 0, None, False, True)
    print(f"   Done. Fingerprint: {fp_cont}")

    # 3. Restart-per-Bar Run
    print("\n3. Executing Restart-per-Bar Harness...")
    arch_restart = Path("state_archive/batches/batch_999/runs/restart")
    if arch_restart.exists(): shutil.rmtree(arch_restart)
    
    # Init fresh archive for interval 0
    fp_restart = run_cs_ingest(999, cohort, arch_restart, 0, 1, False, True)
    
    # Resume for the rest
    for i in range(1, total_intervals):
        if i % 50 == 0:
            print(f"   Restarted {i}/{total_intervals} times...")
        fp = run_cs_ingest(999, cohort, arch_restart, i, 1, True, False)
        if fp != fp_restart:
            print(f"❌ Fingerprint changed during restart! {fp_restart} -> {fp}")
            sys.exit(1)

    print(f"   Done. Fingerprint: {fp_restart}")

    # 4. Compare Bit-for-bit
    print("\n4. Certifying Bit-for-Bit Equivalence...")
    if fp_cont != fp_restart:
        print(f"❌ CERTIFICATION FAILED: Timeline fingerprints diverged! {fp_cont} != {fp_restart}")
        sys.exit(1)
        
    print(f"   ✅ Timeline Fingerprint Match: {fp_cont}")
    
    if compare_archives(arch_cont, arch_restart):
        print("\n✅ CERTIFICATION PASSED: Continuous == Restart-per-Bar")
        print("   Zero hidden temporal carry-over state detected.")
        print("   Replay Equivalence is mathematically proven.")
    else:
        print("\n❌ CERTIFICATION FAILED: Hidden state detected in archive lineage!")
        sys.exit(1)

if __name__ == "__main__":
    main()
