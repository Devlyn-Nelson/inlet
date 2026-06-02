use std::time::{Duration, Instant};

use bevy::{
    ecs::component::Component,
    input::{
        gamepad::{GamepadAxis, GamepadButton},
        keyboard::KeyCode,
        mouse::MouseButton,
    },
    platform::collections::{HashMap, hash_map::Entry},
};

use crate::{InputBinding, axis::MouseAxis};

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum ClashableKind {
    MouseAxis(MouseAxis),
    GamepadAxis(GamepadAxis),
    GamepadButton(GamepadButton),
    Keyboard(KeyCode),
    MouseButton(MouseButton),
    Gamepad(GamepadButton),
}

pub struct Clash {}

#[derive(Debug)]
enum ClashStateKind {
    None,
    Clashing(usize),
    Buffered(Instant, usize),
    Released(usize),
}

impl ClashStateKind {
    fn allowed_to_take_input(&self, chord_len: usize) -> bool {
        match self {
            ClashStateKind::None => true,
            ClashStateKind::Clashing(len) => chord_len == *len,
            ClashStateKind::Buffered(_, _) => false,
            ClashStateKind::Released(len) => chord_len == *len,
        }
    }
    fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
    fn none() -> Self {
        // bevy::log::info!("clash::None");
        Self::None
    }
    fn clashing(len: usize) -> Self {
        // bevy::log::info!("clash::Clashing({len})");
        Self::Clashing(len)
    }
    fn buffered(len: usize) -> Self {
        // bevy::log::info!("clash::Buffered(new,{len})");
        Self::Buffered(Instant::now(), len)
    }
    fn buffered_with_instant(len: usize, i: Instant) -> Self {
        // bevy::log::info!("clash::Buffered(old,{len})");
        Self::Buffered(i, len)
    }
    fn released(len: usize) -> Self {
        // bevy::log::info!("clash::Released({len})");
        Self::Released(len)
    }
    fn replace(&mut self, new: Self) {
        bevy::log::info!("{self:?} -> {new:?}");
        *self = new;
    }
}

struct ClashState {
    /// The last frame this was updated.
    frame: usize,
    kind: ClashStateKind,
}

pub enum ClashSettings {
    /// The highest-priority or longest chord will capture the input. This means any shorter chord can get
    /// the input during the previous frame if not all buttons were pressed in that frame. This will NOT
    /// buffer inputs meaning chords happen as they are pressed, but inputs with shorter chord lengths
    /// will be told to ignore.
    Sorted,
    /// Waits for the highest-priority or longest chord to capture the input within a Duration. This will
    /// always skip the "JustPressed" tick because it wants to wait for the longest input. If the Duration
    /// from the initial press of the clash passes or the input is released: inputs with a chord length that
    /// matches the longest chord length of inputs that tried to gather the input.
    Buffered(Option<Duration>),
}

impl ClashSettings {
    pub fn needs_sorting(&self) -> bool {
        matches!(self, Self::Sorted)
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
pub struct ClashHandler {
    frame: usize,
    clashables: HashMap<ClashableKind, ClashState>,
    settings: ClashSettings,
    /// Rescan bindings for clashes.
    should_rescan: bool,
}

impl Default for ClashHandler {
    fn default() -> Self {
        Self {
            frame: 0,
            clashables: HashMap::default(),
            settings: ClashSettings::Buffered(Some(Duration::from_millis(10))),
            should_rescan: true,
        }
    }
}

impl ClashHandler {
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
        for (clashable, state) in self.clashables.iter_mut() {
            let new = if state.frame != self.frame {
                if state.kind.is_none() {
                    None
                } else {
                    Some(ClashStateKind::none())
                }
            } else if let ClashSettings::Buffered(duration) = &self.settings
                && let ClashStateKind::Buffered(start, len) = &state.kind
            {
                if let Some(d) = duration
                    && start.elapsed() >= *d
                {
                    Some(ClashStateKind::released(*len))
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(new) = new {
                bevy::log::info!("T{clashable:?}");
                state.kind.replace(new);
            }
        }
        self.frame += 1;
    }
    pub fn update_clash_list<K, T>(&mut self, map: &HashMap<K, InputBinding<T>>) {
        let clashables: Vec<ClashableKind> =
            map.values().flat_map(|asdf| asdf.clashables()).collect();
        self.clashables.clear();
        let len = clashables.len();
        for (i, c) in clashables.clone().into_iter().enumerate() {
            if i + 1 < len && clashables[i + 1..].contains(&c) {
                match self.clashables.entry(c) {
                    Entry::Occupied(_) => {}
                    Entry::Vacant(v) => {
                        v.insert_entry(ClashState {
                            frame: self.frame,
                            kind: ClashStateKind::None,
                        });
                    }
                }
            }
        }
    }
    /// returns the new value to use for pressed.
    ///
    /// when dealing with axis `pressed` should be `false` when zero and `true` otherwise. and a return
    /// of `true` means use the value, `false` means use zero.
    pub(crate) fn poll_clash(
        &mut self,
        clashable: &ClashableKind,
        chord_length: usize,
        pressed: bool,
    ) -> bool {
        if let Some(state) = self.clashables.get_mut(clashable) {
            let new_state = if pressed {
                match &state.kind {
                    ClashStateKind::None => match self.settings {
                        ClashSettings::Sorted => Some(ClashStateKind::clashing(chord_length)),
                        ClashSettings::Buffered(_) => Some(ClashStateKind::buffered(chord_length)),
                    },
                    ClashStateKind::Clashing(len) => {
                        if chord_length > *len {
                            Some(ClashStateKind::clashing(chord_length))
                        } else {
                            None
                        }
                    }
                    ClashStateKind::Buffered(instant, len) => {
                        if chord_length > *len {
                            Some(ClashStateKind::buffered_with_instant(
                                chord_length,
                                *instant,
                            ))
                        } else {
                            None
                        }
                    }
                    ClashStateKind::Released(len) => {
                        if chord_length > *len {
                            Some(ClashStateKind::released(chord_length))
                        } else {
                            None
                        }
                    }
                }
            } else {
                match &state.kind {
                    ClashStateKind::None => {
                        // not pressed or clashing.
                        None
                    }
                    ClashStateKind::Clashing(_) => Some(ClashStateKind::none()),
                    ClashStateKind::Buffered(_, len) => Some(ClashStateKind::released(*len)),
                    ClashStateKind::Released(_) => None,
                }
            };
            if let Some(new) = new_state {
                bevy::log::info!("P{clashable:?}");
                state.kind.replace(new);
            }
            if pressed && state.frame != self.frame {
                bevy::log::info!("+{clashable:?}");
                state.frame = self.frame;
            }
            state.kind.allowed_to_take_input(chord_length)
        } else {
            pressed
        }
    }
}
