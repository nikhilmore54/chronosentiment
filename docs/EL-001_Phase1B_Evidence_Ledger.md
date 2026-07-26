# EL-001 — Phase 1B Evidence Ledger
## ChronoSentiment Commercial Validation | v1.0

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | **Operational v1.0** |
| Version | 1.0 |
| Date | 2026-07-26 |
| Owner | Commercial / Product |
| Review Trigger | After every 5 evidence records (rolling synthesis); at Phase 1B completion (final synthesis and go/no-go decision) |

**Relationship to other documents:**
- Executes: CV-001 Commercial Validation Playbook (interview protocol, kill criteria)
- Tests: CS-R-015 Phase 1B hypotheses H1–H7 (Section 7)
- Feeds into: MVP go/no-go decision (CS-R-015 Section 9 success criteria)

---

## Purpose

This ledger is the primary evidence record for Phase 1B. Every piece of primary evidence — customer interviews, expert interviews, public observations, product demonstrations, and prototype evaluations — is recorded here. Hypothesis confidence levels are updated after every 5 evidence records. The final synthesis produces the go/no-go decision for MVP development.

The ledger creates a continuous, traceable chain:

```
CS-R-001 through CS-R-015 (secondary research)
        ↓
Investment Thesis — H1–H7 hypotheses defined
        ↓
CV-001 — Evidence acquisition protocol defined
        ↓
EL-001 — Primary evidence recorded here (all types)
        ↓
MVP go/no-go decision — grounded in recorded evidence
```

**Evidence types supported:**

| Prefix | Type | Description | Evidential weight |
|--------|------|-------------|------------------|
| INT | Customer interview | Direct conversation with a target firm (CIO, PM, compliance) | Highest — direct purchase intent signal |
| EXP | Expert interview | Consultant, former CIO, compliance adviser, industry analyst, recruiter | High — cross-firm pattern visibility |
| OBS | Public observation | Conference panel, webinar, podcast, recorded presentation by target-firm personnel | Medium — first-hand statement, not interactive |
| DEM | Product demonstration | Structured feedback session on proof-of-concept or prototype | High — behavioural signal, not just stated preference |
| POC | Prototype evaluation | Hands-on use of the product with real decisions from the firm | Highest — strongest behavioural signal available |

**Evidential standard:** Every confidence update must cite specific evidence IDs (e.g. INT-003, EXP-001). No confidence level may be upgraded on the basis of general impression. Contradictory evidence must be recorded alongside supporting evidence. Evidence type affects weight — a single POC-001 may carry more weight than three OBS records.

---

## Hypothesis Register

The seven hypotheses from CS-R-015 Section 7. Confidence scale: A (high, multiple independent sources) → B (moderate, consistent pattern) → C (preliminary, limited sources) → D (unvalidated, secondary research only) → X (contradicted by primary evidence).

| ID | Hypothesis | Starting Confidence | Current Confidence | Last Updated | Evidence IDs (all types) |
|----|-----------|--------------------|--------------------|-------------|--------------------------|
| H1 | Mid-size investment firms ($500M–$10B AUM) are actively experiencing governance problems created by AI adoption and are seeking solutions | D | D | — | — |
| H2 | Target customers would pay US$30,000–US$120,000/yr for a platform that solves the decision governance problem | D | D | — | — |
| H3 | Target customers recognise "decision governance" or an equivalent framing as a meaningful category | D | D | — | — |
| H4 | Regulatory requirements are creating active purchasing urgency for decision governance tools | C | C | — | — |
| H5 | Customers value the integrated capability over point solutions addressing individual components | D | D | — | — |
| H6 | The primary buyer is the CIO or Head of Investment, not the compliance officer or CTO | D | D | — | — |
| H7 | One use case (LP reporting, regulatory audit, investment committee governance, or post-mortem analysis) drives disproportionate urgency and willingness to pay | D | D | — | — |

---

## Evidence Register

