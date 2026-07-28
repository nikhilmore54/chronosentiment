# Pilot Facilitator Guide v1.0

**UltraCrew Workforce Decision Assessment**
**For internal use — facilitator reference only**

---

## Purpose

This guide ensures every WOA/WDX demonstration is delivered consistently, that the evidence collected is comparable across sessions, and that the facilitator is prepared for the most common objections and questions.

A demonstration is not a sales pitch. It is a structured diagnostic conversation. The facilitator's job is to listen more than they speak.

---

## Before the Meeting

**48 hours before:**
- Confirm the attendee list and their roles (operations, HR, finance, executive)
- Research the organisation: fleet size, route network, crew base locations, any recent operational disruptions (delays, strikes, weather events)
- Prepare one SunAir Regional scenario that mirrors their operational context (short-haul regional, mixed fleet, or long-haul as appropriate)
- Verify the public deployment is live: open the URL in an incognito window and complete one full optimisation cycle

**30 minutes before:**
- Wake up the Render backend (visit `/api/health` — free tier sleeps after 15 min inactivity)
- Confirm Supabase is accepting writes (check the pilot_sessions table in the Supabase dashboard)
- Have `./start-demo.sh` ready as a fallback if the hosted version is unavailable
- Close all unnecessary browser tabs and notifications

---

## Meeting Structure (60 minutes)

| Segment | Duration | Purpose |
|---------|----------|---------|
| 1. Opening and context | 5 min | Establish the conversation frame |
| 2. WOA diagnostic interview | 15 min | Understand their current state |
| 3. Live demonstration | 15–20 min | Show the platform against their context |
| 4. WDX discussion | 10 min | Explore the path to improvement |
| 5. Pilot proposal | 10 min | Gauge interest in a structured pilot |
| 6. Close and next steps | 5 min | Agree concrete actions |

---

## Segment 1 — Opening and Context (5 minutes)

**Objective:** Frame the conversation as a diagnostic, not a product demo.

**Opening statement (adapt as needed):**

> "Thank you for making time. I want to be upfront about what this session is and isn't. It isn't a product demonstration where I show you features. It's a structured conversation about how your organisation currently makes workforce decisions, and whether there's a better approach. I'll show you a working system, but only to make the conversation concrete. The most valuable thing I'll take away today is your perspective, not the other way around."

**Questions to establish context:**
- "How many crew members are you currently scheduling?"
- "What does your current scheduling process look like — is it manual, tool-assisted, or fully automated?"
- "When was the last time a disruption significantly affected your operations, and how did you respond?"

---

## Segment 2 — WOA Diagnostic Interview (15 minutes)

**Objective:** Establish their current maturity tier and identify the most significant operational pain point.

### WOA Maturity Tiers (for facilitator reference)

| Tier | Label | Characteristics |
|------|-------|----------------|
| 1 | Reactive | Decisions made in response to events; no forward planning; high manual effort |
| 2 | Managed | Structured processes exist; some tooling; decisions are consistent but slow |
| 3 | Optimised | Quantitative analysis informs decisions; trade-offs are explicit; recovery is planned |
| 4 | Adaptive | Real-time decision support; continuous improvement; decisions are auditable |

### Interview Questions

**Scheduling:**
- "How long does it take to produce a compliant crew schedule from scratch?"
- "What percentage of your scheduling time is spent on constraint checking versus actual planning?"
- "How do you currently handle last-minute changes — sick leave, aircraft swaps, weather?"

**Disruption:**
- "When a crew member calls in sick on the day of operation, what happens in the first 30 minutes?"
- "How do you decide which replacement crew member to assign?"
- "How do you track the downstream effects of that decision on the rest of the week?"

**Decision quality:**
- "How do you know if a scheduling decision was good or bad after the fact?"
- "Do you have a way to compare alternative schedules before committing to one?"
- "If a regulator asked you to justify a specific crew assignment, how would you do that?"

**Facilitator note:** Listen for the words "manual", "spreadsheet", "phone call", "gut feel", "we just know". These indicate Tier 1 or Tier 2. Listen for "system", "rule", "constraint", "report" to indicate Tier 2 or Tier 3. Tier 4 organisations are rare and will use words like "model", "optimise", "scenario", "audit trail".

---

## Segment 3 — Live Demonstration (15–20 minutes)

**Objective:** Make the WOA findings concrete by showing what Tier 3/4 decision-making looks like in practice.

### Demo Flow

**Step 1 — Open the platform (1 min)**
- Navigate to the public URL
- "What you're looking at is a live system running on real infrastructure. The scenario is based on SunAir Regional — a synthetic airline built on a published academic benchmark. The operational parameters are representative of a short-haul regional carrier."

**Step 2 — Show the baseline schedule (2 min)**
- "This is the starting point — a compliant crew schedule. Every constraint is satisfied. I want to show you what happens when reality intervenes."
- Point out the coverage metrics and constraint summary

**Step 3 — Introduce a disruption (3 min)**
- Trigger a sick leave event for a crew member on a high-demand day
- "A crew member has called in sick. The system immediately identifies the downstream impact — which duties are affected, which constraints are now violated, and what the recovery options are."
- Let the attendee observe the impact before showing the recovery

**Step 4 — Run the optimiser (3–5 min)**
- "Rather than a scheduler spending 45 minutes on the phone, the system generates a compliant recovery plan in under 60 seconds. More importantly, it shows you the trade-offs — not just one answer, but the reasoning behind it."
- Walk through the constraint report and recommendations

