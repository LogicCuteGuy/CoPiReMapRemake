use nih_plug::params::{BoolParam, IntParam, EnumParam, Params};
use nih_plug::prelude::{Enum, IntRange};
use crate::{PluginParams};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use nih_plug::audio_setup::BufferConfig;
use crate::audio_process::AudioProcess96;

#[derive(Params)]
pub struct KeyNoteParams {
    #[id = "note_mode_midi"]
    pub note_mode_midi: EnumParam<NoteModeMidi>,

    #[id = "find_off_key"]
    pub find_off_key: IntParam,

    #[id = "mute_off_key"]
    pub mute_off_key: BoolParam,

    #[id = "round_up"]
    pub round_up: BoolParam,

    #[id = "note_c"]
    pub note_c: BoolParam,

    #[id = "note_c_sharp"]
    pub note_c_sharp: BoolParam,

    #[id = "note_d"]
    pub note_d: BoolParam,

    #[id = "note_d_sharp"]
    pub note_d_sharp: BoolParam,

    #[id = "note_e"]
    pub note_e: BoolParam,

    #[id = "note_f"]
    pub note_f: BoolParam,

    #[id = "note_f_sharp"]
    pub note_f_sharp: BoolParam,

    #[id = "note_g"]
    pub note_g: BoolParam,

    #[id = "note_g_sharp"]
    pub note_g_sharp: BoolParam,

    #[id = "note_a"]
    pub note_a: BoolParam,

    #[id = "note_a_sharp"]
    pub note_a_sharp: BoolParam,

    #[id = "note_b"]
    pub note_b: BoolParam,
}

