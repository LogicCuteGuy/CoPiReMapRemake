use nih_plug::audio_setup::BufferConfig;
use nih_plug::prelude::Enum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum DetectionMode {
    #[id = "peak"]
    #[name = "Peak"]
    Peak,
    #[id = "rms"]
    #[name = "RMS"]
    Rms,
}

pub struct MyGate {
    pub level: [f32; 2],
    rms_buffer: [Vec<f32>; 2],
    rms_index: [usize; 2],
    rms_sum: [f32; 2],
    pub param: [f32; 2],
    detection_coeff: [f32; 2],
}

impl MyGate {
    pub fn new() -> Self {
        Self {
            level: [0.0; 2],
            rms_buffer: [Vec::new(), Vec::new()],
            rms_index: [0; 2],
            rms_sum: [0.0; 2],
            param: [0.0; 2],
            detection_coeff: [0.0; 2],
        }
    }

    pub fn update_fast_param(
        &mut self,
        sample: f32,
        buffer_config: &BufferConfig,
        threshold: f32,
        attack_ms: f32,
        release_ms: f32,
        buf_size: usize,
        flip: bool,
        audio_id: usize,
        detection_mode: DetectionMode,
    ) -> (bool, bool) {
        // Initialize RMS buffer if needed
        if self.rms_buffer[audio_id].len() != buf_size {
            self.rms_buffer[audio_id] = vec![0.0; buf_size];
            self.rms_index[audio_id] = 0;
            self.rms_sum[audio_id] = 0.0;
        }

        // Calculate detection coefficient for smooth level tracking (10ms window)
        if self.detection_coeff[audio_id] == 0.0 {
            self.detection_coeff[audio_id] =
                (-1.0 / (10.0 * 0.001 * buffer_config.sample_rate)).exp();
        }

        // Update level based on detection mode
        let current_level = match detection_mode {
            DetectionMode::Peak => {
                // Peak detection with smooth decay
                let abs_sample = sample.abs();
                if abs_sample > self.level[audio_id] {
                    abs_sample
                } else {
                    // Smooth decay
                    abs_sample
                        + (self.level[audio_id] - abs_sample) * self.detection_coeff[audio_id]
                }
            }
            DetectionMode::Rms => {
                // Sliding window RMS
                let squared = sample * sample;
                let old_squared = self.rms_buffer[audio_id][self.rms_index[audio_id]];
                self.rms_sum[audio_id] = self.rms_sum[audio_id] - old_squared + squared;
                self.rms_buffer[audio_id][self.rms_index[audio_id]] = squared;
                self.rms_index[audio_id] = (self.rms_index[audio_id] + 1) % buf_size;
                (self.rms_sum[audio_id] / buf_size as f32).sqrt()
            }
        };

        self.level[audio_id] = current_level;

        // Calculate attack/release coefficients per sample
        let attack_coeff = if attack_ms > 0.0 {
            (-1.0 / (attack_ms * 0.001 * buffer_config.sample_rate)).exp()
        } else {
            0.0
        };

        let release_coeff = if release_ms > 0.0 {
            (-1.0 / (release_ms * 0.001 * buffer_config.sample_rate)).exp()
        } else {
            0.0
        };

        // Hysteresis: use different thresholds for opening and closing to prevent chattering
        // Open threshold is the set threshold, close threshold is 3dB below
        let open_threshold = threshold;
        let close_threshold = threshold * 0.707; // -3dB

        // Determine target based on current state and hysteresis
        let target = if self.param[audio_id] > 0.5 {
            // Gate is currently open, use lower threshold to close
            if self.level[audio_id] >= close_threshold {
                1.0
            } else {
                0.0
            }
        } else {
            // Gate is currently closed, use higher threshold to open
            if self.level[audio_id] >= open_threshold {
                1.0
            } else {
                0.0
            }
        };

        // Smooth envelope follower - always use exponential smoothing
        if target > self.param[audio_id] {
            // Attack
            self.param[audio_id] = target + (self.param[audio_id] - target) * attack_coeff;
        } else {
            // Release
            self.param[audio_id] = target + (self.param[audio_id] - target) * release_coeff;
        }

        // Clamp to [0.0, 1.0]
        self.param[audio_id] = self.param[audio_id].clamp(0.0, 1.0);

        // Return gate state
        let is_open = self.param[audio_id] > 0.5;
        if flip {
            (!is_open, is_open)
        } else {
            (is_open, !is_open)
        }
    }

    pub fn get_param(&self, flip: bool, audio_id: usize) -> f32 {
        if flip {
            1.0 - self.param[audio_id]
        } else {
            self.param[audio_id]
        }
    }

    pub fn get_param_inv(&self, flip: bool, audio_id: usize) -> f32 {
        if flip {
            self.param[audio_id]
        } else {
            1.0 - self.param[audio_id]
        }
    }
}