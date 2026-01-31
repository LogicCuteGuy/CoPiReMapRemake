use std::sync::Arc;
use nih_plug::prelude::*;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};

use crate::PluginParams;

// ============================================================================
// ASSETS - Fonts and Images
// ============================================================================

const FINGERCUTE: &[u8] = include_bytes!("../ui/resource/font/Fingercute-Regular.ttf");
const LOGO_PNG: &[u8] = include_bytes!("../ui/resource/img/LogicColour.png");

// ============================================================================
// COLORS
// ============================================================================

const CREAM: Color = Color::rgb(255, 248, 220);

// ============================================================================
// CUSTOM BYPASS IMAGE BUTTON
// ============================================================================

pub struct BypassImageButton {
    param_base: param_base::ParamWidgetBase,
}

impl BypassImageButton {
    pub fn new<L, Params, P, FMap>(
        cx: &mut Context,
        params: L,
        params_to_param: FMap,
    ) -> Handle<'_, Self>
    where
        L: Lens<Target = Params> + Clone,
        Params: 'static,
        P: Param + 'static,
        FMap: Fn(&Params) -> &P + Copy + 'static,
    {
        Self {
            param_base: param_base::ParamWidgetBase::new(cx, params.clone(), params_to_param),
        }
        .build(cx, |cx| {
            Image::new(cx, "logo")
                .size(Stretch(1.0))
                .hoverable(false);
        })
        .checked(param_base::ParamWidgetBase::make_lens(
            params,
            params_to_param,
            |param| param.modulated_normalized_value() >= 0.5,
        ))
    }

    fn toggle_value(&self, cx: &mut EventContext) {
        let current_value = self.param_base.unmodulated_normalized_value();
        let new_value = if current_value >= 0.5 { 0.0 } else { 1.0 };
        self.param_base.begin_set_parameter(cx);
        self.param_base.set_normalized_value(cx, new_value);
        self.param_base.end_set_parameter(cx);
    }
}

impl View for BypassImageButton {
    fn element(&self) -> Option<&'static str> {
        Some("bypass-image-button")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|window_event, meta| match window_event {
            WindowEvent::MouseDown(MouseButton::Left)
            | WindowEvent::MouseDoubleClick(MouseButton::Left)
            | WindowEvent::MouseTripleClick(MouseButton::Left) => {
                self.toggle_value(cx);
                meta.consume();
            }
            _ => {}
        });
    }
}

// ============================================================================
// EDITOR STATE
// ============================================================================

#[derive(Lens)]
pub struct EditorData {
    pub params: Arc<PluginParams>,
    pub latency: Arc<std::sync::atomic::AtomicU32>,
}

impl Model for EditorData {}

// ============================================================================
// EDITOR CREATION
// ============================================================================

pub fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (870, 500))
}

