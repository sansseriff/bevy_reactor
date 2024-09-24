use std::cell::RefCell;

use bevy::prelude::{Entity, EntityWorldMut, World};
use bevy_reactor_signals::TrackingScope;

pub struct UiBuilder<'p, 'w> {
    /// Bevy World
    world: &'w mut World,

    /// The entity that owns the tracking scope (or will own it).
    owner: Entity,

    /// Set of reactive resources referenced by the presenter.
    tracking: RefCell<&'p mut TrackingScope>,
}

impl<'p, 'w> UiBuilder<'p, 'w> {
    /// Construct a new reactive context.
    pub fn new(world: &'w mut World, owner: Entity, tracking: &'p mut TrackingScope) -> Self {
        Self {
            world,
            owner,
            tracking: RefCell::new(tracking),
        }
    }

    /// Access to world from reactive context.
    pub fn world(&self) -> &World {
        self.world
    }

    /// Access to mutable world from reactive context.
    pub fn world_mut(&mut self) -> &mut World {
        self.world
    }

    // pub fn entity(&mut self) {

    // }

    // entity(id)
    // spawn()
    // cond()
    // switch()
    // for_each()
    // for_index()
}

pub trait BuildChildren {
    fn build_children(&mut self) -> Self;
}

impl<'w> BuildChildren for EntityWorldMut<'w> {
    fn build_children(&mut self) -> Self {
        todo!()
    }
}
