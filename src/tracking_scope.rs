use std::any::TypeId;

use bevy::{
    ecs::component::{ComponentId, Tick},
    prelude::*,
    utils::{HashMap, HashSet},
};

use crate::{reaction::ReactionHandle, ViewHandle};

/// A component that tracks the dependencies of a reactive task.
#[derive(Component)]
pub struct TrackingScope {
    /// List of entities that are owned by this scope.
    owned: Vec<Entity>,

    /// Set of components that we are currently subscribed to.
    component_deps: HashSet<(Entity, ComponentId)>,

    /// Set of resources that we are currently subscribed to.
    resource_deps: HashMap<ComponentId, TrackedResource>,

    /// Engine tick used for determining if components have changed. This represents the
    /// time of the previous reaction.
    tick: Tick,
    // contexts
    // debug_name
    // cleanups
}

impl TrackingScope {
    /// Create a new tracking scope.
    pub fn new(tick: Tick) -> Self {
        Self {
            owned: Vec::new(),
            component_deps: HashSet::default(),
            resource_deps: HashMap::default(),
            tick,
        }
    }

    pub(crate) fn add_owned(&mut self, owned: Entity) {
        self.owned.push(owned);
    }

    fn add_resource<T: Resource>(&mut self, resource_id: ComponentId) {
        self.resource_deps
            .entry(resource_id)
            .or_insert_with(|| TrackedResource::new::<T>());
    }

    /// Convenience method for adding a resource dependency.
    pub(crate) fn track_resource<T: Resource>(&mut self, world: &World) {
        self.add_resource::<T>(
            world
                .components()
                .get_resource_id(TypeId::of::<T>())
                .expect("Unknown resource type"),
        );
    }

    /// Convenience method for adding a component dependency.
    pub(crate) fn track_component<C: Component>(&mut self, entity: Entity, world: &World) {
        self.track_component_id(
            entity,
            world
                .components()
                .component_id::<C>()
                .expect("Unknown component type"),
        );
    }

    /// Convenience method for adding a component dependency by component id.
    pub(crate) fn track_component_id(&mut self, entity: Entity, component: ComponentId) {
        self.component_deps.insert((entity, component));
    }

    /// Returns true if any of the dependencies of this scope have been updated since
    /// the previous reaction.
    fn dependencies_changed(&self, world: &World, tick: Tick) -> bool {
        self.components_changed(world, tick)
            || self.mutables_changed(world)
            || self.resource_deps.iter().any(|(_, c)| c.is_changed(world))
    }

    fn components_changed(&self, world: &World, tick: Tick) -> bool {
        self.component_deps.iter().any(|(e, c)| {
            world
                .entity(*e)
                .get_change_ticks_by_id(*c)
                .map(|ct| ct.is_changed(self.tick, tick))
                .unwrap_or(false)
        })
    }

    /// Take the dependencies from another scope. Typically the other scope is a temporary
    /// scope that is used to compute the next set of dependencies.
    pub(crate) fn take_deps(&mut self, other: &mut Self) {
        self.component_deps = std::mem::take(&mut other.component_deps);
        self.resource_deps = std::mem::take(&mut other.resource_deps);
    }
}

/// Trait which allows despawning of any owned objects or reactions in the tracking scope
/// associated with an entity. This operation is recursive in that an owned object may itself
/// own other objects.
pub trait DespawnScopes {
    /// Despawn all owned objects and reactions associated with the given entity.
    fn despawn_owned_recursive(&mut self, scope_entity: Entity);
}

impl DespawnScopes for World {
    fn despawn_owned_recursive(&mut self, scope_entity: Entity) {
        let mut entt = self.entity_mut(scope_entity);
        let Some(mut scope) = entt.get_mut::<TrackingScope>() else {
            return;
        };
        let owned_list = std::mem::take(&mut scope.owned);
        entt.despawn();
        for owned in owned_list {
            self.despawn_owned_recursive(owned);
        }
    }
}

pub struct TrackedResource {
    fn_is_changed: fn(&World) -> bool,
}

impl TrackedResource {
    pub(crate) fn new<T: Resource>() -> Self {
        Self {
            fn_is_changed: |world| world.is_resource_changed::<T>(),
        }
    }

    pub fn is_changed(&self, world: &World) -> bool {
        (self.fn_is_changed)(world)
    }
}

/// Run reactions whose dependencies have changed.
pub fn run_reactions(world: &mut World) {
    let mut scopes = world.query::<(Entity, &mut TrackingScope)>();
    let mut changed = HashSet::<Entity>::default();
    let tick = world.change_tick();
    for (entity, scope) in scopes.iter(world) {
        if scope.dependencies_changed(world, tick) {
            changed.insert(entity);
        }
    }

    for scope_entity in changed.iter() {
        let mut next_scope = TrackingScope::new(tick);
        if let Some(mut entt) = world.get_entity_mut(*scope_entity) {
            if let Some(view_handle) = entt.get_mut::<ViewHandle>() {
                let inner = view_handle.0.clone();
                inner
                    .lock()
                    .unwrap()
                    .react(*scope_entity, world, &mut next_scope);
            } else if let Some(reaction) = entt.get_mut::<ReactionHandle>() {
                let inner = reaction.0.clone();
                inner
                    .lock()
                    .unwrap()
                    .react(*scope_entity, world, &mut next_scope);
            }
        }
        if let Ok((_, mut scope)) = scopes.get_mut(world, *scope_entity) {
            // Swap the scopes so that the next scope becomes the current scope.
            // The old scopes will be dropped at the end of the loop block.
            scope.take_deps(&mut next_scope);
            scope.tick = tick;
        }
    }
}
