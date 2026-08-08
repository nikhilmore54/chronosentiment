#!/usr/bin/env python3
"""
RP-406C Extended Analysis Script
Generates additional benchmark artifacts:
  1. Prefix sums table (Top-1,2,5,10,20,50,100,500,1000,All) per instance
  2. Vector area difference (L1 sum over all 4000 ranks) per instance
  3. First-difference-rank histogram
  4. Text heat map: Instance × Rank-band (Coralys - Published, colour-coded)

Outputs:
  docs/roadef/rp406c_prefix_sums.csv
  docs/roadef/rp406c_heatmap.txt
  docs/roadef/rp406c_first_diff_histogram.txt
  docs/roadef/rp406c_regime_analysis.txt
"""

import csv
import math
import os
import sys

DOCS_DIR = os.path.join(os.path.dirname(__file__), '..', '..', '..', 'docs', 'roadef')
PUBLISHED_BEST_FULL_CSV = os.path.join(DOCS_DIR, 'rp406c_published_best_full.csv')
LOADVEC_PATTERN = os.path.join(DOCS_DIR, 'setA-{:02d}-loadvec-rp406b.csv')
COMPARISON_CSV = os.path.join(DOCS_DIR, 'rp406c_comparison.csv')

PREFIX_SUMS_CSV = os.path.join(DOCS_DIR, 'rp406c_prefix_sums.csv')
HEATMAP_TXT = os.path.join(DOCS_DIR, 'rp406c_heatmap.txt')
FIRST_DIFF_HIST_TXT = os.path.join(DOCS_DIR, 'rp406c_first_diff_histogram.txt')
REGIME_TXT = os.path.join(DOCS_DIR, 'rp406c_regime_analysis.txt')

PREFIX_TOPS = [1, 2, 5, 10, 20, 50, 100, 500, 1000, 4000]
PREFIX_LABELS = ['Top1', 'Top2', 'Top5', 'Top10', 'Top20', 'Top50', 'Top100', 'Top500', 'Top1000', 'All']

# Heat map rank bands (end-exclusive)
HEATMAP_BANDS = [
    ('R1',    0,    1),
    ('R2',    1,    2),
    ('R3-5',  2,    5),
    ('R6-10', 5,   10),
    ('R11-20',10,  20),
    ('R21-50',20,  50),
    ('R51-100',50,100),
    ('R101-500',100,500),
    ('R501-1k',500,1000),
    ('R1k-4k',1000,4000),
]

LEX_TOL = 1e-6


def load_published_best_full():
    result = {}
    with open(PUBLISHED_BEST_FULL_CSV, newline='') as f:
        reader = csv.reader(f)
        header = next(reader)
        n_ranks = len(header) - 2
        for row in reader:
            if not row or not row[0].strip():
                continue
            instance = row[0].strip()
            team = row[1].strip()
            vec = [float(v) for v in row[2:2 + n_ranks]]
            result[instance] = {'team': team, 'vec': vec}
    return result


def load_our_vector(instance_num):
    path = LOADVEC_PATTERN.format(instance_num)
    vec = []
    with open(path, newline='') as f:
        reader = csv.DictReader(f)
        for row in reader:
            vec.append(float(row['load']))
    return vec


def load_comparison():
    rows = {}
    with open(COMPARISON_CSV, newline='') as f:
        reader = csv.DictReader(f)
        for row in reader:
            rows[row['instance']] = row
    return rows


def prefix_sum(vec, top_n):
    return sum(vec[:min(top_n, len(vec))])


def compute_prefix_sums(our_vec, pub_vec):
    """Compute prefix sums for both vectors and their difference."""
    result = {}
    for label, top_n in zip(PREFIX_LABELS, PREFIX_TOPS):
        our_s = prefix_sum(our_vec, top_n)
        pub_s = prefix_sum(pub_vec, top_n)
        diff = our_s - pub_s  # positive = we are worse (more congested)
        result[f'our_{label}'] = our_s
        result[f'pub_{label}'] = pub_s
        result[f'diff_{label}'] = diff
    return result


def vector_area_difference(our_vec, pub_vec):
    """L1 sum: sum of (our[i] - pub[i]) over all ranks. Positive = we are worse overall."""
    n = min(len(our_vec), len(pub_vec))
    return sum(our_vec[i] - pub_vec[i] for i in range(n))