pub fn create_editor(
    params: Arc<PluginParams>,
    editor_state: Arc<ViziaState>,
    latency: Arc<std::sync::atomic::AtomicU32>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        // Register fonts
        assets::register_noto_sans_light(cx);
        cx.add_font_mem(FINGERCUTE);
        
        // Load logo image
        let logo_image = image::load_from_memory(LOGO_PNG)
            .expect("Failed to decode logo PNG");
        cx.load_image("logo", logo_image, ImageRetentionPolicy::Forever);

        // Add custom styles
        cx.add_stylesheet(STYLESHEET).expect("Failed to add stylesheet");

        // Build data model
        EditorData {
            params: params.clone(),
            latency: latency.clone(),
        }
        .build(cx);

        // Main container
        VStack::new(cx, |cx| {
            // ==================== HEADER ====================
            HStack::new(cx, |cx| {
                // Logo/Bypass button
                BypassImageButton::new(cx, EditorData::params, |p| &p.global.bypass)
                    .size(Pixels(65.0))
                    .class("logo-button");

                // Title
                Label::new(cx, "CoPiReMap")
                    .font_family(vec![FamilyOwned::Name(String::from("Fingercute"))])
                    .font_size(44.0)
                    .color(CREAM)
                    .left(Pixels(15.0))
                    .top(Stretch(1.0))
                    .bottom(Stretch(1.0));

                Element::new(cx).width(Stretch(1.0));

                // Gain knobs (right side)
                HStack::new(cx, |cx| {
                    // Dry Gain
                    VStack::new(cx, |cx| {
                        Label::new(cx, "Dry Gain").font_size(9.0).color(CREAM);
                        ParamSlider::new(cx, EditorData::params, |p| &p.global.dry_gain)
                            .class("knob");
                    })
                    .child_space(Stretch(1.0))
                    .row_between(Pixels(2.0));

                    // Wet Gain
                    VStack::new(cx, |cx| {
                        Label::new(cx, "Wet Gain").font_size(9.0).color(CREAM);
                        ParamSlider::new(cx, EditorData::params, |p| &p.global.wet_gain)
                            .class("knob");
                    })
                    .child_space(Stretch(1.0))
                    .row_between(Pixels(2.0));

                    // L/H F dB
                    VStack::new(cx, |cx| {
                        Label::new(cx, "L/H F dB").font_size(9.0).color(CREAM);
                        ParamSlider::new(cx, EditorData::params, |p| &p.global.lhf_gain)
                            .class("knob");
                    })
                    .child_space(Stretch(1.0))
                    .row_between(Pixels(2.0));
                })
                .col_between(Pixels(12.0))
                .top(Stretch(1.0))
                .bottom(Stretch(1.0));
            })
            .height(Pixels(80.0))
            .class("header");

            // ==================== MAIN CONTENT AREA ====================
            HStack::new(cx, |cx| {
                // ===== LEFT PANEL =====
                VStack::new(cx, |cx| {
                    // Low Note Off
                    VStack::new(cx, |cx| {
                        Label::new(cx, "Low Note Off").font_size(9.0).color(CREAM);
                        ParamSlider::new(cx, EditorData::params, |p| &p.global.low_note_off)
                            .class("slider-h");
                    })
                    .row_between(Pixels(2.0));

                    // Low Note Off Mute
                    HStack::new(cx, |cx| {
                        Label::new(cx, "Mute").font_size(9.0).color(CREAM);
                        ParamButton::new(cx, EditorData::params, |p| &p.global.low_note_off_mute)
                            .class("toggle-sm");
                    })
                    .col_between(Pixels(5.0))
                    .child_top(Stretch(1.0))
                    .child_bottom(Stretch(1.0));
                    
                    // Hz Center
                    VStack::new(cx, |cx| {
                        Label::new(cx, "Hz Center").font_size(10.0).color(CREAM);
                        ParamSlider::new(cx, EditorData::params, |p| &p.global.hz_center)
                            .class("slider-h");
                    })
                    .row_between(Pixels(2.0));

                    // InKey Gain knob
                    VStack::new(cx, |cx| {
                        ParamSlider::new(cx, EditorData::params, |p| &p.audio_process.in_key_gain)
                            .class("knob-blue");
                        Label::new(cx, "In Key Gain").font_size(8.0).color(CREAM);
                    })
                    .child_space(Stretch(1.0))
                    .row_between(Pixels(2.0));

                    // Tuning Gain knob
                    VStack::new(cx, |cx| {
                        ParamSlider::new(cx, EditorData::params, |p| &p.audio_process.tuning_gain)
                            .class("knob-blue");
                        Label::new(cx, "Tuning Gain").font_size(8.0).color(CREAM);
                    })
                    .child_space(Stretch(1.0))
                    .row_between(Pixels(2.0));
                })
                .width(Pixels(100.0))
                .class("side-panel");

                // ===== CENTER PANEL (Piano & Controls) =====
                VStack::new(cx, |cx| {
                    // Top row: Mute OffKey button and Round Up button
                    HStack::new(cx, |cx| {
                        VStack::new(cx, |cx| {
                            Label::new(cx, "Mute OffKey").font_size(10.0).color(CREAM);
                            ParamButton::new(cx, EditorData::params, |p| &p.key_note.mute_off_key)
                                .class("toggle-btn");
                        })
                        .row_between(Pixels(2.0));
                        
                        Element::new(cx).width(Stretch(1.0));
                        
                        VStack::new(cx, |cx| {
                            Label::new(cx, "Round Up").font_size(10.0).color(CREAM);
                            ParamButton::new(cx, EditorData::params, |p| &p.key_note.round_up)
                                .class("toggle-btn");
                        })
                        .row_between(Pixels(2.0));
                    })
                    .height(Pixels(50.0))
                    .left(Pixels(10.0))
                    .right(Pixels(10.0));

                    // ===== PIANO KEYBOARD =====
                    ZStack::new(cx, |cx| {
                        // White keys (bottom layer)
                        HStack::new(cx, |cx| {
                            ParamButton::new(cx, EditorData::params, |p| &p.key_note.note_c)
                                .class("white-key").with_label("C");
                            ParamButton::new(cx, EditorData::params, |p| &p.key_note.note_d)
                                .class("white-key").with_label("D");
                            ParamButton::new(cx, EditorData::params, |p| &p.key_note.note_e)
                                .class("white-key").with_label("E");
                            ParamButton::new(cx, EditorData::params, |p| &p.key_note.note_f)
                                .class("white-key").with_label("F");
                            ParamButton::new(cx, EditorData::params, |p| &p.key_note.note_g)
                                .class("white-key").with_label("G");
                            ParamButton::new(cx, EditorData::params, |p| &p.key_note.note_a)
                                .class("white-key").with_label("A");
                            ParamButton::new(cx, EditorData::params, |p| &p.key_note.note_b)
                                .class("white-key").with_label("B");
                        })
                        .col_between(Pixels(3.0))
                        .child_space(Stretch(1.0));

                        // Black keys (top layer)
                        HStack::new(cx, |cx| {
                            Element::new(cx).width(Pixels(30.0));
                            ParamButton::new(cx, EditorData::params, |p| &p.key_note.note_c_sharp)
                                .class("black-key").with_label("C#");
                            Element::new(cx).width(Pixels(8.0));
                            ParamButton::new(cx, EditorData::params, |p| &p.key_note.note_d_sharp)
                                .class("black-key").with_label("D#");
                            Element::new(cx).width(Pixels(50.0));
                            ParamButton::new(cx, EditorData::params, |p| &p.key_note.note_f_sharp)
                                .class("black-key").with_label("F#");
                            Element::new(cx).width(Pixels(8.0));
                            ParamButton::new(cx, EditorData::params, |p| &p.key_note.note_g_sharp)
                                .class("black-key").with_label("G#");
                            Element::new(cx).width(Pixels(8.0));
                            ParamButton::new(cx, EditorData::params, |p| &p.key_note.note_a_sharp)
                                .class("black-key").with_label("A#");
                        })
                        .top(Pixels(0.0))
                        .height(Pixels(55.0));
                    })
                    .class("piano-container");

                    // Note Mode/Midi selector
                    VStack::new(cx, |cx| {
                        Label::new(cx, "Note Mode/Midi").font_size(11.0).color(CREAM);
                        ParamSlider::new(cx, EditorData::params, |p| &p.key_note.note_mode_midi)
                            .class("slider-h");
                    })
                    .row_between(Pixels(3.0))
                    .left(Pixels(10.0))
                    .right(Pixels(10.0));
                })
                .width(Stretch(1.0))
                .class("center-panel");

                // ===== RIGHT PANEL =====
                VStack::new(cx, |cx| {
                    // High Note Off
                    VStack::new(cx, |cx| {
                        Label::new(cx, "High Note Off").font_size(9.0).color(CREAM);
                        ParamSlider::new(cx, EditorData::params, |p| &p.global.high_note_off)
                            .class("slider-h");
                    })
                    .row_between(Pixels(2.0));

                    // High Note Off Mute
                    HStack::new(cx, |cx| {
                        Label::new(cx, "Mute").font_size(9.0).color(CREAM);
                        ParamButton::new(cx, EditorData::params, |p| &p.global.high_note_off_mute)
                            .class("toggle-sm");
                    })
                    .col_between(Pixels(5.0))
                    .child_top(Stretch(1.0))
                    .child_bottom(Stretch(1.0));
                    
                    // Hz Tuning
                    VStack::new(cx, |cx| {
                        Label::new(cx, "Hz Tuning").font_size(10.0).color(CREAM);
                        ParamSlider::new(cx, EditorData::params, |p| &p.global.hz_tuning)
                            .class("slider-h");
                    })
                    .row_between(Pixels(2.0));

                    // Find Off Key knob
                    VStack::new(cx, |cx| {
                        ParamSlider::new(cx, EditorData::params, |p| &p.key_note.find_off_key)
                            .class("knob-gold");
                        Label::new(cx, "Find Off Key").font_size(8.0).color(CREAM);
                    })
                    .child_space(Stretch(1.0))
                    .row_between(Pixels(2.0));

                    // Off Key Gain knob  
                    VStack::new(cx, |cx| {
                        ParamSlider::new(cx, EditorData::params, |p| &p.audio_process.off_key_gain)
                            .class("knob-gold");
                        Label::new(cx, "Off Key Gain").font_size(8.0).color(CREAM);
                    })
                    .child_space(Stretch(1.0))
                    .row_between(Pixels(2.0));
                })
                .width(Pixels(100.0))
                .class("side-panel");
            })
            .height(Pixels(240.0))
            .class("main-content");

            // ==================== BOTTOM SECTION ====================
            HStack::new(cx, |cx| {
                // Global Threshold section
                VStack::new(cx, |cx| {
                    Label::new(cx, "Global Threshold").font_size(12.0).color(CREAM);
                    ParamSlider::new(cx, EditorData::params, |p| &p.global.global_threshold)
                        .class("slider-h");
                    
                    HStack::new(cx, |cx| {
                        VStack::new(cx, |cx| {
                            Label::new(cx, "Global Attack").font_size(8.0).color(CREAM);
                            ParamSlider::new(cx, EditorData::params, |p| &p.global.global_threshold_attack)
                                .class("slider-sm");
                        })
                        .row_between(Pixels(2.0));
                        VStack::new(cx, |cx| {
                            Label::new(cx, "Global Release").font_size(8.0).color(CREAM);
                            ParamSlider::new(cx, EditorData::params, |p| &p.global.global_threshold_release)
                                .class("slider-sm");
                        })
                        .row_between(Pixels(2.0));
                    })
                    .col_between(Pixels(8.0));
                    
                    HStack::new(cx, |cx| {
                        VStack::new(cx, |cx| {
                            Label::new(cx, "Mode").font_size(8.0).color(CREAM);
                            ParamSlider::new(cx, EditorData::params, |p| &p.global.global_threshold_mode)
                                .class("slider-sm");
                        })
                        .row_between(Pixels(2.0));
                        
                        VStack::new(cx, |cx| {
                            Label::new(cx, "Flip").font_size(8.0).color(CREAM);
                            ParamButton::new(cx, EditorData::params, |p| &p.global.global_threshold_flip)
                                .class("toggle-sm");
                        })
                        .row_between(Pixels(2.0));
                    })
                    .col_between(Pixels(8.0));
                })
                .row_between(Pixels(4.0))
                .class("section-box");

                // Center: Pitch Shift section
                VStack::new(cx, |cx| {
                    HStack::new(cx, |cx| {
                        VStack::new(cx, |cx| {
                            Label::new(cx, "Algorithm Mode").font_size(10.0).color(CREAM);
                            ParamSlider::new(cx, EditorData::params, |p| &p.audio_process.algorithm_mode)
                                .class("slider-sm");
                        })
                        .row_between(Pixels(2.0));
                        
                        VStack::new(cx, |cx| {
                            Label::new(cx, "OverSampling").font_size(8.0).color(CREAM);
                            ParamSlider::new(cx, EditorData::params, |p| &p.audio_process.pitch_shift_over_sampling)
                                .class("slider-sm");
                        })
                        .row_between(Pixels(2.0));
                    })
                    .col_between(Pixels(12.0));

                    VStack::new(cx, |cx| {
                        Label::new(cx, "Resonance").font_size(10.0).color(CREAM);
                        ParamSlider::new(cx, EditorData::params, |p| &p.audio_process.resonance)
                            .class("slider-h");
                    })
                    .row_between(Pixels(2.0));

                    HStack::new(cx, |cx| {
                        VStack::new(cx, |cx| {
                            Label::new(cx, "Window ms").font_size(8.0).color(CREAM);
                            ParamSlider::new(cx, EditorData::params, |p| &p.audio_process.pitch_shift_window_duration_ms)
                                .class("slider-sm");
                        })
                        .row_between(Pixels(2.0));
                    })
                    .col_between(Pixels(8.0));


                })
                .row_between(Pixels(4.0))
                .class("section-box");

                // Threshold section
                VStack::new(cx, |cx| {
                    Label::new(cx, "Threshold").font_size(12.0).color(CREAM);
                    ParamSlider::new(cx, EditorData::params, |p| &p.audio_process.threshold)
                        .class("slider-h");
                    
                    HStack::new(cx, |cx| {
                        VStack::new(cx, |cx| {
                            Label::new(cx, "Attack").font_size(8.0).color(CREAM);
                            ParamSlider::new(cx, EditorData::params, |p| &p.audio_process.threshold_attack)
                                .class("slider-sm");
                        })
                        .row_between(Pixels(2.0));
                        VStack::new(cx, |cx| {
                            Label::new(cx, "Release").font_size(8.0).color(CREAM);
                            ParamSlider::new(cx, EditorData::params, |p| &p.audio_process.threshold_release)
                                .class("slider-sm");
                        })
                        .row_between(Pixels(2.0));
                    })
                    .col_between(Pixels(8.0));
                    
                    HStack::new(cx, |cx| {
                        VStack::new(cx, |cx| {
                            Label::new(cx, "Mode").font_size(8.0).color(CREAM);
                            ParamSlider::new(cx, EditorData::params, |p| &p.audio_process.threshold_mode)
                                .class("slider-sm");
                        })
                        .row_between(Pixels(2.0));
                        
                        VStack::new(cx, |cx| {
                            Label::new(cx, "Flip").font_size(8.0).color(CREAM);
                            ParamButton::new(cx, EditorData::params, |p| &p.audio_process.threshold_flip)
                                .class("toggle-sm");
                        })
                        .row_between(Pixels(2.0));
                    })
                    .col_between(Pixels(8.0));
                })
                .row_between(Pixels(4.0))
                .class("section-box");
            })
            .height(Pixels(135.0))
            .class("bottom-section");

            // ==================== FOOTER ====================
            HStack::new(cx, |cx| {
                Label::new(cx, "LogicCuteGuy")
                    .font_family(vec![FamilyOwned::Name(String::from("Fingercute"))])
                    .font_size(14.0)
                    .color(CREAM);

                Element::new(cx).width(Stretch(1.0));

                Label::new(cx, EditorData::latency.map(|l| {
                    let latency_samples = l.load(std::sync::atomic::Ordering::Relaxed);
                    format!("Latency: {}smp", latency_samples)
                }))
                    .font_size(11.0)
                    .color(CREAM);
            })
            .height(Pixels(28.0))
            .class("footer");

            ResizeHandle::new(cx);
        })
        .class("main-window");
    })
}

