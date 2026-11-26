"""
Decision State Engine — v0.4
============================
Maps opportunity inputs + observed path state → ACTION / STATE / HORIZON / WHY / RISK
at each checkpoint: ENTRY, H60, H120, H180.

Rules are frozen from the nine-iteration discovery loop (v0.1–v0.9).
No new thresholds. No new classifiers. Pure translation of discovered model.

Usage
-----
    from scripts.decision_state_engine import DecisionStateEngine
    engine = DecisionStateEngine()
    state = engine.at_entry(record)
    state_h120 = engine.at_h120(record, entry_state)

Output schema (all checkpoints)
--------------------------------
{
    "checkpoint":   "ENTRY" | "H60" | "H120" | "H180",
    "action":       "ACT" | "MONITOR" | "AVOID" | "UPGRADE" | "CONTINUE",
    "state":        "ENTER" | "WAIT-HIGH" | "WAIT-MID" | "WAIT-LOW" | "AVOID"
                  | "ENTER-LATE" | "WAIT-LATE" | "AVOID-LATE" | "UNKNOWN",
    "confidence":   "HIGH" | "MODERATE" | "LOW",
    "horizon":      str,   # human-readable next decision point
    "why":          str,   # one-sentence explanation
    "risk":         str,   # one-sentence risk note
    "metrics": {           # key supporting numbers
        "oqs":             int | None,
        "momentum":        float | None,
        "h60_class":       str | None,
        "h120_ret":        float | None,
        "h300_hist_win":   float | None,   # historical win rate for this state
        "h300_hist_pf":    float | None,   # historical profit factor
        "h300_hist_med":   float | None,   # historical H300 median return
    }
}
"""

from __future__ import annotations
from typing import Any

# ── Historical performance reference (frozen from v0.9 discovery) ─────────────
# Format: (win_rate, profit_factor, h300_median_return)
HIST = {
    # SHORT
    ("SHORT", "ENTER"):      (0.717, 13.35, 0.0076),
    ("SHORT", "WAIT-HIGH"):  (0.775, 99.0,  0.0056),
    ("SHORT", "WAIT-LOW"):   (0.044, 0.04,  -0.0030),
    ("SHORT", "AVOID"):      (0.261, 0.23,  -0.0032),
    # LONG
    ("LONG",  "ENTER"):      (0.750, 19.46, 0.0061),
    ("LONG",  "WAIT-HIGH"):  (0.824, 72.36, 0.0066),
    ("LONG",  "WAIT-MID"):   (0.595, 10.06, 0.0026),
    ("LONG",  "WAIT-LOW"):   (0.283, 0.67,  -0.0023),
    ("LONG",  "AVOID"):      (0.116, 0.08,  -0.0065),
    # LONG WAIT-MID reassessment at H120
    ("LONG",  "ENTER-LATE"): (0.696, 11.67, 0.0046),
    ("LONG",  "WAIT-LATE"):  (0.500, 6.47,  0.0018),
    ("LONG",  "AVOID-LATE"): (0.000, None,  0.0002),
}

THRESHOLD = 0.002  # 0.2% — FAV/ADV boundary


def _bucket(ret: float | None, t: float = THRESHOLD) -> str:
    if ret is None:
        return "FLAT"
    return "FAV" if ret > t else ("ADV" if ret < -t else "FLAT")


def _hist(direction: str, state: str) -> dict:
    key = (direction, state)
    if key not in HIST:
        return {"h300_hist_win": None, "h300_hist_pf": None, "h300_hist_med": None}
    win, pf, med = HIST[key]
    return {"h300_hist_win": win, "h300_hist_pf": pf, "h300_hist_med": med}


