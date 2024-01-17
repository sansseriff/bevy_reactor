use std::sync::{Arc, Mutex};

use bevy::ecs::{component::Component, entity::Entity, query::Added, world::World};

use crate::{
    node_span::NodeSpan,
    text::{DynTextView, TextView},
    Cx,
};

pub trait View {
    fn nodes(&self) -> NodeSpan;
    fn build(&mut self, vc: &mut ViewContext);
}

pub trait IntoView {
    fn into_view(self) -> Box<dyn View + Send + Sync + 'static>;
    fn into_handle(self, world: &mut World) -> Entity;
}

impl IntoView for () {
    fn into_view(self) -> Box<dyn View + Send + Sync + 'static> {
        todo!();
    }

    fn into_handle(self, world: &mut World) -> Entity {
        todo!();
    }
}

impl IntoView for &str {
    fn into_view(self) -> Box<dyn View + Send + Sync> {
        Box::new(TextView::new(self.to_string()))
    }

    fn into_handle(self, world: &mut World) -> Entity {
        world
            .spawn(ViewHandle::new(TextView::new(self.to_string())))
            .id()
    }
}

impl IntoView for String {
    fn into_view(self) -> Box<dyn View + Send + Sync> {
        Box::new(TextView::new(self))
    }

    fn into_handle(self, world: &mut World) -> Entity {
        world.spawn(ViewHandle::new(TextView::new(self))).id()
    }
}

impl<F: Send + Sync + 'static + Fn(&mut Cx) -> String> IntoView for F {
    fn into_view(self) -> Box<dyn View + Send + Sync + 'static> {
        Box::new(DynTextView::new(self))
    }

    fn into_handle(self, world: &mut World) -> Entity {
        world.spawn(ViewHandle::new(DynTextView::new(self))).id()
    }
}

pub struct ViewContext<'p> {
    /// Bevy World
    pub(crate) world: &'p mut World,

    /// Entity representing the current owning scope.
    pub(crate) owner: Option<Entity>,
    // Set of reactive resources referenced by the presenter.
    // pub(crate) tracking: RefCell<&'p mut TrackingScope>,
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

/// System that initializes any views that have been added.
pub(crate) fn build_views(world: &mut World) {
    // Need to copy query result to avoid double-borrow of world.
    let mut view_handles = world.query_filtered::<(Entity, &mut ViewHandle), Added<ViewHandle>>();
    let view_entities: Vec<Entity> = view_handles.iter(world).map(|(e, _)| e).collect();
    for view_entity in view_entities.iter() {
        // let tick = world.change_tick();
        let Ok((_, view_handle)) = view_handles.get(world, *view_entity) else {
            continue;
        };
        let inner = view_handle.view.clone();
        // let mut tracking = TrackingScope::new(tick);
        let mut vc = ViewContext {
            world,
            owner: Some(*view_entity),
            // tracking: RefCell::new(&mut tracking),
        };
        inner.lock().unwrap().build(&mut vc);
        // world.entity_mut(*view_entity).insert(tracking);
    }
}
