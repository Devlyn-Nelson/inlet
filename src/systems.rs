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
    axis::{AxisBinding, AxisBindingKind},
    button::ButtonBinding,
};

//TODO system that automatically detects gamepad connections and disconnection and tries to keep everyone connected.

pub fn gather_button_inputs<K, T>(
    mut writer: MessageWriter<T>,
    mut bindings: Query<&mut InputBindings<K, T>>,
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
    for mut bindings in bindings.iter_mut() {
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
        for bind in bindings.bindings.values_mut() {
            match bind {
                crate::InputBinding::Action(action_binding) => {
                    let pressed = check_button_bindings(
                        action_binding.bindings_mut(),
                        &gamepads,
                        keyboard.as_ref(),
                        mouse.as_ref(),
                        accumulated_mouse_motion.as_ref(),
                        accumulated_mouse_scroll.as_ref(),
                    );
                    if let Some(event) = action_binding.feed_event(pressed) {
                        writer.write(event);
                    }
                }
                crate::InputBinding::Value(value_binding) => {
                    let v = check_axis_bindings(
                        value_binding.bindings_mut(),
                        &gamepads,
                        keyboard.as_ref(),
                        mouse.as_ref(),
                        accumulated_mouse_motion.as_ref(),
                        accumulated_mouse_scroll.as_ref(),
                    );
                    if let Some(event) = value_binding.feed(v) {
                        writer.write(event);
                    }
                }
                crate::InputBinding::DualValue(dual_value_binding) => {
                    let x = check_axis_bindings(
                        dual_value_binding.x_bindings_mut(),
                        &gamepads,
                        keyboard.as_ref(),
                        mouse.as_ref(),
                        accumulated_mouse_motion.as_ref(),
                        accumulated_mouse_scroll.as_ref(),
                    );
                    let y = check_axis_bindings(
                        dual_value_binding.y_bindings_mut(),
                        &gamepads,
                        keyboard.as_ref(),
                        mouse.as_ref(),
                        accumulated_mouse_motion.as_ref(),
                        accumulated_mouse_scroll.as_ref(),
                    );
                    let v = Vec2::new(x, y);
                    if let Some(event) = dual_value_binding.feed(v) {
                        writer.write(event);
                    }
                }
            }
        }
    }
}

fn check_button_bindings(
    bindings: &mut [ButtonBinding],
    gamepads: &[&Gamepad],
    keyboard: &ButtonInput<KeyCode>,
    mouse: &ButtonInput<MouseButton>,
    accumulated_mouse_motion: &AccumulatedMouseMotion,
    accumulated_mouse_scroll: &AccumulatedMouseScroll,
) -> bool {
    let mut pressed = false;
    for button_binding in bindings {
        pressed |= check_button_binding_pressed(
            button_binding,
            gamepads,
            keyboard,
            mouse,
            accumulated_mouse_motion,
            accumulated_mouse_scroll,
        );
    }
    pressed
}

fn check_button_binding_just_pressed(
    binding: &mut ButtonBinding,
    gamepads: &[&Gamepad],
    keyboard: &ButtonInput<KeyCode>,
    mouse: &ButtonInput<MouseButton>,
    accumulated_mouse_motion: &AccumulatedMouseMotion,
    accumulated_mouse_scroll: &AccumulatedMouseScroll,
) -> bool {
    match binding {
        ButtonBinding::Gamepad(gamepad_button_type) => {
            for gpad in gamepads {
                if gpad.just_pressed(*gamepad_button_type) {
                    return true;
                }
            }
            false
        }
        ButtonBinding::Keyboard(key_code) => keyboard.just_pressed(*key_code),
        ButtonBinding::Mouse(key_code) => mouse.just_pressed(*key_code),
        ButtonBinding::Combo(combo) => {
            if check_button_binding_just_pressed(
                combo.next_binding(),
                gamepads,
                keyboard,
                mouse,
                accumulated_mouse_motion,
                accumulated_mouse_scroll,
            ) {
                combo.hit()
            } else {
                false
            }
        }
        ButtonBinding::Chord(button_chord) => {
            let mut out = true;
            for b in button_chord.bindings_mut() {
                if !check_button_binding_pressed(
                    b,
                    gamepads,
                    keyboard,
                    mouse,
                    accumulated_mouse_motion,
                    accumulated_mouse_scroll,
                ) {
                    out = false;
                    break;
                }
            }
            out
        }
        ButtonBinding::Axis(axis_binding) => {
            let value = check_axis_binding(
                axis_binding,
                gamepads,
                keyboard,
                mouse,
                accumulated_mouse_motion,
                accumulated_mouse_scroll,
            );
            value != 0.
        }
    }
}