// ============================================================================
// STYLESHEET
// ============================================================================

const STYLESHEET: &str = r#"
.main-window {
    background-color: #5a4080;
    child-space: 6px;
}

.header {
    background-color: transparent;
    child-left: 12px;
    child-right: 12px;
    child-top: 8px;
    child-bottom: 8px;
}

.logo-button {
    border-radius: 10px;
    border-width: 4px;
    border-color: #ff8c00;
    background-color: #2a1a40;
}

.logo-button:checked {
    border-color: #ff4500;
    background-color: #4a2a60;
}

.logo-button:hover {
    background-color: #3a2050;
}

.main-content {
    background-color: #6a5090;
    border-width: 5px;
    border-color: #ff8c00;
    border-radius: 18px;
    child-space: 8px;
    left: 10px;
    right: 10px;
}

.side-panel {
    child-space: 5px;
    row-between: 6px;
}

.center-panel {
    child-space: 5px;
    row-between: 8px;
}

.piano-container {
    background-color: #3a2a55;
    border-radius: 12px;
    border-width: 3px;
    border-color: #ffd700;
    height: 95px;
    left: 10px;
    right: 10px;
    child-top: 0px;
}

.white-key {
    width: 42px;
    height: 90px;
    background-color: #fffff8;
    border-radius: 0px 0px 6px 6px;
    border-width: 2px;
    border-color: #444;
    color: #333;
    font-size: 10;
    child-top: 1s;
    child-bottom: 5px;
}

