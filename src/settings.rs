use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

use bevy::ecs::{component::Component, resource::Resource};

/// A Resource that defines default [`ClashSettings`] [`InputHandlers`](crate::manager::InputHandler) that
/// don't define their own.
#[derive(Resource, Clone, Copy, Debug)]
pub struct DefaultClashSettings(pub ClashSettings);

impl Deref for DefaultClashSettings {
    type Target = ClashSettings;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DefaultClashSettings {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
/// The settings to use for resolving clashing inputs.
///
/// attach to an entity to override global clash settings.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct ClashSettings {
    strategy: ClashStrategy,
    /// Reset the chord length on tick so that smaller chords can become active
    /// after releasing a larger chord.
    chord_regression: bool,
}

impl ClashSettings {
    pub fn new(strategy: ClashStrategy, chord_regression: bool) -> Self {
        Self {
            strategy,
            chord_regression,
        }
    }
    /// Whether to reset the chord length on tick so that smaller chords can become
    /// active after releasing a larger chord.
    pub fn chord_regression(&self) -> bool {
        self.chord_regression
    }
    /// Whether to reset the chord length on tick so that smaller chords can become
    /// active after releasing a larger chord.
    pub fn set_chord_regression(&mut self, chord_regression: bool) {
        self.chord_regression = chord_regression;
    }
    /// Whether to reset the chord length on tick so that smaller chords can become
    /// active after releasing a larger chord.
    pub fn with_chord_regression(mut self, chord_regression: bool) -> Self {
        self.chord_regression = chord_regression;
        self
    }
    /// Returns the current [`ClashStrategy`].
    pub fn clash_strategy(&self) -> ClashStrategy {
        self.strategy
    }
    /// Sets the [`ClashStrategy`].
    pub fn set_clash_strategy(&mut self, strategy: ClashStrategy) {
        self.strategy = strategy;
    }
    /// Sets the [`ClashStrategy`].
    pub fn with_clash_strategy(mut self, strategy: ClashStrategy) -> Self {
        self.strategy = strategy;
        self
    }
}

impl From<ClashStrategy> for ClashSettings {
    fn from(value: ClashStrategy) -> Self {
        Self {
            strategy: value,
            ..Default::default()
        }
    }
}

/// The settings to use for resolving clashing inputs.
#[derive(Clone, Copy, Debug, Default)]
pub enum ClashStrategy {
    /// Does not buffer inputs, just detects clashes. Inputs that may clash will be re-checked after all inputs
    /// have had a chance to assert their priority.
    ///
    /// # Rules
    ///
    /// - If a high priority binding captures a button, that button must be released before a lower priority
    ///   binding can see it again.
    ///
    #[default]
    Unbuffered,
    /// Buffers inputs that can clash until a timer runs out or unpressed.
    ///
    /// # Rules
    ///
    /// - An input will NEVER be active/shown the first frame it is pressed. Unlike Unbuffered re-checks
    ///   are not necessary because every binding will have a chance to be prioritized for chords.
    /// - If a wait duration is provided the input will be inactive/hidden to all bindings until the button has been
    ///   active for at least that long.
    /// - If the timer runs out or the button is unpressed: the chord with the most active parts will be activated.
    /// - If a high priority binding captures a button, that button must be released before a lower priority
    ///   binding can see it again.
    /// - If a chord has multiple buffered inputs, all inputs start times will be set the the oldest.
    BufferClashing(Option<Duration>),
    /// Buffers all inputs until a timer runs out or unpressed.
    ///
    /// # Rules
    ///
    /// - An input will NEVER be active/shown the first frame it is pressed. Unlike Unbuffered re-checks
    ///   are not necessary because every binding will have a chance to be prioritized for chords.
    /// - If a wait duration is provided the input will be inactive/hidden to all bindings until the button has been
    ///   active for at least that long.
    /// - If the timer runs out or the button is unpressed: the chord with the most active parts will be activated.
    /// - If a high priority binding captures a button, that button must be released before a lower priority
    ///   binding can see it again.
    /// - If a chord has multiple buffered inputs, all inputs start times will be set the the oldest.
    BufferAll(Option<Duration>),
    /// Disables Clash Detection. All presses will become active immediately.
    Disabled,
}

impl ClashStrategy {
    /// Return new settings that use buffered clash resolution where `delay` is the amount of time to wait before
    /// resolving; if `delay` is `None` input will buffer for 1 frame.
    pub fn new_buffered(delay: Option<Duration>) -> Self {
        Self::BufferClashing(delay)
    }
    /// Returns new settings that use un-buffered clash resolution where inputs that might clash re-check after all
    /// bindings have been checked at least once.
    pub fn new_unbuffered() -> Self {
        Self::Unbuffered
    }
    pub fn is_disabled(&self) -> bool {
        matches!(self, Self::Disabled)
    }
}

/// A Resource that defines default [`ComboSettings`] [`InputHandlers`](crate::manager::InputHandler) that
/// don't define their own.
#[derive(Resource, Clone, Copy, Debug)]
pub struct DefaultComboSettings(pub ComboSettings);

impl Deref for DefaultComboSettings {
    type Target = ComboSettings;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DefaultComboSettings {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Settings pertaining to [`ButtonCombos`](crate::button::ButtonCombo).
///
/// ## Interrupt
///
/// see [`ComboInterruptSettings`]
///
/// ## Progression
///
/// see [`ComboProgressionSettings`]
///
/// ## Tolerance
///
/// The maximum allowed time between button presses before the combo resets to expecting the first button in the combo.
#[derive(Debug, Component, Clone, Copy)]
pub struct ComboSettings {
    interrupt: ComboInterruptSettings,
    tolerance: Duration,
    progression: ComboProgressionSettings,
}

impl ComboSettings {
    pub fn interrupt_settings(&self) -> &ComboInterruptSettings {
        &self.interrupt
    }

    pub fn with_interrupt_settings(mut self, settings: ComboInterruptSettings) -> Self {
        self.interrupt = settings;
        self
    }

    pub fn set_interrupt_settings(&mut self, settings: ComboInterruptSettings) {
        self.interrupt = settings;
    }

    pub fn tolerance(&self) -> &Duration {
        &self.tolerance
    }

    pub fn with_tolerance(mut self, settings: Duration) -> Self {
        self.tolerance = settings;
        self
    }

    pub fn set_tolerance(&mut self, settings: Duration) {
        self.tolerance = settings;
    }

    pub fn progression_settings(&self) -> &ComboProgressionSettings {
        &self.progression
    }

    pub fn with_progression_settings(mut self, settings: ComboProgressionSettings) -> Self {
        self.progression = settings;
        self
    }

    pub fn set_progression_settings(&mut self, settings: ComboProgressionSettings) {
        self.progression = settings;
    }
}

impl Default for ComboSettings {
    fn default() -> Self {
        Self {
            interrupt: ComboInterruptSettings::default(),
            tolerance: Duration::from_millis(250),
            progression: ComboProgressionSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ComboInterruptSettings {
    /// Combos don't get cancelled by incorrect inputs.
    NoBreak,
    /// Combos will cancel if a button that isn't the expected button is pressed.
    #[default]
    ButtonsBreak,
    /// Any incorrect input (buttons or axis) will cancel a combo.
    AnythingBreaks,
}

/// Rules for how to determine if a [`ButtonCombo`](crate::button::ButtonCombo) can progress.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ComboProgressionSettings {
    None,
    PreviousMustBeReleased,
    #[default]
    NextMustBeReleased,
}
