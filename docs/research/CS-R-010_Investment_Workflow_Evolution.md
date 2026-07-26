# CS-R-010 — Investment Workflow Evolution
## ChronoSentiment Research Series | v1.0

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | **Research Baseline v1.0** |
| Evidence Version | v1.0 |
| Research Date | July 2026 |
| Evidence Cut-off Date | July 2026 |
| Next Review | January 2027 or upon material new workflow research |
| Owner | ChronoSentiment Programme |
| Review Trigger | New CFA Institute, McKinsey, or Deloitte investment workflow research; material shift in AI tool adoption patterns in investment management |

---

## Confidence Scale

| Rating | Definition |
|--------|-----------|
| **A** | Multiple independent high-quality sources; directly verifiable |
| **B** | Several reliable sources with some estimation or inference |
| **C** | Limited public evidence; industry estimates or analyst commentary |
| **D** | Strategic interpretation; requires validation before acting |

---

## Related Research

| Document | Relationship |
|----------|-------------|
| CS-R-001 Market Landscape v2.0 | Workflow evolution affects all customer segments identified |
| CS-R-003 Customer Problem Evidence v2.0 | Workflow gaps are the operational expression of the customer problems |
| CS-R-009 AI Adoption in Investment Management | AI adoption is the primary driver of workflow evolution |
| CS-R-011 Decision Governance Research | Decision governance requirements emerge from workflow evolution |
| CS-R-012 Build vs Buy Analysis | Workflow integration requirements inform build vs buy decisions |

**Feeds into:** PRD v1.0 (workflow integration requirements), Phase 1B customer validation (workflow mapping), M-series architecture (integration points)

---

## Research Limitations

This document maps investment workflows based on secondary research, industry surveys, and published practitioner accounts. It does not establish:

- The specific workflow configurations at ChronoSentiment's target customer segments
- Which workflow stages are most painful or most amenable to change
- How firms would integrate a new governance layer into existing workflows
- Whether workflow disruption is a barrier to adoption

These questions require Phase 1B primary research including workflow observation and process mapping with target customers.

---

## 1. Purpose and Scope

This document maps the evolution of investment management workflows from traditional manual processes through current AI-assisted workflows, identifying the governance gaps that emerge at each stage. It provides the workflow context for ChronoSentiment's integration strategy.

**Central finding:** Investment workflows are evolving from sequential, human-driven processes toward parallel, AI-assisted processes. This evolution increases decision velocity and information volume but creates new governance gaps: AI-generated content enters the decision process without attribution, and the decision rationale is increasingly distributed across multiple tools and channels rather than captured in a single record.

---

## 2. Evidence

### 2.1 Traditional Investment Workflow (Pre-2020)

The traditional investment workflow in a discretionary asset management firm follows a broadly sequential pattern. **Confidence B.**

**Stage 1 — Research and information gathering:**
Analysts gather information from Bloomberg Terminal, broker research, company filings, and expert networks. Information is synthesised manually into research notes, typically stored in shared drives or email. Time: days to weeks per investment idea.

**Stage 2 — Investment thesis development:**
The analyst develops an investment thesis, typically documented in a research note or investment memo. The memo is reviewed by the portfolio manager and/or investment committee. Decision rationale is captured in the memo, but quality and completeness vary widely.

**Stage 3 — Investment committee review:**
For significant positions, the investment thesis is presented to an investment committee. Discussion occurs verbally; minutes may or may not be taken. The committee decision is communicated verbally or via email.

**Stage 4 — Execution:**
The portfolio manager instructs the trading desk to execute. The instruction is communicated verbally or via email. The trading desk executes via OMS/EMS. TCA is performed post-execution.

**Stage 5 — Monitoring and review:**
Positions are monitored against the original thesis. Review is typically triggered by price movement, earnings events, or portfolio rebalancing. The original thesis may or may not be accessible for comparison.

**Governance characteristics of traditional workflow:**
- Decision rationale captured in research memos (variable quality)
- Information environment at decision time not preserved
- Execution intent communicated informally (verbal, email)
- Post-hoc review difficult due to information decay
- Institutional memory held in individuals, not systems

### 2.2 Current AI-Assisted Workflow (2024–2026)

The current workflow in AI-adopting firms has evolved significantly. AI tools are inserted at multiple stages, increasing velocity and information volume but creating new governance gaps. **Confidence B.**

