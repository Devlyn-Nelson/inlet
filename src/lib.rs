//! Input to Action Binding library for Bevy Game Engine.
//!
//! # Features
//!
//! - Maps Actions to input bindings
//! - Uses `bevy_input` internally, supports Keyboard, Gamepad, and Mouse.
//! - Can produce [`Message`] for common input events.
//! - [`InputBinding`] lets you bind any axis or button to any axis or button like input.
//!   - [`ActionBinding`] has internal states to best represent button like behavior: JustPressed, Pressed,
//!     JustReleased, Released. Can also be used as digital (-1, 0, 1) axis.
//!   - [`ValueBinding`] can return a value (-1.0 to 1.0) from any axis or set of buttons. Can have a stack
//!     of generic functions that modify the output. Can be used as a button, by default it is assumed any non-zero
//!     value is pressed, but modifiers can enable you to control this behavior more finely.
//!   - [`DualValueBinding`] internally behaves as if it is just 2 `ValueBinding`'s.
//! - [`ButtonChord`](crate::button::ButtonChord) (multiple buttons at once) with configurable settings for
//!   resolving clashing inputs.
//! - [`ButtonCombo`](crate::button::ButtonCombo) (multiple sequentially pressed buttons). Think GTA cheats but
//!   without incorrect buttons interrupting it.
//!
//! # Usage
//!
//! ## Binding Types to be aware of
//!
//! - [`BevyInputKind`] which is and enum that is either [`BevyAxisKind`] or [`BevyButtonKind`]. Both inner
//!   types just resolve down to types from
//!   `bevy_input`.
//! - [`BevyAxisButton`](crate::button::BevyAxisButton) this converts an axis to a button.
//! - [`ButtonBinding`](crate::button::ButtonBinding) this what `inlet` uses as an actual binding to a button-like
//!   input. uses [`BevyButtonKind`] or [`BevyAxisButton`](crate::button::BevyAxisButton) to detect presses.
//!   - Can be configured to be a [`ButtonChord`](crate::button::ButtonChord) (multiple buttons that must be pressed all at once).
//!   - Can be configured to be a [`ButtonCombo`](crate::button::ButtonCombo) (multiple buttons pressed one after another).
//! - [`AxisBinding`](crate::axis::AxisBinding) this what `inlet` uses as an actual binding to a axis-like input.
//!
//!
//! ## Poll Only
//!
//! see `examples/poll-only.rs` for code. below are the sets required for setups without using [`Message`].
//!
//! - Create a list of input bindings to be used as a key to register bindings and retrieve values. This type
//!   MUST implement `Hash + PartialEq + Clone + Eq`.
//!
//! - Create a Bindings component and add it to your entity.
//!
//! > [`InputBindingsSimple`] is just a type definition that fills in the message type with a placeholder for when
//! > you don't want to deal with both generic types required for [`InputBindings`].
//!
//! - Make a system that uses the values from bindings
//!
//! - Add [`InputManagementPluginSimple<InputTypes>::default()`] and your system to your bevy app.
//!
//! > [`InputManagementPluginSimple`] is just a type definition that fills in the message type with a placeholder
//! > type for when you don't want to deal with both generic types required for [`InputManagementPlugin`].
//!
//! ## Message Based
//!
//! see `examples/events.rs` for code. below are the sets required for setups using [`Message`] triggered by
//! inputs.
//!
//! - Create a list of input bindings to be used as a key to register bindings and retrieve values. This type
//!   MUST implement `Hash + PartialEq + Eq + Clone`
//!
//! - Also create a type that implements [`Message`]
//!
//! > You can make only 1 type that gets used for both if you want. This example separates them
//! > simply to show they can be separate types for cases where you are mixing message-based and
//! > polling-based bindings.
//!
//! - Create a Bindings component and add it to your entity.
//!
//! - Make a system that uses the values from bindings
//!
//! > you can also use polling in this system or other systems if you would like.
//!
//! - Add [`InputManagementPlugin<InputTypes, MessageType>::default()`] and your system to your bevy app.
//!
//! ## [`ClashSettings`](crate::manager::ClashSettings)
//!
//! If you like [`ButtonChords`](crate::button::ButtonChord) and have opinions about how inputs that clash should
//! behave: you can configure how that happens.
//!
//! ### [`Resource`](bevy::prelude::Resource)
//!
//! You can spawn  a [`ClashSettings`](crate::manager::ClashSettings) resource (preferably on start up) that
//! all new [`InputHandler`](crate::manager::InputHandler) will use. The system that updates bindings
//! will automatically insert [`InputHandler`](crate::manager::InputHandler) on entities that have an
//! [`InputBindings`] attached to them, acting as a default.
//!
//! ### [`Component`]
//!
//! When you insert [`ClashSettings`](crate::manager::ClashSettings) as a component on an entity that also
//! has an attached [`InputBindings`] the settings will update and all current input states will reset. This
//! means you can allow player to configure this on a per-player basis.
//!
//!
pub mod axis;
pub mod button;
// pub mod clash;
pub mod manager;
mod plugins;
mod systems;