One row per evidence record. Evidence IDs are sequential within each type: INT-001, INT-002 … EXP-001, EXP-002 … OBS-001 … DEM-001 … POC-001.

| ID | Date | Firm | AUM | Segment | Interviewee | Title | Problem Resonance (1–5) | AI Tool Usage | Solution Reaction | Category Language Used | WTP Signal | Design Partner Interest | Referrals | Key Quote |
|----|------|------|-----|---------|-------------|-------|------------------------|---------------|-------------------|----------------------|-----------|------------------------|-----------|-----------|
| INT-001 | — | — | — | — | — | — | — | — | — | — | — | — | — | — |

*Add rows as interviews are completed. See CV-001 Section 7 for minimum field definitions.*

---

## Evidence Detail Records

For each evidence record, create a detail record below using the appropriate template. The summary row in the Evidence Register is for scanning; the detail record is for analysis.

---

### Interview Detail Template

Copy this block for each interview completed.

```
---

#### INT-XXX — [Firm Name] | [Date]

**Firm:** [Name] | AUM: [£/US$] | Team size: [N] | Strategy: [e.g. long/short equity]
**Interviewee:** [Name], [Title]
**Segment:** Tier [1/2/3/4] per CV-001

**Problem resonance:** [1–5] — [brief rationale]

**AI tool usage:**
- Tools: [list]
- Workflows: [describe]
- Governance: [how they currently track AI usage, if at all]

**Problem examples cited (verbatim or close paraphrase):**
> "[Quote 1]"
> "[Quote 2]"

**Solution reaction:** [Immediate recognition / Polite interest / Confusion / Negative]
- Notes: [what specifically they said]

**Category language (their words, not ours):**
> "[Exact phrase they used to describe the problem or product]"

**Commercial signals:**
- Willingness to pay: [Named figure / Range / Refused / Not asked]
- Budget authority: [Self / Named person / Unknown]
- Design partner interest: [Strong / Weak / None]

**Hypotheses touched:**
- H1: [Supported / Challenged / Neutral] — [evidence]
- H2: [Supported / Challenged / Neutral] — [evidence]
- H3: [Supported / Challenged / Neutral] — [evidence]
- H4: [Supported / Challenged / Neutral] — [evidence]
- H5: [Supported / Challenged / Neutral] — [evidence]
- H6: [Supported / Challenged / Neutral] — [evidence]
- H7: [Supported / Challenged / Neutral] — [evidence]

**Referrals:** [Names and firms, or none]

**False positive indicators:** [List any, or none]

**Contradictory evidence:** [Anything that challenges the investment thesis]

**Follow-up actions:**
- [ ] [Action 1]
- [ ] [Action 2]

---
```

---

### Expert Interview Detail Template (EXP)

Copy this block for each expert interview completed.

```
---

#### EXP-XXX — [Expert Name / Organisation] | [Date]

**Expert:** [Name], [Title / Role]
**Organisation:** [Firm / Independent]
**Expertise:** [e.g. Asset management technology, Compliance, Former CIO at X]
**Firms they have visibility across:** [N firms / describe]

**Key questions asked:**
1. [Question]
2. [Question]

**Key findings:**
> "[Quote or close paraphrase]"
> "[Quote or close paraphrase]"

**Cross-firm patterns observed:**
- [Pattern 1]
- [Pattern 2]

**Hypotheses touched:**
- H1: [Supported / Challenged / Neutral] — [evidence]
- H2: [Supported / Challenged / Neutral] — [evidence]
- H3: [Supported / Challenged / Neutral] — [evidence]
- H4: [Supported / Challenged / Neutral] — [evidence]
- H5: [Supported / Challenged / Neutral] — [evidence]
- H6: [Supported / Challenged / Neutral] — [evidence]
- H7: [Supported / Challenged / Neutral] — [evidence]

**Target firm referrals:** [Names and firms, or none]

**Contradictory evidence:** [Anything that challenges the investment thesis]

**Follow-up actions:**
- [ ] [Action 1]

---
```

