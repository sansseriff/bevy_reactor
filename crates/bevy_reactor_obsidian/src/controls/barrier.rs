use crate::input_dispatch::{FocusKeyboardInput, SetKeyboardFocus};
use bevy::{ecs::world::DeferredWorld, input::ButtonState, prelude::*};
use bevy_reactor_signals::{Callback, RunCallback};

/// Component for a backdrop element, one that covers the entire screen, blocks click events
/// from reaching elements behind it, and can be used to close a dialog or menu.
#[derive(Component)]
pub struct Barrier {
    pub(crate) on_close: Option<Callback>,
}

pub(crate) fn barrier_on_key_input(
    mut trigger: Trigger<FocusKeyboardInput>,
    q_state: Query<&Barrier>,
    mut world: DeferredWorld,
) {
    if let Ok(bstate) = q_state.get(trigger.entity()) {
        let event = &trigger.event().0;
        if event.state == ButtonState::Pressed
            && !event.repeat
            && (event.key_code == KeyCode::Escape)
        {
            if let Some(on_close) = bstate.on_close {
                trigger.propagate(false);
                world.run_callback(on_close, ());
            }
        }
    }
}

pub(crate) fn barrier_on_pointer_down(
    mut trigger: Trigger<Pointer<Down>>,
    q_state: Query<&Barrier>,
    mut world: DeferredWorld,
) {
    if let Ok(bstate) = q_state.get(trigger.entity()) {
        let checkbox_id = trigger.entity();
        world.set_keyboard_focus(checkbox_id);
        trigger.propagate(false);
        if let Some(on_close) = bstate.on_close {
            world.run_callback(on_close, ());
        }
    }
}
