# ChronoSentiment User Guide

Welcome to **ChronoSentiment**, an execution observability and pre-deployment validation platform. 

This guide will help you navigate the UI and understand how to interpret the data it presents.

## Core Concept: Why ChronoSentiment Exists

Most platforms focus on telling you if your strategy is profitable in theory. ChronoSentiment focuses on answering a different question: **Can your strategy survive real-world execution?**

When you load data into ChronoSentiment, it compares two realities:
1. **The Baseline Path:** How your strategy behaves in a perfect, frictionless world.
2. **The Perturbed Path:** How your strategy behaves when exposed to real-world friction (latency, missed fills, queue exhaustion).

If those paths split, you have **Divergence**. ChronoSentiment's entire UI is designed to help you identify exactly where, how, and why that divergence happened.

---

## The Three Views

The interface is broken down into three core tabs. You should generally navigate them in order from left to right: from the highest level impact down to the microscopic details.

### 1. Analytics Dashboard (The Impact)
**Goal: Understand the high-level damage to your strategy.**

When you first open the app, start here. It summarizes the overall structural impact without forcing you to look at individual trades.
- **Execution Realism Panel:** Shows raw operational friction. Look for high *Avg Delay Ticks* or *Missed Fills*.
- **Divergence Profile Panel:** Shows how much the strategy's structure decayed. 
  - *Sequence Fidelity:* What percentage of the execution sequence matched the ideal baseline perfectly.
  - *Structural Divergence:* What percentage of time your portfolio held the completely wrong position.
  - *Propagation Depth:* How many downstream "cascade events" were triggered by one initial failure.
- **Explanation Summary Panel:** The most important box. It immediately isolates the *Primary Cause* (e.g., `ENTRY_DRIFT`) and the exact tick where the simulation broke. 

> [!TIP]
> If your *Sequence Fidelity* is 100%, your strategy survived execution perfectly. You don't need to look any further.

### 2. Timeline View (The Propagation)
**Goal: Visually trace the cascade of failure.**

If the Analytics Dashboard tells you *what* went wrong, the Timeline shows you *when* and *how* it propagated. 

The Timeline has 4 synchronized lanes:
- **Market Lane:** The underlying price ticks.
- **Signal Lane:** When your strategy decided to act.
- **Execution Lane:** When the order was actually filled (Baseline vs Perturbed).
- **Portfolio Lane:** Your actual capital exposure (Baseline vs Perturbed).

**How to read it:**
Scroll through the timeline until you see a split where the `[BL]` (Baseline) and `[PT]` (Perturbed) paths no longer match. The Timeline explicitly tags "Cascade Events" (like `DELAYED_FILL` or `EXPOSURE_OFFSET_BEGIN`) exactly where they occurred. 

### 3. Trade Inspector (The Specifics)
**Goal: Deep dive into a single broken trade.**

Once you've identified a problematic area on the Timeline, use the Trade Inspector to dissect the specific trade.
- Select a trade from the dropdown menu (e.g., `Trade T1`).
- The Inspector breaks the trade down into the **Decision Layer** (when was it generated?), the **Execution Layer** (how did the fill compare?), and the **Outcome Layer** (did it diverge?).
- **Explanation Panel:** This panel triggers plain-English rules explaining exactly why the trade failed (e.g., *"Missed fills increased due to queue exhaustion"*).

---

## Example Workflow

1. **Load:** You run a momentum strategy through the simulator with a 50ms latency injection.
2. **Observe (Analytics):** You open the UI. The Analytics Dashboard flashes red. Sequence Fidelity is 60%. The Primary Cause says `QUEUE_EXHAUSTION`. 
3. **Trace (Timeline):** You switch to the Timeline tab. You see that at Tick 120, your signal fired, but the Execution Lane shows `[PT: MISSED]`. The Portfolio Lane splits heavily.
4. **Inspect (Trade Inspector):** You open the Trade Inspector, select the trade from Tick 120, and read the exact rules triggered that caused the queue exhaustion.
5. **Decide:** You now know your momentum strategy cannot survive a 50ms latency environment. You optimize your deployment accordingly.
