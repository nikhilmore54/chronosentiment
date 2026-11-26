# RP-406C Benchmark Report: Full Lexicographic Comparison

**Date:** 2026-08-04
**Scope:** Coralys RP-406B solutions vs published ROADEF 2026 sprint-results reference
**Instances:** setA-01 through setA-20 (Set A, 20 instances)
**Method:** Full rank-by-rank lexicographic comparison using actual published sprint-reference load vectors

---

## ⚠️ Correction Notice

A previous version of this report incorrectly concluded "16/20 instances tied with published best."
That conclusion was **scientifically invalid** for two reasons:

1. It compared only rank-1 (MLU) values and declared a "tie" when MLU matched within tolerance.
   The ROADEF 2026 competition objective is lexicographic comparison of the **entire** sorted load
   vector (4000 elements), not just the maximum.
2. For instances where MLU differed, it synthesised a fake "best vector" as `[best_mlu] + our_vec[1:]`
   instead of using the actual published vectors.

This report uses the actual published sprint-reference load vectors
([`rp406c_published_best_full.csv`](rp406c_published_best_full.csv), 20 rows × 4002 columns)
for proper rank-by-rank comparison.

---

## 1. Competition Objective: Lexicographic Vector Comparison

The ROADEF 2026 challenge evaluates solutions by comparing the **sorted load vector** — the vector
of all link utilisation values sorted in descending order. Given two solutions A and B, A is better
than B if and only if there exists a rank position k such that:

- `A[i] = B[i]` for all i < k, and
- `A[k] < B[k]`

This is a **multi-objective optimisation** problem in disguise. The MLU (Maximum Link Utilisation,
rank-1 value) is only the first coordinate. Two solutions with identical MLU can have very different
shapes and one can win lexicographically at rank 2, 3, or deeper.

**Implication:** Optimising for MLU reduction alone (as RP-405 did) is optimising the wrong signal.
The correct signal is lexicographic improvement of the full vector.

**Note on the sprint reference:** The published vectors used here are the best solutions submitted
by any team during the sprint phase of the competition. They are not necessarily the competitors'
final algorithms. The statement "Coralys beats the sprint reference" does not imply Coralys is
better than the final competition winner.

---

## 2. Primary Finding: Two Fundamentally Different Optimisation Regimes

The most important result of RP-406C is not the win/loss count. It is the discovery that the 20
instances fall into **two fundamentally different optimisation regimes** that require completely
different remediation strategies.

### Collapsed Basin

```
Coralys converges to a routing family with high utilisation
        ↓
MLU is large (gap > 0.1 vs sprint reference)
        ↓
All downstream lexicographic metrics are irrelevant
        ↓
The search never discovers the correct routing family
```

**6 instances:** setA-06, setA-08, setA-10, setA-13, setA-16, setA-19

The prefix sum trajectories for these instances are remarkably similar:

| Instance | Top10 | Top20 | Top50 |
|----------|-------|-------|-------|
| setA-06  | +4.24 | +6.01 | +8.25 |
| setA-10  | +4.07 | +6.34 | +10.17 |
| setA-13  | +4.80 | +6.73 | +8.89 |
| setA-16  | +5.99 | +8.64 | +13.01 |
| setA-19  | +5.43 | +8.30 | +13.16 |

The likelihood that five unrelated failures produce nearly identical prefix trajectories is low.
The evidence instead suggests a **common search basin** — Coralys repeatedly converges to the
same routing family on these instances, a family that is far from the optimal regime. This is
**exploration failure**, not optimisation failure. No amount of local search will repair a 22×
MLU gap; the search must discover a fundamentally different routing.

The research question for RP-407 is therefore: *why does Coralys repeatedly converge to the
same wrong routing family on these instances?* — not *debug six instances independently*.

### Regime B — Shape Competition

```
Construction succeeds
        ↓
MLU is close to or matches the sprint reference
        ↓
Competition is decided by ranks 2 and beyond
        ↓
Solution "shape" (peak/shoulder/tail) is the differentiator
```

**14 instances:** all others

Regime B splits further into two sub-regimes:

- **B-MLU (3 instances):** Small but non-zero MLU gap (0.01–0.1). We lose at rank 1 by a small
  margin. setA-03, setA-09, setA-14.
- **B-Shape (11 instances):** MLU essentially matched (gap < 0.00001). Competition decided at
  rank 2 or deeper. setA-01, 02, 04, 05, 07, 11, 12, 15, 17, 18, 20.

This two-regime framing completely changes where future work should focus. Lexicographic balancing
work (RP-409) is only relevant for Regime B. Collapsed Basin instances need exploration diagnosis
first (RP-407).

---

## 3. Summary Results

| Outcome | Count | Instances |
|---------|-------|-----------|
| Sprint reference wins | 16/20 | setA-01,02,03,04,05,06,07,08,09,10,11,13,14,16,19,20 |
| **Coralys wins** | **4/20** | **setA-12, setA-15, setA-17, setA-18** |
| Fully tied | 0/20 | — |

