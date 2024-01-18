use std::{
    any::TypeId,
    cell::{RefCell, RefMut},
    marker::PhantomData,
    sync::atomic::AtomicBool,
};

use bevy::prelude::*;

use crate::{
    mutable::{MutableValue, MutableValueNext},
    scope::TrackingScope,
    Mutable,
};

/// An immutable reactive context, used for reactive closures such as derived signals.
pub trait ReactiveContext<'p> {
    /// The current Bevy [`World`].
    fn world(&self) -> &World;

    /// Set of reactive resources referenced by the presenter.
    fn tracking(&self) -> RefMut<&'p mut TrackingScope>;

    /// Read the value of a mutable variable using Copy semantics. Calling this function adds the
    /// mutable to the current tracking scope.
    fn read_mutable<T>(&self, mutable: Entity) -> T
    where
        T: Send + Sync + Copy + 'static,
    {
        let mutable_entity = self.world().entity(mutable);
        self.tracking().add_mutable(mutable);
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
        self.tracking().add_mutable(mutable);
        mutable_entity
            .get::<MutableValue>()
            .unwrap()
            .value
            .downcast_ref::<T>()
            .unwrap()
            .clone()
    }

    /// Return a reference to the resource of the given type. Calling this function
    /// adds the resource as a dependency of the current presenter invocation.
    fn use_resource<T: Resource>(&self) -> &T {
        self.tracking().add_resource::<T>(
            self.world()
                .components()
                .get_resource_id(TypeId::of::<T>())
                .expect("Unknown resource type"),
        );
        self.world().resource::<T>()
    }
}

/// A mutable reactive context. This allows access to reactive data sources, and can also
/// be used to create new reactions such as mutable variables and effects.
pub trait ReactiveContextMut<'p>: ReactiveContext<'p> {
    /// The current Bevy [`World`].
    fn world_mut(&mut self) -> &mut World;

    /// Create a new [`Mutable`] in this context.
    fn create_mutable<T>(&mut self, init: T) -> Mutable<T>
    where
        T: Send + Sync + 'static,
    {
        let mutable = self
            .world_mut()
            .spawn((MutableValue {
                changed: AtomicBool::new(false),
                value: Box::new(init),
            },))
            .id();
        self.tracking().add_owned(mutable);
        Mutable {
            id: mutable,
            marker: PhantomData,
        }
    }
}

/// Cx is a context parameter that is passed to presenters. It contains the presenter's
/// properties (passed from the parent presenter), plus a reactive scope and access to
/// reactive data sources in the world.
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

    pub(crate) fn write_mutable<T>(&mut self, mutable: Entity, value: T)
    where
        T: Send + Sync + Copy + PartialEq + 'static,
    {
        let mut mutable_entity = self.world.entity_mut(mutable);
        if let Some(mut next) = mutable_entity.get_mut::<MutableValueNext>() {
            *next.0.downcast_mut::<T>().unwrap() = value;
        } else if let Some(current_value) = mutable_entity.get_mut::<MutableValue>() {
            if *current_value.value.downcast_ref::<T>().unwrap() != value {
                mutable_entity.insert(MutableValueNext(Box::new(value)));
            }
        }
    }

    pub(crate) fn write_mutable_clone<T>(&mut self, mutable: Entity, value: T)
    where
        T: Send + Sync + Clone + PartialEq + 'static,
    {
        let mut mutable_entity = self.world.entity_mut(mutable);
        if let Some(mut next) = mutable_entity.get_mut::<MutableValueNext>() {
            *next.0.downcast_mut::<T>().unwrap() = value;
        } else if let Some(current_value) = mutable_entity.get_mut::<MutableValue>() {
            if *current_value.value.downcast_ref::<T>().unwrap() != value {
                mutable_entity.insert(MutableValueNext(Box::new(value.clone())));
            }
        }
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

    // /// Run a function on the view entity. Will only re-run when [`deps`] changes.
    // pub fn use_effect<F: FnOnce(EntityWorldMut), D: Clone + PartialEq + Send + Sync + 'static>(
    //     &mut self,
    //     effect: F,
    //     deps: D,
    // ) {
    //     let handle = self.create_atom_handle::<D>();
    //     let mut entt = self.bc.world.entity_mut(handle.id);
    //     match entt.get_mut::<AtomCell>() {
    //         Some(mut cell) => {
    //             let deps_old = cell.0.downcast_mut::<D>().expect("Atom is incorrect type");
    //             if *deps_old != deps {
    //                 *deps_old = deps;
    //                 (effect)(self.bc.world.entity_mut(self.bc.entity));
    //             }
    //         }
    //         None => {
    //             entt.insert(AtomCell(Box::new(deps)));
    //             (effect)(self.bc.world.entity_mut(self.bc.entity));
    //         }
    //     }
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

    fn tracking(&self) -> RefMut<&'p mut TrackingScope> {
        self.tracking.borrow_mut()
    }
}

impl<'p, 'w, Props> ReactiveContextMut<'p> for Cx<'p, 'w, Props> {
    fn world_mut(&mut self) -> &mut World {
        self.world
    }
}

/// Immutable reactive context, used for reactive closures such as derived signals.
pub struct Re<'p, 'w> {
    /// Bevy World
    pub(crate) world: &'w World,

    /// Set of reactive resources referenced by the presenter.
    pub(crate) tracking: RefCell<&'p mut TrackingScope>,
}

impl<'p, 'w> Re<'p, 'w> {
    pub(crate) fn new(world: &'w World, tracking: &'p mut TrackingScope) -> Self {
        Self {
            world,
            tracking: RefCell::new(tracking),
        }
    }
}

impl<'p, 'w> ReactiveContext<'p> for Re<'p, 'w> {
    fn world(&self) -> &World {
        self.world
    }

    fn tracking(&self) -> RefMut<&'p mut TrackingScope> {
        self.tracking.borrow_mut()
    }
}
