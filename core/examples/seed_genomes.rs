use chronosentiment_core::ga::Strategy;
use std::fs;

fn main() -> std::io::Result<()> {
    fs::create_dir_all("genomes")?;
    
    for i in 0..5 {
        let mut strategy = Strategy::from_seed(100 + i);
        // Force various archetypes to ensure diversity
        strategy.archetype = (i % 4) as u8;
        strategy.selectivity = 100; // Always signal if condition met
        
        // Ensure they have enough edge to pass gates
        strategy.base_edge = 1000;
        strategy.queue_threshold = 1000;
        
        let path = format!("genomes/strategy_{}.json", i);
        strategy.save_to_file(&path)?;
        println!("✅ Saved genome to {}", path);
    }
    
    Ok(())
}
