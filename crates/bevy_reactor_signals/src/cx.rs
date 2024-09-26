use std::{
    cell::RefCell,
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use bevy::{ecs::world::DeferredWorld, prelude::*};

use crate::{
    callback::Callback,
    derived::{Derived, DerivedCell, ReadDerived, ReadDerivedInternal},
    mutable::{MutableCell, ReadMutable, WriteMutable},
    tracking_scope::TrackingScope,
    Mutable, Rcx, Reaction, ReactionCell, Signal,
};

/// An immutable reactive context, used for reactive closures such as derived signals.
pub trait RunContextRead {
    /// Return a reference to the resource of the given type. Calling this function
    /// adds the resource as a dependency of the current tracking scope.
    fn read_resource<T: Resource>(&self) -> &T;

    /// Return a reference to the Component `C` on the given entity. Calling this function
    /// adds the component as a dependency of the current tracking scope.
    fn use_component<C: Component>(&self, entity: Entity) -> Option<&C>;

    /// Return a reference to the Component `C` on the given entity. Calling this function
    /// does not add the component as a dependency of the current tracking scope.
    fn use_component_untracked<C: Component>(&self, entity: Entity) -> Option<&C>;
}

/// A mutable reactive context. This allows write access to reactive data sources.
/// TODO: This is going away.
pub trait RunContextWrite: RunContextRead {
    /// The current Bevy [`World`].
    fn world_mut(&mut self) -> &mut World;
}

/// A "setup context" is similar to a reactive context, but can also be used to create
/// reactive data sources such as mutables and effects.
/// TODO: This is going away.
pub trait RunContextSetup<'p> {
    /// The current Bevy [`World`].
    fn world_mut(&mut self) -> &mut World;

    /// Add an owned entity to this tracking scope.
    fn add_owned(&self, mutable: Entity);

    /// Entity that owns this context.
    fn owner(&self) -> Entity;

    /// Set the debug name of the owner entity.
    fn set_owner_name(&mut self, name: &str) {
        let owner = self.owner();
        self.world_mut()
            .entity_mut(owner)
            .insert(Name::new(name.to_string()));
    }

    /// Create a new [`Mutable`] in this context.
    fn create_mutable<T>(&mut self, init: T) -> Mutable<T>
    where
        T: Send + Sync + 'static,
    {
        let owner = self.owner();
        let cell = self
            .world_mut()
            .spawn(MutableCell::<T>(init))
            .set_parent(owner)
            .id();
        let component = self.world_mut().init_component::<MutableCell<T>>();
        self.add_owned(cell);
        Mutable {
            cell,
            component,
            marker: PhantomData,
        }
    }

    /// Create a new [`Derived`] in this context. This represents a readable signal which
    /// is computed from other signals. The result is not memoized, but is recomputed whenever
    /// the dependencies change.
    ///
    /// Arguments:
    /// * `compute` - The function that computes the output. This will be called with a single
    ///    parameter, which is an [`Rcx`] object.
    fn create_derived<R: 'static, F: Send + Sync + 'static + Fn(&mut Rcx) -> R>(
        &mut self,
        compute: F,
    ) -> Signal<R> {
        let owner = self.owner();
        let derived = self
            .world_mut()
            .spawn(DerivedCell::<R>(Arc::new(compute)))
            .set_parent(owner)
            .id();
        self.add_owned(derived);
        Signal::Derived(Derived {
            id: derived,
            marker: PhantomData,
        })
    }

    /// Create a new [`Memo`] in this context. This represents a readable signal which
    /// is computed from other signals. The result is memoized, which means that downstream
    /// dependants will not be notified unless the output changes.
    ///
    /// Arguments:
    /// * `compute` - The function that computes the output. This will be called with a single
    ///    parameter, which is a [`Cx`] object.
    fn create_memo<
        R: 'static + PartialEq + Send + Sync + Clone,
        F: Send + Sync + 'static + Fn(&mut Cx) -> R,
    >(
        &mut self,
        compute: F,
    ) -> Signal<R> {
        let owner = self.owner();
        let ticks = self.world_mut().change_tick();
        let mut scope = TrackingScope::new(ticks);
        let init = compute(&mut Cx::new(self.world_mut(), owner, &mut scope));
        let mutable = self.create_mutable(init);
        let signal = mutable.signal();
        let reaction = Arc::new(Mutex::new(move |cx: &mut Cx| {
            let prev_value = mutable.get_clone(cx);
            let value = compute(cx);
            if value != prev_value {
                mutable.set_clone(cx, value);
            }
        }));
        self.world_mut().entity_mut(mutable.cell).insert((
            ReactionCell(reaction),
            scope,
            Name::new(format!("Memo::<{}>", std::any::type_name::<R>())),
        ));

        signal
    }

    /// Create an effect. This is a function that is associated with an entity, and which
    /// re-runs whenever any of it's dependencies change.
    ///
    /// Arguments:
    /// * `effect` - The function that computes the output. This will be called with a single
    ///    parameter, which is a [`Cx`] object.
    fn create_effect<F: Send + Sync + 'static + FnMut(&mut Cx)>(&mut self, effect: F) {
        let owner = self.owner();
        let ticks = self.world_mut().change_tick();
        let action = Arc::new(Mutex::new(effect));
        let mut scope = TrackingScope::new(ticks);
        let entity = self.world_mut().spawn_empty().set_parent(owner).id();
        self.add_owned(entity);
        action.lock().unwrap()(&mut Cx::new(self.world_mut(), entity, &mut scope));
        self.world_mut()
            .entity_mut(entity)
            .insert((scope, ReactionCell(action)));
    }
}

