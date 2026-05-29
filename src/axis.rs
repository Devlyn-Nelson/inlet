use std::time::Instant;

use bevy::{
    input::{
        gamepad::{GamepadAxis, GamepadButton},
        keyboard::KeyCode,
    },
    math::Vec2,
};

use crate::button::{ButtonBinding, ButtonState};

#[allow(unpredictable_function_pointer_comparisons)]
#[derive(Debug, Clone, PartialEq)]
pub enum Modifier {
    Simple(fn(f32) -> f32),
    /// I assume the second f32 parameter is the config and the first is the val that is getting modified.
    Configurable(fn(f32, f32) -> f32, f32),
}

impl Modifier {
    pub fn do_thing(&self, val: f32) -> f32 {
        match self {
            Modifier::Simple(f) => f(val),
            Modifier::Configurable(f, config) => f(val, *config),
        }
    }
    pub const INVERT: Self = Self::Simple(axis_mod_invert);
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

pub fn axis_mod_add(value: f32, config: f32) -> f32 {
    value + config
}

impl Eq for Modifier {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AxisBindingButton {
    pub binding: ButtonBinding,
    pub state: ButtonState,
}

impl PartialOrd for AxisBindingButton {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.binding.partial_cmp(&other.binding)
    }
}

impl Ord for AxisBindingButton {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.binding.cmp(&other.binding)
    }
}

