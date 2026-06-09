use bevy::{
    ecs::{entity::Entity, system::Commands, world::Mut},
    input::{
        ButtonInput,
        gamepad::Gamepad,
        mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseButton},
    },
    math::Vec2,
    prelude::{KeyCode, MessageWriter, Query, Res},
};

use crate::{
    BindEvent, InputBindings, InputValue,
    axis::{AxisBinding, AxisBindingKind},
    button::{ButtonBinding, ButtonCombo},
    manager::{ClashSettings, InputHandler},
    plugins::InputKey,
    pressed_to_value,
};

//TODO system that automatically detects gamepad connections and disconnection and tries to keep everyone connected.

/// This is for when a combos expected button is pressed. this will check the proper rules and check to
/// verify them.
fn expected_is_pressed(
    button_combo: &mut ButtonCombo,
    input_handler: &mut Mut<InputHandler>,
) -> InputValue {
    match button_combo.rules() {
        crate::button::ButtonComboRules::None => button_combo.hit().into(),
        crate::button::ButtonComboRules::PreviousMustBeReleased => {
            let prev = button_combo.previous_binding();
            // If a previous button exist check that it is released.
            if let Some(p) = prev {
                // Either differ to re-poll or if previous is not pressed hit the combo.
                let b = vec![p.kind()];
                if let Some(pre_asdf) = input_handler.poll(&b)
                    && !p.apply(pre_asdf)
                {
                    button_combo.hit().into()
                } else {
                    false.into()
                }
            } else {
                // No previous, hit because pressed.
                button_combo.hit().into()
            }
        }
        crate::button::ButtonComboRules::NextMustBeReleased => {
            let next = button_combo.next_binding();
            // If a next button exist check that it is released.
            if let Some(p) = next {
                // Either differ to re-poll or if next is not pressed hit the combo.
                let b = vec![p.kind()];
                if let Some(pre_asdf) = input_handler.poll(&b)
                    && !p.apply(pre_asdf)
                {
                    button_combo.hit().into()
                } else {
                    false.into()
                }
            } else {
                // No next, hit because pressed.
                button_combo.hit().into()
            }
        }
    }
}

