Input library for Bevy Game Engine.

# Features

- Uses bevy_input internally, supports Keyboard, Gamepad, and Mouse.
- Uses any `InputKey` type for keying input types. 
- Can produce `Message`'s for common input events.
- `InputBinding` lets you bind any axis or button to any axis or button like input.
  - `ActionBinding` has internal states to best represent button like behavior: JustPressed, Pressed, JustReleased, Released. Can also be used as digital (-1, 0, 1) axis.
  - `ValueBinding` can return a value (-1.0 to 1.0) from any axis or set of buttons. Can have a stack of generic functions that modify the output. Can be used as a button, by default it is assumed any non-zero value is pressed, but modifiers can enable you to control this behavior more finely.
  - `DualValueBinding` internally behaves as if it is just 2 `ValueBinding`'s.

# Usage

> see `examples/events.rs` to see most of what can be done.

## Binding Types to be aware of

- `BevyInputKind` which is and enum that is either `BevyAxisKind` or `BevyButtonKind`. Both inner types just resolve down to types from `bevy_input`.
- `BevyAxisButton` this converts an axis to a button. 
- `ButtonBinding` this what `inlet` uses as an actual binding to a button-like input. uses `BevyButtonKind` and `BevyAxisButton` to detect presses. 
  - Can be configured to be a Chord (multiple buttons that must be pressed all at once).
  - Can be configured to be a Combo (multiple buttons pressed one after another).
- `AxisBinding` this what `inlet` uses as an actual binding to a axis-like input.


## Poll Only

Create a list of input bindings to be used as a key to register bindings and retrieve values.

This type MUST implement `Hash + Clone + Eq`

```
#[derive(Hash, PartialEq, Eq, Clone)]
enum InputTypes {
    Move,
    Zoom,
    Jump,
}
```

Create a Bindings component and add it to your entity.

> `SimpleInputBindings` is just a type definition that fills in the message type with a placeholder for when you don't want to deal with both generic types required for `InputBindings`.

```
SimpleInputBindings::<InputTypes>::new()
    .with_action_binding(
        InputTypes::Jump,
        vec![KeyCode::Space.into(), GamepadButton::South.into()].into(),
    )
    .with_value_binding(
        InputTypes::Zoom,
        AxisBinding::mouse_y_scroll().invert().into(),
    )
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
    );
```

Make a system that uses the values from bindings

```
fn control_player(
    time: Res<Time>,
    mut player: Single<(&mut Transform, &SimpleInputBindings<InputTypes>)>,
    mut camera: Single<
        &mut Transform,
        (With<Camera3d>, Without<SimpleInputBindings<InputTypes>>),
    >,
) {
    let delta_time = time.delta_secs();
    let mover = player.1.get_dual_value(&InputTypes::Move);
    let y_scale = player.0.scale.y * 0.5;
    let mover = Vec3::new(
        mover.x,
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
```

Add `SimpleInputManagementPlugin<InputTypes>::default()` and your system to your bevy app.

> `SimpleInputManagementPlugin` is just a type definition that fills in the message type with a placeholder type for when you don't want to deal with both generic types required for `InputManagementPlugin`.

## Message Based

Create a list of input bindings to be used as a key to register bindings and retrieve values.

This type MUST implement `Hash + PartialEq + Eq`

Also create a type that implements `Message`

> You can make only 1 type that gets used for both if you want. This example separates them
> simply to show they can be separate types for cases where you are mixing Message-Based and
> Polling-Based bindings.

```
#[derive(Hash, PartialEq, Eq, Clone)]
enum InputTypes {
    Grow,
    Shrink,
}

#[derive(Message)]
enum MessageType {
    Grow,
    Shrink,
}

// These functions are for giving to the bindings to create the messages.
impl MessageType {
    pub fn grow() -> Self {
        Self::Grow
    }
    pub fn shrink() -> Self {
        Self::Shrink
    }
}
```

Create a Bindings component and add it to your entity.

```
InputBindings::<InputTypes, MessageType>::new()
    .with_action_binding(
        InputTypes::Grow,
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
            ],
            ButtonEventBinding::WhenPressed(MessageType::grow),
        )
            .into(),
    )
    .with_action_binding(
        InputTypes::Shrink,
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
            ButtonEventBinding::WhenPressed(MessageType::shrink),
        )
            .into(),
    );
```

Make a system that uses the values from bindings

> you can also use polling in this system or other systems if you would like.

```
fn accept_events(
    mut messages: MessageReader<MessageType>,
    mut player: Single<&mut Transform, With<InputBindings<InputTypes, MessageType>>>,
) {
    for message in messages.read() {
        match cheat {
            MessageType::Grow => player.scale += 1.,
            MessageType::Shrink => player.scale -= 1.,
        }
    }
}
```

Add `InputManagementPlugin<InputTypes, MessageType>::default()` and your system to your bevy app.

# In Progress

- Interrupting Combos when an invalid button is pressed, with setting to disable interrupts.
- Better Clash Detection.
