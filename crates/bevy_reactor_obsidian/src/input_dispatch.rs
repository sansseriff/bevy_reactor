use bevy::{a11y::Focus, ecs::world::DeferredWorld, input::keyboard::KeyboardInput, prelude::*};

#[derive(Clone, Debug, Component)]
pub struct FocusKeyboardInput(pub KeyboardInput);

impl Event for FocusKeyboardInput {
    type Traversal = Parent;

    const AUTO_PROPAGATE: bool = true;
}

#[derive(Clone, Debug, Resource)]
pub struct KeyboardFocus(pub Option<Entity>);

#[derive(Clone, Debug, Resource)]
pub struct KeyboardFocusVisible(pub bool);

/// Marker component for which entity should receive keyboard input if nothing is focused.
#[derive(Clone, Debug, Component)]
pub struct DefaultKeyHandler;

pub trait SetKeyboardFocus {
    fn set_keyboard_focus(&mut self, entity: Entity);
    fn clear_keyboard_focus(&mut self);
}

impl SetKeyboardFocus for World {
    fn set_keyboard_focus(&mut self, entity: Entity) {
        if let Some(mut focus) = self.get_resource_mut::<KeyboardFocus>() {
            focus.0 = Some(entity);
        }
        if let Some(mut focus) = self.get_resource_mut::<Focus>() {
            focus.0 = Some(entity);
        }
    }

    fn clear_keyboard_focus(&mut self) {
        if let Some(mut focus) = self.get_resource_mut::<KeyboardFocus>() {
            focus.0 = None;
        }
        if let Some(mut focus) = self.get_resource_mut::<Focus>() {
            focus.0 = None;
        }
    }
}

impl<'w> SetKeyboardFocus for DeferredWorld<'w> {
    fn set_keyboard_focus(&mut self, entity: Entity) {
        if let Some(mut focus) = self.get_resource_mut::<KeyboardFocus>() {
            focus.0 = Some(entity);
        }
        if let Some(mut focus) = self.get_resource_mut::<Focus>() {
            focus.0 = Some(entity);
        }
    }

    fn clear_keyboard_focus(&mut self) {
        if let Some(mut focus) = self.get_resource_mut::<KeyboardFocus>() {
            focus.0 = None;
        }
        if let Some(mut focus) = self.get_resource_mut::<Focus>() {
            focus.0 = None;
        }
    }
}

/// Plugin which registers the system for dispatching keyboard events based on focus and
/// hover state.
pub struct InputDispatchPlugin;

impl Plugin for InputDispatchPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(KeyboardFocus(None))
            .insert_resource(KeyboardFocusVisible(false))
            .add_systems(Update, (dispatch_keyboard_input, sync_a11y_focus));
    }
}

// Q: Is there a way to do this as an observer or hook?
fn sync_a11y_focus(focus: Res<KeyboardFocus>, mut a11y_focus: ResMut<Focus>) {
    if a11y_focus.0 != focus.0 {
        a11y_focus.0 = focus.0
    }
}

fn dispatch_keyboard_input(
    mut key_events: EventReader<KeyboardInput>,
    focus: Res<KeyboardFocus>,
    q_default_handler: Query<Entity, With<DefaultKeyHandler>>,
    mut commands: Commands,
) {
    // If an element has keyboard focus, then dispatch the key event to that element.
    if let Some(focus_elt) = focus.0 {
        for ev in key_events.read() {
            commands.trigger_targets(FocusKeyboardInput(ev.clone()), focus_elt);
        }
    } else if let Some(ent) = q_default_handler.iter().next() {
        for ev in key_events.read() {
            commands.trigger_targets(FocusKeyboardInput(ev.clone()), ent);
        }
    } else if !key_events.is_empty() {
        warn!("No focus entity and no default keyboard handler: try inserting DefaultKeyHandler on your top-level entity");
    }
}
