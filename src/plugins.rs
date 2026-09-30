use std::{hash::Hash, marker::PhantomData};

use bevy::{
    app::{Plugin, PreUpdate},
    ecs::{
        lifecycle::Add,
        observer::On,
        schedule::IntoScheduleConfigs,
        system::{Commands, Query},
    },
    input::InputSystems,
};

use crate::{InletEvent, InputBindings, PlayerIndex, systems::system_gather_button_inputs};

pub trait InputKey: Hash + Eq + Clone {}

impl<T> InputKey for T where T: Hash + Eq + Clone {}

/// Plugin required for [`InputBindings`](crate::InputBindings) to function.
pub struct InputManagementPlugin<K>(PhantomData<K>);
impl<K> Plugin for InputManagementPlugin<K>
where
    K: InputKey + Sync + Send + 'static,
{
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            PreUpdate,
            system_gather_button_inputs::<K>.after(InputSystems),
        )
        .add_message::<InletEvent<K>>()
        .add_observer(observer_player_index_assign::<K>);
    }
}

fn observer_player_index_assign<K: Send + Sync + 'static>(
    binding: On<Add<InputBindings<K>>>,
    mut cmds: Commands,
    indices: Query<&PlayerIndex>,
) {
    if !indices.contains(binding.entity) {
        let mut taken = indices.iter().map(|pi| pi.0).collect::<Vec<usize>>();
        taken.sort();
        let mut i = 0;
        for t in taken {
            if t == i {
                i += 1;
            } else {
                break;
            }
        }
        cmds.entity(binding.entity).insert(PlayerIndex(i));
    }
}

impl<K> InputManagementPlugin<K>
where
    K: InputKey + Sync + Send + 'static,
{
    #[must_use]
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<K> Default for InputManagementPlugin<K>
where
    K: InputKey + Sync + Send + 'static,
{
    fn default() -> Self {
        InputManagementPlugin::<K>::new()
    }
}
