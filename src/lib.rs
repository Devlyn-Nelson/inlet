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
//!   - [`DualValueBinding`] internally behaves as if it is just 2 [`ValueBinding`]'s.
//! - [`ButtonChord`] (multiple buttons at once) with configurable settings for
//!   resolving clashing inputs.
//! - [`ButtonCombo`] (multiple sequentially pressed buttons). Think GTA cheats codes.
//!
//! # Usage
//!
//! > see [`examples/events.rs`] to see most of what can be done.
//!
//! ## Binding Types to be aware of
//!
//! - [`BevyInputKind`] which is and enum that is either [`BevyAxisKind`] or [`BevyButtonKind`]. Both inner types just resolve down to types from `bevy_input`.
//! - [`BevyAxisButton`] this converts an axis to a button.
//! - [`ButtonBinding`] this what `inlet` uses as an actual binding to a button-like input. uses [`BevyButtonKind`] and [`BevyAxisButton`] to detect presses.
//!   - Can be configured to be a Chord (multiple buttons that must be pressed all at once).
//!   - Can be configured to be a Combo (multiple buttons pressed one after another).
//! - [`AxisBinding`] this what `inlet` uses as an actual binding to a axis-like input.
//!
//! ## Clash Settings
//!
//! The behavior of clash detection can be configured on a per player bias ([`ClashSettings`]) or globally ([`DefaultClashSettings`]).
//!
//! ### Clash Strategies
//!
//! - Unbuffered: Inputs that can clash will be rechecked after all inputs are checked at least once.
//! - BufferClashing: Inputs that can clash will not be reported for the initial frame they become active. Next frame or after provided [`Duration`] the action with the largest chord that is active will be given the inputs. Note that buttons that can NOT clash with other do NOT get buffered.
//! - BufferAll: This will cause ALL inputs to buffer. This exists to make things more consistent.
//! - Disabled: No clash prevention.
//!
//! ### Chord Regression
//!
//! This enables the ability for chords of smaller lengths to become active after a longer chord was already active.
//!
//! ### Component
//!
//! If the [`ClashSettings`] component is present on an entity that has a [`InputBindings`] the attached settings will be used.
//!
//! ### Resource
//!
//! If an entity does not have a [`ClashSettings`] component attached the [`DefaultClashSettings`] resource will be used, otherwise [`ClashSettings::default`] will be used.
//!
//! ## Combo Settings
//!
//! The behavior of combos can be configured on a per player bias ([`ComboSettings`]) or globally ([`DefaultComboSettings`]).
//! Maximum duration between combo button presses can be configured as well a Combo Interruption and Combo Progression
//!
//! ### Combo Interruption
//!
//! - [`NoBreak`] Don't interrupt on incorrect button press.
//! - [`ButtonsBreak`] Interrupt if an incorrect button was pressed.
//! - [`AnythingBreaks`] Interrupt if any input change happened.
//!
//! ### Combo Progression
//!
//! - [`None`] No Rules for progressing a combo.
//! - [`PreviousMustBeReleased`] The combo will not progress unless the previous expected button in not pressed.
//! - [`NextMustBeReleased`] The combo will not progress unless the next expected button in not pressed.
//!
//! ### Component
//!
//! If the [`ComboSettings`] component is present on an entity that has a [`InputBindings`] the attached settings will be used.
//!
//! ### Resource
//!
//! If an entity does not have a [`ComboSettings`] component attached the [`DefaultComboSettings`] resource will be used, otherwise [`ComboSettings::default`] will be used.
//!
//! # Examples
//!
//! ## `visualizer`
//!
//! There are 4 Columns, 1 for each input button (A, S, D, F).
//!
//! There are 3 Rows:
//!
//! - The top row is for chords (inputs that are held together). Each column represents a chord that requires that column's input and all inputs to the left to be pressed before the chord can be triggered.
//! - The middle row is for individual buttons.
//! - The bottom row is for combos. Like the top row, each column requires all inputs to the left, but instead of holding them all at the same time you must press them one after the other in order.
//!
//! See [chord/clash Settings](#clash-settings) and [Combo Settings](#combo-settings) for the explanations for the toggle settings.
//!
//! > Be aware that the time for buffered inputs is set really high (half a second) so if you toggle the clash settings to [`BufferAll`] no button, including the toggle buttons, will become active unless held for a the full half second.
//!
//! #### Controls
//!
//! | Action                           | Input |
//! | -------------------------------- | ----- |
//! | Trigger column 1                 | A     |
//! | Trigger column 2                 | S     |
//! | Trigger column 3                 | D     |
//! | Trigger column 4                 | F     |
//! | Toggle Clash Setting             | F1    |
//! | Toggle Chord Regression          | F2    |
//! | Toggle Combo Interupt Setting    | F3    |
//! | Toggle Combo Progression Setting | F4    |
//!
//! ## `simple`
//!
//! | Action      | Input                            |
//! | ----------- | -------------------------------- |
//! | Move        | WASD                             |
//! | Jump        | Space                            |
//! | Zoom Camera | Mouse Scroll Wheel               |
//! | Grow        | W->S->D->A (One after the other) |
//! | Shrink      | W+A+S+D (All at once)            |
//!
//! ## Code Explanations
//!
//! #### Simple (example)
//!
//! Create a list of input bindings to be used as a key to register bindings and retrieve values.
//!
//! This type MUST implement `Hash + PartialEq + Eq`
//!
//! ```
//! #[derive(Hash, PartialEq, Eq, Clone)]
//! enum InputTypes {
//!     Move,
//!     Zoom,
//!     Jump,
//!     SecretAbility1,
//!     SecretAbility2,
//! }
//! ```
//!
//! Create a Bindings component and add it to your entity.
//!
//! ```
//! InputBindings::<InputTypes>::new()
//!     // register a jump binding that triggers when either the space key or south on a gamepad is pressed.
//!     .with_action_binding(
//!         InputTypes::Jump,
//!         vec![KeyCode::Space.into(), GamepadButton::South.into()].into(),
//!     )
//!     // TODO added gamepad triggers as option for zoom.
//!     // register a zoom binding that reads values from the scroll wheel.
//!     .with_value_binding(
//!         InputTypes::Zoom,
//!         AxisBinding::mouse_y_scroll().invert().into(),
//!     )
//!     // register a move binding that gets the average non-zero value from the wasd on keyboard, the gamepads left stick and dpad.
//!     .with_dual_value_binding(
//!         InputTypes::Move,
//!         (
//!             vec![
//!                 AxisBinding::keyboard_da(),
//!                 AxisBinding::gamepad_left_stick_x(),
//!                 AxisBinding::gamepad_dpad_right_left(),
//!             ],
//!             vec![
//!                 AxisBinding::keyboard_ws(),
//!                 AxisBinding::gamepad_left_stick_y(),
//!                 AxisBinding::gamepad_dpad_up_down(),
//!             ],
//!         )
//!             .into(),
//!     )
//!     // register a cheat code binding activated by pressing forward ->
//!     .with_action_binding(
//!         InputTypes::SecretAbility1,
//!         (
//!             vec![
//!                 // W -> S -> D -> A
//!                 ButtonCombo::new(vec![
//!                     KeyCode::KeyW.into(),
//!                     KeyCode::KeyS.into(),
//!                     KeyCode::KeyD.into(),
//!                     KeyCode::KeyA.into(),
//!                 ])
//!                 .into(),
//!                 // Up -> Down -> Right -> Left on dpad
//!                 ButtonCombo::new(vec![
//!                     GamepadButton::DPadUp.into(),
//!                     GamepadButton::DPadDown.into(),
//!                     GamepadButton::DPadRight.into(),
//!                     GamepadButton::DPadLeft.into(),
//!                 ])
//!                 .into(),
//!                 ButtonChord::new(vec![
//!                     BevyAxisButton::new_positive_only(GamepadAxis::LeftStickY.into())
//!                         .into(),
//!                     BevyAxisButton::new_negative_only(GamepadAxis::LeftStickY.into())
//!                         .into(),
//!                     BevyAxisButton::new_positive_only(GamepadAxis::LeftStickX.into())
//!                         .into(),
//!                     BevyAxisButton::new_negative_only(GamepadAxis::LeftStickX.into())
//!                         .into(),
//!                 ])
//!                 .into(),
//!             ],
//!             ButtonEventBinding::WhenPressed,
//!         )
//!             .into(),
//!     )
//!     .with_action_binding(
//!         InputTypes::SecretAbility2,
//!         (
//!             vec![
//!                 ButtonChord::new(vec![
//!                     KeyCode::KeyW.into(),
//!                     KeyCode::KeyS.into(),
//!                     KeyCode::KeyA.into(),
//!                     KeyCode::KeyD.into(),
//!                 ])
//!                 .into(),
//!                 ButtonChord::new(vec![
//!                     GamepadButton::DPadUp.into(),
//!                     GamepadButton::DPadDown.into(),
//!                     GamepadButton::DPadLeft.into(),
//!                     GamepadButton::DPadRight.into(),
//!                 ])
//!                 .into(),
//!             ],
//!             ButtonEventBinding::WhenPressed,
//!         )
//!             .into(),
//!     )
//! ```
//!
//! Make a system or systems that use the values from bindings
//!
//! > you can also use polling in this system or other systems if you would like.
//!
//! ```
//! fn control_player(
//!     time: Res<Time>,
//!     mut player: Single<(&mut Transform, &InputBindings<InputTypes>)>,
//!     mut camera: Single<&mut Transform, (With<Camera3d>, Without<InputBindings<InputTypes>>)>,
//! ) {
//!     let delta_time = time.delta_secs();
//!     let mover = player.1.get_dual_value(&InputTypes::Move);
//!     let y_scale = player.0.scale.y * 0.5;
//!     let mover = Vec3::new(
//!         // we are inverting x to make the movement in the demo feel more intuitive.
//!         // mostly because we are directly applying the input values to the translation
//!         // instead of doing math to make it move the way you might expect.
//!         -mover.x,
//!         if player.1.get_action_state(&InputTypes::Jump).just_pressed() {
//!             10.0 * y_scale
//!         } else {
//!             0.0
//!         },
//!         mover.y,
//!     );
//!     player.0.translation += mover * delta_time;
//!     let zoom = 1.0 + (player.1.get_value(&InputTypes::Zoom) * delta_time);
//!     camera.translation *= zoom;
//! }
//!
//! fn accept_events(
//!     mut cheats: MessageReader<InletEvent<InputTypes>>,
//!     mut player: Single<&mut Transform, With<InputBindings<InputTypes>>>,
//! ) {
//!     for cheat in cheats.read() {
//!         match cheat.kind {
//!             InputTypes::SecretAbility1 => player.scale += 1.,
//!             InputTypes::SecretAbility2 => player.scale -= 1.,
//!             _ => {}
//!         }
//!     }
//! }
//! ```
//!
//! Add `InputManagementPlugin<InputTypes, MessageType>::default()` and your system to your bevy app.

