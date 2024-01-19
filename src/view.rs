use std::sync::{Arc, Mutex};

use bevy::ecs::{component::Component, entity::Entity, query::Added, world::World};

use crate::{node_span::NodeSpan, scope::TrackingScope, text::TextStatic};

/// Trait that defines a view, which is a template that constructs a hierarchy of
/// entities and components.
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

/// A reference to a view.
pub type ViewRef = Arc<Mutex<dyn View + Sync + Send + 'static>>;

/// Trait that allows a type to be converted into a `ViewHandle`.
pub trait IntoView {
    /// Convert the type into a `ViewRef`.
    fn into_view(self) -> ViewRef;
}

impl IntoView for () {
    fn into_view(self) -> ViewRef {
        Arc::new(Mutex::new(EmptyView))
    }
}

impl IntoView for &str {
    fn into_view(self) -> ViewRef {
        Arc::new(Mutex::new(TextStatic::new(self.to_string())))
    }
}

impl IntoView for String {
    fn into_view(self) -> ViewRef {
        Arc::new(Mutex::new(TextStatic::new(self)))
    }
}

/// Context object that is passed to views during construction and update.
pub struct ViewContext<'p> {
    /// Bevy World
    pub(crate) world: &'p mut World,

    /// Entity representing the current owning scope.
    pub(crate) owner: Option<Entity>,
}

#[derive(Component)]
/// Component which holds the top level of the view hierarchy.
pub struct ViewRoot {
    pub(crate) view: ViewRef,
}

impl ViewRoot {
    /// Construct a new [`ViewRoot`].
    pub fn new(view: impl View + Sync + Send + 'static) -> Self {
        Self {
            view: Arc::new(Mutex::new(view)),
        }
    }

    /// Despawn the view, including the display nodes, and all descendant views.
    pub fn despawn(&mut self, root: Entity, world: &mut World) {
        self.view.lock().unwrap().raze(root, world);
        world.entity_mut(root).despawn();
    }
}

/// Component used to hold a reference to a child view.
#[derive(Component)]
pub struct ViewHandle {
    pub(crate) view: ViewRef,
}

impl ViewHandle {
    /// Construct a new [`ViewHandle`].
    pub fn new(view: impl View + Sync + Send + 'static) -> Self {
        Self {
            view: Arc::new(Mutex::new(view)),
        }
    }

    pub(crate) fn nodes(&self) -> NodeSpan {
        self.view.lock().unwrap().nodes()
    }
}

/// A `[View]` which displays nothing - can be used as a placeholder.
pub struct EmptyView;

impl View for EmptyView {
    fn nodes(&self) -> NodeSpan {
        NodeSpan::Empty
    }

    fn build(&mut self, _view_entity: Entity, _vc: &mut ViewContext) {}
    fn raze(&mut self, _view_entity: Entity, _world: &mut World) {}
}

/// System that initializes any views that have been added.
pub(crate) fn build_added_view_roots(world: &mut World) {
    // Need to copy query result to avoid double-borrow of world.
    let mut roots = world.query_filtered::<(Entity, &mut ViewRoot), Added<ViewRoot>>();
    let roots_copy: Vec<Entity> = roots.iter(world).map(|(e, _)| e).collect();
    for root_entity in roots_copy.iter() {
        let Ok((_, root)) = roots.get(world, *root_entity) else {
            continue;
        };
        let inner = root.view.clone();
        let mut vc = ViewContext {
            world,
            owner: Some(*root_entity),
        };
        inner.lock().unwrap().build(*root_entity, &mut vc);
    }
}
