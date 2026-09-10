//! [`InputHandler`] related types.
use std::{fmt::Display, marker::PhantomData, time::Instant};

use bevy::{
    ecs::component::Component,
    input::{
        ButtonInput,
        gamepad::Gamepad,
        keyboard::KeyCode,
        mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseButton},
    },
    platform::collections::{HashMap, hash_map::Entry},
};

use crate::{
    BevyAxisKind, BevyButtonKind, BevyInputKind, ClashSettings, ClashStrategy, InputBinding,
    InputValue,
};

/// Current state of an input.
#[derive(Debug, Default)]
enum InputStateKind {
    /// State is currently inactive.
    #[default]
    Inactive,
    /// At least 1 input wants to
    Clashing(usize),
    /// Input is being buffered and is being reported as inactive, shall become released with
    /// the same `usize` for at least 1 frame.
    Buffered {
        /// When the buffered state was entered.
        start: Instant,
        /// Current largest chord trying to access the binding.
        chord_len: usize,
        /// Largest chord trying to access the binding in the previous frame.
        last_chord_len: usize,
        /// Amount of inputs trying to access this binding.
        count: usize,
    },
    /// State is currently active if you meet the priority stored.
    Active {
        chord_len: usize,
        last_chord_len: usize,
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
            chord_len: len,
            last_chord_len: len,
            count: 1,
        }
    }
    fn active(len: usize) -> Self {
        Self::Active {
            chord_len: len,
            last_chord_len: len,
        }
    }
    fn replace(&mut self, new: Self) {
        *self = new;
    }
}

impl Display for InputStateKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputStateKind::Inactive => write!(f, "Inactive"),
            InputStateKind::Clashing(len) => write!(f, "Clashing({len})"),
            InputStateKind::Buffered { chord_len, .. } => write!(f, "Buffered({chord_len})"),
            InputStateKind::Active { chord_len, .. } => write!(f, "Active({chord_len})"),
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

/// Management of a players bindings and the states.
#[derive(Component)]
pub struct InputHandler {
    /// a counter that is increased when ever `Self::tick` is called.
    frame: usize,
    /// All known bindings and the state of the input.
    clashables: HashMap<BevyInputKind, InputState>,
}

impl Default for InputHandler {
    fn default() -> Self {
        Self {
            frame: 0,
            clashables: HashMap::default(),
        }
    }
}

/// Disables a InputManger with types `T` and `K`.
#[derive(Debug, Default, Component)]
pub struct DisableInputManager<K, T>(PhantomData<T>, PhantomData<K>);

#[derive(PartialEq, Eq)]
enum Outy {
    Hide,
    Show,
    Repoll,
}

impl InputHandler {
    /// Does some internal cleaning that is only possible between bindings checking for their inputs
    /// because we can assume that all (or none) of the inputs have been given a change to fight for priority.
    ///
    /// - if the input state has a frame not equal to the current frame: change to inactive.
    /// - else if the input state is buffered and the timer has expired: change to active.
    /// - else if the input state is clashing : change to active.
    /// - increases the internal counter for "frames" after all above steps.
    ///
    pub fn tick(&mut self, clash_settings: &ClashSettings) {
        let cr = clash_settings.chord_regretion();
        for (_c, state) in self.clashables.iter_mut() {
            let new = if state.frame != self.frame {
                if matches!(state.kind, InputStateKind::Inactive) {
                    None
                } else {
                    Some(InputStateKind::inactive())
                }
            } else if let ClashStrategy::BufferClashing(duration)
            | ClashStrategy::BufferAll(duration) = clash_settings.clash_strategy()
                && let InputStateKind::Buffered {
                    start,
                    chord_len,
                    last_chord_len,
                    count,
                } = &mut state.kind
            {
                if let Some(d) = duration {
                    if start.elapsed() >= d
                        || (matches!(
                            clash_settings.clash_strategy(),
                            ClashStrategy::BufferClashing(_)
                        ) && *count == 1)
                    {
                        Some(InputStateKind::Active {
                            chord_len: *chord_len,
                            last_chord_len: *last_chord_len,
                        })
                    } else {
                        *count = 0;
                        None
                    }
                } else {
                    Some(InputStateKind::Active {
                        chord_len: *chord_len,
                        last_chord_len: *last_chord_len,
                    })
                }
            } else if let InputStateKind::Clashing(priority) = &state.kind {
                Some(InputStateKind::active(*priority))
            } else {
                None
            };
            match &mut state.kind {
                InputStateKind::Buffered {
                    chord_len,
                    last_chord_len,
                    ..
                }
                | InputStateKind::Active {
                    chord_len,
                    last_chord_len,
                } => {
                    *last_chord_len = *chord_len;
                    if cr {
                        *chord_len = 0;
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
            if let Entry::Vacant(v) = self.clashables.entry(c) {
                v.insert_entry(InputState {
                    frame: self.frame,
                    kind: InputStateKind::default(),
                    value: InputValue::default(),
                });
            }
        }
    }
    /// Used to determine if a combo is broken. Returns `true` if a input that is not in `clashables` is updated this
    /// frame.
    pub(crate) fn poll_interupt(
        &mut self,
        clashable: &[BevyInputKind],
        axis_interupts: bool,
    ) -> bool {
        for (binding, state) in self.clashables.iter() {
            if !clashable.contains(binding) {
                if state.frame == self.frame {
                    if axis_interupts || state.value.is_button() {
                        return true;
                    }
                }
            }
        }
        false
    }
    /// Tries to return the newest value associated with the binding.
    ///
    /// If `None` is returned then you must [`Self::repoll`] after all inputs have been polled
    pub(crate) fn poll(
        &mut self,
        clashable: &[BevyInputKind],
        clash_settings: &ClashSettings,
    ) -> Option<InputValue> {
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
                    #[cfg(feature = "inlet_log")]
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
                    InputStateKind::Inactive => match clash_settings.clash_strategy() {
                        ClashStrategy::Unbuffered => Some(InputStateKind::clashing(chord_length)),
                        ClashStrategy::BufferAll(_) | ClashStrategy::BufferClashing(_) => {
                            Some(InputStateKind::buffered(chord_length))
                        }
                        ClashStrategy::Disabled => Some(InputStateKind::active(chord_length)),
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
                        chord_len,
                        last_chord_len: _,
                        count,
                    } => {
                        *count += 1;
                        if let Some(oldest) = oldest_press
                            && oldest < *start
                        {
                            *start = oldest;
                        }
                        if chord_length > *chord_len {
                            *chord_len = chord_length;
                        }
                        None
                    }
                    InputStateKind::Active {
                        chord_len,
                        last_chord_len,
                    } => {
                        if chord_length > *chord_len {
                            Some(InputStateKind::Active {
                                chord_len: chord_length,
                                last_chord_len: *last_chord_len,
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
                    last_chord_len: chord_len,
                    ..
                } => {
                    if !clash_settings.clash_strategy().is_disabled() && *chord_len != chord_length
                    {
                        repoll = Outy::Hide;
                    }
                }
                InputStateKind::Inactive => {}
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
                    InputStateKind::Clashing(chord_len)
                    | InputStateKind::Active { chord_len, .. } => {
                        if clashable.len() != *chord_len {
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
