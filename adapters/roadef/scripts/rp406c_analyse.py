#!/usr/bin/env python3
"""
RP-406C Analysis Script — Full rank-by-rank lexicographic comparison
Coralys RP-406B solutions vs published ROADEF 2026 sprint-results best.

METHODOLOGY:
  The ROADEF 2026 competition objective is lexicographic comparison of the
  FULL sorted load vector (4000 elements), NOT just MLU (rank-1).

  This script performs proper rank-by-rank comparison using the actual
  published best load vectors from the sprint results leaderboard.

  Metrics computed per instance:
    - lex_status: who wins the lexicographic comparison
    - lex_first_diff_rank: first rank position where vectors differ (1-indexed)
    - lex_winner: 'pub' | 'coralys' | 'tied'
    - mlu_diff: our_mlu - pub_mlu (positive = we are worse at rank 1)
    - mlu_gap_rel_pct: relative MLU gap as percentage
    - l1_dist: sum of |our[i] - pub[i]| over all ranks
    - l2_dist: sqrt(sum of (our[i] - pub[i])^2)
    - rmse: l2_dist / sqrt(n)
    - cosine_sim: dot(our, pub) / (|our| * |pub|)
    - max_dev: max |our[i] - pub[i]|
    - max_dev_rank: rank position of max deviation
    - prefix_match_len: number of leading ranks where |diff| < tol
    - coralys_better_count: ranks where our[i] < pub[i] (we are better)
    - pub_better_count: ranks where pub[i] < our[i] (pub is better)
    - tied_count: ranks where |diff| < tol
    - lli: Lexicographic Loss Index = first_diff_rank * |diff at first_diff_rank|
           (0 if tied, positive if pub wins, negative if coralys wins)
    - Shape metrics on our vector: shoulder_ratio, tail_ratio, band means

OUTPUT:
  docs/roadef/rp406c_comparison.csv
"""

import csv
import math
import os
import sys

DOCS_DIR = os.path.join(os.path.dirname(__file__), '..', '..', '..', 'docs', 'roadef')
PUBLISHED_BEST_FULL_CSV = os.path.join(DOCS_DIR, 'rp406c_published_best_full.csv')
LOADVEC_PATTERN = os.path.join(DOCS_DIR, 'setA-{:02d}-loadvec-rp406b.csv')
OUTPUT_CSV = os.path.join(DOCS_DIR, 'rp406c_comparison.csv')

# Tolerance for declaring two load values equal at a rank position
LEX_TOL = 1e-6


def load_published_best_full():
    """
    Load full published best load vectors from wide CSV.
    Returns dict: instance_name -> {'team': str, 'vec': list[float]}
    The wide CSV has columns: Instance, Best team, 1, 2, ..., 4000
    """
    result = {}
    with open(PUBLISHED_BEST_FULL_CSV, newline='') as f:
        reader = csv.reader(f)
        header = next(reader)
        # header[0]='Instance', header[1]='Best team', header[2]='1', ...
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
    """Load our Coralys load vector. Returns list of floats (rank 1..4000)."""
    path = LOADVEC_PATTERN.format(instance_num)
    vec = []
    with open(path, newline='') as f:
        reader = csv.DictReader(f)
        for row in reader:
            vec.append(float(row['load']))
    return vec  # already sorted descending by rank


def lex_compare(our_vec, pub_vec, tol=LEX_TOL):
    """
    Full rank-by-rank lexicographic comparison.
    Returns (status, first_diff_rank, winner)
      status: 'pub_wins' | 'coralys_wins' | 'tied'
      first_diff_rank: 1-indexed rank of first difference (None if tied)
      winner: 'pub' | 'coralys' | 'tied'
    """
    n = min(len(our_vec), len(pub_vec))
    for i in range(n):
        diff = our_vec[i] - pub_vec[i]
        if diff > tol:
            # our value is higher (worse) at this rank
            return 'pub_wins', i + 1, 'pub'
        elif diff < -tol:
            # our value is lower (better) at this rank
            return 'coralys_wins', i + 1, 'coralys'
    return 'tied', None, 'tied'


