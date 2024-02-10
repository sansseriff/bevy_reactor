use bevy::prelude::*;

use crate::{node_span::NodeSpan, view::View, DespawnScopes, ViewHandle};

/// A `Portal` represents a view that is displayed with no parent, causing it's location to
/// be relative to the window rather than any parent view.
pub struct Portal {
    view: ViewHandle,
    entity: Option<Entity>,
}

impl Portal {
    /// Construct a new `Fragment`.
    pub fn new(view: impl Into<ViewHandle>) -> Self {
        Self {
            view: view.into(),
            entity: None,
        }
    }
}

impl View for Portal {
    fn nodes(&self) -> NodeSpan {
        NodeSpan::Empty
    }

    fn build(&mut self, view_entity: Entity, world: &mut World) {
        world.entity_mut(view_entity).insert(Name::new("Portal"));
        self.entity = Some(ViewHandle::spawn(&self.view, view_entity, world));
    }

    fn raze(&mut self, view_entity: Entity, world: &mut World) {
        self.view.raze(self.entity.unwrap(), world);
        self.entity = None;
        world.despawn_owned_recursive(view_entity);
    }

    fn children_changed(&mut self, _view_entity: Entity, _world: &mut World) -> bool {
        // info!("children_changed handled");
        true
    }
}
