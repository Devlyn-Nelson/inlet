use std::time::{Duration, Instant};

use bevy::{
    input::{keyboard::KeyCode, mouse::MouseButton},
    prelude::GamepadButton,
};

use crate::axis::{AxisBinding, ValueState};

/// A set of buttons that must all be pressed at once to be considered active.
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
    pub fn new(bindings: Vec<ButtonBinding>) -> Self {
        Self { actions: bindings }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ButtonComboRules {
    None,
    PreviousMustBeReleased,
    #[default]
    NextMustBeReleased,
}

/// A set of buttons that must all be pressed one after another to become active.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ButtonCombo {
    actions: Vec<ButtonBinding>,
    current_index: usize,
    last_hit: Instant,
    tolerance: Duration,
    rules: ButtonComboRules,
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
    pub fn rules(&self) -> ButtonComboRules {
        self.rules
    }
    /// Creates a new button combo bindings.
    pub fn new_with_tolerance(bindings: Vec<ButtonBinding>, rules: ButtonComboRules, tolerance: Duration) -> Self {
        if bindings.len() <= 1 {
            bevy::log::warn!(
                "inlet detected a button combo that is less than 2 buttons long."
            )
        }
        ButtonCombo {
            actions: bindings,
            current_index: 0,
            last_hit: Instant::now(),
            tolerance,
            rules,
        }
    }
    /// Creates a new button combo bindings with default [`ButtonComboRules`].
    pub fn new_with_tolerance_default_rules(bindings: Vec<ButtonBinding>, tolerance: Duration) -> Self {
        Self::new_with_tolerance(bindings, ButtonComboRules::default(), tolerance)
    }
    /// Creates a new button combo bindings with a tolerance of 250 milliseconds (quarter second).
    pub fn new(bindings: Vec<ButtonBinding>, rules: ButtonComboRules) -> Self {
        Self::new_with_tolerance(bindings, rules, Duration::from_millis(250))
    }
    /// Creates a new button combo bindings with a tolerance of 250 milliseconds (quarter second).
    pub fn new_default_rules(bindings: Vec<ButtonBinding>) -> Self {
        
        Self::new_with_tolerance(bindings, ButtonComboRules::default(), Duration::from_millis(250))
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
    /// Sets the rules.
    pub fn with_rules(mut self, rules: ButtonComboRules) -> Self {
        self.rules = rules;
        self
    }
    /// Grabs the expected button binding that would need to happen in order for the combo to be progressed.
    /// 
    /// # Warning
    /// 
    /// If the timer between expected presses ran out, it will return the first binding.
    pub fn expected_binding(&self) -> &ButtonBinding {
        let i = if self.current_index != 0 && self.last_hit.elapsed() > self.tolerance {
           0
        }else{
            self.current_index
        };
        &self.actions[i]
    }
    /// Grabs the binding after [`Self::expected_binding`] or `None` if the [`Self::expected_binding`] is the last
    /// binding to expect.
    pub fn next_binding(&self) -> Option<&ButtonBinding> {
        let i = self.current_index + 1;
        if i == self.actions.len() {
            None
        }else{
            Some(&self.actions[i])
        }
    }
    /// Grabs the binding before [`Self::expected_binding`] or `None` if the [`Self::expected_binding`] is the first
    /// binding to expect.
    pub fn previous_binding(&self) -> Option<&ButtonBinding> {
        if self.current_index == 0 {
            None
        }else{
            Some(&self.actions[self.current_index - 1])
        }
    }
    /// Grabs the expected button binding that would need to happen in order for the combo to be progressed.
    /// 
    /// # Warning
    /// 
    /// This will update the state of the combo if the timer between expected presses has run out.
    /// 
    /// If the duration between the last time `self.hit()` and the call of this function is greater than `self.tolerance`
    /// the combo will reset to the beginning of the combo.
    pub fn expected_binding_mut(&mut self) -> &mut ButtonBinding {
        if self.current_index != 0 && self.last_hit.elapsed() > self.tolerance {
            self.current_index = 0;
        }
        &mut self.actions[self.current_index]
    }
    /// Grabs the binding after [`Self::expected_binding`] or `None` if the [`Self::expected_binding`] is the last
    /// binding to expect.
    pub fn next_binding_mut(&mut self) -> Option<&mut ButtonBinding> {
        let i = self.current_index + 1;
        if i == self.actions.len() {
            None
        }else{
            Some(&mut self.actions[i])
        }
    }
    /// Grabs the binding before [`Self::expected_binding`] or `None` if the [`Self::expected_binding`] is the first
    /// binding to expect.
    pub fn previous_binding_mut(&mut self) -> Option<&mut ButtonBinding> {
        if self.current_index == 0 {
            None
        }else{
            Some(&mut self.actions[self.current_index - 1])
        }
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
    /// A set of [`ButtonBinding`] that must all be active at once to be active.
    Chord(ButtonChord),
    /// A set of [`ButtonBinding`] that must be pressed one after another to become active.
    Combo(ButtonCombo),
    /// A [`KeyCode`] that will be checked from `bevy_input`.
    Keyboard(KeyCode),
    /// A [`MouseButton`] that will be checked from `bevy_input`.
    Mouse(MouseButton),
    /// A [`GamepadButton`] that will be checked from `bevy_input`.
    Gamepad(GamepadButton),
    /// An [`AxisBinding`] that will be interpreted as a button. A value of 0 is Released otherwise it is Pressed.
    Axis(Box<AxisBinding>),
    /// Contains a mock input value. Must be set.
    Mock(bool),
}

impl ButtonBinding {
    pub fn is_mock(&self) -> bool {
        matches!(self, Self::Mock(_))
    }
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
                | ButtonBinding::Axis(_)
                | ButtonBinding::Mock(_) => std::cmp::Ordering::Less,
            },
            ButtonBinding::Combo(button_combo) => match other {
                ButtonBinding::Chord(_) => std::cmp::Ordering::Greater,
                ButtonBinding::Combo(other_button_combo) => button_combo.cmp(other_button_combo),
                ButtonBinding::Keyboard(_)
                | ButtonBinding::Mouse(_)
                | ButtonBinding::Gamepad(_)
                | ButtonBinding::Axis(_)
                | ButtonBinding::Mock(_) => std::cmp::Ordering::Less,
            },
            ButtonBinding::Keyboard(asdf) => match other {
                ButtonBinding::Combo(_) | ButtonBinding::Chord(_) => std::cmp::Ordering::Greater,
                ButtonBinding::Keyboard(o_asdf) => asdf.cmp(o_asdf),
                ButtonBinding::Mouse(_)
                | ButtonBinding::Gamepad(_)
                | ButtonBinding::Axis(_)
                | ButtonBinding::Mock(_) => std::cmp::Ordering::Less,
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
                ButtonBinding::Gamepad(_) | ButtonBinding::Axis(_) | ButtonBinding::Mock(_) => {
                    std::cmp::Ordering::Less
                }
            },
            ButtonBinding::Gamepad(asdf) => match other {
                ButtonBinding::Mouse(_)
                | ButtonBinding::Keyboard(_)
                | ButtonBinding::Combo(_)
                | ButtonBinding::Chord(_) => std::cmp::Ordering::Greater,
                ButtonBinding::Gamepad(o_asdf) => asdf.cmp(o_asdf),
                ButtonBinding::Axis(_) | ButtonBinding::Mock(_) => std::cmp::Ordering::Less,
            },
            ButtonBinding::Axis(asdf) => match other {
                ButtonBinding::Gamepad(_)
                | ButtonBinding::Mouse(_)
                | ButtonBinding::Keyboard(_)
                | ButtonBinding::Combo(_)
                | ButtonBinding::Chord(_) => std::cmp::Ordering::Greater,
                ButtonBinding::Axis(o_asdf) => asdf.cmp(o_asdf),
                ButtonBinding::Mock(_) => std::cmp::Ordering::Less,
            },
            ButtonBinding::Mock(asdf) => match other {
                ButtonBinding::Gamepad(_)
                | ButtonBinding::Mouse(_)
                | ButtonBinding::Keyboard(_)
                | ButtonBinding::Combo(_)
                | ButtonBinding::Chord(_)
                | ButtonBinding::Axis(_) => std::cmp::Ordering::Greater,
                ButtonBinding::Mock(o_asdf) => asdf.cmp(o_asdf),
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

/// A stored [`ActionableState`] and the [`Instant`] of [`Self::feed`] changed the state to
/// [`ActionableState::JustPressed`] or [`ActionableState::JustReleased`].
#[derive(Debug, Hash, Copy, Clone, PartialEq, Eq)]
pub struct ButtonState {
    pub(crate) kind: ActionableState,
    pub(crate) start: Instant,
}

impl ButtonState {
    pub fn value_state(&self) -> ValueState {
        let (previous, current) = match self.kind {
            ActionableState::Released => (0.,0.),
            ActionableState::JustPressed => (0., 1.),
            ActionableState::Pressed => (1., 1.),
            ActionableState::JustReleased => (1., 0.),
        };
        ValueState { previous, current, last_transition: self.start }
    }
    pub fn kind(&self) -> &ActionableState {
        &self.kind
    }
    /// The amount of time passed between now and the last time the internal state
    /// was changed to [`ActionableState::JustPressed`] or [`ActionableState::JustReleased`].
    pub fn last_transition(&self) -> Duration {
        self.start.elapsed()
    }
    /// Returns `true` if the internal [`ActionableState`] is `JustPressed`.
    pub fn just_pressed(&self) -> bool {
        matches!(self.kind, ActionableState::JustPressed)
    }
    /// Returns `true` if the internal [`ActionableState`] is `JustPressed` or `Pressed`.
    pub fn pressed(&self) -> bool {
        matches!(
            self.kind,
            ActionableState::Pressed | ActionableState::JustPressed
        )
    }
    /// Returns `true` if the internal [`ActionableState`] is `Pressed` and the result of [`Self::last_transition`]
    /// is greater than or equal to `duration`.
    ///
    /// Note that a state of `JustPressed` will always return `false`.
    pub fn held_for(&self, duration: &Duration) -> bool {
        matches!(self.kind, ActionableState::Pressed) && self.start.elapsed() >= *duration
    }
    /// Returns `true` if the internal [`ActionableState`] is `Pressed` and the result of [`Self::last_transition`]
    /// is greater than or equal to `start` and less than `stop`.
    pub fn held_range(&self, start: &Duration, stop: &Duration) -> bool {
        let elapsed = self.start.elapsed();
        matches!(self.kind, ActionableState::Pressed) && elapsed >= *start && elapsed < *stop
    }
    /// Returns time elapsed for a pressed state or `None`.
    pub fn try_get_held_duration(&self) -> Option<Duration> {
        if matches!(self.kind, ActionableState::Pressed) {
            Some(self.start.elapsed())
        } else {
            None
        }
    }
    /// Returns `true` if the internal [`ActionableState`] is `JustReleased`.
    pub fn just_released(&self) -> bool {
        matches!(self.kind, ActionableState::JustReleased)
    }
    /// Returns `true` if the internal [`ActionableState`] is `JustReleased` or `Released`.
    pub fn released(&self) -> bool {
        matches!(
            self.kind,
            ActionableState::Released | ActionableState::JustReleased
        )
    }
    /// `pressed` will feed the internal state `true` meaning that the action is being held.
    ///
    /// Returning `true` signifies that the internal state has changed.
    pub fn feed(&mut self, pressed: bool) -> bool {
        match self.kind.tick(pressed) {
            ActionableStateTick::None => false,
            ActionableStateTick::Changed => true,
            ActionableStateTick::Transitioned => {
                self.start = Instant::now();
                true
            }
        }
    }
}

impl Default for ButtonState {
    fn default() -> Self {
        Self {
            kind: Default::default(),
            start: Instant::now(),
        }
    }
}

/// Describes if the state of a [`ActionableState`] changed and how.
pub enum ActionableStateTick {
    /// No change
    None,
    /// The state changed state but did not transition.
    Changed,
    /// The state became [`ActionableState::JustPressed`] or [`ActionableState::JustReleased`]
    Transitioned,
}

/// Describes the state of a button.
#[derive(Debug, Hash, Copy, Clone, PartialEq, Eq, Default)]
pub enum ActionableState {
    /// Button is not pressed.
    #[default]
    Released,
    /// Button was pressed this frame.
    JustPressed,
    /// Button has been pressed for more than one frame.
    Pressed,
    /// Button was `Pressed` or `JustPressed` before this frame but is no longer pressed.
    JustReleased,
}

impl ActionableState {
    /// Updates `self` to appropriate state using `pressed` to drive the simple state of the input.
    ///
    /// if:
    ///   - `pressed == true && self.is_pressed && !self.is_just_pressed` => No Change.
    ///   - `pressed == true && self.is_just_pressed` => Change to `ActionableState::Pressed`.
    ///   - `pressed == true && self.is_released` => Transition to `ActionableState::JustPressed`.
    ///   - `pressed == false && self.is_released && !self.is_just_release` => No Change.
    ///   - `pressed == false && self.is_just_release` => Change to `ActionableState::Released`.
    ///   - `pressed == false && self.is_pressed` => Transition to `ActionableState::JustReleased`.
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
/// An Action or Button with an [`ActionableState`], one or many [`ButtonBinding`], and a [`ButtonEventBinding<T>`].
pub struct ActionBinding<T> {
    pub(crate) bindings: Vec<ButtonBinding>,
    pub(crate) event: ButtonEventBinding<T>,
    pub(crate) state: ButtonState,
}

impl<T> ActionBinding<T> {
    pub fn bindings(&self) -> &[ButtonBinding] {
        &self.bindings
    }
    pub fn bindings_mut(&mut self) -> &mut [ButtonBinding] {
        &mut self.bindings
    }
    /// Returns `true` if the internal [`ActionableState`] is `JustPressed`.
    #[inline]
    pub fn just_pressed(&self) -> bool {
        self.state.just_pressed()
    }
    /// Returns `true` if the internal [`ActionableState`] is `JustPressed` or `Pressed`.
    #[inline]
    pub fn pressed(&self) -> bool {
        self.state.pressed()
    }
    /// Returns `true` if the internal [`ActionableState`] is `Pressed` and the result of [`Self::last_transition`]
    /// is greater than or equal to `duration`.
    ///
    /// Note that a state of `JustPressed` will always return `false`.
    #[inline]
    pub fn held_for(&self, duration: &Duration) -> bool {
        self.state.held_for(duration)
    }
    /// Returns `true` if the internal [`ActionableState`] is `Pressed` and the result of [`Self::last_transition`]
    /// is greater than or equal to `start` and less than `stop`.
    #[inline]
    pub fn held_range(&self, start: &Duration, stop: &Duration) -> bool {
        self.state.held_range(start, stop)
    }
    /// Returns time elapsed for a pressed state or `None`.
    #[inline]
    pub fn try_get_held_duration(&self) -> Option<Duration> {
        self.state.try_get_held_duration()
    }
    /// Returns `true` if the internal [`ActionableState`] is `JustReleased`.
    #[inline]
    pub fn just_released(&self) -> bool {
        self.state.just_released()
    }
    /// Returns `true` if the internal [`ActionableState`] is `JustReleased` or `Released`.
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

    /// Returns a reference to the current state of the binding.
    pub fn state(&self) -> &ButtonState {
        &self.state
    }

    /// Feeds the state of the binding.
    pub fn feed(&mut self, pressed: bool) -> bool {
        self.state.feed(pressed)
    }

    /// Feeds the state of the binding and returns a `T` if configured to do so for the current state.
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

/// Conditionals for a [`ActionBinding`] to emit a [`Message`](bevy::prelude::Message).
#[derive(Clone)]
pub enum ButtonEventBinding<T> {
    /// When the state transitions to `JustPressed`.
    WhenPressed(fn() -> T),
    /// While the state is `Pressed`.
    WhilePressed(fn() -> T),
    /// When the state is `Pressed` for a duration.
    WhenPressedFor(Duration, fn() -> T, bool),
    /// While the state is `Pressed` for a duration.
    WhilePressedFor(Duration, fn() -> T),
    /// While the state is `Pressed` for a duration between `start` sand `stop`.
    PressedRange {
        start: Duration,
        end: Duration,
        event: fn() -> T,
    },
    /// Passes the [`Duration`] the state has been Pressed into your function allowing you to optionally return
    /// a [`Message`] if you want it sent.
    CapturePressDuration(fn(Duration) -> Option<T>),
    /// When the state transitions to `JustReleased`.
    WhenReleased(fn() -> T),
    /// While the state is `Released`.
    WhileReleased(fn() -> T),
    /// Never send messages
    None,
}

impl<T> ButtonEventBinding<T> {
    pub fn try_get_event(&mut self, state: &ButtonState) -> Option<T> {
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
            ButtonEventBinding::WhenPressedFor(duration, event, activated) => {
                if state.held_for(duration) {
                    if !*activated {
                        *activated = true;
                        return Some(event());
                    }
                } else if *activated {
                    *activated = false;
                }
            }
            ButtonEventBinding::WhilePressedFor(duration, event) => {
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
                    return event(dur);
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
    pub fn when_pressed_for(event: fn() -> T, duration: Duration) -> Self {
        Self::WhenPressedFor(duration, event, false)
    }
    pub fn while_pressed_for(event: fn() -> T, duration: Duration) -> Self {
        Self::WhilePressedFor(duration, event)
    }
    pub fn pressed_range(event: fn() -> T, start: Duration, end: Duration) -> Self {
        Self::PressedRange { start, end, event }
    }
    pub fn capture_press_duration(event: fn(Duration) -> Option<T>) -> Self {
        Self::CapturePressDuration(event)
    }
}
