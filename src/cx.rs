use std::{
    cell::RefCell,
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use bevy::prelude::*;

use crate::{
    callback::{Callback, CallbackFnCell, CallbackFnMutCell},
    derived::{Derived, DerivedCell, ReadDerived, ReadDerivedInternal},
    mutable::{MutableCell, MutableNextCell, ReadMutable, WriteMutable},
    tracking_scope::TrackingScope,
    Mutable, Reaction, ReactionHandle, Signal,
};

/// An immutable reactive context, used for reactive closures such as derived signals.
pub trait RunContextRead {
    /// Return a reference to the resource of the given type. Calling this function
    /// adds the resource as a dependency of the current tracking scope.
    fn use_resource<T: Resource>(&self) -> &T;

    /// Return a reference to the Component `C` on the given entity. Calling this function
    /// adds the component as a dependency of the current tracking scope.
    fn use_component<C: Component>(&self, entity: Entity) -> Option<&C>;
}

/// A mutable reactive context. This allows write access to reactive data sources.
pub trait RunContextWrite: RunContextRead {
    /// The current Bevy [`World`].
    fn world_mut(&mut self) -> &mut World;

    /// Invoke a callback with the given props.
    ///
    /// Arguments:
    /// * `callback` - The callback to invoke.
    /// * `props` - The props to pass to the callback.
    fn run_callback<P: 'static>(&mut self, callback: Callback<P>, props: P) {
        let world = self.world_mut();
        let tick = world.read_change_tick();
        let mut tracking = TrackingScope::new(tick);
        let mut cx = Cx::new(props, world, &mut tracking);
        let mut callback_entity = cx.world.entity_mut(callback.id);
        if let Some(mut cell) = callback_entity.get_mut::<CallbackFnCell<P>>() {
            let mut callback_fn = cell.inner.take();
            let callback_box = callback_fn.as_ref().expect("Callback is not present");
            callback_box.call(&mut cx);
            let mut callback_entity = cx.world.entity_mut(callback.id);
            callback_entity
                .get_mut::<CallbackFnCell<P>>()
                .unwrap()
                .inner = callback_fn.take();
        } else if let Some(mut cell) = callback_entity.get_mut::<CallbackFnMutCell<P>>() {
            let mut callback_fn = cell.inner.take();
            let callback_box = callback_fn.as_mut().expect("Callback is not present");
            callback_box.call(&mut cx);
            let mut callback_entity = cx.world.entity_mut(callback.id);
            callback_entity
                .get_mut::<CallbackFnMutCell<P>>()
                .unwrap()
                .inner = callback_fn.take();
        } else {
            warn!("No callback found for {:?}", callback.id);
        }
    }
}