---

### Public Observation Detail Template (OBS)

Copy this block for each public observation recorded (conference panel, webinar, podcast, recorded presentation).

```
---

#### OBS-XXX — [Speaker Name / Event] | [Date]

**Source type:** [Conference panel / Webinar / Podcast / Recorded presentation / Other]
**Event / Publication:** [Name]
**Speaker:** [Name], [Title], [Firm]
**URL / Reference:** [Link or citation]

**Relevant statements (verbatim or close paraphrase):**
> "[Quote 1]"
> "[Quote 2]"

**Context:** [What question or topic prompted the statement?]

**Evidential value:** [What does this tell us about the hypothesis?]

**Hypotheses touched:**
- H1: [Supported / Challenged / Neutral] — [evidence]
- H2: [Supported / Challenged / Neutral] — [evidence]
- H3: [Supported / Challenged / Neutral] — [evidence]
- H4: [Supported / Challenged / Neutral] — [evidence]
- H5: [Supported / Challenged / Neutral] — [evidence]
- H6: [Supported / Challenged / Neutral] — [evidence]
- H7: [Supported / Challenged / Neutral] — [evidence]

**Limitations:** [Why this evidence is weaker than an interview — e.g. no interaction, public framing, unknown context]

**Contradictory evidence:** [Anything that challenges the investment thesis]

---
```

---

### Product Demonstration Detail Template (DEM)

Copy this block for each structured product demonstration or feedback session.

```
---

#### DEM-XXX — [Firm Name] | [Date]

**Firm:** [Name] | AUM: [£/US$] | Team size: [N]
**Participant:** [Name], [Title]
**Demo format:** [Live demo / Recorded walkthrough / Interactive prototype]
**Duration:** [N minutes]

**Features demonstrated:**
- [Feature 1]
- [Feature 2]

**Participant reactions (verbatim or close paraphrase):**
> "[Quote 1]"
> "[Quote 2]"

**Positive signals:**
- [Signal 1]
- [Signal 2]

**Friction points / objections:**
- [Objection 1]
- [Objection 2]

**Commercial signals:**
- Willingness to pay: [Named figure / Range / Refused / Not asked]
- Design partner interest: [Strong / Weak / None]

**Hypotheses touched:**
- H1: [Supported / Challenged / Neutral] — [evidence]
- H2: [Supported / Challenged / Neutral] — [evidence]
- H3: [Supported / Challenged / Neutral] — [evidence]
- H4: [Supported / Challenged / Neutral] — [evidence]
- H5: [Supported / Challenged / Neutral] — [evidence]
- H6: [Supported / Challenged / Neutral] — [evidence]
- H7: [Supported / Challenged / Neutral] — [evidence]

**Contradictory evidence:** [Anything that challenges the investment thesis]

**Follow-up actions:**
- [ ] [Action 1]

---
```

---

### Prototype Evaluation Detail Template (POC)

Copy this block for each hands-on prototype evaluation (firm uses the product with real decisions).

```
---

#### POC-XXX — [Firm Name] | [Date range]

**Firm:** [Name] | AUM: [£/US$] | Team size: [N]
**Primary user:** [Name], [Title]
**Evaluation period:** [Start date] – [End date]
**Decisions evaluated:** [N decisions recorded in the prototype]

**Usage observations:**
- Decisions captured: [N]
- Features used: [list]
- Features not used: [list]
- Workflow integration: [describe how they used it in their actual workflow]

**User feedback (verbatim or close paraphrase):**
> "[Quote 1]"
> "[Quote 2]"

**Behavioural signals (what they did, not just what they said):**
- [Signal 1 — e.g. "Used the product unprompted on 3 occasions without being asked"]
- [Signal 2]

**Commercial signals:**
- Willingness to pay: [Named figure / Range / Refused / Not asked]
- Design partner conversion: [Agreed / Declined / Pending]
- Reference willingness: [Agreed / Declined / Pending]

**Hypotheses touched:**
- H1: [Supported / Challenged / Neutral] — [evidence]
- H2: [Supported / Challenged / Neutral] — [evidence]
- H3: [Supported / Challenged / Neutral] — [evidence]
- H4: [Supported / Challenged / Neutral] — [evidence]
- H5: [Supported / Challenged / Neutral] — [evidence]
- H6: [Supported / Challenged / Neutral] — [evidence]
- H7: [Supported / Challenged / Neutral] — [evidence]

**Contradictory evidence:** [Anything that challenges the investment thesis]

**Follow-up actions:**
- [ ] [Action 1]

---
```

