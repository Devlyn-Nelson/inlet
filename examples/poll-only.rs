use bevy::prelude::*;
use inlet::{
    InputBindingsSimple, InputManagementPluginSimple,
    axis::AxisBinding,
    button::{ButtonChord, ButtonCombo},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Add the plugin, if you are not using [`Message`] types use [`SimpleInputManagementPlugin`]
        // instead because it only need the InputType.
        .add_plugins(InputManagementPluginSimple::<InputTypes>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (gravity, control_player))
        .run();
}

/// All of the different controls that exist. These are the keys to bindings.
#[derive(Hash, PartialEq, Eq, Clone)]
enum InputTypes {
    Move,
    Zoom,
    Jump,
    SecretAbility1,
    SecretAbility2,
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(20.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // cube
    commands.spawn((
        // Provide bindings to a entity.
        //
        // If you are using [`SimpleInputManagementPlugin`] because you don't using messages
        // use [`SimpleInputBindings`] here instead.
        InputBindingsSimple::<InputTypes>::new()
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
                vec![
                    // W -> S -> D -> A
                    ButtonCombo::new_default_rules(vec![
                        KeyCode::KeyW.into(),
                        KeyCode::KeyS.into(),
                        KeyCode::KeyD.into(),
                        KeyCode::KeyA.into(),
                    ])
                    .into(),
                    // Up -> Down -> Right -> Left on dpad
                    ButtonCombo::new_default_rules(vec![
                        GamepadButton::DPadUp.into(),
                        GamepadButton::DPadDown.into(),
                        GamepadButton::DPadRight.into(),
                        GamepadButton::DPadLeft.into(),
                    ])
                    .into(),
                ]
                .into(),
            )
            .with_action_binding(
                InputTypes::SecretAbility2,
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
                ]
                .into(),
            ),
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-10., 18., -36.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn control_player(
    time: Res<Time>,
    mut player: Single<(&mut Transform, &InputBindingsSimple<InputTypes>)>,
    mut camera: Single<&mut Transform, (With<Camera3d>, Without<InputBindingsSimple<InputTypes>>)>,
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
    if player.1.just_pressed(&InputTypes::SecretAbility1) {
        player.0.scale += 1.
    }
    if player.1.just_pressed(&InputTypes::SecretAbility2) {
        player.0.scale -= 1.
    }
}

fn gravity(
    time: Res<Time>,
    mut player: Single<&mut Transform, With<InputBindingsSimple<InputTypes>>>,
) {
    let delta_time = time.delta_secs();
    let ground_y = player.scale.y * 0.5;
    let new = player.translation.y - (18. * delta_time * delta_time);
    if new <= ground_y {
        player.translation.y = ground_y;
    } else {
        player.translation.y = new;
    }
}