/// A "setup context" is similar to a reactive context, but can also be used to create
/// reactive data sources such as mutables and effects.
pub trait RunContextSetup<'p> {
    /// The current Bevy [`World`].
    fn world_mut(&mut self) -> &mut World;

    /// Add an owned entity to this tracking scope.
    fn add_owned(&self, mutable: Entity);

    /// Create a new [`Mutable`] in this context.
    fn create_mutable<T>(&mut self, init: T) -> Mutable<T>
    where
        T: Send + Sync + 'static,
    {
        let mutable = self.world_mut().spawn(MutableCell(Box::new(init))).id();
        self.add_owned(mutable);
        Mutable {
            id: mutable,
            marker: PhantomData,
        }
    }

    /// Create a new [`Callback`] in this context. This holds a `Fn` within an entity.
    ///
    /// Arguments:
    /// * `callback` - The callback function to invoke. This will be called with a single
    ///    parameter, which is a [`Cx`] object. The context may or may not have props.
    fn create_callback<P: 'static, F: Send + Sync + 'static + Fn(&mut Cx<P>)>(
        &mut self,
        callback: F,
    ) -> Callback<P> {
        let callback = self
            .world_mut()
            .spawn(CallbackFnCell::<P> {
                inner: Some(Box::new(callback)),
            })
            .id();
        self.add_owned(callback);
        Callback {
            id: callback,
            marker: PhantomData,
        }
    }

    /// Create a new [`CallbackFnMut`] in this context. This holds a `FnMut` within an entity.
    ///
    /// Arguments:
    /// * `callback` - The callback function to invoke. This will be called with a single
    ///    parameter, which is a [`Cx`] object. The context may or may not have props.
    fn create_callback_mut<P: 'static, F: FnMut(&mut Cx<P>)>(&mut self, callback: F) -> Callback<P>
    where
        F: Send + Sync + 'static,
    {
        let callback = self
            .world_mut()
            .spawn(CallbackFnMutCell {
                inner: Some(Box::new(callback)),
            })
            .id();
        self.add_owned(callback);
        Callback {
            id: callback,
            marker: PhantomData,
        }
    }

    /// Create a new [`Derived`] in this context. This represents a readable signal which
    /// is computed from other signals. The result is not memoized, but is recomputed whenever
    /// the dependencies change.
    ///
    /// Arguments:
    /// * `compute` - The function that computes the output. This will be called with a single
    ///    parameter, which is a [`Cx`] object.
    fn create_derived<R: 'static, F: Send + Sync + 'static + Fn(&mut Rcx) -> R>(
        &mut self,
        compute: F,
    ) -> Signal<R> {
        let derived = self
            .world_mut()
            .spawn(DerivedCell::<R>(Arc::new(compute)))
            .id();
        self.add_owned(derived);
        Signal::Derived(Derived {
            id: derived,
            marker: PhantomData,
        })
    }

    // /// Create a new [`Memo`] in this context. This represents a readable signal which
    // /// is computed from other signals. The result is memoized, which means that downstream
    // /// dependants will not be notified unless the output changes.
    // ///
    // /// Arguments:
    // /// * `compute` - The function that computes the output. This will be called with a single
    // ///    parameter, which is a [`Cx`] object.
    // fn create_memo<R: 'static, F: Send + Sync + 'static + Fn(&mut Rcx) -> R>(
    //     &mut self,
    //     compute: F,
    // ) -> Signal<R> {
    //     let ticks = self.world_mut().read_change_tick();
    //     let derived = self
    //         .world_mut()
    //         .spawn((
    //             TrackingScope::new(ticks),
    //             MutableCell(Box::new(init)),
    //             DerivedCell::<R>(Arc::new(compute)),
    //         ))
    //         .id();
    //     self.add_owned(derived);
    //     Signal::Derived(Derived {
    //         id: derived,
    //         marker: PhantomData,
    //     })
    // }

    /// Create an effect. This is a function that is associated with an entity, and which
    /// re-runs whenever any of it's dependencies change.
    ///
    /// Arguments:
    /// * `effect` - The function that computes the output. This will be called with a single
    ///    parameter, which is a [`Cx`] object.
    fn create_effect<F: Send + Sync + 'static + FnMut(&mut Cx<()>)>(&mut self, effect: F) {
        let ticks = self.world_mut().read_change_tick();
        let action = Arc::new(Mutex::new(effect));
        let mut scope = TrackingScope::new(ticks);
        action.lock().unwrap()(&mut Cx::new((), self.world_mut(), &mut scope));
        let entity = self
            .world_mut()
            .spawn((scope, ReactionHandle(action.clone())))
            .id();
        self.add_owned(entity);
    }

    // /// Register a cleanup function for the current tracking scope. This will be called when
    // /// the scope is dropped, or (if it's an effect) just before the effect is re-run.
    // ///
    // /// Arguments:
    // /// * `cleanup` - The function that performs the cleanup.
    // fn on_cleanup<F: Send + Sync + 'static + FnOnce(&mut Cx<()>)>(&mut self, cleanup: F) {
    //     let cleanup_action = Arc::new(Mutex::new(cleanup));
    //     self.add_cleanup(cleanup_action);
    // }
}

