# scripts/nse_calendar.py
"""Utility to determine NSE trading days.
Uses a simple weekday filter plus a static holiday CSV file.
The CSV should contain one column `date` in YYYY-MM-DD format.
"""
import csv
import datetime
from pathlib import Path

HOLIDAY_CSV = Path(__file__).parent / "nse_holidays.csv"

def load_holidays() -> set:
    holidays = set()
    if HOLIDAY_CSV.is_file():
        with open(HOLIDAY_CSV, newline="") as f:
            reader = csv.DictReader(f)
            for row in reader:
                try:
                    d = datetime.datetime.strptime(row["date"], "%Y-%m-%d").date()
                    holidays.add(d)
                except Exception:
                    continue
    return holidays

_HOLIDAYS = load_holidays()

def is_trading_day(date: datetime.date) -> bool:
    """Return True if *date* is a regular NSE trading day.
    Excludes weekends (Saturday/Sunday) and dates present in the holiday list.
    """
    if date.weekday() >= 5:  # 5 = Saturday, 6 = Sunday
        return False
    if date in _HOLIDAYS:
        return False
    return True

def trading_days_between(start: str, end: str) -> list:
    """Return a list of date strings (YYYY-MM-DD) that are trading days
    inclusive of *start* and *end*.
    """
    start_dt = datetime.datetime.strptime(start, "%Y-%m-%d").date()
    end_dt = datetime.datetime.strptime(end, "%Y-%m-%d").date()
    days = []
    cur = start_dt
    while cur <= end_dt:
        if is_trading_day(cur):
            days.append(cur.isoformat())
        cur += datetime.timedelta(days=1)
    return days
