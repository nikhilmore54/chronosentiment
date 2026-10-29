#!/usr/bin/env python3
"""
S1-06: Generate sunair_inrc_export.xml — INRC-II compatible XML export.

Maps the SunAir demo scenario and optimizer solution into the INRC-II
SchedulingPeriod XML format, suitable for interoperability with third-party
nurse/crew rostering tools that consume the INRC-II standard.

INRC-II reference: https://www.inrc-ii.be/

Usage:
    python3 scripts/gen_sunair_inrc_xml.py

Outputs:
    fixtures/demo/sunair_inrc_export.xml
"""

import json
import xml.etree.ElementTree as ET
from xml.dom import minidom
from pathlib import Path

SCHEDULE_PATH = Path("fixtures/demo/sunair_schedule.json")
SCENARIO_PATH = Path("fixtures/demo/sunair_demo.json")
OUTPUT_PATH   = Path("fixtures/demo/sunair_inrc_export.xml")

# Planning horizon anchor: 2026-07-21 00:00 UTC (Monday, week start)
HORIZON_START_DATE = "2026-07-21"
HORIZON_END_DATE   = "2026-07-27"  # 168h = 7 days


def hours_to_time(hour: int) -> str:
    """Convert an absolute hour offset into HH:MM string (wraps at 24h)."""
    return f"{hour % 24:02d}:00"


