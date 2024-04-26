use bevy::core::Name;
use bevy::ecs::entity::Entity;
use bevy::ecs::world::World;

use crate::View;
use crate::{DespawnScopes, DisplayNodeChanged, Rcx, TrackingScope, ViewRef};

use crate::node_span::NodeSpan;

/// A dynamic view which can change its content based on a function. The inner view is razed
/// and rebuilt whenever the function reacts.
pub struct Dynamic<V: Into<ViewRef>, F: Fn(&Rcx) -> V + Send> {
    state: Option<(ViewRef, Entity)>,
    item_fn: F,
}

impl<V: Into<ViewRef>, F: Fn(&Rcx) -> V + Send> Dynamic<V, F> {
    /// Construct a new dynamic view.
    pub fn new(item_fn: F) -> Self {
        Self {
            state: None,
            item_fn,
        }
    }
}

impl<V: Into<ViewRef>, F: Fn(&Rcx) -> V + Send> View for Dynamic<V, F> {
    fn nodes(&self) -> NodeSpan {
        match self.state {
            None => NodeSpan::Empty,
            Some((ref view, _)) => view.nodes(),
        }
    }

    fn build(&mut self, view_entity: bevy::prelude::Entity, world: &mut World) {
        let mut tracking = TrackingScope::new(world.change_tick());
        self.react(view_entity, world, &mut tracking);
        world
            .entity_mut(view_entity)
            .insert((tracking, Name::new("Dynamic")));
    }

    fn react(&mut self, view_entity: Entity, world: &mut World, tracking: &mut TrackingScope) {
        if let Some((view, entity)) = self.state.take() {
            view.raze(entity, world);
        }

        let view = (self.item_fn)(&Rcx::new(world, view_entity, tracking)).into();
        let entity = ViewRef::spawn(&view, view_entity, world);
        world.entity_mut(view_entity).insert(DisplayNodeChanged);
        self.state = Some((view, entity));
    }

    fn raze(&mut self, view_entity: Entity, world: &mut World) {
        if let Some((ref view, entity)) = self.state {
            view.raze(entity, world)
        }
        self.state = None;
        world.despawn_owned_recursive(view_entity);
    }
}

impl<V: Into<ViewRef> + 'static, F: Fn(&Rcx) -> V + Sync + Send + 'static> From<Dynamic<V, F>>
    for ViewRef
{
    fn from(value: Dynamic<V, F>) -> Self {
        ViewRef::new(value)
    }
}

// Commented out because it conflicts with the ViewTemplate -> ViewRef conversion.
// impl<V: Into<ViewRef> + 'static, F: Fn(&Rcx) -> V + Sync + Send + 'static> From<F> for ViewRef {
//     fn from(value: F) -> Self {
//         ViewRef::new(Dynamic::new(value))
//     }
// }
