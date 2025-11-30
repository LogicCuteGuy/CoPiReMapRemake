mod hertz_calculator;
mod key_note_midi_gen;
mod audio_process;
mod delay;
mod filter;
mod pitch;
mod gate;
mod sine_gen;

use std::{sync::Arc, num::NonZeroU32};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use atomic_float::AtomicF64;
use nih_plug::util::db_to_gain;
use nih_plug::{nih_export_clap, nih_export_vst3};
use nih_plug::params::persist::PersistentField;
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
mod editor;
use simple_eq::design::Curve;
use crate::audio_process::{AlgorithmMode, AudioProcess96, AudioProcessParams};
use crate::delay::{Delay, latency_average96};
use crate::filter::MyFilter;
use crate::gate::{DetectionMode, MyGate};
use crate::hertz_calculator::hz_cal_clh;
use crate::key_note_midi_gen::{KeyNoteParams, MidiNote, NoteModeMidi};

#[derive(Params)]
pub struct PluginParams {

    #[nested(group = "global")]
    pub global: Arc<GlobalParams>,

    #[nested(group = "audio_process")]
    pub audio_process: Arc<AudioProcessParams>,

    #[nested(group = "key_note")]
    pub key_note: Arc<KeyNoteParams>,

}

#[derive(Params)]
pub struct GlobalParams {

    #[id = "scale_gui"]
    pub scale_gui: FloatParam,

    #[id = "bypass"]
    pub bypass: BoolParam,

    #[id = "wet_gain"]
    pub wet_gain: FloatParam,

    #[id = "dry_gain"]
    pub dry_gain: FloatParam,

    #[id = "lhf_gain"]
    pub lhf_gain: FloatParam,

    #[id = "global_threshold"]
    pub global_threshold: FloatParam,

    #[id = "global_threshold_flip"]
    pub global_threshold_flip: BoolParam,

    #[id = "global_threshold_attack"]
    pub global_threshold_attack: FloatParam,

    #[id = "global_threshold_release"]
    pub global_threshold_release: FloatParam,

    #[id = "global_threshold_mode"]
    pub global_threshold_mode: EnumParam<DetectionMode>,

    #[id = "low_note_off"]
    pub low_note_off: IntParam,

    #[id = "high_note_off"]
    pub high_note_off: IntParam,

    #[id = "low_note_off_mute"]
    pub low_note_off_mute: BoolParam,

    #[id = "high_note_off_mute"]
    pub high_note_off_mute: BoolParam,

    #[id = "hz_center"]
    pub hz_center: FloatParam,

    #[id = "hz_tuning"]
    pub hz_tuning: FloatParam,
}

