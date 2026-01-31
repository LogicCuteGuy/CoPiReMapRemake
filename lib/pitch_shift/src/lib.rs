use realfft::{RealFftPlanner};
use realfft::RealToComplex;
use realfft::ComplexToReal;
use realfft::num_complex::Complex;

use std::f32::consts::PI;
use std::f32::consts::TAU;
use std::sync::Arc; // = 2xPI
const COMPLEX_ZERO: Complex<f32> = Complex::new(0.0, 0.0);

/// See [`PitchShifter::new`] & [`PitchShifter::shift_pitch`]
pub struct PitchShifter {
    forward_fft: Arc<dyn RealToComplex<f32>>,
    inverse_fft: Arc<dyn ComplexToReal<f32>>,
    ffft_scratch_len: usize,
    ifft_scratch_len: usize,
    fft_scratch: Vec<Complex<f32>>,
    fft_real: Vec<f32>,
    fft_cplx: Vec<Complex<f32>>,

    in_fifo: Vec<f32>,
    out_fifo: Vec<f32>,

    last_phase: Vec<f32>,
    phase_sum: Vec<f32>,
    windowing: Vec<f32>,
    output_accumulator: Vec<f32>,
    synthesized_frequency: Vec<f32>,
    synthesized_magnitude: Vec<f32>,

    frame_size: u32,
    overlap: u32,

    //pitch

    fifo_latency: u32,
    half_frame_size: u32,
    shift: f32,
    expected: f32,
    pitch_weight: f32,
    oversamp_weight: f32,
    over_sampling: u8,
    step: u32,
    mean_expected: f32,
    bin_frequencies: f32,
}

impl PitchShifter {
    /// Phase Vocoding works by extracting overlapping windows
    /// from a buffer and processing them individually before
    /// merging the results into the output buffer.
    ///
    /// You must set a duration in miliseconds for these windows;
    /// 50ms is a good value.
    ///
    /// The sample rate argument must correspond to the sample
    /// rate of the buffer(s) you will provide to
    /// [`PitchShifter::shift_pitch`], which is how many values
    /// correspond to one second of audio in the buffer.
    pub fn new(window_duration_ms: u8, sample_rate: u32, over_sampling: u8, shift: f32) -> Self {
        let mut frame_size: u32 = sample_rate * window_duration_ms as u32 / 1000;
        frame_size += frame_size % 2;
        let fs_real = frame_size;

        let double_frame_size = frame_size * 2;

        let mut planner = RealFftPlanner::<f32>::new();
        let forward_fft = planner.plan_fft_forward(frame_size as usize);
        let inverse_fft = planner.plan_fft_inverse(frame_size as usize);
        let ffft_scratch_len = forward_fft.get_scratch_len();
        let ifft_scratch_len = inverse_fft.get_scratch_len();
        let scratch_len = ffft_scratch_len.max(ifft_scratch_len);
        let mut windowing = vec![0.0; frame_size as usize];
        for k in 0..frame_size {
            windowing[k as usize] = -0.5 * (TAU * (k as f32) / fs_real as f32).cos() + 0.5;
        }

        //pitch
        let fs_real = frame_size as f32;
        let half_frame_size = (frame_size / 2) + 1;

        let step = frame_size / over_sampling as u32;
        let bin_frequencies = sample_rate as f32 / fs_real;
        let expected = TAU / (over_sampling as f32);
        let fifo_latency = frame_size - step;
        println!("{}", fifo_latency);
        let overlap= fifo_latency;

        let pitch_weight = shift * bin_frequencies;
        let oversamp_weight = ((over_sampling as f32) / TAU) * pitch_weight;
        let mean_expected = expected / bin_frequencies;

        Self {
            forward_fft,
            inverse_fft,
            ffft_scratch_len,
            ifft_scratch_len,
            fft_scratch: vec![COMPLEX_ZERO; scratch_len],
            fft_real: vec![0.0; frame_size as usize],
            fft_cplx: vec![COMPLEX_ZERO; half_frame_size as usize],

            in_fifo: vec![0.0; frame_size as usize],
            out_fifo: vec![0.0; frame_size as usize],

            last_phase: vec![0.0; half_frame_size as usize],
            phase_sum: vec![0.0; half_frame_size as usize],
            windowing,
            output_accumulator: vec![0.0; double_frame_size as usize],
            synthesized_frequency: vec![0.0; frame_size as usize],
            synthesized_magnitude: vec![0.0; frame_size as usize],

            frame_size,
            overlap,

            fifo_latency,
            half_frame_size,
            shift,
            expected,
            pitch_weight,
            oversamp_weight,
            over_sampling,
            step,
            mean_expected,
            bin_frequencies,
        }
    }

