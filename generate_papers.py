import json

papers = []

papers.append({
    "title": "Evolutionary Optimization for Airline Crew Scheduling: Re-evaluating the GENCOL Baseline with Coralys",
    "question": "Can a multi-objective genetic algorithm (Coralys) match or exceed the constructive optimization baselines of GENCOL on large-scale crew pairing topologies when subjected to identical regulatory constraints (e.g., layover thresholds)?",
    "domain": "Coralys evolutionary optimization / Airline Crew Scheduling",
    "contribution": "Provides empirical isolation of regulatory constants (8h vs 10h layovers) from algorithmic performance, demonstrating that deterministic chronological grouping strongly dictates pairing topology in dense networks.",
    "files": [
        "docs/research/UltraCrew_Layover_Threshold_Experiment.md",
        "docs/research/UltraCrew_GENCOL_Pipeline_Divergence_Analysis.md",
        "docs/research/UltraCrew_Pairing_Topology_Mutation_Evaluation.md",
        "docs/research/UltraCrew_Coralys_Native_Scheduler_Section3.md"
    ],
    "evidence_available": "End-to-end benchmark data on 7 GERAD instances; precise ablation of 8h vs 10h layovers; mutation operator timing constraints; compliance rate and pairing count ratios.",
    "evidence_missing": "Full multi-objective alignment with GENCOL (TAFB, hotel nights, deadhead proxies) is planned but incomplete; 'landscape_sample' robustness testing.",
    "type": "Empirical / Benchmark-driven",
    "maturity": "Experimentally evaluated",
    "strength": "HIGH"
})

papers.append({
    "title": "Ecological Survivability of Alpha Strategies: Population Dynamics in Continuous State-Space",
    "question": "How do decision strategies (rules) persist, mutate, or degrade when subjected to continuous adversarial topological fragmentation rather than isolated single-scalar backtests?",
    "domain": "Decision ecology / rule ecology",
    "contribution": "Introduces the 'Execution Ecology Specification' and demonstrates that strategy robustness can be quantified via cluster stability and generational persistence under varying visibility topologies.",
    "files": [
        "product_validation/CS-P-006/discovery/20260814T195327Z/ecology/ECOLOGY.md",
        "product_validation/CS-P-006/discovery/20260815T051900Z_c3/rule_ecology/ECOLOGY.md",
        "product_validation/CS-P-005_factor_ecology_v0.1/*",
        "archive/research_outputs/cluster_stability_report.md",
        "docs/research/ECOLOGY_COMPARISON_PROTOCOL_v1.md"
    ],
    "evidence_available": "Live-rule ecology generation data; PCA clustering of ecological fingerprints; survival rates across multiple populations (Search #1 vs Search #2).",
    "evidence_missing": "Longitudinal degradation analysis over extended (multi-year) continuous periods; integration of cross-strategy dependency metrics.",
    "type": "Observational / Mixed",
    "maturity": "Experimentally evaluated",
    "strength": "HIGH"
})

papers.append({
    "title": "Deterministic Point-in-Time Observatories: A Cryptographic Approach to Temporal Integrity",
    "question": "How can chronological simulation environments guarantee causal isolation and prevent semantic leakage (look-ahead bias) during complex historical replays?",
    "domain": "Historical replay methodology / Temporal integrity",
    "contribution": "Proposes a Replay Equivalence Contract and cryptographic execution substrate that strictly bounds historical context, formally separating mechanistic optimization from semantic evaluation.",
    "files": [
        "docs/research/REPLAY_MANIFEST_SPECIFICATION_v1.md",
        "docs/research/REPLAY_EQUIVALENCE_CONTRACT_v1.md",
        "docs/research/CRYPTO_SUBSTRATE_CONTRACT_v1.md",
        "docs/research/TOPOLOGY_PERTURBATION_CONTRACT_v1.md",
        "docs/research/RESEARCH_LOG.md",
        "infrastructure/core/chronology/*"
    ],
    "evidence_available": "Architectural blueprints; latency/dispersion observatory logs; implementation of deterministic replay manifests.",
    "evidence_missing": "Large-scale performance benchmarking of the cryptographic hashing overhead during high-frequency tick simulations.",
    "type": "Systems/architecture",
    "maturity": "Implemented",
    "strength": "MEDIUM"
})

