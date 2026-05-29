pub mod axis;
pub mod button;
mod plugins;
mod systems;

use std::hash::Hash;

use button::ActionBinding;
pub use plugins::{InputManagementPlugin, InputManagementPluginSimple};

use bevy::{
    math::Vec2,
    platform::collections::HashMap,
    prelude::{Component, Entity, Message},
};

use crate::{
    axis::{DualValueBinding, ValueBinding},
    button::ButtonState,
};

pub trait BindEvent: Message {}

impl<T> BindEvent for T where T: Message {}

#[derive(Debug, Message)]
pub struct SimpleMessage;

pub type InputBindingsSimple<K> = InputBindings<K, SimpleMessage>;

pub enum InputBinding<T> {
    Action(ActionBinding<T>),
    Value(ValueBinding<T>),
    DualValue(DualValueBinding<T>),
}

impl<T> InputBinding<T> {
    pub fn state(&self) -> ButtonState {
        match self {
            InputBinding::Action(action_binding) => *action_binding.state(),
            InputBinding::Value(value_binding) => ButtonState {
                ty: if value_binding.value() == 0. {
                    button::ActionableState::Pressed
                } else {
                    button::ActionableState::Released
                },
                start: value_binding.last_change(),
            },
            InputBinding::DualValue(dual_value_binding) => {
                let out = dual_value_binding.value();

                ButtonState {
                    ty: if out.x == 0. && out.y == 0. {
                        button::ActionableState::Released
                    } else {
                        button::ActionableState::Pressed
                    },
                    start: dual_value_binding.last_change(),
                }
            }
        }
    }
    /// Returns a boolean value from the input.
    ///
    /// # [`ActionBinding`]
    ///
    /// Returns `true` if the button is pressed.
    ///
    /// # [`ValueBinding`]
    ///
    /// Returns `true` if the value is not 0.0.
    ///
    /// # [`DualValueBinding`]
    ///
    /// Returns `true` if the neither value is 0.0.
    pub fn pressed(&self) -> bool {
        match self {
            InputBinding::Action(action_binding) => action_binding.pressed(),
            InputBinding::Value(value_binding) => value_binding.value() != 0.,
            InputBinding::DualValue(dual_value_binding) => {
                let out = dual_value_binding.value();
                !(out.x == 0. || out.y == 0.)
            }
        }
    }
    /// Returns a single `f32` value from the input.
    ///
    /// # [`ActionBinding`]
    ///
    /// When called on actions or button bindings: 0.0 means unpressed, 1.0 means pressed.
    ///
    /// # [`ValueBinding`]
    ///
    /// This will just return the value from the [`ValueBinding`]
    ///
    /// # [`DualValueBinding`]
    ///
    /// The output will be the average of the two values output from the [`DualValueBinding`].
    pub fn value(&self) -> f32 {
        match self {
            InputBinding::Action(action_binding) => {
                if action_binding.pressed() {
                    1.0
                } else {
                    0.0
                }
            }
            InputBinding::Value(value_binding) => value_binding.value(),
            InputBinding::DualValue(dual_value_binding) => {
                let out = dual_value_binding.value();
                (out.x + out.y) * 0.5
            }
        }
    }
    /// Returns a [`Vec2`] from the input.
    ///
    /// # [`ActionBinding`]
    ///
    /// When called on actions or button bindings both values will be the same: 0.0 means unpressed, 1.0 means pressed.
    ///
    /// # [`ValueBinding`]
    ///
    /// This will just return the value from the [`ValueBinding`] for both values.
    ///
    /// # [`DualValueBinding`]
    ///
    /// Simply passes the output from the [`DualValueBinding`].
    pub fn dual_value(&self) -> Vec2 {
        match self {
            InputBinding::Action(action_binding) => {
                Vec2::splat(if action_binding.pressed() { 1.0 } else { 0.0 })
            }
            InputBinding::Value(value_binding) => Vec2::splat(value_binding.value()),
            InputBinding::DualValue(dual_value_binding) => dual_value_binding.value(),
        }
    }
}

