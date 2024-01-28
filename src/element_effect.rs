use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use bevy::ecs::{bundle::Bundle, component::Component, entity::Entity, world::World};

use crate::{scope::TrackingScope, Cx, Rcx, Reaction, ReactionHandle};

/// A reactive effect that modifies a target entity.
pub trait ElementEffect: Sync + Send {
    /// Start the effect.
    ///
    /// Arguments:
    /// - `owner`: The entity that tracks ownership of this reaction, the reaction
    ///     will be deleted when the owner is deleted.
    /// - `target`: The display entity that the bundle will be inserted into.
    /// - `world`: The Bevy world.
    fn start(&mut self, tracking: &mut TrackingScope, target: Entity, world: &mut World);
}

/// Allows a reaction's target entity to be set.
pub trait Targetable {
    fn set_target(&mut self, target: Entity);
}

/// Inserts a static, pre-constructed bundle into the target entity. No reactivity.
pub struct InsertBundleEffect<B: Bundle> {
    pub(crate) bundle: Option<B>,
}

impl<B: Bundle> ElementEffect for InsertBundleEffect<B> {
    // For a static bundle, we can just insert it once.
    fn start(&mut self, _tracking: &mut TrackingScope, target: Entity, world: &mut World) {
        world.entity_mut(target).insert(self.bundle.take().unwrap());
    }
}

/// Effect that runs a reaction function (reactively).
pub struct RunReactionEffect<R: Targetable> {
    reaction: Arc<Mutex<R>>,
}

impl<R: Targetable> RunReactionEffect<R> {
    pub(crate) fn new(reaction: R) -> Self {
        Self {
            reaction: Arc::new(Mutex::new(reaction)),
        }
    }
}

impl<R: Targetable + Reaction + Send + Sync + 'static> ElementEffect for RunReactionEffect<R> {
    // Start a reaction which updates the bundle.
    fn start(&mut self, parent_scope: &mut TrackingScope, target: Entity, world: &mut World) {
        // Create a tracking scope for the reaction.
        let mut scope = TrackingScope::new(world.change_tick());

        // Unwrap the reaction and update the target entity, since this was not known at
        // the time the reaction was constucted.
        let mut reaction = self.reaction.lock().unwrap();
        reaction.set_target(target);

        // Store the reaction in a handle and add it to the world.
        let reaction_id = world.spawn(ReactionHandle(self.reaction.clone())).id();

        // Call `react` the first time, update the scope with initial deps.
        reaction.react(reaction_id, world, &mut scope);

        // Store the scope in the reaction entity.
        world.entity_mut(reaction_id).insert(scope);

        // Add the reaction id to the parent scope so that it can be despawned later.
        parent_scope.add_owned(reaction_id);
    }
}

/// Calls a closure which computes a bundle reactively, returns the bundle as a result.
/// This is then inserted into the target.
pub struct ComputedBundleReaction<B: Bundle, F: FnMut(&mut Rcx) -> B> {
    target: Option<Entity>,
    factory: F,
}

impl<B: Bundle, F: Sync + Send + FnMut(&mut Rcx) -> B> ComputedBundleReaction<B, F> {
    pub(crate) fn new(factory: F) -> Self {
        Self {
            target: None,
            factory,
        }
    }
}

impl<B: Bundle, F: Sync + Send + FnMut(&mut Rcx) -> B> Reaction for ComputedBundleReaction<B, F> {
    fn react(&mut self, _owner: Entity, world: &mut World, tracking: &mut TrackingScope) {
        let mut re = Rcx::new(world, tracking);
        let b = (self.factory)(&mut re);
        let mut entt = world.entity_mut(self.target.unwrap());
        entt.insert(b);
    }
}

impl<B: Bundle, F: Sync + Send + FnMut(&mut Rcx) -> B> Targetable for ComputedBundleReaction<B, F> {
    fn set_target(&mut self, target: Entity) {
        self.target = Some(target);
    }
}

/// Produces a bundle reactively, returns the bundle as a result.
pub struct BundleComputedRefReaction<C: Component, F: FnMut(&mut Rcx, &mut C)> {
    target: Option<Entity>,
    updater: F,
    marker: PhantomData<C>,
}

impl<C: Component, F: FnMut(&mut Rcx, &mut C)> Reaction for BundleComputedRefReaction<C, F> {
    fn react(&mut self, _owner: Entity, world: &mut World, tracking: &mut TrackingScope) {
        let mut entt = world.entity_mut(self.target.unwrap());
        let mut _cmp = entt.get_mut::<C>().unwrap();
        let mut _re = Rcx::new(entt.world(), tracking);
        // (self.updater)(&mut re, &mut cmp);
        todo!("BundleUpdateReaction::react - needs to borrow world while mutating component");
    }
}

// /// Allows reactively mutating a single component. Allows specifying the initial component value,
// /// as well as an update function the mutates the component in place.
// pub struct BundleComputedRef<C: Component, F1: FnMut() -> C, F2: FnMut(&mut Re, &mut C)> {
//     pub(crate) target: Option<Entity>,
//     pub(crate) init: F1,
//     pub(crate) update: F2,
//     pub(crate) marker: PhantomData<C>,
//     pub(crate) tracker: Option<Entity>,
// }

// impl<C: Component, F1: Send + Sync + FnMut() -> C, F2: Send + Sync + FnMut(&mut Re, &mut C)>
//     BundleProducer for BundleComputedRef<C, F1, F2>
// {
//     // Insert the bundle on build, then update.
//     fn init(&mut self, target: Entity, world: &mut World) {
//         assert!(self.tracker.is_none());
//         let mut scope = TrackingScope::new(world.change_tick());
//         let b = (self.init)();
//         let mut entt = world.entity_mut(target);
//         entt.insert(b);
//         self.target = Some(target);
//         self.react(target, world, &mut scope);
//         let tracker = world.spawn(scope);
//         self.tracker = Some(tracker.id());
//     }
// }

// impl<C: Component, F1: Send + Sync + FnMut() -> C, F2: Send + Sync + FnMut(&mut Re, &mut C)>
//     Reaction for BundleComputedRef<C, F1, F2>
// {
//     fn react(&mut self, target: Entity, world: &mut World, tracking: &mut TrackingScope) {
//         let mut entt = world.entity_mut(target);
//         let mut cmp = entt.get_mut::<C>().unwrap();
//         let mut re = Re::new(world, tracking);
//         todo!();
//         // (self.update)(&mut re, &mut cmp);
//     }

//     fn cleanup(&mut self, owner: Entity, world: &mut World) {
//         assert!(self.tracker.is_some());
//         world.entity_mut(self.tracker.unwrap()).despawn();
//         self.tracker = None;
//     }
// }

/// Produces a bundle reactively, returns the bundle as a result.
pub struct UpdateReaction<F: FnMut(&mut Cx, Entity)> {
    target: Option<Entity>,
    effect: F,
}

impl<F: FnMut(&mut Cx, Entity)> UpdateReaction<F> {
    pub(crate) fn new(effect: F) -> Self {
        Self {
            target: None,
            effect,
        }
    }
}

impl<F: FnMut(&mut Cx, Entity)> Reaction for UpdateReaction<F> {
    fn react(&mut self, _owner: Entity, world: &mut World, tracking: &mut TrackingScope) {
        let mut cx = Cx::new(&(), world, tracking);
        (self.effect)(&mut cx, self.target.unwrap());
    }
}

impl<F: FnMut(&mut Cx, Entity)> Targetable for UpdateReaction<F> {
    fn set_target(&mut self, target: Entity) {
        self.target = Some(target);
    }
}