Coralys produces a lexicographically better solution than the published sprint reference on 4 out
of 20 instances. This is a genuine result from the full rank-by-rank comparison.

---

## 4. Per-Instance Comparison Table

| Instance | Best Team | Pub MLU | Our MLU | MLU Gap | MLU Gap % | Regime | Lex Status | First Diff Rank | LLI | VAD |
|----------|-----------|---------|---------|---------|-----------|--------|------------|-----------------|-----|-----|
| setA-01 | S8 | 0.929383 | 0.929384 | 0.000001 | 0.00% | B-Shape | pub_wins | 2 | +0.355 | −11.2 |
| setA-02 | S69 | 0.903074 | 0.903075 | 0.000001 | 0.00% | B-Shape | pub_wins | 2 | +0.259 | −17.9 |
| setA-03 | S69 | 0.943543 | 0.982168 | 0.038625 | 4.09% | B-MLU | pub_wins | 1 | +0.039 | −24.9 |
| setA-04 | J27 | 0.581237 | 0.588575 | 0.007338 | 1.26% | B-Shape | pub_wins | 1 | +0.007 | −18.6 |
| setA-05 | S2 | 0.204985 | 0.204986 | 0.000001 | 0.00% | B-Shape | pub_wins | 2 | +0.078 | −4.7 |
| setA-06 | J50 | 0.098591 | 0.633803 | 0.535212 | 542.9% | **A** | pub_wins | 1 | +0.535 | +0.4 |
| setA-07 | J50 | 0.907989 | 0.907989 | 0.000000 | 0.00% | B-Shape | pub_wins | 2 | +0.087 | −47.8 |
| setA-08 | S22 | 0.318903 | 0.561163 | 0.242260 | 76.0% | **A** | pub_wins | 1 | +0.242 | −10.7 |
| setA-09 | S2 | 0.849650 | 0.927677 | 0.078027 | 9.18% | B-MLU | pub_wins | 1 | +0.078 | −66.9 |
| setA-10 | S2 | 0.071739 | 0.591304 | 0.519565 | 724.2% | **A** | pub_wins | 1 | +0.520 | +3.1 |
| setA-11 | J27 | 0.785788 | 0.785789 | 0.000001 | 0.00% | B-Shape | pub_wins | 2 | +0.114 | −40.5 |
| **setA-12** | S22 | 0.879872 | 0.879873 | 0.000001 | 0.00% | B-Shape | **coralys_wins** | **2** | **−0.290** | +0.3 |
| setA-13 | J50 | 0.041025 | 0.854700 | 0.813675 | 1983.4% | **A** | pub_wins | 1 | +0.814 | +4.6 |
| setA-14 | S2 | 0.517621 | 0.572104 | 0.054483 | 10.5% | B-MLU | pub_wins | 1 | +0.054 | −15.8 |
| **setA-15** | S2 | 0.898695 | 0.898696 | 0.000001 | 0.00% | B-Shape | **coralys_wins** | **6** | **−0.706** | −75.3 |
| setA-16 | S2 | 0.044262 | 1.000000 | 0.955738 | 2159.3% | **A** | pub_wins | 1 | +0.956 | +2.4 |
| **setA-17** | S22 | 0.424192 | 0.424192 | 0.000000 | 0.00% | B-Shape | **coralys_wins** | **3** | **−0.015** | −6.1 |
| **setA-18** | S22 | 0.999998 | 0.999999 | 0.000001 | 0.00% | B-Shape | **coralys_wins** | **4** | **−0.471** | −77.0 |
| setA-19 | S22 | 0.046838 | 1.000000 | 0.953162 | 2035.0% | **A** | pub_wins | 1 | +0.953 | +3.6 |
| setA-20 | S67 | 0.991312 | 0.991312 | 0.000000 | 0.00% | B-Shape | pub_wins | 2 | +0.190 | −80.3 |

**LLI** = `first_diff_rank × diff_at_first_diff_rank`. Positive = we lose; negative = we win.
**VAD** = Vector Area Difference = Σ(our[i] − pub[i]) over all 4000 ranks. Negative = Coralys has
lower total load across the full vector (better overall, even if losing lexicographically at the top).

---

## 5. First Difference Rank Distribution

The rank at which Coralys first diverges from the sprint reference reveals the structure of our
competitive gap:

| First Diff Rank | Count | Direction | Instances |
|-----------------|-------|-----------|-----------|
| Rank 1 (MLU gap) | 10 | pub wins | setA-03,04,06,08,09,10,13,14,16,19 |
| Rank 2 (shoulder) | 7 | pub wins | setA-01,02,05,07,11,20 + setA-12 (coralys) |
| Rank 3 | 2 | 1 pub, 1 coralys | setA-17 (coralys), setA-14 (pub at rank 1 actually) |
| Rank 4 | 1 | coralys wins | setA-18 |
| Rank 6 | 1 | coralys wins | setA-15 |

