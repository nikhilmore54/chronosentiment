import json
import glob
import os
from datetime import datetime, timezone

def main():
    ledger_files = glob.glob("live_capture/ledger/entries/LIVE-005-20260917-0345-*.json")
    records = []
    
    for f in ledger_files:
        with open(f, 'r') as file:
            entry = json.load(file)
            
        # Parse snap_unix from source_snapshot_timestamp
        dt = datetime.strptime(entry["source_snapshot_timestamp"], "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc)
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
            "cohort_date": "2026-09-17",
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
        
    out_path = "datasets/today_live_dataset.json"
    with open(out_path, 'w') as out:
        json.dump(records, out, indent=2)
        
    print(f"Bridged {len(records)} LIVE-005 entries into {out_path}")

if __name__ == '__main__':
    main()