impl<F: Send + Sync + 'static + FnMut(&mut Cx)> Reaction for F {
    fn react(&mut self, owner: Entity, world: &mut World, tracking: &mut TrackingScope) {
        let mut cx = Cx::new(world, owner, tracking);
        (self)(&mut cx);
    }
}

/// Cx is a context parameter that is passed to presenters and callbacks. It contains the
/// presenter's properties (passed from the parent presenter), plus a reactive scope and
/// access to reactive data sources in the world.
/// TODO: This is going away.
pub struct Cx<'p, 'w> {
    /// Bevy World
    world: &'w mut World,

    /// The entity that owns the tracking scope (or will own it).
    owner: Entity,

    /// Set of reactive resources referenced by the presenter.
    pub(crate) tracking: RefCell<&'p mut TrackingScope>,
}

impl<'p, 'w> Cx<'p, 'w> {
    /// Construct a new reactive context.
    pub fn new(world: &'w mut World, owner: Entity, tracking: &'p mut TrackingScope) -> Self {
        Self {
            world,
            owner,
            tracking: RefCell::new(tracking),
        }
    }

    /// Access to world from reactive context.
    pub fn world(&self) -> &World {
        self.world
    }

    /// Access to mutable world from reactive context.
    pub fn world_mut(&mut self) -> &mut World {
        self.world
    }

    /// Spawn an empty [`Entity`]. The caller is responsible for despawning the entity.
    pub fn create_entity(&mut self) -> Entity {
        self.world_mut().spawn_empty().id()
    }

    /// Spawn an empty [`Entity`]. The entity will be despawned when the tracking scope is dropped.
    pub fn create_owned_entity(&mut self) -> Entity {
        let entity = self.world_mut().spawn_empty().id();
        self.tracking.borrow_mut().add_owned(entity);
        entity
    }

    /// Create a new callback in this context. This registers a one-shot system with the world.
    /// The callback will be unregistered when the tracking scope is dropped.
    ///
    /// Note: This function takes no deps argument, the callback is only registered once the first
    /// time it is called. Subsequent calls will return the original callback.
    pub fn create_callback<P: Send + Sync + 'static, M, S: IntoSystem<P, (), M> + 'static>(
        &mut self,
        callback: S,
    ) -> Callback<P> {
        let id = self.world_mut().register_system(callback);
        Callback { id }
    }

    // /// Return a reference to the Component `C` on the given entity. Adds the component to
    // /// the current tracking scope.
    // pub fn use_component<C: Component>(&self, entity: Entity) -> Option<&C> {
    //     let component = self
    //         .world
    //         .components()
    //         .component_id::<C>()
    //         .expect("Unknown component type");

    //     match self.world.get_entity(entity) {
    //         Some(c) => {
    //             self.tracking
    //                 .borrow_mut()
    //                 .track_component_id(entity, component);
    //             c.get::<C>()
    //         }
    //         None => None,
    //     }
    // }

    // /// Return a reference to the Component `C` on the given entity. Does not add the component
    // /// to the tracking scope.
    // pub fn use_component_untracked<C: Component>(&self, entity: Entity) -> Option<&C> {
    //     match self.world.get_entity(entity) {
    //         Some(c) => c.get::<C>(),
    //         None => None,
    //     }
    // }

    // /// Return a reference to the Component `C` on the owner entity of the current
    // /// context, or one of it's ancestors. This searches up the entity tree until it finds
    // /// a component of the given type.
    // pub fn use_inherited_component<C: Component>(&self) -> Option<&C> {
    //     let mut entity = self.owner;
    //     loop {
    //         let ec = self.use_component(entity);
    //         if ec.is_some() {
    //             return ec;
    //         }
    //         match self.world.entity(entity).get::<Parent>() {
    //             Some(parent) => entity = **parent,
    //             _ => return None,
    //         }
    //     }
    // }

    // /// Return a reference to the Component `C` on the given entity. This version does not
    // /// add the component to the tracking scope, and is intended for components that update
    // /// frequently.
    // pub fn use_component_untracked<C: Component>(&self, entity: Entity) -> Option<&C> {
    //     match self.bc.world.get_entity(entity) {
    //         Some(c) => c.get::<C>(),
    //         None => None,
    //     }
    // }

