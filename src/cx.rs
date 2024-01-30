use std::{any::TypeId, cell::RefCell, marker::PhantomData};

use bevy::prelude::*;

use crate::{
    callback::{CallbackFn, CallbackFnMutValue, CallbackFnValue},
    mutable::{MutableValue, MutableValueNext},
    scope::TrackingScope,
    Mutable,
};

/// An immutable reactive context, used for reactive closures such as derived signals.
pub trait ReactiveContext<'p> {
    /// The current Bevy [`World`].
    fn world(&self) -> &World;

    /// Read the value of a mutable variable using Copy semantics. Calling this function adds the
    /// mutable to the current tracking scope.
    fn read_mutable<T>(&self, mutable: Entity) -> T
    where
        T: Send + Sync + Copy + 'static;

    /// Read the value of a mutable variable using Clone semantics. Calling this function adds the
    /// mutable to the current tracking scope.
    fn read_mutable_clone<T>(&self, mutable: Entity) -> T
    where
        T: Send + Sync + Clone + 'static;

    /// Return a reference to the resource of the given type. Calling this function
    /// adds the resource as a dependency of the current presenter invocation.
    fn use_resource<T: Resource>(&self) -> &T;
}

/// A mutable reactive context. This allows write access to reactive data sources.
pub trait ReactiveContextMut<'p>: ReactiveContext<'p> {
    /// The current Bevy [`World`].
    fn world_mut(&mut self) -> &mut World;

    /// Write the value of a mutable variable using Copy semantics. Does nothing if
    /// the value being set matches the existing value.
    fn write_mutable<T>(&mut self, mutable: Entity, value: T)
    where
        T: Send + Sync + Copy + PartialEq + 'static,
    {
        let mut mutable_entity = self.world_mut().entity_mut(mutable);
        if let Some(mut next) = mutable_entity.get_mut::<MutableValueNext>() {
            *next.0.downcast_mut::<T>().unwrap() = value;
        } else if let Some(current_value) = mutable_entity.get_mut::<MutableValue>() {
            if *current_value.value.downcast_ref::<T>().unwrap() != value {
                mutable_entity.insert(MutableValueNext(Box::new(value)));
            }
        }
    }

    /// Write the value of a mutable variable using Clone semantics. Does nothing if the
    /// value being set matches the existing value.
    fn write_mutable_clone<T>(&mut self, mutable: Entity, value: T)
    where
        T: Send + Sync + Clone + PartialEq + 'static,
    {
        let mut mutable_entity = self.world_mut().entity_mut(mutable);
        if let Some(mut next) = mutable_entity.get_mut::<MutableValueNext>() {
            *next.0.downcast_mut::<T>().unwrap() = value;
        } else if let Some(current_value) = mutable_entity.get_mut::<MutableValue>() {
            if *current_value.value.downcast_ref::<T>().unwrap() != value {
                mutable_entity.insert(MutableValueNext(Box::new(value.clone())));
            }
        }
    }

    /// Write the value of a mutable variable by modifying in place. Note that unlike the
    /// other versions, this function does not check for equality before updating the value,
    /// and always triggers change detection / reactions.
    fn write_mutable_ref<T, F: FnMut(&T)>(&mut self, mutable: Entity, mut mutator: F)
    where
        T: Send + Sync + Clone + PartialEq + 'static,
    {
        let mut mutable_entity = self.world_mut().entity_mut(mutable);
        if let Some(mut next) = mutable_entity.get_mut::<MutableValueNext>() {
            mutator(next.0.downcast_mut::<T>().unwrap());
        } else if let Some(mut current_value) = mutable_entity.get_mut::<MutableValue>() {
            mutator(current_value.value.downcast_mut::<T>().unwrap());
        }
    }

