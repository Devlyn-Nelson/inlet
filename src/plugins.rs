use std::{hash::Hash, marker::PhantomData};

use bevy::{
    app::{Plugin, PreUpdate},
    ecs::schedule::IntoScheduleConfigs,
    input::InputSystems,
};

use crate::{BindEvent, SimpleMessage, systems::gather_button_inputs};

pub type InputManagementPluginSimple<K> = InputManagementPlugin<K, SimpleMessage>;

pub trait InputKey: Hash + Eq + Clone {}

impl<T> InputKey for T where T: Hash + Eq + Clone {}

/// This will generate or load chunks based off the transforms of any entity with a [`ActiveChunkPosition`]
/// component on them, a sphere of chunks will load around those positions with the [`ChunksSettings`] width.
pub struct InputManagementPlugin<K, I>(PhantomData<K>, PhantomData<I>);
impl<K, I> Plugin for InputManagementPlugin<K, I>
where
    K: InputKey + Sync + Send + 'static,
    I: BindEvent + Sync + Send + 'static,
{
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(PreUpdate, gather_button_inputs::<K, I>.after(InputSystems))
            .add_message::<I>();
    }
}

impl<K, I> InputManagementPlugin<K, I>
where
    K: InputKey + Sync + Send + 'static,
    I: BindEvent + Sync + Send + 'static,
{
    #[must_use]
    pub fn new() -> Self {
        Self(PhantomData, PhantomData)
    }
}

impl<K, I> Default for InputManagementPlugin<K, I>
where
    K: InputKey + Sync + Send + 'static,
    I: BindEvent + Sync + Send + 'static,
{
    fn default() -> Self {
        InputManagementPlugin::<K, I>::new()
    }
}
