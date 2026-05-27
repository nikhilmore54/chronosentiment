# ChronoSentiment

ChronoSentiment is a constitutionally governed, replay-certified operational substrate. 
It provides a tightly constrained execution engine designed to guarantee byte-for-byte replay equivalence and strictly isolate operational divergence when that equivalence breaks.

---

## ⏱ The 3-Minute Understanding Path

### Instant Architectural Orientation

| Question | Immediate Answer |
| --- | --- |
| **What is this?** | A replay-certified causal substrate. |
| **What does it guarantee?** | Deterministic replay equivalence across architectures. |
| **What makes it unusual?** | Constitutional authority separation that resists drift. |
| **What is observability allowed to do?** | Evidentiary attestation only (read-only). |
| **What is explicitly forbidden?** | Adaptive governance and semantic mutation. |

### Explicit Non-Claims (What This Is NOT)
To be absolutely clear, ChronoSentiment is **not**:
- A distributed orchestration platform.
- An adaptive AI governance engine.
- Autonomous trading infrastructure.
- A self-optimizing execution intelligence layer.

The repository explicitly denies these surfaces. We enforce bounded legitimacy signaling: the architecture is mechanically sovereign but intentionally avoids semantic autonomy and distributed federation.

---

## 🏛 Authority Geometry

ChronoSentiment enforces a strict presentation-grade directory topology to segregate authority. The most important architectural insight is not how data moves, but **which layer is forbidden from becoming another layer.**

```mermaid
graph TD
    %% Define Top-Level Layers
    subgraph S_INFRA [ /infrastructure (Generic Causal Infrastructure) ]
        Core[Causal Substrate]
        Obs[Evidentiary Observability]
    end
    
    subgraph S_FIN [ /financial (Financial Semantic Realization) ]
        ESE[Financial ESE]
        Strat[Intent Generation]
    end

    %% Allowed Data Flows
    Strat -->|Submits Intents| ESE
    ESE -->|Mutates State deterministically| Core
    Core -->|Emits Replay Traces| Obs

    %% Forbidden Authority (Red Dashed Lines)
    Obs -.->|FORBIDDEN: Adaptive Remediation| Core
    Obs -.->|FORBIDDEN: Semantic Orchestration| Strat
    
    %% Styling
    style Core fill:#1f2937,stroke:#9ca3af,color:#fff
    style Obs fill:#065f46,stroke:#34d399,color:#fff
    style ESE fill:#1e3a8a,stroke:#60a5fa,color:#fff
    style Strat fill:#701a75,stroke:#f472b6,color:#fff
    
    linkStyle 2 stroke:#ef4444,stroke-width:2px,stroke-dasharray: 5 5;
    linkStyle 3 stroke:#ef4444,stroke-width:2px,stroke-dasharray: 5 5;
```

> [!WARNING]
> **Constitutional Constraint:** Directory separation does not imply runtime independence. The topology explicitly exposes the authority boundaries between the Causal Infrastructure and the Financial Realization layer, but both execute entirely inside a single, unified, replay-certified operational appliance. Distributed orchestration is explicitly forbidden until it can earn equal replay legitimacy.

---

## 🛡 Why Replay Certification Exists

Replay certification is not analytics; it is **operational infrastructure.**

- Deterministic replay prevents silent semantic drift.
- Replay equivalence mechanically constrains runtime mutation.
- Observability remains purely evidentiary because replay integrity must remain the absolute authority.
- By forcing the runtime to prove it generates identical state transitions offline, we ensure the core engine remains causally honest.

---

## ⚙️ Operational Demonstration

The system proves its composure through operation, not explanation.
You can execute the entire **Air-Gapped Operational Certification** sequence on your host to mechanically prove artifact sovereignty, compiler independence, and resource scarcity composure.

```bash
# 1. Package the deterministic appliance natively or via cross-compilation
./chrono package

# 2. Extract the read-only operational footprint (no cargo, no git)
./chrono bootstrap

# 3. Mechanically verify execution traces against the unified canonical hash
./chrono release-verify

# 4. Run the bounded offline ingestion smoke tests
./chrono smoke
```

When tested under our strict validation harnesses, this sequence succeeds even in a transient container with `--network none` and `--memory 512m`, proving the appliance relies on mechanical composure rather than environmental abundance.

---

## 📜 The 15-Minute Deep Path

For deep architectural scrutiny, ChronoSentiment is guarded by a constitution that explicitly defines the authority bounds of its components. Review the following non-executable governance documents to understand how the system structurally resists illegitimate evolution.

- **[AUTHORITY_MAP.md](AUTHORITY_MAP.md)**: The holistic authority geography of the entire repository.
- **[Observatory Authority](docs/constitution/OBSERVATORY_AUTHORITY.md)**: Why telemetry must never become governance.
- **[Semantic Realization Gates](docs/constitution/SEMANTIC_REALIZATION_GATES.md)**: Why expansion is gated by operational pressure, not roadmap momentum.
- **[Generic Layer](docs/constitution/GENERIC_LAYER.md)**: Why the substrate preserves causal history rather than "truth."
- **[Operational Sovereignty](docs/constitution/OPERATIONAL_SOVEREIGNTY.md)**: The "appliance philosophy" underpinning offline deployment.

*(For detailed execution instructions, see [Workflow Guide](WORKFLOW_GUIDE.md) and [Divergence Walkthrough](DIVERGENCE_WALKTHROUGH.md).)*
