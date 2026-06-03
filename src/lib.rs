pub mod axis;
pub mod button;
// pub mod clash;
pub mod org;
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
    org::BevyInputKind,
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
    pub fn mock_press(&mut self, pressed: bool) {
        match self {
            InputBinding::Action(action_binding) => action_binding.mock(pressed),
            InputBinding::Value(value_binding) => value_binding.mock(pressed_to_value(pressed)),
            InputBinding::DualValue(dual_value_binding) => {
                dual_value_binding.mock_x(pressed_to_value(pressed));
                dual_value_binding.mock_y(pressed_to_value(pressed));
            }
        }
    }
    pub fn mock_value(&mut self, value: f32) {
        match self {
            InputBinding::Action(action_binding) => {
                action_binding.mock(value_to_press(value));
            }
            InputBinding::Value(value_binding) => value_binding.mock(value),
            InputBinding::DualValue(dual_value_binding) => {
                dual_value_binding.mock_x(value);
                dual_value_binding.mock_y(value);
            }
        }
    }
    pub fn mock_x_value(&mut self, value: f32) {
        match self {
            InputBinding::Action(action_binding) => {
                action_binding.mock(value_to_press(value));
            }
            InputBinding::Value(value_binding) => value_binding.mock(value),
            InputBinding::DualValue(dual_value_binding) => {
                dual_value_binding.mock_x(value);
            }
        }
    }
    pub fn mock_y_value(&mut self, value: f32) {
        match self {
            InputBinding::Action(action_binding) => {
                action_binding.mock(value_to_press(value));
            }
            InputBinding::Value(value_binding) => value_binding.mock(value),
            InputBinding::DualValue(dual_value_binding) => {
                dual_value_binding.mock_y(value);
            }
        }
    }
    pub fn mock_clear(&mut self) {
        match self {
            InputBinding::Action(action_binding) => action_binding.mock_clear(),
            InputBinding::Value(value_binding) => value_binding.mock_clear(),
            InputBinding::DualValue(dual_value_binding) => {
                dual_value_binding.mock_clear();
            }
        }
    }
    pub fn input_kinds(&self) -> Vec<BevyInputKind> {
        match self {
            InputBinding::Action(action_binding) => action_binding.input_kinds(),
            InputBinding::Value(value_binding) => value_binding.input_kinds(),
            InputBinding::DualValue(dual_value_binding) => dual_value_binding.input_kinds(),
        }
    }
    pub fn state(&self) -> ButtonState {
        match self {
            InputBinding::Action(action_binding) => *action_binding.state(),
            InputBinding::Value(value_binding) => ButtonState {
                kind: if value_binding.value() == 0. {
                    button::ActionableState::Pressed
                } else {
                    button::ActionableState::Released
                },
                start: value_binding.last_transition(),
            },
            InputBinding::DualValue(dual_value_binding) => {
                let out = dual_value_binding.value();

                ButtonState {
                    kind: if out.x == 0. && out.y == 0. {
                        button::ActionableState::Released
                    } else {
                        button::ActionableState::Pressed
                    },
                    start: dual_value_binding.last_transition(),
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
            InputBinding::Value(value_binding) => value_to_press(value_binding.value()),
            InputBinding::DualValue(dual_value_binding) => {
                let out = dual_value_binding.value();
                value_to_press(out.x) && value_to_press(out.y)
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
            InputBinding::Action(action_binding) => pressed_to_value(action_binding.pressed()),
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
                Vec2::splat(pressed_to_value(action_binding.pressed()))
            }
            InputBinding::Value(value_binding) => Vec2::splat(value_binding.value()),
            InputBinding::DualValue(dual_value_binding) => dual_value_binding.value(),
        }
    }
}

#[inline]
pub fn value_to_press(val: f32) -> bool {
    val != 0.
}

#[inline]
pub fn pressed_to_value(pressed: bool) -> f32 {
    if pressed { 1.0 } else { 0.0 }
}

#[derive(Component)]
pub struct InputBindings<K, T: BindEvent> {
    pub(crate) bindings: HashMap<K, InputBinding<T>>,
    pub(crate) assigned_gamepad: Option<Entity>,
    pub(crate) changed: bool,
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
    pub(crate) fn changed(&mut self) -> bool {
        if self.changed {
            self.changed = false;
            true
        } else {
            false
        }
    }
    pub fn register_binding(
        &mut self,
        name: K,
        bindings: InputBinding<T>,
    ) -> Option<InputBinding<T>> {
        self.changed = true;
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
            changed: false,
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
