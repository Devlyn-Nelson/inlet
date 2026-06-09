use std::time::{Duration, Instant};

use bevy::{
    input::{keyboard::KeyCode, mouse::MouseButton},
    prelude::GamepadButton,
};

use crate::{
    axis::ValueState,
    clash_manager::{BevyAxisKind, BevyButtonKind, BevyInputKind, InputValue},
    value_to_press,
};
#[allow(unpredictable_function_pointer_comparisons)]
#[derive(Debug, Clone, PartialEq)]
pub struct BevyAxisButton {
    axis: BevyAxisKind,
    is_pressed_fn: fn(f32) -> bool,
}

impl BevyAxisButton {
    /// Returns a new `BevyAxisButton` where zero is unpressed and any other value is pressed.
    pub fn new_standard(axis: BevyAxisKind) -> Self {
        Self {
            axis,
            is_pressed_fn: value_to_press,
        }
    }
    /// Returns a new `BevyAxisButton` with a custom function for mapping axis values to button pressed.
    pub fn new_custom(axis: BevyAxisKind, is_pressed_fn: fn(f32) -> bool) -> Self {
        Self {
            axis,
            is_pressed_fn,
        }
    }
    /// Returns a new `BevyAxisButton` where negative values are pressed otherwise unpressed.
    pub fn new_negative_only(axis: BevyAxisKind) -> Self {
        Self {
            axis,
            is_pressed_fn: f32::is_sign_negative,
        }
    }
    /// Returns a new `BevyAxisButton` where positive non-zero values are pressed otherwise unpressed.
    pub fn new_positive_only(axis: BevyAxisKind) -> Self {
        Self {
            axis,
            is_pressed_fn: positive_only,
        }
    }
    pub(crate) fn apply(&self, value: InputValue) -> bool {
        (self.is_pressed_fn)(value.get_value())
    }
}

impl From<GamepadButton> for BevyAxisButton {
    fn from(value: GamepadButton) -> Self {
        BevyAxisButton {
            axis: BevyAxisKind::GamepadButton(value),
            is_pressed_fn: value_to_press,
        }
    }
}

impl From<BevyAxisKind> for BevyAxisButton {
    fn from(value: BevyAxisKind) -> Self {
        BevyAxisButton {
            axis: value,
            is_pressed_fn: value_to_press,
        }
    }
}

fn positive_only(asdf: f32) -> bool {
    asdf > 0.
}

#[derive(Debug, Clone, PartialEq)]
pub enum ButtonBindingKind {
    Standard(BevyButtonKind),
    Axis(BevyAxisButton),
}

impl ButtonBindingKind {
    pub fn kind(&self) -> BevyInputKind {
        match self {
            ButtonBindingKind::Standard(bevy_button_kind) => {
                BevyInputKind::Button(*bevy_button_kind)
            }
            ButtonBindingKind::Axis(bevy_axis_button) => BevyInputKind::Axis(bevy_axis_button.axis),
        }
    }
    pub(crate) fn apply(&self, value: InputValue) -> bool {
        match self {
            ButtonBindingKind::Standard(_) => value.is_pressed(),
            ButtonBindingKind::Axis(bevy_axis_button) => bevy_axis_button.apply(value),
        }
    }
}

impl From<KeyCode> for ButtonBindingKind {
    fn from(value: KeyCode) -> Self {
        Self::Standard(value.into())
    }
}

impl From<MouseButton> for ButtonBindingKind {
    fn from(value: MouseButton) -> Self {
        Self::Standard(value.into())
    }
}

impl From<GamepadButton> for ButtonBindingKind {
    fn from(value: GamepadButton) -> Self {
        Self::Standard(value.into())
    }
}

impl From<BevyButtonKind> for ButtonBindingKind {
    fn from(value: BevyButtonKind) -> Self {
        Self::Standard(value.into())
    }
}

impl From<BevyAxisKind> for ButtonBindingKind {
    fn from(value: BevyAxisKind) -> Self {
        Self::Axis(value.into())
    }
}

impl From<BevyInputKind> for ButtonBindingKind {
    fn from(value: BevyInputKind) -> Self {
        match value {
            BevyInputKind::Axis(bevy_axis_kind) => bevy_axis_kind.into(),
            BevyInputKind::Button(bevy_button_kind) => bevy_button_kind.into(),
        }
    }
}

impl From<BevyAxisButton> for ButtonBindingKind {
    fn from(value: BevyAxisButton) -> Self {
        Self::Axis(value)
    }
}

