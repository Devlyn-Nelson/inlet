use std::time::Duration;

use bevy::{color::palettes::basic, prelude::*};
use inlet::{
    InputBindingsSimple, InputManagementPluginSimple,
    button::{ActionBinding, ButtonChord, ButtonCombo},
    manager::{
        ClashSettings, ClashStrategy, ComboInterputSettings, ComboProgressionSettings,
        ComboSettings,
    },
};

const BUFFER_TIME: Duration = Duration::from_millis(500);
const AUTO_HOLD: Duration = Duration::from_millis(500);
const COMBO_TOLERANCE: Duration = Duration::from_millis(500);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InputManagementPluginSimple::<InputTypes>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (update, update_text))
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
    ToggleClashStrategy,
    ToggleChordRegression,
    ToggleComboInteruptSetting,
    ToggleComboProgressionSetting,
}

#[derive(Component)]
struct ClashStrategyText;
#[derive(Component)]
struct ChordRegressionText;
#[derive(Component)]
struct ComboInteruptText;
#[derive(Component)]
struct ComboProgressionText;

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
        ClashSettings::from(ClashStrategy::Unbuffered),
        ComboSettings::default().with_tolerence(COMBO_TOLERANCE),
        InputBindingsSimple::<InputTypes>::new()
            .with_action_binding(InputTypes::ToggleClashStrategy, KeyCode::F1.into())
            .with_action_binding(InputTypes::ToggleChordRegression, KeyCode::F2.into())
            .with_action_binding(InputTypes::ToggleComboInteruptSetting, KeyCode::F3.into())
            .with_action_binding(
                InputTypes::ToggleComboProgressionSetting,
                KeyCode::F4.into(),
            )
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
                ActionBinding::new_no_event(vec![
                    ButtonCombo::new(vec![KeyCode::KeyA.into(), KeyCode::KeyS.into()]).into(),
                ])
                .with_auto_hold(AUTO_HOLD),
            )
            .with_action_binding(
                InputTypes::ComboOneTwoThree,
                ActionBinding::new_no_event(vec![
                    ButtonCombo::new(vec![
                        KeyCode::KeyA.into(),
                        KeyCode::KeyS.into(),
                        KeyCode::KeyD.into(),
                    ])
                    .into(),
                ])
                .with_auto_hold(AUTO_HOLD),
            )
            .with_action_binding(
                InputTypes::ComboOneTwoThreeFour,
                ActionBinding::new_no_event(vec![
                    ButtonCombo::new(vec![
                        KeyCode::KeyA.into(),
                        KeyCode::KeyS.into(),
                        KeyCode::KeyD.into(),
                        KeyCode::KeyF.into(),
                    ])
                    .into(),
                ])
                .with_auto_hold(AUTO_HOLD),
            ),
    ));

    // Display the current clash settings.
    commands
        .spawn((Node::default(), Text::new("Settings:")))
        .with_children(|p| {
            p.spawn((TextSpan::new("\nF1 Clash Setting: "),))
                .with_child((
                    TextSpan::new(format!("{:?}", ClashSettings::default())),
                    ClashStrategyText,
                ));
            p.spawn((TextSpan::new("\nF2 Chord Regression: "),))
                .with_child((
                    TextSpan::new(format!("{:?}", ClashSettings::default())),
                    ChordRegressionText,
                ));
            p.spawn((TextSpan::new("\nF3 Combo Interupt Setting: "),))
                .with_child((
                    TextSpan::new(format!("{:?}", ComboInterputSettings::default())),
                    ComboInteruptText,
                ));
            p.spawn((TextSpan::new("\nF4 Combo Progression Setting: "),))
                .with_child((
                    TextSpan::new(format!("{:?}", ComboProgressionSettings::default())),
                    ComboProgressionText,
                ));
        });

    let red = materials.add(Color::from(basic::RED));
    let green = materials.add(Color::from(basic::GREEN));
    // TODO add labels for collums (A squares, S squares, D squares, F squares)
    // TODO add labels for rows (chrod squares, single-press squares, combo squares)

    // # Singles
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(red.clone()),
        Transform::default()
            .with_translation(Vec3::new(-256. - 128., 0., 0.))
            .with_scale(Vec3::splat(128.)),
        One,
    ));
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
    mut player: Single<(
        &InputBindingsSimple<InputTypes>,
        &mut ClashSettings,
        &mut ComboSettings,
    )>,
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
    if player.0.just_pressed(&InputTypes::ToggleClashStrategy) {
        let new_setting = match player.1.clash_strategy() {
            ClashStrategy::Unbuffered => ClashStrategy::BufferClashing(Some(BUFFER_TIME)),
            ClashStrategy::BufferClashing(_) => ClashStrategy::BufferAll(Some(BUFFER_TIME)),
            ClashStrategy::BufferAll(_) => ClashStrategy::Disabled,
            ClashStrategy::Disabled => ClashStrategy::Unbuffered,
        };
        player.1.set_clash_strategy(new_setting);
    }
    if player.0.just_pressed(&InputTypes::ToggleChordRegression) {
        let new_setting = !player.1.chord_regretion();
        player.1.set_chord_regretion(new_setting);
    }
    if player
        .0
        .just_pressed(&InputTypes::ToggleComboInteruptSetting)
    {
        let new_setting = match player.2.interupt_settings() {
            ComboInterputSettings::NoBreak => ComboInterputSettings::AnythingBreaks,
            ComboInterputSettings::ButtonsBreak => ComboInterputSettings::NoBreak,
            ComboInterputSettings::AnythingBreaks => ComboInterputSettings::ButtonsBreak,
        };
        player.2.set_interupt_settings(new_setting);
    }
    if player
        .0
        .just_pressed(&InputTypes::ToggleComboProgressionSetting)
    {
        let new_setting = match player.2.progression_settings() {
            ComboProgressionSettings::None => ComboProgressionSettings::NextMustBeReleased,
            ComboProgressionSettings::PreviousMustBeReleased => ComboProgressionSettings::None,
            ComboProgressionSettings::NextMustBeReleased => {
                ComboProgressionSettings::PreviousMustBeReleased
            }
        };
        player.2.set_progression_settings(new_setting);
    }
}

fn update_text(
    player: Single<(&ClashSettings, &ComboSettings)>,
    mut clash_strat_text: Single<
        &'static mut TextSpan,
        (
            With<ClashStrategyText>,
            Without<ComboInteruptText>,
            Without<ComboProgressionText>,
            Without<ChordRegressionText>,
        ),
    >,
    mut chord_regression_text: Single<
        &'static mut TextSpan,
        (
            With<ChordRegressionText>,
            Without<ClashStrategyText>,
            Without<ComboInteruptText>,
            Without<ComboProgressionText>,
        ),
    >,
    mut interupt_text: Single<
        &'static mut TextSpan,
        (
            With<ComboInteruptText>,
            Without<ClashStrategyText>,
            Without<ComboProgressionText>,
            Without<ChordRegressionText>,
        ),
    >,
    mut progression_text: Single<
        &'static mut TextSpan,
        (
            With<ComboProgressionText>,
            Without<ClashStrategyText>,
            Without<ComboInteruptText>,
            Without<ChordRegressionText>,
        ),
    >,
) {
    ***clash_strat_text = format!("{:?}", player.0.clash_strategy());
    ***chord_regression_text = format!("{}", player.0.chord_regretion());
    ***interupt_text = format!("{:?}", player.1.interupt_settings());
    ***progression_text = format!("{:?}", player.1.progression_settings());
}
