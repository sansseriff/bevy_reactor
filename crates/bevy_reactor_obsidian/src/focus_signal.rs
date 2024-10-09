use bevy::{
    a11y::Focus,
    ecs::{entity::Entity, world::World},
    hierarchy::Parent,
};
use bevy_reactor_builder::UiBuilder;
use bevy_reactor_signals::Signal;

use crate::input_dispatch::KeyboardFocusVisible;

/// True if the given entity is a descendant of the given ancestor.
fn is_descendant(world: &World, e: &Entity, ancestor: &Entity) -> bool {
    let mut ha = e;
    loop {
        if ha == ancestor {
            return true;
        }
        match world.get_entity(*ha).map(|e| e.get::<Parent>()) {
            Ok(Some(parent)) => ha = parent,
            _ => return false,
        }
    }
}

/// Method to create a signal that tracks whether a target entity has focus.
pub trait CreateFocusSignal {
    /// Signal that returns true when the the target has focus.
    fn create_focus_signal(&mut self, target: Entity) -> Signal<bool>;

    /// Signal that returns true when the the target, or a descendant, has focus.
    fn create_focus_within_signal(&mut self, target: Entity) -> Signal<bool>;

    /// Signal that returns true when the the target has focus and the focus ring is visible.
    fn create_focus_visible_signal(&mut self, target: Entity) -> Signal<bool>;

    /// Signal that returns true when the the target, or a descendant, has focus, and the
    /// focus ring is visible.
    fn create_focus_within_visible_signal(&mut self, target: Entity) -> Signal<bool>;
}

impl<'w> CreateFocusSignal for UiBuilder<'w> {
    fn create_focus_signal(&mut self, target: Entity) -> Signal<bool> {
        self.create_derived(move |rcx| {
            let focus = rcx.read_resource::<Focus>();
            focus.0 == Some(target)
        })
    }

    fn create_focus_within_signal(&mut self, target: Entity) -> Signal<bool> {
        self.create_derived(move |rcx| {
            let focus = rcx.read_resource::<Focus>();
            match focus.0 {
                Some(focus) => is_descendant(rcx.world(), &focus, &target),
                None => false,
            }
        })
    }

    fn create_focus_visible_signal(&mut self, target: Entity) -> Signal<bool> {
        self.create_derived(move |rcx| {
            let visible = rcx.read_resource::<KeyboardFocusVisible>();
            let focus = rcx.read_resource::<Focus>();
            visible.0 && focus.0 == Some(target)
        })
    }

    fn create_focus_within_visible_signal(&mut self, target: Entity) -> Signal<bool> {
        self.create_derived(move |rcx| {
            let visible = rcx.read_resource::<KeyboardFocusVisible>();
            if !visible.0 {
                return false;
            }
            let focus = rcx.read_resource::<Focus>();
            match focus.0 {
                Some(focus) => is_descendant(rcx.world(), &focus, &target),
                None => false,
            }
        })
    }
}
