use bevy::{
    ecs::{
        component::{ComponentId, Tick},
        world::{Command, DeferredWorld},
    },
    prelude::*,
    utils::HashSet,
};

use crate::ReactionCell;

/// A component that tracks the dependencies of a reactive task.
#[derive(Component)]
pub struct TrackingScope {
    /// List of entities that are owned by this scope.
    pub(crate) owned: Vec<Entity>,

    /// Set of components that we are currently subscribed to.
    component_deps: HashSet<(Entity, ComponentId)>,

    /// Set of resources that we are currently subscribed to.
    resource_deps: HashSet<ComponentId>,

    /// Engine tick used for determining if components have changed. This represents the
    /// time of the previous reaction.
    pub tick: Tick,

    /// List of cleanup functions to call when the scope is dropped.
    #[allow(clippy::type_complexity)]
    pub cleanups: Vec<Box<dyn FnOnce(&mut DeferredWorld) + 'static + Sync + Send>>,
}

/// A resource which, if inserted, displays the view entities that have reacted this frame.
#[derive(Resource)]
pub struct TrackingScopeTracing(pub Vec<Entity>);

impl FromWorld for TrackingScopeTracing {
    fn from_world(_world: &mut World) -> Self {
        Self(Vec::new())
    }
}

impl TrackingScope {
    /// Create a new tracking scope.
    pub fn new(tick: Tick) -> Self {
        Self {
            owned: Vec::new(),
            component_deps: HashSet::default(),
            resource_deps: HashSet::default(),
            tick,
            cleanups: Vec::new(),
        }
    }

    /// Add an entity which is owned by this scope. When the scope is dropped, the entity
    /// will be despawned.
    pub fn add_owned(&mut self, owned: Entity) {
        self.owned.push(owned);
    }

    /// Add a cleanup function which will be run once before the next reaction.
    pub fn add_cleanup(
        &mut self,
        cleanup: impl FnOnce(&mut DeferredWorld) + 'static + Sync + Send,
    ) {
        self.cleanups.push(Box::new(cleanup));
    }