use std::hash::Hash;

use button::ActionBinding;
pub use plugins::{InputManagementPlugin, InputManagementPluginSimple};

use bevy::{
    input::{
        gamepad::{GamepadAxis, GamepadButton},
        keyboard::KeyCode,
        mouse::MouseButton,
    },
    math::Vec2,
    platform::collections::HashMap,
    prelude::{Component, Entity, Message},
};

use crate::{
    axis::{DualValueBinding, MouseAxis, ValueBinding},
    button::ButtonState,
};

/// `inlet` trait for describing [`Message`] events, currently only requires [`Message`] to be auto-implemented.
/// This type only exists to make it easier for me if a add requirements later.
pub trait BindEvent: Message {}

impl<T> BindEvent for T where T: Message {}

/// A value from any input.
#[derive(Debug, Clone)]
pub enum InputValue {
    /// Input was a button.
    Pressed(bool),
    /// Input was a axis.
    Value(f32),
}

impl From<f32> for InputValue {
    fn from(value: f32) -> Self {
        Self::Value(value)
    }
}
impl From<bool> for InputValue {
    fn from(value: bool) -> Self {
        Self::Pressed(value)
    }
}
impl Default for InputValue {
    fn default() -> Self {
        Self::Pressed(false)
    }
}

impl InputValue {
    /// Returns true if `self` is:
    /// - `Self::Button(true)`.
    /// - `Self::Value(val)` where `val != 0`.
    pub fn is_pressed(&self) -> bool {
        match self {
            InputValue::Pressed(p) => *p,
            InputValue::Value(val) => value_to_press(*val),
        }
    }
    /// Returns:
    /// - `1.0` if `Self::Button(true)`, `0.0` if `Self::Button(false)`.
    /// - `val` when `Self::Value(val)`.
    pub fn get_value(&self) -> f32 {
        match self {
            InputValue::Pressed(p) => pressed_to_value(*p),
            InputValue::Value(val) => *val,
        }
    }
}

/// A enum of all supported `bevy_input` types that can be used as axis-like bindings.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum BevyAxisKind {
    MouseAxis(MouseAxis),
    GamepadAxis(GamepadAxis),
    GamepadButton(GamepadButton),
}

impl From<MouseAxis> for BevyAxisKind {
    fn from(value: MouseAxis) -> Self {
        Self::MouseAxis(value)
    }
}

impl From<GamepadAxis> for BevyAxisKind {
    fn from(value: GamepadAxis) -> Self {
        Self::GamepadAxis(value)
    }
}

impl From<GamepadButton> for BevyAxisKind {
    fn from(value: GamepadButton) -> Self {
        Self::GamepadButton(value)
    }
}

/// A enum of all supported `bevy_input` types that can be used as button-like bindings.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum BevyButtonKind {
    GamepadButton(GamepadButton),
    KeyCode(KeyCode),
    MouseButton(MouseButton),
}

impl From<GamepadButton> for BevyButtonKind {
    fn from(value: GamepadButton) -> Self {
        Self::GamepadButton(value)
    }
}

impl From<KeyCode> for BevyButtonKind {
    fn from(value: KeyCode) -> Self {
        Self::KeyCode(value)
    }
}

impl From<MouseButton> for BevyButtonKind {
    fn from(value: MouseButton) -> Self {
        Self::MouseButton(value)
    }
}

/// A enum of all supported `bevy_input` types that can be used as bindings.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum BevyInputKind {
    /// An button-kind from `bevy_input`.
    Button(BevyButtonKind),
    /// An axis-kind from `bevy_input`.
    Axis(BevyAxisKind),
}

impl From<BevyButtonKind> for BevyInputKind {
    fn from(value: BevyButtonKind) -> Self {
        Self::Button(value)
    }
}

impl From<BevyAxisKind> for BevyInputKind {
    fn from(value: BevyAxisKind) -> Self {
        Self::Axis(value)
    }
}