class DecisionStateEngine:
    """
    Stateless engine. Each method takes a record dict (from p4_opportunity_dataset.json
    or a live equivalent) and returns a decision state dict.

    Record schema expected:
        direction:                  "LONG" | "SHORT"
        opportunity_dimensions:
            opportunity_quality_score:  int
        path_5m:
            h60_classification:     "ENTER" | "WAIT" | "AVOID"
            momentum_persistence:   float   (0–1)
            h15_ret:                float | None
            h30_ret:                float | None
            h60_ret:                float | None
            h120_ret:               float | None
            h180_ret:               float | None
            h300_ret:               float | None
            mfe_h60:                float | None
            mfe_h120:               float | None
    """

    # ── Entry-time classification ─────────────────────────────────────────────

    def _entry_state(self, record: dict) -> str:
        direction = record["direction"]
        cls = record["path_5m"]["h60_classification"]
        oqs = record["opportunity_dimensions"]["opportunity_quality_score"]
        mp  = record["path_5m"].get("momentum_persistence", 0.0)

        if cls == "ENTER":
            return "ENTER"
        if cls == "AVOID":
            return "AVOID"
        # WAIT branch
        if direction == "SHORT":
            return "WAIT-HIGH" if mp >= 0.5 else "WAIT-LOW"
        else:  # LONG
            if oqs >= 65 and mp >= 0.5:
                return "WAIT-HIGH"
            if 40 <= oqs < 65 and mp >= 0.5:
                return "WAIT-MID"
            return "WAIT-LOW"

    def at_entry(self, record: dict) -> dict:
        direction = record["direction"]
        state     = self._entry_state(record)
        oqs       = record["opportunity_dimensions"]["opportunity_quality_score"]
        mp        = record["path_5m"].get("momentum_persistence", 0.0)
        h60_cls   = record["path_5m"]["h60_classification"]

        action, confidence, horizon, why, risk = _entry_narrative(direction, state, oqs, mp)

        return {
            "checkpoint": "ENTRY",
            "action":     action,
            "state":      state,
            "confidence": confidence,
            "horizon":    horizon,
            "why":        why,
            "risk":       risk,
            "metrics": {
                "oqs":           oqs,
                "momentum":      round(mp, 3),
                "h60_class":     h60_cls,
                "h120_ret":      None,
                **_hist(direction, state),
            },
        }

    # ── H60 update ────────────────────────────────────────────────────────────

    def at_h60(self, record: dict, entry_state: dict) -> dict:
        direction  = record["direction"]
        state      = entry_state["state"]
        h60_ret    = record["path_5m"].get("h60_ret")
        mfe_h60    = record["path_5m"].get("mfe_h60")
        h60_bucket = _bucket(h60_ret)

        action, confidence, horizon, why, risk = _h60_narrative(
            direction, state, h60_bucket, h60_ret, mfe_h60
        )

        return {
            "checkpoint": "H60",
            "action":     action,
            "state":      state,
            "confidence": confidence,
            "horizon":    horizon,
            "why":        why,
            "risk":       risk,
            "metrics": {
                "oqs":           entry_state["metrics"]["oqs"],
                "momentum":      entry_state["metrics"]["momentum"],
                "h60_class":     entry_state["metrics"]["h60_class"],
                "h60_ret":       round(h60_ret * 100, 3) if h60_ret is not None else None,
                "mfe_h60":       round(mfe_h60 * 100, 3) if mfe_h60 is not None else None,
                "h120_ret":      None,
                **_hist(direction, state),
            },
        }

    # ── H120 reassessment ─────────────────────────────────────────────────────

    def at_h120(self, record: dict, entry_state: dict) -> dict:
        direction  = record["direction"]
        state      = entry_state["state"]
        h120_ret   = record["path_5m"].get("h120_ret")
        mfe_h120   = record["path_5m"].get("mfe_h120")

        # Only LONG WAIT-MID gets a formal reassessment
        if direction == "LONG" and state == "WAIT-MID":
            new_state = _h120_reassess(h120_ret, mfe_h120)
        else:
            new_state = state  # all other states: no change at H120

        action, confidence, horizon, why, risk = _h120_narrative(
            direction, state, new_state, h120_ret
        )

        return {
            "checkpoint": "H120",
            "action":     action,
            "state":      new_state,
            "confidence": confidence,
            "horizon":    horizon,
            "why":        why,
            "risk":       risk,
            "metrics": {
                "oqs":           entry_state["metrics"]["oqs"],
                "momentum":      entry_state["metrics"]["momentum"],
                "h60_class":     entry_state["metrics"]["h60_class"],
                "h120_ret":      round(h120_ret * 100, 3) if h120_ret is not None else None,
                **_hist(direction, new_state),
            },
        }

    # ── H180 confirmation ─────────────────────────────────────────────────────

    def at_h180(self, record: dict, h120_state: dict) -> dict:
        direction = record["direction"]
        state     = h120_state["state"]
        h180_ret  = record["path_5m"].get("h180_ret")
        h180_bkt  = _bucket(h180_ret)

        action, confidence, horizon, why, risk = _h180_narrative(
            direction, state, h180_bkt, h180_ret
        )

        return {
            "checkpoint": "H180",
            "action":     action,
            "state":      state,
            "confidence": confidence,
            "horizon":    horizon,
            "why":        why,
            "risk":       risk,
            "metrics": {
                "oqs":           h120_state["metrics"]["oqs"],
                "momentum":      h120_state["metrics"]["momentum"],
                "h60_class":     h120_state["metrics"]["h60_class"],
                "h120_ret":      h120_state["metrics"]["h120_ret"],
                "h180_ret":      round(h180_ret * 100, 3) if h180_ret is not None else None,
                **_hist(direction, state),
            },
        }

    # ── Convenience: full timeline ────────────────────────────────────────────

    def full_timeline(self, record: dict) -> list[dict]:
        e    = self.at_entry(record)
        h60  = self.at_h60(record, e)
        h120 = self.at_h120(record, e)
        h180 = self.at_h180(record, h120)
        return [e, h60, h120, h180]