    /// Convenience method for adding a resource dependency.
    pub fn track_resource<T: Resource>(&mut self, world: &World) {
        self.resource_deps.insert(
            world
                .components()
                .resource_id::<T>()
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
    pub fn dependencies_changed(&self, world: &World, tick: Tick) -> bool {
        self.components_changed(world, tick) || self.resources_changed(world, tick)
    }

    fn components_changed(&self, world: &World, tick: Tick) -> bool {
        self.component_deps.iter().any(|(e, c)| {
            world.get_entity(*e).map_or(false, |e| {
                e.get_change_ticks_by_id(*c)
                    .map(|ct| ct.is_changed(self.tick, tick))
                    .unwrap_or(false)
            })
        })
    }

    fn resources_changed(&self, world: &World, tick: Tick) -> bool {
        self.resource_deps.iter().any(|c| {
            world
                .get_resource_change_ticks_by_id(*c)
                .map(|ct| ct.is_changed(self.tick, tick))
                .unwrap_or(false)
        })
    }

    /// Take the dependencies from another scope. Typically the other scope is a temporary
    /// scope that is used to compute the next set of dependencies.
    pub fn take_deps(&mut self, other: &mut Self) {
        self.component_deps = std::mem::take(&mut other.component_deps);
        self.resource_deps = std::mem::take(&mut other.resource_deps);
        self.cleanups = std::mem::take(&mut other.cleanups);
    }
}

pub(crate) fn cleanup_tracking_scopes(world: &mut World) {
    world
        .register_component_hooks::<TrackingScope>()
        .on_remove(|mut world, entity, _component| {
            let mut scope = world.get_mut::<TrackingScope>(entity).unwrap();
            let mut cleanups = std::mem::take(&mut scope.cleanups);
            let mut owned = std::mem::take(&mut scope.owned);
            // let mut hooks = std::mem::take(&mut scope.hook_states);
            for cleanup_fn in cleanups.drain(..) {
                cleanup_fn(&mut world);
            }
            for ent in owned.drain(..) {
                world.commands().queue(DespawnEntityCmd(ent));
            }
            // for hook in hooks.drain(..).rev() {
            //     match hook {
            //         HookState::Entity(ent) => {
            //             world.commands().add(DespawnEntityCmd(ent));
            //         }
            //         HookState::Mutable(mutable_ent, _) => {
            //             world.commands().add(DespawnEntityCmd(mutable_ent));
            //         }
            //         HookState::Callback(callback) => {
            //             world.commands().add(UnregisterCallbackCmd(callback));
            //         }
            //         HookState::Effect(_) | HookState::Memo(_) => {
            //             // Nothing to do
            //         }
            //     }
            // }
        });
}

struct DespawnEntityCmd(Entity);

impl Command for DespawnEntityCmd {
    fn apply(self, world: &mut World) {
        world.entity_mut(self.0).remove_parent();
        world.despawn(self.0);
    }
}

fn run_cleanups(world: &mut World, changed: &[Entity]) {
    let mut deferred = DeferredWorld::from(world);
    for scope_entity in changed.iter() {
        let Some(mut scope) = deferred.get_mut::<TrackingScope>(*scope_entity) else {
            continue;
        };
        let mut cleanups = std::mem::take(&mut scope.cleanups);
        for cleanup_fn in cleanups.drain(..) {
            cleanup_fn(&mut deferred);
        }
    }
}

/// Run reactions whose dependencies have changed.
pub(crate) fn run_reactions(world: &mut World) {
    let mut scopes = world.query::<(Entity, &mut TrackingScope, &ReactionCell)>();
    let mut changed: Vec<Entity> = Vec::with_capacity(64);
    let tick = world.change_tick();
    for (entity, scope, _) in scopes.iter(world) {
        if scope.dependencies_changed(world, tick) {
            changed.push(entity);
        }
    }

    // Record the changed entities for debugging purposes.
    if let Some(mut tracing) = world.get_resource_mut::<TrackingScopeTracing>() {
        // Check for empty first to avoid setting mutation flag.
        if !tracing.0.is_empty() {
            tracing.0.clear();
        }
        if !changed.is_empty() {
            tracing.0.extend(changed.iter().copied());
        }
    }

    run_cleanups(world, &changed);

    for scope_entity in changed.iter() {
        // Call registered cleanup functions
        // let (_, mut scope, _) = scopes.get_mut(world, *scope_entity).unwrap();
        // let mut cleanups = std::mem::take(&mut scope.cleanups);
        // for cleanup_fn in cleanups.drain(..) {
        //     cleanup_fn(world);
        // }

        // Run the reaction
        // let (_, _, thunk) = scopes.get_mut(world, *scope_entity).unwrap();
        let thunk = world.entity(*scope_entity).get::<ReactionCell>().unwrap();
        let mut next_scope = TrackingScope::new(tick);
        let inner = thunk.0.clone();
        let mut lock = inner.lock().unwrap();
        lock.react(*scope_entity, world, &mut next_scope);

        // Replace deps and cleanups in the current scope with the next scope.
        let (_, mut scope, _) = scopes.get_mut(world, *scope_entity).unwrap();
        scope.take_deps(&mut next_scope);
        scope.tick = tick;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource, Default)]
    struct TestResource(bool);

    #[test]
    fn test_resource_deps_changed() {
        let mut world = World::default();
        let tick = world.change_tick();
        let mut scope = TrackingScope::new(tick);

        // No dependencies, so the result should be false
        assert!(!scope.dependencies_changed(&world, tick));

        world.increment_change_tick();
        world.insert_resource(TestResource(false));
        scope.track_resource::<TestResource>(&world);
        assert!(scope.resource_deps.len() == 1);

        // Resource added
        let tick = world.change_tick();
        assert!(scope.dependencies_changed(&world, tick));

        // Reset scope tick
        scope.tick = tick;
        assert!(!scope.dependencies_changed(&world, tick));

        // Mutate the resource
        world.increment_change_tick();
        world.get_resource_mut::<TestResource>().unwrap().0 = true;
        let tick = world.change_tick();
        assert!(scope.dependencies_changed(&world, tick));
    }
}