def band_mean_diff(our_vec, pub_vec, start, end):
    """Mean of (our[i] - pub[i]) over rank band [start, end)."""
    n = min(len(our_vec), len(pub_vec))
    end = min(end, n)
    if start >= end:
        return 0.0
    diffs = [our_vec[i] - pub_vec[i] for i in range(start, end)]
    return sum(diffs) / len(diffs)


def classify_regime(our_mlu, pub_mlu):
    """Classify instance into optimisation regime."""
    gap = our_mlu - pub_mlu
    if gap > 0.1:
        return 'Regime-A: Construction/Search Failure'
    elif gap > 0.01:
        return 'Regime-B-MLU: Moderate MLU Gap'
    else:
        return 'Regime-B-Shape: MLU Matched, Shape Diverges'


def write_prefix_sums(instances_data):
    """Write prefix sums CSV."""
    fieldnames = ['instance', 'pub_team', 'regime', 'vector_area_diff']
    for label in PREFIX_LABELS:
        fieldnames += [f'pub_{label}', f'our_{label}', f'diff_{label}']

    with open(PREFIX_SUMS_CSV, 'w', newline='') as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames, extrasaction='ignore')
        writer.writeheader()
        writer.writerows(instances_data)
    print(f'Wrote {PREFIX_SUMS_CSV}')


def write_heatmap(instances_data):
    """Write text heat map: Instance × Rank-band, showing mean diff per band."""
    lines = []
    lines.append('RP-406C HEAT MAP: Mean(Coralys - Published) per Rank Band')
    lines.append('Positive = Coralys is MORE congested (worse); Negative = Coralys is LESS congested (better)')
    lines.append('Colour: [+++] strongly worse  [+] slightly worse  [=] equal  [-] slightly better  [---] strongly better')
    lines.append('')

    # Header
    band_names = [b[0] for b in HEATMAP_BANDS]
    header = f"{'Instance':12s} {'Regime':8s} " + '  '.join(f'{b:9s}' for b in band_names)
    lines.append(header)
    lines.append('-' * len(header))

    def colour(diff):
        if diff > 0.05:
            return '[+++]'
        elif diff > 0.01:
            return '[ + ]'
        elif diff > -0.01:
            return '[ = ]'
        elif diff > -0.05:
            return '[ - ]'
        else:
            return '[---]'

    for row in instances_data:
        instance = row['instance']
        regime_short = 'A' if 'Regime-A' in row['regime'] else ('B-MLU' if 'B-MLU' in row['regime'] else 'B-Shp')
        cells = []
        for band_name, start, end in HEATMAP_BANDS:
            diff = row.get(f'band_diff_{band_name}', 0.0)
            cells.append(f'{colour(diff):5s}({diff:+.3f})')
        lines.append(f"{instance:12s} {regime_short:8s} " + '  '.join(cells))

    lines.append('')
    lines.append('Legend:')
    lines.append('  [+++] diff > +0.05  (Coralys strongly worse)')
    lines.append('  [ + ] diff > +0.01  (Coralys slightly worse)')
    lines.append('  [ = ] |diff| <= 0.01 (essentially equal)')
    lines.append('  [ - ] diff < -0.01  (Coralys slightly better)')
    lines.append('  [---] diff < -0.05  (Coralys strongly better)')

    with open(HEATMAP_TXT, 'w') as f:
        f.write('\n'.join(lines) + '\n')
    print(f'Wrote {HEATMAP_TXT}')