impl From<ButtonBinding> for AxisBindingButton {
    fn from(value: ButtonBinding) -> Self {
        Self {
            binding: value,
            state: ButtonState::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MouseAxis {
    MotionX,
    MotionY,
    ScrollX,
    ScrollY,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AxisBindingKind {
    Mouse(MouseAxis),
    GamepadAxis(GamepadAxis),
    GamepadButton(GamepadButton),
    Buttons {
        plus: Option<AxisBindingButton>,
        minus: Option<AxisBindingButton>,
    },
}

// impl AxisBindingKind {
//     fn index(&self) -> u8 {
//         match self {
//             AxisBindingKind::Mouse(mouse_axis) => 0,
//             AxisBindingKind::GamepadAxis(gamepad_axis) => 1,
//             AxisBindingKind::GamepadButton(gamepad_button) => todo!(),
//             AxisBindingKind::Buttons { plus, minus } => todo!(),
//         }
//     }
// }

fn stick_index(axis: &GamepadAxis) -> u8 {
    match axis {
        GamepadAxis::LeftStickX => 0,
        GamepadAxis::LeftStickY => 1,
        GamepadAxis::LeftZ => 2,
        GamepadAxis::RightStickX => 3,
        GamepadAxis::RightStickY => 4,
        GamepadAxis::RightZ => 5,
        GamepadAxis::Other(_) => 6,
    }
}
impl PartialOrd for AxisBindingKind {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for AxisBindingKind {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self {
            AxisBindingKind::Mouse(mouse_axis) => match other {
                AxisBindingKind::Mouse(other_mouse_axis) => mouse_axis.cmp(other_mouse_axis),
                AxisBindingKind::GamepadAxis(_) |
                AxisBindingKind::GamepadButton(_) |
                AxisBindingKind::Buttons { .. } => std::cmp::Ordering::Less,
            },
            AxisBindingKind::GamepadAxis(gamepad_axis) => match other {
                AxisBindingKind::Mouse(_) => std::cmp::Ordering::Greater,
                AxisBindingKind::GamepadAxis(other_gamepad_axis) => match gamepad_axis {
                    GamepadAxis::Other(custom_axis) => if let GamepadAxis::Other(other_custom_axis) = other_gamepad_axis {
                        custom_axis.cmp(other_custom_axis)
                    }else{
                        std::cmp::Ordering::Greater
                    },
                    _ => {
                        stick_index(gamepad_axis).cmp(&stick_index(other_gamepad_axis))
                    }
                },
                AxisBindingKind::GamepadButton(_) |
                AxisBindingKind::Buttons { .. } => std::cmp::Ordering::Less,
            },
            AxisBindingKind::GamepadButton(gamepad_button) => match other {
                AxisBindingKind::Mouse(_) |
                AxisBindingKind::GamepadAxis(_) => std::cmp::Ordering::Greater,
                AxisBindingKind::GamepadButton(other_gamepad_button) => gamepad_button.cmp(other_gamepad_button),
                AxisBindingKind::Buttons { .. } => std::cmp::Ordering::Less,
            },
            AxisBindingKind::Buttons { plus, minus } => match other {
                AxisBindingKind::Mouse(_) |
                AxisBindingKind::GamepadAxis(_) |
                AxisBindingKind::GamepadButton(_) => std::cmp::Ordering::Greater,
                AxisBindingKind::Buttons { plus: other_plus, minus: other_minus } => {
                    let p = plus.cmp(other_plus);
                    if matches!(p, std::cmp::Ordering::Equal) {
                        minus.cmp(other_minus)
                    }else{
                        p
                    }
                }
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AxisBinding {
    kind: AxisBindingKind,
    mod_stack: Vec<Modifier>,
}

impl PartialOrd for AxisBinding {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.kind.partial_cmp(&other.kind)
    }
}

impl Ord for AxisBinding {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.kind.cmp(&other.kind)
    }
}

impl AxisBinding {
    pub fn mods(&self) -> &[Modifier] {
        &self.mod_stack
    }
    pub fn with_modifier(mut self, m: Modifier) -> Self {
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
        self.with_modifier(Modifier::INVERT.clone())
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

pub struct ValueBinding<T> {
    bindings: Vec<AxisBinding>,
    mod_stack: Vec<Modifier>,
    event: fn(f32) -> Option<T>,
    /// The last value feed into this binding.
    state: f32,
    /// Last instant that the value transitioned from zero to a non-zero value or a non-zero value to zero.
    last_change: Instant,
}

impl<T> ValueBinding<T> {
    pub fn last_change(&self) -> Instant {
        self.last_change
    }
    pub fn value(&self) -> f32 {
        let mut v = self.state;
        for m in &self.mod_stack {
            v = m.do_thing(v);
        }
        v
    }
    pub fn bindings(&self) -> &[AxisBinding] {
        &self.bindings
    }
    pub fn bindings_mut(&mut self) -> &mut [AxisBinding] {
        &mut self.bindings
    }
    pub fn feed(&mut self, value: f32) -> Option<T> {
        if (self.state == 0. && value != 0.) || (self.state != 0. && value == 0.) {
            self.last_change = Instant::now();
        }
        self.state = value;
        (self.event)(value)
    }
    pub fn from_parts(
        bindings: Vec<AxisBinding>,
        mod_stack: Vec<Modifier>,
        event: fn(f32) -> Option<T>,
    ) -> Self {
        Self {
            bindings,
            mod_stack,
            event,
            state: 0.,
            last_change: Instant::now(),
        }
    }
    pub fn from_bindings(bindings: Vec<AxisBinding>) -> Self {
        Self {
            bindings,
            mod_stack: vec![],
            event: no_event,
            state: 0.,
            last_change: Instant::now(),
        }
    }
    pub fn from_binding(binding: AxisBinding) -> Self {
        Self {
            bindings: vec![binding],
            mod_stack: vec![],
            event: no_event,
            state: 0.,
            last_change: Instant::now(),
        }
    }
    pub fn with_event(mut self, event: fn(f32) -> Option<T>) -> Self {
        self.event = event;
        self
    }
    pub fn with_modifier(mut self, modifier: Modifier) -> Self {
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
    x_bindings: Vec<AxisBinding>,
    x_mod_stack: Vec<Modifier>,
    y_bindings: Vec<AxisBinding>,
    y_mod_stack: Vec<Modifier>,
    event: fn(Vec2) -> Option<T>,
    state: Vec2,
    /// Last instant that the value transitioned from zero to a non-zero value or a non-zero value to zero.
    last_change: Instant,
}
impl<T> DualValueBinding<T> {
    pub fn last_change(&self) -> Instant {
        self.last_change
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
        if (self.state.y == 0. && value.y != 0.)
            || (self.state.y != 0. && value.y == 0.)
            || (self.state.x == 0. && value.x != 0.)
            || (self.state.x != 0. && value.x == 0.)
        {
            self.last_change = Instant::now();
        }
        self.state = value;
        (self.event)(value)
    }
    pub fn value(&self) -> Vec2 {
        let mut v = self.state;
        for m in &self.x_mod_stack {
            v.x = m.do_thing(v.x);
        }
        for m in &self.y_mod_stack {
            v.y = m.do_thing(v.y);
        }
        v
    }
    pub fn with_x_modifier(mut self, modifier: Modifier) -> Self {
        self.x_mod_stack.push(modifier);
        self
    }
    pub fn with_y_modifier(mut self, modifier: Modifier) -> Self {
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
            state: Vec2::default(),
            last_change: Instant::now(),
        }
    }
    pub fn from_bindings(x: Vec<AxisBinding>, y: Vec<AxisBinding>) -> Self {
        Self {
            x_bindings: x,
            y_bindings: y,
            x_mod_stack: vec![],
            y_mod_stack: vec![],
            event: no_event_dual,
            state: Vec2::default(),
            last_change: Instant::now(),
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