impl KeyNoteParams {
    pub fn new(update_key_note: Arc<AtomicBool>, update_key_note_12: Arc<AtomicBool>) -> Self {

        Self {
            note_mode_midi: EnumParam::new("Note Mode/Midi", NoteModeMidi::Scale)
                .with_callback(
                {
                    let update_key_note = update_key_note.clone();
                    Arc::new(move |_| {
                        update_key_note.store(true, Ordering::Release);
                    })
                }
            ),
            find_off_key: IntParam::new("Find Off Key", 2, IntRange::Linear{ min: 0, max: 72 })
                .with_callback(
                {
                    let update_key_note = update_key_note.clone();
                    Arc::new(move |_| {
                        update_key_note.store(true, Ordering::Release);
                    })
                }
            ),
            mute_off_key: BoolParam::new("Mute Off Key", true),
            round_up: BoolParam::new("Round Up", false)
                .with_callback(
                {
                    let update_key_note = update_key_note.clone();
                    Arc::new(move |_| {
                        update_key_note.store(true, Ordering::Release);
                    })
                }
            ),
            note_c: BoolParam::new("Note C", false)
                .with_callback(
                {
                    let update_key_note_12 = update_key_note_12.clone();
                    Arc::new(move |_| {
                        update_key_note_12.store(true, Ordering::Release);
                    })
                }
            ),
            note_c_sharp: BoolParam::new("Note C#", false)
                .with_callback(
                {
                    let update_key_note_12 = update_key_note_12.clone();
                    Arc::new(move |_| {
                        update_key_note_12.store(true, Ordering::Release);
                    })
                }
            ),
            note_d: BoolParam::new("Note D", false)
                .with_callback(
                {
                    let update_key_note_12 = update_key_note_12.clone();
                    Arc::new(move |_| {
                        update_key_note_12.store(true, Ordering::Release);
                    })
                }
            ),
            note_d_sharp: BoolParam::new("Note D#", false)
                .with_callback(
                {
                    let update_key_note_12 = update_key_note_12.clone();
                    Arc::new(move |_| {
                        update_key_note_12.store(true, Ordering::Release);
                    })
                }
            ),
            note_e: BoolParam::new("Note E", false)
                .with_callback(
                {
                    let update_key_note_12 = update_key_note_12.clone();
                    Arc::new(move |_| {
                        update_key_note_12.store(true, Ordering::Release);
                    })
                }
            ),
            note_f: BoolParam::new("Note F", false)
                .with_callback(
                {
                    let update_key_note_12 = update_key_note_12.clone();
                    Arc::new(move |_| {
                        update_key_note_12.store(true, Ordering::Release);
                    })
                }
            ),
            note_f_sharp: BoolParam::new("Note F#", false)
                .with_callback(
                {
                    let update_key_note_12 = update_key_note_12.clone();
                    Arc::new(move |_| {
                        update_key_note_12.store(true, Ordering::Release);
                    })
                }
            ),
            note_g: BoolParam::new("Note G", false)
                .with_callback(
                {
                    let update_key_note_12 = update_key_note_12.clone();
                    Arc::new(move |_| {
                        update_key_note_12.store(true, Ordering::Release);
                    })
                }
            ),
            note_g_sharp: BoolParam::new("Note G#", false)
                .with_callback(
                {
                    let update_key_note_12 = update_key_note_12.clone();
                    Arc::new(move |_| {
                        update_key_note_12.store(true, Ordering::Release);
                    })
                }
            ),
            note_a: BoolParam::new("Note A", false)
                .with_callback(
                {
                    let update_key_note_12 = update_key_note_12.clone();
                    Arc::new(move |_| {
                        update_key_note_12.store(true, Ordering::Release);
                    })
                }
            ),
            note_a_sharp: BoolParam::new("Note A#", false)
                .with_callback(
                {
                    let update_key_note_12 = update_key_note_12.clone();
                    Arc::new(move |_| {
                        update_key_note_12.store(true, Ordering::Release);
                    })
                }
            ),
            note_b: BoolParam::new("Note B", false)
                .with_callback(
                {
                    let update_key_note_12 = update_key_note_12.clone();
                    Arc::new(move |_| {
                        update_key_note_12.store(true, Ordering::Release);
                    })
                }
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
#[non_exhaustive]
pub enum NoteModeMidi {
    #[id = "scale"]
    #[name = "Scale"]
    Scale,
    #[id = "midi_scale"]
    #[name = "Midi Scale"]
    MidiScale,
    #[id = "midi_whistle"]
    #[name = "Midi Whistle"]
    MidiWhistle,
}

pub struct MidiNote {
    pub midi_note: [bool; 96],
    pub i2t: [i8; 96],
    pub im2t: [i8; 96],
}

impl Default for MidiNote {
    fn default() -> Self {
        Self {
            midi_note: [false; 96],
            i2t: [0; 96],
            im2t: [0; 96]
        }
    }
}

impl MidiNote {

    pub fn update(&mut self, params: Arc<PluginParams>, audio_process: &mut [AudioProcess96], buffer_config: &BufferConfig) {
        let mut notes: [i8; 96];
        let mut note_on_keys = [false; 96];
        let mut notes_sel: [i8; 96] = [-128; 96];
        for i in 0..96 {
            let a: u8 = i % 12;
            match a {
                0 => { note_on_keys[i as usize] = params.key_note.note_c.value(); }
                1 => { note_on_keys[i as usize] = params.key_note.note_c_sharp.value(); }
                2 => { note_on_keys[i as usize] = params.key_note.note_d.value(); }
                3 => { note_on_keys[i as usize] = params.key_note.note_d_sharp.value(); }
                4 => { note_on_keys[i as usize] = params.key_note.note_e.value(); }
                5 => { note_on_keys[i as usize] = params.key_note.note_f.value(); }
                6 => { note_on_keys[i as usize] = params.key_note.note_f_sharp.value(); }
                7 => { note_on_keys[i as usize] = params.key_note.note_g.value(); }
                8 => { note_on_keys[i as usize] = params.key_note.note_g_sharp.value(); }
                9 => { note_on_keys[i as usize] = params.key_note.note_a.value(); }
                10 => { note_on_keys[i as usize] = params.key_note.note_a_sharp.value(); }
                11 => { note_on_keys[i as usize] = params.key_note.note_b.value(); }
                _ => {}
            }
        }
        self.find_off_key(params.clone(), &note_on_keys, &mut notes_sel);
        notes = notes_sel;
        self.i2t = notes;
        AudioProcess96::fn_update_pitch_shift_and_after_bandpass(params, audio_process, buffer_config, notes);
    }

    pub fn update_midi(&mut self, params: Arc<PluginParams>, audio_process: &mut [AudioProcess96], buffer_config: &BufferConfig) {
        let mut notes: [i8; 96] = [0; 96];
        match params.key_note.note_mode_midi.value() {
            NoteModeMidi::MidiScale => {
                let mut note_on_keys = [false; 96];
                let mut note_keys = [false; 12];
                let mut notes_sel: [i8; 96] = [-128; 96];
                for i in 0..96 {
                    let a: u8 = i % 12;
                    match self.midi_note[i as usize] {
                        true => { note_keys[a as usize] = true; }
                        false => {}
                    }
                }
                for i in 0..96 {
                    let a: u8 = i % 12;
                    match a {
                        0 => { note_on_keys[i as usize] = note_keys[0]; }
                        1 => { note_on_keys[i as usize] = note_keys[1]; }
                        2 => { note_on_keys[i as usize] = note_keys[2]; }
                        3 => { note_on_keys[i as usize] = note_keys[3]; }
                        4 => { note_on_keys[i as usize] = note_keys[4]; }
                        5 => { note_on_keys[i as usize] = note_keys[5]; }
                        6 => { note_on_keys[i as usize] = note_keys[6]; }
                        7 => { note_on_keys[i as usize] = note_keys[7]; }
                        8 => { note_on_keys[i as usize] = note_keys[8]; }
                        9 => { note_on_keys[i as usize] = note_keys[9]; }
                        10 => { note_on_keys[i as usize] = note_keys[10]; }
                        11 => { note_on_keys[i as usize] = note_keys[11]; }
                        _ => {}
                    }
                }
                self.find_off_key(params.clone(), &note_on_keys, &mut notes_sel);
                notes = notes_sel;
            }
            NoteModeMidi::MidiWhistle => {
                let mut notes_sel: [i8; 96] = [-128; 96];
                self.find_off_key(params.clone(), &self.midi_note, &mut notes_sel);
                notes = notes_sel;
            }
            _ => {}
        }
        self.im2t = notes;
        AudioProcess96::fn_update_pitch_shift_and_after_bandpass(params, audio_process, buffer_config, notes);
    }

    fn find_off_key(&self, params: Arc<PluginParams>, note_on_keys: &[bool; 96], notes_sel: &mut [i8; 96]) {
        for i in 0..params.key_note.find_off_key.value() {
            match params.key_note.round_up.value() {
                true => {
                    for (j, note_on_key) in note_on_keys.iter().enumerate() {
                        match note_on_key {
                            true => {
                                notes_sel[j] = 0;
                                if j as i8 - i as i8 > -1 && notes_sel[j - i as usize] == -128 {
                                    notes_sel[j - i as usize] = 0;
                                    notes_sel[j - i as usize] = i as i8;
                                }
                                if j + i as usize <= 95 && notes_sel[j + i as usize] == -128 {
                                    notes_sel[j + i as usize] = 0;
                                    notes_sel[j + i as usize] = -i as i8;
                                    
                                }
                            }
                            false => {

                            }
                        }
                    }
                }
                false => {
                    for (j, note_on_key) in note_on_keys.iter().enumerate().rev() {
                        match note_on_key {
                            true => {
                                notes_sel[j] = 0;
                                if j as i8 - i as i8 > -1 && notes_sel[j - i as usize] == -128 {
                                    notes_sel[j - i as usize] = 0;
                                    notes_sel[j - i as usize] = i as i8;
                                }
                                if j + i as usize <= 95 && notes_sel[j + i as usize] == -128 {
                                    notes_sel[j + i as usize] = 0;
                                    notes_sel[j + i as usize] = -i as i8;
                                }
                            }
                            false => {

                            }
                        }
                    }
                }
            }
        }
    }

    pub fn param_update(&mut self, params: Arc<PluginParams>, audio_process: &mut [AudioProcess96], buffer_config: &BufferConfig) {
        match params.key_note.note_mode_midi.value() {
            NoteModeMidi::MidiScale | NoteModeMidi::MidiWhistle => self.update_midi(params, audio_process, buffer_config),
            NoteModeMidi::Scale => self.update(params, audio_process, buffer_config),
        }
    }

}