    /// Insert a component on the owner entity of the current context. This component can
    /// be accessed by this context any any child contexts via [`use_inherited_component`].
    pub fn insert(&mut self, component: impl Component) {
        let owner = self.owner;
        self.world_mut().entity_mut(owner).insert(component);
    }

    /// Add a cleanup function which is run once before the next reaction, or when the owner
    /// entity for this context is despawned.
    pub fn on_cleanup(&mut self, cleanup: impl FnOnce(&mut DeferredWorld) + Send + Sync + 'static) {
        self.tracking.borrow_mut().add_cleanup(cleanup);
    }
}

impl<'p, 'w> ReadMutable for Cx<'p, 'w> {
    fn read_mutable<T>(&self, mutable: &Mutable<T>) -> T
    where
        T: Send + Sync + Copy + 'static,
    {
        self.tracking
            .borrow_mut()
            .track_component_id(mutable.cell, mutable.component);
        self.world.read_mutable(mutable)
    }

    fn read_mutable_clone<T>(&self, mutable: &Mutable<T>) -> T
    where
        T: Send + Sync + Clone + 'static,
    {
        self.tracking
            .borrow_mut()
            .track_component_id(mutable.cell, mutable.component);
        self.world.read_mutable_clone(mutable)
    }

    fn read_mutable_as_ref<T>(&self, mutable: &Mutable<T>) -> &T
    where
        T: Send + Sync + 'static,
    {
        self.tracking
            .borrow_mut()
            .track_component_id(mutable.cell, mutable.component);
        self.world.read_mutable_as_ref(mutable)
    }

    fn read_mutable_map<T, U, F: Fn(&T) -> U>(&self, mutable: &Mutable<T>, f: F) -> U
    where
        T: Send + Sync + 'static,
    {
        self.tracking
            .borrow_mut()
            .track_component_id(mutable.cell, mutable.component);
        self.world.read_mutable_map(mutable, f)
    }
}

impl<'p, 'w> WriteMutable for Cx<'p, 'w> {
    fn write_mutable<T>(&mut self, mutable: Entity, value: T)
    where
        T: Send + Sync + Copy + PartialEq + 'static,
    {
        self.world.write_mutable(mutable, value);
    }

    fn write_mutable_clone<T>(&mut self, mutable: Entity, value: T)
    where
        T: Send + Sync + Clone + PartialEq + 'static,
    {
        self.world.write_mutable_clone(mutable, value);
    }
}

impl<'p, 'w> ReadDerived for Cx<'p, 'w> {
    fn read_derived<R>(&self, derived: &Derived<R>) -> R
    where
        R: Send + Sync + Copy + 'static,
    {
        self.world
            .read_derived_with_scope(derived.id, &mut self.tracking.borrow_mut())
    }

    fn read_derived_clone<R>(&self, derived: &Derived<R>) -> R
    where
        R: Send + Sync + Clone + 'static,
    {
        self.world
            .read_derived_clone_with_scope(derived.id, &mut self.tracking.borrow_mut())
    }

    fn read_derived_map<R, U, F: Fn(&R) -> U>(&self, derived: &Derived<R>, f: F) -> U
    where
        R: Send + Sync + 'static,
    {
        self.world
            .read_derived_map_with_scope(derived.id, &mut self.tracking.borrow_mut(), f)
    }
}

impl<'p, 'w> RunContextRead for Cx<'p, 'w> {
    fn read_resource<T: Resource>(&self) -> &T {
        self.tracking.borrow_mut().track_resource::<T>(self.world);
        self.world.resource::<T>()
    }

    fn use_component<C: Component>(&self, entity: Entity) -> Option<&C> {
        self.tracking
            .borrow_mut()
            .track_component::<C>(entity, self.world);
        self.world.entity(entity).get::<C>()
    }

    fn use_component_untracked<C: Component>(&self, entity: Entity) -> Option<&C> {
        self.world.entity(entity).get::<C>()
    }
}

impl<'p, 'w> RunContextWrite for Cx<'p, 'w> {
    fn world_mut(&mut self) -> &mut World {
        self.world
    }
}

impl<'p, 'w> RunContextSetup<'p> for Cx<'p, 'w> {
    fn world_mut(&mut self) -> &mut World {
        self.world
    }

    fn owner(&self) -> Entity {
        self.owner
    }

    fn add_owned(&self, entity: Entity) {
        self.tracking.borrow_mut().add_owned(entity);
    }
}

impl RunContextRead for World {
    fn read_resource<T: Resource>(&self) -> &T {
        self.resource::<T>()
    }

    fn use_component<C: Component>(&self, entity: Entity) -> Option<&C> {
        self.entity(entity).get::<C>()
    }

    fn use_component_untracked<C: Component>(&self, entity: Entity) -> Option<&C> {
        self.entity(entity).get::<C>()
    }
}

impl RunContextWrite for World {
    fn world_mut(&mut self) -> &mut World {
        self
    }
}
