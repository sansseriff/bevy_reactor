use std::sync::{Arc, Mutex};

use bevy::ecs::{component::Component, entity::Entity, world::World};

use crate::scope::TrackingScope;

/// Trait representing a reaction to changes in dependencies. The trait's [`react`] method
/// is called when the dependencies change (dependencies are tracked in a separate
/// [`TrackingScope`] component).
///
/// Note that the reaction is not automatically run when it is first created - it's the
/// responsibility of the caller to call [`react`] at least once. The reason for this is
/// that under normal circumstances, we want [`react`] to be run synchronously.
pub trait Reaction {
    /// Update the reaction code in response to changes in dependencies.
    ///
    /// Arguments:
    /// - `owner`: The entity that owns this reaction and tracking scope.
    /// - `world`: The Bevy world.
    /// -
    fn react(&mut self, owner: Entity, world: &mut World, tracking: &mut TrackingScope);

    /// Release any resources used by the reaction.
    fn cleanup(&mut self, _owner: Entity, _world: &mut World) {}
}

/// A reference to a reaction.
pub type ReactionRef = Arc<Mutex<dyn Reaction + Sync + Send + 'static>>;

/// Component which contains a reference to a reaction. Generally the entity will also
/// have a [`TrackingScope`] component.
#[derive(Component)]
pub struct ReactionHandle(pub(crate) ReactionRef);

impl ReactionHandle {
    /// Construct a new [`ReactionHandle`].
    pub fn new(view: impl Reaction + Sync + Send + 'static) -> Self {
        Self(Arc::new(Mutex::new(view)))
    }
}