pub fn gather_button_inputs<K, T>(
    mut commands: Commands,
    mut writer: MessageWriter<T>,
    mut bindings: Query<(
        Entity,
        &mut InputBindings<K, T>,
        Option<&mut InputHandler>,
        Option<&ClashSettings>,
    )>,
    gamepad_query: Query<&Gamepad>,
    default_clash_settings: Option<Res<ClashSettings>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    accumulated_mouse_scroll: Res<AccumulatedMouseScroll>,
) where
    K: InputKey + Send + Sync + 'static,
    T: BindEvent + 'static,
{
    let players = bindings.count();
    for (entity, mut bindings, mut input_handler, new_settings) in bindings.iter_mut() {
        let Some(input_handler) = &mut input_handler else {
            if let Ok(mut e_cmds) = commands.get_entity(entity) {
                let handler = if let Some(asdf) = &default_clash_settings {
                    (**asdf).into()
                } else {
                    InputHandler::default()
                };
                e_cmds.try_insert(handler);
            }
            continue;
        };
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

        if let Some(new) = new_settings {
            if let Ok(mut e_cmds) = commands.get_entity(entity) {
                e_cmds.try_remove::<ClashSettings>();
            }
            input_handler.set_settings(*new);
            bindings.change();
        }

        if bindings.changed() {
            input_handler.update_list(&bindings.bindings);
        } else {
            input_handler.tick();
        }

        input_handler.update(
            &gamepads,
            keyboard.as_ref(),
            mouse.as_ref(),
            accumulated_mouse_motion.as_ref(),
            accumulated_mouse_scroll.as_ref(),
        );

        // TODO figure out a way to do input binding sorting so non-buffered clash
        // detection can work better.
        //
        // if maybe_clash.map(|c| c.settings().needs_sorting()).unwrap_or_default() {
        //     for (key, binding) in bindings.bindings.iter() {
        //     }
        // }else{
        let mut repoll = Vec::default();
        // TODO check if repoll can use key references instead of cloned.
        for (key, bind) in bindings.bindings.iter_mut() {
            match bind {
                crate::InputBinding::Action(action_binding) => {
                    let mut pressed = action_binding.mocked;
                    let mut re = Vec::default();
                    for (i, button_binding) in action_binding.bindings.iter_mut().enumerate() {
                        let v = match button_binding {
                            ButtonBinding::Chord(button_chord) => input_handler
                                .poll(&button_chord.input_kinds())
                                .map(|v| button_chord.apply(v)),
                            ButtonBinding::Combo(button_combo) => {
                                // Either differ to re-poll or check if the next expected button is pressed.
                                let expected = button_combo.expected_binding_mut();
                                let out = input_handler.poll(&vec![expected.kind()]);
                                if let Some(o) = out {
                                    if expected.apply(o) {
                                        Some(
                                            expected_is_pressed(button_combo, input_handler)
                                                .is_pressed(),
                                        )
                                    } else {
                                        Some(false)
                                    }
                                } else {
                                    None
                                }
                            }
                            ButtonBinding::Single(bevy_input_kind) => input_handler
                                .poll(&[bevy_input_kind.kind()])
                                .map(|v| v.is_pressed()),
                        };
                        if let Some(p) = v {
                            pressed |= p;
                        } else {
                            re.push(i);
                        }
                    }
                    if re.is_empty() {
                        if let Some(event) = action_binding.feed(pressed) {
                            writer.write(event);
                        }
                    } else {
                        repoll.push(Repoll {
                            key: key.clone(),
                            x_i: re,
                            x: InputValue::Pressed(pressed),
                            y: InputValue::default(),
                            y_i: vec![],
                        });
                    }
                }
                crate::InputBinding::Value(value_binding) => {
                    match check_axes(
                        &mut value_binding.bindings,
                        input_handler,
                        value_binding.mock,
                    ) {
                        Ok(v) => {
                            if let Some(event) = value_binding.feed(v) {
                                writer.write(event);
                            }
                        }
                        Err((value, re)) => repoll.push(Repoll {
                            key: key.clone(),
                            x: InputValue::Value(value),
                            x_i: re,
                            y: InputValue::default(),
                            y_i: vec![],
                        }),
                    }
                }
                crate::InputBinding::DualValue(dual_value_binding) => {
                    let x = check_axes(
                        &mut dual_value_binding.x_bindings,
                        input_handler,
                        dual_value_binding.x_mock,
                    );
                    let y = check_axes(
                        &mut dual_value_binding.y_bindings,
                        input_handler,
                        dual_value_binding.y_mock,
                    );

                    match (x, y) {
                        (Ok(x), Ok(y)) => {
                            let v = Vec2::new(x, y);
                            if let Some(event) = dual_value_binding.feed(v) {
                                writer.write(event);
                            }
                        }
                        (Err((x, x_i)), Err((y, y_i))) => {
                            repoll.push(Repoll {
                                key: key.clone(),
                                x: x.into(),
                                x_i,
                                y: y.into(),
                                y_i,
                            });
                        }
                        (Err((x, x_i)), Ok(y)) => {
                            repoll.push(Repoll {
                                key: key.clone(),
                                x: x.into(),
                                x_i,
                                y: y.into(),
                                y_i: vec![],
                            });
                        }
                        (Ok(x), Err((y, y_i))) => {
                            repoll.push(Repoll {
                                key: key.clone(),
                                x: x.into(),
                                x_i: vec![],
                                y: y.into(),
                                y_i,
                            });
                        }
                    }
                }
            }
        }
        for r in repoll {
            if let Some(bind) = bindings.bindings.get_mut(&r.key) {
                match bind {
                    crate::InputBinding::Action(action_binding) => {
                        let mut pressed = false;
                        for index in r.x_i {
                            let button_binding = &mut action_binding.bindings[index];
                            pressed |= match button_binding {
                                ButtonBinding::Chord(button_chord) => {
                                    let out = input_handler.repoll(&button_chord.input_kinds());
                                    button_chord.apply(out)
                                }
                                ButtonBinding::Combo(button_combo) => {
                                    let b = button_combo.expected_binding_mut();
                                    let out = input_handler.repoll(&vec![b.kind()]);
                                    if b.apply(out) {
                                        expected_is_pressed(button_combo, input_handler)
                                            .is_pressed()
                                    } else {
                                        false
                                    }
                                }
                                ButtonBinding::Single(bevy_input_kind) => {
                                    let b = vec![bevy_input_kind.kind()];
                                    input_handler.repoll(&b).is_pressed()
                                }
                            };
                        }
                        if let Some(event) = action_binding.feed(pressed) {
                            writer.write(event);
                        }
                    }
                    crate::InputBinding::Value(value_binding) => {
                        let v = re_check_axes(
                            &value_binding.bindings,
                            &r.x_i,
                            r.x.get_value(),
                            input_handler,
                        );
                        if let Some(event) = value_binding.feed(v) {
                            writer.write(event);
                        }
                    }
                    crate::InputBinding::DualValue(dual_value_binding) => {
                        let x = re_check_axes(
                            &dual_value_binding.x_bindings,
                            &r.x_i,
                            r.x.get_value(),
                            input_handler,
                        );
                        let y = re_check_axes(
                            &dual_value_binding.y_bindings,
                            &r.y_i,
                            r.y.get_value(),
                            input_handler,
                        );
                        if let Some(event) = dual_value_binding.feed(Vec2 { x, y }) {
                            writer.write(event);
                        }
                    }
                }
            }
        }
    }
}