def write_first_diff_histogram(instances_data, comparison):
    """Write first-difference-rank histogram."""
    lines = []
    lines.append('RP-406C FIRST DIFFERENCE RANK HISTOGRAM')
    lines.append('Shows at which rank position Coralys first diverges from published best.')
    lines.append('')

    # Collect first diff ranks
    rank_counts = {}
    for row in instances_data:
        instance = row['instance']
        cmp = comparison.get(instance, {})
        fdr = cmp.get('lex_first_diff_rank', '')
        winner = cmp.get('lex_winner', '')
        if fdr == '':
            fdr_key = 'tied'
        else:
            fdr_key = int(fdr)
        if fdr_key not in rank_counts:
            rank_counts[fdr_key] = {'pub_wins': [], 'coralys_wins': []}
        if winner == 'pub':
            rank_counts[fdr_key]['pub_wins'].append(instance)
        elif winner == 'coralys':
            rank_counts[fdr_key]['coralys_wins'].append(instance)

    lines.append(f"{'First Diff Rank':18s} {'Count':6s} {'Winner':12s} {'Instances'}")
    lines.append('-' * 80)

    for rank_key in sorted(rank_counts.keys(), key=lambda x: (0, x) if isinstance(x, int) else (1, 0)):
        entry = rank_counts[rank_key]
        for winner_key, instances in [('pub_wins', 'pub wins'), ('coralys_wins', 'coralys wins')]:
            inst_list = entry[winner_key]
            if inst_list:
                bar = '█' * len(inst_list)
                lines.append(f"  rank {str(rank_key):12s} {len(inst_list):4d}   {winner_key:14s} {bar}  {', '.join(inst_list)}")

    lines.append('')
    lines.append('Summary:')
    pub_rank1 = sum(len(v['pub_wins']) for k, v in rank_counts.items() if k == 1)
    pub_rank2 = sum(len(v['pub_wins']) for k, v in rank_counts.items() if k == 2)
    pub_deep = sum(len(v['pub_wins']) for k, v in rank_counts.items() if isinstance(k, int) and k > 2)
    cor_wins = sum(len(v['coralys_wins']) for v in rank_counts.values())
    lines.append(f'  Published wins at rank 1 (MLU gap):      {pub_rank1:2d} instances')
    lines.append(f'  Published wins at rank 2 (shoulder gap):  {pub_rank2:2d} instances')
    lines.append(f'  Published wins at rank 3+ (deep gap):     {pub_deep:2d} instances')
    lines.append(f'  Coralys wins (any rank):                  {cor_wins:2d} instances')

    with open(FIRST_DIFF_HIST_TXT, 'w') as f:
        f.write('\n'.join(lines) + '\n')
    print(f'Wrote {FIRST_DIFF_HIST_TXT}')


def write_regime_analysis(instances_data, comparison):
    """Write regime analysis: two-regime framing."""
    lines = []
    lines.append('RP-406C REGIME ANALYSIS')
    lines.append('=' * 70)
    lines.append('')
    lines.append('PRIMARY FINDING: Two Fundamentally Different Optimisation Regimes')
    lines.append('')
    lines.append('Regime A — Construction/Search Failure:')
    lines.append('  Construction or local search fails to find a low-utilisation routing.')
    lines.append('  MLU is large. All downstream metrics are irrelevant.')
    lines.append('  Diagnosis required before any lexicographic work.')
    lines.append('')
    lines.append('Regime B — Shape Competition:')
    lines.append('  Construction succeeds. MLU is close to or matches published best.')
    lines.append('  Competition is decided by ranks 2+.')
    lines.append('  Two sub-regimes:')
    lines.append('    B-MLU: Small but non-zero MLU gap (we lose at rank 1 by a small margin)')
    lines.append('    B-Shape: MLU matched; we lose (or win) at rank 2 or deeper')
    lines.append('')
    lines.append(f"{'Instance':12s} {'Regime':35s} {'Our MLU':9s} {'Pub MLU':9s} {'Gap':8s} {'First Diff':10s} {'Winner'}")
    lines.append('-' * 100)

    regime_a = []
    regime_b_mlu = []
    regime_b_shape_loss = []
    regime_b_shape_win = []

    for row in instances_data:
        instance = row['instance']
        cmp = comparison.get(instance, {})
        regime = row['regime']
        our_mlu = float(cmp.get('our_mlu', 0))
        pub_mlu = float(cmp.get('pub_mlu', 0))
        gap = float(cmp.get('mlu_gap_abs', 0))
        fdr = cmp.get('lex_first_diff_rank', 'tied')
        winner = cmp.get('lex_winner', '?')

        lines.append(f"{instance:12s} {regime:35s} {our_mlu:9.6f} {pub_mlu:9.6f} {gap:8.6f} {str(fdr):10s} {winner}")

        if 'Regime-A' in regime:
            regime_a.append(instance)
        elif 'B-MLU' in regime:
            regime_b_mlu.append(instance)
        elif winner == 'coralys':
            regime_b_shape_win.append(instance)
        else:
            regime_b_shape_loss.append(instance)

    lines.append('')
    lines.append('REGIME SUMMARY:')
    lines.append(f'  Regime A  (construction/search failure): {len(regime_a):2d} instances — {", ".join(regime_a)}')
    lines.append(f'  Regime B-MLU (small MLU gap):            {len(regime_b_mlu):2d} instances — {", ".join(regime_b_mlu)}')
    lines.append(f'  Regime B-Shape (MLU tied, pub wins):     {len(regime_b_shape_loss):2d} instances — {", ".join(regime_b_shape_loss)}')
    lines.append(f'  Regime B-Shape (MLU tied, we win):       {len(regime_b_shape_win):2d} instances — {", ".join(regime_b_shape_win)}')
    lines.append('')
    lines.append('IMPLICATION:')
    lines.append(f'  {len(regime_a)} instances require construction/search diagnosis (RP-407/408).')
    lines.append(f'  {len(regime_b_mlu) + len(regime_b_shape_loss)} instances require lexicographic balancing (RP-409).')
    lines.append(f'  {len(regime_b_shape_win)} instances already beat the sprint reference — preserve these.')

    with open(REGIME_TXT, 'w') as f:
        f.write('\n'.join(lines) + '\n')
    print(f'Wrote {REGIME_TXT}')


