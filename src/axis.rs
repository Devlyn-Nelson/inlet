use std::time::{Duration, Instant};

use bevy::{
    input::{
        gamepad::{GamepadAxis, GamepadButton},
        keyboard::KeyCode,
    },
    math::Vec2,
};

use crate::{
    button::{ActionableState, ButtonBinding, ButtonState},
    clash::ClashableKind,
};

/// Allows you to customize the behavior of an axis.
#[allow(unpredictable_function_pointer_comparisons)]
#[derive(Debug, Clone, PartialEq)]
pub enum AxisModifier {
    Simple(fn(f32) -> f32),
    /// the f32 stored will get passed into the second param of your function.
    Configurable(fn(f32, f32) -> f32, f32),
    /// the f32's stored will get passed into the second and third param of your function.
    DoubleConfigurable(fn(f32, f32, f32) -> f32, f32, f32),
}

impl AxisModifier {
    pub fn do_thing(&self, val: f32) -> f32 {
        match self {
            AxisModifier::Simple(f) => f(val),
            AxisModifier::Configurable(f, config) => f(val, *config),
            AxisModifier::DoubleConfigurable(f, config, two) => f(val, *config, *two),
        }
    }
    /// A modifier that inverts the sign of the input.
    pub const INVERT: Self = Self::Simple(axis_mod_invert);
    /// A modifier that returns the input if it is positive but 0 when negative.
    pub const POSITIVE_ONLY: Self = Self::Simple(axis_mod_positive_only);
    /// A modifier that returns the input if it is negative but 0 when positive.
    pub const NEGATIVE_ONLY: Self = Self::Simple(axis_mod_negative_only);
    /// Returns a Modifier that multiplies the input by `config`.
    pub fn sensitivity(config: f32) -> Self {
        Self::Configurable(axis_mod_sensitivity, config)
    }
    /// Returns a Modifier that returns 0. if the input is less than `config`.
    pub fn dead_zone(config: f32) -> Self {
        Self::Configurable(axis_mod_dead_zone, config)
    }
    /// Returns a Modifier that returns 0. if the value is not `one <= input <= two`.
    pub fn window(one: f32, two: f32) -> Self {
        Self::DoubleConfigurable(axis_mod_window, one, two)
    }
    /// Returns a Modifier that adds `config` to the input.
    pub fn add(config: f32) -> Self {
        Self::Configurable(axis_mod_add, config)
    }
}

pub fn axis_mod_invert(val: f32) -> f32 {
    -val
}

pub fn axis_mod_positive_only(value: f32) -> f32 {
    if value < 0. { 0. } else { value }
}

pub fn axis_mod_negative_only(value: f32) -> f32 {
    if value > 0. { 0. } else { value }
}

pub fn axis_mod_sensitivity(value: f32, config: f32) -> f32 {
    value * config
}

pub fn axis_mod_dead_zone(value: f32, config: f32) -> f32 {
    if value < config { 0. } else { value }
}

pub fn axis_mod_window(value: f32, one: f32, two: f32) -> f32 {
    if value >= one && value <= two {
        value
    } else {
        0.
    }
}

pub fn axis_mod_add(value: f32, config: f32) -> f32 {
    value + config
}

impl Eq for AxisModifier {}

#[derive(Debug, Clone, PartialEq)]
pub struct AxisBindingButton {
    pub binding: ButtonBinding,
    pub state: ButtonState,
}

