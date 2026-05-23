use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TopologyField {
    UniformDelay { delay_ticks: u32 },
    Oscillatory {
        period: u32,
        amplitude: f64,
        noise: f64,
    },
    PlateauLow {
        occupancy: f64,
    },
    ImpulseShock {
        at_tick: u64,
        magnitude: f64,
    },
    DriftField {
        min_acceptance: f64,
    },
    FragmentedRegime {
        switch_period: u32,
    },
    Baseline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeformationState {
    pub acceptance_ratio: f64,
    pub strict_ratio: f64,
}

impl TopologyField {
    pub fn apply(&self, tick_index: u64, total_ticks: u64) -> DeformationState {
        match self {
            TopologyField::Baseline => DeformationState {
                acceptance_ratio: 1.0,
                strict_ratio: 1.0,
            },
            TopologyField::UniformDelay { .. } => DeformationState {
                acceptance_ratio: 1.0,
                strict_ratio: 0.0,
            },
            TopologyField::Oscillatory { period, amplitude, noise } => {
                let wave = (std::f64::consts::PI * 2.0 * (tick_index as f64 / *period as f64)).cos();
                let wave_norm = (wave + 1.0) / 2.0; // 0.0 to 1.0
                
                let mut wave_prog = (1.0 - amplitude) + (amplitude * wave_norm);
                
                if *noise > 0.0 {
                    // Simple deterministic pseudo-noise for rust-side (avoids random crate dependency initially)
                    let pseudo_noise = ((tick_index.wrapping_mul(1103515245).wrapping_add(12345) % 1000) as f64 / 1000.0) * 2.0 - 1.0;
                    wave_prog += pseudo_noise * noise;
                }
                
                let wave_prog = wave_prog.clamp(0.0, 1.0);
                
                DeformationState {
                    acceptance_ratio: (wave_prog + 0.3).min(1.0),
                    strict_ratio: wave_prog,
                }
            },
            TopologyField::PlateauLow { occupancy } => DeformationState {
                acceptance_ratio: *occupancy,
                strict_ratio: *occupancy,
            },
            TopologyField::ImpulseShock { at_tick, magnitude } => {
                // A shock window of 10 ticks starting at `at_tick`
                if tick_index >= *at_tick && tick_index < *at_tick + 10 {
                    DeformationState {
                        acceptance_ratio: 1.0 - magnitude,
                        strict_ratio: 1.0 - magnitude,
                    }
                } else {
                    DeformationState {
                        acceptance_ratio: 1.0,
                        strict_ratio: 1.0,
                    }
                }
            },
            TopologyField::DriftField { min_acceptance } => {
                let progress = tick_index as f64 / total_ticks as f64;
                let wave_prog = 1.0 - (progress * (1.0 - min_acceptance));
                let wave_prog = wave_prog.clamp(0.0, 1.0);
                
                DeformationState {
                    acceptance_ratio: wave_prog,
                    strict_ratio: wave_prog,
                }
            },
            TopologyField::FragmentedRegime { switch_period } => {
                let regime_idx = tick_index / (*switch_period as u64);
                let wave_prog = if regime_idx % 2 == 0 { 1.0 } else { 0.1 };
                
                DeformationState {
                    acceptance_ratio: wave_prog,
                    strict_ratio: wave_prog,
                }
            }
        }
    }
}
