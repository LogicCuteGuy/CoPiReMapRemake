use crate::audio_process::AudioProcess96;

pub struct Delay {
    pub delay_samples: [Vec<f32>; 2],
}

impl Delay {

    pub fn set_delay(&mut self, delay: u32) {
        self.delay_samples[0].resize(delay as usize, 0.0);
        self.delay_samples[1].resize(delay as usize, 0.0);
    }

    pub fn reset(&mut self) {
        self.delay_samples[0].fill(0.0);
        self.delay_samples[1].fill(0.0);
    }

    pub fn process(&mut self, input: f32, audio_id: usize) -> f32 {
        self.delay_samples[audio_id].push(input);
        let out = *self.delay_samples[audio_id].first().unwrap();
        self.delay_samples[audio_id].remove(0);
        out
    }

    pub fn get_latency(&self) -> u32 {
        self.delay_samples[0].len() as u32
    }

}

impl Default for Delay {
    fn default() -> Self {
        const ARRAY_REPEAT_VALUE: Vec<f32> = Vec::new();
        let mut delay_samples: [Vec<f32>; 2] = [ARRAY_REPEAT_VALUE; 2];
        for _i in 0..1 {
            delay_samples[0].push(0.0);
            delay_samples[1].push(0.0);
        }
        Self {
            delay_samples,
        }
    }
}

pub fn latency_average96(ap: &[AudioProcess96]) -> u32 {
    ap[0].get_latency()
}