# ── Narrative helpers ─────────────────────────────────────────────────────────

def _entry_narrative(direction, state, oqs, mp):
    if state == "ENTER":
        if direction == "SHORT":
            return ("ACT", "HIGH",
                    "Act within 5-hour session",
                    "H15 and H60 both favourable — strong intraday SHORT signal.",
                    "Avg loss −0.89% if wrong; monitor for early adverse reversal.")
        else:
            return ("ACT", "HIGH",
                    "Act now; opportunity peaks around H60",
                    "H15 and H60 both favourable — strong intraday LONG signal.",
                    "Return decays after H60; do not hold expecting daily continuation.")

    if state == "WAIT-HIGH":
        if direction == "SHORT":
            return ("ACT", "HIGH",
                    "Act within 5-hour session",
                    f"Momentum persistence {mp:.2f} — price moving favourably for >{mp*100:.0f}% of session.",
                    "Avg loss only −0.20% if wrong; PF >99x historically.")
        else:
            return ("ACT", "HIGH",
                    "Act now; hold through session",
                    f"OQS {oqs} and momentum {mp:.2f} — high-quality LONG with sustained path.",
                    "Daily return stronger than H300; allow full session to develop.")

    if state == "WAIT-MID":
        return ("MONITOR", "MODERATE",
                "Reassess at H120 (2 hours)",
                f"OQS {oqs} with momentum {mp:.2f} — developing LONG opportunity, insufficient early signal.",
                "Winners and losers indistinguishable before H120; do not act at entry.")

    if state == "WAIT-LOW":
        return ("AVOID", "LOW",
                "No action required",
                f"Momentum persistence {mp:.2f} — price not moving favourably during session.",
                "PF 0.04x historically; acting here destroys value.")

    if state == "AVOID":
        return ("AVOID", "HIGH",
                "No action required",
                "H15 and H60 both adverse — confirmed negative intraday path.",
                "PF 0.08–0.23x historically; strong avoidance signal.")

    return ("AVOID", "LOW", "Unknown", "Unclassified state.", "No historical reference.")


def _h60_narrative(direction, state, h60_bucket, h60_ret, mfe_h60):
    ret_str = f"{h60_ret*100:+.2f}%" if h60_ret is not None else "—"
    mfe_str = f"{mfe_h60*100:+.2f}%" if mfe_h60 is not None else "—"

    if state in ("ENTER", "WAIT-HIGH"):
        if h60_bucket == "FAV":
            return ("ACT", "HIGH",
                    "Continue; reassess at H120 if needed",
                    f"H60 return {ret_str}, MFE {mfe_str} — opportunity developing as expected.",
                    "Path on track; watch for reversal after H60 peak (LONG) or continuation (SHORT).")
        elif h60_bucket == "ADV":
            return ("MONITOR", "LOW",
                    "Reassess immediately",
                    f"H60 return {ret_str} — path moving adversely despite strong entry signal.",
                    "Early adverse path is a warning; consider reducing exposure.")
        else:
            return ("MONITOR", "MODERATE",
                    "Continue monitoring; reassess at H120",
                    f"H60 return {ret_str} — path flat, opportunity not yet confirmed.",
                    "Flat H60 is neutral; wait for H120 to clarify direction.")

    if state == "WAIT-MID":
        return ("MONITOR", "MODERATE",
                "Continue monitoring; formal reassessment at H120",
                f"H60 return {ret_str} — Group B opportunity, H60 does not separate winners from losers.",
                "Do not act on H60 signal alone for WAIT-MID; wait for H120.")

    if state in ("WAIT-LOW", "AVOID"):
        return ("AVOID", "HIGH",
                "No action required",
                f"H60 return {ret_str} — confirmed low-quality path.",
                "No change from entry assessment.")

    return ("MONITOR", "LOW", "Unknown", "Unclassified state at H60.", "No historical reference.")


def _h120_reassess(h120_ret, mfe_h120):
    if h120_ret is None:
        return "WAIT-LATE"
    if mfe_h120 is not None and mfe_h120 < 0.001:
        return "AVOID-LATE"
    b = _bucket(h120_ret)
    if b == "FAV":
        return "ENTER-LATE"
    if b == "ADV":
        return "AVOID-LATE"
    return "WAIT-LATE"


