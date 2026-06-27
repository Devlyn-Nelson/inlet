//! [`InputHandler`] related types.
use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
    time::{Duration, Instant},
};

use bevy::{
    ecs::{component::Component, resource::Resource},
    input::{
        ButtonInput,
        gamepad::Gamepad,
        keyboard::KeyCode,
        mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseButton},
    },
    platform::collections::{HashMap, hash_map::Entry},
};

use crate::{BevyAxisKind, BevyButtonKind, BevyInputKind, InputBinding, InputValue};

/// Current state of an input.
#[derive(Debug, Default)]
enum InputStateKind {
    /// Exactly 1 binding has been made to this input. Clash checks can be ignored.
    #[default]
    NoClash,
    /// State is currently inactive.
    Inactive,
    /// At least 1 input wants to
    Clashing(usize),
    /// Input is being buffered and is being reported as inactive, shall become released with
    /// the same `usize` for at least 1 frame.
    Buffered {
        start: Instant,
        coord_len: usize,
        last_coord_len: usize,
    },
    /// State is currently active if you meet the priority stored.
    Active {
        coord_len: usize,
        last_coord_len: usize,
    },
}

impl InputStateKind {
    fn inactive() -> Self {
        Self::Inactive
    }
    fn clashing(len: usize) -> Self {
        Self::Clashing(len)
    }
    fn buffered(len: usize) -> Self {
        Self::Buffered {
            start: Instant::now(),
            coord_len: len,
            last_coord_len: len,
        }
    }
    // fn buffered_with_instant(len: usize, i: Instant) -> Self {
    //     Self::Buffered {
    //         start: i,
    //         coord_len: len,
    //         last_coord_len: len,
    //     }
    // }
    fn active(len: usize) -> Self {
        Self::Active {
            coord_len: len,
            last_coord_len: len,
        }
    }
    fn replace(&mut self, new: Self) {
        *self = new;
    }
}

impl Display for InputStateKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputStateKind::NoClash => write!(f, "NoClash"),
            InputStateKind::Inactive => write!(f, "Inactive"),
            InputStateKind::Clashing(len) => write!(f, "Clashing({len})"),
            InputStateKind::Buffered { coord_len, .. } => write!(f, "Buffered({coord_len})"),
            InputStateKind::Active { coord_len, .. } => write!(f, "Active({coord_len})"),
        }
    }
}

#[derive(Debug, Default)]
struct InputState {
    /// The last frame this was updated by a poll call.
    frame: usize,
    /// The actual state.
    kind: InputStateKind,
    /// The last input feed into the state.
    value: InputValue,
}

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
/// # Component
///
/// If inserted on an entity that has a InputHandler, the InputHandler will use new settings and remove the
///
/// # Resource
///
/// Defines a default settings that new [`InputHandler`] can pull from.
///
#[derive(Component, Clone, Copy, Debug)]
/// component to avoid extra checks.
#[derive(Default)]
pub enum ClashSettings {
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
}

impl ClashSettings {
    /// Return new settings that use buffered clash resolution where `delay` is the amount of time to wait before
    /// resolving; if `delay` is `None` input will buffer for 1 frame.
    pub fn new_buffered(delay: Option<Duration>) -> Self {
        Self::BufferClashing(delay)
    }
    /// Returns new settings that use unbuffered clash resolution where inputs that might clash re-check after all
    /// bindings have been checked at least once.
    pub fn new_unbuffered() -> Self {
        Self::Unbuffered
    }
    fn buffer_all(&self) -> bool {
        matches!(self, Self::BufferAll(_))
    }
}

/// Management of a players bindings and the states.
#[derive(Component)]
pub struct InputHandler {
    /// a counter that is increased when ever `Self::tick` is called.
    frame: usize,
    /// All known bindings and the state of the input.
    clashables: HashMap<BevyInputKind, InputState>,
    /// The settings used for the resolution of clashing bindings.
    settings: ClashSettings,
    /// reset the coord length on tick so that smaller coords can become active
    /// after releasing a larger coord.
    coord_regretion: bool,
}

