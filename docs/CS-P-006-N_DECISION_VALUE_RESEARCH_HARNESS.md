# CS-P-006-N — Decision Value Research Harness

**Document type:** Measurement and protocol-validation harness  
**Status:** Implemented — C.3 protocol opened separately; this harness does not start Search #2  
**Date:** 2026-08-15  
**Parent:** CS-P-006-M.1  
**Does not:** evolve, select a new PolicyArtifact, score evaluation into Coralys, rewrite Search #1, implement fitness inside `coralys-moga`, authorize C.3  

`.cursor/rules/chronosentiment-core.mdc`: ChronoSentiment remains the evaluator; a harness that measures decision value must not become a second search.

---

## What this freeze is

Verify that the implementation enforces CS-P-006-M.1 **before** any C.3 authorization decision.

```text
CS-P-006-N
Decision Value Harness
        ↓
PASS / FAIL
        ↓
C.3 authorization decision
        ↓
Coralys search   ← not this document
```

Do not call the next step a Coralys search until N has passed **and** a separate C.3 decision is recorded. This document records N PASS as engineering validation. It does **not** authorize C.3.

Sidecar: `product_validation/CS-P-006/discovery/20260814T195327Z/harness/`

Search #1 remains `9a887827e8f41988987208f13e4ccbac507b3241692026c55f38d11f85971ac0`.

---

## Required contract

```text
Certified State(T) → PolicyArtifact → Action → V(action)
V(LONG)=R   V(SHORT)=−R   V(NO_TRADE)=0
```

Then independently:

```text
protocol_value = mean of seven per-instrument means of V
regret, unique_best, opportunity_cost  → diagnostics only
```

`ProtocolValue` can be constructed only from per-instrument V means. There is no `from_regret` / `from_unique_best`. Evaluation cannot produce a search-admissible protocol value.

### Mandatory symbol matrices

7 symbols × 13 Development + 13 Selection + 13 Evaluation = 39 decisions each. The harness **fails** if any symbol×slice is not 13 rows.

**Table A — Decision distribution** (what the policy recommended)

```text
Symbol × Slice × LONG / SHORT / NO_TRADE
```

**Table B — Decision value** (what those recommendations subsequently generated)

```text
Symbol × Slice × Mean V / Median V / Mean regret / Unique-best / Opportunity cost
```

These tables are part of the contract, not a visualization. A chart must not replace them.

---

## Harness output on Search #1

Identity-gated run. Authoritative matrices: `product_validation/CS-P-006/discovery/20260814T195327Z/harness/HARNESS.md`. C.3 remains unauthorized.

Because each symbol has exactly 13 decisions per slice, the M.1 protocol scalar equals the equal-weighted row mean on this grid. It still differs from Search #1 fitness: NO_TRADE enters as 0, so development protocol V is **+0.7559%** versus Search #1’s traded-only **+1.6325%**.

| Slice | Search-admissible | Protocol V | Search #1 traded-only | Mean regret | Unique-best |
|-------|-------------------|-----------:|----------------------:|------------:|------------:|
| development | yes | +0.7559% | +1.6325% | 4.78% | 31.9% |
| selection | yes | +0.6724% | +1.9938% | 4.38% | 25.3% |
| evaluation | **no** | −0.5825% | −0.0229% | 5.62% | 18.7% |
| all | **no** | +0.2819% | +0.825% | 4.92% | 25.3% |

SHORT is **0** in every cell. This sealed policy never competes LONG against SHORT.

### Table A — Decision distribution (what was recommended)

| Symbol | Dev LONG | Dev NT | Sel LONG | Sel NT | Eval LONG | Eval NT | All % LONG |
|--------|---------:|-------:|---------:|-------:|----------:|--------:|-----------:|
| HDFCBANK.NS | 7 | 6 | 7 | 6 | 4 | 9 | 46.2% |
| ICICIBANK.NS | 5 | 8 | 7 | 6 | **1** | **12** | 33.3% |
| INFY.NS | 5 | 8 | 7 | 6 | 5 | 8 | 43.6% |
| RELIANCE.NS | 8 | 5 | 5 | 8 | 6 | 7 | 48.7% |
| TCS.NS | **10** | 3 | **3** | 10 | 5 | 8 | 46.2% |
| IDEA.NS | 9 | 4 | 6 | 7 | 7 | 6 | **56.4%** |
| MAHABANK.NS | 5 | 8 | 4 | 9 | 5 | 8 | 35.9% |

Every slice total is 13. Every SHORT is 0. The action mix is **not** uniform: TCS is 76.9% LONG in development and 23.1% in selection. ICICI is almost entirely NO_TRADE in evaluation (1/13 LONG).

### Table B — Decision value (what those recommendations generated)

Evaluation (diagnostic only):

| Symbol | LONG | Mean V | Mean regret | Unique-best | Mean V when acted |
|--------|-----:|-------:|------------:|------------:|------------------:|
| HDFCBANK.NS | 4 | +1.03% | 3.26% | 3/13 | +3.34% |
| ICICIBANK.NS | 1 | +0.58% | 2.47% | 1/13 | +7.54% |
| INFY.NS | 5 | +1.04% | 3.56% | 4/13 | +2.70% |
| RELIANCE.NS | 6 | −1.19% | 4.38% | 2/13 | −2.58% |
| TCS.NS | 5 | +1.12% | 2.32% | 4/13 | +2.92% |
| IDEA.NS | 7 | **−4.34%** | **15.59%** | 2/13 | **−8.06%** |
| MAHABANK.NS | 5 | −2.31% | 7.74% | 1/13 | −6.02% |

MAHABANK reverses: selection mean V **+2.50%** (acted +8.11%) → evaluation **−2.31%** (acted −6.02%). IDEA already leaks in selection (−0.76%) and collapses in evaluation.

Full 28-row Table A and Table B are in `HARNESS.md`.

---

## Differentiated-value inspection

N **PASS** is engineering validation. It is **not** a C.3 authorization.

Does M.1 `V` produce enough differentiated value for Coralys to learn from TMV?

**What N shows**

* Value and regret are **not** uniform across symbols. Evaluation regret is concentrated in IDEA (15.59%) and MAHABANK (7.74%). HDFC / INFY / TCS stay positive on evaluation mean V.
* The action mix changes by name and by slice. That is mostly how often each name is Bearish, because the sealed rule is `Bearish → LONG`.
* SHORT is never used, so this artifact cannot tell us whether TMV separates three competing actions.
* Unique-best remains **18.7%** on evaluation. Most states do not uniquely identify an action.
* Including NO_TRADE as 0 **halves** the development scalar versus Search #1’s traded-only mean. Standing aside is no longer invisible.

**What N does not show**

Enough structure to justify another Coralys search. Heterogeneity can be learned or overfit. Concentration in IDEA is a warning, not a reason to drop the ticker after seeing the holdout.

C.3 remains a **separate authorization**. Possible later deficiencies, if a search is ever justified: information at T, representation, objective, horizon, or insufficient TMV structure. N does not choose among those.

---

## Code

| Piece | Location |
|-------|----------|
| Harness | `adapters/chronosentiment/src/decision_support/decision_value_harness.rs` |
| Binary | `src/bin/csp006_decision_value_harness.rs` |
| Runner | `run_csp006_decision_value_harness.sh` |
| Sidecar | `product_validation/CS-P-006/discovery/20260814T195327Z/harness/` |
| Tests | `adapters/chronosentiment/tests/csp006n_decision_value_harness_tests.rs` |

Engine version remains **`unfrozen-dev`**. No real capital. C.3 is not authorized by this harness. The protocol document is CS-P-006-C.3. Search #2 is not started.
