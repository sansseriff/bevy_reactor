use std::{
    cell::RefCell,
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use bevy::{ecs::world::DeferredWorld, prelude::*};

use crate::{
    derived::{Derived, ReadDerived, ReadDerivedInternal},
    mutable::{MutableCell, ReadMutable},
    tracking_scope::TrackingScope,
    Mutable, Reaction, ReactionCell, Signal,
};

/// An immutable reactive context, used for reactive closures such as derived signals.
pub trait RunContextRead {
    /// Return a reference to the resource of the given type. Calling this function
    /// adds the resource as a dependency of the current tracking scope.
    fn read_resource<T: Resource>(&self) -> &T;

    /// Return a reference to the Component `C` on the given entity. Calling this function
    /// adds the component as a dependency of the current tracking scope.
    fn read_component<C: Component>(&self, entity: Entity) -> Option<&C>;
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
        let component = self.world_mut().register_component::<MutableCell<T>>();
        self.add_owned(cell);
        Mutable {
            cell,
            component,
            marker: PhantomData,
        }
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
                mutable.set_clone(cx.world_mut(), value);
            }
        }));
        self.world_mut().entity_mut(mutable.cell).insert((
            ReactionCell(reaction),
            scope,
            Name::new(format!("Memo::<{}>", std::any::type_name::<R>())),
        ));

        signal
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
    _owner: Entity,

    /// Set of reactive resources referenced by the presenter.
    pub(crate) tracking: RefCell<&'p mut TrackingScope>,
}

impl<'p, 'w> Cx<'p, 'w> {
    /// Construct a new reactive context.
    fn new(world: &'w mut World, owner: Entity, tracking: &'p mut TrackingScope) -> Self {
        Self {
            world,
            _owner: owner,
            tracking: RefCell::new(tracking),
        }
    }

    /// Access to mutable world from reactive context.
    fn world_mut(&mut self) -> &mut World {
        self.world
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
