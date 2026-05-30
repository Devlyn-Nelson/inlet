use std::time::{Duration, Instant};

use bevy::{
    input::{keyboard::KeyCode, mouse::MouseButton},
    prelude::GamepadButton,
};

use crate::axis::AxisBinding;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ButtonChord {
    actions: Vec<ButtonBinding>,
}

impl Ord for ButtonChord {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let len = self.actions.len().min(other.actions.len());
        for i in 0..len {
            let cmp = self.actions[i].cmp(&other.actions[i]);
            if matches!(cmp, std::cmp::Ordering::Greater | std::cmp::Ordering::Less) {
                return cmp;
            }
        }
        std::cmp::Ordering::Equal
    }
}

impl PartialOrd for ButtonChord {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl ButtonChord {
    pub fn bindings(&self) -> &[ButtonBinding] {
        &self.actions
    }
    pub fn bindings_mut(&mut self) -> &mut [ButtonBinding] {
        &mut self.actions
    }
    /// Creates a new button combo bindings with a tolerance of 250 milliseconds (quarter second).
    pub fn new(bindings: Vec<ButtonBinding>) -> Self {
        Self { actions: bindings }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ButtonCombo {
    actions: Vec<ButtonBinding>,
    current_index: usize,
    last_hit: Instant,
    tolerance: Duration,
}

impl Ord for ButtonCombo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let len = self.actions.len().min(other.actions.len());
        for i in 0..len {
            let cmp = self.actions[i].cmp(&other.actions[i]);
            if matches!(cmp, std::cmp::Ordering::Greater | std::cmp::Ordering::Less) {
                return cmp;
            }
        }
        std::cmp::Ordering::Equal
    }
}

impl PartialOrd for ButtonCombo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl ButtonCombo {
    /// Creates a new button combo bindings with a tolerance of 250 milliseconds (quarter second).
    pub fn new(bindings: Vec<ButtonBinding>) -> Self {
        ButtonCombo {
            actions: bindings,
            current_index: 0,
            last_hit: Instant::now(),
            tolerance: Duration::from_millis(250),
        }
    }
    /// Returns the amount of time allowed to pass before the combo gets reset.
    pub fn tolerance(&self) -> Duration {
        self.tolerance
    }
    /// Sets the amount of time allowed to pass before the combo gets reset.
    pub fn with_tolerance(mut self, tolerance: Duration) -> Self {
        self.tolerance = tolerance;
        self
    }
    /// Grabs the next expected button binding that would need to happen in order for the combo to be progressed.
    /// If the duration between the last time `self.hit()` and the call of this function is greater than `self.tolerance`
    /// the combo will reset to the beginning of the combo.
    pub fn next_binding(&mut self) -> &mut ButtonBinding {
        if self.current_index != 0 && self.last_hit.elapsed() > self.tolerance {
            self.current_index = 0;
        }
        &mut self.actions[self.current_index]
    }
    /// Tells the combo that the next expected button was pressed "on time". Returns `true` if the combo was
    /// completed, which also indicates that the combo will reset to expect the first button press.
    pub fn hit(&mut self) -> bool {
        self.last_hit = Instant::now();
        let next = self.current_index + 1;
        let out = if next == self.actions.len() {
            self.current_index = 0;
            true
        } else {
            self.current_index = next;
            false
        };
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ButtonBinding {
    Chord(ButtonChord),
    Combo(ButtonCombo),
    Keyboard(KeyCode),
    Mouse(MouseButton),
    Gamepad(GamepadButton),
    Axis(Box<AxisBinding>),
}

impl PartialOrd for ButtonBinding {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn mouse_index(b: &MouseButton) -> u8 {
    match b {
        MouseButton::Left => 0,
        MouseButton::Right => 1,
        MouseButton::Middle => 2,
        MouseButton::Back => 3,
        MouseButton::Forward => 4,
        MouseButton::Other(_) => 5,
    }
}

impl Ord for ButtonBinding {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self {
            ButtonBinding::Chord(button_chord) => match other {
                ButtonBinding::Chord(other_button_chord) => button_chord.cmp(other_button_chord),
                ButtonBinding::Combo(_)
                | ButtonBinding::Keyboard(_)
                | ButtonBinding::Mouse(_)
                | ButtonBinding::Gamepad(_)
                | ButtonBinding::Axis(_) => std::cmp::Ordering::Less,
            },
            ButtonBinding::Combo(button_combo) => match other {
                ButtonBinding::Chord(_) => std::cmp::Ordering::Greater,
                ButtonBinding::Combo(other_button_combo) => button_combo.cmp(other_button_combo),
                ButtonBinding::Keyboard(_)
                | ButtonBinding::Mouse(_)
                | ButtonBinding::Gamepad(_)
                | ButtonBinding::Axis(_) => std::cmp::Ordering::Less,
            },
            ButtonBinding::Keyboard(asdf) => match other {
                ButtonBinding::Combo(_) | ButtonBinding::Chord(_) => std::cmp::Ordering::Greater,
                ButtonBinding::Keyboard(o_asdf) => asdf.cmp(o_asdf),
                ButtonBinding::Mouse(_) | ButtonBinding::Gamepad(_) | ButtonBinding::Axis(_) => {
                    std::cmp::Ordering::Less
                }
            },
            ButtonBinding::Mouse(asdf) => match other {
                ButtonBinding::Keyboard(_) | ButtonBinding::Combo(_) | ButtonBinding::Chord(_) => {
                    std::cmp::Ordering::Greater
                }
                ButtonBinding::Mouse(o_asdf) => {
                    if let MouseButton::Other(one) = asdf
                        && let MouseButton::Other(two) = o_asdf
                    {
                        one.cmp(two)
                    } else {
                        mouse_index(asdf).cmp(&mouse_index(o_asdf))
                    }
                }
                ButtonBinding::Gamepad(_) | ButtonBinding::Axis(_) => std::cmp::Ordering::Less,
            },
            ButtonBinding::Gamepad(asdf) => match other {
                ButtonBinding::Mouse(_)
                | ButtonBinding::Keyboard(_)
                | ButtonBinding::Combo(_)
                | ButtonBinding::Chord(_) => std::cmp::Ordering::Greater,
                ButtonBinding::Gamepad(o_asdf) => asdf.cmp(o_asdf),
                ButtonBinding::Axis(_) => std::cmp::Ordering::Less,
            },
            ButtonBinding::Axis(asdf) => match other {
                ButtonBinding::Gamepad(_)
                | ButtonBinding::Mouse(_)
                | ButtonBinding::Keyboard(_)
                | ButtonBinding::Combo(_)
                | ButtonBinding::Chord(_) => std::cmp::Ordering::Greater,
                ButtonBinding::Axis(o_asdf) => asdf.cmp(o_asdf),
            },
        }
    }
}

impl From<KeyCode> for ButtonBinding {
    fn from(value: KeyCode) -> Self {
        ButtonBinding::Keyboard(value)
    }
}

impl From<MouseButton> for ButtonBinding {
    fn from(value: MouseButton) -> Self {
        ButtonBinding::Mouse(value)
    }
}

impl From<GamepadButton> for ButtonBinding {
    fn from(value: GamepadButton) -> Self {
        ButtonBinding::Gamepad(value)
    }
}

impl From<ButtonCombo> for ButtonBinding {
    fn from(value: ButtonCombo) -> Self {
        ButtonBinding::Combo(value)
    }
}

impl From<ButtonChord> for ButtonBinding {
    fn from(value: ButtonChord) -> Self {
        ButtonBinding::Chord(value)
    }
}

impl From<AxisBinding> for ButtonBinding {
    fn from(value: AxisBinding) -> Self {
        ButtonBinding::Axis(Box::new(value))
    }
}

#[derive(Debug, Hash, Copy, Clone, PartialEq, Eq)]
pub struct ButtonState {
    pub(crate) ty: ActionableState,
    pub(crate) start: Instant,
}

impl ButtonState {
    pub fn just_pressed(&self) -> bool {
        matches!(self.ty, ActionableState::JustPressed)
    }
    pub fn pressed(&self) -> bool {
        matches!(
            self.ty,
            ActionableState::Pressed | ActionableState::JustPressed
        )
    }
    pub fn held_until(&self, duration: &Duration) -> bool {
        matches!(self.ty, ActionableState::Pressed) && self.start.elapsed() < *duration
    }
    pub fn held_for(&self, duration: &Duration) -> bool {
        matches!(self.ty, ActionableState::Pressed) && self.start.elapsed() >= *duration
    }
    pub fn held_range(&self, start: &Duration, stop: &Duration) -> bool {
        let elapsed = self.start.elapsed();
        matches!(self.ty, ActionableState::Pressed) && elapsed >= *start && elapsed < *stop
    }
    pub fn try_get_held_duration(&self) -> Option<Duration> {
        if matches!(self.ty, ActionableState::Pressed) {
            Some(self.start.elapsed())
        } else {
            None
        }
    }
    pub fn just_released(&self) -> bool {
        matches!(self.ty, ActionableState::JustReleased)
    }
    pub fn released(&self) -> bool {
        matches!(
            self.ty,
            ActionableState::Released | ActionableState::JustReleased
        )
    }
    /// `pressed` will feed the internal state `true` meaning that the action is being held.
    ///
    /// Returning `true` signifies that the internal state has changed.
    pub fn feed(&mut self, pressed: bool) -> bool {
        match self.ty.tick(pressed) {
            ActionableStateTick::None => false,
            ActionableStateTick::Changed | ActionableStateTick::Transitioned => true,
        }
    }
}

impl Default for ButtonState {
    fn default() -> Self {
        Self {
            ty: Default::default(),
            start: Instant::now(),
        }
    }
}

pub enum ActionableStateTick {
    /// No change
    None,
    /// The state changed state but did not transition.
    Changed,
    /// The state became [`ActionableState::JustPressed`] or [`ActionableState::JustReleased`]
    Transitioned,
}

#[derive(Debug, Hash, Copy, Clone, PartialEq, Eq, Default)]
pub enum ActionableState {
    /// Button is not being pressed
    #[default]
    Released,
    /// Button was pressed this frame.
    JustPressed,
    /// Button has been pressed for more than one frame but is not "held".
    Pressed,
    /// Button was `Self::Pressed | Self::JustPressed` before this frame but is no longer pressed.
    JustReleased,
}

impl ActionableState {
    /// Returns `true` when the state has transitioned between Pressed and Unpressed.
    pub fn tick(&mut self, pressed: bool) -> ActionableStateTick {
        if pressed {
            match self {
                ActionableState::Released | ActionableState::JustReleased => {
                    *self = ActionableState::JustPressed;
                    ActionableStateTick::Transitioned
                }
                ActionableState::JustPressed => {
                    *self = ActionableState::Pressed;
                    ActionableStateTick::Changed
                }
                _ => ActionableStateTick::None,
            }
        } else {
            match self {
                ActionableState::Pressed | ActionableState::JustPressed => {
                    *self = ActionableState::JustReleased;
                    ActionableStateTick::Transitioned
                }
                ActionableState::Released => ActionableStateTick::None,
                ActionableState::JustReleased => {
                    *self = ActionableState::Released;
                    ActionableStateTick::Changed
                }
            }
        }
    }
    pub fn is_released(&self) -> bool {
        matches!(self, Self::JustReleased | Self::Released)
    }
    pub fn is_pressed(&self) -> bool {
        matches!(self, Self::JustPressed | Self::Pressed)
    }
    pub fn is_just_pressed(&self) -> bool {
        matches!(self, Self::JustPressed)
    }
    pub fn is_just_released(&self) -> bool {
        matches!(self, Self::JustReleased)
    }
}

// TODO Add the last frames state to the bindings so that we don't create the same event twice (unless that is
// the desired effect). This would also enable the ability to use this library without events.
// TODO Add some way to add conditions to the event activation for example should a event happen
// `while_pressed`, `when_pressed`, `just_pressed`, `when_released`, `while_pressed_for`, `when_pressed_for`,
// `while_pressed_between`, `when_pressed_between`.
pub struct ActionBinding<T> {
    bindings: Vec<ButtonBinding>,
    event: ButtonEventBinding<T>,
    state: ButtonState,
}

impl<T> ActionBinding<T> {
    pub fn bindings(&self) -> &[ButtonBinding] {
        &self.bindings
    }
    pub fn bindings_mut(&mut self) -> &mut [ButtonBinding] {
        &mut self.bindings
    }
    #[inline]
    pub fn just_pressed(&self) -> bool {
        self.state.just_pressed()
    }
    #[inline]
    pub fn pressed(&self) -> bool {
        self.state.pressed()
    }
    #[inline]
    pub fn held_until(&self, duration: &Duration) -> bool {
        self.state.held_until(duration)
    }
    #[inline]
    pub fn held_for(&self, duration: &Duration) -> bool {
        self.state.held_for(duration)
    }
    #[inline]
    pub fn held_range(&self, start: &Duration, stop: &Duration) -> bool {
        self.state.held_range(start, stop)
    }
    #[inline]
    pub fn try_get_held_duration(&self) -> Option<Duration> {
        self.state.try_get_held_duration()
    }
    #[inline]
    pub fn just_released(&self) -> bool {
        self.state.just_released()
    }
    #[inline]
    pub fn released(&self) -> bool {
        self.state.released()
    }
    pub fn new(bindings: Vec<ButtonBinding>, event: ButtonEventBinding<T>) -> Self {
        Self {
            bindings,
            event,
            state: ButtonState::default(),
        }
    }
    pub fn new_no_event(bindings: Vec<ButtonBinding>) -> Self {
        Self {
            bindings,
            event: ButtonEventBinding::None,
            state: ButtonState::default(),
        }
    }
    pub fn state(&self) -> &ButtonState {
        &self.state
    }

    pub fn feed(&mut self, pressed: bool) -> bool {
        self.state.feed(pressed)
    }

    pub fn feed_event(&mut self, pressed: bool) -> Option<T> {
        if self.state.feed(pressed) {
            self.event.try_get_event(&self.state)
        } else {
            None
        }
    }
}

impl<T> From<(ButtonBinding, ButtonEventBinding<T>)> for ActionBinding<T> {
    fn from(value: (ButtonBinding, ButtonEventBinding<T>)) -> Self {
        ActionBinding::new(vec![value.0], value.1)
    }
}

impl<T> From<ButtonBinding> for ActionBinding<T> {
    fn from(value: ButtonBinding) -> Self {
        ActionBinding::new_no_event(vec![value])
    }
}

impl<T> From<KeyCode> for ActionBinding<T> {
    fn from(value: KeyCode) -> Self {
        ActionBinding::new_no_event(vec![ButtonBinding::Keyboard(value)])
    }
}

impl<T> From<MouseButton> for ActionBinding<T> {
    fn from(value: MouseButton) -> Self {
        ActionBinding::new_no_event(vec![ButtonBinding::Mouse(value)])
    }
}

impl<T> From<GamepadButton> for ActionBinding<T> {
    fn from(value: GamepadButton) -> Self {
        ActionBinding::new_no_event(vec![ButtonBinding::Gamepad(value)])
    }
}

impl<T> From<Vec<ButtonBinding>> for ActionBinding<T> {
    fn from(value: Vec<ButtonBinding>) -> Self {
        ActionBinding::new_no_event(value)
    }
}

impl<T> From<(Vec<ButtonBinding>, ButtonEventBinding<T>)> for ActionBinding<T> {
    fn from(value: (Vec<ButtonBinding>, ButtonEventBinding<T>)) -> Self {
        ActionBinding::new(value.0, value.1)
    }
}

#[derive(Clone)]
pub enum ButtonEventBinding<T> {
    WhenPressed(fn() -> T),
    WhilePressed(fn() -> T),
    PressedUntil(Duration, fn() -> T),
    PressedFor(Duration, fn() -> T),
    PressedRange {
        start: Duration,
        end: Duration,
        event: fn() -> T,
    },
    CapturePressDuration(fn(Duration) -> T),
    WhenReleased(fn() -> T),
    WhileReleased(fn() -> T),
    None,
}

impl<T> ButtonEventBinding<T> {
    pub fn try_get_event(&self, state: &ButtonState) -> Option<T> {
        match self {
            ButtonEventBinding::WhenPressed(event) => {
                if state.just_pressed() {
                    return Some(event());
                }
            }
            ButtonEventBinding::WhilePressed(event) => {
                if state.pressed() {
                    return Some(event());
                }
            }
            ButtonEventBinding::PressedUntil(duration, event) => {
                if state.held_until(duration) {
                    return Some(event());
                }
            }
            ButtonEventBinding::PressedFor(duration, event) => {
                if state.held_for(duration) {
                    return Some(event());
                }
            }
            ButtonEventBinding::PressedRange { start, end, event } => {
                if state.held_range(start, end) {
                    return Some(event());
                }
            }
            ButtonEventBinding::CapturePressDuration(event) => {
                if let Some(dur) = state.try_get_held_duration() {
                    return Some(event(dur));
                }
            }
            ButtonEventBinding::WhenReleased(event) => {
                if state.just_released() {
                    return Some(event());
                }
            }
            ButtonEventBinding::WhileReleased(event) => {
                if state.released() {
                    return Some(event());
                }
            }
            ButtonEventBinding::None => {}
        }
        None
    }
    pub fn none() -> Self {
        Self::None
    }
    pub fn when_pressed(event: fn() -> T) -> Self {
        Self::WhenPressed(event)
    }
    pub fn while_pressed(event: fn() -> T) -> Self {
        Self::WhilePressed(event)
    }
    pub fn when_released(event: fn() -> T) -> Self {
        Self::WhenReleased(event)
    }
    pub fn while_released(event: fn() -> T) -> Self {
        Self::WhileReleased(event)
    }
    pub fn pressed_until(event: fn() -> T, duration: Duration) -> Self {
        Self::PressedUntil(duration, event)
    }
    pub fn pressed_for(event: fn() -> T, duration: Duration) -> Self {
        Self::PressedFor(duration, event)
    }
    pub fn pressed_range(event: fn() -> T, start: Duration, end: Duration) -> Self {
        Self::PressedRange { start, end, event }
    }
    pub fn capture_press_duration(event: fn(Duration) -> T) -> Self {
        Self::CapturePressDuration(event)
    }
}
