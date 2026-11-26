# PROVISIONAL PATENT SPECIFICATION & CLAIM OUTLINE

**Title of the Invention:**  
**A HIGH-SPEED EVOLUTIONARY SYSTEM AND DETERMINISTIC FEASIBILITY RESTORATION METHOD FOR LARGE-SCALE REGULATED AIRLINE CREW ROSTER OPTIMIZATION**

**Filing Jurisdiction:** Indian Patent Office (IPO) / International Patent System (PCT)  
**Applicant / Assignee:** Coralys Research / ChronoSentiment  
**Classification (IPC/CPC):** G06Q 10/06 (Human resources & personnel scheduling), G06N 3/12 (Evolutionary algorithms), B64F 5/00 / G06Q 50/40 (Air transportation & crew management)  
**Applicable Government Schemes:** Maharashtra State Innovation Society (MSInS) Patent Reimbursement Scheme (up to ₹2,00,000 for Indian filing; up to ₹10,00,000 for PCT); DPIIT Startup Patent Fast-Track & 80% Fee Concession.

---

## 1. TECHNICAL FIELD OF THE INVENTION

The present invention relates generally to computerized optimization systems for transportation networks, and more particularly to a high-speed multi-objective evolutionary optimization engine and deterministic polynomial-time feasibility repair mechanism for solving large-scale personalized airline crew rostering problems (PCRP) under non-linear statutory aviation regulations, station capacity constraints, and operational disruption scenarios.

---

## 2. BACKGROUND OF THE INVENTION & PRIOR ART LIMITATIONS

Commercial airline crew scheduling is one of the most operationally critical and computationally intractable problems in industrial operations research. Airlines typically partition the problem into two sequential phases:
1. **Crew Pairing Generation (CPP):** Slicing the master flight timetable into multi-day sequences of flight duties and intermediate layovers (pairings) that depart from and terminate at designated crew bases.
2. **Personalized Crew Rostering (PCRP):** Assigning pairings and pre-assigned activities (vacations, training, medical blocks) to individual pilots over a monthly horizon (e.g., 31 days) subject to strict civil aviation regulations (such as Transport Canada CAR 700, FAA 14 CFR Part 117, and DGCA CAR Section 7).

### Limitations of the Prior Art:

1. **Exact Mathematical Programming (Branch-and-Price / Column Generation):**
   - Established solvers such as GENCOL (Kasirzadeh, Saddoune, and Soumis, 2017) solve the PCRP as a set-partitioning integer linear program using column generation and branch-and-price.
   - *Limitation:* For a fleet of 305 pilots and 1,648 pairings (GERAD Instance 7), GENCOL requires over 3.0 hours (11,085.6 seconds), and the 7-instance suite requires 7.66 hours (27,579.6 seconds). Such latency prevents real-time disruption mitigation in Airline Operations Control Centers (AOCC).
   - Furthermore, rigid daily station availability constraints force exact solvers to leave 181 flights uncovered (0.62%), generate wide workload disparity ($\sigma = 10.82$ hours), and leave multiple pilots completely unutilized (0 flight hours).

2. **Machine-Learning Constructive Heuristics:**
   - Heuristics such as `seqAsg` (Racette, Quesnel, Lodi, and Soumis, 2024) use neural networks trained via CMA-ES to sequentially build rosters.
   - *Limitation:* Pure ML heuristics sacrifice significant solution quality, exhibiting a 7.20% average preference score optimality loss (exceeding 10.76% on Instance 5), and fail to provide mathematical guarantees of zero statutory rest or cumulative overtime violations.

3. **Machine-Learning Accelerated Windowing (`win-ML`):**
   - Windowing frameworks (Racette et al., 2025/2026) seed rolling 10-day or 15-day branch-and-price windows in GENCOL with ML heuristics.
   - *Limitation:* While reducing solve times to 50–197 seconds on single instances, `win-ML` still requires multi-minute compute budgets, depends on expensive offline supervised training of neural networks on historical fleet runs, and cannot handle ad-hoc disruption recovery without retraining.

