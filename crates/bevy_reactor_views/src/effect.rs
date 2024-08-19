use bevy::prelude::{Entity, World};

/// A reactive effect that modifies a target entity.
pub trait Effect: Sync + Send {
    /// Start the effect.
    ///
    /// Arguments:
    /// - `owner`: The entity that tracks ownership of this reaction, the reaction
    ///     will be deleted when the owner is deleted.
    /// - `display`: The display entity that will be modified.
    /// - `world`: The Bevy world.
    fn start(&mut self, owner: Entity, display: Entity, world: &mut World);
}
