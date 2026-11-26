# COM-001 — Commercial Intelligence Database
## ChronoSentiment Phase 1BA | v1.0

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | **Operational v1.0** |
| Version | 1.0 |
| Date | 2026-07-26 |
| Owner | Commercial |
| Review Trigger | Continuous — updated as intelligence is gathered and relationships progress |

**Relationship to other documents:**
- Feeds into: CV-001 Commercial Validation Playbook (outreach prioritisation)
- Links to: EL-001 Phase 1B Evidence Ledger (evidence records cite firm IDs from this database)
- Informed by: CS-R-001 (market segments), CS-R-003 (customer problems), CV-001 Phase 1BA (intelligence sources)

---

## Purpose

This database is the commercial knowledge base for Phase 1BA and Phase 1B. Every target firm has a dossier. Every contact has a relationship stage. Every piece of evidence in EL-001 links back to a firm record here.

The database creates a complete commercial operating system:

```
CS-R series (secondary research)
        ↓
COM-001 (firm intelligence + relationship tracking)
        ↓
CV-001 (validation protocol)
        ↓
EL-001 (evidence records)
        ↓
MVP go/no-go decision
```

**Firm IDs:** Each firm is assigned a sequential ID: FIRM-001, FIRM-002, etc. EL-001 evidence records reference these IDs (e.g. "INT-003 — FIRM-012").

---

## Relationship Stage Register

Track every contact's relationship stage. This measures whether you are building a network, not just collecting interview notes.

| Stage | Label | Definition |
|-------|-------|-----------|
| 0 | **Unknown** | Firm identified; no contact identified |
| 1 | **Identified** | Specific contact identified (name, title, LinkedIn) |
| 2 | **Following** | Following on LinkedIn; monitoring public activity |
| 3 | **Engaged** | Commented on their posts; liked their content; visible to them |
| 4 | **Connected** | LinkedIn connection accepted or email address obtained |
| 5 | **Introduced** | Warm introduction obtained from a mutual contact |
| 6 | **First contact** | First email or message sent; awaiting response |
| 7 | **Responded** | They have responded; conversation initiated |
| 8 | **First call** | First conversation completed (INT or EXP record created) |
| 9 | **Second call** | Follow-up conversation completed |
| 10 | **Demo** | Product demonstration completed (DEM record created) |
| 11 | **Champion** | Internal advocate identified; actively helping progress |
| 12 | **Design partner** | Formal design partner agreement in place (POC record created) |

**Target distribution at Phase 1B completion:**
- Stages 0–3: 100–150 firms (intelligence gathered, not yet contacted)
- Stages 4–7: 30–50 firms (contact initiated)
- Stages 8–9: 20–30 firms (conversations completed)
- Stages 10–11: 5–10 firms (demonstrations completed)
- Stage 12: 1–3 firms (design partners)

---

## Firm Summary Register

One row per firm. Add rows as firms are identified. Update relationship stage and evidence links as intelligence is gathered.

| Firm ID | Firm Name | AUM | Geography | Segment (Tier) | Primary Contact | Relationship Stage | AI Maturity (1–5) | Regulatory Exposure | Design Partner Probability | Evidence IDs | Next Action |
|---------|-----------|-----|-----------|---------------|----------------|-------------------|------------------|--------------------|-----------------------------|-------------|------------|
| FIRM-001 | — | — | — | — | — | 0 | — | — | — | — | — |

*Add rows as firms are identified. Target: 100–200 firms before outreach begins.*

---

## Firm Dossier Template

For each firm, create a full dossier below. The summary row in the Firm Summary Register is for scanning; the dossier is for preparation and analysis.

---

### Firm Dossier Template

Copy this block for each firm.