**Step 5 — Show the decision audit (2 min)**
- "Every decision is recorded. If a regulator, a union representative, or your own management asks why a specific crew member was assigned to a specific duty, you have an answer. That's not possible with a spreadsheet."

**Step 6 — Commercial evidence capture (2 min)**
- "Before we move on, I'd like to record a few baseline numbers from your operation — with your permission. This becomes the foundation of a WOA report I'll send you after the meeting."
- Complete the Step 6 form with their actual numbers (scheduling time, disruption frequency, etc.)

### What to do if the demo fails

If the hosted system is unavailable:
1. Do not apologise excessively — say "Let me switch to the local version" and launch `./start-demo.sh`
2. The local version is functionally identical
3. If both fail, continue the WOA interview and WDX discussion without the live demo — the conversation is more valuable than the software

---

## Segment 4 — WDX Discussion (10 minutes)

**Objective:** Show the path from their current tier to the next tier, and quantify the value of that transition.

**Opening:**
> "Based on what you've described, I'd place your current operation at [Tier X]. That's not a criticism — it's where most organisations of your size are. The question is: what would it mean for your operation to move to [Tier X+1]?"

### Value framing by tier transition

**Tier 1 → 2 (Reactive → Managed):**
- Reduce scheduling time by 40–60%
- Eliminate compliance errors from manual constraint checking
- Create an auditable record of scheduling decisions

**Tier 2 → 3 (Managed → Optimised):**
- Reduce disruption recovery time from hours to minutes
- Quantify the cost of each scheduling decision before committing
- Compare alternative schedules on cost, fairness, and compliance simultaneously

**Tier 3 → 4 (Optimised → Adaptive):**
- Real-time decision support during live operations
- Predictive disruption modelling
- Continuous improvement from operational data

**Questions to ask:**
- "If you could reduce your scheduling cycle from [their number] to [target], what would that free up for your team?"
- "What is the cost of a single crew-related delay to your operation?"
- "If you had to justify your crew assignments to a regulator tomorrow, how confident are you in your current documentation?"

---

## Segment 5 — Pilot Proposal (10 minutes)

**Objective:** Gauge genuine interest in a structured pilot engagement. Do not close a sale — close a next step.

**Framing:**
> "What I'm proposing isn't a software purchase. It's a structured pilot — typically 4–6 weeks — where we run your actual operational data through the system alongside your current process. At the end, you have a WOA report with your baseline metrics, a WDX report showing what improvement looks like for your specific operation, and a clear recommendation on whether a full deployment makes sense. You only proceed if the evidence supports it."

**Pilot structure to describe:**
1. Week 1–2: Data collection and baseline measurement
2. Week 3–4: Parallel operation (current process + UltraCrew)
3. Week 5–6: Comparison, WOA report, WDX report, recommendation

**Questions to gauge interest:**
- "Does that kind of structured evaluation make sense for your organisation?"
- "Who else would need to be involved in a decision like this?"
- "What would need to be true for you to say yes to a pilot?"

---

## Segment 6 — Close and Next Steps (5 minutes)

**Objective:** Leave with one concrete agreed action.

**Always agree on one of:**
- A follow-up meeting with additional stakeholders
- A request for their operational data to prepare a more specific WOA assessment
- A written summary of the session (send within 48 hours)
- A pilot proposal document (send within 1 week)

**Closing statement:**
> "I'll send you a summary of what we discussed today, including the baseline numbers we captured, within 48 hours. The most useful thing you can do before then is think about one specific operational scenario — a disruption, a scheduling challenge, a compliance question — that you'd want to see the system handle with your own data. That will make the next conversation much more concrete."

---

## Questions NOT to Answer Immediately

These questions require follow-up rather than an on-the-spot answer. Say: *"That's an important question — let me take that away and give you a proper answer rather than speculating."*

- "How does this integrate with [specific HR/ERP/rostering system]?"
- "What does implementation look like for an organisation our size?"
- "What are your pricing / licensing terms?"
- "Can you handle [specific regulatory framework — EASA, FAA, DGCA]?"
- "What's your data security certification?"
- "Do you have other airline customers?"
- "What's your roadmap for [specific feature]?"

---

## Feedback Form

Capture these data points at the end of every session (or in the Step 6 form during the demo):

| Field | Notes |
|-------|-------|
| Organisation name | |
| Attendee names and roles | |
| Fleet size (aircraft) | |
| Crew size (total FTE) | |
| Current scheduling tool | Manual / Spreadsheet / Legacy system / Modern system |
| Scheduling cycle time (hours) | |
| Disruption frequency (events/month) | |
| Average disruption recovery time (hours) | |
| WOA maturity tier assessed | 1 / 2 / 3 / 4 |
| Primary pain point identified | |
| Interest level | Cold / Warm / Hot |
| Agreed next step | |
| Follow-up deadline | |

---

## Post-Meeting Actions (within 48 hours)

1. Send meeting summary with baseline numbers captured
2. Update the pilot_sessions record in Supabase with the commercial evidence fields
3. Record the WOA tier assessment and primary pain point in your CRM or tracking sheet
4. If interest level is Warm or Hot: prepare a pilot proposal document
5. If interest level is Cold: send a brief follow-up in 4 weeks with one relevant case study or benchmark result

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-07-28 | Initial release — pre-first customer demo |

---

*This document is internal. Do not share with prospects.*
