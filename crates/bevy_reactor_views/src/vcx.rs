use std::cell::RefCell;

use bevy::{ecs::world::DeferredWorld, prelude::*};
use bevy_reactor_signals::{CreateMutable, Mutable, TrackingScope};

/// Vcx is a context parameter that is passed to view templates. It a reference to the world,
/// and a tracking scope which is used for ownership of any created reactions.
pub struct Vcx<'p, 'w> {
    /// Bevy World
    world: &'w mut World,

    /// The entity that owns the tracking scope (or will own it).
    owner: Entity,

    /// Set of reactive resources referenced by the presenter.
    pub(crate) tracking: RefCell<&'p mut TrackingScope>,
}

impl<'p, 'w> Vcx<'p, 'w> {
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

    /// Create a new [`Mutable`] in this context.
    pub fn create_mutable<T>(&mut self, init: T) -> Mutable<T>
    where
        T: Send + Sync + 'static,
    {
        let mutable = self.world.create_mutable(init);
        self.tracking.borrow_mut().add_owned(mutable.id());
        mutable
    }

    // /// Create a new callback in this context. This registers a one-shot system with the world.
    // /// The callback will be unregistered when the tracking scope is dropped.
    // ///
    // /// Note: This function takes no deps argument, the callback is only registered once the first
    // /// time it is called. Subsequent calls will return the original callback.
    // pub fn create_callback<P: Send + Sync + 'static, M, S: IntoSystem<P, (), M> + 'static>(
    //     &mut self,
    //     callback: S,
    // ) -> Callback<P> {
    //     // let hook = self.tracking.borrow_mut().next_hook();
    //     let id = self.world_mut().register_system(callback);
    //     let result = Callback { id };
    //     self.tracking
    //         .borrow_mut()
    //         .push_hook(HookState::Callback(Arc::new(result)));
    //     result
    // }

    // /// Create a new [`Callback`] in this context. This holds a `Fn` within an entity.
    // ///
    // /// Arguments:
    // /// * `callback` - The callback function to invoke. This will be called with a single
    // ///    parameter, which is a [`Vcx`] object. The context may or may not have props.
    // fn create_callback<P: 'static, F: Send + Sync + 'static + Fn(&mut Vcx, P)>(
    //     &mut self,
    //     callback: F,
    // ) -> Callback<P> {
    //     let owner = self.owner();
    //     let callback = self
    //         .world_mut()
    //         .spawn(CallbackFnCell::<P> {
    //             inner: Some(Box::new(callback)),
    //         })
    //         .set_parent(owner)
    //         .id();
    //     self.add_owned(callback);
    //     Callback {
    //         id: callback,
    //         marker: PhantomData,
    //     }
    // }

    // /// Create a new [`CallbackFnMut`] in this context. This holds a `FnMut` within an entity.
    // ///
    // /// Arguments:
    // /// * `callback` - The callback function to invoke. This will be called with a single
    // ///    parameter, which is a [`Vcx`] object. The context may or may not have props.
    // fn create_callback_mut<P: 'static, F: Send + Sync + 'static + FnMut(&mut Vcx, P)>(
    //     &mut self,
    //     callback: F,
    // ) -> Callback<P> {
    //     let owner = self.owner();
    //     let callback = self
    //         .world_mut()
    //         .spawn(CallbackFnMutCell {
    //             inner: Some(Box::new(callback)),
    //         })
    //         .set_parent(owner)
    //         .id();
    //     self.add_owned(callback);
    //     Callback {
    //         id: callback,
    //         marker: PhantomData,
    //     }
    // }

    // /// Create a new [`Derived`] in this context. This represents a readable signal which
    // /// is computed from other signals. The result is not memoized, but is recomputed whenever
    // /// the dependencies change.
    // ///
    // /// Arguments:
    // /// * `compute` - The function that computes the output. This will be called with a single
    // ///    parameter, which is an [`Rcx`] object.
    // fn create_derived<R: 'static, F: Send + Sync + 'static + Fn(&mut Rcx) -> R>(
    //     &mut self,
    //     compute: F,
    // ) -> Signal<R> {
    //     let owner = self.owner();
    //     let derived = self
    //         .world_mut()
    //         .spawn(DerivedCell::<R>(Arc::new(compute)))
    //         .set_parent(owner)
    //         .id();
    //     self.add_owned(derived);
    //     Signal::Derived(Derived {
    //         id: derived,
    //         marker: PhantomData,
    //     })
    // }

    // /// Create a new [`Memo`] in this context. This represents a readable signal which
    // /// is computed from other signals. The result is memoized, which means that downstream
    // /// dependants will not be notified unless the output changes.
    // ///
    // /// Arguments:
    // /// * `compute` - The function that computes the output. This will be called with a single
    // ///    parameter, which is a [`Vcx`] object.
    // fn create_memo<
    //     R: 'static + PartialEq + Send + Sync + Clone,
    //     F: Send + Sync + 'static + Fn(&mut Vcx) -> R,
    // >(
    //     &mut self,
    //     compute: F,
    // ) -> Signal<R> {
    //     let owner = self.owner();
    //     let ticks = self.world_mut().change_tick();
    //     let mut scope = TrackingScope::new(ticks);
    //     let init = compute(&mut Vcx::new(self.world_mut(), owner, &mut scope));
    //     let mutable = self.create_mutable(init);
    //     let signal = mutable.signal();
    //     let reaction = Arc::new(Mutex::new(move |cx: &mut Vcx| {
    //         let prev_value = mutable.get_clone(cx);
    //         let value = compute(cx);
    //         if value != prev_value {
    //             mutable.set_clone(cx, value);
    //         }
    //     }));
    //     self.world_mut().entity_mut(mutable.cell).insert((
    //         ReactionCell(reaction),
    //         scope,
    //         Name::new(format!("Memo::<{}>", std::any::type_name::<R>())),
    //     ));

    //     signal
    // }

    // /// Create an effect. This is a function that is associated with an entity, and which
    // /// re-runs whenever any of it's dependencies change.
    // ///
    // /// Arguments:
    // /// * `effect` - The function that computes the output. This will be called with a single
    // ///    parameter, which is a [`Vcx`] object.
    // fn create_effect<F: Send + Sync + 'static + FnMut(&mut Vcx)>(&mut self, effect: F) {
    //     let owner = self.owner();
    //     let ticks = self.world_mut().change_tick();
    //     let action = Arc::new(Mutex::new(effect));
    //     let mut scope = TrackingScope::new(ticks);
    //     let entity = self.world_mut().spawn_empty().set_parent(owner).id();
    //     self.add_owned(entity);
    //     action.lock().unwrap()(&mut Vcx::new(self.world_mut(), entity, &mut scope));
    //     self.world_mut()
    //         .entity_mut(entity)
    //         .insert((scope, ReactionCell(action)));
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

// impl<F: Send + Sync + 'static + FnMut(&mut Rcx)> Reaction for F {
//     fn react(&mut self, owner: Entity, world: &mut World, tracking: &mut TrackingScope) {
//         let mut cx = Rcx::new(world, owner, tracking);
//         (self)(&mut cx);
//     }
// }