fn check_button_binding_pressed(
    binding: &mut ButtonBinding,
    gamepads: &[&Gamepad],
    keyboard: &ButtonInput<KeyCode>,
    mouse: &ButtonInput<MouseButton>,
    accumulated_mouse_motion: &AccumulatedMouseMotion,
    accumulated_mouse_scroll: &AccumulatedMouseScroll,
) -> bool {
    match binding {
        ButtonBinding::Gamepad(gamepad_button_type) => {
            for gpad in gamepads {
                if gpad.pressed(*gamepad_button_type) {
                    return true;
                }
            }
            false
        }
        ButtonBinding::Keyboard(key_code) => keyboard.pressed(*key_code),
        ButtonBinding::Mouse(key_code) => mouse.pressed(*key_code),
        ButtonBinding::Combo(combo) => {
            if check_button_binding_just_pressed(
                combo.next_binding(),
                gamepads,
                keyboard,
                mouse,
                accumulated_mouse_motion,
                accumulated_mouse_scroll,
            ) {
                combo.hit()
            } else {
                false
            }
        }
        ButtonBinding::Chord(button_chord) => {
            let mut out = true;
            for b in button_chord.bindings_mut() {
                if !check_button_binding_pressed(
                    b,
                    gamepads,
                    keyboard,
                    mouse,
                    accumulated_mouse_motion,
                    accumulated_mouse_scroll,
                ) {
                    out = false;
                    break;
                }
            }
            out
        }
        ButtonBinding::Axis(axis_binding) => {
            let value = check_axis_binding(
                axis_binding,
                gamepads,
                keyboard,
                mouse,
                accumulated_mouse_motion,
                accumulated_mouse_scroll,
            );
            value != 0.
        }
    }
}

fn check_axis_bindings(
    bindings: &mut [AxisBinding],
    gamepads: &[&Gamepad],
    keyboard: &ButtonInput<KeyCode>,
    mouse: &ButtonInput<MouseButton>,
    accumulated_mouse_motion: &AccumulatedMouseMotion,
    accumulated_mouse_scroll: &AccumulatedMouseScroll,
) -> f32 {
    let mut value = 0.;
    let mut count = 0;
    for b in bindings {
        let v = check_axis_binding(
            b,
            gamepads,
            keyboard,
            mouse,
            accumulated_mouse_motion,
            accumulated_mouse_scroll,
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
    binding: &mut AxisBinding,
    gamepads: &[&Gamepad],
    keyboard: &ButtonInput<KeyCode>,
    mouse: &ButtonInput<MouseButton>,
    accumulated_mouse_motion: &AccumulatedMouseMotion,
    accumulated_mouse_scroll: &AccumulatedMouseScroll,
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
                    &mut plus.binding,
                    gamepads,
                    keyboard,
                    mouse,
                    accumulated_mouse_motion,
                    accumulated_mouse_scroll,
                ) {
                1.0
            } else {
                0.0
            };
            if let Some(minus) = minus
                && check_button_binding_pressed(
                    &mut minus.binding,
                    gamepads,
                    keyboard,
                    mouse,
                    accumulated_mouse_motion,
                    accumulated_mouse_scroll,
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
    };
    for m in binding.mods() {
        out = m.do_thing(out);
    }
    out
}
