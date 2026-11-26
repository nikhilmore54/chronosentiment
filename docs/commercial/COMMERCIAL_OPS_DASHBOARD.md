# Coralys Commercial Operations Dashboard

**Version:** 1.0
**Update cadence:** Weekly (every Monday before standup)
**Owner:** Commercial Lead
**Purpose:** Single cockpit for running the Coralys commercial operation — pipeline, product learning, engineering health, and delivery commitments on one page.

---

## How to Use This Document

Update every section weekly. The dashboard is not a reporting tool — it is a decision tool. If a number is red, something needs to change this week. If a section has no entries, that is also a signal.

The four quadrants answer four questions:

| Quadrant | Question |
|----------|---------|
| Revenue Pipeline | Are we converting engagements into revenue? |
| Product Learning | What are customers telling us to build? |
| Engineering | Is the platform reliable enough to sell? |
| Delivery | What commitments are due this week? |

---

## Quadrant 1 — Revenue Pipeline

*Updated after every engagement. Source of truth for commercial conversion.*

### Funnel

| Stage | UltraCrew (Airline) | UltraRoster (Healthcare) | Total |
|-------|--------------------:|-------------------------:|------:|
| Initial contact | | | |
| Discovery completed | | | |
| WOA completed | | | |
| WDX demonstrated | | | |
| Assessment proposal sent | | | |
| Assessment won | | | |
| Pilot proposal sent | | | |
| Pilot won | | | |
| Subscription signed | | | |

**Conversion to watch:** WOA completed → Assessment won. A drop here is a commercial offer problem, not a product problem.

### Active Opportunities

| Organisation | Domain | Solution | Stage | Next action | Owner | Due |
|-------------|--------|----------|-------|-------------|-------|-----|
| | | | | | | |

### Revenue

| Item | Target | Actual | Notes |
|------|-------:|-------:|-------|
| Assessment revenue (cumulative) | | | |
| Pilot revenue (cumulative) | | | |
| Subscription ARR | | | |
| Average deal cycle (days) | | | |

### Conversion Rates

| Conversion | Target | Actual |
|-----------|-------:|-------:|
| Discovery → WOA | >80% | |
| WOA → WDX | >90% | |
| WDX → Assessment proposal | >60% | |
| Assessment proposal → Assessment won | >50% | |
| Assessment won → Pilot | >70% | |
| Pilot → Subscription | >60% | |

---

## Quadrant 2 — Product Learning

*Sourced from the Commercial Evidence Register. Updated after every session.*

### Platform Evidence (applies to all Solution Engines)

| Observation | Sessions | Severity | Status |
|-------------|:--------:|----------|--------|
| | | | |

**Platform evidence threshold:** 2+ sessions → v1.1 candidate. Add to platform backlog.

### Solution Evidence — UltraCrew (Airline)

| Observation | Sessions | Severity | Status |
|-------------|:--------:|----------|--------|
| | | | |

### Solution Evidence — UltraRoster (Healthcare)

| Observation | Sessions | Severity | Status |
|-------------|:--------:|----------|--------|
| | | | |

### Cross-Domain Patterns

*Observations that appear in both Airline and Healthcare engagements. These are the most valuable signals — they indicate platform-level needs.*

| Observation | Airline sessions | Healthcare sessions | Action |
|-------------|:---------------:|:-------------------:|--------|
| | | | |

### Objection Tracker

*Objections raised across all sessions. Recurring objections require a prepared response or a product change.*

| Objection | Times raised | Domains | Addressed in session? | Resolution |
|-----------|:-----------:|---------|----------------------|-----------|
| | | | | |

### Customer Confidence Trend

*Average confidence ratings across all sessions. A declining trend is an early warning signal.*

| Dimension | Session 1–5 avg | Session 6–10 avg | Trend |
|-----------|:--------------:|:----------------:|-------|
| Operational credibility | | | |
| Decision transparency | | | |
| Overall confidence | | | |

---

## Quadrant 3 — Engineering

*Updated by engineering lead. Reflects platform and demo reliability.*

### Demo Reliability

