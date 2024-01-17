use std::sync::{Arc, Mutex};

use bevy::ecs::{component::Component, entity::Entity, query::Added, world::World};

use crate::{
    node_span::NodeSpan,
    scope::TrackingScope,
    text::{DynTextView, TextView},
    Cx,
};

pub trait View {
    /// Returns the display nodes produced by this `View`.
    fn nodes(&self) -> NodeSpan;

    /// Initialize the view, creating any entities needed.
    fn build(&mut self, view_entity: Entity, vc: &mut ViewContext);

    /// Update the view, reacting to changes in dependencies.
    fn react(
        &mut self,
        _view_entity: Entity,
        _vc: &mut ViewContext,
        _tracking: &mut TrackingScope,
    ) {
    }

    /// Destroy the view, including the display nodes, and all descendant views.
    fn raze(&mut self, view_entity: Entity, world: &mut World);
}

// A reference to a view.
pub type ViewRef = Arc<Mutex<dyn View + Sync + Send + 'static>>;

/// Trait that allows a type to be converted into a `ViewHandle`.
pub trait IntoView {
    fn into_view(self) -> ViewRef;
}

impl IntoView for () {
    fn into_view(self) -> ViewRef {
        todo!();
    }
}

impl IntoView for &str {
    fn into_view(self) -> ViewRef {
        Arc::new(Mutex::new(TextView::new(self.to_string())))
    }
}

impl IntoView for String {
    fn into_view(self) -> ViewRef {
        Arc::new(Mutex::new(TextView::new(self)))
    }
}

impl<F: Send + Sync + 'static + Fn(&mut Cx) -> String> IntoView for F {
    fn into_view(self) -> ViewRef {
        Arc::new(Mutex::new(DynTextView::new(self)))
    }
}

// impl<V: View, F: Send + Sync + 'static + Fn(&mut Cx) -> V> IntoViewHandle for F {
//     fn into_view_handle(self) -> ViewHandle {
//         ViewHandle::new(DynTextView::new(self))
//     }
// }

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
    pub(crate) view: ViewRef,
}

impl ViewHandle {
    pub fn new(view: impl View + Sync + Send + 'static) -> Self {
        Self {
            view: Arc::new(Mutex::new(view)),
        }
    }

    pub(crate) fn nodes(&self) -> NodeSpan {
        self.view.lock().unwrap().nodes()
    }
}

/// System that initializes any views that have been added.
pub(crate) fn build_added_views(world: &mut World) {
    // Need to copy query result to avoid double-borrow of world.
    let mut view_handles = world.query_filtered::<(Entity, &mut ViewHandle), Added<ViewHandle>>();
    let view_entities: Vec<Entity> = view_handles.iter(world).map(|(e, _)| e).collect();
    for view_entity in view_entities.iter() {
        let Ok((_, view_handle)) = view_handles.get(world, *view_entity) else {
            continue;
        };
        let inner = view_handle.view.clone();
        let mut vc = ViewContext {
            world,
            owner: Some(*view_entity),
        };
        inner.lock().unwrap().build(*view_entity, &mut vc);
    }
}
