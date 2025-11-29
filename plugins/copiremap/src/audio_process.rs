use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use nih_plug::audio_setup::BufferConfig;
use nih_plug::formatters;
use nih_plug::params::{BoolParam, EnumParam, FloatParam, IntParam, Params};
use nih_plug::prelude::{Enum, FloatRange, IntRange};
use nih_plug::util::db_to_gain;
use simple_eq::design::Curve;
use crate::{PluginParams};
use crate::delay::Delay;
use crate::filter::MyFilter;
use crate::gate::MyGate;
use crate::hertz_calculator::{hz_cal_clh, hz_cal_tlh};
use crate::key_note_midi_gen::{MidiNote, NoteModeMidi};
use crate::pitch::MyPitch;

#[derive(Params)]
pub struct AudioProcessParams {
    #[id = "threshold"]
    pub threshold: FloatParam,
    
    #[id = "threshold_flip"]
    pub threshold_flip: BoolParam,
    
    #[id = "threshold_attack"]
    pub threshold_attack: FloatParam,
    
    #[id = "threshold_release"]
    pub threshold_release: FloatParam,

    #[id = "resonance"]
    pub resonance: FloatParam,

    #[id = "bandpass_mode"]
    pub bandpass_mode: EnumParam<BandpassMode>,

    #[id = "pitch_shift"]
    pub pitch_shift: BoolParam,

    #[id = "pitch_shift_node"]
    pub pitch_shift_node: EnumParam<PitchShiftNode>,

    #[id = "pitch_shift_over_sampling"]
    pub pitch_shift_over_sampling: IntParam,

    #[id = "pitch_shift_window_duration_ms"]
    pub pitch_shift_window_duration_ms: IntParam,
    
    #[id = "in_key_gain"]
    pub in_key_gain: FloatParam,

    #[id = "tuning_gain"]
    pub tuning_gain: FloatParam,

    #[id = "off_key_gain"]
    pub off_key_gain: FloatParam,

}