4. **The "End-of-Month Cliff" and Upstream Structural Deficits:**
   - Upstream crew pairing solvers (Quesnel et al., 2017; Zeighami and Soumis, 2019) generate static pairings without visibility into crew monthly legal hour ceilings.
   - On tightly constrained fleets (Instance 7), pilots burn through their 85.00h monthly statutory flying cap during the first 25 days, causing an "end-of-month cliff" where 28 of 29 uncovered pairings occur between Day 26 and Day 30. Prior solvers either drop these flights or deploy indiscriminate positioning that oversaturates fleet capacity.

Accordingly, there is a profound unmet need for an optimization system capable of solving full-scale 31-day airline fleets in seconds, guaranteeing 100% regulatory compliance with zero statutory violations, equalizing pilot workload variance, and dynamically recovering structural station deficits.

---

## 3. SUMMARY OF THE INVENTION

The present invention solves the aforementioned challenges by providing the **Coralys Evolutionary Scheduler**, an ultra-fast, multi-objective optimization engine coupled with a deterministic polynomial-time invariant-preserving feasibility restoration mechanism.

### Key Inventive Concepts:

1. **Deterministic Polynomial-Time Feasibility Restoration Operator $\mathcal{R}(g)$:**
   An algorithmic repair operator with worst-case complexity $\mathcal{O}(|P|\log|P| + |P|\cdot|C|)$ that transforms any arbitrary, infeasible offspring chromosome into a strictly compliant commercial roster. The operator executes a three-phase pipeline:
   - *Phase 1 (Chronological Invariant Audit):* Sorts pairings chronologically per crew member and evicts blocks violating base compatibility, minimum rest gaps ($\ge 12.0$ hours), or the cumulative monthly flying hour ceiling ($\le 85.00$ hours) into a dynamic reserve pool buffer $|C|$.
   - *Phase 2 (Workforce Utilization Seeding):* Identifies empty crew lines and seeds base-compatible candidate pairings to eliminate zero-hour idle pilots.
   - *Phase 3 (Equitable Headroom-Aware Packing):* Iteratively reallocates unassigned pairings to eligible crew lines possessing maximal legal headroom, satisfying all non-linear statutory invariants.

2. **Directional Deficit-Targeted Cross-Base Positioning:**
   A network flow mechanism that calculates station-level supply-demand differentials ($\text{Statutory Headroom} - \text{Required Flight Block Time}$). The system strictly prohibits crew repositioning into surplus domiciles, exclusively routing deadheads from surplus bases to deficit stations, preventing total fleet capacity oversaturation.

3. **Multi-Level Ejection Chain Calendar Friction Polish:**
   A specialized neighborhood transition operator that resolves pre-assigned calendar locks (vacations, AST blocks) via an ordered multi-tier swap cascade ($P_{\text{unassigned}} \to R_1$, $V_1 \to R_2$, $V_2 \to R_3$), freeing congested flying slots without inflating deadhead credit burns.

4. **Month-End Rollover Carry-Over Accounting and Hub-Split Fallback:**
   An operational transition pipeline that evaluates pairings crossing the monthly horizon ($M_1 \to M_2$). Flight hours prior to 23:59 UTC on Day 31 are counted against Month 1's statutory cap, rolling post-midnight legs into Month 2 as pre-assigned Carry-In Locks. If a crew member lacks legal pre-midnight headroom, the system automatically falls back to splitting the pairing at intermediate home-base hub layovers into two legal, base-closed pairings.

---

## 4. DETAILED DESCRIPTION OF PREFERRED EMBODIMENTS

### 4.1 Chromosome Representation & Domain Filtering

Let $P = \{p_1, \dots, p_N\}$ denote the set of flight pairings and $C = \{c_1, \dots, c_M\}$ denote the pilot workforce. The chromosome is encoded as a discrete vector:
$$g \in \{0, 1, \dots, M\}^N$$
where $g[i] = c$ assigns pairing $p_i$ to pilot $c$, and index $M = |C|$ denotes an unassigned reserve pairing.

To eliminate base mismatch penalties, each pairing $p_i$ is mapped to a pre-screened domain:
$$\mathcal{V}(p_i) = \{c \in C \mid \text{base}(c) = \text{base}(p_i)\} \cup \{|C|\}$$
Genetic recombination operators sample exclusively within $\mathcal{V}(p_i)$.

### 4.2 Mathematical Logic of Repair Operator $\mathcal{R}(g)$