impl From<ClashSettings> for InputHandler {
    fn from(value: ClashSettings) -> Self {
        Self {
            frame: 0,
            clashables: HashMap::default(),
            settings: value,
            coord_regretion: false,
        }
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::from(ClashSettings::default())
    }
}

#[derive(PartialEq, Eq)]
enum Outy {
    Hide,
    Show,
    Repoll,
}

impl InputHandler {
    /// The settings used for clash handling.
    pub fn settings(&self) -> &ClashSettings {
        &self.settings
    }
    /// Please update_list after using this, because some input may be in a state that will not
    /// allow the input to enter a state that is correct for the new settings.
    pub fn set_settings(&mut self, new: ClashSettings) {
        self.settings = new;
    }
    /// Whether to reset the coord length on tick so that smaller coords can become
    /// active after releasing a larger coord.
    pub fn coord_regretion(&self) -> bool {
        self.coord_regretion
    }
    /// Whether to reset the coord length on tick so that smaller coords can become
    /// active after releasing a larger coord.
    pub fn set_coord_regretion(&mut self, coord_regretion: bool) {
        self.coord_regretion = coord_regretion;
    }
    /// Does some internal cleaning that is only possible between bindings checking for their inputs
    /// because we can assume that all (or none) of the inputs have been given a change to fight for priority.
    ///
    /// - if the input state has a frame not equal to the current frame: change to inactive.
    /// - else if the input state is buffered and the timer has expired: change to active.
    /// - else if the input state is clashing : change to active.
    /// - increases the internal counter for "frames" after all above steps.
    ///
    pub fn tick(&mut self) {
        let cr = self.coord_regretion();
        for (_c, state) in self.clashables.iter_mut() {
            let new = if state.frame != self.frame {
                if matches!(
                    state.kind,
                    InputStateKind::Inactive | InputStateKind::NoClash
                ) {
                    None
                } else {
                    Some(InputStateKind::inactive())
                }
            } else if let ClashSettings::BufferClashing(duration)
            | ClashSettings::BufferAll(duration) = &self.settings
                && let InputStateKind::Buffered {
                    start,
                    coord_len,
                    last_coord_len,
                } = &state.kind
            {
                if let Some(d) = duration {
                    if start.elapsed() >= *d {
                        Some(InputStateKind::Active {
                            coord_len: *coord_len,
                            last_coord_len: *last_coord_len,
                        })
                    } else {
                        None
                    }
                } else {
                    Some(InputStateKind::Active {
                        coord_len: *coord_len,
                        last_coord_len: *last_coord_len,
                    })
                }
            } else if let InputStateKind::Clashing(priority) = &state.kind {
                Some(InputStateKind::active(*priority))
            } else {
                None
            };
            match &mut state.kind {
                InputStateKind::Buffered {
                    coord_len,
                    last_coord_len,
                    ..
                }
                | InputStateKind::Active {
                    coord_len,
                    last_coord_len,
                } => {
                    *last_coord_len = *coord_len;
                    if cr {
                        *coord_len = 0;
                    }
                }
                _ => {}
            }
            if let Some(new) = new {
                state.kind.replace(new);
            }
        }
        self.frame += 1;
    }
    /// Updates the internal binding map and resets all states.
    pub fn update_list<K, T>(&mut self, map: &HashMap<K, InputBinding<T>>) {
        let clashables: Vec<BevyInputKind> =
            map.values().flat_map(|asdf| asdf.input_kinds()).collect();
        // TODO need to provide a way to clean up unused inputs.
        // self.clashables.clear();
        for c in clashables.into_iter() {
            match self.clashables.entry(c) {
                Entry::Occupied(mut o) => {
                    let state = o.get_mut();
                    if matches!(state.kind, InputStateKind::NoClash) {
                        state.kind = InputStateKind::Inactive;
                    }
                }
                Entry::Vacant(v) => {
                    v.insert_entry(InputState {
                        frame: self.frame,
                        kind: InputStateKind::default(),
                        value: InputValue::default(),
                    });
                }
            }
        }
    }
    /// Tries to return the newest value associated with the binding.
    ///
    /// If `None` is returned then you must [`Self::repoll`] after all inputs have been polled
    pub(crate) fn poll(&mut self, clashable: &[BevyInputKind]) -> Option<InputValue> {
        if clashable.is_empty() {
            return Some(InputValue::default());
        }
        // Are all inputs pressed
        let mut pressed = true;
        // the buffered input with the oldest instant.
        let mut oldest_press = Ok(Instant::now());
        for c in clashable.iter() {
            match self.clashables.entry(*c) {
                Entry::Occupied(o) => {
                    if o.get().value.is_pressed() {
                        if let InputStateKind::Buffered { start, .. } = &o.get().kind {
                            match oldest_press {
                                Err(oldest) | Ok(oldest) => {
                                    if oldest > *start {
                                        oldest_press = Err(*start);
                                    }
                                }
                            }
                        }
                    } else {
                        pressed = false;
                    }
                }
                Entry::Vacant(v) => {
                    bevy::log::warn!("polled unregistered bevy input in manager. ({c:?})");
                    v.insert(InputState {
                        frame: self.frame,
                        kind: InputStateKind::inactive(),
                        value: InputValue::default(),
                    });
                }
            }
        }
        let oldest_press = oldest_press.err();
        let mut repoll = if pressed { Outy::Show } else { Outy::Hide };
        let chord_length = clashable.len();
        for c in clashable.iter() {
            // UNWRAP the first for loop pass should insure that all clashables are in the map.
            let state = self.clashables.get_mut(c).unwrap();
            let new_state = if pressed {
                match &mut state.kind {
                    InputStateKind::NoClash => {
                        if self.settings.buffer_all() {
                            Some(InputStateKind::buffered(chord_length))
                        } else {
                            None
                        }
                    }
                    InputStateKind::Inactive => match self.settings {
                        ClashSettings::Unbuffered => Some(InputStateKind::clashing(chord_length)),
                        ClashSettings::BufferAll(_) | ClashSettings::BufferClashing(_) => {
                            Some(InputStateKind::buffered(chord_length))
                        }
                    },
                    InputStateKind::Clashing(len) => {
                        if chord_length > *len {
                            Some(InputStateKind::clashing(chord_length))
                        } else {
                            None
                        }
                    }
                    InputStateKind::Buffered {
                        start,
                        coord_len,
                        last_coord_len,
                    } => {
                        if let Some(oldest) = oldest_press
                            && oldest < *start
                        {
                            *start = oldest;
                        }
                        if chord_length > *coord_len {
                            Some(InputStateKind::Buffered {
                                start: *start,
                                coord_len: chord_length,
                                last_coord_len: *last_coord_len,
                            })
                        } else {
                            None
                        }
                    }
                    InputStateKind::Active {
                        coord_len,
                        last_coord_len,
                    } => {
                        if chord_length > *coord_len {
                            Some(InputStateKind::Active {
                                coord_len: chord_length,
                                last_coord_len: *last_coord_len,
                            })
                        } else {
                            None
                        }
                    }
                }
            } else {
                None
            };
            if let Some(new) = new_state {
                state.kind.replace(new);
            }
            if pressed && state.frame != self.frame {
                state.frame = self.frame;
            }
            match &state.kind {
                InputStateKind::Clashing(_) => {
                    if matches!(repoll, Outy::Show) {
                        repoll = Outy::Repoll;
                    }
                }
                InputStateKind::Buffered { .. } => {
                    if matches!(repoll, Outy::Show | Outy::Repoll) {
                        repoll = Outy::Hide;
                    }
                }
                InputStateKind::Active {
                    last_coord_len: coord_len,
                    ..
                } => {
                    bevy::log::info!("{c:?} = {coord_len}");
                    if *coord_len != chord_length && matches!(repoll, Outy::Show | Outy::Repoll) {
                        repoll = Outy::Hide;
                    }
                }
                InputStateKind::NoClash | InputStateKind::Inactive => {}
            }
        }

        match repoll {
            Outy::Hide => Some(InputValue::default()),
            Outy::Show => {
                // UNWRAP the first for loop pass should insure that all clashables are in the map.
                let val = self
                    .clashables
                    .get(&clashable[0])
                    .map(|asdf| asdf.value.clone())
                    .unwrap_or_default();
                Some(val)
            }
            Outy::Repoll => None,
        }
    }
    /// Only preforms a check of what input to use, does not preform any state changing.
    ///
    /// It is expected that this is only ever called on inputs that got a `None` from [`Self::poll`].
    pub(crate) fn repoll(&self, clashable: &[BevyInputKind]) -> InputValue {
        if clashable.is_empty() {
            return InputValue::default();
        }
        for c in clashable.iter() {
            if let Some(state) = self.clashables.get(c) {
                match &state.kind {
                    InputStateKind::Inactive | InputStateKind::Buffered { .. } => {
                        return InputValue::default();
                    }
                    InputStateKind::NoClash => {}
                    InputStateKind::Clashing(coord_len)
                    | InputStateKind::Active { coord_len, .. } => {
                        if clashable.len() != *coord_len {
                            return InputValue::default();
                        }
                    }
                }
            }
        }
        self.clashables
            .get(&clashable[0])
            .map(|asdf| asdf.value.clone())
            .unwrap_or_default()
    }
    /// Updates values for input types from `bevy_input`.
    pub(crate) fn update(
        &mut self,
        gamepads: &[&Gamepad],
        keycodes: &ButtonInput<KeyCode>,
        // keys: &ButtonInput<Key>,
        mouse: &ButtonInput<MouseButton>,
        accumulated_mouse_motion: &AccumulatedMouseMotion,
        accumulated_mouse_scroll: &AccumulatedMouseScroll,
    ) {
        for (kind, state) in self.clashables.iter_mut() {
            let new_value = match kind {
                BevyInputKind::Axis(bevy_axis_kind) => match bevy_axis_kind {
                    BevyAxisKind::MouseAxis(mouse_axis) => InputValue::Value(match mouse_axis {
                        crate::axis::MouseAxis::MotionX => accumulated_mouse_motion.delta.x,
                        crate::axis::MouseAxis::MotionY => accumulated_mouse_motion.delta.y,
                        crate::axis::MouseAxis::ScrollX => accumulated_mouse_scroll.delta.x,
                        crate::axis::MouseAxis::ScrollY => accumulated_mouse_scroll.delta.y,
                    }),
                    BevyAxisKind::GamepadAxis(gamepad_axis) => {
                        let mut value = 0.;
                        let mut count = 0;
                        for gpad in gamepads {
                            if let Some(v) = gpad.get(*gamepad_axis)
                                && v != 0.
                            {
                                value += v;
                                count += 1;
                            }
                        }
                        InputValue::Value(if count == 0 {
                            0.
                        } else {
                            value / (count as f32)
                        })
                    }
                    BevyAxisKind::GamepadButton(gamepad_button) => {
                        let mut value = 0.;
                        let mut count = 0;
                        for gpad in gamepads {
                            if let Some(v) = gpad.get(*gamepad_button)
                                && v != 0.
                            {
                                value += v;
                                count += 1;
                            }
                        }
                        InputValue::Value(if count == 0 {
                            0.
                        } else {
                            value / (count as f32)
                        })
                    }
                },
                BevyInputKind::Button(bevy_button_kind) => match bevy_button_kind {
                    BevyButtonKind::GamepadButton(gamepad_button) => {
                        let mut out = false;
                        for gpad in gamepads {
                            if gpad.pressed(*gamepad_button) {
                                out |= true;
                                break;
                            }
                        }
                        InputValue::Pressed(out)
                    }
                    BevyButtonKind::KeyCode(key_code) => {
                        InputValue::Pressed(keycodes.pressed(*key_code))
                    }
                    BevyButtonKind::MouseButton(mouse_button) => {
                        InputValue::Pressed(mouse.pressed(*mouse_button))
                    }
                },
            };
            state.value = new_value;
        }
    }
}
