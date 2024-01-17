use std::sync::{atomic::Ordering, Arc, Mutex};

use bevy::{
    ecs::component::{ComponentId, Tick},
    prelude::*,
    utils::HashSet,
};

use crate::{mutable::MutableValue, Cx};

/// A component that tracks the dependencies of a reactive task.
#[derive(Component)]
pub struct TrackingScope {
    /// Scope that owns this scope. This scope will be dropped when the owner is dropped.
    owner: Option<Entity>,

    /// List of scopes that are owned by this scope.
    owned: Vec<Entity>,

    /// The set of mutables that this scope is subscribed to.
    mutable_deps: HashSet<Entity>,

    /// Set of components that we are currently subscribed to.
    component_deps: HashSet<(Entity, ComponentId)>,

    /// Set of resources that we are currently subscribed to.
    resource_deps: HashSet<ComponentId>,

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
            owner: None,
            owned: Vec::new(),
            mutable_deps: HashSet::default(),
            component_deps: HashSet::default(),
            resource_deps: HashSet::default(),
            tick,
        }
    }

    pub fn for_owner(tick: Tick, owner: Entity) -> Self {
        Self {
            owner: Some(owner),
            owned: Vec::new(),
            mutable_deps: HashSet::default(),
            component_deps: HashSet::default(),
            resource_deps: HashSet::default(),
            tick,
        }
    }

    pub(crate) fn add_owned(&mut self, owned: Entity) {
        self.owned.push(owned);
    }

    pub(crate) fn add_mutable(&mut self, mutable: Entity) {
        self.mutable_deps.insert(mutable);
    }

    pub(crate) fn add_resource(&mut self, resource_id: ComponentId) {
        self.resource_deps.insert(resource_id);
    }

    /// Reset all reactive dependencies, in preparation for a new reaction.
    fn reset(&mut self) {
        self.component_deps.clear();
        self.resource_deps.clear();
        self.mutable_deps.clear();
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
        })
    }
}

#[derive(Component)]
#[allow(clippy::type_complexity)]
pub struct Reaction {
    /// Effect function
    inner: Arc<dyn Fn(&mut Cx) + Send + Sync>,
}

pub fn run_reactions(world: &mut World) {
    let mut scopes = world.query::<(Entity, &mut TrackingScope, Option<&mut Reaction>)>();
    let mut changed = HashSet::<Entity>::default();
    for (entity, scope, _) in scopes.iter(world) {
        if scope.dependencies_changed(world) {
            changed.insert(entity);
        }
    }

    let tick = world.change_tick();
    for scope_entity in changed.iter() {
        let mut next_scope = TrackingScope::new(tick);
        if let Ok((_, _, Some(reaction))) = scopes.get_mut(world, *scope_entity) {
            let inner = reaction.inner.clone();
            let mut cx = Cx::new(&(), world, &mut next_scope);
            (inner)(&mut cx);
        }
        if let Ok((_, mut scope, _)) = scopes.get_mut(world, *scope_entity) {
            // TODO: Find a way to swap rather than clone.
            scope.mutable_deps.clone_from(&next_scope.mutable_deps);
            scope.component_deps.clone_from(&next_scope.component_deps);
            scope.resource_deps.clone_from(&next_scope.resource_deps);
        }
    }
}