pub struct Clash {}

pub enum ClashReport {
    Capture,
    Ignore,
    None,
}

// pub struct ClashHandler<K> {
//     // clashes:
// }

// impl<K> ClashHandler<K> {
//     /// Called when a new binding been inserted.
//     pub fn update_register(&mut self, key: &K) {
//         todo!();
//     }
//     /// Called when a change has been applied to a binding.
//     pub fn update_change(&mut self, key: &K, bindings: &HashMap<K, InputBinding<T>>) {
//         todo!();
//     }
//     /// checks for a clash possibility.
//     pub fn has_clash(&self, key: &K) -> ClashReport {
//         todo!();
//     }
// }

#[derive(Component)]
pub struct InputBindings<K, T: BindEvent> {
    pub(crate) bindings: HashMap<K, InputBinding<T>>,
    pub(crate) assigned_gamepad: Option<Entity>,
}

impl<K, T> InputBindings<K, T>
where
    K: Eq + Hash,
    T: BindEvent,
{
    pub fn with_action_binding(mut self, name: K, bindings: ActionBinding<T>) -> Self {
        self.register_action_binding(name, bindings);
        self
    }
    pub fn with_value_binding(mut self, name: K, bindings: ValueBinding<T>) -> Self {
        self.register_value_binding(name, bindings);
        self
    }
    pub fn with_dual_value_binding(mut self, name: K, bindings: DualValueBinding<T>) -> Self {
        self.register_dual_value_binding(name, bindings);
        self
    }
    pub fn register_binding(
        &mut self,
        name: K,
        bindings: InputBinding<T>,
    ) -> Option<InputBinding<T>> {
        self.bindings.insert(name, bindings)
    }
    pub fn register_action_binding(
        &mut self,
        name: K,
        bindings: ActionBinding<T>,
    ) -> Option<InputBinding<T>> {
        self.register_binding(name, InputBinding::Action(bindings))
    }
    pub fn register_value_binding(
        &mut self,
        name: K,
        bindings: ValueBinding<T>,
    ) -> Option<InputBinding<T>> {
        self.register_binding(name, InputBinding::Value(bindings))
    }
    pub fn register_dual_value_binding(
        &mut self,
        name: K,
        bindings: DualValueBinding<T>,
    ) -> Option<InputBinding<T>> {
        self.register_binding(name, InputBinding::DualValue(bindings))
    }
    pub fn new() -> Self {
        Self {
            bindings: HashMap::default(),
            assigned_gamepad: None,
        }
    }
    pub fn get_binding(&self, name: &K) -> Option<&InputBinding<T>> {
        self.bindings.get(name)
    }
    pub fn get_action_state(&self, name: &K) -> ButtonState {
        self.get_binding(name)
            .map(|binding| binding.state().clone())
            .unwrap_or_default()
    }
    pub fn just_pressed(&self, name: &K) -> bool {
        self.get_binding(name)
            .map(|binding| binding.state().just_pressed())
            .unwrap_or_default()
    }
    pub fn pressed(&self, name: &K) -> bool {
        self.get_binding(name)
            .map(|binding| binding.state().pressed())
            .unwrap_or_default()
    }
    pub fn just_released(&self, name: &K) -> bool {
        self.get_binding(name)
            .map(|binding| binding.state().just_released())
            .unwrap_or_default()
    }
    pub fn released(&self, name: &K) -> bool {
        self.get_binding(name)
            .map(|binding| binding.state().released())
            .unwrap_or_default()
    }
    pub fn get_value(&self, name: &K) -> f32 {
        self.get_binding(name)
            .map(|binding| binding.value())
            .unwrap_or_default()
    }
    pub fn get_dual_value(&self, name: &K) -> Vec2 {
        self.get_binding(name)
            .map(|binding| binding.dual_value())
            .unwrap_or_default()
    }
}