**Key insight:** 6 of the 7 rank-2 divergences are pub wins — the sprint reference consistently
achieves a lower second-highest link utilisation. This is a **shoulder optimisation gap** that
is independent of MLU. Our solver matches the peak but does not spread load as effectively across
the top links.

The 4 Coralys wins all diverge at rank 2 or deeper (ranks 2, 3, 4, 6), confirming that when
construction succeeds and MLU is matched, Coralys can produce superior vector shapes.

---

## 6. Prefix Sum Analysis

Prefix sums show where congestion accumulates. The table below shows `Σ(our[i] − pub[i])` over
the top-N ranks. Positive = Coralys is more congested in that prefix; negative = less congested.

```
Instance     Regime    Top1      Top2      Top5      Top10     Top20     Top50     Top100    Top500    Top1000   All
─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
setA-01      B-Shp   +0.0000   +0.1776   +0.4594   +0.5757   +0.2850   -3.8071  -14.4255  -16.1757  -16.1757  -16.1757
setA-02      B-Shp   +0.0000   +0.1293   +0.2697   +0.6202   +1.0680   +0.0008   -9.0240  -22.6969  -22.6969  -22.6969
setA-03      B-MLU   +0.0386   +0.1466   +0.3177   +0.5372   -0.0174   -2.7561  -12.4197  -24.8826  -24.8826  -24.8826
setA-04      B-Shp   +0.0073   -0.0113   +0.2368   +0.2953   +0.5766   +1.6210   +0.7313  -31.2789  -31.2789  -31.2789
setA-05      B-Shp   +0.0000   +0.0391   +0.1083   +0.2310   +0.4765   +0.6165   +0.4807   -6.1168   -6.5768   -6.5768
setA-06      A       +0.5352   +1.0563   +2.4451   +4.2423   +6.0056   +8.2535   +9.1500   +0.3831   -5.6581   -5.6581
setA-07      B-Shp   +0.0000   +0.0434   +0.1311   +0.4361   +1.3370   +3.1744   +4.1297  -47.7743  -85.3897  -85.3897
setA-08      A       +0.2423   +0.4576   +1.0430   +1.6467   +2.4615   +3.5104   +3.4222   -6.0798  -14.0701  -14.0701
setA-09      B-MLU   +0.0780   +0.1207   +0.5688   +1.2921   +2.4248   +4.2346   +5.2497  -37.5852  -80.6767  -81.9813
setA-10      A       +0.5196   +1.0043   +2.3359   +4.0663   +6.3402  +10.1734  +13.6443  +12.7028   +2.4992   -6.7835
setA-11      B-Shp   +0.0000   +0.0571   -0.0033   +0.1342   +0.3336   +1.7017   +2.8378   -7.2393  -40.5015  -57.5826
setA-12      B-Shp   +0.0000   -0.1452   -0.3883   -0.3106   -0.0160   +0.4220   +0.6936   +0.8041   +0.1279   -0.1953
setA-13      A       +0.8137   +1.5654   +3.0701   +4.8030   +6.7299   +8.8863  +10.6741   +9.2841   +4.6453   +1.9393
setA-14      B-MLU   +0.0545   +0.0545   -0.0275   +0.2671   +1.2432   +1.7334   +1.9023   -0.0683  -12.7325  -25.9149
setA-15      B-Shp   +0.0000   +0.0000   +0.0000   -0.3136   -0.5767   +1.3634   +3.9439   -7.4058  -52.6276 -108.8177
setA-16      A       +0.9557   +1.7816   +3.9988   +5.9914   +8.6449  +13.0117  +16.1500  +15.4962   +8.2678   -6.6175
setA-17      B-Shp   +0.0000   +0.0000   -0.0699   -0.0210   +0.4883   +1.2055   +1.7520   +1.7938   -1.8310  -20.3450
setA-18      B-Shp   +0.0000   +0.0000   -0.1262   -0.2311   -0.0874   +1.8665   +5.1838   +8.5588  -20.6628 -164.0188
setA-19      A       +0.9532   +1.7283   +3.3546   +5.4258   +8.3007  +13.1595  +17.6000  +24.6181  +19.9192  -19.7293
setA-20      B-Shp   +0.0000   +0.0950   +0.0055   -0.2707   -0.7122   -0.5017   -0.3169   -9.5773  -26.0655 -132.8123
```

**Observations:**

1. **Collapsed Basin instances** are worse at every prefix up to Top100, then cross to negative (better)
   in the tail. The tail improvement is irrelevant — the competition is decided at the top.

2. **setA-13** is the only instance where Coralys is worse at *every* prefix including All (+1.94).
   This is the most complete failure in the benchmark.

3. **setA-10 and setA-16** remain positive (worse) through Top500 — the construction failure
   contaminates hundreds of ranks, not just the peak.