impl AudioProcessParams {
    pub fn new(update_pitch_shift_over_sampling: Arc<AtomicBool>, update_pitch_shift_window_duration_ms: Arc<AtomicBool>, update_pitch_shift_and_after_bandpass: Arc<AtomicBool>, update_bpf_center_hz: Arc<AtomicBool>, set_pitch_shift_12_node: Arc<AtomicBool>) -> Self {
        Self {
            threshold: FloatParam::new(
                "Threshold",
                db_to_gain(-90.0),
                FloatRange::Linear {
                    min: db_to_gain(-100.0),
                    max: db_to_gain(0.0),
                },
            ).with_unit(" dB")
                .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
                .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            threshold_flip: BoolParam::new("Threshold Flip", false),
            threshold_attack: FloatParam::new("Threshold Attack", 0.1, FloatRange::Linear {
                min: 0.1,
                max: 5.0,
            }).with_unit("ms").with_step_size(0.01),
            threshold_release: FloatParam::new("Threshold Release", 0.1, FloatRange::Linear {
                min: 0.1,
                max: 5.0,
            }).with_unit("ms").with_step_size(0.01),
            resonance: FloatParam::new("Resonance", 50.0, FloatRange::Linear{ min: 20.0, max: 300.0 })
                .with_callback(
                    {
                        let update_bpf_center_hz = update_bpf_center_hz.clone();
                        Arc::new(move |_| {
                            update_bpf_center_hz.store(true, Ordering::Release);
                        })
                    }
                ).with_step_size(0.01),
            bandpass_mode: EnumParam::new("Bandpass Mode", BandpassMode::Iir),
            pitch_shift: BoolParam::new(
                "Pitch Shift",
                true,
            ).with_callback({
                let update_pitch_shift_and_after_bandpass = update_pitch_shift_and_after_bandpass.clone();
                Arc::new(move |_| {
                    update_pitch_shift_and_after_bandpass.store(true, Ordering::Release);
                })
            }),
            pitch_shift_node: EnumParam::new("Pitch Shift Node", PitchShiftNode::Node12).with_callback({
                let set_pitch_shift_12_node = set_pitch_shift_12_node.clone();
                Arc::new(move |_| {
                    set_pitch_shift_12_node.store(true, Ordering::Release);
                })
            }),
            pitch_shift_over_sampling: IntParam::new(
                "Pitch Shift Over Sampling",
                1,
                IntRange::Linear {
                    min: 1,
                    max: 8,
                }
            ).with_callback(
                {
                    Arc::new(move |_| {
                        update_pitch_shift_over_sampling.store(true, Ordering::Release);
                    })
                }
            ),
            pitch_shift_window_duration_ms: IntParam::new(
                "Pitch Shift Window Duration",
                7,
                IntRange::Linear {
                    min: 1,
                    max: 100,
                }
            ).with_unit("ms")
                .with_callback(
                    {
                        Arc::new(move |_| {
                            update_pitch_shift_window_duration_ms.store(true, Ordering::Release);
                        })
                    }
                ),
            in_key_gain: FloatParam::new(
                "In Key Gain",
                db_to_gain(0.0),
                FloatRange::Skewed {
                    min: db_to_gain(-65.0),
                    max: db_to_gain(12.0),
                    factor: FloatRange::gain_skew_factor(-65.0, 12.0),
                }
            ).with_unit(" dB")
                .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
                .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            tuning_gain: FloatParam::new(
                "Tuning Gain",
                db_to_gain(0.0),
                FloatRange::Skewed {
                    min: db_to_gain(-65.0),
                    max: db_to_gain(12.0),
                    factor: FloatRange::gain_skew_factor(-65.0, 12.0),
                }
            ).with_unit(" dB")
                .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
                .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            off_key_gain: FloatParam::new(
                "Off Key Gain",
                db_to_gain(0.0),
                FloatRange::Skewed {
                    min: db_to_gain(-65.0),
                    max: db_to_gain(12.0),
                    factor: FloatRange::gain_skew_factor(-65.0, 12.0),
                }
            ).with_unit(" dB")
                .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
                .with_string_to_value(formatters::s2v_f32_gain_to_db()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
#[non_exhaustive]
pub enum PitchShiftNode {
    #[id = "96_node"]
    #[name = "96"]
    Node96,
    #[id = "12_node"]
    #[name = "12"]
    Node12,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
#[non_exhaustive]
pub enum BandpassMode {
    #[id = "iir"]
    #[name = "IIR"]
    Iir,
    #[id = "fft_512"]
    #[name = "FFT 512"]
    Fft512,
    #[id = "fft_1024"]
    #[name = "FFT 1024"]
    Fft1024,
    #[id = "fft_2048"]
    #[name = "FFT 2048"]
    Fft2048,
}

pub struct AudioProcess96 {
    bpf: MyFilter,
    pub tuning: Option<MyPitch>,
    delay: Delay,
    pub(crate) gate: MyGate,
    open: bool,
    pub note: u8,
    pub note_pitch: i8,
}

impl AudioProcess96 {
    pub fn reset(&mut self) {
        if let Some(value) = self.tuning.as_mut() {
            value.reset()
        }
        self.bpf.reset();
        self.delay.reset();
    }

    pub fn get_latency(&self) -> u32 {
        let mut sum: u32 = 0;
        sum += match self.tuning.as_ref() {
            Some(value) => {
                value.get_latency()
            }
            None => {
                0
            }
        };
        sum
    }

    pub fn set_delay(&mut self, delay: u32) {
        self.delay.set_delay(delay);
    }

    pub fn setup(&mut self, params: Arc<PluginParams>, note: u8, buffer_config: &BufferConfig, midi_notes: &MidiNote) {
        let note_pitch: i8 = match params.key_note.note_mode_midi.value() {
            NoteModeMidi::MidiWhistle | NoteModeMidi::MidiScale => midi_notes.im2t[self.note as usize],
            _ => midi_notes.i2t[self.note as usize]
        };
        let mut pitch_tune_hz: f32 = 0.0;
        let mut bandpass: f32 = 0.0;
        self.note_pitch = note_pitch;
        hz_cal_tlh(note, note_pitch, &mut pitch_tune_hz, &mut bandpass, params.global.hz_center.value(), params.global.hz_tuning.value(), !params.audio_process.pitch_shift.value());
        if params.audio_process.pitch_shift_node.value() == PitchShiftNode::Node12 && note < ((params.global.low_note_off.value() as usize - 36) + 12) as u8 {
            self.tuning = Some(MyPitch::set_window_duration_ms(params.audio_process.pitch_shift_window_duration_ms.value() as u8, buffer_config.sample_rate, params.audio_process.pitch_shift_over_sampling.value() as u8, pitch_tune_hz));
        } else {
            self.tuning = None;
        }
        if params.audio_process.pitch_shift_node.value() == PitchShiftNode::Node96 {
            self.tuning = Some(MyPitch::set_window_duration_ms(params.audio_process.pitch_shift_window_duration_ms.value() as u8, buffer_config.sample_rate, params.audio_process.pitch_shift_over_sampling.value() as u8, pitch_tune_hz));
        }
        self.bpf.set(Curve::Bandpass, bandpass, params.audio_process.resonance.value(), 0.0, buffer_config.sample_rate);
        self.note = note;
    }

    pub fn set_pitch_shift_12_node(&mut self, params: Arc<PluginParams>, buffer_config: &BufferConfig, midi_notes: &MidiNote) {
        let note_pitch: i8 = match params.key_note.note_mode_midi.value() {
            NoteModeMidi::MidiWhistle | NoteModeMidi::MidiScale => midi_notes.im2t[self.note as usize],
            _ => midi_notes.i2t[self.note as usize]
        };
        let mut pitch_tune_hz: f32 = 0.0;
        let mut bandpass: f32 = 0.0;
        self.note_pitch = note_pitch;
        hz_cal_tlh(self.note, note_pitch, &mut pitch_tune_hz, &mut bandpass, params.global.hz_center.value(), params.global.hz_tuning.value(), !params.audio_process.pitch_shift.value());
        if params.audio_process.pitch_shift_node.value() == PitchShiftNode::Node12 && self.note < ((params.global.low_note_off.value() as usize - 36) + 12) as u8 {
            self.tuning = Some(MyPitch::set_window_duration_ms(params.audio_process.pitch_shift_window_duration_ms.value() as u8, buffer_config.sample_rate, params.audio_process.pitch_shift_over_sampling.value() as u8, pitch_tune_hz));
        } else {
            self.tuning = None;
        }
        if params.audio_process.pitch_shift_node.value() == PitchShiftNode::Node96 {
            self.tuning = Some(MyPitch::set_window_duration_ms(params.audio_process.pitch_shift_window_duration_ms.value() as u8, buffer_config.sample_rate, params.audio_process.pitch_shift_over_sampling.value() as u8, pitch_tune_hz));
        }
    }

    pub fn set_pitch_shift_and_after_bandpass(&mut self, params: Arc<PluginParams>, note_pitch: i8, buffer_config: &BufferConfig) {
        let mut bandpass: f32 = 0.0;
        let mut pitch_tune_hz: f32 = 0.0;
        hz_cal_tlh(self.note, note_pitch, &mut pitch_tune_hz, &mut bandpass, params.global.hz_center.value(), params.global.hz_tuning.value(), !params.audio_process.pitch_shift.value());
        self.bpf.set(Curve::Bandpass, bandpass, params.audio_process.resonance.value(), 0.0, buffer_config.sample_rate);
        self.note_pitch = note_pitch;
        if let Some(value) = self.tuning.as_mut() {
            value.set_pitch(pitch_tune_hz);
        }
    }

    pub fn set_pitch_shift_over_sampling(&mut self, params: Arc<PluginParams>) {
        match self.tuning.as_mut() {
            None => {}
            Some(v) => {
                v.set_over_sampling(params.audio_process.pitch_shift_over_sampling.value() as u8);
            }
        }
    }

    pub fn set_pitch_shift_window_duration_ms(&mut self, params: Arc<PluginParams>, buffer_config: &BufferConfig) {
        match self.tuning.as_mut() {
            None => {}
            Some(v) => {
                *v = MyPitch::set_window_duration_ms(params.audio_process.pitch_shift_window_duration_ms.value() as u8, buffer_config.sample_rate, params.audio_process.pitch_shift_over_sampling.value() as u8, v.get_pitch());
            }
        }
    }

    pub fn set_bpf_center_hz(&mut self, params: Arc<PluginParams>, buffer_config: &BufferConfig, midi_notes: &MidiNote) {
        let note_pitch: i8 = match params.key_note.note_mode_midi.value() {
            NoteModeMidi::MidiWhistle | NoteModeMidi::MidiScale => midi_notes.im2t[self.note as usize],
            _ => midi_notes.i2t[self.note as usize]
        };
        let mut center_hz: f32 = 0.0;
        hz_cal_clh(self.note, note_pitch, &mut center_hz, params.global.hz_center.value(), !params.audio_process.pitch_shift.value());
        self.bpf.set(Curve::Bandpass, center_hz, params.audio_process.resonance.value(), 0.0, buffer_config.sample_rate);
    }

    pub fn process(&mut self, input: f32, params: Arc<PluginParams>, audio_id: usize, input_param: f32, buffer_config: &BufferConfig, buf_size: usize) -> f32 {
        let pitch: f32 = match params.audio_process.pitch_shift.value() && !(self.note_pitch == 0 || self.note_pitch == -128) && !!(params.audio_process.pitch_shift_node.value() == PitchShiftNode::Node12 || self.open) {
            true => match self.tuning.as_mut() {
                None => {
                    0.0
                }
                Some(v) => {
                    v.process(input, audio_id)
                }
            },
            false => match self.tuning.as_ref() {
                None => {
                    0.0
                }
                Some(_) => {
                    self.delay.process(input, audio_id)
                }
            }
        };
        let bpf: f32 = match params.audio_process.pitch_shift_node.value() {
            PitchShiftNode::Node12 => {
                pitch
            }
            PitchShiftNode::Node96 => {
                if input_param > db_to_gain(-60.0) {
                    self.process_bpf(pitch, audio_id, input_param, params.clone())
                } else {
                    0.0
                }
            }
        };
        let flip = params.audio_process.threshold_flip.value();
        self.open = self.gate.update_fast_param(bpf, buffer_config, params.audio_process.threshold.value(), params.audio_process.threshold_attack.value(), params.audio_process.threshold_release.value(), buf_size, flip, audio_id).0;
        // output = if self.note_pitch == -128 { 0.0 } else { output };
        bpf * self.gate.get_param(flip, audio_id)
    }

    pub fn process_bpf(&mut self, input: f32, audio_id: usize, input_param: f32, params: Arc<PluginParams>) -> f32 {
        if !(self.note_pitch == -128 && params.key_note.mute_off_key.value()) {self.bpf.process(input, audio_id) * input_param } else { 0.0 }
    }

    pub fn fn_update_pitch_shift_and_after_bandpass(params: Arc<PluginParams>, audio_process: &mut [AudioProcess96], buffer_config: &BufferConfig, note_pitch: [i8; 96]) {
        for (i, ap) in audio_process.iter_mut().enumerate() {
            ap.set_pitch_shift_and_after_bandpass(params.clone(), note_pitch[i], buffer_config);
        }
        println!("Doing {:?}", note_pitch)
    }
}

impl Default for AudioProcess96 {
    fn default() -> Self {
        Self {
            bpf: MyFilter::default(),
            tuning: None,
            delay: Delay::default(),
            gate: MyGate::new(),
            open: false,
            note: 0,
            note_pitch: 0,
        }
    }

}