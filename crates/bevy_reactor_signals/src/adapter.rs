use bevy::{
    log::warn,
    prelude::{Component, Entity, World},
};

use crate::{Reaction, ReactionCell, TrackingScope};

/// An `Adapter` is a stateless object that can be used to access an object implementing
/// [`Reaction`] in a type-erased fashion, without needing to store a `dyn Reaction`. This allows
/// querying for ECS components that contain implementations of the `Reaction` trait without knowing
/// the concrete type. Note that it must be stateless since it is created statically, and passed
/// around by static reference.
///
/// In order for this to work, two ECS components are needed: [`ReactionThunk`] and a second
/// component to actually hold the concrete view, such as [`ReactionCell`].
///
/// `ReactionThunk` is a type-erased component that provides access to the concrete `Reaction` via
/// the `Adapter`. `ReactionCell` is a component that contains the actual `Reaction` and it's state.
/// Because `ReactionCell` is generic, it cannot be queried directly (since it would require a
/// separate query for every specialization), so we use `ReactionThunk` as an adapter. `Adapter`
/// also can work with other component types that contain a `Reaction` implementation.
///
/// The the methods of `Adapter` and `ReactionThunk` take an entity id to identify the view rather
/// than `self``; the `self` parameter is only there to make the methods dyn-trait compatible. This
/// allows the adapter to be created as a static object, since all of its state is external.
///
/// See [`AnyAdapter`] and [`ReactionThunk`].
pub struct Adapter<R: Reaction> {
    marker: std::marker::PhantomData<R>,
}

/// The dynamic trait used by [`ViewAdapter`]. See also [`ViewThunk`].
pub trait AnyAdapter: Sync + Send + 'static {
    fn react(&self, owner: Entity, world: &mut World, tracking: &mut TrackingScope);
}

impl<R: Reaction + Send + Sync + 'static> AnyAdapter for Adapter<R> {
    fn react(&self, owner: Entity, world: &mut World, scope: &mut TrackingScope) {
        // let mut cx = Cx::new(world, owner, scope);
        if let Some(view_cell) = world.entity_mut(owner).get_mut::<ReactionCell<R>>() {
            let inner = view_cell.0.clone();
            let mut reaction = inner.lock().unwrap();
            reaction.react(owner, world, scope);
        } else {
            warn!("No ReactionCell found for entity {}", owner);
        }
    }
}

/// An ECS component which wraps a type-erased [`Adapter`].
#[derive(Component, Clone, Copy)]
pub struct ReactionThunk(pub(crate) &'static dyn AnyAdapter);

impl ReactionThunk {
    /// Invoke the reaction trait stored in the given entity.
    pub fn react(&self, owner: Entity, world: &mut World, scope: &mut TrackingScope) {
        self.0.react(owner, world, scope)
    }

    /// Create a new `ReactionThunk` for the given `Reaction` type.
    pub fn for_reaction<R: Reaction + Send + Sync + 'static>() -> Self {
        Self(&Adapter::<R> {
            marker: std::marker::PhantomData,
        })
    }
}
