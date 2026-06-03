use std::{
    ops::{Add, Sub},
    time::{Duration, Instant},
};

use bevy::{
    ecs::component::Component,
    input::{
        ButtonInput,
        gamepad::{Gamepad, GamepadAxis, GamepadButton},
        keyboard::KeyCode,
        mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseButton},
    },
    platform::collections::{HashMap, hash_map::Entry},
};

use crate::{
    InputBinding, axis::MouseAxis,  pressed_to_value, value_to_press,
};

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

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum BevyInputKind {
    Axis(BevyAxisKind),
    Button(BevyButtonKind),
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

pub struct Clash {}

#[derive(Debug, Default)]
enum InputStateKind {
    #[default]
    Unclashable,
    Clashable,
    Clashing(usize),
    Buffered(Instant, usize),
    Released(usize),
}

impl InputStateKind {
    fn repoll(&self) -> bool {
        match self {
            InputStateKind::Unclashable | InputStateKind::Clashable => false,
            InputStateKind::Clashing(_) => true,
            InputStateKind::Buffered(_, _) => false,
            InputStateKind::Released(_) => false,
        }
    }
    fn is_clashable(&self) -> bool {
        matches!(self, Self::Clashable)
    }
    fn clashable() -> Self {
        Self::Clashable
    }
    fn clashing(len: usize) -> Self {
        Self::Clashing(len)
    }
    fn buffered(len: usize) -> Self {
        Self::Buffered(Instant::now(), len)
    }
    fn buffered_with_instant(len: usize, i: Instant) -> Self {
        Self::Buffered(i, len)
    }
    fn released(len: usize) -> Self {
        Self::Released(len)
    }
    fn replace(&mut self, new: Self) {
        *self = new;
    }
}

#[derive(Debug, Clone)]
pub enum InputValue {
    Pressed(bool),
    Value(f32),
}

impl Default for InputValue {
    fn default() -> Self {
        Self::Pressed(false)
    }
}

impl InputValue {
    pub fn is_pressed(&self) -> bool {
        match self {
            InputValue::Pressed(p) => *p,
            InputValue::Value(val) => value_to_press(*val),
        }
    }
    pub fn get_value(&self) -> f32 {
        match self {
            InputValue::Pressed(p) => pressed_to_value(*p),
            InputValue::Value(val) => *val,
        }
    }
}

impl Add for InputValue {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            InputValue::Pressed(pressed) => InputValue::Pressed(
                match rhs {
                    InputValue::Pressed(o) => o,
                    InputValue::Value(v) => value_to_press(v),
                } | pressed,
            ),
            InputValue::Value(val) => InputValue::Value(
                match rhs {
                    InputValue::Pressed(pressed) => pressed_to_value(pressed),
                    InputValue::Value(v) => v,
                } + val,
            ),
        }
    }
}

impl Sub for InputValue {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match self {
            InputValue::Pressed(pressed) => InputValue::Pressed(
                if match rhs {
                    InputValue::Pressed(o) => o,
                    InputValue::Value(v) => value_to_press(v),
                } {
                    false
                } else {
                    pressed
                },
            ),
            InputValue::Value(val) => InputValue::Value(
                val - match rhs {
                    InputValue::Pressed(pressed) => pressed_to_value(pressed),
                    InputValue::Value(v) => v,
                },
            ),
        }
    }
}

struct InputState {
    /// The last frame this was updated.
    frame: usize,
    kind: InputStateKind,
    value: InputValue,
}

pub enum ClashSettings {
    /// This mode is a work in progress. Don't work correctly, you should use [`Self::Buffered`] until its fixed.
    ///
    /// Does not buffer inputs, just detects clashes.
    ///
    /// # Rules
    ///
    /// - If a high priority binding captures a button, that button must be released before a lower priority
    ///   binding can see it again.
    ///
    /// # Warning
    ///
    /// The sorting required for this to work is not done. So lower priority inputs may still get activated,
    /// when they shouldn't, for a single frame.
    Unbuffered,
    /// Buffers inputs that can clash until a timer runs out or unpressed.
    ///
    /// # Rules
    ///
    /// - An input will NEVER be active the first frame it is pressed. Because of this we don't need to sort
    /// the order input bindings should be checked like [`Self::Sorted`].
    /// - If a wait duration is provided the input will not be released to anyone until the button has been
    ///   active for that long.
    /// - If the timer runs out or the button is unpressed the binding will be released to all inputs with
    ///   the maximum chord length polled during the Press but inactive period. If the button was unpressed
    ///   to release the input it will stay active for 1 frame before going inactive.
    /// - If a high priority binding captures a button, that button must be released before a lower priority
    ///   binding can see it again.
    Buffered(Option<Duration>),
}