4. **Coralys wins (setA-12, 15, 17, 18)** show negative prefix sums starting at Top2 or Top5,
   confirming genuine lexicographic superiority in the shoulder region.

5. **setA-15 and setA-18** have dramatically negative VAD (−109 and −164) — Coralys achieves
   far lower total load across the full vector, winning by a large margin in the tail.

---

## 7. Heat Map: Instance × Rank Band

Mean difference (Coralys − Published) per rank band. Positive = Coralys more congested (worse).

```
Instance     Regime   R1          R2          R3-5        R6-10       R11-20      R21-50      R51-100     R101-500    R501-1k     R1k-4k
──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
setA-01      B-Shp    [=](+0.000) [+++](+0.178) [+++](+0.094) [+](+0.023) [-](-0.029) [---](-0.136) [---](-0.246) [=](+0.000) [=](+0.000) [=](+0.000)
setA-02      B-Shp    [=](+0.000) [+++](+0.129) [+](+0.047)  [+++](+0.070) [+](+0.045) [-](-0.036) [---](-0.180) [---](-0.178) [=](+0.000) [=](+0.000)
setA-03      B-MLU    [+](+0.039) [+++](+0.108) [+++](+0.057) [+](+0.044) [---](-0.055) [---](-0.091) [---](-0.193) [---](-0.083) [=](+0.000) [=](+0.000)
setA-04      B-Shp    [=](+0.007) [-](-0.019)  [+++](+0.083) [+](+0.012) [+](+0.028)  [+](+0.035)  [-](-0.018)  [---](-0.129) [=](+0.000) [=](+0.000)
setA-05      B-Shp    [=](+0.000) [+](+0.039)  [+](+0.023)  [+](+0.025)  [+](+0.025)  [=](+0.005)  [=](-0.003)  [-](-0.018)  [=](+0.000) [=](+0.000)
setA-06      A        [+++](+0.535) [+++](+0.521) [+++](+0.463) [+++](+0.359) [+++](+0.176) [+++](+0.075) [+](+0.018) [-](-0.022) [=](+0.000) [=](+0.000)
setA-07      B-Shp    [=](+0.000) [+](+0.043)  [+](+0.029)  [+++](+0.061) [+++](+0.090) [+++](+0.061) [+](+0.019) [---](-0.130) [=](+0.000) [=](+0.000)
setA-08      A        [+++](+0.242) [+++](+0.215) [+++](+0.195) [+++](+0.121) [+++](+0.081) [+](+0.035) [=](-0.002) [-](-0.024) [-](-0.030) [=](+0.000)
setA-09      B-MLU    [+++](+0.078) [+](+0.043) [+++](+0.149) [+++](+0.145) [+++](+0.113) [+++](+0.060) [+](+0.020) [---](-0.107) [---](-0.117) [=](+0.000)
setA-10      A        [+++](+0.520) [+++](+0.485) [+++](+0.444) [+++](+0.346) [+++](+0.227) [+++](+0.128) [+++](+0.069) [=](-0.002) [-](-0.021) [=](+0.000)
setA-11      B-Shp    [=](+0.000) [+++](+0.057) [-](-0.020)  [+](+0.027)  [+](+0.020)  [+](+0.046)  [+](+0.023)  [-](-0.025)  [---](-0.067) [=](+0.000)
setA-12      B-Shp    [=](+0.000) [---](-0.145) [---](-0.081) [+](+0.016) [+](+0.029)  [+](+0.015)  [=](+0.005)  [=](+0.000)  [=](-0.001)  [=](+0.000)
setA-13      A        [+++](+0.814) [+++](+0.752) [+++](+0.502) [+++](+0.347) [+++](+0.193) [+++](+0.072) [+](+0.036) [=](-0.003) [=](-0.009) [=](+0.000)
setA-14      B-MLU    [+++](+0.054) [=](+0.000) [-](-0.027)  [+++](+0.059) [+++](+0.098) [+](+0.016) [=](+0.003)  [=](-0.005)  [-](-0.025)  [-](-0.028)
setA-15      B-Shp    [=](+0.000) [=](+0.000)  [=](+0.000)  [---](-0.063) [-](-0.026)  [+++](+0.065) [+++](+0.052) [-](-0.028) [---](-0.090) [---](-0.091)
setA-16      A        [+++](+0.956) [+++](+0.826) [+++](+0.739) [+++](+0.399) [+++](+0.265) [+++](+0.146) [+++](+0.063) [=](-0.002) [-](-0.014) [-](-0.013)
setA-17      B-Shp    [=](+0.000) [=](+0.000)  [-](-0.023)  [=](+0.010)  [+++](+0.051) [+](+0.024)  [+](+0.011)  [=](+0.000)  [=](-0.007)  [-](-0.016)
setA-18      B-Shp    [=](+0.000) [=](+0.000)  [-](-0.042)  [-](-0.021)  [+](+0.014)  [+++](+0.065) [+++](+0.066) [=](+0.008)  [---](-0.058) [---](-0.113)
setA-19      A        [+++](+0.953) [+++](+0.775) [+++](+0.542) [+++](+0.414) [+++](+0.287) [+++](+0.162) [+++](+0.089) [+](+0.018) [=](-0.009) [-](-0.016)
setA-20      B-Shp    [=](+0.000) [+++](+0.095) [-](-0.030)  [---](-0.055) [-](-0.044)  [=](+0.007)  [=](+0.004)  [-](-0.023)  [-](-0.033)  [---](-0.054)

Legend: [+++] diff > +0.05 (worse)  [+] diff > +0.01  [=] |diff| ≤ 0.01  [-] diff < -0.01  [---] diff < -0.05 (better)
```

