use std::time::Duration;

use bevy::{color::palettes::basic, prelude::*};
use inlet::{
    InputBindingsSimple, InputManagementPluginSimple,
    button::{ButtonChord, ButtonCombo},
    manager::ClashSettings,
};

const BUFFER_TIME: Duration = Duration::from_millis(500);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InputManagementPluginSimple::<InputTypes>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

/// All of the different controls that exist. These are the keys to bindings.
#[derive(Hash, PartialEq, Eq, Clone)]
enum InputTypes {
    One,
    Two,
    Three,
    Four,
    ChordOneTwo,
    ChordOneTwoThree,
    ChordOneTwoThreeFour,
    ComboOneTwo,
    ComboOneTwoThree,
    ComboOneTwoThreeFour,
    ToggleSettings,
}

#[derive(Component)]
struct ClashText;

// # Singles
#[derive(Component)]
struct One;
#[derive(Component)]
struct Two;
#[derive(Component)]
struct Three;
#[derive(Component)]
struct Four;

// # Chords
#[derive(Component)]
struct ChordOneTwo;
#[derive(Component)]
struct ChordOneTwoThree;
#[derive(Component)]
struct ChordOneTwoThreeFour;

// # Combos
#[derive(Component)]
struct ComboOneTwo;
#[derive(Component)]
struct ComboOneTwoThree;
#[derive(Component)]
struct ComboOneTwoThreeFour;

#[derive(Resource)]
struct Colors {
    red: Handle<ColorMaterial>,
    green: Handle<ColorMaterial>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Camera2d,
        ClashSettings::Unbuffered,
        InputBindingsSimple::<InputTypes>::new()
            .with_action_binding(InputTypes::ToggleSettings, KeyCode::F1.into())
            .with_action_binding(InputTypes::One, KeyCode::KeyA.into())
            .with_action_binding(InputTypes::Two, KeyCode::KeyS.into())
            .with_action_binding(InputTypes::Three, KeyCode::KeyD.into())
            .with_action_binding(InputTypes::Four, KeyCode::KeyF.into())
            .with_action_binding(
                InputTypes::ChordOneTwo,
                ButtonChord::new(vec![KeyCode::KeyA.into(), KeyCode::KeyS.into()]).into(),
            )
            .with_action_binding(
                InputTypes::ChordOneTwoThree,
                ButtonChord::new(vec![
                    KeyCode::KeyA.into(),
                    KeyCode::KeyS.into(),
                    KeyCode::KeyD.into(),
                ])
                .into(),
            )
            .with_action_binding(
                InputTypes::ChordOneTwoThreeFour,
                ButtonChord::new(vec![
                    KeyCode::KeyA.into(),
                    KeyCode::KeyS.into(),
                    KeyCode::KeyD.into(),
                    KeyCode::KeyF.into(),
                ])
                .into(),
            )
            .with_action_binding(
                InputTypes::ComboOneTwo,
                ButtonCombo::new(vec![KeyCode::KeyA.into(), KeyCode::KeyS.into()]).into(),
            )
            .with_action_binding(
                InputTypes::ComboOneTwoThree,
                ButtonCombo::new(vec![
                    KeyCode::KeyA.into(),
                    KeyCode::KeyS.into(),
                    KeyCode::KeyD.into(),
                ])
                .into(),
            )
            .with_action_binding(
                InputTypes::ComboOneTwoThreeFour,
                ButtonCombo::new(vec![
                    KeyCode::KeyA.into(),
                    KeyCode::KeyS.into(),
                    KeyCode::KeyD.into(),
                    KeyCode::KeyF.into(),
                ])
                .into(),
            ),
    ));

    // Display the current clash settings.
    commands
        .spawn((
            Text::new("Clash Settings: "),
            TextFont {
                font_size: FontSize::Px(32.0),
                ..default()
            },
        ))
        .with_child((
            TextSpan::new("Unbuffered"),
            TextFont {
                font_size: FontSize::Px(32.0),
                ..Default::default()
            },
            ClashText,
        ));

    let red = materials.add(Color::from(basic::RED));
    let green = materials.add(Color::from(basic::GREEN));

    // # Singles
    commands
        .spawn((
            Mesh2d(meshes.add(Rectangle::default())),
            MeshMaterial2d(red.clone()),
            Transform::default()
                .with_translation(Vec3::new(-256. - 128., 0., 0.))
                .with_scale(Vec3::splat(128.)),
            One,
        ))
        .with_child(Text::new("A"));
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(red.clone()),
        Transform::default()
            .with_translation(Vec3::new(-128., 0., 0.))
            .with_scale(Vec3::splat(128.)),
        Two,
    ));
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(red.clone()),
        Transform::default()
            .with_translation(Vec3::new(128., 0., 0.))
            .with_scale(Vec3::splat(128.)),
        Three,
    ));
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(red.clone()),
        Transform::default()
            .with_translation(Vec3::new(256. + 128., 0., 0.))
            .with_scale(Vec3::splat(128.)),
        Four,
    ));

    // # Combos
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(red.clone()),
        Transform::default()
            .with_translation(Vec3::new(-128.0, 264., 0.))
            .with_scale(Vec3::splat(128.)),
        ChordOneTwo,
    ));
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(red.clone()),
        Transform::default()
            .with_translation(Vec3::new(128., 264., 0.))
            .with_scale(Vec3::splat(128.)),
        ChordOneTwoThree,
    ));
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(red.clone()),
        Transform::default()
            .with_translation(Vec3::new(256. + 128., 264., 0.))
            .with_scale(Vec3::splat(128.)),
        ChordOneTwoThreeFour,
    ));

    // # Chords
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(red.clone()),
        Transform::default()
            .with_translation(Vec3::new(-128.0, -264., 0.))
            .with_scale(Vec3::splat(128.)),
        ComboOneTwo,
    ));
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(red.clone()),
        Transform::default()
            .with_translation(Vec3::new(128., -264., 0.))
            .with_scale(Vec3::splat(128.)),
        ComboOneTwoThree,
    ));
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(red.clone()),
        Transform::default()
            .with_translation(Vec3::new(256. + 128., -264., 0.))
            .with_scale(Vec3::splat(128.)),
        ComboOneTwoThreeFour,
    ));

    commands.insert_resource(Colors { red, green });
}

