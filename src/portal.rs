use std::sync::{Arc, Mutex};

use bevy::prelude::*;

use crate::{node_span::NodeSpan, view::View, DespawnScopes, IntoView, ViewHandle, ViewRef};

/// A `Portal` represents a view that is displayed with no parent, causing it's location to
/// be relative to the window rather than any parent view.
pub struct Portal {
    view: ViewRef,
    entity: Option<Entity>,
}

impl Portal {
    /// Construct a new `Fragment`.
    pub fn new(view: impl IntoView) -> Self {
        Self {
            view: view.into_view(),
            entity: None,
        }
    }
}

impl View for Portal {
    fn nodes(&self) -> NodeSpan {
        NodeSpan::Empty
    }

    fn build(&mut self, view_entity: Entity, world: &mut World) {
        self.entity = Some(ViewHandle::spawn(&self.view, view_entity, world));
    }

    fn raze(&mut self, view_entity: Entity, world: &mut World) {
        let inner = self.view.clone();
        inner.lock().unwrap().raze(self.entity.unwrap(), world);
        self.entity = None;
        world.despawn_owned_recursive(view_entity);
    }
}

impl IntoView for Portal {
    fn into_view(self) -> ViewRef {
        Arc::new(Mutex::new(self))
    }
}
