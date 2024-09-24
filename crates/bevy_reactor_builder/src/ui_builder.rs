use bevy::prelude::{BuildChildren, Bundle, Entity, EntityWorldMut, IntoSystem, World};
use bevy_reactor_signals::{Callback, CallbackOwner};

pub struct UiBuilder<'w> {
    /// Bevy World
    world: &'w mut World,

    /// The entity that will be the parent of all of the children and other resources created
    /// in this scope.
    parent: Entity,
}

impl<'w> UiBuilder<'w> {
    /// Construct a new reactive context.
    pub fn new(world: &'w mut World, owner: Entity) -> Self {
        Self {
            world,
            parent: owner,
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

    /// Returns the parent entity
    pub fn parent(&self) -> Entity {
        self.parent
    }

    /// Spawn a new child of the parent entity with the given bundle.
    pub fn spawn(&mut self, bundle: impl Bundle) -> EntityWorldMut {
        let mut ent = self.world.spawn(bundle);
        ent.set_parent(self.parent);
        ent
    }

    /// Spawn a new, empty child of the parent entity.
    pub fn spawn_empty(&mut self) -> EntityWorldMut {
        let mut ent = self.world.spawn_empty();
        ent.set_parent(self.parent);
        ent
    }

    // pub fn entity(&mut self) {

    // }

    // entity(id)
    // cond()
    // switch()
    // for_each()
    // for_index()

    /// Create a new callback which is owned by the parent entity.
    fn create_callback<P: Send + Sync + 'static, M, S: IntoSystem<P, (), M> + 'static>(
        &mut self,
        callback: S,
    ) -> Callback<P> {
        let id = self.world_mut().register_system(callback);
        let result = Callback::new(id);
        let parent = self.parent();
        match self.world.get_mut::<CallbackOwner>(parent) {
            Some(mut owner) => {
                owner.add(result);
            }
            None => {
                let mut owner = CallbackOwner::new();
                owner.add(result);
                self.world.entity_mut(parent).insert(owner);
            }
        }
        result
    }
}

pub trait CreateChilden {
    fn create_children(&mut self, spawn_children: impl FnOnce(&mut UiBuilder)) -> &mut Self;
}

impl<'w> CreateChilden for EntityWorldMut<'w> {
    fn create_children(&mut self, spawn_children: impl FnOnce(&mut UiBuilder)) -> &mut Self {
        let parent = self.id();
        self.world_scope(|world| {
            spawn_children(&mut UiBuilder { world, parent });
        });
        self
    }
}