fn update(
    mut commands: Commands,
    colors: Option<Res<Colors>>,
    mut player: Single<(&InputBindingsSimple<InputTypes>, &mut ClashSettings)>,
    one: Single<Entity, With<One>>,
    two: Single<Entity, With<Two>>,
    three: Single<Entity, With<Three>>,
    four: Single<Entity, With<Four>>,
    chord_onetwo: Single<Entity, With<ChordOneTwo>>,
    chord_onetwothree: Single<Entity, With<ChordOneTwoThree>>,
    chord_onetwothreefour: Single<Entity, With<ChordOneTwoThreeFour>>,
    combo_onetwo: Single<Entity, With<ComboOneTwo>>,
    combo_onetwothree: Single<Entity, With<ComboOneTwoThree>>,
    combo_onetwothreefour: Single<Entity, With<ComboOneTwoThreeFour>>,
    mut clash_text: Single<&mut TextSpan, With<ClashText>>,
) {
    let Some(colors) = colors else {
        return;
    };

    // Singles
    match player.0.get_action_state(&InputTypes::One).kind() {
        inlet::button::ActionableState::JustPressed => {
            commands
                .get_entity(one.entity())
                .unwrap()
                .insert(MeshMaterial2d(colors.green.clone()));
        }
        inlet::button::ActionableState::JustReleased => {
            commands
                .get_entity(one.entity())
                .unwrap()
                .insert(MeshMaterial2d(colors.red.clone()));
        }
        _ => {}
    }

    match player.0.get_action_state(&InputTypes::Two).kind() {
        inlet::button::ActionableState::JustPressed => {
            commands
                .get_entity(two.entity())
                .unwrap()
                .insert(MeshMaterial2d(colors.green.clone()));
        }
        inlet::button::ActionableState::JustReleased => {
            commands
                .get_entity(two.entity())
                .unwrap()
                .insert(MeshMaterial2d(colors.red.clone()));
        }
        _ => {}
    }
    match player.0.get_action_state(&InputTypes::Three).kind() {
        inlet::button::ActionableState::JustPressed => {
            commands
                .get_entity(three.entity())
                .unwrap()
                .insert(MeshMaterial2d(colors.green.clone()));
        }
        inlet::button::ActionableState::JustReleased => {
            commands
                .get_entity(three.entity())
                .unwrap()
                .insert(MeshMaterial2d(colors.red.clone()));
        }
        _ => {}
    }
    match player.0.get_action_state(&InputTypes::Four).kind() {
        inlet::button::ActionableState::JustPressed => {
            commands
                .get_entity(four.entity())
                .unwrap()
                .insert(MeshMaterial2d(colors.green.clone()));
        }
        inlet::button::ActionableState::JustReleased => {
            commands
                .get_entity(four.entity())
                .unwrap()
                .insert(MeshMaterial2d(colors.red.clone()));
        }
        _ => {}
    }

    // # Chords
    match player.0.get_action_state(&InputTypes::ChordOneTwo).kind() {
        inlet::button::ActionableState::JustPressed => {
            commands
                .get_entity(chord_onetwo.entity())
                .unwrap()
                .insert(MeshMaterial2d(colors.green.clone()));
        }
        inlet::button::ActionableState::JustReleased => {
            commands
                .get_entity(chord_onetwo.entity())
                .unwrap()
                .insert(MeshMaterial2d(colors.red.clone()));
        }
        _ => {}
    }
    match player
        .0
        .get_action_state(&InputTypes::ChordOneTwoThree)
        .kind()
    {
        inlet::button::ActionableState::JustPressed => {
            commands
                .get_entity(chord_onetwothree.entity())
                .unwrap()
                .insert(MeshMaterial2d(colors.green.clone()));
        }
        inlet::button::ActionableState::JustReleased => {
            commands
                .get_entity(chord_onetwothree.entity())
                .unwrap()
                .insert(MeshMaterial2d(colors.red.clone()));
        }
        _ => {}
    }
    match player
        .0
        .get_action_state(&InputTypes::ChordOneTwoThreeFour)
        .kind()
    {
        inlet::button::ActionableState::JustPressed => {
            commands
                .get_entity(chord_onetwothreefour.entity())
                .unwrap()
                .insert(MeshMaterial2d(colors.green.clone()));
        }
        inlet::button::ActionableState::JustReleased => {
            commands
                .get_entity(chord_onetwothreefour.entity())
                .unwrap()
                .insert(MeshMaterial2d(colors.red.clone()));
        }
        _ => {}
    }

    // # Combos
    match player.0.get_action_state(&InputTypes::ComboOneTwo).kind() {
        inlet::button::ActionableState::JustPressed => {
            commands
                .get_entity(combo_onetwo.entity())
                .unwrap()
                .insert(MeshMaterial2d(colors.green.clone()));
        }
        inlet::button::ActionableState::JustReleased => {
            commands
                .get_entity(combo_onetwo.entity())
                .unwrap()
                .insert(MeshMaterial2d(colors.red.clone()));
        }
        _ => {}
    }
    match player
        .0
        .get_action_state(&InputTypes::ComboOneTwoThree)
        .kind()
    {
        inlet::button::ActionableState::JustPressed => {
            commands
                .get_entity(combo_onetwothree.entity())
                .unwrap()
                .insert(MeshMaterial2d(colors.green.clone()));
        }
        inlet::button::ActionableState::JustReleased => {
            commands
                .get_entity(combo_onetwothree.entity())
                .unwrap()
                .insert(MeshMaterial2d(colors.red.clone()));
        }
        _ => {}
    }
    match player
        .0
        .get_action_state(&InputTypes::ComboOneTwoThreeFour)
        .kind()
    {
        inlet::button::ActionableState::JustPressed => {
            commands
                .get_entity(combo_onetwothreefour.entity())
                .unwrap()
                .insert(MeshMaterial2d(colors.green.clone()));
        }
        inlet::button::ActionableState::JustReleased => {
            commands
                .get_entity(combo_onetwothreefour.entity())
                .unwrap()
                .insert(MeshMaterial2d(colors.red.clone()));
        }
        _ => {}
    }

    // # Settings
    if player.0.just_pressed(&InputTypes::ToggleSettings) {
        *player.1 = match *player.1 {
            ClashSettings::Unbuffered => {
                ***clash_text = format!("BufferClashing");
                ClashSettings::BufferClashing(Some(BUFFER_TIME))
            }
            ClashSettings::BufferClashing(_) => {
                ***clash_text = format!("BufferAll");
                ClashSettings::BufferAll(Some(BUFFER_TIME))
            }
            ClashSettings::BufferAll(_) => {
                ***clash_text = format!("Disabled");
                ClashSettings::Disabled
            }
            ClashSettings::Disabled => {
                ***clash_text = format!("Unbuffered");
                ClashSettings::Unbuffered
            }
        };
    }
}
