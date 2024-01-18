use std::{marker::PhantomData, sync::atomic::Ordering};

use bevy::{
    ecs::component::{ComponentId, Tick},
    prelude::*,
    utils::{HashMap, HashSet},
};

use crate::{mutable::MutableValue, reaction::ReactionHandle, ViewContext, ViewHandle};

/// A component that tracks the dependencies of a reactive task.
#[derive(Component)]
pub struct TrackingScope {
    /// List of scopes that are owned by this scope.
    owned: Vec<Entity>,

    /// The set of mutables that this scope is subscribed to.
    mutable_deps: HashSet<Entity>,

    /// Set of components that we are currently subscribed to.
    component_deps: HashSet<(Entity, ComponentId)>,

    /// Set of resources that we are currently subscribed to.
    resource_deps: HashMap<ComponentId, Box<dyn AnyResource>>,

    /// Engine tick used for determining if components have changed. This represents the
    /// time of the previous reaction.
    tick: Tick,
    // contexts
    // debug_name
    // cleanups
}

impl TrackingScope {
    pub fn new(tick: Tick) -> Self {
        Self {
            owned: Vec::new(),
            mutable_deps: HashSet::default(),
            component_deps: HashSet::default(),
            resource_deps: HashMap::default(),
            tick,
        }
    }

    pub(crate) fn add_owned(&mut self, owned: Entity) {
        self.owned.push(owned);
    }

    pub(crate) fn add_mutable(&mut self, mutable: Entity) {
        self.mutable_deps.insert(mutable);
    }

    pub(crate) fn add_resource<T: Resource>(&mut self, resource_id: ComponentId) {
        self.resource_deps
            .entry(resource_id)
            .or_insert_with(|| Box::new(TrackedResource::<T>::new()));
    }

    /// Returns true if any of the dependencies of this scope have been updated since
    /// the previous reaction.
    fn dependencies_changed(&self, world: &World) -> bool {
        self.mutable_deps.iter().any(|m| {
            world
                .entity(*m)
                .get::<MutableValue>()
                .map(|m| m.changed.load(Ordering::Relaxed))
                .unwrap_or(false)
        }) || self.resource_deps.iter().any(|(_, c)| c.is_changed(world))
    }

    /// Swap the dependencies with another scope. Typically the other scope is a temporary
    /// scope that is used to compute the next set of dependencies.
    pub(crate) fn swap_deps(&mut self, other: &mut Self) {
        std::mem::swap(&mut self.mutable_deps, &mut other.mutable_deps);
        std::mem::swap(&mut self.component_deps, &mut other.component_deps);
        std::mem::swap(&mut self.resource_deps, &mut other.resource_deps);
    }
}

pub trait AnyResource: Send + Sync {
    fn is_changed(&self, world: &World) -> bool;
}

#[derive(PartialEq, Eq)]
pub struct TrackedResource<T> {
    pub marker: PhantomData<T>,
}

impl<T> TrackedResource<T> {
    pub(crate) fn new() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl<T> AnyResource for TrackedResource<T>
where
    T: Resource,
{
    fn is_changed(&self, world: &World) -> bool {
        world.is_resource_changed::<T>()
    }
}

/// Run reactions whose dependencies have changed.
pub fn run_reactions(world: &mut World) {
    let mut scopes = world.query::<(Entity, &mut TrackingScope)>();
    let mut changed = HashSet::<Entity>::default();
    for (entity, scope) in scopes.iter(world) {
        if scope.dependencies_changed(world) {
            changed.insert(entity);
        }
    }

    let tick = world.change_tick();
    for scope_entity in changed.iter() {
        let mut next_scope = TrackingScope::new(tick);
        let mut entt = world.entity_mut(*scope_entity);
        if let Some(view_handle) = entt.get_mut::<ViewHandle>() {
            let inner = view_handle.view.clone();
            let mut vc = ViewContext {
                world,
                owner: Some(*scope_entity),
            };
            inner
                .lock()
                .unwrap()
                .react(*scope_entity, &mut vc, &mut next_scope);
        } else if let Some(reaction) = entt.get_mut::<ReactionHandle>() {
            let inner = reaction.0.clone();
            inner
                .lock()
                .unwrap()
                .react(*scope_entity, world, &mut next_scope);
        }
        if let Ok((_, mut scope)) = scopes.get_mut(world, *scope_entity) {
            // Swap the scopes so that the next scope becomes the current scope.
            // The old scopes will be dropped at the end of the loop block.
            scope.swap_deps(&mut next_scope);
            scope.tick = tick;
        }
    }
}