impl From<ButtonBinding> for AxisBindingButton {
    fn from(value: ButtonBinding) -> Self {
        Self {
            binding: value,
            state: ButtonState::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MouseAxis {
    MotionX,
    MotionY,
    ScrollX,
    ScrollY,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AxisBindingKind {
    Mouse(MouseAxis),
    GamepadAxis(GamepadAxis),
    GamepadButton(GamepadButton),
    Buttons {
        plus: Option<AxisBindingButton>,
        minus: Option<AxisBindingButton>,
    },
    Mock(f32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AxisBinding {
    kind: AxisBindingKind,
    mod_stack: Vec<AxisModifier>,
}
impl AxisBinding {
    pub fn clashables(&self) -> Vec<ClashableKind> {
        let mut out = Vec::default();
        match &self.kind {
            AxisBindingKind::Mouse(mouse_axis) => out.push(ClashableKind::MouseAxis(*mouse_axis)),
            AxisBindingKind::GamepadAxis(gamepad_axis) => {
                out.push(ClashableKind::GamepadAxis(*gamepad_axis))
            }
            AxisBindingKind::GamepadButton(gamepad_button) => {
                out.push(ClashableKind::GamepadButton(*gamepad_button))
            }
            AxisBindingKind::Buttons { plus, minus } => {
                if let Some(p) = plus {
                    out.extend(p.binding.clashables());
                }
                if let Some(m) = minus {
                    out.extend(m.binding.clashables());
                }
            }
            AxisBindingKind::Mock(_) =>{}
        }
        out
    }

    pub fn mods(&self) -> &[AxisModifier] {
        &self.mod_stack
    }
    pub fn with_modifier(mut self, m: AxisModifier) -> Self {
        self.mod_stack.push(m);
        self
    }
    pub fn kind(&self) -> &AxisBindingKind {
        &self.kind
    }
    pub fn kind_mut(&mut self) -> &mut AxisBindingKind {
        &mut self.kind
    }
    pub fn keyboard_right_left() -> Self {
        AxisBindingKind::Buttons {
            plus: Some(ButtonBinding::Keyboard(KeyCode::ArrowRight).into()),
            minus: Some(ButtonBinding::Keyboard(KeyCode::ArrowLeft).into()),
        }
        .into()
    }
    pub fn keyboard_up_down() -> Self {
        AxisBindingKind::Buttons {
            plus: Some(ButtonBinding::Keyboard(KeyCode::ArrowUp).into()),
            minus: Some(ButtonBinding::Keyboard(KeyCode::ArrowDown).into()),
        }
        .into()
    }
    pub fn keyboard_da() -> Self {
        AxisBindingKind::Buttons {
            plus: Some(ButtonBinding::Keyboard(KeyCode::KeyD).into()),
            minus: Some(ButtonBinding::Keyboard(KeyCode::KeyA).into()),
        }
        .into()
    }
    pub fn keyboard_ws() -> Self {
        AxisBindingKind::Buttons {
            plus: Some(ButtonBinding::Keyboard(KeyCode::KeyW).into()),
            minus: Some(ButtonBinding::Keyboard(KeyCode::KeyS).into()),
        }
        .into()
    }
    pub fn mouse_x_motion() -> Self {
        AxisBindingKind::Mouse(MouseAxis::MotionX).into()
    }
    pub fn mouse_y_motion() -> Self {
        AxisBindingKind::Mouse(MouseAxis::MotionY).into()
    }
    pub fn mouse_x_scroll() -> Self {
        AxisBindingKind::Mouse(MouseAxis::ScrollX).into()
    }
    pub fn mouse_y_scroll() -> Self {
        AxisBindingKind::Mouse(MouseAxis::ScrollY).into()
    }
    pub fn gamepad_right_stick_x() -> Self {
        AxisBindingKind::GamepadAxis(GamepadAxis::RightStickX).into()
    }
    pub fn gamepad_right_stick_y() -> Self {
        AxisBindingKind::GamepadAxis(GamepadAxis::RightStickY).into()
    }
    pub fn gamepad_left_stick_x() -> Self {
        AxisBindingKind::GamepadAxis(GamepadAxis::LeftStickX).into()
    }
    pub fn gamepad_left_stick_y() -> Self {
        AxisBindingKind::GamepadAxis(GamepadAxis::LeftStickY).into()
    }
    pub fn gamepad_dpad_right_left() -> Self {
        AxisBindingKind::Buttons {
            plus: Some(ButtonBinding::Gamepad(GamepadButton::DPadRight).into()),
            minus: Some(ButtonBinding::Gamepad(GamepadButton::DPadLeft).into()),
        }
        .into()
    }
    pub fn gamepad_dpad_up_down() -> Self {
        AxisBindingKind::Buttons {
            plus: Some(ButtonBinding::Gamepad(GamepadButton::DPadUp).into()),
            minus: Some(ButtonBinding::Gamepad(GamepadButton::DPadDown).into()),
        }
        .into()
    }
    pub fn keyboard_plus_minus() -> Self {
        AxisBindingKind::Buttons {
            plus: Some(ButtonBinding::Keyboard(KeyCode::Equal).into()),
            minus: Some(ButtonBinding::Keyboard(KeyCode::Minus).into()),
        }
        .into()
    }
    pub fn buttons(plus: ButtonBinding, minus: ButtonBinding) -> Self {
        AxisBindingKind::Buttons {
            plus: Some(plus.into()),
            minus: Some(minus.into()),
        }
        .into()
    }
    pub fn buttons_optional(plus: Option<ButtonBinding>, minus: Option<ButtonBinding>) -> Self {
        AxisBindingKind::Buttons {
            plus: plus.map(|asdf| asdf.into()),
            minus: minus.map(|asdf| asdf.into()),
        }
        .into()
    }
    pub fn gamepad_axis(axis: GamepadAxis) -> Self {
        AxisBindingKind::GamepadAxis(axis).into()
    }
    pub fn gamepad_button(axis: GamepadButton) -> Self {
        AxisBindingKind::GamepadButton(axis).into()
    }
    pub fn mouse(axis: MouseAxis) -> Self {
        AxisBindingKind::Mouse(axis).into()
    }
    pub fn invert(self) -> Self {
        self.with_modifier(AxisModifier::INVERT.clone())
    }
}

impl From<GamepadAxis> for AxisBinding {
    fn from(value: GamepadAxis) -> Self {
        Self::gamepad_axis(value)
    }
}

impl From<GamepadButton> for AxisBinding {
    fn from(value: GamepadButton) -> Self {
        Self::gamepad_button(value)
    }
}

impl From<MouseAxis> for AxisBinding {
    fn from(value: MouseAxis) -> Self {
        Self::mouse(value)
    }
}

impl From<AxisBindingKind> for AxisBinding {
    fn from(value: AxisBindingKind) -> Self {
        Self {
            kind: value,
            mod_stack: vec![],
        }
    }
}

impl From<(AxisBindingButton, AxisBindingButton)> for AxisBinding {
    fn from(value: (AxisBindingButton, AxisBindingButton)) -> Self {
        AxisBindingKind::Buttons {
            plus: Some(value.0),
            minus: Some(value.1),
        }
        .into()
    }
}

impl From<(ButtonBinding, ButtonBinding)> for AxisBinding {
    fn from(value: (ButtonBinding, ButtonBinding)) -> Self {
        AxisBindingKind::Buttons {
            plus: Some(value.0.into()),
            minus: Some(value.1.into()),
        }
        .into()
    }
}

pub struct ValueState {
    pub(crate) previous: f32,
    /// The last value feed into this binding.
    pub(crate) current: f32,
    /// Last instant that the value transitioned from zero to a non-zero value or a non-zero value to zero.
    pub(crate) last_transition: Instant,
}

impl ValueState {
    #[inline]
    pub fn action_state(&self) -> ButtonState {
        ButtonState {
            kind: if self.pressed() {
                if self.previous == 0. {
                    ActionableState::JustPressed
                } else {
                    ActionableState::Pressed
                }
            } else {
                if self.previous == 0. {
                    ActionableState::Released
                } else {
                    ActionableState::JustReleased
                }
            },
            start: self.last_transition,
        }
    }
    #[inline]
    pub fn current(&self) -> f32 {
        self.current
    }
    #[inline]
    pub fn previous(&self) -> f32 {
        self.previous
    }
    /// The amount of time passed between now and the last time the internal state transitioned from:
    /// - 0 to a non-zero value.
    /// - A non-zero value to 0.
    #[inline]
    pub fn last_transition(&self) -> Duration {
        self.last_transition.elapsed()
    }
    /// Returns `true` if [`Self::current`] would return a zero non-zero value and [`Self::current`] would return zero.
    #[inline]
    pub fn just_pressed(&self) -> bool {
        self.previous == 0. && self.current != 0.
    }
    /// Returns `true` if [`Self::current`] would return a zero non-zero value.
    #[inline]
    pub fn pressed(&self) -> bool {
        self.current != 0.
    }
    /// Returns `true` if the internal state has been a non-zero value for `duration`, otherwise `false`.
    pub fn held_for(&self, duration: &Duration) -> bool {
        self.pressed() && self.last_transition.elapsed() >= *duration
    }
    /// Returns `true` if the internal state has been a non-zero value for at least `start` but less than `stop`.
    pub fn held_range(&self, start: &Duration, stop: &Duration) -> bool {
        let elapsed = self.last_transition.elapsed();
        self.pressed() && elapsed >= *start && elapsed < *stop
    }
    /// Returns time elapsed for the internal state being a non-zero value state or `None`.
    pub fn try_get_held_duration(&self) -> Option<Duration> {
        if self.pressed() {
            Some(self.last_transition.elapsed())
        } else {
            None
        }
    }
    /// Returns `true` if [`Self::current`] would return zero and [`Self::current`] would return a non-zero value.
    pub fn just_released(&self) -> bool {
        self.previous != 0. && self.current == 0.
    }
    /// Returns `true` if [`Self::current`] would return zero.
    pub fn released(&self) -> bool {
        self.current == 0.
    }
    /// `value` will feed the internal current state and update necessary values.
    pub fn feed(&mut self, value: f32) {
        if (self.current == 0. && value != 0.) || (self.current != 0. && value == 0.) {
            self.last_transition = Instant::now();
        }
        self.previous = self.current;
        self.current = value;
    }
}

impl Default for ValueState {
    fn default() -> Self {
        Self {
            previous: 0.,
            current: 0.,
            last_transition: Instant::now(),
        }
    }
}

pub struct ValueBinding<T> {
    pub(crate) bindings: Vec<AxisBinding>,
    pub(crate) mod_stack: Vec<AxisModifier>,
    pub(crate) event: fn(f32) -> Option<T>,
    pub(crate) state: ValueState,
}

impl<T> ValueBinding<T> {
    pub fn clashables(&self) -> Vec<ClashableKind> {
        let mut out = Vec::default();
        for b in &self.bindings {
            out.extend(b.clashables());
        }
        out
    }
    pub fn state(&self) -> &ValueState {
        &self.state
    }
    pub fn last_transition(&self) -> Instant {
        self.state.last_transition
    }
    pub fn value(&self) -> f32 {
        self.state.current
    }
    pub fn bindings(&self) -> &[AxisBinding] {
        &self.bindings
    }
    pub fn bindings_mut(&mut self) -> &mut [AxisBinding] {
        &mut self.bindings
    }
    pub fn feed(&mut self, value: f32) -> Option<T> {
        self.state.feed(value);
        (self.event)(self.value())
    }
    pub fn from_parts(
        bindings: Vec<AxisBinding>,
        mod_stack: Vec<AxisModifier>,
        event: fn(f32) -> Option<T>,
    ) -> Self {
        Self {
            bindings,
            mod_stack,
            event,
            state: ValueState::default(),
        }
    }
    pub fn from_bindings(bindings: Vec<AxisBinding>) -> Self {
        Self {
            bindings,
            mod_stack: vec![],
            event: no_event,
            state: ValueState::default(),
        }
    }
    pub fn from_binding(binding: AxisBinding) -> Self {
        Self {
            bindings: vec![binding],
            mod_stack: vec![],
            event: no_event,
            state: ValueState::default(),
        }
    }
    pub fn with_event(mut self, event: fn(f32) -> Option<T>) -> Self {
        self.event = event;
        self
    }
    pub fn with_modifier(mut self, modifier: AxisModifier) -> Self {
        self.mod_stack.push(modifier);
        self
    }
}

impl<T> From<GamepadAxis> for ValueBinding<T> {
    fn from(value: GamepadAxis) -> Self {
        Self::from_binding(value.into())
    }
}

impl<T> From<GamepadButton> for ValueBinding<T> {
    fn from(value: GamepadButton) -> Self {
        Self::from_binding(value.into())
    }
}

impl<T> From<MouseAxis> for ValueBinding<T> {
    fn from(value: MouseAxis) -> Self {
        Self::from_binding(value.into())
    }
}

impl<T> From<AxisBinding> for ValueBinding<T> {
    fn from(value: AxisBinding) -> Self {
        Self::from_binding(value)
    }
}

impl<T> From<Vec<AxisBinding>> for ValueBinding<T> {
    fn from(value: Vec<AxisBinding>) -> Self {
        Self::from_bindings(value)
    }
}

fn no_event<T>(_: f32) -> Option<T> {
    None
}

pub struct DualValueBinding<T> {
    pub(crate) x_bindings: Vec<AxisBinding>,
    pub(crate) x_mod_stack: Vec<AxisModifier>,
    pub(crate) y_bindings: Vec<AxisBinding>,
    pub(crate) y_mod_stack: Vec<AxisModifier>,
    pub(crate) event: fn(Vec2) -> Option<T>,
    pub(crate) x_state: ValueState,
    pub(crate) y_state: ValueState,
}
impl<T> DualValueBinding<T> {
    pub fn clashables(&self) -> Vec<ClashableKind> {
        let mut out = Vec::default();
        for b in &self.x_bindings {
            out.extend(b.clashables());
        }
        for b in &self.y_bindings {
            out.extend(b.clashables());
        }
        out
    }
    pub fn x_state(&self) -> &ValueState {
        &self.x_state
    }
    pub fn y_state(&self) -> &ValueState {
        &self.y_state
    }
    pub fn last_transition(&self) -> Instant {
        let x = self.x_state.last_transition;
        let y = self.y_state.last_transition;
        if x < y { y } else { x }
    }
    pub fn x_bindings(&self) -> &[AxisBinding] {
        &self.x_bindings
    }
    pub fn y_bindings(&self) -> &[AxisBinding] {
        &self.y_bindings
    }
    pub fn x_bindings_mut(&mut self) -> &mut [AxisBinding] {
        &mut self.x_bindings
    }
    pub fn y_bindings_mut(&mut self) -> &mut [AxisBinding] {
        &mut self.y_bindings
    }
    pub fn feed(&mut self, value: Vec2) -> Option<T> {
        self.x_state.feed(value.x);
        self.y_state.feed(value.y);
        (self.event)(self.value())
    }
    pub fn value(&self) -> Vec2 {
        Vec2::new(self.x_state.current, self.y_state.current)
    }
    pub fn with_x_modifier(mut self, modifier: AxisModifier) -> Self {
        self.x_mod_stack.push(modifier);
        self
    }
    pub fn with_y_modifier(mut self, modifier: AxisModifier) -> Self {
        self.y_mod_stack.push(modifier);
        self
    }
    pub fn with_event(mut self, event: fn(Vec2) -> Option<T>) -> Self {
        self.event = event;
        self
    }
    pub fn from_binding(x: AxisBinding, y: AxisBinding) -> Self {
        Self {
            x_bindings: vec![x],
            y_bindings: vec![y],
            x_mod_stack: vec![],
            y_mod_stack: vec![],
            event: no_event_dual,
            x_state: ValueState::default(),
            y_state: ValueState::default(),
        }
    }
    pub fn from_bindings(x: Vec<AxisBinding>, y: Vec<AxisBinding>) -> Self {
        Self {
            x_bindings: x,
            y_bindings: y,
            x_mod_stack: vec![],
            y_mod_stack: vec![],
            event: no_event_dual,
            x_state: ValueState::default(),
            y_state: ValueState::default(),
        }
    }
}

impl<T> From<(AxisBinding, AxisBinding)> for DualValueBinding<T> {
    fn from((x, y): (AxisBinding, AxisBinding)) -> Self {
        Self::from_binding(x, y)
    }
}

impl<T> From<(Vec<AxisBinding>, Vec<AxisBinding>)> for DualValueBinding<T> {
    fn from((x, y): (Vec<AxisBinding>, Vec<AxisBinding>)) -> Self {
        Self::from_bindings(x, y)
    }
}

fn no_event_dual<T>(_: Vec2) -> Option<T> {
    None
}

// #[derive(Clone)]
// pub struct DualAxisBindings<T> {
//     bindings_x: Vec<AxisBinding>,
//     bindings_y: Vec<AxisBinding>,
//     event: fn(f32, f32) -> T,
// }
// #[derive(Clone)]
// pub struct TriAxisBindings<T> {
//     bindings_x: Vec<AxisBinding>,
//     bindings_y: Vec<AxisBinding>,
//     bindings_z: Vec<AxisBinding>,
//     event: fn(f32, f32, f32) -> T,
// }

// #[derive(Clone)]
// pub struct AxisBindings<T> {
//     pub bindings: Vec<ButtonBinding>,
//     pub event: ButtonEventBinding<T>,
//     pub state: f32,
// }
