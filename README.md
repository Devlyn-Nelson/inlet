Input to Action Binding library for Bevy Game Engine.

# Features

- Maps Actions to input bindings
- Uses `bevy_input` internally, supports Keyboard, Gamepad, and Mouse.
- Can produce `Message` for common input events.
- `InputBinding` lets you bind any axis or button to any axis or button like input.
  - `ActionBinding` has internal states to best represent button like behavior: JustPressed, Pressed,
    JustReleased, Released. Can also be used as digital (-1, 0, 1) axis.
  - `ValueBinding` can return a value (-1.0 to 1.0) from any axis or set of buttons. Can have a stack
    of generic functions that modify the output. Can be used as a button, by default it is assumed any non-zero
    value is pressed, but modifiers can enable you to control this behavior more finely.
  - `DualValueBinding` internally behaves as if it is just 2 `ValueBinding`'s.
- `ButtonChord` (multiple buttons at once) with configurable settings for
  resolving clashing inputs.
- `ButtonCombo` (multiple sequentially pressed buttons). Think GTA cheats codes.

# Usage

> see `examples/events.rs` to see most of what can be done.

## Binding Types to be aware of

- `BevyInputKind` which is and enum that is either `BevyAxisKind` or `BevyButtonKind`. Both inner types just resolve down to types from `bevy_input`.
- `BevyAxisButton` this converts an axis to a button.
- `ButtonBinding` this what `inlet` uses as an actual binding to a button-like input. uses `BevyButtonKind` and `BevyAxisButton` to detect presses.
  - Can be configured to be a Chord (multiple buttons that must be pressed all at once).
  - Can be configured to be a Combo (multiple buttons pressed one after another).
- `AxisBinding` this what `inlet` uses as an actual binding to a axis-like input.

## Clash Settings

The behavior of clash detection can be configured on a per player bias (`ClashSettings`) or globally (`DefaultClashSettings`).

### Clash Strategies

- Unbuffered: Inputs that can clash will be rechecked after all inputs are checked at least once.
- BufferClashing: Inputs that can clash will not be reported for the initial frame they become active. Next frame or after provided `Duration` the action with the largest chord that is active will be given the inputs. Note that buttons that can NOT clash with other do NOT get buffered.
- BufferAll: This will cause ALL inputs to buffer. This exists to make things more consistent.
- Disabled: No clash prevention.

### Chord Regression

This enables the ability for chords of smaller lengths to become active after a longer chord was already active.

### Component

If the `ClashSettings` component is present on an entity that has a `InputBindings` the attached settings will be used.

### Resource

If an entity does not have a `ClashSettings` component attached the `DefaultClashSettings` resource will be used, otherwise `ClashSettings::default` will be used.

## Combo Settings

The behavior of combos can be configured on a per player bias (`ComboSettings`) or globally (`DefaultComboSettings`).
Maximum duration between combo button presses can be configured as well a Combo Interruption and Combo Progression

### Combo Interruption

- `NoBreak` Don't interrupt on incorrect button press.
- `ButtonsBreak` Interrupt if an incorrect button was pressed.
- `AnythingBreaks` Interrupt if any input change happened.

### Combo Progression

- `None` No Rules for progressing a combo.
- `PreviousMustBeReleased` The combo will not progress unless the previous expected button in not pressed.
- `NextMustBeReleased` The combo will not progress unless the next expected button in not pressed.

### Component

If the `ComboSettings` component is present on an entity that has a `InputBindings` the attached settings will be used.

### Resource

If an entity does not have a `ComboSettings` component attached the `DefaultComboSettings` resource will be used, otherwise `ComboSettings::default` will be used.

# Examples

## `visualizer`

There are 4 Columns, 1 for each input button (A, S, D, F).

There are 3 Rows:

- The top row is for chords (inputs that are held together). Each column represents a chord that requires that column's input and all inputs to the left to be pressed before the chord can be triggered.
- The middle row is for individual buttons.
- The bottom row is for combos. Like the top row, each column requires all inputs to the left, but instead of holding them all at the same time you must press them one after the other in order.