impl GlobalParams {
    fn new(update_lowpass: Arc<AtomicBool>, update_highpass: Arc<AtomicBool>, update_bpf_center_hz: Arc<AtomicBool>, update_pitch_shift_and_after_bandpass: Arc<AtomicBool>, update_gui_scale: Arc<AtomicBool>) -> Self {
        Self {
            scale_gui: FloatParam::new("Scale Gui", 1.0, FloatRange::Linear {
                min: 0.50,
                max: 1.5,
            }).with_unit("x").with_step_size(0.01).with_callback(
                {
                    let update_gui_scale = update_gui_scale.clone();
                    Arc::new(move |_| {
                        update_gui_scale.store(true, Ordering::Release);
                    })
                }
            ),
            bypass: BoolParam::new("Bypass", false)
                .with_value_to_string(formatters::v2s_bool_bypass())
                .with_string_to_value(formatters::s2v_bool_bypass())
                .make_bypass(),
            wet_gain: FloatParam::new("Wet Gain", db_to_gain(0.0), FloatRange::Skewed {
                min: db_to_gain(-24.0),
                max: db_to_gain(12.0),
                factor: FloatRange::gain_skew_factor(-24.0, 12.0),
            }).with_unit(" dB")
                .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
                .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            dry_gain: FloatParam::new("Dry Gain", db_to_gain(-24.0), FloatRange::Skewed {
                min: db_to_gain(-60.0),
                max: db_to_gain(6.0),
                factor: FloatRange::gain_skew_factor(-60.0, 6.0),
            }).with_unit(" dB")
                .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
                .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            lhf_gain: FloatParam::new("Low/HighPass Gain", db_to_gain(0.0), FloatRange::Skewed {
                min: db_to_gain(-48.0),
                max: db_to_gain(12.0),
                factor: FloatRange::gain_skew_factor(-48.0, 12.0),
            }).with_unit(" dB")
                .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
                .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            global_threshold: FloatParam::new("Global Threshold", db_to_gain(-90.0), FloatRange::Linear {
                min: db_to_gain(-100.0),
                max: db_to_gain(0.0),
            }).with_unit(" dB")
                .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
                .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            global_threshold_flip: BoolParam::new("Global Threshold Flip", false),
            global_threshold_attack: FloatParam::new("Global Threshold Attack", 0.1, FloatRange::Linear {
                min: 0.1,
                max: 100.0,
            }).with_unit("ms").with_step_size(0.01),
            global_threshold_release: FloatParam::new("Global Threshold Release", 0.1, FloatRange::Linear {
                min: 0.1,
                max: 100.0,
            }).with_unit("ms").with_step_size(0.01),
            global_threshold_mode: EnumParam::new("Global Threshold Mode", DetectionMode::Rms),
            low_note_off: IntParam::new(
                "Low Note Off",
                36,
                IntRange::Linear {
                    min: 36,
                    max: 131,
                },
            ).with_value_to_string(formatters::v2s_i32_note_formatter())
                .with_string_to_value(formatters::s2v_i32_note_formatter())
                .with_callback(
                {
                    let update_lowpass = update_lowpass.clone();
                    Arc::new(move |_| {
                        update_lowpass.store(true, Ordering::Release);
                    })
                }
            ),
            high_note_off: IntParam::new(
                "High Note Off",
                131,
                IntRange::Linear {
                    min: 36,
                    max: 131,
                }
            ).with_value_to_string(formatters::v2s_i32_note_formatter())
                .with_string_to_value(formatters::s2v_i32_note_formatter())
                .with_callback(
                    {
                        let update_highpass = update_highpass.clone();
                        Arc::new(move |_| {
                            update_highpass.store(true, Ordering::Release);
                        })
                    }
                ),
            low_note_off_mute: BoolParam::new(
                "Low Note Off Mute",
                false,
            ),
            high_note_off_mute: BoolParam::new(
                "High Note Off Mute",
                false,
            ),
            hz_center: FloatParam::new("Hz Center", 440.0, FloatRange::Linear{ min: 415.3046976, max: 466.1637615 })
                .with_value_to_string(formatters::v2s_f32_hz_then_khz(2))
                .with_string_to_value(formatters::s2v_f32_hz_then_khz())
                .with_callback(
                {
                    let update_bpf_center_hz = update_bpf_center_hz.clone();
                    Arc::new(move |_| {
                        update_bpf_center_hz.store(true, Ordering::Release);
                    })
                }
            ),
            hz_tuning: FloatParam::new("Hz Tuning", 440.0, FloatRange::Linear{ min: 415.3046976, max: 466.1637615 })
                .with_value_to_string(formatters::v2s_f32_hz_then_khz(2))
                .with_string_to_value(formatters::s2v_f32_hz_then_khz())
                .with_callback(
                {
                    let update_pitch_shift_and_after_bandpass = update_pitch_shift_and_after_bandpass.clone();
                    Arc::new(move |_| {
                        update_pitch_shift_and_after_bandpass.store(true, Ordering::Release);
                    })
                }
            )
        }
    }
}

pub struct CoPiReMapPlugin {
    params: Arc<PluginParams>,
    editor_state: Arc<ViziaState>,
    buffer_config: BufferConfig,
    midi_note: MidiNote,
    audio_process96: Vec<AudioProcess96>,
    lpf: MyFilter,
    hpf: MyFilter,
    delay: Delay,
    gate: MyGate,
    zero: MyGate,
    update_lowpass: Arc<AtomicBool>,
    update_highpass: Arc<AtomicBool>,

    update_pitch_shift_and_after_bandpass: Arc<AtomicBool>,
    update_pitch_shift_over_sampling: Arc<AtomicBool>,
    update_pitch_shift_window_duration_ms: Arc<AtomicBool>,
    update_bpf_center_hz: Arc<AtomicBool>,
    update_algorithm_mode: Arc<AtomicBool>,

    update_key_note: Arc<AtomicBool>,
    update_key_note_12: Arc<AtomicBool>,

