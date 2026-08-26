use crate::traits::{MemoryModel, TopologyModel};
use serde::{Deserialize, Serialize};

/// Defines the deterministic state evolution constraints for a chronological container.
/// This strictly models memory physics.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CognitionGeometry {
    RollingBounded { window: usize },
    EventReset { drop_threshold_pct: f64 },
    Accumulator,
}

/// A deterministic state container that evolves under a specific CognitionGeometry.
#[derive(Debug, Clone)]
pub struct MemoryState {
    pub geometry: CognitionGeometry,
    pub buffer: Vec<f64>,
    pub running_max: f64,
}

impl MemoryState {
    pub fn new(geometry: CognitionGeometry) -> Self {
        Self {
            geometry,
            buffer: Vec::new(),
            running_max: 0.0,
        }
    }

    pub fn overlap_ratio(&self, baseline: &MemoryState) -> f64 {
        if baseline.buffer.is_empty() {
            return 0.0;
        }

        let base_len = baseline.buffer.len();
        let frag_len = self.buffer.len();

        let mut overlap_count = 0;

        for i in 0..base_len {
            let base_val = baseline.buffer[base_len - 1 - i];
            let frag_val = if i < frag_len {
                self.buffer[frag_len - 1 - i]
            } else {
                0.0
            };

            if (base_val - frag_val).abs() < f64::EPSILON {
                overlap_count += 1;
            }
        }

        overlap_count as f64 / base_len as f64
    }
}

// Wire to the Coralys trait
impl MemoryModel<f64> for MemoryState {
    type State = Self;

    fn observe(&mut self, value: f64) {
        if self.buffer.is_empty() || value > self.running_max {
            self.running_max = value;
        }

        match self.geometry {
            CognitionGeometry::RollingBounded { window } => {
                self.buffer.push(value);
                if self.buffer.len() > window {
                    self.buffer.remove(0);
                }
            }
            CognitionGeometry::EventReset { drop_threshold_pct } => {
                let threshold_val = self.running_max * (1.0 - drop_threshold_pct);
                if !self.buffer.is_empty() && value < threshold_val {
                    self.buffer.clear();
                    self.running_max = value;
                }
                self.buffer.push(value);
            }
            CognitionGeometry::Accumulator => {
                self.buffer.push(value);
            }
        }
    }

    fn state(&self) -> &Self::State {
        self
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TopologyField {
    UniformDelay {
        delay_ticks: u32,
    },
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

// Wire to the Coralys trait. The input is (tick_index, total_ticks)
impl TopologyModel<(u64, u64)> for TopologyField {
    type Output = DeformationState;

    fn transform(&self, input: (u64, u64)) -> Self::Output {
        let (tick_index, total_ticks) = input;
        match self {
            TopologyField::Baseline => DeformationState {
                acceptance_ratio: 1.0,
                strict_ratio: 1.0,
            },
            TopologyField::UniformDelay { .. } => DeformationState {
                acceptance_ratio: 1.0,
                strict_ratio: 0.0,
            },
            TopologyField::Oscillatory {
                period,
                amplitude,
                noise,
            } => {
                let wave =
                    (std::f64::consts::PI * 2.0 * (tick_index as f64 / *period as f64)).cos();
                let wave_norm = (wave + 1.0) / 2.0;

                let mut wave_prog = (1.0 - amplitude) + (amplitude * wave_norm);

                if *noise > 0.0 {
                    let pseudo_noise = ((tick_index.wrapping_mul(1103515245).wrapping_add(12345)
                        % 1000) as f64
                        / 1000.0)
                        * 2.0
                        - 1.0;
                    wave_prog += pseudo_noise * noise;
                }

                let wave_prog = wave_prog.clamp(0.0, 1.0);

                DeformationState {
                    acceptance_ratio: (wave_prog + 0.3).min(1.0),
                    strict_ratio: wave_prog,
                }
            }
            TopologyField::PlateauLow { occupancy } => DeformationState {
                acceptance_ratio: *occupancy,
                strict_ratio: *occupancy,
            },
            TopologyField::ImpulseShock { at_tick, magnitude } => {
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
            }
            TopologyField::DriftField { min_acceptance } => {
                let progress = tick_index as f64 / total_ticks.max(1) as f64;
                let wave_prog = 1.0 - (progress * (1.0 - min_acceptance));
                let wave_prog = wave_prog.clamp(0.0, 1.0);

                DeformationState {
                    acceptance_ratio: wave_prog,
                    strict_ratio: wave_prog,
                }
            }
            TopologyField::FragmentedRegime { switch_period } => {
                let regime_idx = tick_index / (*switch_period as u64).max(1);
                let wave_prog = if regime_idx % 2 == 0 { 1.0 } else { 0.1 };

                DeformationState {
                    acceptance_ratio: wave_prog,
                    strict_ratio: wave_prog,
                }
            }
        }
    }
}