papers.append({
    "title": "Capital Allocation and Universe Robustness in Continuous Portfolio Optimization",
    "question": "To what extent do allocation paradigms (e.g., EqualWeight vs MaxPerSymbol) and universe sizes influence the objective convergence of portfolio strategies over continuous lifecycles?",
    "domain": "Prospective decision validation / Coralys optimization",
    "contribution": "Provides exhaustive empirical ablation of capital constraints and universe subsets against a frozen decision engine, identifying thresholds where structural invariants override alpha signals.",
    "files": [
        "historical_runs/portfolio_v04_allocation_experiment_v3/v04_A_25_equal/REPORT.md",
        "historical_runs/portfolio_v04_allocation_experiment_v3/v04_B_25_max/REPORT.md",
        "historical_runs/portfolio_v03_universe_robustness/v03_C_100/REPORT.md",
        "historical_runs/portfolio_continuous_v021_2026-08-16/continuous_REPORT.md"
    ],
    "evidence_available": "Vast arrays of experimental replay logs; exact causal configuration AB testing; validation outcomes across varying universe sizes (25 vs 50 vs 100 instruments).",
    "evidence_missing": "Direct comparison against external, real-world live trading counterparts (paper trading validation).",
    "type": "Empirical",
    "maturity": "Validated (Historical)",
    "strength": "HIGH"
})

papers.append({
    "title": "Evidence Governance in Algorithmic Decision Systems: A Constitutional Framework",
    "question": "How can large-scale algorithmic execution platforms maintain deterministic provenance and human-interpretable rationale for every atomic decision generated?",
    "domain": "Evidence governance / provenance",
    "contribution": "Formalizes the 'Evidence Sufficiency Matrix' and 'Decision Governance' architecture, demonstrating a working pipeline from abstract intent generation to cryptographic 'G-GATE' certification.",
    "files": [
        "docs/constitution/architecture.md",
        "docs/governance/V006_SERIALIZATION_LAW_DECLARATION.md",
        "docs/research/CS-R-011_Decision_Governance_Research.md",
        "docs/research/CS-R-007_Explainability_Research.md",
        "r3_evidence/20260814T023457Z_B4/G_GATE/*"
    ],
    "evidence_available": "Implementation of G-GATEs; constitutional specifications; end-to-end provenance traces; discrepancy reports ensuring strict UI/Backend boundary separation.",
    "evidence_missing": "Qualitative user-studies verifying that the generated cryptographic rationale actually improves human trust and oversight.",
    "type": "Conceptual / Systems",
    "maturity": "Implemented",
    "strength": "MEDIUM"
})

out = ["# Candidate Research Papers", ""]
for i, p in enumerate(papers, 1):
    out.append(f"## {i}. {p['title']}")
    out.append(f"**Core Research Question:** {p['question']}")
    out.append(f"**Research Domain:** {p['domain']}")
    out.append(f"**Central Contribution:** {p['contribution']}")
    out.append("**Relevant Markdown Files:**")
    for f in p['files']: out.append(f"- `{f}`")
    out.append(f"**Experimental Evidence Already Available:** {p['evidence_available']}")
    out.append(f"**Evidence That Is Still Missing:** {p['evidence_missing']}")
    out.append(f"**Work Type:** {p['type']}")
    out.append(f"**Current Maturity:** {p['maturity']}")
    out.append(f"**Estimated Paper Strength:** {p['strength']}")
    out.append("\n---\n")

out_path = "/Users/nikhil/.gemini/antigravity-ide/brain/56bf6219-8a81-4cb7-bf4a-84ff1f284672/candidate_papers.md"
with open(out_path, "w", encoding="utf-8") as f:
    f.write("\n".join(out))

print("Created artifact.")