impl<F: Send + Sync + 'static + FnMut(&mut Cx<()>)> Reaction for F {
    fn react(&mut self, _owner: Entity, world: &mut World, tracking: &mut TrackingScope) {
        let mut cx = Cx::new((), world, tracking);
        (self)(&mut cx);
    }
}

/// Cx is a context parameter that is passed to presenters and callbacks. It contains the
/// presenter's properties (passed from the parent presenter), plus a reactive scope and
/// access to reactive data sources in the world.
pub struct Cx<'p, 'w, Props = ()> {
    /// The properties that were passed to the presenter from it's parent.
    pub props: Props,

    /// Bevy World
    world: &'w mut World,

    /// Set of reactive resources referenced by the presenter.
    pub(crate) tracking: RefCell<&'p mut TrackingScope>,
}

impl<'p, 'w, Props> Cx<'p, 'w, Props> {
    pub(crate) fn new(props: Props, world: &'w mut World, tracking: &'p mut TrackingScope) -> Self {
        Self {
            props,
            world,
            tracking: RefCell::new(tracking),
        }
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

    // /// Return a reference to the Component `C` on the given entity.
    // pub fn use_component<C: Component>(&self, entity: Entity) -> Option<&C> {
    //     match self.bc.world.get_entity(entity) {
    //         Some(c) => {
    //             self.add_tracked_component::<C>(entity);
    //             c.get::<C>()
    //         }
    //         None => None,
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

    // /// Return a reference to the Component `C` on the entity that contains the current
    // /// presenter invocation.
    // pub fn use_view_component<C: Component>(&self) -> Option<&C> {
    //     self.add_tracked_component::<C>(self.bc.entity);
    //     self.bc.world.entity(self.bc.entity).get::<C>()
    // }

    // /// Return a reference to the entity that holds the current presenter invocation.
    // pub fn use_view_entity(&self) -> EntityRef<'_> {
    //     self.bc.world.entity(self.bc.entity)
    // }

    // /// Return a mutable reference to the entity that holds the current presenter invocation.
    // pub fn use_view_entity_mut(&mut self) -> EntityWorldMut<'_> {
    //     self.bc.world.entity_mut(self.bc.entity)
    // }

    // /// Create a scoped value. This can be used to pass data to child presenters.
    // /// The value is accessible by all child presenters.
    // pub fn define_scoped_value<T: Clone + Send + Sync + PartialEq + 'static>(
    //     &mut self,
    //     key: ScopedValueKey<T>,
    //     value: T,
    // ) {
    //     let mut ec = self.bc.world.entity_mut(self.bc.entity);
    //     match ec.get_mut::<ScopedValueMap>() {
    //         Some(mut ctx) => {
    //             if let Some(v) = ctx.0.get(&key.id()) {
    //                 // Don't update if value hasn't changed
    //                 if v.downcast_ref::<T>() == Some(&value) {
    //                     return;
    //                 }
    //             }
    //             ctx.0.insert(key.id(), Box::new(value));
    //         }
    //         None => {
    //             let mut map = ScopedValueMap::default();
    //             map.0.insert(key.id(), Box::new(value));
    //             ec.insert(map);
    //         }
    //     }
    // }

    // /// Retrieve the value of a context variable.
    // pub fn get_scoped_value<T: Clone + Send + Sync + 'static>(
    //     &self,
    //     key: ScopedValueKey<T>,
    // ) -> Option<T> {
    //     let mut entity = self.bc.entity;
    //     loop {
    //         let ec = self.bc.world.entity(entity);
    //         if let Some(ctx) = ec.get::<ScopedValueMap>() {
    //             if let Some(val) = ctx.0.get(&key.id()) {
    //                 let cid = self
    //                     .bc
    //                     .world
    //                     .component_id::<ScopedValueMap>()
    //                     .expect("ScopedValueMap component type is not registered");
    //                 self.tracking.borrow_mut().components.insert((entity, cid));
    //                 return val.downcast_ref::<T>().cloned();
    //             }
    //         }
    //         match ec.get::<Parent>() {
    //             Some(parent) => entity = **parent,
    //             _ => return None,
    //         }
    //     }
    // }

    // fn add_tracked_component<C: Component>(&self, entity: Entity) {
    //     let cid = self
    //         .bc
    //         .world
    //         .component_id::<C>()
    //         .expect("Unregistered component type");
    //     self.tracking.borrow_mut().components.insert((entity, cid));
    // }
}

impl<'p, 'w, Props> ReadMutable for Cx<'p, 'w, Props> {
    fn read_mutable<T>(&self, mutable: &Mutable<T>) -> T
    where
        T: Send + Sync + Copy + 'static,
    {
        self.tracking.borrow_mut().add_mutable(mutable.id);
        self.world.read_mutable(mutable)
    }

