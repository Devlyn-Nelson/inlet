use std::hash::Hash;

use bevy::{
    input::{
        ButtonInput,
        gamepad::Gamepad,
        mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseButton},
    },
    math::Vec2,
    prelude::{KeyCode, MessageWriter, Query, Res},
};

use crate::{
    BindEvent, InputBindings,
    axis::{AxisBinding, AxisBindingKind, ValueState},
    button::{ButtonBinding, ButtonState},
    clash::ClashHandler,
};

//TODO system that automatically detects gamepad connections and disconnection and tries to keep everyone connected.

pub fn gather_button_inputs<K, T>(
    mut writer: MessageWriter<T>,
    mut bindings: Query<(&mut InputBindings<K, T>, Option<&mut ClashHandler>)>,
    gamepad_query: Query<&Gamepad>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    accumulated_mouse_scroll: Res<AccumulatedMouseScroll>,
    // gamepad_buttons: Res<ButtonInput<GamepadButton>>,
    // gamepad_axis: Res<Axis<GamepadAxis>>,
) where
    K: Hash + Eq + Send + Sync + 'static,
    T: BindEvent + 'static,
{
    let players = bindings.count();
    for (mut bindings, mut maybe_clash) in bindings.iter_mut() {
        if let Some(clash_handler) = &mut maybe_clash {
            if bindings.changed() {
                clash_handler.update_clash_list(&bindings.bindings);
            }
            clash_handler.tick();
        }
        let gamepads = if players == 1 {
            gamepad_query.iter().collect()
        } else if let Some(gamepad) = &bindings.assigned_gamepad {
            match gamepad_query.get(*gamepad) {
                Ok(inputs) => vec![inputs],
                Err(_) => {
                    bevy::log::warn!("controller disconnected");
                    bindings.assigned_gamepad = None;
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };
        let mut maybe_clash = maybe_clash.as_mut().map(|asdf| asdf.as_mut());
        // TODO figure out a way to do input binding sorting so non-buffered clash
        // detection can work better.
        //
        // if maybe_clash.map(|c| c.settings().needs_sorting()).unwrap_or_default() {
        //     for (key, binding) in bindings.bindings.iter() {
        //     }
        // }else{
        for bind in bindings.bindings.values_mut() {
            match bind {
                crate::InputBinding::Action(action_binding) => {
                    let pressed = check_button_bindings(
                        &action_binding.state,
                        &mut action_binding.bindings[..],
                        &gamepads,
                        keyboard.as_ref(),
                        mouse.as_ref(),
                        accumulated_mouse_motion.as_ref(),
                        accumulated_mouse_scroll.as_ref(),
                        &mut maybe_clash,
                        1,
                    );
                    if let Some(event) = action_binding.feed_event(pressed) {
                        writer.write(event);
                    }
                }
                crate::InputBinding::Value(value_binding) => {
                    let v = check_axis_bindings(
                        &value_binding.state,
                        &mut value_binding.bindings[..],
                        &gamepads,
                        keyboard.as_ref(),
                        mouse.as_ref(),
                        accumulated_mouse_motion.as_ref(),
                        accumulated_mouse_scroll.as_ref(),
                        &mut maybe_clash,
                        1,
                    );
                    if let Some(event) = value_binding.feed(v) {
                        writer.write(event);
                    }
                }
                crate::InputBinding::DualValue(dual_value_binding) => {
                    let x = check_axis_bindings(
                        &dual_value_binding.x_state,
                        &mut dual_value_binding.x_bindings[..],
                        &gamepads,
                        keyboard.as_ref(),
                        mouse.as_ref(),
                        accumulated_mouse_motion.as_ref(),
                        accumulated_mouse_scroll.as_ref(),
                        &mut maybe_clash,
                        1,
                    );
                    let y = check_axis_bindings(
                        &dual_value_binding.y_state,
                        &mut dual_value_binding.y_bindings[..],
                        &gamepads,
                        keyboard.as_ref(),
                        mouse.as_ref(),
                        accumulated_mouse_motion.as_ref(),
                        accumulated_mouse_scroll.as_ref(),
                        &mut maybe_clash,
                        1,
                    );
                    let v = Vec2::new(x, y);
                    if let Some(event) = dual_value_binding.feed(v) {
                        writer.write(event);
                    }
                }
            }
        }
        // }
    }
}

fn check_button_clash(
    binding: &mut ButtonBinding,
    maybe_clash: &mut Option<&mut ClashHandler>,
    pressed: bool,
    chord_length: usize,
) -> bool {
    if let Some(clasher) = maybe_clash {
        let mut allowed = false;
        for c in binding.clashables() {
            allowed |= clasher.poll_clash(&c, chord_length, pressed);
        }
        allowed && pressed
    } else {
        pressed
    }
}

fn check_button_bindings(
    current_state: &ButtonState,
    bindings: &mut [ButtonBinding],
    gamepads: &[&Gamepad],
    keyboard: &ButtonInput<KeyCode>,
    mouse: &ButtonInput<MouseButton>,
    accumulated_mouse_motion: &AccumulatedMouseMotion,
    accumulated_mouse_scroll: &AccumulatedMouseScroll,
    clash: &mut Option<&mut ClashHandler>,
    chord_length: usize,
) -> bool {
    let mut pressed = false;
    for button_binding in bindings {
        pressed |= check_button_binding_pressed(
            current_state,
            button_binding,
            gamepads,
            keyboard,
            mouse,
            accumulated_mouse_motion,
            accumulated_mouse_scroll,
            clash,
            chord_length,
        );
    }
    pressed
}

fn check_button_binding_pressed(
    current_state: &ButtonState,
    binding: &mut ButtonBinding,
    gamepads: &[&Gamepad],
    keyboard: &ButtonInput<KeyCode>,
    mouse: &ButtonInput<MouseButton>,
    accumulated_mouse_motion: &AccumulatedMouseMotion,
    accumulated_mouse_scroll: &AccumulatedMouseScroll,
    maybe_clash: &mut Option<&mut ClashHandler>,
    chord_length: usize,
) -> bool {
    match binding {
        ButtonBinding::Gamepad(gamepad_button_type) => {
            let mut out = false;
            for gpad in gamepads {
                if gpad.pressed(*gamepad_button_type) {
                    out |= true;
                    break;
                }
            }
            check_button_clash(binding, maybe_clash, out, chord_length)
        }
        ButtonBinding::Keyboard(key_code) => {
            let pressed = keyboard.pressed(*key_code);
            check_button_clash(binding, maybe_clash, pressed, chord_length)
        }
        ButtonBinding::Mouse(key_code) => {
            let pressed = mouse.pressed(*key_code);
            check_button_clash(binding, maybe_clash, pressed, chord_length)
        }
        ButtonBinding::Combo(combo) => {
            let pressed = check_button_binding_pressed(
                current_state,
                combo.expected_binding_mut(),
                gamepads,
                keyboard,
                mouse,
                accumulated_mouse_motion,
                accumulated_mouse_scroll,
                maybe_clash,
                chord_length,
            );
            let conditional = match combo.rules() {
                crate::button::ButtonComboRules::None => true,
                crate::button::ButtonComboRules::PreviousMustBeReleased => {
                    if let Some(p) = combo.previous_binding_mut() {
                        !check_button_binding_pressed(
                            current_state,
                            p,
                            gamepads,
                            keyboard,
                            mouse,
                            accumulated_mouse_motion,
                            accumulated_mouse_scroll,
                            maybe_clash,
                            chord_length,
                        )
                    } else {
                        true
                    }
                }
                crate::button::ButtonComboRules::NextMustBeReleased => {
                    if let Some(p) = combo.next_binding_mut() {
                        !check_button_binding_pressed(
                            current_state,
                            p,
                            gamepads,
                            keyboard,
                            mouse,
                            accumulated_mouse_motion,
                            accumulated_mouse_scroll,
                            maybe_clash,
                            chord_length,
                        )
                    } else {
                        true
                    }
                }
            };
            if conditional && pressed {
                combo.hit()
            } else {
                false
            }
        }
        ButtonBinding::Chord(button_chord) => {
            let mut out = true;
            let chord_length = button_chord.len();
            for b in button_chord.bindings_mut() {
                let pressed = check_button_binding_pressed(
                    current_state,
                    b,
                    gamepads,
                    keyboard,
                    mouse,
                    accumulated_mouse_motion,
                    accumulated_mouse_scroll,
                    &mut None,
                    chord_length,
                );
                if !pressed {
                    out = false;
                    break;
                }
            }
            if out && maybe_clash.is_some() {
                for b in button_chord.bindings_mut() {
                    if !check_button_clash(b, maybe_clash, true, chord_length) {
                        out = false;
                        break;
                    }
                }
            }
            out
        }
        ButtonBinding::Axis(axis_binding) => {
            let value = check_axis_binding(
                &current_state.value_state(),
                axis_binding,
                gamepads,
                keyboard,
                mouse,
                accumulated_mouse_motion,
                accumulated_mouse_scroll,
                maybe_clash,
                chord_length,
            );
            value != 0.
        }
        ButtonBinding::Mock(out) => *out,
    }
}

fn check_axis_bindings(
    current_state: &ValueState,
    bindings: &mut [AxisBinding],
    gamepads: &[&Gamepad],
    keyboard: &ButtonInput<KeyCode>,
    mouse: &ButtonInput<MouseButton>,
    accumulated_mouse_motion: &AccumulatedMouseMotion,
    accumulated_mouse_scroll: &AccumulatedMouseScroll,
    maybe_clash: &mut Option<&mut ClashHandler>,
    chord_length: usize,
) -> f32 {
    let mut value = 0.;
    let mut count = 0;
    for b in bindings {
        let v = check_axis_binding(
            current_state,
            b,
            gamepads,
            keyboard,
            mouse,
            accumulated_mouse_motion,
            accumulated_mouse_scroll,
            maybe_clash,
            chord_length,
        );
        if v != 0. {
            value += v;
            count += 1;
        }
    }
    if count == 0 {
        0.
    } else {
        value / (count as f32)
    }
}

fn check_axis_binding(
    current_state: &ValueState,
    binding: &mut AxisBinding,
    gamepads: &[&Gamepad],
    keyboard: &ButtonInput<KeyCode>,
    mouse: &ButtonInput<MouseButton>,
    accumulated_mouse_motion: &AccumulatedMouseMotion,
    accumulated_mouse_scroll: &AccumulatedMouseScroll,
    maybe_clash: &mut Option<&mut ClashHandler>,
    chord_length: usize,
) -> f32 {
    let mut out = match binding.kind_mut() {
        AxisBindingKind::GamepadAxis(gamepad_axis) => {
            let mut value = 0.;
            let mut count = 0;
            for gpad in gamepads {
                if let Some(v) = gpad.get(gamepad_axis.clone())
                    && v != 0.
                {
                    value += v;
                    count += 1;
                }
            }
            if count == 0 {
                0.
            } else {
                value / (count as f32)
            }
        }
        AxisBindingKind::GamepadButton(b) => {
            let mut value = 0.;
            let mut count = 0;
            for gpad in gamepads {
                if let Some(v) = gpad.get(b.clone())
                    && v != 0.
                {
                    value += v;
                    count += 1;
                }
            }
            if count == 0 {
                0.
            } else {
                value / (count as f32)
            }
        }
        AxisBindingKind::Buttons { plus, minus } => {
            let mut value = if let Some(plus) = plus
                && check_button_binding_pressed(
                    &current_state.action_state(),
                    &mut plus.binding,
                    gamepads,
                    keyboard,
                    mouse,
                    accumulated_mouse_motion,
                    accumulated_mouse_scroll,
                    maybe_clash,
                    chord_length,
                ) {
                1.0
            } else {
                0.0
            };
            if let Some(minus) = minus
                && check_button_binding_pressed(
                    &current_state.action_state(),
                    &mut minus.binding,
                    gamepads,
                    keyboard,
                    mouse,
                    accumulated_mouse_motion,
                    accumulated_mouse_scroll,
                    maybe_clash,
                    chord_length,
                )
            {
                value -= 1.0;
            }
            value
        }
        AxisBindingKind::Mouse(mouse_axis) => match mouse_axis {
            crate::axis::MouseAxis::MotionX => accumulated_mouse_motion.delta.x,
            crate::axis::MouseAxis::MotionY => accumulated_mouse_motion.delta.y,
            crate::axis::MouseAxis::ScrollX => accumulated_mouse_scroll.delta.x,
            crate::axis::MouseAxis::ScrollY => accumulated_mouse_scroll.delta.y,
        },
        AxisBindingKind::Mock(val) => *val,
    };
    for m in binding.mods() {
        out = m.do_thing(out);
    }
    out
}