impl From<MouseAxis> for BevyInputKind {
    fn from(value: MouseAxis) -> Self {
        let new: BevyAxisKind = value.into();
        new.into()
    }
}

impl From<GamepadAxis> for BevyInputKind {
    fn from(value: GamepadAxis) -> Self {
        let new: BevyAxisKind = value.into();
        new.into()
    }
}

impl From<GamepadButton> for BevyInputKind {
    fn from(value: GamepadButton) -> Self {
        let new: BevyButtonKind = value.into();
        new.into()
    }
}

impl From<KeyCode> for BevyInputKind {
    fn from(value: KeyCode) -> Self {
        let new: BevyButtonKind = value.into();
        new.into()
    }
}

impl From<MouseButton> for BevyInputKind {
    fn from(value: MouseButton) -> Self {
        let new: BevyButtonKind = value.into();
        new.into()
    }
}

/// Simple [`Message`] type used by [`InputManagementPluginSimple`].
#[derive(Debug, Message)]
pub struct SimpleMessage;

/// Generic binding for an input.
pub enum InputBinding<T> {
    Action(ActionBinding<T>),
    Value(ValueBinding<T>),
    DualValue(DualValueBinding<T>),
}

impl<T> InputBinding<T> {
    /// Sets the binding to have a pressed value by default.
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
    /// Sets the binding to use `value` as the default value in axis polling.
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
    /// Sets the binding to use `value` as the default value in axis polling on the X axis.
    ///
    /// # Warning
    ///
    /// This is intended for cases where you know that the binding is a [`Self::DualValue`], but this will still
    /// set the mock values for inner bindings regardless of if thats true or not.
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
    /// Sets the binding to use `value` as the default value in axis polling on the Y axis.
    ///
    /// # Warning
    ///
    /// This is intended for cases where you know that the binding is a [`Self::DualValue`], but this will still
    /// set the mock values for inner bindings regardless of if thats true or not.
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
    /// Clears mock inputs from the binding.
    pub fn mock_clear(&mut self) {
        match self {
            InputBinding::Action(action_binding) => action_binding.mock_clear(),
            InputBinding::Value(value_binding) => value_binding.mock_clear(),
            InputBinding::DualValue(dual_value_binding) => {
                dual_value_binding.mock_clear();
            }
        }
    }
    /// Returns all possible [`BevyInputKind`] that are associated with this input.
    pub fn input_kinds(&self) -> Vec<BevyInputKind> {
        match self {
            InputBinding::Action(action_binding) => action_binding.input_kinds(),
            InputBinding::Value(value_binding) => value_binding.input_kinds(),
            InputBinding::DualValue(dual_value_binding) => dual_value_binding.input_kinds(),
        }
    }
    /// Returns a [`ButtonState`] for the binging. If the binding is not a [`Self::Action`] we create a simulated
    /// one where non-zero values on the axis are `true` for the press state.
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

/// Default logic for converting a axis value to a button press.
///
/// non-zero value are `true`, zero returns `false`.
#[inline]
pub fn value_to_press(val: f32) -> bool {
    val != 0.
}

/// Default logic for converting a button press to axis value.
///
/// when `pressed` is `true` `1.0` will be returned, otherwise `0.0` is returned.
#[inline]
pub fn pressed_to_value(pressed: bool) -> f32 {
    if pressed { 1.0 } else { 0.0 }
}

/// Map actions `K` to an [`InputBinding<T>`] without a custom [`Message`] type. Also tracks the assigned
/// [`Gamepads`](bevy::prelude::Gamepad).
pub type InputBindingsSimple<K> = InputBindings<K, SimpleMessage>;

/// Map actions `K` to an [`InputBinding<T>`] where `T` is a [`Message`]. Also tracks the assigned
/// [`Gamepads`](bevy::prelude::Gamepad).
#[derive(Component)]
pub struct InputBindings<K, T: BindEvent> {
    pub(crate) bindings: HashMap<K, InputBinding<T>>,
    pub(crate) assigned_gamepad: Option<Entity>,
    pub(crate) changed: bool,
}

impl<K, T> Default for InputBindings<K, T>
where
    K: Eq + Hash,
    T: BindEvent,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, T> InputBindings<K, T>
where
    K: Eq + Hash,
    T: BindEvent,
{
    /// Builder style function for mapping an action to a [`InputBinding::Action`].
    pub fn with_action_binding(mut self, name: K, bindings: ActionBinding<T>) -> Self {
        self.register_action_binding(name, bindings);
        self
    }
    /// Builder style function for mapping an action to a [`InputBinding::Value`].
    pub fn with_value_binding(mut self, name: K, bindings: ValueBinding<T>) -> Self {
        self.register_value_binding(name, bindings);
        self
    }
    /// Builder style function for mapping an action to a [`InputBinding::DualValue`].
    pub fn with_dual_value_binding(mut self, name: K, bindings: DualValueBinding<T>) -> Self {
        self.register_dual_value_binding(name, bindings);
        self
    }
    pub(crate) fn change(&mut self) {
        self.changed = true;
    }
    /// Returns `true` when binding detects changes to inner map. The input system should also set changed
    /// when a new [`ClashSettings`](crate::manager::ClashSettings) is applied.
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
    /// Map an action to a [`InputBinding::Action`].
    pub fn register_action_binding(
        &mut self,
        name: K,
        bindings: ActionBinding<T>,
    ) -> Option<InputBinding<T>> {
        self.register_binding(name, InputBinding::Action(bindings))
    }
    /// Map an action to a [`InputBinding::Value`].
    pub fn register_value_binding(
        &mut self,
        name: K,
        bindings: ValueBinding<T>,
    ) -> Option<InputBinding<T>> {
        self.register_binding(name, InputBinding::Value(bindings))
    }
    /// Map an action to a [`InputBinding::DualValue`].
    pub fn register_dual_value_binding(
        &mut self,
        name: K,
        bindings: DualValueBinding<T>,
    ) -> Option<InputBinding<T>> {
        self.register_binding(name, InputBinding::DualValue(bindings))
    }
    /// Returns a new blank instance.
    pub fn new() -> Self {
        Self {
            bindings: HashMap::default(),
            assigned_gamepad: None,
            changed: false,
        }
    }
    /// Returns mapped [`InputBinding`] for key `K`.
    pub fn get_binding(&self, name: &K) -> Option<&InputBinding<T>> {
        self.bindings.get(name)
    }
    /// Returns a [`ButtonState`] that describes the state if the [`InputBinding`] mapped to key `K`.
    pub fn get_action_state(&self, name: &K) -> ButtonState {
        self.get_binding(name)
            .map(|binding| binding.state())
            .unwrap_or_default()
    }
    /// Returns `true` if the state of the [`InputBinding`] mapped to key `K` could be considered
    /// [`ActionableState::JustPressed`](crate::button::ActionableState::JustPressed).
    pub fn just_pressed(&self, name: &K) -> bool {
        self.get_binding(name)
            .map(|binding| binding.state().just_pressed())
            .unwrap_or_default()
    }
    /// Returns `true` if the state of the [`InputBinding`] mapped to key `K` could be considered
    /// [`ActionableState::Pressed`](crate::button::ActionableState::Pressed) or
    /// [`ActionableState::JustPressed`](crate::button::ActionableState::JustPressed).
    pub fn pressed(&self, name: &K) -> bool {
        self.get_binding(name)
            .map(|binding| binding.state().pressed())
            .unwrap_or_default()
    }
    /// Returns `true` if the state of the [`InputBinding`] mapped to key `K` could be considered
    /// [`ActionableState::JustReleased`](crate::button::ActionableState::JustReleased).
    pub fn just_released(&self, name: &K) -> bool {
        self.get_binding(name)
            .map(|binding| binding.state().just_released())
            .unwrap_or_default()
    }
    /// Returns `true` if the state of the [`InputBinding`] mapped to key `K` could be considered
    /// [`ActionableState::Released`](crate::button::ActionableState::Released) or
    /// [`ActionableState::JustReleased`](crate::button::ActionableState::JustReleased).
    pub fn released(&self, name: &K) -> bool {
        self.get_binding(name)
            .map(|binding| binding.state().released())
            .unwrap_or_default()
    }
    /// Returns result from [`InputBinding::value()`] if the key `K` has a mapping binding, otherwise `0.0` is
    /// returned.
    pub fn get_value(&self, name: &K) -> f32 {
        self.get_binding(name)
            .map(|binding| binding.value())
            .unwrap_or_default()
    }
    /// Returns result from [`InputBinding::dual_value()`] if the key `K` has a mapping binding, otherwise
    /// [`Vec2::default()`] is returned.
    pub fn get_dual_value(&self, name: &K) -> Vec2 {
        self.get_binding(name)
            .map(|binding| binding.dual_value())
            .unwrap_or_default()
    }
}