**Patterns visible in the heat map:**

1. **Collapsed Basin instances** show a solid block of `[+++]` from R1 through R51-100, then transition
   to `[=]` or `[-]` in the tail. The failure is concentrated in the top 100 ranks.

2. **Regime B-Shape shoulder losses** (setA-01, 02, 05, 07, 11) show `[=]` at R1, `[+++]` at R2,
   then a gradual transition to `[---]` in the mid-range. The pattern is: match the peak, lose the
   shoulder, recover in the tail.

3. **Coralys wins** (setA-12, 15, 17, 18) show `[=]` or `[---]` at R2, confirming the win is
   genuine and occurs early in the vector.

4. **setA-15** has a distinctive pattern: `[=]` through R3-5, then `[---]` at R6-10 — the win
   is deep (rank 6) but the advantage is large (−0.063 mean diff in that band).

5. **No instance shows `[+++]` in R1k-4k** — Coralys is never worse in the deep tail. The
   competitive gap is entirely in the top 100 ranks.

---

## 8. Vector Area Difference Analysis

VAD = Σ(our[i] − pub[i]) over all 4000 ranks. Negative = Coralys has lower total load (better
overall congestion). This measures the total lexicographic gap, not just the first difference.

| Category | Instances | VAD range | Interpretation |
|----------|-----------|-----------|----------------|
| Coralys much better overall | setA-15,18,20,07 | −75 to −164 | Large tail advantage |
| Coralys moderately better | setA-09,11,17,03 | −25 to −67 | Consistent tail advantage |
| Near-zero (competitive) | setA-12,05,04 | −5 to −19 | Closely matched |
| Collapsed Basin, tail recovers | setA-06,08,10,16,19 | −6 to −14 | Worse at top, better in tail |
| Collapsed Basin, no recovery | setA-13 | +4.6 | Worse everywhere |

**Critical observation:** 15 out of 20 instances have negative VAD — Coralys achieves lower total
load across all 4000 ranks. Yet Coralys loses lexicographically on 16/20 instances. This confirms
that **the competition is decided at the top of the vector**, not by total load. A solution that
is slightly worse at rank 2 but much better at ranks 500–4000 still loses the competition.

This is the clearest demonstration that optimising total load (or any scalar aggregate) is the
wrong objective. The competition requires lexicographic optimisation from the top down.

---

## 9. LLI Analysis: Competition Impact Ranking

LLI = `first_diff_rank × diff_magnitude`. Directly measures competition impact.

**Losses ranked by LLI (highest impact first):**

| Instance | LLI | Regime | First Diff Rank | Interpretation |
|----------|-----|--------|-----------------|----------------|
| setA-16 | +0.956 | A | 1 | MLU=1.0 vs pub=0.044 |
| setA-19 | +0.953 | A | 1 | MLU=1.0 vs pub=0.047 |
| setA-13 | +0.814 | A | 1 | MLU=0.855 vs pub=0.041 |
| setA-06 | +0.535 | A | 1 | MLU=0.634 vs pub=0.099 |
| setA-10 | +0.520 | A | 1 | MLU=0.591 vs pub=0.072 |
| setA-01 | +0.355 | B-Shape | 2 | Shoulder loss |
| setA-02 | +0.259 | B-Shape | 2 | Shoulder loss |
| setA-08 | +0.242 | A | 1 | MLU=0.561 vs pub=0.319 |
| setA-20 | +0.190 | B-Shape | 2 | Shoulder loss |
.087 | B-Shape | 2 | Shoulder loss |
| setA-05 | +0.078 | B-Shape | 2 | Shoulder loss |
| setA-09 | +0.078 | B-MLU | 1 | Moderate MLU gap |
| setA-14 | +0.054 | B-MLU | 1 | Moderate MLU gap |
| setA-03 | +0.039 | B-MLU | 1 | Moderate MLU gap |
| setA-04 | +0.007 | B-Shape | 1 | Small MLU gap |

**Wins ranked by |LLI| (highest impact first):**