```
---

#### FIRM-XXX — [Firm Name]

**Firm profile:**
- Full name: [Legal name]
- AUM: [£/US$]
- Geography: [HQ city, country]
- Investment strategy: [e.g. long/short equity, multi-asset, fixed income]
- Team size: [N investment professionals]
- Founded: [Year]
- Ownership: [Independent / Part of group / Family-owned]

**Segment:** Tier [1/2/3/4] per CV-001 Section 1

**Primary contact:**
- Name: [Name]
- Title: [e.g. CIO, Head of Investment]
- LinkedIn: [URL]
- Email: [if known]
- Background: [Brief — previous firms, education, public profile]

**Secondary contacts:**
- [Name], [Title] — [relevance]
- [Name], [Title] — [relevance]

**AI adoption signals:**
- Job postings mentioning AI: [Y/N — describe]
- Public statements on AI: [Y/N — cite OBS record if applicable]
- Known AI tools in use: [list or unknown]
- AI maturity score (1–5): [1=no AI, 3=experimenting, 5=deeply integrated]

**Technology stack (where public):**
- Data vendors: [Bloomberg / FactSet / Refinitiv / other / unknown]
- Research tools: [AlphaSense / Sentieo / other / unknown]
- Portfolio management: [Aladdin / SimCorp / other / unknown]
- Other relevant tools: [list or unknown]

**Regulatory exposure:**
- FCA registration: [Y/N / reference]
- EU AI Act exposure: [High / Medium / Low / Unknown]
- SEC registration: [Y/N / reference]
- Recent regulatory events: [describe or none known]

**Hiring activity:**
- Recent relevant hires: [describe or none known]
- Open roles: [describe or none known]
- Signals: [growth / contraction / capability gap / none]

**Public commentary:**
- Conference presentations: [cite OBS records or none known]
- Podcast appearances: [cite OBS records or none known]
- LinkedIn activity: [describe or none known]
- Press mentions: [describe or none known]

**Warm introduction paths:**
- Via: [Name at FIRM-XXX / EXP-XXX / mutual connection]
- Via: [Name at FIRM-XXX / EXP-XXX / mutual connection]
- Cold only: [Y/N]

**Intelligence assessment:**
- Estimated urgency (1–5): [1=low, 5=high] — [rationale] — Confidence: [A/B/C/D]
- Estimated WTP: [£/US$ range] — [rationale] — Confidence: [A/B/C/D]
- Design partner probability: [High / Medium / Low] — [rationale] — Confidence: [A/B/C/D]
- Best outreach angle: [specific problem or observation to reference]

**Intelligence confidence key:**
- A — Confirmed directly by the firm (INT or POC record)
- B — Multiple independent public sources (2+ OBS or EXP records)
- C — Single credible source (1 OBS or EXP record)
- D — Inference from public information (no direct source)

**Evidence links:**
- OBS: [OBS-XXX — brief description]
- EXP: [EXP-XXX — brief description]
- INT: [INT-XXX — brief description]
- DEM: [DEM-XXX — brief description]
- POC: [POC-XXX — brief description]

**Relationship history:**
- [Date]: [Action taken — e.g. "Connected on LinkedIn", "Sent intro email", "First call completed"]
- [Date]: [Action taken]

**Current relationship stage:** [0–12 per stage register]

**Next action:**
- [ ] [Specific action with target date]

---
```

---

## Contact Ecosystem Register

ChronoSentiment has multiple contact types, not just end customers. Each type teaches something different and provides different value.

| Contact Type | What they teach | What they provide | Priority |
|-------------|----------------|-------------------|---------|
| **Current CIO / Head of Investment** | Problem urgency, WTP, buying process | Purchase intent, design partner potential | Highest |
| **Current Portfolio Manager** | Daily workflow, pain points, feature priorities | User feedback, internal champion potential | High |
| **Current Compliance Officer** | Regulatory urgency, compliance workflow | Buyer validation, regulatory intelligence | High |
| **Former CIO** | Cross-firm patterns, historical context | Introductions, expert evidence (EXP) | High |
| **Asset management consultant** | Cross-firm patterns, vendor landscape | Introductions, expert evidence (EXP) | High |
| **Technology implementation partner** | Technology stack, integration complexity | Introductions, technical validation | Medium |
| **Compliance adviser** | Regulatory requirements, compliance workflow | Regulatory intelligence, introductions | Medium |
| **Industry analyst** | Market sizing, competitive landscape | Market intelligence, credibility | Medium |
| **Recruiter (investment management)** | Hiring trends, capability gaps, firm intelligence | Firm intelligence, introductions | Medium |
| **Conference organiser** | Access to CIOs at scale | Speaking opportunities, attendee lists | Medium |
| **LP adviser / placement agent** | LP perspective on governance, reporting | LP intelligence, introductions to GPs | Low–Medium |
| **AI governance specialist** | Regulatory framing, AI governance landscape | Regulatory intelligence, credibility | Low–Medium |
| **Academic (finance / AI)** | Research evidence, credibility | Research validation, introductions | Low |

