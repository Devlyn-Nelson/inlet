use bevy::platform::collections::HashMap;

use crate::{InputBinding, axis::{AxisBinding, AxisBindingKind}, button::ButtonBinding};

enum Clashable {
    Button(ButtonBinding),
    Axis(AxisBinding),
}

impl PartialEq for Clashable {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Button(l0), Self::Button(r0)) => l0 == r0,
            (Self::Axis(l0), Self::Axis(r0)) => l0 == r0,
            (Self::Button(button_binding), Self::Axis(axis_binding)) |
            (Self::Axis(axis_binding), Self::Button(button_binding)) => {
                if let ButtonBinding::Axis(button_axis) = button_binding {
                    button_axis.as_ref() == axis_binding
                }else if let AxisBindingKind::Buttons { plus, minus } = axis_binding.kind() {
                    if let Some(p) = plus {
                        p.binding == *button_binding
                    }else if let Some(p) = minus {
                        p.binding == *button_binding
                    }else{
                        false
                    }
                }else{
                    false
                }
            }
        }
    }
}

pub struct Clash {}

pub enum ClashReport {
    Capture,
    Ignore,
    None,
}

pub struct ClashHandler<K> {
    clashes: K,
}

impl<K> ClashHandler<K> {
    /// Called when a new binding has been inserted.
    pub fn update_register(&mut self, key: &K) {
        todo!();
    }
    /// Called when a change has been applied to a binding.
    pub fn update_change<T>(&mut self, key: &K, bindings: &HashMap<K, InputBinding<T>>) {
        todo!();
    }
    /// checks for a clash possibility.
    pub fn has_clash(&self, key: &K) -> ClashReport {
        todo!();
    }
}