    fn read_mutable_clone<T>(&self, mutable: &Mutable<T>) -> T
    where
        T: Send + Sync + Clone + 'static,
    {
        self.tracking.borrow_mut().add_mutable(mutable.id);
        self.world.read_mutable_clone(mutable)
    }

    fn read_mutable_as_ref<T>(&self, mutable: &Mutable<T>) -> &T
    where
        T: Send + Sync + 'static,
    {
        self.tracking.borrow_mut().add_mutable(mutable.id);
        self.world.read_mutable_as_ref(mutable)
    }

    fn read_mutable_map<T, U, F: Fn(&T) -> U>(&self, mutable: &Mutable<T>, f: F) -> U
    where
        T: Send + Sync + 'static,
    {
        self.tracking.borrow_mut().add_mutable(mutable.id);
        self.world.read_mutable_map(mutable, f)
    }
}

impl<'p, 'w, Props> WriteMutable for Cx<'p, 'w, Props> {
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

impl<'p, 'w, Props> ReadDerived for Cx<'p, 'w, Props> {
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

impl<'p, 'w, Props> RunContextRead for Cx<'p, 'w, Props> {
    fn use_resource<T: Resource>(&self) -> &T {
        self.tracking.borrow_mut().track_resource::<T>(self.world);
        self.world.resource::<T>()
    }

    fn use_component<C: Component>(&self, entity: Entity) -> Option<&C> {
        self.tracking
            .borrow_mut()
            .track_component::<C>(entity, self.world);
        self.world.entity(entity).get::<C>()
    }
}

impl<'p, 'w, Props> RunContextWrite for Cx<'p, 'w, Props> {
    fn world_mut(&mut self) -> &mut World {
        self.world
    }
}

impl<'p, 'w, Props> RunContextSetup<'p> for Cx<'p, 'w, Props> {
    fn world_mut(&mut self) -> &mut World {
        self.world
    }

    fn add_owned(&self, entity: Entity) {
        self.tracking.borrow_mut().add_owned(entity);
    }
}

/// Immutable reactive context, used for reactive closures such as derived signals.
/// This is a stripped down version of [`Cx`] that does not allow creating new reactions,
/// and which has no parameters.
pub struct Rcx<'p, 'w> {
    /// Bevy World
    pub(crate) world: &'w World,

    /// Set of reactive resources referenced by the presenter.
    pub(crate) tracking: RefCell<&'p mut TrackingScope>,
}

impl<'p, 'w> Rcx<'p, 'w> {
    pub(crate) fn new(world: &'w World, tracking: &'p mut TrackingScope) -> Self {
        Self {
            world,
            tracking: RefCell::new(tracking),
        }
    }
}

impl<'p, 'w> ReadMutable for Rcx<'p, 'w> {
    fn read_mutable<T>(&self, mutable: &Mutable<T>) -> T
    where
        T: Send + Sync + Copy + 'static,
    {
        self.tracking.borrow_mut().add_mutable(mutable.id);
        self.world.read_mutable(mutable)
    }

