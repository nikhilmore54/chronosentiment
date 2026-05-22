import os
import shutil
import time
from pathlib import Path
import argparse

# Retention Tiers (in days)
TIER_2_WARM_DAYS = 14
TIER_3_DEBRIS_DAYS = 3

# Do NOT delete or quarantine these files (Tier 1)
PROTECTED_FILES = {
    "propagation_snapshots.jsonl",
    "trl_summary.json",
    "rho_n.json",
    "live_session_steps.jsonl",
    "provider_propagation_trace.jsonl"
}

def get_dir_size(path: Path) -> int:
    total = 0
    for dirpath, _, filenames in os.walk(path):
        for f in filenames:
            fp = os.path.join(dirpath, f)
            if not os.path.islink(fp):
                total += os.path.getsize(fp)
    return total

def format_size(size_bytes: int) -> str:
    if size_bytes < 1024:
        return f"{size_bytes} B"
    elif size_bytes < 1024**2:
        return f"{size_bytes/1024:.1f} KB"
    else:
        return f"{size_bytes/(1024**2):.1f} MB"

def inventory(archive_dir: Path):
    print("=" * 60)
    print(" 🧹 CHRONOSENTIMENT STORAGE INVENTORY")
    print("=" * 60)
    
    if not archive_dir.exists():
        print(f"Archive directory {archive_dir} not found.")
        return

    for item in archive_dir.iterdir():
        if item.is_dir():
            size = get_dir_size(item)
            print(f"  {item.name:<25} : {format_size(size)}")

def quarantine_debris(archive_dir: Path, quarantine_dir: Path, execute: bool):
    print("\n" + "=" * 60)
    print(" 🛡️  QUARANTINE SCAN (Tier 3 Debris & Old Tier 2)")
    print("=" * 60)
    
    now = time.time()
    if not quarantine_dir.exists() and execute:
        quarantine_dir.mkdir(parents=True)

    moved_count = 0
    moved_bytes = 0

    for root, dirs, files in os.walk(archive_dir):
        if "archive_quarantine" in root:
            continue
            
        for file in files:
            file_path = Path(root) / file
            
            # Skip protected files entirely
            if file in PROTECTED_FILES:
                continue
                
            # Skip frozen manifests (Tier 1)
            if "manifest" in file.lower() and file.endswith(".json"):
                continue

            stat = file_path.stat()
            age_days = (now - stat.st_mtime) / (24 * 3600)
            
            is_debris = file.endswith(".log") or "nohup" in file or "test" in root.lower() or "debug" in root.lower() or "replay_equiv" in root.lower()
            
            should_move = False
            if is_debris and age_days > TIER_3_DEBRIS_DAYS:
                should_move = True
            elif age_days > TIER_2_WARM_DAYS:
                should_move = True

            if should_move:
                rel_path = file_path.relative_to(archive_dir)
                target_path = quarantine_dir / rel_path
                
                print(f"  [Quarantine] {rel_path} (Age: {age_days:.1f} days)")
                
                if execute:
                    target_path.parent.mkdir(parents=True, exist_ok=True)
                    shutil.move(str(file_path), str(target_path))
                    moved_bytes += stat.st_size
                    moved_count += 1

    if execute:
        print(f"\n✅ Quarantined {moved_count} files ({format_size(moved_bytes)}).")
    else:
        print("\nDRY RUN: Pass --execute to actually move files.")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="ChronoSentiment Storage Governor")
    parser.add_argument("--archive", default="state_archive", help="Path to archive")
    parser.add_argument("--quarantine", default="archive_quarantine", help="Path to quarantine")
    parser.add_argument("--execute", action="store_true", help="Execute the moves")
    args = parser.parse_args()
    
    archive_path = Path(args.archive)
    quarantine_path = Path(args.quarantine)
    
    inventory(archive_path)
    quarantine_debris(archive_path, quarantine_path, args.execute)