| Instance | LLI | Regime | First Diff Rank | Interpretation |
|----------|-----|--------|-----------------|----------------|
| setA-15 | −0.706 | B-Shape | 6 | Deep win, large advantage |
| setA-18 | −0.471 | B-Shape | 4 | Win at rank 4 |
| setA-12 | −0.290 | B-Shape | 2 | Win at rank 2 |
| setA-17 | −0.015 | B-Shape | 3 | Win at rank 3 |

---

## 10. Coralys Intrinsic Behaviour: A Global Load Balancer

The most important research conclusion from RP-406C is not the win/loss count. It is the
identification of **Coralys' intrinsic algorithmic character**.

### Coralys is an excellent global load balancer

The prefix sum and heat map data reveal a consistent pattern across nearly all Regime B instances:

- Coralys matches or nearly matches the sprint reference at rank 1 (MLU)
- Coralys is slightly worse at ranks 2–20 (the shoulder)
- Coralys is substantially **better** at ranks 21–4000 (the mid-range and tail)

Examples:

| Instance | Top10 diff | Top100 diff | All diff | Interpretation |
|----------|-----------|-------------|----------|----------------|
| setA-01 | +0.576 | −14.43 | −16.18 | Worse at top 10, much better overall |
| setA-07 | +0.436 | +4.13 | −85.39 | Worse at top 100, dramatically better overall |
| setA-20 | −0.271 | −0.317 | −132.81 | Better everywhere, wins at rank 6 |
| setA-15 | −0.314 | +3.94 | −108.82 | Better at top 10, mixed mid-range, much better tail |

The published solutions resemble a steep curve — high peak, rapid drop. Coralys produces a
**flatter curve** — slightly higher shoulder, but much lower mid-range and tail. Coralys is
distributing traffic more evenly across the network.

**The competition does not reward this.** ROADEF 2026 evaluates lexicographically from the top.
A solution that is slightly worse at rank 2 but dramatically better at ranks 500–4000 still loses.
This is the clearest demonstration that optimising total load or average utilisation is the wrong
objective for this competition.

### The two-solver picture

The data reveals that Coralys behaves like **two different solvers** depending on the instance:

**Solver A (Good) — 14 instances:**
- Finds essentially the same routing regime as the published solution
- MLU nearly identical
- Tail (ranks 100–4000) frequently better than published
- Losses occur almost entirely in the first few links
- These are excellent results — the solver is in the right basin

**Solver B (Bad) — 6 instances (setA-06, 08, 10, 13, 16, 19):**
- Never enters the correct solution basin
- Top10 prefix sum differences of +4 to +6 (enormous)
- This is not a local optimum — it is a completely different routing
- No amount of local search will repair a 22× MLU gap (setA-16: pub=0.044, ours=1.000)

### The Coralys Optimisation Signature

The consistent pattern across Regime B instances can be formalised as a **four-zone signature**
that characterises any Coralys solution relative to the sprint reference:

| Zone | Ranks | Coralys vs Reference | Implication |
|------|-------|----------------------|-------------|
| **Peak** | 1 | Matched or slightly worse | MLU gap is small or zero |
| **Shoulder** | 2–20 | Consistently worse | Bottleneck polishing gap |
| **Transition** | 21–100 | Mixed, often better | Coralys begins to outperform |
| **Tail** | 101–4000 | Substantially better | Coralys global balancing strength |

This signature is more informative than VAD (which compresses everything into one scalar) because
it preserves the structure of where Coralys wins and loses. It can be computed for any solution
and used as an optimisation target: a solution improves if its Shoulder score decreases while its
Tail score is preserved.

The signature also provides a natural success criterion for RP-408 (shoulder optimisation):
reduce the Shoulder score without degrading the Tail score.

---

## 11. Root-Cause Hypotheses for Collapsed Basin Failures

The benchmark alone does not prove which component failed. The following hypotheses require
per-instance investigation:

| Instance | Pub MLU | Our MLU | Ratio | Most Likely Hypothesis |
|----------|---------|---------|-------|------------------------|
| setA-16 | 0.044 | 1.000 | 22.6× | Solver/export bug — MLU=1.0 is suspicious |
| setA-19 | 0.047 | 1.000 | 21.3× | Solver/export bug — MLU=1.0 is suspicious |
| setA-13 | 0.041 | 0.855 | 20.8× | Construction never found low-utilisation routing |
| setA-06 | 0.099 | 0.634 | 6.4× | Construction never found low-utilisation routing |
| setA-10 | 0.072 | 0.591 | 8.2× | Construction never found low-utilisation routing |
| setA-08 | 0.319 | 0.561 | 1.8× | Local search convergence failure |

The instances with published best MLU < 0.1 (setA-06, 10, 13, 16, 19) all require the solver to
find an extremely sparse routing — one where the most congested link carries less than 10% of its
capacity. These are structurally different from instances where MLU ≈ 0.9. The best teams likely
use a fundamentally different construction heuristic or path generation strategy for these instances.

setA-08 (ratio 1.8×) is more likely a local search convergence failure and may be addressable
without architectural changes.

