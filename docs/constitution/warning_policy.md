# Constitutional Warning Policy

To prevent **warning entropy normalization** during infrastructure decomposition, ChronoSentiment classifies warnings into strict categories. This policy ensures that semantic drift or architectural decay cannot hide within compiler output noise.

## Warning Classification

| Category | Description | Action |
| --- | --- | --- |
| **Quarantined Legacy Warning** | Warnings originating from deprecated modules, legacy API boundaries, or intentionally dead code awaiting removal. | **Tolerated** (but tracked) |
| **Unused Extraction Residue** | Leftover variables, imports, or dead branches generated *immediately* following an architectural extraction. | **Temporary** (must be cleared before next phase) |
| **Deterministic Drift Warning** | Any warning suggesting non-deterministic float ordering, hash map iteration instability, or random number generator leakage. | **Critical** (investigate immediately) |
| **Replay Instability** | Warnings from truth harnesses or CI gates regarding test flakiness or state snapshot discrepancies. | **Fatal** (stop migration, revert, recertify) |
| **Dependency Inversion** | Build warnings or lint errors indicating an upward reference against the acyclic layer topology. | **Fatal** (stop migration, revert) |

## Enforcement

- No new extraction zone can be approved if there are unclassified or critical warnings.
- "While we're here" cleanups are discouraged during extraction steps; residue cleanup should be performed as isolated, non-semantic commits.
- Replay truth remains the ultimate arbiter over code elegance.
