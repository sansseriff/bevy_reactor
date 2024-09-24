use std::sync::{Arc, Mutex};

use bevy::ecs::{component::Component, entity::Entity, world::World};

use crate::tracking_scope::TrackingScope;

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
pub struct ReactionCell(pub Arc<Mutex<dyn Reaction + Send + Sync + 'static>>);

impl ReactionCell {
    /// Construct a new [`ReactionCell`].
    pub fn new<R: Reaction + Send + Sync + 'static>(reaction: R) -> Self {
        Self(Arc::new(Mutex::new(reaction)))
    }
}

// /// Command which performs the initial (startup) reaction.
// pub struct InitialReactionCommand(Entity);

// impl Command for InitialReactionCommand {
//     fn apply(self, world: &mut World) {
//         let entity = self.0;
//         let Some(reaction) = world.get::<ReactionCell>(entity).map(|r| r.0.clone()) else {
//             return;
//         };
//         let Some(mut tracking) = world.get_mut::<TrackingScope>(entity) else {
//             return;
//         };
//         reaction
//             .lock()
//             .unwrap()
//             .react(entity, world, &mut *tracking);
//     }
// }