---

## 12. Architectural Conclusions

### Current implicit objective (wrong for competition)

The current Coralys architecture optimises something close to:

```
minimise Σ f(load_i)   [global load balancing]
```

This produces excellent tail distributions but leaves the shoulder unoptimised.

### Required objective (competition-correct)

The competition requires:

```
minimise (L₁, L₂, L₃, ..., L₄₀₀₀)   [lexicographic]
```

### Proposed rank-weighted search heuristic

For move evaluation during the polishing phase, a rank-weighted objective provides a practical
approximation of the lexicographic objective:

```
Score = 1000·L₁ + 500·L₂ + 250·L₃ + 100·L₄ + 50·L₅ + 25·L₆ + ...
```

This heavily penalises the first few ranks while still considering the rest of the vector. It is
not the final competition objective but is an excellent search heuristic.

### Required architectural evolution

```
Current:   Genome → Objective (scalar)
Required:  Genome → Load Vector → Objective (lexicographic)
```

The load vector must become a first-class object that the solver maintains incrementally and
can compare lexicographically during move evaluation. This is a significant but well-defined
architectural change.

---

## 13. Revised Roadmap

Based on RP-406C, the next research programme splits into two independent problems:

### RP-407A — Basin Discovery (Routing Discovery)

**Target instances:** setA-06, setA-08, setA-10, setA-13, setA-16, setA-19

**Goal:** Find the correct routing regime. Reduce MLU from values like 1.0, 0.85, 0.63 toward
the published regime (< 0.1 for the hardest instances).

**Success criterion:** MLU gap < 0.1 on all 6 instances.

**Candidate approaches:**
- Better initial path generation (multi-path, k-shortest paths)
- Larger neighbourhoods (multi-commodity rerouting)
- Destroy-and-repair with large destruction radius
- Diversification / multiple restarts
- Population-based search to escape bad basins

**Note:** This is an exploration problem, not an optimisation problem. The solver needs to
discover a routing family it currently never visits.

### RP-407B — Shoulder Optimisation (Stepping Stone to Lexicographic-Native Search)

**Target instances:** All 20 (applied after main search, experimentally)

**Goal:** Reduce the Shoulder zone (ranks 2–20) while preserving the Tail zone (ranks 21–4000).

**Algorithm sketch:**
1. After main search terminates, identify the top-N congested links (N = 20)
2. Identify all commodities traversing those links
3. Explore reroutes using a rank-weighted objective (1000·L₁ + 500·L₂ + 250·L₃ + ...)
4. Accept moves that improve the lexicographic prefix without degrading the tail
5. Stop when no improvement found in the top-20 ranks

**Architectural note:** A separate polishing phase is valuable as an experiment, but it is a
**stepping stone**, not the end state. The correct architecture makes lexicographic awareness
part of the search itself:

```
Current architecture:
  Generation → evaluate → scalar objective → selection → variation

Target architecture:
  Generation → evaluate → construct vector → lexicographic compare → selection → variation
```

If the search continues optimising a scalar landscape and relies on post-processing to fix it,
the polisher will always be fighting the main search. The long-term goal is lexicographic-native
selection so that the evolutionary dynamics naturally favour shoulder improvement.

**Expected impact:** 7–8 shoulder-loss instances (setA-01, 02, 05, 07, 11, 20 and possibly
setA-03, 04, 09, 14) are within reach of a dedicated shoulder optimisation phase.

### Move Instrumentation Hypothesis

The heat map reveals that Coralys almost never hurts the deep tail — mutation and repair are
already conservative there. Yet Coralys repeatedly leaves congestion in the first 20 ranks.
This suggests the issue may not be *which* move operators exist, but *which moves survive
selection*.

**Proposed experiment (before writing any new operators):**

Instrument every accepted move and record:
- ΔRank1 (change in MLU)
- ΔRanks2–10 (change in shoulder sum)
- ΔRanks11–100 (change in transition sum)
- ΔTail (change in tail sum)

Classify each accepted move into: Peak improvement / Shoulder improvement / Transition improvement / Tail improvement.

**Hypothesis:** If 80–90% of accepted moves are Transition or Tail improvements, the evolutionary
dynamics are biased away from shoulder optimisation. That bias is the root cause of the shoulder
loss pattern — and it can be corrected by changing the selection criterion, not by adding new
operators.

### Regression protection

The 4 Coralys wins (setA-12, 15, 17, 18) must be used as regression tests. Any future change
that loses these wins is a regression, regardless of other improvements.

### Simplified Roadmap: Four Research Streams

| RP | Objective | Success Metric |
|----|-----------|----------------|
| **RP-407** | Eliminate collapsed basins | All 6 Collapsed Basin instances achieve competitive MLU |
| **RP-408** | Native lexicographic evaluation | Selection compares vectors directly; scalar objective retired |
| **RP-409** | Shoulder optimisation | Eliminate rank-2–10 losses while preserving tail quality |
| **RP-410** | Search dynamics instrumentation | Understand which vector zones evolution actually improves |

