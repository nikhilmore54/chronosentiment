import json
import glob
import os
import sys
import argparse
from datetime import datetime, timezone

def main():
    parser = argparse.ArgumentParser(description="Bridge LIVE-005 artifacts into today_live_dataset.json")
    parser.add_argument("--as-of-date", required=True, help="Session date in YYYY-MM-DD format")
    args = parser.parse_args()
    
    session_date = args.as_of_date
    session_date_stripped = session_date.replace("-", "")
    
    # Select only LIVE-005 ledger artifacts belonging to that exact session date
    all_ledger_files = glob.glob(f"live_capture/ledger/entries/LIVE-005-{session_date_stripped}-*.json")
    
    if not all_ledger_files:
        print(f"ERROR: No LIVE-005 artifacts found for session date {session_date}")
        sys.exit(1)
        
    # Group files by their Run ID timestamp (HHMM) to extract only the latest generation
    # Format is: LIVE-005-YYYYMMDD-HHMM-TICKER.json
    run_groups = {}
    for f in all_ledger_files:
        basename = os.path.basename(f)
        parts = basename.split('-')
        if len(parts) >= 4:
            hhmm = parts[3]
            run_groups.setdefault(hhmm, []).append(f)
            
    if not run_groups:
        print(f"ERROR: Could not parse Run IDs from artifacts for {session_date}")
        sys.exit(1)
        
    # The canonical generation is the latest chronological run for the session date
    latest_hhmm = sorted(run_groups.keys())[-1]
    ledger_files = run_groups[latest_hhmm]
    
    print(f"Detected {len(run_groups)} distinct generations for {session_date}.")
    print(f"Selecting canonical run {latest_hhmm} with {len(ledger_files)} decisions.")
        
    records = []
    
    for f in ledger_files:
        with open(f, 'r') as file:
            entry = json.load(file)
            
        # Parse snap_unix from source_snapshot_timestamp
        dt_str = entry["source_snapshot_timestamp"].replace("Z", "+00:00")
        dt = datetime.fromisoformat(dt_str)
        snap_unix = int(dt.timestamp())
        
        raw_action = entry.get("action", "Watch")
        if raw_action in ["Buy", "Sell"]:
            h60_classification = "ENTER"
            oqs = 100
        else:
            h60_classification = "AVOID"
            oqs = 0
        
        # We synthesize a RawOpportunityRecord
        rec = {
            "decision_id": entry["decision_id"],
            "date": session_date,
            "cohort_date": session_date,
            "ticker": entry["ticker"],
            "direction": entry["direction"].upper(),
            "action": raw_action,
            "opportunity_dimensions": {
                "opportunity_quality_score": oqs
            },
            "path_5m": {
                "h60_classification": h60_classification,
                "entry_price": entry.get("reference_price")
            },
            "reference_price": entry.get("reference_price"),
            "snap_unix": snap_unix,
            "adaptive_target": entry.get("adaptive_target"),
            "adaptive_risk": entry.get("adaptive_risk"),
            "adaptive_horizon_sessions": entry.get("adaptive_horizon_sessions")
        }
        records.append(rec)
        
    out_path = f"datasets/live005_{session_date.replace("-", "")}.json"
    with open(out_path, 'w') as out:
        json.dump(records, out, indent=2)
        
    print(f"Bridged {len(records)} LIVE-005 entries for {session_date} into {out_path}")

if __name__ == '__main__':
    main()