def main():
    print('Loading data...')
    published = load_published_best_full()
    comparison = load_comparison()

    instances_data = []
    for i in range(1, 21):
        instance = f'setA-{i:02d}'
        if instance not in published:
            print(f'WARNING: {instance} not in published best', file=sys.stderr)
            continue
        pub_vec = published[instance]['vec']
        pub_team = published[instance]['team']
        our_vec = load_our_vector(i)
        if not our_vec:
            print(f'WARNING: no load vector for {instance}', file=sys.stderr)
            continue

        cmp = comparison.get(instance, {})
        our_mlu = float(cmp.get('our_mlu', our_vec[0]))
        pub_mlu = float(cmp.get('pub_mlu', pub_vec[0]))

        regime = classify_regime(our_mlu, pub_mlu)
        prefix = compute_prefix_sums(our_vec, pub_vec)
        vad = vector_area_difference(our_vec, pub_vec)

        row = {
            'instance': instance,
            'pub_team': pub_team,
            'regime': regime,
            'vector_area_diff': vad,
        }
        row.update(prefix)

        # Band diffs for heat map
        for band_name, start, end in HEATMAP_BANDS:
            row[f'band_diff_{band_name}'] = band_mean_diff(our_vec, pub_vec, start, end)

        instances_data.append(row)
        print(f'{instance}: regime={regime.split(":")[0]} vad={vad:+.4f} '
              f'diff_Top1={prefix["diff_Top1"]:+.6f} diff_Top10={prefix["diff_Top10"]:+.6f} '
              f'diff_All={prefix["diff_All"]:+.4f}')

    write_prefix_sums(instances_data)
    write_heatmap(instances_data)
    write_first_diff_histogram(instances_data, comparison)
    write_regime_analysis(instances_data, comparison)

    # Print prefix sum table to stdout for report
    print('\n=== PREFIX SUM DIFFERENCES (Coralys - Published, positive = worse) ===')
    print(f"{'Instance':12s} {'Regime':8s} " +
          '  '.join(f'{l:>10s}' for l in PREFIX_LABELS))
    print('-' * 120)
    for row in instances_data:
        regime_short = 'A' if 'Regime-A' in row['regime'] else ('B-MLU' if 'B-MLU' in row['regime'] else 'B-Shp')
        diffs = [row[f'diff_{l}'] for l in PREFIX_LABELS]
        print(f"{row['instance']:12s} {regime_short:8s} " +
              '  '.join(f'{d:+10.4f}' for d in diffs))

    print('\n=== VECTOR AREA DIFFERENCE (sum over all 4000 ranks) ===')
    for row in instances_data:
        print(f"  {row['instance']}: VAD={row['vector_area_diff']:+.4f}")


if __name__ == '__main__':
    main()