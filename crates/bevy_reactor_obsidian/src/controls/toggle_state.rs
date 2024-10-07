use crate::input_dispatch::{FocusKeyboardInput, SetKeyboardFocus};
use bevy::{ecs::world::DeferredWorld, input::ButtonState, prelude::*};
use bevy_reactor_signals::{Callback, RunCallback, Signal};

use super::Disabled;

#[derive(Component)]
pub struct ToggleState {
    pub(crate) checked: Signal<bool>,
    pub(crate) on_change: Option<Callback<bool>>,
}

pub(crate) fn toggle_on_key_input(
    mut trigger: Trigger<FocusKeyboardInput>,
    q_state: Query<(&ToggleState, Has<Disabled>)>,
    mut world: DeferredWorld,
) {
    if let Ok((tstate, disabled)) = q_state.get(trigger.entity()) {
        let event = &trigger.event().0;
        if !disabled
            && event.state == ButtonState::Pressed
            && !event.repeat
            && (event.key_code == KeyCode::Enter || event.key_code == KeyCode::Space)
        {
            let is_checked = tstate.checked.get(&world);
            if let Some(on_change) = tstate.on_change {
                trigger.propagate(false);
                world.run_callback(on_change, !is_checked);
            }
        }
    }
}

pub(crate) fn toggle_on_pointer_click(
    mut trigger: Trigger<Pointer<Click>>,
    q_state: Query<(&ToggleState, Has<Disabled>)>,
    mut world: DeferredWorld,
) {
    if let Ok((tstate, disabled)) = q_state.get(trigger.entity()) {
        let checkbox_id = trigger.entity();
        world.set_keyboard_focus(checkbox_id);
        trigger.propagate(false);
        if let Some(on_change) = tstate.on_change {
            if !disabled {
                let is_checked = tstate.checked.get(&world);
                world.run_callback(on_change, !is_checked);
            }
        }
    }
}
