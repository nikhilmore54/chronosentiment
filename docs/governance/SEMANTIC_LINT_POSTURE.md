# Semantic Lint Posture

**Purpose** – Document the intent and operating mode of the semantic linting infrastructure.

## Observation‑Only Policy
- The `semantic_lint.py` script runs in **warning‑only** mode. It **never blocks merges** and does **not modify any repository files**.
- Its sole function is to surface terminology drift, prohibited synonyms, experimental‑term usage, and large‑artifact introductions.
- All warnings must be **reviewed and acknowledged** by a human reviewer in the PR description.

## Human‑Centred Authority
- The glossary (`docs/constitution/glossary.md`) remains the **single source of truth** for constitutional terms. It is **not** automatically updated by any tooling.
- The `link_glossary.py` script is a **manual one‑off utility**; it is **not** executed in CI and therefore does not rewrite code automatically.

## Governance Intent
- This infrastructure provides **visibility** into semantic drift and replay‑adjacent risks while preserving **interpretive authority** with the maintainers.
- Future iterations may transition to **hard‑fail** enforcement, but only after a sustained observation period confirms low false‑positive rates and stable terminology.

## Operational Guidance
1. When a warning appears, the author must explain the context in the PR and, if appropriate, update the glossary manually.
2. Reviewers must verify that the warning is justified and that no unintended semantic changes are introduced.
3. Do **not** merge a PR that relies on the lint warnings as a substitute for proper review.

*This document embodies the principle that tooling supplies telemetry, not authority.*
