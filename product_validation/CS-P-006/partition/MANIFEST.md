# Chronological dataset partition — CS-P-006-B.1

**Authorization:** PASS

Domain kinds: **development / selection / evaluation**.  
Protocol mapping (provenance only): development = TRAIN, selection = VALIDATION, evaluation = TEST.

`.cursor/rules/chronosentiment-core.mdc`: same inputs → same partition; evaluate across instruments; no invented year folds.

| Field | Value |
|-------|--------|
| Method | `chronological_partition.contiguous_equal_thirds.v1` |
| Atomic unit | timestamp (all seven instruments move together) |
| Timestamps | 39 |
| Instruments per timestamp | 7 |
| Observations | 273 |
| Tie-break | none_applicable (39 = 3 × 13) |
| Partition hash | `4354c81ef546003b1d11ec98cba83dd5f8c56b13c8b6055b8451614abdc4cfca` |
| Snapshot identity | `c21ec256133fb63656b35e68c5e1e72b72751ad2fb45f11c12f99ddb34a628c6` |

| Partition | Protocol role | Timestamps | Observations | First | Last | Inclusive start | Exclusive end |
|-----------|---------------|------------|--------------|-------|------|-----------------|---------------|
| development | TRAIN | 13 | 91 | 2021-10-31 15:30 UTC | 2022-10-31 15:30 UTC | 2021-10-31T15:30:00Z | 2022-10-31T15:30:01Z |
| selection | VALIDATION | 13 | 91 | 2022-11-30 15:30 UTC | 2023-11-30 15:30 UTC | 2022-11-30T15:30:00Z | 2023-11-30T15:30:01Z |
| evaluation | TEST | 13 | 91 | 2023-12-31 15:30 UTC | 2024-12-31 15:30 UTC | 2023-12-31T15:30:00Z | 2024-12-31T15:30:01Z |

Sealed `PolicyArtifact` windows use `exclusive_end` of development = selection start, and `exclusive_end` of selection = evaluation start, so TRAIN < VALIDATION < TEST with no overlap.

## Search visibility

| Partition | Outcomes | Performance | Fitness |
|-----------|----------|-------------|---------|
| development | evolution | evolution | evolution |
| selection | selection feedback only | selection feedback only | **not** for further evolution |
| evaluation | **forbidden** | **forbidden** | **forbidden** |

Coralys must never see evaluation outcomes, evaluation performance, or evaluation-derived fitness.

Timestamp lists: `development_timestamps.txt`, `selection_timestamps.txt`, `evaluation_timestamps.txt`.
Machine manifest: `manifest.json`.