**Stage 1 — AI-assisted research and synthesis:**
Analysts use AI tools (ChatGPT Enterprise, Claude Enterprise, AlphaSense AI, Bloomberg AI) to accelerate information gathering and synthesis. Earnings call transcripts are summarised by AI. Filings are analysed by AI. News is synthesised by AI. Research velocity increases significantly — what took days now takes hours.

**Governance gap introduced:** AI-generated summaries influence the analyst's view but are not attributed in the research record. The specific AI model, version, prompt, and data used to generate the summary are not captured. If the AI summary contained an error or bias, it cannot be traced.

**Stage 2 — AI-assisted thesis development:**
AI tools assist in drafting investment memos, generating comparable company analyses, and synthesising risk factors. The analyst edits and refines the AI-generated draft. The final memo may not distinguish between AI-generated and human-authored content.

**Governance gap introduced:** The investment memo does not record which sections were AI-generated, which AI model was used, or what data the AI processed. The memo cannot be reproduced from its inputs.

**Stage 3 — Investment committee review (largely unchanged):**
Investment committee processes have not changed significantly with AI adoption. Discussion remains verbal; minutes remain inconsistent. AI-generated content presented in committee is not attributed.

**Stage 4 — Execution (largely unchanged):**
Execution processes have not changed significantly. OMS/EMS systems are the same. The gap between decision intent and execution record remains.

**Stage 5 — Monitoring with AI assistance:**
AI tools are increasingly used for portfolio monitoring — flagging news, earnings surprises, and risk events. AI-generated alerts influence monitoring decisions without attribution.

**Governance characteristics of current AI-assisted workflow:**
- AI-generated content influences decisions at every stage without attribution
- Research velocity has increased; governance has not kept pace
- Decision rationale is increasingly distributed across AI tool outputs, emails, and verbal discussion
- The information environment at decision time is even harder to reconstruct (AI outputs are not version-pinned)
- Institutional memory problem is compounded by AI tool proliferation

### 2.3 Emerging Workflow Patterns (2025–2026)

Several emerging workflow patterns are relevant to ChronoSentiment's positioning. **Confidence B.**

**Agentic AI workflows:** Some firms are experimenting with AI agents that autonomously gather information, synthesise research, and generate investment recommendations. These workflows increase automation but create new governance challenges: the agent's reasoning process is not transparent, and the data it accessed is not recorded.

**Multi-model workflows:** Firms are using multiple AI models for different tasks (e.g., Claude for document analysis, GPT-4o for synthesis, Gemini for multimodal analysis). The provenance of AI-generated content becomes increasingly complex as multiple models contribute to a single decision.

**Collaborative AI workflows:** AI tools are being used in collaborative settings — investment committees where AI-generated analysis is shared and discussed. Attribution of AI-generated content in collaborative settings is particularly difficult.

**Confidence B** for all emerging patterns — based on industry reports and practitioner accounts, not systematic survey data.

### 2.4 Workflow Integration Points for ChronoSentiment

Based on the workflow mapping above, ChronoSentiment's natural integration points are: **Confidence D — strategic interpretation.**

| Workflow Stage | Current Tool | Governance Gap | ChronoSentiment Integration |
|---------------|-------------|----------------|----------------------------|
| Research synthesis | AlphaSense, Bloomberg AI, ChatGPT | AI output not attributed | Capture AI tool, version, prompt, output at synthesis stage |
| Thesis development | ChatGPT, Claude, Word/Google Docs | AI-generated content not distinguished | Structured decision template with AI attribution fields |
| Committee review | Email, verbal, meeting notes | Decision rationale not captured | Decision record capture at committee stage |
| Execution instruction | Email, verbal, OMS | Intent not formally recorded | Execution intent capture linked to decision record |
| Post-trade review | TCA tools, spreadsheets | Original intent not accessible | Decision replay — compare original intent with actual execution |
| Monitoring | Bloomberg alerts, AI tools | Monitoring decisions not linked to original thesis | Monitoring event linked to original decision record |

---

## 3. Research Findings

### Finding 1: AI adoption has increased workflow velocity but not governance (Confidence B)

The introduction of AI tools at the research and synthesis stages has significantly increased the velocity of information processing. However, governance infrastructure has not kept pace: AI-generated content enters the decision process without attribution, version control, or audit trails. The governance gap is a direct consequence of workflow evolution.

### Finding 2: Decision rationale is increasingly distributed across tools and channels (Confidence B)

In traditional workflows, decision rationale was concentrated in research memos and investment committee minutes (variable quality). In AI-assisted workflows, decision rationale is distributed across: AI tool outputs, analyst annotations, email threads, Slack/Teams messages, and verbal discussion. This distribution makes post-hoc reconstruction of decision rationale significantly more difficult.

