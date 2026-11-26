import os
import csv
from datetime import datetime, timezone

BASE_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BENCHMARK_DIR = os.path.join(BASE_DIR, "benchmarks", "gerad-g2014-22")
OUT_DIR = os.path.join(BASE_DIR, "apps", "ultracrew-pilot-portal", "src")

# The epoch used to normalize timestamps to integers
EPOCH = datetime(2000, 1, 1, 11, 0, 0, tzinfo=timezone.utc)

def parse_utc(s: str) -> datetime:
    return datetime.fromisoformat(s.replace("Z", "+00:00"))

def generate_for_instance(instance_name: str, out_file: str):
    instance_dir = os.path.join(BENCHMARK_DIR, instance_name)
    crew_path = os.path.join(instance_dir, "crew.csv")
    duties_path = os.path.join(instance_dir, "duties.csv")
    
    if not os.path.exists(crew_path) or not os.path.exists(duties_path):
        print(f"Skipping {instance_name} (missing files)")
        return
    
    # 1. Parse Crew
    workers = []
    qualifications = set()
    bases = set()
    crew_map = {}
    with open(crew_path, newline='') as f:
        reader = csv.DictReader(f)
        for i, row in enumerate(reader):
            cid = row["crew_id"]
            qual = row["qualification"]
            base = row["base"]
            qualifications.add(qual)
            bases.add(base)
            numeric_id = i + 1
            worker = {
                "id": numeric_id,
                "skills": [qual],
                "name": row["name"],
                "base": base,
                "gerad_id": cid,
                "contract_type": row["contract_type"]
            }
            workers.append(worker)
            crew_map[cid] = numeric_id

    # 2. Parse Duties
    shifts = []
    max_hour = 0
    with open(duties_path, newline='') as f:
        reader = csv.DictReader(f)
        for i, row in enumerate(reader):
            duty_id = row["duty_id"]
            crew_id = row["crew_id"]
            flight_ids = row["flight_ids"]
            report = parse_utc(row["report_utc"])
            release = parse_utc(row["release_utc"])
            
            start_hour = int(round((report - EPOCH).total_seconds() / 3600))
            duration = int(round((release - report).total_seconds() / 3600))
            
            # Find the worker to get the required_skill
            worker = next(w for w in workers if w["gerad_id"] == crew_id)
            
            shift = {
                "id": i + 1,
                "start_hour": start_hour,
                "duration_hours": duration,
                "required_skill": worker["skills"][0],
                "gerad_duty_id": duty_id,
                "gerad_crew_id": crew_id,
                "flight_ids": flight_ids
            }
            shifts.append(shift)
            
            end_hour = start_hour + duration
            if end_hour > max_hour:
                max_hour = end_hour
                
    horizon = max_hour + 24 # Add a buffer to the horizon

    instance_num = instance_name.replace("instance", "")
    var_prefix = f"GERAD_INSTANCE{instance_num}"
    
    meta = {
        "source": f"GERAD G-2014-22 Instance {instance_num} (Kasirzadeh, Saddoune & Soumis 2014)",
        "total_crew": len(workers),
        "total_duties": len(shifts),
        "qualifications": sorted(list(qualifications)),
        "bases": sorted(list(bases)),
        "horizon_hours": horizon,
        "max_hours_per_worker": 80,
        "normalization_offset_hours": 11,
        "note": "start_hour values are normalized (min subtracted) so the optimizer receives relative hours rather than absolute epoch offsets. Temporal structure (gaps, overlaps, rest periods) is preserved."
    }
    
    # 3. Write JS File
    import json
    with open(out_file, "w") as f:
        f.write("// AUTO-GENERATED — do not edit by hand.\n")
        f.write(f"// Source: benchmarks/gerad-g2014-22/{instance_name}/crew.csv + duties.csv\n")
        f.write("// Generator: scripts/gen_all_gerad_js.py\n")
        f.write(f"// Pipeline: GERAD {instance_name} → Roster/Duty/CrewMember → workers[]/shifts[]\n")
        f.write("//\n")
        f.write(f"// GERAD G-2014-22 Instance {instance_num} (Kasirzadeh, Saddoune & Soumis 2014)\n")
        f.write(f"// {len(workers)} crew · {len(shifts)} duties · {horizon}h horizon\n")
        f.write("// Normalization offset: 11h subtracted from all start_hour values.\n\n")
        
        f.write(f"export const {var_prefix}_META = ")
        json.dump(meta, f, indent=2)
        f.write(";\n\n")
        
        f.write(f"// workers[]: each GERAD CrewMember projected to UltraCrew Worker schema.\n")
        f.write(f"// id: numeric crew_id (C0001 → 1), skills: [qualification]\n")
        f.write(f"export const {var_prefix}_WORKERS = ")
        json.dump(workers, f, indent=2)
        f.write(";\n\n")
        
        f.write(f"// shifts[]: each GERAD Duty projected to UltraCrew Shift schema.\n")
        f.write(f"// id: numeric duty_id, start_hour: normalized FDP report time,\n")
        f.write(f"// duration_hours: FDP length (release - report), required_skill: crew qualification.\n")
        f.write(f"export const {var_prefix}_SHIFTS = ")
        json.dump(shifts, f, indent=2)
        f.write(";\n")
        
    print(f"Generated {out_file}")

if __name__ == "__main__":
    for i in range(1, 8):
        generate_for_instance(f"instance{i}", os.path.join(OUT_DIR, f"geradInstance{i}.js"))