---

## Rolling Synthesis

After every 5 evidence records (of any type), complete a synthesis block. Do not wait until the end of Phase 1B to synthesise — update confidence levels continuously.

---

### Synthesis Template

Copy this block after every 5 interviews.

```
---

#### Synthesis after INT-[N] (Interviews [X]–[Y])

**Date:** [Date]
**Interviews completed:** [N total]
**Interviews remaining:** [30 - N]

**Hypothesis confidence updates:**

| Hypothesis | Previous | Current | Direction | Evidence basis |
|-----------|---------|---------|-----------|----------------|
| H1 | [prev] | [curr] | [↑/↓/→] | [INT-XXX, INT-XXX: brief rationale] |
| H2 | [prev] | [curr] | [↑/↓/→] | [INT-XXX, INT-XXX: brief rationale] |
| H3 | [prev] | [curr] | [↑/↓/→] | [INT-XXX, INT-XXX: brief rationale] |
| H4 | [prev] | [curr] | [↑/↓/→] | [INT-XXX, INT-XXX: brief rationale] |
| H5 | [prev] | [curr] | [↑/↓/→] | [INT-XXX, INT-XXX: brief rationale] |
| H6 | [prev] | [curr] | [↑/↓/→] | [INT-XXX, INT-XXX: brief rationale] |
| H7 | [prev] | [curr] | [↑/↓/→] | [INT-XXX, INT-XXX: brief rationale] |

**Emerging patterns:**
- [Pattern 1 — cite interview IDs]
- [Pattern 2 — cite interview IDs]

**Contradictory evidence:**
- [Contradiction 1 — cite interview IDs]
- [Contradiction 2 — cite interview IDs]

**Category language emerging:**
- Most common phrase: "[phrase]" (N interviews)
- Alternative phrases: "[phrase]" (N), "[phrase]" (N)

**Segment observations:**
- Tier 1 (Independent AM): [resonance pattern]
- Tier 2 (Family Office): [resonance pattern]
- Tier 3 (Hedge Fund): [resonance pattern]

**Kill criteria status:**
- Stop criteria: [Not triggered / Approaching / Triggered — specify]
- Reposition criteria: [Not triggered / Approaching / Triggered — specify]

**Protocol adjustments (if any):**
- [Any changes to interview approach based on evidence so far]
- [Rationale for change]

---
```

---

## Final Synthesis and Go/No-Go Decision

Complete this section at the end of Phase 1B (after all interviews, proof-of-concept, and design partner conversations).

---

### Final Synthesis Template

