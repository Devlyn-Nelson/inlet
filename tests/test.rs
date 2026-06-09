use bevy::{
    input::gamepad::GamepadAxis,
    math::Vec2,
    prelude::{GamepadButton, Message},
};
use inlet::{
    InputBindings,
    axis::{AxisBinding, DualValueBinding},
    button::{ActionBinding, ButtonEventBinding},
};

/// The Input types. used as a key to organize which binding belongs to which input.
#[derive(Eq, Hash, PartialEq)]
pub enum GenericActions {
    Jump,
    Move,
    Zoom,
}

/// This defines [`Messages`] that can be sent. This is optional, using `SimpleInputBindings` will use a
/// placeholder message type that can just be ignored.
#[derive(Clone, Message)]
pub enum GenericActionsMessage {
    Jump,
    Move(Vec2),
}

// functions to pass into bindings for Message creation on trigger.
impl GenericActionsMessage {
    // this is for the jump message.
    pub fn jump() -> Self {
        Self::Jump
    }
    // this is a dual axis input message for movement.
    // It is required that its only parameter be a [`Vec2`] and its output be [`Option<Self>`].
    // if a message should be triggered return `Some` otherwise `None`.
    pub fn move_player(values: Vec2) -> Option<Self> {
        if values == Vec2::ZERO {
            Some(Self::Move(values))
        } else {
            None
        }
    }
}

#[test]
fn test_fn() {
    // Create a new bindings. All of the `register_` function call below could be done using `with_` equivalents.
    let mut bindings = InputBindings::<GenericActions, GenericActionsMessage>::new();
    // Bindings live in an enum that can be any type of binding but these functions are nicer to use.
    //
    // A simple button activated action that sends a `GenericActionsMessage::Jump` Message.
    bindings.register_action_binding(
        GenericActions::Jump,
        ActionBinding::new(
            vec![GamepadButton::South.into()],
            ButtonEventBinding::when_pressed(GenericActionsMessage::jump),
        ),
    );
    // Single axis that does not send any Message
    bindings.register_value_binding(
        GenericActions::Zoom,
        AxisBinding::gamepad_axis(GamepadAxis::RightZ).into(),
    );

    let dvb: DualValueBinding<GenericActionsMessage> = (
        AxisBinding::gamepad_right_stick_x(),
        AxisBinding::gamepad_right_stick_y(),
    )
        .into();
    bindings.register_dual_value_binding(
        GenericActions::Move,
        dvb.with_event(GenericActionsMessage::move_player),
    );

    // Use may notice not checks for the type of input exist.
    assert!(bindings.get_action_state(&GenericActions::Jump).released());
    assert_eq!(bindings.get_value(&GenericActions::Zoom), 0.);
    assert_eq!(
        bindings.get_dual_value(&GenericActions::Move),
        Vec2::default()
    );
    // Any input type can be used as any input type.
    // for example actions can be use as values (unpressed would be 0.0 and pressed would be 1.0)
    assert_eq!(bindings.get_value(&GenericActions::Jump), 0.);
    // Or values can be actions (unpressed if a value is 0.0, pressed otherwise).
    assert!(bindings.get_action_state(&GenericActions::Move).released(),);
}