def compute_distance_metrics(our_vec, pub_vec):
    """Compute full-vector distance metrics between our and published vectors."""
    n = min(len(our_vec), len(pub_vec))
    diffs = [our_vec[i] - pub_vec[i] for i in range(n)]
    abs_diffs = [abs(d) for d in diffs]

    l1 = sum(abs_diffs)
    l2 = math.sqrt(sum(d * d for d in diffs))
    rmse = l2 / math.sqrt(n) if n > 0 else 0.0

    dot = sum(our_vec[i] * pub_vec[i] for i in range(n))
    norm_our = math.sqrt(sum(v * v for v in our_vec[:n]))
    norm_pub = math.sqrt(sum(v * v for v in pub_vec[:n]))
    cosine_sim = dot / (norm_our * norm_pub) if norm_our > 0 and norm_pub > 0 else 0.0

    max_dev = max(abs_diffs) if abs_diffs else 0.0
    max_dev_rank = abs_diffs.index(max_dev) + 1 if abs_diffs else 0

    prefix_match = 0
    for d in abs_diffs:
        if d < LEX_TOL:
            prefix_match += 1
        else:
            break

    coralys_better = sum(1 for d in diffs if d < -LEX_TOL)
    pub_better = sum(1 for d in diffs if d > LEX_TOL)
    tied_count = n - coralys_better - pub_better

    return {
        'l1_dist': l1,
        'l2_dist': l2,
        'rmse': rmse,
        'cosine_sim': cosine_sim,
        'max_dev': max_dev,
        'max_dev_rank': max_dev_rank,
        'prefix_match_len': prefix_match,
        'coralys_better_count': coralys_better,
        'pub_better_count': pub_better,
        'tied_count': tied_count,
    }


def compute_shape_metrics(vec, label='our'):
    """Compute shape metrics describing peak/shoulder/tail structure."""
    n = len(vec)
    if n == 0:
        return {}

    mlu = vec[0]

    def band_mean(start, end):
        band = vec[start:min(end, n)]
        return sum(band) / len(band) if band else 0.0

    shoulder_ratio = vec[99] / mlu if mlu > 0 and n >= 100 else 0.0
    tail_ratio = vec[999] / mlu if mlu > 0 and n >= 1000 else 0.0

    return {
        f'{label}_mlu': mlu,
        f'{label}_mean_load': sum(vec) / n,
        f'{label}_band_top10_mean': band_mean(0, 10),
        f'{label}_band_top100_mean': band_mean(0, 100),
        f'{label}_band_top1000_mean': band_mean(0, 1000),
        f'{label}_band_tail_mean': band_mean(1000, n),
        f'{label}_shoulder_ratio': shoulder_ratio,
        f'{label}_tail_ratio': tail_ratio,
    }


def analyse_instance(instance, pub_team, pub_vec, our_vec):
    """Full analysis of one instance. Returns dict of all metrics."""
    our_mlu = our_vec[0] if our_vec else float('nan')
    pub_mlu = pub_vec[0] if pub_vec else float('nan')
    mlu_diff = our_mlu - pub_mlu
    mlu_gap_abs = abs(mlu_diff)
    mlu_gap_rel_pct = (mlu_gap_abs / pub_mlu * 100) if pub_mlu > 0 else float('nan')

    lex_status, first_diff_rank, lex_winner = lex_compare(our_vec, pub_vec)

    dist = compute_distance_metrics(our_vec, pub_vec)

    # LLI: Lexicographic Loss Index
    # = first_diff_rank * |diff at first_diff_rank|
    # Positive if pub wins (we lose), negative if coralys wins, 0 if tied
    if first_diff_rank is not None:
        diff_at_first = our_vec[first_diff_rank - 1] - pub_vec[first_diff_rank - 1]
        lli = first_diff_rank * diff_at_first  # positive = we lose
    else:
        lli = 0.0

    our_shape = compute_shape_metrics(our_vec, 'our')
    pub_shape = compute_shape_metrics(pub_vec, 'pub')

    row = {
        'instance': instance,
        'pub_team': pub_team,
        'pub_mlu': pub_mlu,
        'our_mlu': our_mlu,
        'mlu_diff': mlu_diff,
        'mlu_gap_abs': mlu_gap_abs,
        'mlu_gap_rel_pct': mlu_gap_rel_pct,
        'lex_status': lex_status,
        'lex_first_diff_rank': first_diff_rank if first_diff_rank is not None else '',
        'lex_winner': lex_winner,
        'lli': lli,
    }
    row.update(dist)
    row.update(our_shape)
    row.update(pub_shape)
    return row