impl ClashSettings {
    pub fn needs_sorting(&self) -> bool {
        matches!(self, Self::Unbuffered)
    }
}

// pub(crate) struct BindSort<K> {
//     pub(crate) order: Vec<BindLookup<K>>
// }

// impl<K> BindSort<K> {
//     pub(crate) fn new() -> Self {
//         Self { order: Vec::default() }
//     }
//     pub(crate) fn update_list<K, T>(&mut self, map: &HashMap<K, InputBinding<T>>)where K:Clone {
//         let new = Vec::default();
//         for (key, value) in map {
//             match &value {
//                 InputBinding::Action(action_binding) => {
//                 }
//                 InputBinding::Value(value_binding) => todo!(),
//                 InputBinding::DualValue(dual_value_binding) => todo!(),
//             }
//         }
//     }
// }

// fn get_button_binding_lookups<T>(action_binding: ActionBinding<T>) {
//     for (index, binding) in action_binding.bindings().iter().enumerate() {

//     }
// }

// fn get_axis_binding_lookups<T>(axis_binding: ValueBinding<T>) {
//     for (index, binding) in axis_binding.bindings().iter().enumerate() {

//     }
// }

// pub(crate) struct BindLookup<K> {
//     chord_len: usize,
//     /// Key in hte InputBindings
//     key: K,
//     /// Index of the binding within the InputBinding.
//     index: usize,
// }

// impl<K> PartialEq for BindLookup<K> {
//     fn eq(&self, other: &Self) -> bool {
//         self.chord_len == other.chord_len
//     }
// }

// impl<K> Eq for BindLookup<K> {}

// impl<K> PartialOrd for BindLookup<K> {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         Some(self.cmp(other))
//     }
// }

// impl<K> Ord for BindLookup<K> {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         other.chord_len.cmp(&self.chord_len)
//     }
// }

#[derive(Component)]
pub struct InputHandler {
    frame: usize,
    clashables: HashMap<BevyInputKind, InputState>,
    settings: ClashSettings,
    /// Rescan bindings for clashes.
    should_rescan: bool,
}