See [chord/clash Settings](#clash-settings) and [Combo Settings](#combo-settings) for the explanations for the toggle settings.

> Be aware that the time for buffered inputs is set really high (half a second) so if you toggle the clash settings to `BufferAll` no button, including the toggle buttons, will become active unless held for a the full half second.

#### Controls

| Action                           | Input |
| -------------------------------- | ----- |
| Trigger column 1                 | A     |
| Trigger column 2                 | S     |
| Trigger column 3                 | D     |
| Trigger column 4                 | F     |
| Toggle Clash Setting             | F1    |
| Toggle Chord Regression          | F2    |
| Toggle Combo Interupt Setting    | F3    |
| Toggle Combo Progression Setting | F4    |

## `events` and `poll-only`

| Action      | Input                            |
| ----------- | -------------------------------- |
| Move        | WASD                             |
| Jump        | Space                            |
| Zoom Camera | Mouse Scroll Wheel               |
| Grow        | W->S->D->A (One after the other) |
| Shrink      | W+A+S+D (All at once)            |

## Code Explanations

#### Simple (example)

Create a list of input bindings to be used as a key to register bindings and retrieve values.

This type MUST implement `Hash + PartialEq + Eq`

```
#[derive(Hash, PartialEq, Eq, Clone)]
enum InputTypes {
    Move,
    Zoom,
    Jump,
    SecretAbility1,
    SecretAbility2,
}
```

Create a Bindings component and add it to your entity.

```
InputBindings::<InputTypes>::new()
    // register a jump binding that triggers when either the space key or south on a gamepad is pressed.
    .with_action_binding(
        InputTypes::Jump,
        vec![KeyCode::Space.into(), GamepadButton::South.into()].into(),
    )
    // TODO added gamepad triggers as option for zoom.
    // register a zoom binding that reads values from the scroll wheel.
    .with_value_binding(
        InputTypes::Zoom,
        AxisBinding::mouse_y_scroll().invert().into(),
    )
    // register a move binding that gets the average non-zero value from the wasd on keyboard, the gamepads left stick and dpad.
    .with_dual_value_binding(
        InputTypes::Move,
        (
            vec![
                AxisBinding::keyboard_da(),
                AxisBinding::gamepad_left_stick_x(),
                AxisBinding::gamepad_dpad_right_left(),
            ],
            vec![
                AxisBinding::keyboard_ws(),
                AxisBinding::gamepad_left_stick_y(),
                AxisBinding::gamepad_dpad_up_down(),
            ],
        )
            .into(),
    )
    // register a cheat code binding activated by pressing forward ->
    .with_action_binding(
        InputTypes::SecretAbility1,
        (
            vec![
                // W -> S -> D -> A
                ButtonCombo::new(vec![
                    KeyCode::KeyW.into(),
                    KeyCode::KeyS.into(),
                    KeyCode::KeyD.into(),
                    KeyCode::KeyA.into(),
                ])
                .into(),
                // Up -> Down -> Right -> Left on dpad
                ButtonCombo::new(vec![
                    GamepadButton::DPadUp.into(),
                    GamepadButton::DPadDown.into(),
                    GamepadButton::DPadRight.into(),
                    GamepadButton::DPadLeft.into(),
                ])
                .into(),
                ButtonChord::new(vec![
                    BevyAxisButton::new_positive_only(GamepadAxis::LeftStickY.into())
                        .into(),
                    BevyAxisButton::new_negative_only(GamepadAxis::LeftStickY.into())
                        .into(),
                    BevyAxisButton::new_positive_only(GamepadAxis::LeftStickX.into())
                        .into(),
                    BevyAxisButton::new_negative_only(GamepadAxis::LeftStickX.into())
                        .into(),
                ])
                .into(),
            ],
            ButtonEventBinding::WhenPressed,
        )
            .into(),
    )
    .with_action_binding(
        InputTypes::SecretAbility2,
        (
            vec![
                ButtonChord::new(vec![
                    KeyCode::KeyW.into(),
                    KeyCode::KeyS.into(),
                    KeyCode::KeyA.into(),
                    KeyCode::KeyD.into(),
                ])
                .into(),
                ButtonChord::new(vec![
                    GamepadButton::DPadUp.into(),
                    GamepadButton::DPadDown.into(),
                    GamepadButton::DPadLeft.into(),
                    GamepadButton::DPadRight.into(),
                ])
                .into(),
            ],
            ButtonEventBinding::WhenPressed,
        )
            .into(),
    )
```

Make a system or systems that use the values from bindings

> you can also use polling in this system or other systems if you would like.

```
fn control_player(
    time: Res<Time>,
    mut player: Single<(&mut Transform, &InputBindings<InputTypes>)>,
    mut camera: Single<&mut Transform, (With<Camera3d>, Without<InputBindings<InputTypes>>)>,
) {
    let delta_time = time.delta_secs();
    let mover = player.1.get_dual_value(&InputTypes::Move);
    let y_scale = player.0.scale.y * 0.5;
    let mover = Vec3::new(
        // we are inverting x to make the movement in the demo feel more intuitive.
        // mostly because we are directly applying the input values to the translation
        // instead of doing math to make it move the way you might expect.
        -mover.x,
        if player.1.get_action_state(&InputTypes::Jump).just_pressed() {
            10.0 * y_scale
        } else {
            0.0
        },
        mover.y,
    );
    player.0.translation += mover * delta_time;
    let zoom = 1.0 + (player.1.get_value(&InputTypes::Zoom) * delta_time);
    camera.translation *= zoom;
}

fn accept_events(
    mut cheats: MessageReader<InletEvent<InputTypes>>,
    mut player: Single<&mut Transform, With<InputBindings<InputTypes>>>,
) {
    for cheat in cheats.read() {
        match cheat.kind {
            InputTypes::SecretAbility1 => player.scale += 1.,
            InputTypes::SecretAbility2 => player.scale -= 1.,
            _ => {}
        }
    }
}
```

Add `InputManagementPlugin<InputTypes, MessageType>::default()` and your system to your bevy app.
