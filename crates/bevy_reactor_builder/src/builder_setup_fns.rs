use bevy::prelude::IntoSystem;
use bevy_reactor_signals::{Callback, CallbackOwner};

use crate::ui_builder::UiBuilder;

pub trait BuilderSetup {
    /// Create a new one-shot system which is owned by the parent entity, and which will be
    /// unregistered when the entity is despawned.
    fn create_callback<P: Send + Sync + 'static, M, S: IntoSystem<P, (), M> + 'static>(
        &mut self,
        callback: S,
    ) -> Callback<P>;
}

impl<'w> BuilderSetup for UiBuilder<'w> {
    fn create_callback<P: Send + Sync + 'static, M, S: IntoSystem<P, (), M> + 'static>(
        &mut self,
        callback: S,
    ) -> Callback<P> {
        let id = self.world_mut().register_system(callback);
        let result = Callback::new(id);
        let parent = self.parent();
        match self.world_mut().get_mut::<CallbackOwner>(parent) {
            Some(mut owner) => {
                owner.add(result);
            }
            None => {
                let mut owner = CallbackOwner::new();
                owner.add(result);
                self.world_mut().entity_mut(parent).insert(owner);
            }
        }
        result
    }
}
