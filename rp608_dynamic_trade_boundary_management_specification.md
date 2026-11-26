# RP-608 — Dynamic Trade Boundary Management Specification

**Status**: SPECIFIED / GATED (POST-RP-605 & RP-607)  
**Governance Standard**: RP-600 Pre-Check Verified | Execution-Layer Boundary Protocol  
**Target Mechanism**: Live Trade Stop-Loss & Target Recommendation, Validation & Atomic State Transitions  
**Strict Invariant**: Zero Mutation of Active RP-605 Prospective Baseline; Evaluated as an Independent Policy  

---

## 1. Executive Summary & Architectural Scope

This specification establishes the architectural contract for **Dynamic Trade Boundary Management** (Stop-Loss & Target updates for active positions).

It resolves the trade-off between **"freeze forever"** and **"recalculate on every tick"**:

> **Central Architectural Invariant**:  
> A live trade's stop-loss and target are frozen at entry by default. They remain unchanged between explicit, validated updates. They must NOT mutate on every tick, but may be updated when a deliberate, evidence-based trade-management recommendation is explicitly validated and applied.

---

## 2. Six-Step Live Trade Lifecycle

```text
1. TRADE ENTRY
   Capture initial SL & target; freeze as initial risk/reward boundaries
        │
        ▼
2. LIVE MONITORING
   Monitor market ticks & PnL; frozen values remain unmutated by ticks
        │
        ▼
3. RECOMMENDATION DETECTED
   Generate explicit recommendation: PROPOSED state (SL/target, rationale, timestamp)
        │
        ▼
4. RECOMMENDATION VALIDATION
   Validate against price, direction, risk limits & execution constraints: VALIDATED state
        │
        ▼
5. ATOMIC STATE TRANSITION
   Atomically update active trade boundaries: APPLIED state with version history
        │
        ▼
6. CONTINUED MONITORING
   Updated values become active boundaries until next validated recommendation or exit
```

---

## 3. Three-State Recommendation Lifecycle Engine

To prevent theoretical recommendations from leaking into active execution prior to validation, every proposed update must transition through three explicit states:

```text
    PROPOSED ──────────► VALIDATED ──────────► APPLIED
       │                    │
       ▼                    ▼
   REJECTED             EXPIRED
```

| State | Definition & Execution Rules |
| :--- | :--- |
| **`PROPOSED`** | A better stop-loss or target has been calculated by the recommendation layer. Contains proposed values, timestamp, and supporting rationale. Not yet active. |
| **`VALIDATED`** | Recommendation has passed all risk limits, directional monotonicity rules, price crossing checks, and execution constraints. |
| **`APPLIED`** | The active trade record has been atomically updated. New boundaries take effect immediately from the effective timestamp onward. |
| **`REJECTED`** | Recommendation failed validation (e.g. price already crossed, risk limit breach, non-monotonic SL move). |
| **`EXPIRED`** | Recommendation was superseded by a newer market state prior to application. |

---

## 4. Mandatory Update Safeguards

### 4.1 Monotonic Stop-Loss Movement
- **LONG Positions**: Stop-loss may ONLY move **upward** (tightening/protecting profit). Loosening requires a separately authorized, explicitly gated strategy.
- **SHORT Positions**: Stop-loss may ONLY move **downward** (tightening/protecting profit).

### 4.2 Target Flexibility
- Targets may be moved closer or farther when supported by a validated recommendation. Targets must NEVER alter simply because a new tick arrives.

### 4.3 Atomic Update Transaction
- Stop-loss and target changes forming a single recommendation MUST be applied as **one atomic update transaction** to prevent inconsistent intermediate trade states.

### 4.4 Price Validation & Crossed-Boundary Rejection
- Any proposed update whose stop-loss or target is already crossed by the current market price MUST be rejected immediately.

### 4.5 Version History & Audit Trail
- Every trade record MUST retain its initial entry boundaries, every `PROPOSED`, `VALIDATED`, `REJECTED`, or `APPLIED` recommendation, and atomic transition timestamps.

### 4.6 No Retroactive Effect
- Accepted updates apply strictly from their effective timestamp onward. They must NEVER rewrite earlier trade risk boundaries or historical execution decisions.

---

## 5. Architectural Insulation & Governance Control Matrix

```text
CORALYS CORE (Frozen)  ──► Continues producing baseline signals & decision cards
                                  │
                                  ▼
EXECUTION LAYER         ──► Evaluates recommendations, validates & applies updates
                                  │
                                  ▼
TRADE LEDGER            ──► Logs initial boundaries, update version history & exits
                                  │
                                  ▼
RP-606/607 OBSERVERS    ──► Read-only tracking of boundary lifecycles (zero feedback)
```

### Governance Control Matrix

| System Component | Governance Constraint |
| :--- | :--- |
| **Active RP-605 Baseline** | 🟢 **100% FROZEN / ISOLATED**. Dynamic boundary updates are strictly **FORBIDDEN** within the live RP-605 control baseline. |
| **RP-608 Evaluation Policy** | 🧪 **SEPARATE POLICY**. RP-608 MUST be evaluated as a distinct execution-layer policy to ensure clean attribution. |
| **Coralys Core Logic** | 🔒 **UNTOUCHED**. Core decision state engine remains 100% frozen. |
| **Trade Ledger Integrity** | 📄 **FULL VERSIONING**. Every accepted and rejected update is logged with an immutable timestamp. |

---

## 6. Concrete Example

Consider a LONG trade opened at ₹1,000:

| Lifecycle Phase | Market Price | Active Stop-Loss | Active Target | Recommendation State | Action / Rationale |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **Initial Entry** | ₹1,000 | ₹990 | ₹1,020 | `APPLIED (v0)` | Frozen initial boundaries |
| **Market Tick** | ₹1,010 | ₹990 | ₹1,020 | None | Ticks do not alter boundaries |
| **Recommendation** | ₹1,010 | ₹1,005 | ₹1,025 | `PROPOSED (v1)` | Monotonic SL move (+₹15), Target (+₹5) |
| **Validation** | ₹1,010 | ₹1,005 | ₹1,025 | `VALIDATED (v1)` | Passed price & risk checks |
| **State Transition**| ₹1,010 | **₹1,005** | **₹1,025** | `APPLIED (v1)` | Boundaries atomically updated |
| **Market Tick** | ₹1,018 | ₹1,005 | ₹1,025 | None | Active boundaries protect profit at ₹1,005 |