    /// Invoke a callback with the given props.
    ///
    /// Arguments:
    /// * `callback` - The callback to invoke.
    /// * `props` - The props to pass to the callback.
    fn run_callback<P: 'static>(&mut self, callback: CallbackFn<P>, props: P) {
        let world = self.world_mut();
        let tick = world.change_tick();
        let mut tracking = TrackingScope::new(tick);
        let mut cx = Cx::new(&props, world, &mut tracking);
        let mut callback_entity = cx.world.entity_mut(callback.id);
        if let Some(mut callback_cmp) = callback_entity.get_mut::<CallbackFnValue<P>>() {
            let mut callback_fn = callback_cmp.inner.take();
            let callback_box = callback_fn.as_ref().expect("CallbackFn is not present");
            callback_box.call(&mut cx);
            let mut callback_entity = cx.world.entity_mut(callback.id);
            callback_entity
                .get_mut::<CallbackFnValue<P>>()
                .unwrap()
                .inner = callback_fn.take();
        } else if let Some(mut callback_cmp) = callback_entity.get_mut::<CallbackFnMutValue<P>>() {
            let mut _callback_fn = callback_cmp.inner.take();
            todo!("Mutable callbacks");
        } else {
            warn!("No callback found for {:?}", callback.id);
        }
    }
}

/// A "setup context" is similar to a reactive context, but can also be used to create
/// reactive data sources such as mutables and effects.
pub trait SetupContext<'p> {
    /// The current Bevy [`World`].
    fn world_mut(&mut self) -> &mut World;

    /// Add an owned entity to this tracking scope.
    fn add_owned(&self, mutable: Entity);

    /// Create a new [`Mutable`] in this context.
    fn create_mutable<T>(&mut self, init: T) -> Mutable<T>
    where
        T: Send + Sync + 'static,
    {
        let mutable = self
            .world_mut()
            .spawn((MutableValue {
                value: Box::new(init),
            },))
            .id();
        self.add_owned(mutable);
        Mutable {
            id: mutable,
            marker: PhantomData,
        }
    }

    /// Create a new [`CallbackFn`] in this context. This holds a `Fn` within an entity.
    ///
    /// Arguments:
    /// * `callback` - The callback function to invoke. This will be called with a single
    ///    parameter, which is a [`Cx`] object. The context may or may not have props.
    fn create_callback<P: 'static, F: Send + Sync + 'static + Fn(&mut Cx<P>)>(
        &mut self,
        callback: F,
    ) -> CallbackFn<P> {
        let callback = self
            .world_mut()
            .spawn(CallbackFnValue::<P> {
                inner: Some(Box::new(callback)),
            })
            .id();
        self.add_owned(callback);
        CallbackFn {
            id: callback,
            marker: PhantomData,
        }
    }

    /// Create a new [`CallbackFnMut`] in this context. This holds a `FnMut` within an entity.
    ///
    /// Arguments:
    /// * `callback` - The callback function to invoke. This will be called with a single
    ///    parameter, which is a [`Cx`] object. The context may or may not have props.
    fn create_callback_mut<P: 'static, F: FnMut(&mut Cx<P>)>(
        &mut self,
        callback: F,
    ) -> CallbackFn<P>
    where
        F: Send + Sync + 'static,
    {
        let callback = self
            .world_mut()
            .spawn(CallbackFnMutValue {
                inner: Some(Box::new(callback)),
            })
            .id();
        self.add_owned(callback);
        CallbackFn {
            id: callback,
            marker: PhantomData,
        }
    }
}

/// Cx is a context parameter that is passed to presenters and callbacks. It contains the
/// presenter's properties (passed from the parent presenter), plus a reactive scope and
/// access to reactive data sources in the world.
pub struct Cx<'p, 'w, Props = ()> {
    /// The properties that were passed to the presenter from it's parent.
    pub props: &'p Props,

    /// Bevy World
    pub(crate) world: &'w mut World,

    /// Set of reactive resources referenced by the presenter.
    pub(crate) tracking: RefCell<&'p mut TrackingScope>,
}

