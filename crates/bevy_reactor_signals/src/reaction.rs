use std::sync::{Arc, Mutex};

use bevy::ecs::{component::Component, entity::Entity, world::World};

use crate::{tracking_scope::TrackingScope, ReactionThunk};

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
    /// - `tracking`: The tracking scope for the reaction.
    fn react(&mut self, owner: Entity, world: &mut World, tracking: &mut TrackingScope);
}

/// Component which contains a reference to a reaction. Generally the entity will also
/// have a [`TrackingScope`] component.
#[derive(Component)]
pub struct ReactionCell<R: Reaction>(pub Arc<Mutex<R>>);

impl<R: Reaction + Send + Sync + 'static> ReactionCell<R> {
    /// Construct a new [`ReactionCell`].
    pub fn new(reaction: R) -> Self {
        Self(Arc::new(Mutex::new(reaction)))
    }

    /// Create a [`ReactionThunk`] for this reaction type.
    pub fn thunk() -> ReactionThunk {
        ReactionThunk::for_reaction::<R>()
    }
}

/// In some cases, reactions are targeted at an entity other than the owner, where the entity
/// id is not known until the reaction is started. This component tracks the target entity.
#[derive(Component)]
pub struct ReactionTarget(pub Entity);
