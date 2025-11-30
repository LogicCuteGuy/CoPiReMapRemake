use simple_eq::design::{Curve, Design};
use simple_eq::filter::Filter;

#[allow(dead_code)]

// ============================================================================
// IIR Filter
// ============================================================================

pub struct MyFilter {
    filter: [Filter<f32>; 2],
}

#[allow(dead_code)]
impl MyFilter {
    pub fn set(&mut self, curve: Curve, frequency: f32, resonance: f32, gain: f32, sample_rate: f32) {
        self.filter[0].set(curve, frequency, resonance, gain, sample_rate);
        self.filter[1].set(curve, frequency, resonance, gain, sample_rate);
    }
    
    pub fn get_design(&self) -> [Design<f32>; 2] {
        [self.filter[0].get_design(), self.filter[1].get_design()]
    }

    pub fn set_curve(&mut self, curve: Curve) {
        self.filter[0].set_curve(curve);
        self.filter[1].set_curve(curve);
    }

    pub fn set_frequency(&mut self, freq_hz: f32) {
        self.filter[0].set_frequency(freq_hz);
        self.filter[1].set_frequency(freq_hz);
    }

    pub fn set_gain(&mut self, gain_d_b: f32) {
        self.filter[0].set_gain(gain_d_b);
        self.filter[1].set_gain(gain_d_b);
    }

    pub fn set_resonance(&mut self, resonance: f32) {
        self.filter[0].set_resonance(resonance);
        self.filter[1].set_resonance(resonance);
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.filter[0].set_sample_rate(sample_rate);
        self.filter[1].set_sample_rate(sample_rate);
    }

    pub fn reset(&mut self) {
        self.filter[0].reset();
        self.filter[1].reset();
    }

    pub fn process(&mut self, input: f32, audio_id: usize) -> f32 {
        self.filter[audio_id].filter(input)
    }
}

impl Default for MyFilter {
    fn default() -> Self {
        Self {
            filter: [Filter::new(44100.0); 2]
        }
    }
}