```
---

## Final Synthesis — Phase 1B

**Date:** [Date]
**Total interviews completed:** [N]
**Firms represented:** [N]
**Segments covered:** Tier 1: [N], Tier 2: [N], Tier 3: [N], Tier 4: [N]

---

### Hypothesis final confidence levels

| Hypothesis | Starting | Final | Evidence basis |
|-----------|---------|-------|----------------|
| H1 | D | [final] | [summary] |
| H2 | D | [final] | [summary] |
| H3 | D | [final] | [summary] |
| H4 | C | [final] | [summary] |
| H5 | D | [final] | [summary] |
| H6 | D | [final] | [summary] |
| H7 | D | [final] | [summary] |

---

### Minimum criteria assessment (CS-R-015 Section 9)

| Criterion | Target | Actual | Met? |
|-----------|--------|--------|------|
| Firms confirming problem is real, active, not currently solved | ≥ 5 of 20+ | [N] | [Y/N] |
| Firms expressing WTP ≥ US$30,000/yr | ≥ 3 | [N] | [Y/N] |
| Design partnerships agreed | ≥ 1 | [N] | [Y/N] |
| PoC demonstrates feasibility within 6-month timeline | Yes | [Y/N] | [Y/N] |

---

### Category language finding

The language customers use to describe the problem (unprompted):
- Most common: "[phrase]" ([N] interviews)
- Second: "[phrase]" ([N] interviews)
- Third: "[phrase]" ([N] interviews)

Recommended product category name: [recommendation with rationale]

---

### Beachhead use case finding

Use case that generated strongest urgency and WTP signal:
- [Use case name]: [N] interviews cited as primary; [N] cited as secondary
- Rationale: [why this use case leads]

---

### Primary buyer finding

Confirmed buying committee structure at Tier 1 firms:
- Initiator: [title]
- Primary buyer: [title]
- Approver: [title]
- Blocker risk: [title and condition]

---

### Contradictory evidence summary

Evidence that challenges the investment thesis:
1. [Contradiction 1 — cite interviews — severity: High/Medium/Low]
2. [Contradiction 2 — cite interviews — severity: High/Medium/Low]

---

### Go/No-Go Decision

**Decision:** [PROCEED TO MVP / REPOSITION / TERMINATE]

**Rationale:** [2–3 sentences grounded in the evidence above]

**If PROCEED:** Design partner(s) confirmed: [firm names]. MVP scope: [any adjustments from Product Blueprint v1.0 based on Phase 1B findings].

**If REPOSITION:** What changes: [segment / framing / use case / pricing]. What stays the same: [product direction]. Timeline to re-validate: [N weeks].

**If TERMINATE:** Primary reason: [cite specific evidence]. Options considered: [pivot directions evaluated and why rejected].

**Decision recorded by:** [Name]
**Date:** [Date]

---
```

---

## Evidence Quality Standards

These standards apply to all entries in this ledger. They are consistent with the evidential discipline established in the CS-R research series.

**Confidence levels:**

| Level | Definition |
|-------|-----------|
| A | High confidence — multiple independent sources, consistent pattern, no significant contradictions |
| B | Moderate confidence — consistent pattern across several sources, minor contradictions noted |
| C | Preliminary — limited sources, pattern emerging but not yet robust |
| D | Unvalidated — hypothesis stated but no primary evidence yet |
| X | Contradicted — primary evidence actively challenges the hypothesis |

**Upgrade rules:**
- D → C: At least 3 interviews independently support the hypothesis with specific examples
- C → B: At least 7 interviews support; pattern is consistent; contradictions are minor or explained
- B → A: At least 12 interviews support; pattern is robust; contradictions are documented and addressed
- Any level → X: 3 or more interviews provide specific evidence that directly contradicts the hypothesis

**Downgrade rules:**
- Any level may be downgraded immediately if contradictory evidence is strong and specific
- A downgrade must cite the specific interview IDs that triggered it

**False positive discipline:**
- Polite interest does not count as problem confirmation
- "We'd probably use this" does not count as WTP signal
- Enthusiasm without a named buyer does not count as design partner interest
- See CV-001 Section 3 for full false positive criteria

---

*EL-001 Phase 1B Evidence Ledger v1.0 | July 2026 | ChronoSentiment Commercial Validation*
*Operational document — updated continuously throughout Phase 1B.*
*Executes CV-001 Commercial Validation Playbook. Tests CS-R-015 hypotheses H1–H7.*
*Review trigger: Every 5 interviews (rolling synthesis); Phase 1B completion (final synthesis and go/no-go).*