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
// CUSTOM BYPASS IMAGE BUTTON
// ============================================================================

/// A bypass button that displays an image and toggles the bypass parameter on click
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
}

impl Model for EditorData {}

// ============================================================================
// EDITOR CREATION
// ============================================================================

pub fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (800, 420))
}

pub fn create_editor(
    params: Arc<PluginParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        // Register fonts
        assets::register_noto_sans_light(cx);
        cx.add_font_mem(FINGERCUTE);
        
        // Load logo image - decode PNG bytes to DynamicImage
        let logo_image = image::load_from_memory(LOGO_PNG)
            .expect("Failed to decode logo PNG");
        cx.load_image("logo", logo_image, ImageRetentionPolicy::Forever);

        // Build data model
        EditorData {
            params: params.clone(),
        }
        .build(cx);

        // Main container with gradient background
        VStack::new(cx, |cx| {
            // ========== HEADER ==========
            HStack::new(cx, |cx| {
                // Bypass button with logo image
                BypassImageButton::new(cx, EditorData::params, |p| &p.global.bypass)
                    .size(Pixels(45.0))
                    .top(Stretch(1.0))
                    .bottom(Stretch(1.0));

                // Title
                Label::new(cx, "CoPiReMap")
                    .font_family(vec![FamilyOwned::Name(String::from("Fingercute"))])
                    .font_size(38.0)
                    .color(Color::white())
                    .left(Pixels(10.0))
                    .top(Stretch(1.0))
                    .bottom(Stretch(1.0));

                // Spacer
                Element::new(cx).width(Stretch(1.0));

                // Dry Gain
                VStack::new(cx, |cx| {
                    Label::new(cx, "Dry").font_size(10.0).color(Color::white());
                    ParamSlider::new(cx, EditorData::params, |p| &p.global.dry_gain)
                        .width(Pixels(70.0));
                })
                .child_space(Pixels(2.0))
                .row_between(Pixels(2.0));

                // Wet Gain
                VStack::new(cx, |cx| {
                    Label::new(cx, "Wet").font_size(10.0).color(Color::white());
                    ParamSlider::new(cx, EditorData::params, |p| &p.global.wet_gain)
                        .width(Pixels(70.0));
                })
                .child_space(Pixels(2.0))
                .row_between(Pixels(2.0));

                // L/H F Gain
                VStack::new(cx, |cx| {
                    Label::new(cx, "L/H F").font_size(10.0).color(Color::white());
                    ParamSlider::new(cx, EditorData::params, |p| &p.global.lhf_gain)
                        .width(Pixels(70.0));
                })
                .child_space(Pixels(2.0))
                .row_between(Pixels(2.0));
            })
            .height(Pixels(55.0))
            .child_left(Pixels(10.0))
            .child_right(Pixels(10.0))
            .col_between(Pixels(8.0));

            // Divider
            Element::new(cx)
                .height(Pixels(2.0))
                .background_color(Color::rgba(255, 255, 255, 80))
                .left(Pixels(10.0))
                .right(Pixels(10.0));

            // ========== PIANO BAR (Note Range) ==========
            HStack::new(cx, |cx| {
                // Low Mute
                VStack::new(cx, |cx| {
                    Label::new(cx, "Mute").font_size(9.0).color(Color::white());
                    ParamButton::new(cx, EditorData::params, |p| &p.global.low_note_off_mute)
                        .width(Pixels(35.0));
                })
                .child_space(Pixels(2.0));

                // Low Note
                VStack::new(cx, |cx| {
                    Label::new(cx, "Low Note").font_size(10.0).color(Color::white());
                    ParamSlider::new(cx, EditorData::params, |p| &p.global.low_note_off)
                        .width(Stretch(1.0));
                })
                .width(Stretch(1.0))
                .child_space(Pixels(2.0));

                // High Note
                VStack::new(cx, |cx| {
                    Label::new(cx, "High Note").font_size(10.0).color(Color::white());
                    ParamSlider::new(cx, EditorData::params, |p| &p.global.high_note_off)
                        .width(Stretch(1.0));
                })
                .width(Stretch(1.0))
                .child_space(Pixels(2.0));

                // High Mute
                VStack::new(cx, |cx| {
                    Label::new(cx, "Mute").font_size(9.0).color(Color::white());
                    ParamButton::new(cx, EditorData::params, |p| &p.global.high_note_off_mute)
                        .width(Pixels(35.0));
                })
                .child_space(Pixels(2.0));
            })
            .height(Pixels(45.0))
            .child_left(Pixels(10.0))
            .child_right(Pixels(10.0))
            .col_between(Pixels(5.0));

            // ========== MIDDLE ROW: Hz, Gains, Piano Keys ==========
            HStack::new(cx, |cx| {
                // Left side controls
                VStack::new(cx, |cx| {
                    // Hz Center
                    VStack::new(cx, |cx| {
                        Label::new(cx, "Hz Center").font_size(9.0).color(Color::white());
                        ParamSlider::new(cx, EditorData::params, |p| &p.global.hz_center)
                            .width(Pixels(80.0));
                    })
                    .row_between(Pixels(2.0));

                    // Hz Tuning
                    VStack::new(cx, |cx| {
                        Label::new(cx, "Hz Tuning").font_size(9.0).color(Color::white());
                        ParamSlider::new(cx, EditorData::params, |p| &p.global.hz_tuning)
                            .width(Pixels(80.0));
                    })
                    .row_between(Pixels(2.0));
                })
                .row_between(Pixels(5.0));

                // Gain controls
                VStack::new(cx, |cx| {
                    HStack::new(cx, |cx| {
                        VStack::new(cx, |cx| {
                            Label::new(cx, "InKey").font_size(8.0).color(Color::white());
                            ParamSlider::new(cx, EditorData::params, |p| &p.audio_process.in_key_gain)
                                .width(Pixels(55.0));
                        });
                        VStack::new(cx, |cx| {
                            Label::new(cx, "Tuning").font_size(8.0).color(Color::white());
                            ParamSlider::new(cx, EditorData::params, |p| &p.audio_process.tuning_gain)
                                .width(Pixels(55.0));
                        });
                    })
                    .col_between(Pixels(3.0));

                    HStack::new(cx, |cx| {
                        VStack::new(cx, |cx| {
                            Label::new(cx, "Mute").font_size(8.0).color(Color::white());
                            ParamButton::new(cx, EditorData::params, |p| &p.key_note.mute_off_key)
                                .width(Pixels(40.0));
                        });
                        VStack::new(cx, |cx| {
                            Label::new(cx, "Round").font_size(8.0).color(Color::white());
                            ParamButton::new(cx, EditorData::params, |p| &p.key_note.round_up)
                                .width(Pixels(40.0));
                        });
                    })
                    .col_between(Pixels(3.0));
                })
                .row_between(Pixels(3.0));

                // ===== 12 Piano Note Buttons =====
                VStack::new(cx, |cx| {
                    // Black keys row
                    HStack::new(cx, |cx| {
                        ParamButton::new(cx, EditorData::params, |p| &p.key_note.note_c_sharp)
                            .width(Pixels(28.0)).height(Pixels(22.0)).class("black-key");
                        ParamButton::new(cx, EditorData::params, |p| &p.key_note.note_d_sharp)
                            .width(Pixels(28.0)).height(Pixels(22.0)).class("black-key");
                        Element::new(cx).width(Pixels(20.0)); // Gap for E
                        ParamButton::new(cx, EditorData::params, |p| &p.key_note.note_f_sharp)
                            .width(Pixels(28.0)).height(Pixels(22.0)).class("black-key");
                        ParamButton::new(cx, EditorData::params, |p| &p.key_note.note_g_sharp)
                            .width(Pixels(28.0)).height(Pixels(22.0)).class("black-key");
                        ParamButton::new(cx, EditorData::params, |p| &p.key_note.note_a_sharp)
                            .width(Pixels(28.0)).height(Pixels(22.0)).class("black-key");
                    })
                    .col_between(Pixels(2.0))
                    .left(Pixels(15.0));

                    // White keys row
                    HStack::new(cx, |cx| {
                        ParamButton::new(cx, EditorData::params, |p| &p.key_note.note_c)
                            .width(Pixels(28.0)).height(Pixels(28.0)).class("white-key");
                        ParamButton::new(cx, EditorData::params, |p| &p.key_note.note_d)
                            .width(Pixels(28.0)).height(Pixels(28.0)).class("white-key");
                        ParamButton::new(cx, EditorData::params, |p| &p.key_note.note_e)
                            .width(Pixels(28.0)).height(Pixels(28.0)).class("white-key");
                        ParamButton::new(cx, EditorData::params, |p| &p.key_note.note_f)
                            .width(Pixels(28.0)).height(Pixels(28.0)).class("white-key");
                        ParamButton::new(cx, EditorData::params, |p| &p.key_note.note_g)
                            .width(Pixels(28.0)).height(Pixels(28.0)).class("white-key");
                        ParamButton::new(cx, EditorData::params, |p| &p.key_note.note_a)
                            .width(Pixels(28.0)).height(Pixels(28.0)).class("white-key");
                        ParamButton::new(cx, EditorData::params, |p| &p.key_note.note_b)
                            .width(Pixels(28.0)).height(Pixels(28.0)).class("white-key");
                    })
                    .col_between(Pixels(2.0));

                    // Note Mode
                    HStack::new(cx, |cx| {
                        Label::new(cx, "Mode").font_size(9.0).color(Color::white());
                        ParamSlider::new(cx, EditorData::params, |p| &p.key_note.note_mode_midi)
                            .width(Pixels(100.0));
                    })
                    .col_between(Pixels(5.0));
                })
                .row_between(Pixels(2.0));

                // Right side controls
                VStack::new(cx, |cx| {
                    HStack::new(cx, |cx| {
                        VStack::new(cx, |cx| {
                            Label::new(cx, "FindOff").font_size(8.0).color(Color::white());
                            ParamSlider::new(cx, EditorData::params, |p| &p.key_note.find_off_key)
                                .width(Pixels(45.0));
                        });
                        VStack::new(cx, |cx| {
                            Label::new(cx, "OffKey").font_size(8.0).color(Color::white());
                            ParamSlider::new(cx, EditorData::params, |p| &p.audio_process.off_key_gain)
                                .width(Pixels(55.0));
                        });
                    })
                    .col_between(Pixels(3.0));
                })
                .row_between(Pixels(5.0));
            })
            .height(Pixels(110.0))
            .child_left(Pixels(10.0))
            .child_right(Pixels(10.0))
            .col_between(Pixels(15.0));

            // Divider
            Element::new(cx)
                .height(Pixels(1.0))
                .background_color(Color::rgba(255, 255, 255, 60))
                .left(Pixels(10.0))
                .right(Pixels(10.0));

            // ========== BOTTOM ROW: Thresholds, Pitch Shift, Resonance ==========
            HStack::new(cx, |cx| {
                // Global Threshold Section
                VStack::new(cx, |cx| {
                    Label::new(cx, "Global Threshold").font_size(10.0).color(Color::white());
                    ParamSlider::new(cx, EditorData::params, |p| &p.global.global_threshold)
                        .width(Pixels(120.0));
                    HStack::new(cx, |cx| {
                        VStack::new(cx, |cx| {
                            Label::new(cx, "Atk").font_size(8.0).color(Color::white());
                            ParamSlider::new(cx, EditorData::params, |p| &p.global.global_threshold_attack)
                                .width(Pixels(45.0));
                        });
                        VStack::new(cx, |cx| {
                            Label::new(cx, "Rel").font_size(8.0).color(Color::white());
                            ParamSlider::new(cx, EditorData::params, |p| &p.global.global_threshold_release)
                                .width(Pixels(45.0));
                        });
                        VStack::new(cx, |cx| {
                            Label::new(cx, "Flip").font_size(8.0).color(Color::white());
                            ParamButton::new(cx, EditorData::params, |p| &p.global.global_threshold_flip)
                                .width(Pixels(30.0));
                        });
                    })
                    .col_between(Pixels(3.0));
                })
                .row_between(Pixels(2.0));

                Element::new(cx).width(Stretch(1.0));

                // Pitch Shift Section
                VStack::new(cx, |cx| {
                    Label::new(cx, "Pitch Shift").font_size(10.0).color(Color::white());
                    HStack::new(cx, |cx| {
                        VStack::new(cx, |cx| {
                            Label::new(cx, "P/S").font_size(8.0).color(Color::white());
                            ParamButton::new(cx, EditorData::params, |p| &p.audio_process.pitch_shift)
                                .width(Pixels(35.0));
                        });
                        VStack::new(cx, |cx| {
                            Label::new(cx, "Node").font_size(8.0).color(Color::white());
                            ParamSlider::new(cx, EditorData::params, |p| &p.audio_process.pitch_shift_node)
                                .width(Pixels(55.0));
                        });
                        VStack::new(cx, |cx| {
                            Label::new(cx, "OS").font_size(8.0).color(Color::white());
                            ParamSlider::new(cx, EditorData::params, |p| &p.audio_process.pitch_shift_over_sampling)
                                .width(Pixels(45.0));
                        });
                        VStack::new(cx, |cx| {
                            Label::new(cx, "Win").font_size(8.0).color(Color::white());
                            ParamSlider::new(cx, EditorData::params, |p| &p.audio_process.pitch_shift_window_duration_ms)
                                .width(Pixels(50.0));
                        });
                    })
                    .col_between(Pixels(3.0));

                    // Resonance
                    HStack::new(cx, |cx| {
                        Label::new(cx, "Resonance").font_size(9.0).color(Color::white());
                        ParamSlider::new(cx, EditorData::params, |p| &p.audio_process.resonance)
                            .width(Pixels(80.0));
                    })
                    .col_between(Pixels(5.0));
                })
                .row_between(Pixels(3.0));

                Element::new(cx).width(Stretch(1.0));

                // Threshold Section
                VStack::new(cx, |cx| {
                    Label::new(cx, "Threshold").font_size(10.0).color(Color::white());
                    ParamSlider::new(cx, EditorData::params, |p| &p.audio_process.threshold)
                        .width(Pixels(120.0));
                    HStack::new(cx, |cx| {
                        VStack::new(cx, |cx| {
                            Label::new(cx, "Atk").font_size(8.0).color(Color::white());
                            ParamSlider::new(cx, EditorData::params, |p| &p.audio_process.threshold_attack)
                                .width(Pixels(45.0));
                        });
                        VStack::new(cx, |cx| {
                            Label::new(cx, "Rel").font_size(8.0).color(Color::white());
                            ParamSlider::new(cx, EditorData::params, |p| &p.audio_process.threshold_release)
                                .width(Pixels(45.0));
                        });
                        VStack::new(cx, |cx| {
                            Label::new(cx, "Flip").font_size(8.0).color(Color::white());
                            ParamButton::new(cx, EditorData::params, |p| &p.audio_process.threshold_flip)
                                .width(Pixels(30.0));
                        });
                    })
                    .col_between(Pixels(3.0));
                })
                .row_between(Pixels(2.0));
            })
            .height(Pixels(95.0))
            .child_left(Pixels(10.0))
            .child_right(Pixels(10.0))
            .col_between(Pixels(10.0));

            // Spacer
            Element::new(cx).height(Stretch(1.0));

            // ========== FOOTER ==========
            HStack::new(cx, |cx| {
                Label::new(cx, "LogicCuteGuy")
                    .font_family(vec![FamilyOwned::Name(String::from("Fingercute"))])
                    .font_size(14.0)
                    .color(Color::white());

                Element::new(cx).width(Stretch(1.0));

                Label::new(cx, "CoPiReMap v0.0.1")
                    .font_family(vec![FamilyOwned::Name(String::from("Fingercute"))])
                    .font_size(12.0)
                    .color(Color::rgba(255, 255, 255, 180));
            })
            .height(Pixels(25.0))
            .child_left(Pixels(10.0))
            .child_right(Pixels(10.0));

            // Resize handle
            ResizeHandle::new(cx);
        })
        .background_gradient(
            LinearGradientBuilder::with_direction("120deg")
                .add_stop(Color::rgb(34, 86, 255))   // #2256ff
                .add_stop(Color::rgb(255, 59, 59))   // #ff3b3b  
                .add_stop(Color::rgb(255, 187, 0))   // #ffbb00
        )
        .class("main");
    })
}
