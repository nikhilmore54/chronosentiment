# Coralys — Platform Strategy

**Document type:** Platform Strategy
**Version:** 1.1
**Status:** Baseline
**Date:** 2026-07-26
**Owner:** Strategy / Product

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | Baseline v1.1 |
| Review Trigger | Material change in product portfolio; new domain adapter launched; go-to-market strategy revision |

**Relationship to other documents:**
- Informed by: `CORALYS_PLATFORM_ARCHITECTURE.md` (platform architecture — primitives, lifecycle, Continuous Learning Engine)
- Informs: `ChronoSentiment_Product_Strategy_v1.md` (ChronoSentiment product positioning)
- Informs: `PRODUCT_BLUEPRINT.md` (product development strategy)
- Complementary to: `PRODUCT_STRATEGY.md` (platform positioning, product hierarchy, go-to-market)

---

## Purpose

This document defines how Coralys is positioned commercially — how the platform relates to the products built on it, what each product sells, and how the platform's capabilities are exposed (or not) to customers. It is a strategy document, not an architecture document. The architecture is defined in `CORALYS_PLATFORM_ARCHITECTURE.md`.

---

## Platform vs Product

Coralys is primarily an enabling platform. Most customers interact with Coralys through domain-specific products built upon it. The platform provides the lifecycle, governance, and knowledge evolution capabilities. The products provide the domain vocabulary, the user experience, and the commercial proposition.

This separation is intentional and durable:

- The platform can evolve without changing the product's commercial identity.
- The product can evolve without changing the platform's architecture.
- New products can be built on the platform without modifying the platform's core.

---

## Platform Hierarchy

```
                    Coralys
         Knowledge Evolution Platform
                    │
        ┌───────────┼───────────┐
        │           │           │
        ▼           ▼           ▼
  UltraCrew   ChronoSentiment   Future
              Enterprise        products
        │           │
        ▼           ▼
 Workforce      Financial
 Decision       Decision
 Engine         Intelligence
                Platform
```

Coralys is never the product the customer sees first. It is the platform that makes the product defensible.

---

## Three-Layer Architecture

| Layer | Identity | Audience |
|-------|----------|----------|
| **Coralys** | Knowledge Evolution Platform | Platform architects, engineering teams, future solution builders |
| **Products** | Decision Engines / Knowledge Platform | Customers |
| **Continuous Learning Engine** | Core computational capability | Engineers and platform developers |

---

## Product Identities

### UltraCrew — Workforce Decision Engine

**What the customer buys:** Better schedules, better disruption recovery, better operational decisions.

**What Coralys does invisibly:** Every disruption, every recovery, every manual override, and every optimisation becomes operational knowledge. The Knowledge Graph accumulates across scheduling cycles. Future schedules benefit from past experience. The platform is hidden. The decision quality is visible.

**Commercial positioning:** Workforce Decision Engine. The knowledge layer is not front and centre — it is the engine behind the decisions.

---

### ChronoSentiment Enterprise — Financial Decision Intelligence Platform

**What the customer buys:** Better investment decisions, governance, explainability, and institutional memory.

**What Coralys does invisibly:** Every investment thesis, every committee review, every outcome, and every lesson becomes organisational knowledge. The Knowledge Graph accumulates across decision cycles. Future decisions benefit from past experience. The platform is hidden. The decision quality is visible.

**Commercial positioning:** Financial Decision Intelligence Platform. The knowledge layer surfaces as "institutional memory" and "explainability" — customer-facing benefits rather than platform capabilities.

---

### ChronoSentiment Personal — Personal Investment Knowledge Platform

**What the customer buys:** A better way to build, organise, and improve their own investment knowledge over time.

**What Coralys does visibly:** Research workspaces, hypothesis versioning, evidence management, quarterly reviews, research timelines, and the Personal Investment Knowledge Graph are all front and centre. The customer's primary objective is knowledge evolution — not just decision quality.

**Commercial positioning:** Personal Investment Knowledge Platform. This is the exception in the portfolio — the one product where exposing the platform's knowledge-centric nature strengthens the product story rather than complicating it.

**Why this product is different:** An individual investor is explicitly trying to build knowledge. The Research Dossier, the Investment Thesis, the Research Timeline, and the Personal Investment Learning Loop are all things the customer actively wants to see and use. The platform's capabilities are the product's features.

---

### Future Products — Decision Engines in Their Domains

Clinical decision engine. Engineering decision engine. Corporate strategy decision engine. M&A decision engine.

In each case, the pattern is the same as UltraCrew and ChronoSentiment Enterprise: Coralys evolves knowledge invisibly. The product delivers decision quality visibly. The platform is hidden. The domain expertise is front and centre.

---

## Portfolio Principle

| Product | Customer buys | Coralys role | Platform visibility |
|---------|--------------|--------------|---------------------|
| UltraCrew | Better operational decisions | Enabling platform | Hidden |
| ChronoSentiment Enterprise | Better investment decisions | Enabling platform | Hidden |
| Future: Clinical | Better clinical decisions | Enabling platform | Hidden |
| Future: Engineering | Better engineering decisions | Enabling platform | Hidden |
| ChronoSentiment Personal | Better personal knowledge | Enabling platform | **Visible** |

Knowledge Evolution Platform is the right identity for Coralys. Decision Engine is the right identity for most commercial products built on it. ChronoSentiment Personal is the one product where the platform's knowledge-centric nature should be front and centre — because the customer's goal is knowledge evolution, not just decision quality.

---

## Licensing and OEM Considerations

Coralys is primarily an enabling platform today. In the future, if Coralys is licensed to partners or OEMs, or if industry-specific solutions are built on top of it, the platform may be sold in some form directly. This document does not constrain that possibility. The platform strategy should be revisited when the first licensing or OEM conversation becomes material.

---

*Coralys Platform Strategy v1.1 | July 2026 | Status: Baseline*
*v1.0: Initial document — product portfolio positioning, product identities (UltraCrew, ChronoSentiment Enterprise, ChronoSentiment Personal), platform hierarchy, portfolio principle, licensing considerations.*
*v1.1: Promoted from Draft to Baseline. Content confirmed against platform architecture review (9.9–10/10). No structural changes — architecture review validated product identities, portfolio principle, and platform/product separation as correct.*
*Review trigger: Material change in product portfolio; new domain adapter launched; go-to-market strategy revision.*