    fn read_mutable_clone<T>(&self, mutable: &Mutable<T>) -> T
    where
        T: Send + Sync + Clone + 'static,
    {
        self.tracking.borrow_mut().add_mutable(mutable.id);
        self.world.read_mutable_clone(mutable)
    }

    fn read_mutable_as_ref<T>(&self, mutable: &Mutable<T>) -> &T
    where
        T: Send + Sync + 'static,
    {
        self.tracking.borrow_mut().add_mutable(mutable.id);
        self.world.read_mutable_as_ref(mutable)
    }

    fn read_mutable_map<T, U, F: Fn(&T) -> U>(&self, mutable: &Mutable<T>, f: F) -> U
    where
        T: Send + Sync + 'static,
    {
        self.tracking.borrow_mut().add_mutable(mutable.id);
        self.world.read_mutable_map(mutable, f)
    }
}

impl<'p, 'w> ReadDerived for Rcx<'p, 'w> {
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

impl<'p, 'w> RunContextRead for Rcx<'p, 'w> {
    fn use_resource<T: Resource>(&self) -> &T {
        self.tracking.borrow_mut().track_resource::<T>(self.world);
        self.world.resource::<T>()
    }

    fn use_component<C: Component>(&self, entity: Entity) -> Option<&C> {
        self.tracking
            .borrow_mut()
            .track_component::<C>(entity, self.world);
        self.world.entity(entity).get::<C>()
    }
}

impl ReadMutable for World {
    fn read_mutable<T>(&self, mutable: &Mutable<T>) -> T
    where
        T: Send + Sync + Copy + 'static,
    {
        let mutable_entity = self.entity(mutable.id);
        *mutable_entity
            .get::<MutableCell>()
            .unwrap()
            .0
            .downcast_ref::<T>()
            .unwrap()
    }

    fn read_mutable_clone<T>(&self, mutable: &Mutable<T>) -> T
    where
        T: Send + Sync + Clone + 'static,
    {
        let mutable_entity = self.entity(mutable.id);
        mutable_entity
            .get::<MutableCell>()
            .unwrap()
            .0
            .downcast_ref::<T>()
            .unwrap()
            .clone()
    }

    fn read_mutable_as_ref<T>(&self, mutable: &Mutable<T>) -> &T
    where
        T: Send + Sync + 'static,
    {
        let mutable_entity = self.entity(mutable.id);
        mutable_entity
            .get::<MutableCell>()
            .unwrap()
            .0
            .downcast_ref::<T>()
            .unwrap()
    }

    fn read_mutable_map<T, U, F: Fn(&T) -> U>(&self, mutable: &Mutable<T>, f: F) -> U
    where
        T: Send + Sync + 'static,
    {
        let mutable_entity = self.entity(mutable.id);
        f(mutable_entity
            .get::<MutableCell>()
            .unwrap()
            .0
            .downcast_ref::<T>()
            .unwrap())
    }
}

impl WriteMutable for World {
    /// Write the value of a mutable variable using Copy semantics. Does nothing if
    /// the value being set matches the existing value.
    fn write_mutable<T>(&mut self, mutable: Entity, value: T)
    where
        T: Send + Sync + Copy + PartialEq + 'static,
    {
        let mut mutable_entity = self.entity_mut(mutable);
        if let Some(mut next) = mutable_entity.get_mut::<MutableNextCell>() {
            *next.0.as_mut().unwrap().downcast_mut::<T>().unwrap() = value;
        } else if let Some(current_value) = mutable_entity.get_mut::<MutableCell>() {
            if *current_value.0.downcast_ref::<T>().unwrap() != value {
                mutable_entity.insert(MutableNextCell(Some(Box::new(value))));
            }
        }
    }

