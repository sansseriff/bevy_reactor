use bevy::prelude::*;

use crate::{node_span::NodeSpan, view::View, DespawnScopes, ViewRef};

/// A `Portal` represents a view that is displayed with no parent, causing it's location to
/// be relative to the window rather than any parent view.
pub struct Portal {
    view: ViewRef,
    entity: Option<Entity>,
}

impl Portal {
    /// Construct a new `Fragment`.
    pub fn new(view: impl Into<ViewRef>) -> Self {
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
        assert!(self.entity.is_none());
        world.entity_mut(view_entity).insert(Name::new("Portal"));
        self.entity = Some(ViewRef::spawn(&self.view, view_entity, world));
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

impl From<Portal> for ViewRef {
    fn from(value: Portal) -> Self {
        ViewRef::new(value)
    }
}
