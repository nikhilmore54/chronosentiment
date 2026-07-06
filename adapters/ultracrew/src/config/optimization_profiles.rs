// UltraCrew optimisation profiles – translate high‑level policies to the generic EvolutionConfig

use coralys_moga::config::{EvolutionConfig, EvolutionConfig as Config};

/// UltraCrew optimisation profiles.
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OptimizationProfile {
    Fast,
    Balanced,
    Thorough,
    Research,
}

impl OptimizationProfile {
    /// One‑line description for the CLI list output
    pub fn description(&self) -> &'static str {
        match self {
            OptimizationProfile::Fast => "Quick interactive demonstrations",
            OptimizationProfile::Balanced => "Recommended for everyday scheduling",
            OptimizationProfile::Thorough => "Longer optimization for maximum schedule quality",
            OptimizationProfile::Research => "Manual parameter tuning and experimentation",
        }
    }
    /// Return a fully populated EvolutionConfig for the selected profile.
    pub fn config(&self) -> EvolutionConfig {
        match self {
            OptimizationProfile::Fast => EvolutionConfig {
                population_size: 50,
                generation_limit: 50,
                elite_count: 2,
                mutation_rate: 0.6,
                crossover_rate: 0.8,
                seed: Some(42),
                tournament_size: Some(3),
                ..Default::default()
            },
            OptimizationProfile::Balanced => EvolutionConfig {
                population_size: 100,
                generation_limit: 100,
                elite_count: 2,
                mutation_rate: 0.8,
                crossover_rate: 0.8,
                seed: Some(42),
                tournament_size: Some(3),
                ..Default::default()
            },
            OptimizationProfile::Thorough => EvolutionConfig {
                population_size: 250,
                generation_limit: 250,
                elite_count: 3,
                mutation_rate: 0.8,
                crossover_rate: 0.8,
                seed: None,
                tournament_size: Some(5),
                ..Default::default()
            },
            OptimizationProfile::Research => EvolutionConfig::default(),
        }
    }
}