.white-key:checked {
    background-color: #ffd700;
    color: #222;
}

.white-key:hover {
    background-color: #fffacd;
}

.black-key {
    width: 32px;
    height: 55px;
    background-color: #1a1a1a;
    border-radius: 0px 0px 5px 5px;
    border-width: 2px;
    border-color: #333;
    color: #ddd;
    font-size: 8;
    child-top: 1s;
    child-bottom: 3px;
}

.black-key:checked {
    background-color: #4169e1;
    color: #fff;
}

.black-key:hover {
    background-color: #333;
}

.knob {
    width: 52px;
    height: 52px;
    border-radius: 26px;
    border-width: 4px;
    border-color: #4169e1;
    background-color: #1a1a3a;
}

.knob:hover {
    border-color: #6495ed;
}

.knob-blue {
    width: 58px;
    height: 58px;
    border-radius: 29px;
    border-width: 5px;
    border-color: #4169e1;
    background-color: #1a1a3a;
}

.knob-blue:hover {
    border-color: #6495ed;
    background-color: #2a2a4a;
}

.knob-gold {
    width: 58px;
    height: 58px;
    border-radius: 29px;
    border-width: 5px;
    border-color: #daa520;
    background-color: #3a2a1a;
}

.knob-gold:hover {
    border-color: #ffd700;
    background-color: #4a3a2a;
}