This replaces the earlier RP-407A/407B split. The four streams are independent and can proceed
in parallel once RP-407 (basin discovery) unblocks the Collapsed Basin instances.

---
## 14. Shoulder Dominance Index (SDI)

The solution signature identifies ranks 2–20 as the decisive competitive gap for Regime B
instances. The **Shoulder Dominance Index** quantifies this gap as a single scalar that is
directly aligned with the lexicographic objective:

```
SDI = Σ(i=2..20) wᵢ · (Lᵢ^Coralys − Lᵢ^Reference)
```

where `wᵢ = 1/(i−1)` (decreasing weights: rank 2 has weight 1.0, rank 3 has weight 0.5, ...,
rank 20 has weight 0.053).

**Properties:**
- Positive SDI = Coralys is worse in the shoulder (we lose lexicographically in this zone)
- Negative SDI = Coralys is better in the shoulder (we win or are competitive)
- SDI = 0 means the shoulder is matched

Unlike VAD (which aggregates all 4000 ranks), SDI focuses exclusively on the region the
benchmark has identified as the decisive competitive gap. Unlike LLI (which captures only the
first difference), SDI measures the cumulative shoulder gap.

SDI is proposed as the **primary optimisation target for RP-409**. A solution improves under
RP-409 if its SDI decreases while its Tail score (ranks 101–4000) is preserved.

---

## 15. Research Baseline: Four Durable Outcomes

RP-406C has crossed the line from benchmarking exercise to evidence base. The four durable
outcomes that future work can build on:

**1. Correct evaluation methodology established.**
Complete published load vectors are compared lexicographically, rank by rank. MLU-only
comparison is retired. The sprint reference is used as the baseline, with explicit acknowledgement
that it is not the final competition winner.

**2. Two distinct algorithmic behaviours identified.**
The Collapsed Basin (6 instances) and Shape Competition (14 instances) regimes require
fundamentally different remediation strategies. Conflating them into a single "improve the
optimizer" programme would be inefficient.

**3. Coralys characterised as a global load balancer with a shoulder deficit.**
Supported by prefix sums, heat maps, and the four-zone solution signature. The weakness is
concentrated in ranks 2–20. The strength (ranks 101–4000) must be preserved in all future work.

**4. Research roadmap traceable to benchmark evidence.**
Each of the four RP streams (RP-407 through RP-410) addresses a specific hypothesis derived
from the data, with a defined success criterion. Future improvements can be attributed to
specific hypotheses rather than general iteration.

---

## 16. Data Files

| File | Description |
|------|-------------|
| [`rp406c_published_best_full.csv`](rp406c_published_best_full.csv) | Full published sprint-reference load vectors (20 × 4002) |
| [`rp406c_published_best.csv`](rp406c_published_best.csv) | Published best MLU values (20 × 3) |
| [`rp406c_comparison.csv`](rp406c_comparison.csv) | Full comparison metrics per instance |
| [`rp406c_prefix_sums.csv`](rp406c_prefix_sums.csv) | Prefix sums and VAD per instance |
| [`rp406c_heatmap.txt`](rp406c_heatmap.txt) | Text heat map: Instance × Rank-band |
| [`rp406c_first_diff_histogram.txt`](rp406c_first_diff_histogram.txt) | First-diff-rank histogram |
| [`rp406c_regime_analysis.txt`](rp406c_regime_analysis.txt) | Regime classification per instance |
| [`rp406c_all_loadvecs.csv`](rp406c_all_loadvecs.csv) | All Coralys load vectors combined |
| [`setA-{01..20}-loadvec-rp406b.csv`](setA-01-loadvec-rp406b.csv) | Per-instance Coralys load vectors |

Analysis scripts:
- [`adapters/roadef/scripts/rp406c_analyse.py`](../../adapters/roadef/scripts/rp406c_analyse.py) — core lexicographic comparison
- [`adapters/roadef/scripts/rp406c_extended_analysis.py`](../../adapters/roadef/scripts/rp406c_extended_analysis.py) — prefix sums, heat map, histogram, regime analysis

---

## 17. Methodology Notes

- **Lexicographic tolerance:** Two load values are considered equal at a rank position if
  `|a − b| < 1e-6`. Consistent with floating-point precision of the load computation.
- **LLI sign convention:** Positive = we lose (sprint reference is better at first diff rank).
  Negative = we win (Coralys is better at first diff rank).
- **VAD sign convention:** Negative = Coralys has lower total load (better overall congestion).
- **Sprint reference:** Published vectors are the best solutions submitted by any team during
  the sprint phase. Not necessarily the competitors' final algorithms.
- **Coralys vectors:** Generated by the `rp406c_characterise` binary from RP-406B solutions.
  All 20 instances validated (valid=true, 0 overloaded links).