struct Repoll<K> {
    key: K,
    x: InputValue,
    x_i: Vec<usize>,
    y: InputValue,
    y_i: Vec<usize>,
}

fn check_axes(
    bindings: &mut [AxisBinding],
    handler: &mut InputHandler,
    mock: Option<f32>,
) -> Result<f32, (f32, Vec<usize>)> {
    let mut re = Vec::default();
    let (mut value, mut count) = if let Some(m) = mock { (m, 1) } else { (0., 0) };
    for (i, b) in bindings.iter_mut().enumerate() {
        let v = match b.kind() {
            AxisBindingKind::Single(bevy_input_kind) => handler.poll(&[*bevy_input_kind]),
            AxisBindingKind::Buttons { plus, minus } => {
                let p = if let Some(binding) = plus {
                    handler
                        .poll(&[binding.kind()])
                        .map(|out| pressed_to_value(binding.apply(out)))
                } else {
                    Some(0.)
                };
                let m = if let Some(binding) = minus {
                    handler
                        .poll(&[binding.kind()])
                        .map(|out| pressed_to_value(binding.apply(out)))
                } else {
                    Some(0.)
                };
                if let (Some(p), Some(m)) = (p, m) {
                    Some(InputValue::Value(p - m))
                } else {
                    None
                }
            }
        };
        let v = if let Some(v) = v {
            v
        } else {
            re.push(i);
            continue;
        };
        if v.is_pressed() {
            value += v.get_value();
            count += 1;
        }
    }
    let avg = if count == 0 {
        0.
    } else {
        value / (count as f32)
    };
    if re.is_empty() {
        Ok(avg)
    } else {
        Err((avg, re))
    }
}

fn re_check_axes(
    bindings: &[AxisBinding],
    indexes: &[usize],
    mut value: f32,
    handler: &mut InputHandler,
) -> f32 {
    let mut count = if value == 0. { 0 } else { 1 };
    for i in indexes.iter() {
        let b = &bindings[*i];
        let v = match b.kind() {
            AxisBindingKind::Single(bevy_input_kind) => {
                handler.repoll(&[*bevy_input_kind]).get_value()
            }
            AxisBindingKind::Buttons { plus, minus } => {
                let p = if let Some(binding) = plus {
                    let out = handler.repoll(&[binding.kind()]);
                    pressed_to_value(binding.apply(out))
                } else {
                    0.
                };
                let m = if let Some(binding) = minus {
                    let out = handler.repoll(&[binding.kind()]);
                    pressed_to_value(binding.apply(out))
                } else {
                    0.
                };
                p - m
            }
        };
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