impl Default for InputHandler {
    fn default() -> Self {
        Self {
            frame: 0,
            clashables: HashMap::default(),
            settings: ClashSettings::Buffered(Some(Duration::from_millis(10))),
            should_rescan: true,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Outy {
    Start,
    Value,
    Repoll,
}

impl InputHandler {
    pub fn should_rescan(&self) -> bool {
        self.should_rescan
    }
    pub fn signal_rescan(&mut self) {
        self.should_rescan = true;
    }
    pub fn settings(&self) -> &ClashSettings {
        &self.settings
    }
    pub fn tick(&mut self) {
        for state in self.clashables.values_mut() {
            let new = if state.frame != self.frame {
                if state.kind.is_clashable() {
                    None
                } else {
                    Some(InputStateKind::clashable())
                }
            } else if let ClashSettings::Buffered(duration) = &self.settings
                && let InputStateKind::Buffered(start, len) = &state.kind
            {
                if let Some(d) = duration {
                    if start.elapsed() >= *d {
                        Some(InputStateKind::released(*len))
                    } else {
                        None
                    }
                } else {
                    Some(InputStateKind::released(*len))
                }
            } else {
                None
            };
            if let Some(new) = new {
                state.kind.replace(new);
            }
        }
        self.frame += 1;
    }
    pub fn update_list<K, T>(&mut self, map: &HashMap<K, InputBinding<T>>) {
        let clashables: Vec<BevyInputKind> =
            map.values().flat_map(|asdf| asdf.input_kinds()).collect();
        self.clashables.clear();
        for c in clashables.clone().into_iter() {
            match self.clashables.entry(c) {
                Entry::Occupied(mut o) => {
                    let state = o.get_mut();
                    if matches!(state.kind, InputStateKind::Unclashable) {
                        state.kind = InputStateKind::Clashable;
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
        if clashable.len() == 0 {
            return Some(InputValue::default());
        }
        let mut count = 1;
        for c in clashable.iter() {
            if let Some(state) = self.clashables.get(c)
                && state.value.is_pressed()
            {
                count += 1;
            }
        }
        if count != clashable.len() {
            return Some(InputValue::default());
        }
        let mut repoll = Outy::Start;
        let mut out = InputValue::default();
        let chord_length = clashable.len();
        for c in clashable.iter() {
            if let Some(state) = self.clashables.get_mut(c) {
                let new_state = if state.value.is_pressed() {
                    match &state.kind {
                        InputStateKind::Unclashable => None,
                        InputStateKind::Clashable => match self.settings {
                            ClashSettings::Unbuffered => {
                                Some(InputStateKind::clashing(chord_length))
                            }
                            ClashSettings::Buffered(_) => {
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
                        InputStateKind::Buffered(instant, len) => {
                            if chord_length > *len {
                                Some(InputStateKind::buffered_with_instant(
                                    chord_length,
                                    *instant,
                                ))
                            } else {
                                None
                            }
                        }
                        InputStateKind::Released(len) => {
                            if chord_length > *len {
                                Some(InputStateKind::released(chord_length))
                            } else {
                                None
                            }
                        }
                    }
                } else {
                    match &state.kind {
                        InputStateKind::Released(_)
                        | InputStateKind::Clashable
                        | InputStateKind::Unclashable => {
                            // not pressed or clashing.
                            None
                        }
                        InputStateKind::Clashing(_) => Some(InputStateKind::clashable()),
                        InputStateKind::Buffered(_, len) => Some(InputStateKind::released(*len)),
                    }
                };
                if let Some(new) = new_state {
                    state.kind.replace(new);
                }
                if state.value.is_pressed() && state.frame != self.frame {
                    state.frame = self.frame;
                }
                match repoll {
                    Outy::Start => {
                        repoll = Outy::Value;
                        out = state.value.clone();
                    }
                    Outy::Value => {}
                    Outy::Repoll => {}
                }
                match &state.kind {
                    InputStateKind::Clashing(_) => {
                        repoll = Outy::Repoll;
                    }
                    InputStateKind::Buffered(_, _) => {
                        out = InputValue::default();
                    }
                    InputStateKind::Unclashable => todo!(),
                    InputStateKind::Clashable => todo!(),
                    InputStateKind::Released(len) => if chord_length < *len {
                        out = InputValue::default();
                    },
                }
            }
        }

        match repoll {
            Outy::Start | Outy::Value => Some(out),
            Outy::Repoll => None,
        }
    }
    /// Only preforms a check of what input to use, does not preform any state changing.
    ///
    /// It is expected that this is only ever called on inputs that got a `None` from [`Self::poll`].
    pub(crate) fn repoll(&self, clashable: &[BevyInputKind]) -> InputValue {
        let mut first = true;
        let mut out = InputValue::default();
        for c in clashable.iter() {
            if let Some(state) = self.clashables.get(c) {
                if first {
                    first = false;
                    out = state.value.clone();
                }
                match &state.kind {
                    InputStateKind::Buffered(_, _) => {
                        return InputValue::default();
                    }
                    InputStateKind::Unclashable |
                    InputStateKind::Clashable => {}
                    InputStateKind::Clashing(len) |
                    InputStateKind::Released(len) => if clashable.len() < *len {
                        return InputValue::default();
                    },
                }
            }
        }
        out
    }
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
                BevyInputKind::Axis(bevy_axis_kind) => {
                    match bevy_axis_kind {
                        BevyAxisKind::MouseAxis(mouse_axis) => {
                            InputValue::Value(match mouse_axis {
                                crate::axis::MouseAxis::MotionX => accumulated_mouse_motion.delta.x,
                                crate::axis::MouseAxis::MotionY => accumulated_mouse_motion.delta.y,
                                crate::axis::MouseAxis::ScrollX => accumulated_mouse_scroll.delta.x,
                                crate::axis::MouseAxis::ScrollY => accumulated_mouse_scroll.delta.y,
                            })
                        }
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
                        } // BevyInputKind::Key(key) => InputValue::Pressed(keys.pressed(*key)),
                    }
                }
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

pub enum PollResult {
    RePoll,
    Use(InputValue),
    None,
}