| Metric | Target | Actual | Notes |
|--------|-------:|-------:|-------|
| Demo completion rate | >95% | | |
| Backend uptime during demos | >99% | | |
| UltraCrew frontend issues (last 30 days) | 0 | | |
| UltraRoster frontend issues (last 30 days) | 0 | | |
| start-demo.sh failures (last 30 days) | 0 | | |

### Open Issues

| Issue | Scope | Severity | Assigned | ETA |
|-------|-------|----------|----------|-----|
| | | | | |

**Scope key:** Platform = affects all Solution Engines. Solution = affects one engine only.

### Platform Backlog (v1.1 candidates)

*Items that have appeared in 2+ sessions and are confirmed for the next release.*

| Item | Evidence sessions | Scope | Priority | Status |
|------|:-----------------:|-------|----------|--------|
| | | | | |

### Solution Backlog — UltraCrew

| Item | Evidence sessions | Priority | Status |
|------|:-----------------:|----------|--------|
| | | | |

### Solution Backlog — UltraRoster

| Item | Evidence sessions | Priority | Status |
|------|:-----------------:|----------|--------|
| | | | |

### Security & Compliance

| Item | Status | Last reviewed |
|------|--------|--------------|
| cargo audit (P0 vulnerabilities) | | |
| npm audit — UltraCrew | | |
| npm audit — UltraRoster | | |
| OWASP Top 10 review | | |
| Rate limiting active | | |
| Supabase RLS policies reviewed | | |

---

## Quadrant 4 — Delivery

*Updated weekly. Commitments made to customers and internal deadlines.*

### Upcoming Demonstrations

| Date | Organisation | Domain | Solution | Facilitator | Prep status |
|------|-------------|--------|----------|-------------|-------------|
| | | | | | |

**Prep checklist before every demo:**
- [ ] Backend starts cleanly (`./start-demo.sh --airline-only` or `--healthcare-only`)
- [ ] Demo dataset loaded and verified
- [ ] Scenario scripts reviewed
- [ ] Objection responses prepared (check Objection Tracker above)
- [ ] Evidence Register session template copied and ready

### Outstanding Follow-Ups

| Organisation | Commitment | Owner | Due | Status |
|-------------|-----------|-------|-----|--------|
| | | | | |

**Rule:** No follow-up should be more than 5 business days old without a status update.

### Proposal Deadlines

| Organisation | Proposal type | Owner | Due | Status |
|-------------|--------------|-------|-----|--------|
| | | | | |

### This Week's Priorities

*Three things that must happen this week to move the commercial operation forward.*

1.
2.
3.

---

## Weekly Review Checklist

Run through this before closing the dashboard each week.

**Pipeline**
- [ ] All active opportunities have a next action and owner
- [ ] No opportunity has been in the same stage for more than 3 weeks without a note
- [ ] Conversion rates reviewed — any stage with <50% conversion flagged

**Product Learning**
- [ ] All sessions from the past week have been logged in the Evidence Register
- [ ] Any observation appearing in 2+ sessions has been added to the relevant backlog
- [ ] Cross-domain patterns section updated

**Engineering**
- [ ] Demo reliability metrics updated
- [ ] Any P0 security issues have a resolution date
- [ ] Platform and solution backlogs reflect current evidence

**Delivery**
- [ ] All follow-ups from the past week are resolved or have a new due date
- [ ] Next week's demonstrations have prep status confirmed
- [ ] This week's three priorities are set

---

## Architecture Reference

```
                    Coralys Platform
                          │
          ┌───────────────┼────────────────┐
          │               │                │
       WOA Framework   WDX Framework   Evidence Register
                          │
               Domain Package Template
          ┌───────────────┼────────────────┐
          │               │                │
       UltraCrew     UltraRoster     UltraShift (future)
        Airline       Healthcare     Manufacturing
```

WOA, WDX, and the Evidence Register sit above individual solutions. Evidence tagged Platform feeds the platform backlog. Evidence tagged Solution feeds the relevant Domain Package backlog.

---

*Dashboard version: 1.0*
*Created: 2026-07-28*
*Maintained by: Coralys Commercial Lead*