def hour_to_date(hour: int) -> str:
    """Convert an absolute hour offset to a calendar date string."""
    from datetime import date, timedelta
    base = date.fromisoformat(HORIZON_START_DATE)
    return (base + timedelta(days=hour // 24)).isoformat()


def prettify(element: ET.Element) -> str:
    """Return a pretty-printed XML string for the given Element."""
    raw = ET.tostring(element, encoding="unicode")
    reparsed = minidom.parseString(raw)
    return reparsed.toprettyxml(indent="  ", encoding=None)


def main() -> None:
    # ── Load inputs ──────────────────────────────────────────────────────────
    schedule = json.loads(SCHEDULE_PATH.read_text())
    scenario = json.loads(SCENARIO_PATH.read_text())

    shifts  = {s["id"]: s for s in scenario["shifts"]}
    workers = {w["id"]: w for w in scenario["workers"]}

    # Collect all unique skills
    all_skills: list[str] = sorted({s["required_skill"] for s in scenario["shifts"]})

    # Collect all unique shift durations (ShiftTypes in INRC-II)
    # Key: (duration_hours,) → ShiftType id
    shift_type_map: dict[int, str] = {}
    for s in scenario["shifts"]:
        dur = s["duration_hours"]
        if dur not in shift_type_map:
            shift_type_map[dur] = f"ST_{dur}H"

    # ── Root element ─────────────────────────────────────────────────────────
    root = ET.Element("SchedulingPeriod")
    root.set("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance")
    root.set("ID", "SunAir_Demo_P001")

    # ── StartDate / EndDate ──────────────────────────────────────────────────
    ET.SubElement(root, "StartDate").text = HORIZON_START_DATE
    ET.SubElement(root, "EndDate").text   = HORIZON_END_DATE

    # ── Skills ───────────────────────────────────────────────────────────────
    skills_el = ET.SubElement(root, "Skills")
    for skill in all_skills:
        s_el = ET.SubElement(skills_el, "Skill")
        ET.SubElement(s_el, "ID").text   = skill
        ET.SubElement(s_el, "Name").text = skill

    # ── ShiftTypes ───────────────────────────────────────────────────────────
    shift_types_el = ET.SubElement(root, "ShiftTypes")
    for dur_h, st_id in sorted(shift_type_map.items(), key=lambda x: x[0]):
        st_el = ET.SubElement(shift_types_el, "Shift")
        ET.SubElement(st_el, "ID").text          = st_id
        ET.SubElement(st_el, "Name").text        = f"{dur_h}-Hour Shift"
        ET.SubElement(st_el, "Duration").text    = str(dur_h * 60)  # minutes
        ET.SubElement(st_el, "StartTime").text   = "00:00"          # generic
        ET.SubElement(st_el, "Color").text       = "#4A90D9"

    # ── Employees ────────────────────────────────────────────────────────────
    employees_el = ET.SubElement(root, "Employees")
    for wid, worker in sorted(workers.items()):
        raw_skills = worker["skills"]
        worker_skills = [
            s["0"] if isinstance(s, dict) else s
            for s in raw_skills
        ]
        emp_el = ET.SubElement(employees_el, "Employee")
        ET.SubElement(emp_el, "ID").text        = str(wid)
        ET.SubElement(emp_el, "Name").text      = f"Worker_{wid}"
        contracts_el = ET.SubElement(emp_el, "ContractID")
        contracts_el.text = "StandardContract"
        skills_ref_el = ET.SubElement(emp_el, "Skills")
        for sk in worker_skills:
            ET.SubElement(skills_ref_el, "Skill").text = sk

    # ── Contracts (single standard contract) ─────────────────────────────────
    contracts_el = ET.SubElement(root, "Contracts")
    contract_el = ET.SubElement(contracts_el, "Contract")
    ET.SubElement(contract_el, "ID").text                    = "StandardContract"
    ET.SubElement(contract_el, "Description").text           = "SunAir standard 7-day contract"
    ET.SubElement(contract_el, "MaxNumAssignments").text     = "6"
    ET.SubElement(contract_el, "MinNumAssignments").text     = "1"
    ET.SubElement(contract_el, "MaxConsecutiveWorkingDays").text = "5"
    ET.SubElement(contract_el, "MinConsecutiveWorkingDays").text = "1"
    ET.SubElement(contract_el, "MinConsecutiveDaysOff").text = "1"
    ET.SubElement(contract_el, "MaxWorkingWeekends").text    = "1"

    # ── CoverRequirements (one per shift in the scenario) ────────────────────
    cover_el = ET.SubElement(root, "CoverRequirements")
    for shift in sorted(scenario["shifts"], key=lambda s: s["id"]):
        date_str = hour_to_date(shift["start_hour"])
        start_t  = hours_to_time(shift["start_hour"])
        dur_h    = shift["duration_hours"]
        st_id    = shift_type_map[dur_h]
        skill    = shift["required_skill"]

        dc_el = ET.SubElement(cover_el, "DayOfWeekCover")
        ET.SubElement(dc_el, "Day").text          = date_str
        ET.SubElement(dc_el, "ShiftTypeID").text  = st_id
        ET.SubElement(dc_el, "StartTime").text    = start_t
        ET.SubElement(dc_el, "Skill").text        = skill
        ET.SubElement(dc_el, "ShiftID").text      = str(shift["id"])
        cover_req_el = ET.SubElement(dc_el, "Cover")
        ET.SubElement(cover_req_el, "Min").text       = "1"
        ET.SubElement(cover_req_el, "Preferred").text = "1"
        ET.SubElement(cover_req_el, "Max").text       = "1"

    # ── ShiftAssignments (solution) ───────────────────────────────────────────
    assignments_el = ET.SubElement(root, "ShiftAssignments")
    for shift_id_str, worker_id in sorted(
        schedule["assignments"].items(), key=lambda x: int(x[0])
    ):
        shift_id = int(shift_id_str)
        shift    = shifts[shift_id]
        date_str = hour_to_date(shift["start_hour"])
        start_t  = hours_to_time(shift["start_hour"])
        dur_h    = shift["duration_hours"]
        st_id    = shift_type_map[dur_h]

        sa_el = ET.SubElement(assignments_el, "Assignment")
        ET.SubElement(sa_el, "Employee").text    = str(worker_id)
        ET.SubElement(sa_el, "ShiftID").text     = str(shift_id)
        ET.SubElement(sa_el, "ShiftTypeID").text = st_id
        ET.SubElement(sa_el, "Date").text        = date_str
        ET.SubElement(sa_el, "StartTime").text   = start_t
        ET.SubElement(sa_el, "Duration").text    = str(dur_h * 60)
        ET.SubElement(sa_el, "Skill").text       = shift["required_skill"]

    # ── SolutionQuality (metadata from optimizer) ─────────────────────────────
    quality_el = ET.SubElement(root, "SolutionQuality")
    ET.SubElement(quality_el, "HardViolations").text  = str(schedule["hard_violations"])
    ET.SubElement(quality_el, "RestViolations").text  = str(schedule["rest_violations"])
    ET.SubElement(quality_el, "FitnessScore").text    = str(round(schedule["fitness"], 4))
    ET.SubElement(quality_el, "FairnessPenalty").text = str(round(schedule["fairness_penalty"], 4))
    ET.SubElement(quality_el, "FatiguePenalty").text  = str(round(schedule["fatigue_penalty"], 4))
    ET.SubElement(quality_el, "GeneratedBy").text     = "UltraCrew Coralys Optimizer"
    ET.SubElement(quality_el, "GeneratedAt").text     = "2026-07-22T22:52:00+05:30"

    # ── Write output ─────────────────────────────────────────────────────────
    xml_str = prettify(root)
    # Remove the redundant <?xml ...?> header added by minidom (we add our own)
    lines = xml_str.splitlines()
    if lines and lines[0].startswith("<?xml"):
        lines[0] = '<?xml version="1.0" encoding="UTF-8"?>'
    OUTPUT_PATH.write_text("\n".join(lines))

    # Count elements for confirmation
    n_employees   = len(scenario["workers"])
    n_shifts      = len(scenario["shifts"])
    n_assignments = len(schedule["assignments"])
    print(f"INRC-II XML written to {OUTPUT_PATH}")
    print(f"  Employees:   {n_employees}")
    print(f"  Shifts:      {n_shifts}")
    print(f"  Assignments: {n_assignments}")
    print(f"  Skills:      {', '.join(all_skills)}")
    print(f"  ShiftTypes:  {', '.join(shift_type_map.values())}")


if __name__ == "__main__":
    main()