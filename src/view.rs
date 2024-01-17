use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

use bevy::ecs::{component::Component, entity::Entity, query::Added, world::World};

use crate::{node_span::NodeSpan, scope::TrackingScope};

pub trait View {
    fn nodes(&self) -> NodeSpan;
    fn build(&mut self, vc: &mut ViewContext);
}

pub trait IntoView {
    fn into_view(self) -> Box<dyn View + Send + Sync + 'static>;
}

impl IntoView for () {
    fn into_view(self) -> Box<dyn View + Send + Sync + 'static> {
        todo!();
    }
}

impl<V: View + Send + Sync + 'static, F: Fn() -> V> IntoView for F {
    fn into_view(self) -> Box<dyn View + Send + Sync + 'static> {
        Box::new(self())
    }
}

pub struct ViewContext<'p> {
    /// Bevy World
    pub(crate) world: &'p mut World,

    /// Entity representing the current owning scope.
    pub(crate) owner: Option<Entity>,

    /// Set of reactive resources referenced by the presenter.
    pub(crate) tracking: RefCell<&'p mut TrackingScope>,
}

#[derive(Component)]
pub struct ViewHandle {
    pub(crate) view: Arc<Mutex<dyn View + Sync + Send + 'static>>,
}

impl ViewHandle {
    pub fn new(view: impl View + Sync + Send + 'static) -> Self {
        Self {
            view: Arc::new(Mutex::new(view)),
        }
    }
}

pub(crate) fn build_view_handles(world: &mut World) {
    // Need to copy query result to avoid double-borrow of world.
    let mut view_handles = world.query_filtered::<(Entity, &mut ViewHandle), Added<ViewHandle>>();
    let view_entities: Vec<Entity> = view_handles.iter(world).map(|(e, _)| e).collect();
    for view_entity in view_entities.iter() {
        let tick = world.change_tick();
        let Ok((_, view_handle)) = view_handles.get(world, *view_entity) else {
            continue;
        };
        let inner = view_handle.view.clone();
        let mut tracking: TrackingScope = TrackingScope::new(tick);
        let mut vc = ViewContext {
            world,
            owner: Some(*view_entity),
            tracking: RefCell::new(&mut tracking),
        };
        inner.lock().unwrap().build(&mut vc);
        world.entity_mut(*view_entity).insert(tracking);
    }
}