def _h120_narrative(direction, entry_state, new_state, h120_ret):
    ret_str = f"{h120_ret*100:+.2f}%" if h120_ret is not None else "—"

    if entry_state == "WAIT-MID":
        if new_state == "ENTER-LATE":
            return ("UPGRADE", "HIGH",
                    "Act now; H120 confirms opportunity",
                    f"H120 return {ret_str} — FAV path at 2-hour mark upgrades WAIT-MID to ENTER-LATE.",
                    "PF 11.67x, 70% win historically; avg loss small.")
        if new_state == "WAIT-LATE":
            return ("CONTINUE", "MODERATE",
                    "Continue monitoring; reassess at H180",
                    f"H120 return {ret_str} — flat path, opportunity still developing.",
                    "WAIT-LATE still positive (PF 6.47x, 50% win); do not exit prematurely.")
        if new_state == "AVOID-LATE":
            return ("AVOID", "MODERATE",
                    "Consider exiting or reducing",
                    f"H120 return {ret_str} — adverse path at 2-hour mark.",
                    "N=1 historically; avoidance rule not yet fully evidenced.")

    # Non-WAIT-MID states: H120 is informational only
    if new_state in ("ENTER", "WAIT-HIGH"):
        return ("ACT", "HIGH",
                "Continue; H120 confirms",
                f"H120 return {ret_str} — path continuing favourably.",
                "On track with entry assessment.")
    if new_state in ("WAIT-LOW", "AVOID"):
        return ("AVOID", "HIGH",
                "No action required",
                f"H120 return {ret_str} — path confirmed adverse.",
                "No change from entry assessment.")

    return ("MONITOR", "LOW", "Unknown", "Unclassified state at H120.", "No historical reference.")


def _h180_narrative(direction, state, h180_bucket, h180_ret):
    ret_str = f"{h180_ret*100:+.2f}%" if h180_ret is not None else "—"

    if state in ("ENTER", "WAIT-HIGH", "ENTER-LATE"):
        if h180_bucket == "FAV":
            return ("ACT", "HIGH",
                    "Final hour; opportunity confirmed",
                    f"H180 return {ret_str} — path sustained through 3-hour mark.",
                    "Allow to run to H300; watch for late reversal.")
        elif h180_bucket == "ADV":
            return ("MONITOR", "LOW",
                    "Reassess; late adverse path",
                    f"H180 return {ret_str} — path reversed after strong early development.",
                    "Late reversal is a known failure mode; consider reducing.")
        else:
            return ("CONTINUE", "MODERATE",
                    "Final hour; path flat",
                    f"H180 return {ret_str} — opportunity developing slowly.",
                    "Flat H180 is neutral; allow to H300.")

    if state == "WAIT-LATE":
        return ("CONTINUE", "MODERATE",
                "Final hour; continue monitoring",
                f"H180 return {ret_str} — WAIT-LATE path, still positive historically.",
                "PF 6.47x; do not exit on flat path alone.")

    if state in ("WAIT-LOW", "AVOID", "AVOID-LATE"):
        return ("AVOID", "HIGH",
                "No action required",
                f"H180 return {ret_str} — confirmed adverse path.",
                "No change from prior assessment.")

    return ("MONITOR", "LOW", "Unknown", "Unclassified state at H180.", "No historical reference.")


# ── CLI demo ──────────────────────────────────────────────────────────────────

if __name__ == "__main__":
    import json, sys

    data = json.load(open("datasets/p4_opportunity_dataset.json"))
    engine = DecisionStateEngine()

    # Show one example of each entry state
    shown = set()
    examples = []
    for r in data:
        e = engine.at_entry(r)
        key = (r["direction"], e["state"])
        if key not in shown:
            shown.add(key)
            examples.append((r, e))

    for record, entry in sorted(examples, key=lambda x: (x[0]["direction"], x[1]["state"])):
        timeline = engine.full_timeline(record)
        direction = record["direction"]
        ticker = record.get("instrument", "?")
        print(f"\n{'='*70}")
        print(f"{direction} | {ticker}")
        for step in timeline:
            cp = step["checkpoint"]
            print(f"\n  [{cp}]")
            print(f"    ACTION:     {step['action']}")
            print(f"    STATE:      {step['state']}")
            print(f"    CONFIDENCE: {step['confidence']}")
            print(f"    HORIZON:    {step['horizon']}")
            print(f"    WHY:        {step['why']}")
            print(f"    RISK:       {step['risk']}")
            m = step["metrics"]
            hist_win = m.get("h300_hist_win")
            hist_pf  = m.get("h300_hist_pf")
            hist_med = m.get("h300_hist_med")
            if hist_win is not None:
                pf_str = f"{hist_pf:.2f}x" if hist_pf and hist_pf <= 99 else (">99x" if hist_pf else "—")
                print(f"    HIST:       {hist_win*100:.0f}% win | PF {pf_str} | med {hist_med*100:+.2f}%")