pub mod axis;
pub mod button;
// pub mod clash;
pub mod manager;
mod plugins;
mod settings;
mod systems;

use std::{
    hash::Hash,
    ops::{Deref, DerefMut},
    time::Duration,
};

use button::ActionBinding;
pub use plugins::InputManagementPlugin;
pub use settings::*;

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
    /// Returns `true` if the value is a `Self::Pressed(_)`
    /// or `false` if `Self::Value(_)`.
    pub fn is_button(&self) -> bool {
        matches!(self, Self::Pressed(_))
    }
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

/// Generic binding for an input.
pub enum InputBinding {
    Action(ActionBinding),
    Value(ValueBinding),
    DualValue(DualValueBinding),
}

impl InputBinding {
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
    /// set the mock values for inner bindings regardless of if that is true or not.
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
    /// set the mock values for inner bindings regardless of if that is true or not.
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

/// Map actions `K` to an [`InputBinding`]. Also tracks the assigned
/// [`Gamepads`](bevy::prelude::Gamepad).
#[derive(Component)]
pub struct InputBindings<K> {
    pub(crate) bindings: HashMap<K, InputBinding>,
    pub(crate) assigned_gamepad: Option<Entity>,
    pub(crate) changed: bool,
}

impl<K> Default for InputBindings<K>
where
    K: Eq + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K> InputBindings<K>
where
    K: Eq + Hash,
{
    /// Returns a new blank instance.
    pub fn new() -> Self {
        Self {
            bindings: HashMap::default(),
            assigned_gamepad: None,
            changed: true,
        }
    }
    /// Returns `true` when binding detects changes to inner map. The input system should also set changed
    /// when a new [`ClashSettings`](crate::manager::ClashSettings) is applied.
    pub(crate) fn changed(&mut self) -> bool {
        std::mem::take(&mut self.changed)
    }

    pub fn register_binding(&mut self, name: K, bindings: InputBinding) -> Option<InputBinding> {
        self.changed = true;
        self.bindings.insert(name, bindings)
    }
    /// Map an action to a [`InputBinding::Action`].
    pub fn register_action_binding(
        &mut self,
        name: K,
        bindings: ActionBinding,
    ) -> Option<InputBinding> {
        self.register_binding(name, InputBinding::Action(bindings))
    }
    /// Map an action to a [`InputBinding::Value`].
    pub fn register_value_binding(
        &mut self,
        name: K,
        bindings: ValueBinding,
    ) -> Option<InputBinding> {
        self.register_binding(name, InputBinding::Value(bindings))
    }
    /// Map an action to a [`InputBinding::DualValue`].
    pub fn register_dual_value_binding(
        &mut self,
        name: K,
        bindings: DualValueBinding,
    ) -> Option<InputBinding> {
        self.register_binding(name, InputBinding::DualValue(bindings))
    }
    /// Builder style function for mapping an action to a [`InputBinding::Action`].
    pub fn with_action_binding(mut self, name: K, bindings: ActionBinding) -> Self {
        self.register_action_binding(name, bindings);
        self
    }
    /// Builder style function for mapping an action to a [`InputBinding::Value`].
    pub fn with_value_binding(mut self, name: K, bindings: ValueBinding) -> Self {
        self.register_value_binding(name, bindings);
        self
    }
    /// Builder style function for mapping an action to a [`InputBinding::DualValue`].
    pub fn with_dual_value_binding(mut self, name: K, bindings: DualValueBinding) -> Self {
        self.register_dual_value_binding(name, bindings);
        self
    }
    /// Returns mapped [`InputBinding`] for key `K`.
    pub fn get_binding(&self, name: &K) -> Option<&InputBinding> {
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

/// If a player index is not provided, an observer will automatically assign
/// the lowest available number.
///
/// If you remove the `PlayerIndex` component from an entity, the entity will no longer emit `Messages`.
#[derive(Debug, Component, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct PlayerIndex(pub usize);

impl Deref for PlayerIndex {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PlayerIndex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Message)]
pub struct InletEvent<T> {
    /// The `PlayerIndex` of the player who triggered the event.
    pub player: usize,
    /// The event kind.
    pub kind: T,
    /// The value of the input at the time of triggering the event.
    pub data: TriggerValue,
    /// How long the input was active before triggering the event.
    pub duration: Duration,
}

/// A value from any input.
#[derive(Debug, Clone)]
pub enum TriggerValue {
    /// Input was a button.
    Pressed(bool),
    /// Input was a axis.
    Value(f32),
    /// Input was a dual axis.
    DualValue(Vec2),
}