### Finding 3: The execution intent gap has not been addressed by workflow evolution (Confidence B)

Despite significant workflow evolution at the research and synthesis stages, the execution stage has not changed materially. The gap between decision intent (what the portfolio manager intended) and execution record (what the OMS/EMS recorded) remains. TCA tools measure execution quality against market benchmarks but do not validate execution against decision intent.

### Finding 4: Agentic AI workflows will intensify the governance gap (Confidence C)

Emerging agentic AI workflows — where AI agents autonomously gather information and generate recommendations — will intensify the governance gap. If an AI agent's reasoning process is not transparent and its data sources are not recorded, the governance problem becomes significantly more complex. This is a forward-looking finding based on current trajectory, not established practice.

### Finding 5: ChronoSentiment's integration points are at the research-to-decision transition (Confidence D)

The highest-value integration point for ChronoSentiment is at the transition from research synthesis to decision capture — the moment when AI-generated research is translated into an investment decision. This is where the governance gap is most acute and where structured capture would have the highest impact. This is a strategic interpretation requiring Phase 1B validation.

---

## 4. Implications

**4.1 ChronoSentiment must integrate with existing workflows, not replace them.** Investment workflows are deeply embedded in firm culture and existing tool stacks. A governance layer that requires significant workflow change will face adoption resistance. ChronoSentiment must integrate at the natural decision capture points without disrupting the research and synthesis workflow.

**4.2 The research-to-decision transition is the critical integration point.** The moment when an analyst or portfolio manager translates AI-generated research into an investment decision is the highest-value capture point. ChronoSentiment should be designed to make this capture as frictionless as possible.

**4.3 AI tool attribution is a new requirement that existing tools do not address.** No current tool captures which AI model, version, prompt, and data contributed to a specific investment decision. This is a new requirement created by AI adoption and is not addressed by any existing workflow tool.

**4.4 Workflow mapping is a Phase 1B requirement.** The workflow analysis in this document is based on secondary research. Phase 1B must include workflow observation and process mapping with target customers to understand the specific integration points and friction points in their actual workflows.

---

## 5. Recommendations

**Recommendation 1: Design ChronoSentiment as a workflow integration layer, not a workflow replacement.**
The product must integrate at the natural decision capture points in existing workflows — research memo creation, investment committee review, execution instruction — without requiring firms to change their core research and analysis processes. *Priority: High. Product design.*

**Recommendation 2: Prioritise frictionless decision capture at the research-to-decision transition.**
The highest-value feature is the ability to capture a decision record at the moment of decision, with minimal friction. This means: one-click capture from existing tools (email, Slack, research platforms), structured template that pre-populates from context, and AI attribution fields that automatically capture tool and version. *Priority: High. MVP design.*

**Recommendation 3: Include workflow observation in Phase 1B.**
Phase 1B interviews should include workflow observation sessions where the interviewer watches a target customer execute their actual investment workflow. This will reveal integration points and friction points that cannot be identified from interviews alone. *Priority: High. Phase 1B.*

**Recommendation 4: Monitor agentic AI workflow development.**
Agentic AI workflows are emerging and will intensify the governance gap. ChronoSentiment's architecture should be designed to capture AI agent reasoning and data sources, not just human decision rationale. This is a Phase 3+ requirement but should be considered in the M-series architecture. *Priority: Low. Phase 3+.*

---

## Evidence Quality

| Source Type | Examples | Confidence |
|-------------|---------|-----------|
| Industry surveys | PwC AWM 2025, McKinsey AI Survey 2025, CFA Institute AI Pioneers 2026 | B |
| Practitioner accounts | Published case studies, conference presentations | B–C |
| Workflow analysis | Structured analysis of published workflow descriptions | B |
| Integration point identification | Strategic interpretation of workflow analysis | D |

---

## Evidence Classification

**Published evidence:** PwC AWM 2025, McKinsey AI Survey 2025, CFA Institute AI Pioneers 2026, Deloitte IM Outlook 2025 — all documenting AI adoption patterns in investment workflows.

**Derived findings:** Workflow stage mapping derived from published practitioner accounts and industry surveys; governance gap identification derived from workflow analysis and AI adoption evidence.

**Strategic interpretation (Confidence D):** Integration point identification; research-to-decision transition as highest-value capture point; agentic AI workflow trajectory. These require Phase 1B workflow observation before adoption as the basis for product design decisions.

---

*CS-R-010 v1.0 | ChronoSentiment Research Series | Evidence cut-off: July 2026*