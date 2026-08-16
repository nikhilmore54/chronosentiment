# Search #1 decision-value landscape

Existing 273 recommendations only. Not Search #2. No borderline band is frozen.

- artifact: `9a887827e8f41988987208f13e4ccbac507b3241692026c55f38d11f85971ac0`
- rows: 273
- mean recommended value (all, NO_TRADE=0): 0.002819
- mean regret vs best alternative: 0.049243

| Slice | n | Acted | Stood aside | Mean value | Mean regret | Acted better than NO_TRADE | Unique best |
|-------|---|-------|-------------|------------|-------------|----------------------------|-------------|
| all | 273 | 121 | 152 | 0.002819 | 0.049243 | 57.0% | 25.3% |
| development | 91 | 49 | 42 | 0.007559 | 0.047787 | 59.2% | 31.9% |
| selection | 91 | 39 | 52 | 0.006724 | 0.043759 | 59.0% | 25.3% |
| evaluation | 91 | 33 | 58 | -0.005825 | 0.056182 | 51.5% | 18.7% |

Advantage versus alternatives is observational. It is not Coralys fitness.
Evaluation is diagnostic. Coralys receives no feedback. Search #2 is not authorized.