### Contact Register

One row per contact. Contacts may be associated with a firm (FIRM-XXX) or independent.

| Contact ID | Name | Title | Organisation | Firm ID | Contact Type | Relationship Stage | Evidence IDs | Next Action |
|-----------|------|-------|-------------|---------|-------------|-------------------|-------------|------------|
| CON-001 | — | — | — | — | — | 0 | — | — |

---

## Intelligence Synthesis

After every 20 firm dossiers completed, record a synthesis observation. This tracks whether the intelligence is revealing patterns that should update the Phase 1B approach.

---

### Intelligence Synthesis Template

```
---

#### Intelligence Synthesis after FIRM-[N] ([Date])

**Firms profiled:** [N total]
**Firms at Stage 4+ (contact initiated):** [N]
**Firms at Stage 8+ (conversations completed):** [N]

**Patterns emerging from intelligence:**
- [Pattern 1 — e.g. "AI adoption is concentrated in firms with AUM > £1B"]
- [Pattern 2]

**Segment observations:**
- Tier 1 (Independent AM): [AI maturity, regulatory exposure, outreach response rate]
- Tier 2 (Family Office): [AI maturity, regulatory exposure, outreach response rate]
- Tier 3 (Hedge Fund): [AI maturity, regulatory exposure, outreach response rate]

**Best outreach angles identified:**
- [Angle 1 — e.g. "Reference to FCA AI governance consultation works well with compliance-heavy firms"]
- [Angle 2]

**Warm introduction paths identified:**
- [Path 1 — e.g. "EXP-003 (consultant) has offered introductions to 4 Tier 1 firms"]
- [Path 2]

**Adjustments to outreach strategy:**
- [Adjustment 1 — rationale]
- [Adjustment 2 — rationale]

---
```

---

## Outreach Templates

Standard outreach messages for each relationship stage transition. Adapt to the specific firm and contact — do not send generic messages.

### Stage 3 → 4: LinkedIn connection request

> "Hi [Name], I've been following your work at [Firm] — particularly your comments on [specific topic from OBS record or LinkedIn activity]. I'm building a platform for investment decision management and would value connecting. [Your name]"

*Keep it short. Do not pitch. Reference something specific.*

---

### Stage 4 → 6: First outreach email

> "Hi [Name],
>
> I've spent the last few months researching how mid-market asset managers are managing investment decision documentation as AI tools become more embedded in the research and decision process.
>
> [Specific observation about their firm — e.g. "I noticed [Firm] has been expanding its AI research capability" or "Your comments at [conference] on [topic] resonated with what I'm hearing from other CIOs."]
>
> I'm building a prototype that reconstructs investment decisions and generates explainable decision records — the kind of documentation that makes LP reporting and regulatory review significantly faster. I'd value 30 minutes of your perspective on whether this addresses real operational problems at firms like yours.
>
> Would you be open to a brief call?
>
> [Your name]"

*Reference something specific to their firm. Lead with the product, not the research. Keep it under 150 words.*

---

### Stage 7 → 8: Scheduling the first call

> "Thank you for responding. I'd suggest a 45-minute call — I'll spend the first 20 minutes understanding your current process, and the last 25 minutes showing you what we've built and getting your reaction.
>
> [Calendar link or proposed times]"

---

### Stage 10 → 11: Design partner conversation

> "Based on our conversations, I think [Firm] would be an excellent design partner for the next phase. What that would involve: early access to the product, two structured feedback sessions per month, and — if it works for you — a reference conversation with future customers. In exchange, you'd have direct influence over the product direction.
>
> Would that be worth exploring?"

---

*COM-001 Commercial Intelligence Database v1.0 | July 2026 | ChronoSentiment Phase 1BA*
*Operational document — updated continuously throughout Phase 1BA and Phase 1B.*
*Links to: CV-001 (validation protocol), EL-001 (evidence records).*
*Review trigger: After every 20 firm dossiers (intelligence synthesis); at Phase 1B completion.*