.slider-h {
    width: 100%;
    height: 24px;
    border-radius: 12px;
    border-width: 2px;
    border-color: #daa520;
    background-color: #2a1a40;
}

.slider-h:hover {
    border-color: #ffd700;
}

.slider-sm {
    width: 85px;
    height: 20px;
    border-radius: 10px;
    border-width: 2px;
    border-color: #888;
    background-color: #2a1a40;
}

.slider-sm:hover {
    border-color: #aaa;
}

.toggle-btn {
    width: 50px;
    height: 32px;
    border-radius: 6px;
    border-width: 3px;
    border-color: #666;
    background-color: #2a1a40;
}

.toggle-btn:checked {
    background-color: #4169e1;
    border-color: #6495ed;
}

.toggle-btn:hover {
    background-color: #3a2a50;
}

.toggle-sm {
    width: 45px;
    height: 24px;
    border-radius: 5px;
    border-width: 2px;
    border-color: #666;
    background-color: #2a1a40;
}

.toggle-sm:checked {
    background-color: #4169e1;
    border-color: #6495ed;
}

.section-box {
    background-color: #5a4080;
    border-width: 4px;
    border-color: #ff8c00;
    border-radius: 14px;
    child-space: 10px;
    width: 1s;
}

.section-box:hover {
    background-color: #6a5090;
}

.bottom-section {
    child-left: 10px;
    child-right: 10px;
    col-between: 10px;
}

.footer {
    child-left: 15px;
    child-right: 15px;
    child-top: 1s;
    child-bottom: 1s;
}
"#;
