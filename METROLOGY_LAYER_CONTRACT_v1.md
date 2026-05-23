# METROLOGY LAYER CONTRACT v1

## 1. Constitutional Authority
This contract formally separates the laboratory's observational instrumentation into two distinct architectural layers: the **Morphology Layer** and the **Metrology Layer**. As the laboratory transitions toward advanced measurement science, it must explicitly defend against "premature measurement ontology"—the assumption that metric projections are physical realities.

## 2. Layer Separation
1. **Morphology Layer:** Generates bounded observational projections (Occupancy, Entropy, Autocorrelation) based on specific representation choices (e.g., linear, squared, binary_thresh). It does NOT assert that these projections are complete, true, or physically intrinsic.
2. **Metrology Layer:** Audits the validity, elasticity, and degeneracy of the Morphology Layer's projections. Its sole purpose is to test when and how the instruments break.

## 3. The Instrument Confidence Mandate
A morphology metric value is considered **DANGEROUS** and **INVALID** if it is not accompanied by a Metrology Confidence Region. 

The Metrology Layer is responsible for defining:
- **Saturation Degeneracy Bounds:** Where variance collapses, rendering metrics like Autocorrelation mathematically undefined.
- **Representation Elasticity:** How sensitive a metric is to arbitrary representation choices (e.g., Entropy is highly representation-sensitive; AC is relatively robust).
- **Horizon Sensitivity:** Whether the metric survives observational rescaling.
- **Implementation Fragility:** The stability of the metric under arbitrary quantization, binning, or lag-window choices.

## 4. Prohibition on Metric Anthropomorphism
The laboratory strictly prohibits language that reifies metrics into intrinsic continuity mechanics. 
- PROHIBITED: "The topology disappeared."
- PERMITTED: "The instrumentation stack effectively shut off due to saturation degeneracy."
- PROHIBITED: "Topology-induced wavelength."
- PERMITTED: "The currently measured lag-decay structure remained relatively stable across the tested representation transforms."

We measure *instrumented deformation observability*. We recognize that the geometry we observe is always partially a product of the representation stack.
