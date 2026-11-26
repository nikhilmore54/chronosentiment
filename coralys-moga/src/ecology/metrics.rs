pub fn distribution_gini(counts: &[usize]) -> f64 {
    if counts.is_empty() {
        return 0.0;
    }
    let n = counts.len() as f64;
    let mut sorted = counts.to_vec();
    sorted.sort_unstable();

    let sum: usize = sorted.iter().sum();
    if sum == 0 {
        return 0.0;
    }

    let mut diff_sum = 0.0;
    for (i, &val) in sorted.iter().enumerate() {
        diff_sum += (i as f64 + 1.0) * val as f64;
    }

    let _mean = sum as f64 / n;
    (2.0 * diff_sum) / (n * sum as f64) - (n + 1.0) / n
}

pub fn distribution_variance(counts: &[usize]) -> f64 {
    if counts.is_empty() {
        return 0.0;
    }
    let sum: usize = counts.iter().sum();
    let mean = sum as f64 / counts.len() as f64;

    let variance_sum: f64 = counts.iter().map(|&x| (x as f64 - mean).powi(2)).sum();

    variance_sum / counts.len() as f64
}

pub fn compute_pearson(x: &[f64], y: &[f64]) -> f64 {
    if x.len() < 2 {
        return 0.0;
    }
    let mx = x.iter().sum::<f64>() / x.len() as f64;
    let my = y.iter().sum::<f64>() / y.len() as f64;
    let mut cov = 0.0;
    let mut var_x = 0.0;
    let mut var_y = 0.0;
    for (vx, vy) in x.iter().zip(y.iter()) {
        let dx = vx - mx;
        let dy = vy - my;
        cov += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }
    if var_x == 0.0 || var_y == 0.0 {
        return 0.0;
    }
    cov / (var_x * var_y).sqrt()
}

pub fn rank_array(arr: &[f64]) -> Vec<f64> {
    let mut indexed: Vec<(usize, f64)> = arr.iter().copied().enumerate().collect();
    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut ranks = vec![0.0; arr.len()];
    for (rank, (idx, _)) in indexed.iter().enumerate() {
        ranks[*idx] = rank as f64;
    }
    ranks
}

pub fn compute_spearman(x: &[f64], y: &[f64]) -> f64 {
    let rank_x = rank_array(x);
    let rank_y = rank_array(y);
    compute_pearson(&rank_x, &rank_y)
}
