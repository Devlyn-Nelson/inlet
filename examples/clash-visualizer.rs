use bevy::{color::palettes::basic, prelude::*};
use inlet::{
    InputBindingsSimple, InputManagementPluginSimple, button::ButtonChord,
};

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
}

#[derive(Component)]
struct One;
#[derive(Component)]
struct Two;
#[derive(Component)]
struct Three;

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
        InputBindingsSimple::<InputTypes>::new()
            .with_action_binding(InputTypes::One, KeyCode::KeyA.into())
            .with_action_binding(
                InputTypes::Two,
                ButtonChord::new(vec![KeyCode::KeyA.into(), KeyCode::KeyS.into()]).into(),
            )
            .with_action_binding(
                InputTypes::Three,
                ButtonChord::new(vec![
                    KeyCode::KeyA.into(),
                    KeyCode::KeyS.into(),
                    KeyCode::KeyD.into(),
                ])
                .into(),
            ),
    ));

    let red = materials.add(Color::from(basic::RED));
    let green = materials.add(Color::from(basic::GREEN));

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(red.clone()),
        Transform::default()
            .with_translation(Vec3::new(-256., 0., 0.))
            .with_scale(Vec3::splat(128.)),
        One,
    ));
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(red.clone()),
        Transform::default().with_scale(Vec3::splat(128.)),
        Two,
    ));
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(red.clone()),
        Transform::default()
            .with_translation(Vec3::new(256., 0., 0.))
            .with_scale(Vec3::splat(128.)),
        Three,
    ));

    commands.insert_resource(Colors { red, green });
}

fn update(
    mut commands: Commands,
    colors: Option<Res<Colors>>,
    controller: Single<&InputBindingsSimple<InputTypes>>,
    one: Single<Entity, With<One>>,
    two: Single<Entity, With<Two>>,
    three: Single<Entity, With<Three>>,
) {
    let Some(colors) = colors else {
        return;
    };
    match controller.get_action_state(&InputTypes::One).kind() {
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
    match controller.get_action_state(&InputTypes::Two).kind() {
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
    match controller.get_action_state(&InputTypes::Three).kind() {
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
}