FIELDNAMES = [
    'instance', 'pub_team', 'pub_mlu', 'our_mlu',
    'mlu_diff', 'mlu_gap_abs', 'mlu_gap_rel_pct',
    'lex_status', 'lex_first_diff_rank', 'lex_winner', 'lli',
    'l1_dist', 'l2_dist', 'rmse', 'cosine_sim',
    'max_dev', 'max_dev_rank', 'prefix_match_len',
    'coralys_better_count', 'pub_better_count', 'tied_count',
    'our_mlu', 'our_mean_load',
    'our_band_top10_mean', 'our_band_top100_mean',
    'our_band_top1000_mean', 'our_band_tail_mean',
    'our_shoulder_ratio', 'our_tail_ratio',
    'pub_mlu', 'pub_mean_load',
    'pub_band_top10_mean', 'pub_band_top100_mean',
    'pub_band_top1000_mean', 'pub_band_tail_mean',
    'pub_shoulder_ratio', 'pub_tail_ratio',
]

# Deduplicate fieldnames (pub_mlu and our_mlu appear twice above)
seen = set()
FIELDNAMES_DEDUP = []
for f in FIELDNAMES:
    if f not in seen:
        FIELDNAMES_DEDUP.append(f)
        seen.add(f)


def main():
    print('Loading published best full vectors...')
    published = load_published_best_full()
    print(f'  Loaded {len(published)} instances from published best CSV')

    rows = []
    for i in range(1, 21):
        instance = f'setA-{i:02d}'
        if instance not in published:
            print(f'WARNING: {instance} not in published best CSV', file=sys.stderr)
            continue
        pub_info = published[instance]
        pub_team = pub_info['team']
        pub_vec = pub_info['vec']
        our_vec = load_our_vector(i)
        if not our_vec:
            print(f'WARNING: no load vector for {instance}', file=sys.stderr)
            continue

        row = analyse_instance(instance, pub_team, pub_vec, our_vec)
        rows.append(row)

        fdr = row['lex_first_diff_rank']
        fdr_str = f'rank={fdr}' if fdr != '' else 'all-tied'
        print(f'{instance}: pub_mlu={row["pub_mlu"]:.6f} our_mlu={row["our_mlu"]:.6f} '
              f'gap={row["mlu_gap_abs"]:.6f} ({row["mlu_gap_rel_pct"]:.2f}%) '
              f'lex={row["lex_status"]} first_diff={fdr_str} lli={row["lli"]:.6f}')

    with open(OUTPUT_CSV, 'w', newline='') as f:
        writer = csv.DictWriter(f, fieldnames=FIELDNAMES_DEDUP, extrasaction='ignore')
        writer.writeheader()
        writer.writerows(rows)

    print(f'\nWrote {len(rows)} rows to {OUTPUT_CSV}')

    # Summary
    pub_wins = [r for r in rows if r['lex_status'] == 'pub_wins']
    coralys_wins = [r for r in rows if r['lex_status'] == 'coralys_wins']
    tied = [r for r in rows if r['lex_status'] == 'tied']

    print(f'\n=== LEXICOGRAPHIC COMPARISON SUMMARY ===')
    print(f'Published best wins:  {len(pub_wins):2d}/20')
    print(f'Coralys wins:         {len(coralys_wins):2d}/20')
    print(f'Fully tied:           {len(tied):2d}/20')

    if pub_wins:
        print(f'\nInstances where published best wins (sorted by LLI desc):')
        for r in sorted(pub_wins, key=lambda x: -x['lli']):
            fdr = r['lex_first_diff_rank']
            print(f'  {r["instance"]}: first_diff=rank {fdr}, '
                  f'mlu_gap={r["mlu_gap_abs"]:.6f} ({r["mlu_gap_rel_pct"]:.1f}%), '
                  f'LLI={r["lli"]:.4f}')

    if coralys_wins:
        print(f'\nInstances where Coralys wins:')
        for r in coralys_wins:
            fdr = r['lex_first_diff_rank']
            print(f'  {r["instance"]}: first_diff=rank {fdr}, LLI={r["lli"]:.4f}')

    if tied:
        print(f'\nFully tied instances (identical vectors within tol={LEX_TOL}):')
        for r in tied:
            print(f'  {r["instance"]}')

    # Large MLU gap instances
    large_gap = [r for r in rows if r['mlu_gap_abs'] > 0.1]
    if large_gap:
        print(f'\nLarge MLU gap instances (gap > 0.1) — likely algorithmic failures:')
        for r in sorted(large_gap, key=lambda x: -x['mlu_gap_abs']):
            print(f'  {r["instance"]}: pub={r["pub_mlu"]:.6f} our={r["our_mlu"]:.6f} '
                  f'gap={r["mlu_gap_abs"]:.6f} ({r["mlu_gap_rel_pct"]:.0f}%)')


if __name__ == '__main__':
    main()