/// A set of buttons that must all be pressed at once to be considered active.
#[derive(Debug, Clone, PartialEq)]
pub struct ButtonChord {
    actions: Vec<ButtonBindingKind>,
}

impl ButtonChord {
    pub fn bindings(&self) -> &[ButtonBindingKind] {
        &self.actions
    }
    pub fn len(&self) -> usize {
        self.actions.len()
    }
    pub fn input_kinds(&self) -> Vec<BevyInputKind> {
        self.actions.iter().map(|asdf| asdf.kind()).collect()
    }
    pub fn new(bindings: Vec<ButtonBindingKind>) -> Self {
        Self { actions: bindings }
    }
    pub(crate) fn apply(&self, value: InputValue) -> bool {
        if self.actions.is_empty() {
            value.is_pressed()
        } else {
            self.actions[0].apply(value)
        }
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
#[derive(Debug, Clone, PartialEq)]
pub struct ButtonCombo {
    actions: Vec<ButtonBindingKind>,
    current_index: usize,
    last_hit: Instant,
    tolerance: Duration,
    rules: ButtonComboRules,
}

impl ButtonCombo {
    pub fn bindings(&self) -> &[ButtonBindingKind] {
        &self.actions
    }
    pub fn input_kinds(&self) -> Vec<BevyInputKind> {
        self.actions.iter().map(|a| a.kind()).collect()
    }
    pub fn rules(&self) -> ButtonComboRules {
        self.rules
    }
    /// Creates a new button combo bindings.
    pub fn new_with_tolerance(
        bindings: Vec<ButtonBindingKind>,
        rules: ButtonComboRules,
        tolerance: Duration,
    ) -> Self {
        if bindings.len() <= 1 {
            bevy::log::warn!("inlet detected a button combo that is less than 2 buttons long.")
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
    pub fn new_with_tolerance_default_rules(
        bindings: Vec<ButtonBindingKind>,
        tolerance: Duration,
    ) -> Self {
        Self::new_with_tolerance(bindings, ButtonComboRules::default(), tolerance)
    }
    /// Creates a new button combo bindings with a tolerance of 250 milliseconds (quarter second).
    pub fn new(bindings: Vec<ButtonBindingKind>, rules: ButtonComboRules) -> Self {
        Self::new_with_tolerance(bindings, rules, Duration::from_millis(250))
    }
    /// Creates a new button combo bindings with a tolerance of 250 milliseconds (quarter second).
    pub fn new_default_rules(bindings: Vec<ButtonBindingKind>) -> Self {
        Self::new_with_tolerance(
            bindings,
            ButtonComboRules::default(),
            Duration::from_millis(250),
        )
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
    pub fn expected_binding(&self) -> &ButtonBindingKind {
        let i = if self.current_index != 0 && self.last_hit.elapsed() > self.tolerance {
            0
        } else {
            self.current_index
        };
        &self.actions[i]
    }
    /// Grabs the binding after [`Self::expected_binding`] or `None` if the [`Self::expected_binding`] is the last
    /// binding to expect.
    pub fn next_binding(&self) -> Option<&ButtonBindingKind> {
        let i = self.current_index + 1;
        if i == self.actions.len() {
            None
        } else {
            Some(&self.actions[i])
        }
    }
    /// Grabs the binding before [`Self::expected_binding`] or `None` if the [`Self::expected_binding`] is the first
    /// binding to expect.
    pub fn previous_binding(&self) -> Option<&ButtonBindingKind> {
        if self.current_index == 0 {
            None
        } else {
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
    pub fn expected_binding_mut(&mut self) -> &mut ButtonBindingKind {
        if self.current_index != 0 && self.last_hit.elapsed() > self.tolerance {
            self.current_index = 0;
        }
        &mut self.actions[self.current_index]
    }
    /// Grabs the binding after [`Self::expected_binding`] or `None` if the [`Self::expected_binding`] is the last
    /// binding to expect.
    pub fn next_binding_mut(&mut self) -> Option<&mut ButtonBindingKind> {
        let i = self.current_index + 1;
        if i == self.actions.len() {
            None
        } else {
            Some(&mut self.actions[i])
        }
    }
    /// Grabs the binding before [`Self::expected_binding`] or `None` if the [`Self::expected_binding`] is the first
    /// binding to expect.
    pub fn previous_binding_mut(&mut self) -> Option<&mut ButtonBindingKind> {
        if self.current_index == 0 {
            None
        } else {
            Some(&mut self.actions[self.current_index - 1])
        }
    }
    /// Tells the combo that the next expected button was pressed "on time". Returns `true` if the combo was
    /// completed, which also indicates that the combo will reset to expect the first button press.
    pub fn hit(&mut self) -> bool {
        self.last_hit = Instant::now();
        let next = self.current_index + 1;
        let out = next == self.actions.len();
        if out {
            self.current_index = 0;
        } else {
            self.current_index = next;
        };
        out
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ButtonBinding {
    /// A set of [`ButtonBinding`] that must all be active at once to be active.
    Chord(ButtonChord),
    /// A set of [`ButtonBinding`] that must be pressed one after another to become active.
    Combo(ButtonCombo),
    /// standard button from bevy inputs
    Single(ButtonBindingKind),
}

impl ButtonBinding {
    pub fn input_kinds(&self) -> Vec<BevyInputKind> {
        let mut out = Vec::with_capacity(1);
        match self {
            ButtonBinding::Chord(button_chord) => out.extend(button_chord.input_kinds()),
            ButtonBinding::Combo(button_combo) => out.extend(button_combo.input_kinds()),
            ButtonBinding::Single(input) => out.push(input.kind()),
        }
        out
    }
}

impl From<KeyCode> for ButtonBinding {
    fn from(value: KeyCode) -> Self {
        ButtonBinding::Single(value.into())
    }
}

impl From<MouseButton> for ButtonBinding {
    fn from(value: MouseButton) -> Self {
        ButtonBinding::Single(value.into())
    }
}

impl From<GamepadButton> for ButtonBinding {
    fn from(value: GamepadButton) -> Self {
        ButtonBinding::Single(value.into())
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

impl From<BevyButtonKind> for ButtonBinding {
    fn from(value: BevyButtonKind) -> Self {
        Self::Single(value.into())
    }
}

impl From<BevyInputKind> for ButtonBinding {
    fn from(value: BevyInputKind) -> Self {
        Self::Single(value.into())
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
            ActionableState::Released => (0., 0.),
            ActionableState::JustPressed => (0., 1.),
            ActionableState::Pressed => (1., 1.),
            ActionableState::JustReleased => (1., 0.),
        };
        ValueState {
            previous,
            current,
            last_transition: self.start,
        }
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
    pub(crate) mocked: bool,
}

impl<T> ActionBinding<T> {
    pub fn input_kinds(&self) -> Vec<BevyInputKind> {
        let mut out = Vec::default();
        for b in &self.bindings {
            out.extend(b.input_kinds());
        }
        out
    }
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
            mocked: false,
        }
    }
    pub fn new_no_event(bindings: Vec<ButtonBinding>) -> Self {
        Self {
            bindings,
            event: ButtonEventBinding::None,
            state: ButtonState::default(),
            mocked: false,
        }
    }

    /// Returns a reference to the current state of the binding.
    pub fn state(&self) -> &ButtonState {
        &self.state
    }

    /// Feeds the state of the binding and returns a `T` if configured to do so for the current state.
    pub fn feed(&mut self, pressed: bool) -> Option<T> {
        if self.state.feed(pressed) {
            self.event.try_get_event(&self.state)
        } else {
            None
        }
    }
    pub fn mock(&mut self, pressed: bool) {
        self.mocked = pressed;
    }
    pub fn mock_clear(&mut self) {
        self.mocked = false;
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
        ActionBinding::new_no_event(vec![ButtonBinding::Single(value.into())])
    }
}

impl<T> From<MouseButton> for ActionBinding<T> {
    fn from(value: MouseButton) -> Self {
        ActionBinding::new_no_event(vec![ButtonBinding::Single(value.into())])
    }
}

impl<T> From<GamepadButton> for ActionBinding<T> {
    fn from(value: GamepadButton) -> Self {
        ActionBinding::new_no_event(vec![ButtonBinding::Single(value.into())])
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

impl<T> From<ButtonChord> for ActionBinding<T> {
    fn from(value: ButtonChord) -> Self {
        ActionBinding::new_no_event(vec![ButtonBinding::Chord(value)])
    }
}

impl<T> From<ButtonCombo> for ActionBinding<T> {
    fn from(value: ButtonCombo) -> Self {
        ActionBinding::new_no_event(vec![ButtonBinding::Combo(value)])
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
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
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