    update_gui_scale: Arc<AtomicBool>,

    latency: Arc<AtomicU32>,
    user_scale: Arc<AtomicF64>,
}

impl Default for CoPiReMapPlugin {
    fn default() -> Self {
        let update_lowpass = Arc::new(AtomicBool::new(false));
        let update_highpass = Arc::new(AtomicBool::new(false));

        let update_pitch_shift_and_after_bandpass = Arc::new(AtomicBool::new(false));
        let update_pitch_shift_over_sampling = Arc::new(AtomicBool::new(false));
        let update_pitch_shift_window_duration_ms = Arc::new(AtomicBool::new(false));
        let update_bpf_center_hz = Arc::new(AtomicBool::new(false));
        let update_algorithm_mode = Arc::new(AtomicBool::new(false));

        let update_key_note = Arc::new(AtomicBool::new(false));
        let update_key_note_12 = Arc::new(AtomicBool::new(false));

        let update_gui_scale = Arc::new(AtomicBool::new(false));

        let mut audio_process96 = Vec::with_capacity(96);
        for _ in 0..96 {
            audio_process96.push(AudioProcess96::default());
        };

        let latency = Arc::new(AtomicU32::new(0));

        let editor_state = editor::default_state();

        Self {
            params: Arc::new(PluginParams {
                global: Arc::new(GlobalParams::new(update_lowpass.clone(), update_highpass.clone(), update_bpf_center_hz.clone(), update_pitch_shift_and_after_bandpass.clone(),  update_gui_scale.clone())),
                audio_process: Arc::new(AudioProcessParams::new(update_pitch_shift_over_sampling.clone(), update_pitch_shift_window_duration_ms.clone(), update_pitch_shift_and_after_bandpass.clone(), update_bpf_center_hz.clone(), update_algorithm_mode.clone())),
                key_note: Arc::new(KeyNoteParams::new(update_key_note.clone(), update_key_note_12.clone())),
            }),
            editor_state,
            buffer_config: BufferConfig {
                sample_rate: 1.0,
                min_buffer_size: None,
                max_buffer_size: 0,
                process_mode: ProcessMode::Realtime,
            },
            midi_note: MidiNote::default(),
            audio_process96,
            lpf: MyFilter::default(),
            hpf: MyFilter::default(),
            delay: Delay::default(),
            gate: MyGate::new(),
            zero: MyGate::new(),
            update_lowpass,
            update_highpass,
            update_pitch_shift_and_after_bandpass,
            update_pitch_shift_over_sampling,
            update_pitch_shift_window_duration_ms,
            update_bpf_center_hz,
            update_algorithm_mode,
            update_key_note,
            update_key_note_12,
            update_gui_scale,
            latency,
            user_scale: Arc::new(AtomicF64::new(1.0))
        }
    }
}

impl Plugin for CoPiReMapPlugin {
    type BackgroundTask = ();
    type SysExMessage = ();

