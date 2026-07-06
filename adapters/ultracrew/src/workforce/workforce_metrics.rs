/// Calculates variance specifically for weekend assignments.
pub fn weekend_variance(counts: &[usize]) -> f64 {
    if counts.is_empty() {
        return 0.0;
    }
    let sum: usize = counts.iter().sum();
    let mean = sum as f64 / counts.len() as f64;
    
    let variance_sum: f64 = counts.iter()
        .map(|&x| (x as f64 - mean).powi(2))
        .sum();
        
    variance_sum / counts.len() as f64
}