    #[inline]
    pub fn set_pitch(&mut self, shift: f32) {
        self.shift = shift;
        self.pitch_weight = self.shift * self.bin_frequencies;
        self.oversamp_weight = ((self.over_sampling as f32) / TAU) * self.pitch_weight;
    }

    #[inline]
    pub fn set_over_sampling(&mut self, over_sampling: u8) {
        self.step = self.frame_size / over_sampling as u32;
        self.expected = TAU / (over_sampling as f32);
        self.fifo_latency = self.frame_size - self.step;
        self.overlap = self.fifo_latency;
        self.oversamp_weight = ((over_sampling as f32) / TAU) * self.pitch_weight;
        self.mean_expected = self.expected / self.bin_frequencies;
    }

    #[inline]
    pub fn get_latency(&self) -> u32 {
        self.fifo_latency
    }

    #[inline]
    pub fn get_pitch(&self) -> f32 {
        self.shift
    }

    pub fn reset(&mut self) {
        self.out_fifo.fill(0.0);
        self.overlap = self.fifo_latency;
    }
    /// This is where the magic happens.
    ///
    /// The bigger `over_sampling`, the longer it will take to
    /// process, but the better the results. I put `16` in the
    /// `shift-wav` binary.
    ///
    /// `shift` is how many semitones to apply to the buffer.
    /// It is signed: a negative value will lower the tone and
    /// vice-versa.
    ///
    /// `in_b` is where the input buffer goes, and you must pass
    /// an output buffer of the same length in `out_b`.
    ///
    /// Note: It's actually not magic, sadly.
    #[inline]
    pub fn process(&mut self, signal: f32) -> f32 {
        self.in_fifo[self.overlap as usize] = signal;
        let out = self.out_fifo[(self.overlap - self.fifo_latency) as usize];
        self.overlap += 1;
        if self.overlap >= self.frame_size {
            self.overlap = self.fifo_latency;

            for k in 0..self.frame_size {
                self.fft_real[k as usize] = self.in_fifo[k as usize] * self.windowing[k as usize];
            }

            let _ = self.forward_fft.process_with_scratch(
                &mut self.fft_real,
                &mut self.fft_cplx,
                &mut self.fft_scratch[..self.ffft_scratch_len],
            );//.unwrap();

            self.synthesized_magnitude.fill(0.0);
            self.synthesized_frequency.fill(0.0);

            for k in 0..self.half_frame_size {
                let k_real = k as f32;
                let index = (k_real * self.shift).round() as usize;
                if index < self.half_frame_size as usize {
                    let (magnitude, phase) = self.fft_cplx[k as usize].to_polar();
                    let mut delta_phase = (phase - self.last_phase[k as usize]) - k_real * self.expected;
                    // must not round here for some reason
                    let mut qpd = (delta_phase / PI) as i64;

                    if qpd >= 0 {
                        qpd += qpd & 1;
                    } else {
                        qpd -= qpd & 1;
                    }

                    delta_phase -= PI * qpd as f32;
                    self.last_phase[k as usize] = phase;
                    self.synthesized_magnitude[index] += magnitude;
                    self.synthesized_frequency[index] = k_real * self.pitch_weight + self.oversamp_weight * delta_phase;
                }
            }

            self.fft_cplx.fill(COMPLEX_ZERO);

            for k in 0..self.half_frame_size {
                self.phase_sum[k as usize] += self.mean_expected * self.synthesized_frequency[k as usize];

                let (sin, cos) = self.phase_sum[k as usize].sin_cos();
                let magnitude = self.synthesized_magnitude[k as usize];

                self.fft_cplx[k as usize].im = sin * magnitude;
                self.fft_cplx[k as usize].re = cos * magnitude;
            }

            let _ = self.inverse_fft.process_with_scratch(
                &mut self.fft_cplx,
                &mut self.fft_real,
                &mut self.fft_scratch[..self.ifft_scratch_len],
            );//.unwrap();

            let acc_oversamp: f32 = 2.0 / (self.half_frame_size * self.over_sampling as u32) as f32;

            for k in 0..self.frame_size {
                let product = self.windowing[k as usize] * self.fft_real[k as usize] * acc_oversamp;
                self.output_accumulator[k as usize] += product / 2.0;
            }

            self.out_fifo[..self.step as usize].copy_from_slice(&self.output_accumulator[..self.step as usize]);
            self.output_accumulator.copy_within(self.step as usize..((self.step + self.frame_size) as usize), 0);
            self.in_fifo.copy_within(self.step as usize..((self.step + self.fifo_latency) as usize), 0);
        }
        out
    }

    #[inline]
    pub fn process_buffer(&mut self, input: &mut [f32]) {
        for sample in input.iter_mut() {
            *sample = self.process(*sample);
        }
    }
}