    const NAME: &'static str = "CoPiReMap";
    const VENDOR: &'static str = "LogicCuteGuy";
    const URL: &'static str = "copiremap.logiccuteguy.com";
    const EMAIL: &'static str = "contact@logiccuteguy.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::MidiCCs;

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),

            ..AudioIOLayout::const_default()
        }
    ];

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>
    ) -> bool
    {
        self.buffer_config = *buffer_config;
        
        let mut lowpass: f32 = 0.0;
        hz_cal_clh((self.params.global.low_note_off.value() - 36) as u8, 0, &mut lowpass, self.params.global.hz_tuning.value(), self.params.audio_process.algorithm_mode.value() == AlgorithmMode::Off);
        self.lpf.set(Curve::Lowpass, lowpass, 1.0, 0.0, self.buffer_config.sample_rate);
        let mut highpass: f32 = 0.0;
        hz_cal_clh((self.params.global.high_note_off.value() - 36) as u8, 0, &mut highpass, self.params.global.hz_tuning.value(), self.params.audio_process.algorithm_mode.value() == AlgorithmMode::Off);
        self.hpf.set(Curve::Highpass, highpass, 1.0, 0.0, self.buffer_config.sample_rate);
        for (i, audio_process) in self.audio_process96.iter_mut().enumerate() {
            audio_process.setup(self.params.clone(), i as u8, &self.buffer_config, &self.midi_note);
        }
        
        self.midi_note.param_update(self.params.clone(), &mut self.audio_process96, &self.buffer_config);
        true
    }

    fn reset(&mut self) {
        for ap in self.audio_process96.iter_mut() {
            ap.reset();
        }
    }

    fn deactivate(&mut self) {
        // Ensure all audio processing is fully stopped and cleaned up
        for ap in self.audio_process96.iter_mut() {
            ap.reset();
        }
        
        // Clear all pending update flags to prevent access after deactivation
        self.update_lowpass.store(false, Ordering::Release);
        self.update_highpass.store(false, Ordering::Release);
        self.update_pitch_shift_and_after_bandpass.store(false, Ordering::Release);
        self.update_pitch_shift_over_sampling.store(false, Ordering::Release);
        self.update_pitch_shift_window_duration_ms.store(false, Ordering::Release);
        self.update_bpf_center_hz.store(false, Ordering::Release);
        self.update_algorithm_mode.store(false, Ordering::Release);
        self.update_key_note.store(false, Ordering::Release);
        self.update_key_note_12.store(false, Ordering::Release);
        self.update_gui_scale.store(false, Ordering::Release);
        
        // Reset latency
        self.latency.store(0, Ordering::Release);
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create_editor(self.params.clone(), self.editor_state.clone(), self.latency.clone())
    }

    fn process(
        &mut self,
        buffer: &mut Buffer<'_>,
        _aux: &mut AuxiliaryBuffers<'_>,
        context: &mut impl ProcessContext<Self>
    ) -> ProcessStatus
    {
        match self.params.global.bypass.value() {
            true => {
                if self.delay.get_latency() != 0 {
                    self.delay.set_delay(0);
                    context.set_latency_samples(0);
                }
            }
            false => {
                let latency = latency_average96(&self.audio_process96);
                if self.delay.get_latency() != latency {
                    self.delay.set_delay(latency);
                    for ap in self.audio_process96.iter_mut() {
                        ap.set_delay(latency)
                    }
                    self.latency.set(latency);
                    context.set_latency_samples(latency);
                }
                if self
                    .update_gui_scale
                    .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                {
                    self.user_scale.store(self.params.global.scale_gui.value() as f64, Ordering::Release);

                }
                if self
                    .update_lowpass
                    .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                {
                    let mut lowpass: f32 = 0.0;
                    let low_note = self.params.global.low_note_off.value() as usize - 36;
                    hz_cal_clh(low_note as u8, 0, &mut lowpass, self.params.global.hz_tuning.value(), self.params.audio_process.algorithm_mode.value() == AlgorithmMode::Off);
                    self.lpf.set_frequency(lowpass);
                    self.audio_process96.iter_mut().for_each(
                        |ap| {
                            ap.set_algorithm_mode(self.params.clone(), &self.buffer_config, &self.midi_note);
                        }
                    );
                }
                if self
                    .update_highpass
                    .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                {
                    let mut highpass: f32 = 0.0;
                    hz_cal_clh((self.params.global.high_note_off.value() - 36) as u8, 0, &mut highpass, self.params.global.hz_tuning.value(), self.params.audio_process.algorithm_mode.value() == AlgorithmMode::Off);
                    self.hpf.set_frequency(highpass);
                }
                if self
                    .update_pitch_shift_and_after_bandpass
                    .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                {
                    let note_table: [i8; 96] = match self.params.key_note.note_mode_midi.value() {
                        NoteModeMidi::MidiWhistle | NoteModeMidi::MidiScale => self.midi_note.im2t,
                        _ => self.midi_note.i2t
                    };
                    self.update_bpf_center_hz.set(false);
                    AudioProcess96::fn_update_pitch_shift_and_after_bandpass(self.params.clone(), &mut self.audio_process96, &self.buffer_config, note_table);
                }
                if self
                    .update_bpf_center_hz
                    .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok() {
                    for ap in self.audio_process96.iter_mut() {
                        ap.set_bpf_center_hz(self.params.clone(), &self.buffer_config, &self.midi_note);
                    }
                }
                if self
                    .update_pitch_shift_over_sampling
                    .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                {
                    for ap in self.audio_process96.iter_mut() {
                        ap.set_pitch_shift_over_sampling(self.params.clone());
                    }
                }
                if self
                    .update_pitch_shift_window_duration_ms
                    .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                {
                    for ap in self.audio_process96.iter_mut() {
                        ap.set_pitch_shift_window_duration_ms(self.params.clone(), &self.buffer_config);
                    }
                }
                if self
                    .update_algorithm_mode
                    .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                {
                    for ap in self.audio_process96.iter_mut() {
                        ap.set_algorithm_mode(self.params.clone(), &self.buffer_config, &self.midi_note);
                    }
                }
                if self
                    .update_key_note
                    .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                {
                    self.midi_note.param_update(self.params.clone(), &mut self.audio_process96, &self.buffer_config);
                }
                if self
                    .update_key_note_12
                    .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                {
                    self.midi_note.update(self.params.clone(), &mut self.audio_process96, &self.buffer_config);
                }
                while let Some(event) = context.next_event() {
                    match event {
                        NoteEvent::NoteOn {
                            timing: _timing,
                            voice_id: _voice_id,
                            channel: _channel,
                            note,
                            velocity: _velocity,
                        } => {
                            if (24..=119).contains(&note) {
                                let idx = (note as usize) - 24; // map MIDI note 24..119 -> 0..95
                                self.midi_note.midi_note[idx] = true;
                                match self.params.key_note.note_mode_midi.value() {
                                    NoteModeMidi::MidiWhistle | NoteModeMidi::MidiScale => self.midi_note.param_update(self.params.clone(), &mut self.audio_process96, &self.buffer_config),
                                    _ => {}
                                }
                            }
                        },
                        NoteEvent::NoteOff {
                            timing: _timing,
                            voice_id: _voice_id,
                            channel: _channel,
                            note,
                            velocity: _velocity,
                        } => {
                            if (24..=119).contains(&note) {
                                let idx = (note as usize) - 24; // map MIDI note 24..119 -> 0..95
                                self.midi_note.midi_note[idx] = false;
                                match self.params.key_note.note_mode_midi.value() {
                                    NoteModeMidi::MidiWhistle | NoteModeMidi::MidiScale => self.midi_note.param_update(self.params.clone(), &mut self.audio_process96, &self.buffer_config),
                                    _ => {}
                                }
                            }
                        },
                        _ => (),
                    }
                }
                let mut audio_process: f32 = 0.0;
                
                for (i, channel) in buffer.as_slice().iter_mut().enumerate() {
                    let size = channel.len();
                    for sample in channel.iter_mut() {
                        let flip = self.params.global.global_threshold_flip.value();
                        let detection_mode = self.params.global.global_threshold_mode.value();
                        let gate_zero = self.zero.update_fast_param(*sample, &self.buffer_config, db_to_gain(-99.0), 0.1, 0.1, size, false, i, detection_mode);
                        let gate_on: (bool, bool) = self.gate.update_fast_param(*sample, &self.buffer_config, self.params.global.global_threshold.value(), self.params.global.global_threshold_attack.value(), self.params.global.global_threshold_release.value(), size, flip, i, detection_mode);
                        let delay = self.delay.process(*sample, i);
                        if gate_on.0 && gate_zero.0 {
                            let lpf_mute = match self.params.global.low_note_off_mute.value() { true => 0.0, false => self.lpf.process(delay, i) };
                            let hpf_mute = match self.params.global.high_note_off_mute.value() { true => 0.0, false => self.hpf.process(delay, i) };
                            
                            let algorithm_mode = self.params.audio_process.algorithm_mode.value();
                            
                            match algorithm_mode {
                                AlgorithmMode::PitchShift => {
                                    // PitchShift mode: PitchShift -> Bandpass (IIR only, no FFT)
                                    let note_mode = self.params.key_note.note_mode_midi.value();
                                    
                                    match note_mode {
                                        NoteModeMidi::Scale | NoteModeMidi::MidiScale => {
                                            // 12 pitch shifters mode: reuse shifters cyclically
                                            let low_note = self.params.global.low_note_off.value() as usize - 36;
                                            let mut pitch: [f32; 12] = [0.0; 12];
                                            
                                            // Single loop: compute pitch shifts and apply bandpass + gate
                                            let mut index = low_note % 12;
                                            for ap in self.audio_process96.iter_mut().filter(|ap| ap.note >= low_note as u8 && ap.note <= (self.params.global.high_note_off.value() as usize - 36) as u8) {
                                                if index >= 12 {
                                                    index = 0;
                                                }
                                                
                                                // Compute pitch shift if this note has a tuning
                                                if ap.tuning.is_some() {
                                                    pitch[index] = ap.process_pitch_only(*sample, i);
                                                }
                                                
                                                // Apply bandpass + gate using the pitch-shifted signal
                                                let input_param: f32 = if ap.note_pitch == 0 { self.params.audio_process.in_key_gain.value() } else if ap.note_pitch == -128 { self.params.audio_process.off_key_gain.value() } else { self.params.audio_process.tuning_gain.value() };
                                                audio_process += ap.process_bpf_with_gate(pitch[index], i, input_param, self.params.clone(), &self.buffer_config, size);
                                                index += 1;
                                            }
                                        }
                                        NoteModeMidi::MidiWhistle => {
                                            // 96 pitch shifters mode: each note has its own shifter
                                            for ap in self.audio_process96.iter_mut().filter(|ap| ap.note >= (self.params.global.low_note_off.value() as usize - 36) as u8 && ap.note <= (self.params.global.high_note_off.value() as usize - 36) as u8) {
                                                let input_param: f32 = if ap.note_pitch == 0 { self.params.audio_process.in_key_gain.value() } else if ap.note_pitch == -128 { self.params.audio_process.off_key_gain.value() } else { self.params.audio_process.tuning_gain.value() };
                                                audio_process += ap.process(*sample, self.params.clone(), i, input_param, &self.buffer_config, size);
                                            }
                                        }
                                    }
                                }
                                AlgorithmMode::Off => {
                                    // Off mode: Bandpass (IIR) -> Output
                                    for ap in self.audio_process96.iter_mut().filter(|ap| ap.note >= (self.params.global.low_note_off.value() as usize - 36) as u8 && ap.note <= (self.params.global.high_note_off.value() as usize - 36) as u8) {
                                        let input_param: f32 = if ap.note_pitch == 0 { self.params.audio_process.in_key_gain.value() } else if ap.note_pitch == -128 { self.params.audio_process.off_key_gain.value() } else { self.params.audio_process.tuning_gain.value() };
                                        audio_process += ap.process_bpf(delay, i, input_param, self.params.clone());
                                    }
                                }
                                AlgorithmMode::SineGen => {
                                    // SineGen mode: Bandpass (IIR) -> Sine Generator
                                    for ap in self.audio_process96.iter_mut().filter(|ap| ap.note >= (self.params.global.low_note_off.value() as usize - 36) as u8 && ap.note <= (self.params.global.high_note_off.value() as usize - 36) as u8) {
                                        let input_param: f32 = if ap.note_pitch == 0 { self.params.audio_process.in_key_gain.value() } else if ap.note_pitch == -128 { self.params.audio_process.off_key_gain.value() } else { self.params.audio_process.tuning_gain.value() };
                                        audio_process += ap.process_sine_gen_iir(delay, i, input_param, self.params.clone(), &self.buffer_config, size);
                                    }
                                }
                            }
                            *sample = (((audio_process * self.params.global.wet_gain.value()) + (delay * self.params.global.dry_gain.value()) + ((lpf_mute + hpf_mute) * self.params.global.lhf_gain.value())) * self.gate.get_param(flip, i)) * self.zero.get_param(false, i);
                            audio_process = 0.0;
                        }
                        if gate_on.1 || gate_zero.1 {
                            *sample = (delay * self.gate.get_param_inv(flip, i)) + if gate_on.0 && gate_zero.0 { *sample } else { 0.0 };
                        }
                    }
                }
            }
        }
        ProcessStatus::Normal
    }
}

// GUI removed: no Vizia Data/Model or font/resource helpers

impl ClapPlugin for CoPiReMapPlugin {
    const CLAP_ID: &'static str = "com.logiccuteguy.copiremap";
    const CLAP_DESCRIPTION: Option<&'static str> = None;
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Filter,
        ClapFeature::Equalizer,
    ];
}

// TODO: Interactive piano rows (click/drag to set low/high note)
impl Vst3Plugin for CoPiReMapPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"CoPiReMapPlugins";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx, Vst3SubCategory::Filter, Vst3SubCategory::Eq];
}

nih_export_clap!(CoPiReMapPlugin);
nih_export_vst3!(CoPiReMapPlugin);
