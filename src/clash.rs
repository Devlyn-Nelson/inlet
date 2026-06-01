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
            ClashStateKind::Buffered(_, len) => chord_len == *len,
            ClashStateKind::Released(len) => chord_len == *len,
        }
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
    LongestHeld,
    /// Waits for the highest-priority or longest chord to capture the input within a Duration. This will
    /// always skip the "JustPressed" tick because it wants to wait for the longest input. If the Duration
    /// from the initial press of the clash passes or the input is released: inputs with a chord length that
    /// matches the longest chord length of inputs that tried to gather the input.
    Buffered(Option<Duration>),
}

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
            settings: ClashSettings::LongestHeld,
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
    pub fn set_clash_settings(&mut self, settings: ClashSettings) {
        self.settings = settings;
    }
    pub fn tick(&mut self) {
        for c in self.clashables.values_mut() {
            let new = if c.frame != self.frame {
                Some(ClashStateKind::None)
            } else {
                if let ClashSettings::Buffered(duration) = &self.settings
                    && let ClashStateKind::Buffered(start, len) = &c.kind
                {
                    if let Some(d) = duration
                        && start.elapsed() >= *d
                    {
                        Some(ClashStateKind::Released(*len))
                    } else {
                        None
                    }
                } else {
                    None
                }
            };
            if let Some(new) = new {
                c.kind = new;
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
                        ClashSettings::LongestHeld => {
                            bevy::log::info!("len -> {chord_length}");
                            Some(ClashStateKind::Clashing(chord_length))
                        }
                        ClashSettings::Buffered(_) => {
                            bevy::log::info!("len -> {chord_length}");
                            Some(ClashStateKind::Buffered(Instant::now(), chord_length))
                        }
                    },
                    ClashStateKind::Clashing(len) => {
                        if chord_length > *len {
                            bevy::log::info!("len -> {chord_length}");
                            Some(ClashStateKind::Clashing(chord_length))
                        } else {
                            None
                        }
                    }
                    ClashStateKind::Buffered(instant, len) => {
                        if chord_length > *len {
                            bevy::log::info!("len -> {chord_length}");
                            Some(ClashStateKind::Buffered(*instant, chord_length))
                        } else {
                            None
                        }
                    }
                    ClashStateKind::Released(_) => None,
                }
            } else {
                match &state.kind {
                    ClashStateKind::None => {
                        // not pressed or clashing.
                        None
                    }
                    ClashStateKind::Clashing(len) => Some(ClashStateKind::Released(*len)),
                    ClashStateKind::Buffered(_, len) => Some(ClashStateKind::Released(*len)),
                    ClashStateKind::Released(_) => None,
                }
            };
            if let Some(new) = new_state {
                state.kind = new;
            }
            if state.frame != self.frame {
                state.frame = self.frame;
            }
            state.kind.allowed_to_take_input(chord_length)
        } else {
            pressed
        }
    }
}