    /// Write the value of a mutable variable using Clone semantics. Does nothing if the
    /// value being set matches the existing value.
    fn write_mutable_clone<T>(&mut self, mutable: Entity, value: T)
    where
        T: Send + Sync + Clone + PartialEq + 'static,
    {
        let mut mutable_entity = self.entity_mut(mutable);
        if let Some(mut next) = mutable_entity.get_mut::<MutableNextCell>() {
            *next.0.as_mut().unwrap().downcast_mut::<T>().unwrap() = value;
        } else if let Some(current_value) = mutable_entity.get_mut::<MutableCell>() {
            if *current_value.0.downcast_ref::<T>().unwrap() != value {
                mutable_entity.insert(MutableNextCell(Some(Box::new(value.clone()))));
            }
        }
    }
}

impl ReadDerived for World {
    fn read_derived<R>(&self, derived: &Derived<R>) -> R
    where
        R: Send + Sync + Copy + 'static,
    {
        let ticks = self.read_change_tick();
        let mut scope = TrackingScope::new(ticks);
        self.read_derived_with_scope(derived.id, &mut scope)
    }

    fn read_derived_clone<R>(&self, derived: &Derived<R>) -> R
    where
        R: Send + Sync + Clone + 'static,
    {
        let ticks = self.read_change_tick();
        let mut scope = TrackingScope::new(ticks);
        self.read_derived_clone_with_scope(derived.id, &mut scope)
    }

    fn read_derived_map<R, U, F: Fn(&R) -> U>(&self, derived: &Derived<R>, f: F) -> U
    where
        R: Send + Sync + 'static,
    {
        let ticks = self.read_change_tick();
        let mut scope = TrackingScope::new(ticks);
        self.read_derived_map_with_scope(derived.id, &mut scope, f)
    }
}

impl ReadDerivedInternal for World {
    fn read_derived_with_scope<R>(&self, derived: Entity, scope: &mut TrackingScope) -> R
    where
        R: Send + Sync + Copy + 'static,
    {
        let derived_entity = self.entity(derived);
        match derived_entity.get::<DerivedCell<R>>() {
            Some(cell) => {
                let derived_fn = cell.0.clone();
                let mut cx = Rcx::new(self, scope);
                derived_fn.call(&mut cx)
            }
            _ => panic!("No derived found for {:?}", derived),
        }
    }

    fn read_derived_clone_with_scope<R>(&self, derived: Entity, scope: &mut TrackingScope) -> R
    where
        R: Send + Sync + Clone + 'static,
    {
        let derived_entity = self.entity(derived);
        match derived_entity.get::<DerivedCell<R>>() {
            Some(cell) => {
                let derived_fn = cell.0.clone();
                let mut cx = Rcx::new(self, scope);
                derived_fn.call(&mut cx).clone()
            }
            _ => panic!("No derived found for {:?}", derived),
        }
    }

    fn read_derived_map_with_scope<R, U, F: Fn(&R) -> U>(
        &self,
        derived: Entity,
        scope: &mut TrackingScope,
        f: F,
    ) -> U
    where
        R: Send + Sync + 'static,
    {
        let derived_entity = self.entity(derived);
        match derived_entity.get::<DerivedCell<R>>() {
            Some(cell) => {
                let derived_fn = cell.0.clone();
                let mut cx = Rcx::new(self, scope);
                f(&derived_fn.call(&mut cx))
            }
            _ => panic!("No derived found for {:?}", derived),
        }
    }
}

impl RunContextRead for World {
    fn use_resource<T: Resource>(&self) -> &T {
        self.resource::<T>()
    }

    fn use_component<C: Component>(&self, entity: Entity) -> Option<&C> {
        self.entity(entity).get::<C>()
    }
}

impl RunContextWrite for World {
    fn world_mut(&mut self) -> &mut World {
        self
    }
}
