use bevy::prelude::{Entity, World};

/// A reactive effect that modifies a target entity.
pub trait Effect: Sync + Send {
    /// Start the effect.
    ///
    /// Arguments:
    /// - `owner`: The entity that this effect is attached to.
    /// - `world`: The Bevy world.
    fn start(self: Box<Self>, owner: Entity, world: &mut World);
}