impl<'p, 'w, Props> Cx<'p, 'w, Props> {
    pub(crate) fn new(
        props: &'p Props,
        world: &'w mut World,
        tracking: &'p mut TrackingScope,
    ) -> Self {
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

    // /// Spawn an empty [`Entity`] which is owned by this presenter. The entity will be
    // /// despawned when the presenter state is razed.
    // pub fn create_entity(&mut self) -> Entity {
    //     let mut tracking = self.tracking.borrow_mut();
    //     let index = tracking.next_entity_index;
    //     tracking.next_entity_index = index + 1;
    //     match index.cmp(&tracking.owned_entities.len()) {
    //         Ordering::Less => tracking.owned_entities[index],
    //         Ordering::Equal => {
    //             let id = self.bc.world.spawn_empty().id();
    //             tracking.owned_entities.push(id);
    //             id
    //         }
    //         Ordering::Greater => panic!("Invalid presenter entity index"),
    //     }
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

    // // / Return an object which can be used to send a message to the current presenter.
    // // pub fn use_callback<In, Marker>(&mut self, sys: impl IntoSystem<In, (), Marker>) {
    // //     todo!()
    // // }

    // fn add_tracked_component<C: Component>(&self, entity: Entity) {
    //     let cid = self
    //         .bc
    //         .world
    //         .component_id::<C>()
    //         .expect("Unregistered component type");
    //     self.tracking.borrow_mut().components.insert((entity, cid));
    // }
}

impl<'p, 'w, Props> ReactiveContext<'p> for Cx<'p, 'w, Props> {
    fn world(&self) -> &World {
        self.world
    }

    fn read_mutable<T>(&self, mutable: Entity) -> T
    where
        T: Send + Sync + Copy + 'static,
    {
        self.tracking.borrow_mut().add_mutable(mutable);
        self.world().read_mutable(mutable)
    }

    fn read_mutable_clone<T>(&self, mutable: Entity) -> T
    where
        T: Send + Sync + Clone + 'static,
    {
        self.tracking.borrow_mut().add_mutable(mutable);
        self.world().read_mutable_clone(mutable)
    }

    fn use_resource<T: Resource>(&self) -> &T {
        self.tracking.borrow_mut().add_resource::<T>(
            self.world()
                .components()
                .get_resource_id(TypeId::of::<T>())
                .expect("Unknown resource type"),
        );
        self.world().resource::<T>()
    }
}

impl<'p, 'w, Props> ReactiveContextMut<'p> for Cx<'p, 'w, Props> {
    fn world_mut(&mut self) -> &mut World {
        self.world
    }
}

impl<'p, 'w, Props> SetupContext<'p> for Cx<'p, 'w, Props> {
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

impl<'p, 'w> ReactiveContext<'p> for Rcx<'p, 'w> {
    fn world(&self) -> &World {
        self.world
    }

    fn read_mutable<T>(&self, mutable: Entity) -> T
    where
        T: Send + Sync + Copy + 'static,
    {
        self.tracking.borrow_mut().add_mutable(mutable);
        self.world().read_mutable(mutable)
    }

    fn read_mutable_clone<T>(&self, mutable: Entity) -> T
    where
        T: Send + Sync + Clone + 'static,
    {
        self.tracking.borrow_mut().add_mutable(mutable);
        self.world().read_mutable_clone(mutable)
    }

    fn use_resource<T: Resource>(&self) -> &T {
        self.tracking.borrow_mut().add_resource::<T>(
            self.world()
                .components()
                .get_resource_id(TypeId::of::<T>())
                .expect("Unknown resource type"),
        );
        self.world().resource::<T>()
    }
}

impl<'p> ReactiveContext<'p> for World {
    fn world(&self) -> &World {
        self
    }

    /// Read the value of a mutable variable using Copy semantics. Calling this function adds the
    /// mutable to the current tracking scope.
    fn read_mutable<T>(&self, mutable: Entity) -> T
    where
        T: Send + Sync + Copy + 'static,
    {
        let mutable_entity = self.world().entity(mutable);
        *mutable_entity
            .get::<MutableValue>()
            .unwrap()
            .value
            .downcast_ref::<T>()
            .unwrap()
    }

    /// Read the value of a mutable variable using Clone semantics. Calling this function adds the
    /// mutable to the current tracking scope.
    fn read_mutable_clone<T>(&self, mutable: Entity) -> T
    where
        T: Send + Sync + Clone + 'static,
    {
        let mutable_entity = self.world().entity(mutable);
        mutable_entity
            .get::<MutableValue>()
            .unwrap()
            .value
            .downcast_ref::<T>()
            .unwrap()
            .clone()
    }

    fn use_resource<T: Resource>(&self) -> &T {
        self.world().resource::<T>()
    }
}

impl<'p> ReactiveContextMut<'p> for World {
    fn world_mut(&mut self) -> &mut World {
        self
    }
}