```
Algorithm 1: Deterministic Feasibility Restoration Operator R(g)
Input: Candidate chromosome g, Pairings P, Pilots C, Delta_rest (720 min), H_max (85.00h)
Output: Strictly feasible chromosome g*

1: For each crew member c in C:
2:     Let P_c = {p in P | g[p] == c}
3:     Sort P_c in ascending order of start_time(p)
4:     accumulated_hours = 0
5:     last_end_time = -infinity
6:     For each pairing p in P_c:
7:         If (start_time(p) - last_end_time < Delta_rest) OR
              (accumulated_hours + credited_hours(p) > H_max) OR
              (conflicts_with_preassigned_leave(c, p)):
8:             g[p] = |C|  // Evict to reserve pool
9:         Else:
10:            accumulated_hours += credited_hours(p)
11:            last_end_time = end_time(p)

12: // Underutilization check
13: For each crew member c in C with accumulated_hours == 0:
14:     Find first unassigned pairing p in reserve pool where base(p) == base(c) and legal(c, p)
15:     If found: assign g[p] = c; update accumulated_hours

16: // Equitable Headroom-Aware Packing
17: Let Unassigned = {p in P | g[p] == |C|}
18: Shuffle Unassigned
19: For each pairing p in Unassigned:
20:     eligible_candidates = {c in C | base(c) == base(p) AND legal_insert(c, p)}
21:     If eligible_candidates is not empty:
22:         best_c = argmin_{c in eligible_candidates} (accumulated_hours(c))
23:         g[p] = best_c
24:         accumulated_hours(best_c) += credited_hours(p)

25: Return g
```

---

## 5. PATENT CLAIMS (FORMAL DRAFT)

### WE CLAIM:

**Claim 1 (Independent System Claim):**  
A computer-implemented system for real-time optimization of personalized airline crew rosters across a multi-base transportation network, the system comprising:
- one or more processors; and
- a memory storing instructions that, when executed by the one or more processors, cause the system to:
  - ingest a master schedule of flight pairings, pilot profile records, station domicile affiliations, pre-assigned leave commitments, and statutory flight duty limitations;
  - generate a population of candidate roster chromosomes bounded by pre-screened base-compatible candidate domains;
  - execute an evolutionary multi-objective search using crossover and mutation operators;
  - transform every candidate chromosome via a deterministic polynomial-time feasibility restoration module $\mathcal{R}(g)$ that eliminates statutory rest violations, consecutive duty breaches, and cumulative monthly flying hour ceiling violations without human intervention; and
  - output a legally compliant 31-day crew roster exhibiting zero regulatory violations and minimized workforce flying hour variance within a computational runtime of less than sixty seconds.

**Claim 2 (Independent Method Claim):**  
A computer-implemented method for restoring legal feasibility in an airline crew rostering chromosome representing pairing-to-pilot assignments, comprising:
- parsing, for each crew member, assigned flight pairings into a chronologically sorted sequence;
- identifying conflicting pairings that violate a minimum statutory rest interval between consecutive duty periods or cause cumulative monthly credited flight hours to exceed an absolute statutory threshold;
- evicting said conflicting pairings into a dynamic unassigned reserve pool;
- identifying unutilized crew members possessing zero assigned flying hours and seeding at least one base-compatible pairing from the unassigned reserve pool into each unutilized crew member's rotation; and
- reallocating remaining unassigned pairings from the reserve pool to eligible crew members having the lowest accumulated flying hours subject to preserving strict regulatory rest and monthly flying hour limits, wherein the method executes in deterministic $\mathcal{O}(|P|\log|P| + |P|\cdot|C|)$ computational complexity.

**Claim 3 (Dependent Claim - Deficit-Targeted Repositioning):**  
The method of Claim 2, further comprising:
- calculating a net station capacity balance vector for each crew domicile by subtracting total required pairing block hours from statutory base pilot capacity;
- classifying each domicile as either a surplus domicile or a deficit domicile; and
- routing crew deadhead repositioning flights exclusively from surplus domiciles into deficit domiciles, while strictly barring cross-base deadheads terminating at surplus domiciles.

