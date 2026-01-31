use std::f32::consts::PI;

/// Sine wave generator with amplitude tracking
pub struct SineGen {
    phase: [f32; 2],           // Phase accumulator for stereo
    smoothed_amplitude: [f32; 2], // Smoothed amplitude for envelope following
    frequency: f32,
    sample_rate: f32,
}

impl SineGen {
    pub fn new(frequency: f32, sample_rate: f32) -> Self {
        Self {
            phase: [0.0, 0.0],
            smoothed_amplitude: [0.0, 0.0],
            frequency,
            sample_rate,
        }
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    pub fn reset(&mut self) {
        self.phase = [0.0, 0.0];
        self.smoothed_amplitude = [0.0, 0.0];
    }

    /// Generate sine wave with given amplitude
    /// amplitude: detected amplitude from bandpass filter
    /// audio_id: 0 for left, 1 for right channel
    pub fn process(&mut self, amplitude: f32, audio_id: usize) -> f32 {
        // Smooth the amplitude with a simple one-pole lowpass filter
        // This prevents the amplitude from changing too quickly and creating square wave artifacts
        let smoothing_factor = 0.995; // Higher = smoother (0.0 to 1.0)
        self.smoothed_amplitude[audio_id] = 
            smoothing_factor * self.smoothed_amplitude[audio_id] + (1.0 - smoothing_factor) * amplitude;
        
        // Calculate phase increment (-2 octaves = frequency / 4)
        let phase_increment = 2.0 * PI * (self.frequency / 2.0) / self.sample_rate;
        
        // Generate sine wave with smoothed amplitude
        let output = self.smoothed_amplitude[audio_id] * self.phase[audio_id].sin();
        
        // Update phase
        self.phase[audio_id] += phase_increment;
        
        // Wrap phase to avoid precision issues
        if self.phase[audio_id] >= 2.0 * PI {
            self.phase[audio_id] -= 2.0 * PI;
        }
        
        output
    }
}

impl Default for SineGen {
    fn default() -> Self {
        Self::new(440.0, 44100.0)
    }
}
