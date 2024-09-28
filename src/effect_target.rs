use std::sync::{Arc, Mutex};

use bevy::{
    core::Name,
    ecs::{bundle::Bundle, entity::Entity, world::World},
    hierarchy::BuildChildren,
};

use bevy_reactor_signals::{Cx, Rcx, Reaction, ReactionCell, TrackingScope};

/// A reactive effect that modifies a target entity.
pub trait EntityEffect: Sync + Send {
    /// Start the effect.
    ///
    /// Arguments:
    /// - `owner`: The entity that tracks ownership of this reaction, the reaction
    ///     will be deleted when the owner is deleted.
    /// - `display`: The display entity that will be modified.
    /// - `world`: The Bevy world.
    /// - `tracking`: Tracking scope attached to `owner`.
    fn start(
        &mut self,
        owner: Entity,
        display: Entity,
        world: &mut World,
        tracking: &mut TrackingScope,
    );
}

/// An object which can have effects applied to it.
pub trait EffectTarget
where
    Self: Sized,
{
    /// Add a reactive effct to the element.
    fn add_effect(&mut self, effect: Box<dyn EntityEffect>);

    /// Add a reaction to the element. This is a convenience method for adding a reactive
    /// effect that is already in the form of a `Reaction`.
    fn add_reaction<R: Reaction + Send + Sync + 'static>(&mut self, reaction: R) {
        self.add_effect(Box::new(RunReactionEffect::new(reaction)));
    }

    /// Start a reaction which updates the entity.
    fn start_reaction<R: Reaction + Send + Sync + 'static>(
        &mut self,
        reaction: R,
        owner: Entity,
        _target: Entity,
        world: &mut World,
        _parent_scope: &mut TrackingScope,
    ) {
        // Create a tracking scope for the reaction.
        let mut scope = TrackingScope::new(world.change_tick());

        // Unwrap the reaction and update the target entity, since this was not known at
        // the time the reaction was constucted.
        let reaction_arc = Arc::new(Mutex::new(reaction));
        let mut reaction = reaction_arc.lock().unwrap();

        // Store the reaction in a handle and add it to the world.
        let reaction_id = world
            .spawn((
                ReactionCell(reaction_arc.clone()),
                Name::new("EffectTarget::start_reaction"),
            ))
            .set_parent(owner)
            .id();

        // Add the reaction id to the parent scope so that it can be despawned later.
        // parent_scope.add_owned(reaction_id);

        // Call `react` the first time, update the scope with initial deps.
        // Note that we need to insert the ReactionTarget first!
        reaction.react(reaction_id, world, &mut scope);

        // Store the scope in the reaction entity.
        world.entity_mut(reaction_id).insert(scope);
    }

    /// Create a reactive effect which is attached to the element.
    fn create_effect<F: Send + Sync + 'static + FnMut(&mut Cx, Entity)>(self, _effect: F) -> Self {
        // self.add_reaction(UpdateReaction::new(effect));
        self
    }

    /// Add a static bundle to the element.
    fn insert<T: Bundle>(mut self, bundle: T) -> Self {
        self.add_effect(Box::new(InsertBundleEffect {
            bundle: Some(bundle),
        }));
        self
    }

    /// Add a static bundle to the element, if a condition is true.
    fn insert_if<T: Bundle>(mut self, cond: bool, bundle: T) -> Self {
        if cond {
            self.add_effect(Box::new(InsertBundleEffect {
                bundle: Some(bundle),
            }));
        }
        self
    }

    /// Add a computed bundle to the element.
    fn insert_computed<T: Bundle, F: Send + Sync + 'static + FnMut(&mut Rcx) -> T>(
        self,
        _factory: F,
    ) -> Self {
        // self.add_reaction(ComputedBundleReaction::new(factory));
        self
    }
}

/// Inserts a static, pre-constructed bundle into the target entity. No reactivity.
pub struct InsertBundleEffect<B: Bundle> {
    pub(crate) bundle: Option<B>,
}

impl<B: Bundle> EntityEffect for InsertBundleEffect<B> {
    // For a static bundle, we can just insert it once.
    fn start(
        &mut self,
        _owner: Entity,
        target: Entity,
        world: &mut World,
        _tracking: &mut TrackingScope,
    ) {
        world.entity_mut(target).insert(self.bundle.take().unwrap());
    }
}

/// Effect that runs a reaction function (reactively).
pub struct RunReactionEffect<R> {
    reaction: Arc<Mutex<R>>,
}

impl<R> RunReactionEffect<R> {
    pub(crate) fn new(reaction: R) -> Self {
        Self {
            reaction: Arc::new(Mutex::new(reaction)),
        }
    }
}

impl<R: Reaction + Send + Sync + 'static> EntityEffect for RunReactionEffect<R> {
    // Start a reaction which updates the bundle.
    fn start(
        &mut self,
        owner: Entity,
        _target: Entity,
        world: &mut World,
        _parent_scope: &mut TrackingScope,
    ) {
        // Compute debug name for reaction.
        let reaction_name = world
            .entity(owner)
            .get::<Name>()
            .map_or(Name::new("RunReactionEffect"), |n| {
                Name::new(format!("{}::RunReactionEffect", n))
            });

        // Create a tracking scope for the reaction.
        let mut scope = TrackingScope::new(world.change_tick());

        // Unwrap the reaction and update the target entity, since this was not known at
        // the time the reaction was constucted.
        let mut reaction = self.reaction.lock().unwrap();

        // Store the reaction in a handle and add it to the world.
        let reaction_id = world
            .spawn((ReactionCell(self.reaction.clone()), reaction_name))
            .set_parent(owner)
            .id();

        // Add the reaction id to the parent scope so that it can be despawned later.
        // parent_scope.add_owned(reaction_id);

        // Call `react` the first time, update the scope with initial deps.
        // Note that we need to insert the ReactionTarget first!
        reaction.react(reaction_id, world, &mut scope);

        // Store the scope in the reaction entity.
        world.entity_mut(reaction_id).insert(scope);
    }
}