**Claim 4 (Dependent Claim - Multi-Level Ejection Chains):**  
The method of Claim 2, wherein unassigned pairings obstructed by calendar congestion or pre-assigned leaves are resolved via a multi-level ejection chain comprising:
- identifying a high-priority unassigned pairing $P_{\text{unassigned}}$ belonging to a first rotation $R_1$;
- displacing an overlapping victim pairing $V_1$ from $R_1$ into a second rotation $R_2$ having legal headroom; and
- recursively displacing a secondary victim pairing $V_2$ from $R_2$ into a third rotation $R_3$, thereby unblocking $P_{\text{unassigned}}$ without generating deadhead flight hours.

**Claim 5 (Dependent Claim - Month-End Rollover Carry-Over Accounting):**  
The method of Claim 2, further comprising:
- identifying rollover pairings that initiate prior to midnight on the final day of a first monthly horizon and terminate in a subsequent second monthly horizon;
- crediting flight duty hours flown prior to 23:59 UTC against the first monthly statutory ceiling;
- exporting flight legs scheduled after 00:00 UTC as locked pre-assigned carry-in activities for the second monthly planning horizon; and
- preserving crew assignment continuity across the monthly boundary without splitting said pairings at outstations.

**Claim 6 (Dependent Claim - Hub-Split Fallback Mechanism):**  
The method of Claim 5, wherein if a crew member assigned to a rollover pairing has insufficient legal flying headroom prior to midnight, the system dynamically decouples the rollover pairing at an intermediate home-base hub layover into an outbound pairing terminating at the home domicile and a separate inbound pairing starting at the home domicile.

**Claim 7 (Dependent Claim - Multi-Objective Equity Optimization):**  
The system of Claim 1, wherein the multi-objective search optimizes a lexicographic fitness hierarchy prioritizing complete flight leg coverage and complete pairing coverage as primary invariants, and utilizing sample variance $\sigma^2$ of pilot credited flying hours as an equity tie-breaker, resulting in a fleet-wide workload standard deviation of $\sigma \le 3.5$ hours.

**Claim 8 (Dependent Claim - Statutory Invariant Enforcement):**  
The system of Claim 1, wherein the statutory limitations enforced by the feasibility restoration module comprise:
- minimum rest interval $\ge 12.0$ hours between consecutive pairings;
- maximum consecutive working days $\le 6$ days;
- minimum calendar days off $\ge 10$ days per 31-day period; and
- an absolute monthly credited flying ceiling $\le 85.00$ hours.

**Claim 9 (Dependent Claim - Real-Time Performance & Disruption Recovery):**  
The system of Claim 1, wherein the system optimizes a fleet comprising at least 300 crew members and at least 1,600 flight pairings in under 6.0 seconds for nominal baseline planning, and under 90.0 seconds for complex networks requiring cross-base deadhead positioning.

**Claim 10 (Dependent Claim - Anonymous Profile & Privacy Architecture):**  
The system of Claim 1, wherein pilot identity is fully anonymized during the evolutionary search, mapping crew members strictly to domicile capability vectors to ensure bias-free and privacy-preserving schedule generation.

---

## 6. GOVERNMENT SCHEME & PATENT FILING ROADMAP

### Indian Patent Office (IPO) Filing Roadmap:
1. **Filing Forms Required:**
   - **Form 1:** Application for Grant of Patent.
   - **Form 2:** Provisional Specification (attaching Sections 1–4 of this document).
   - **Form 3:** Statement and Undertaking regarding foreign filings (Section 8).
   - **Form 5:** Declaration as to Inventorship.
2. **Startup Concessions:**
   - Under Rule 24C of the Indian Patent Rules, a DPIIT-recognized startup receives an **80% official fee reduction** and eligibility for **Expedited Examination**.
   - Official filing fee for a startup provisional application: **₹1,600** (vs. ₹8,000 for large entities).
3. **Maharashtra State Innovation Society (MSInS) Reimbursement:**
   - As a Maharashtra-registered entity, the applicant is eligible for up to **₹2,00,000 reimbursement** for Indian patent attorney and filing fees under the Maharashtra State Innovative Startup Policy.
   - For subsequent International PCT filing (within 12 months of provisional priority date), MSInS reimburses up to **₹10,00,000**.
4. **12-Month Convention Priority Period:**
   - Filing the Indian provisional application locks in the global priority date under the Paris Convention and Patent Cooperation Treaty (PCT).
   - Once filed, the applicant may freely share, publish, or present the academic paper (`paper_gerad_coralys_v2.tex`) worldwide without destroying patent novelty.
