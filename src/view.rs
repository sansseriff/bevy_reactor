use std::sync::{Arc, Mutex};

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::{Added, With},
        world::World,
    },
    hierarchy::{BuildWorldChildren, Parent},
    log::warn,
};

use crate::{node_span::NodeSpan, text::TextStatic, tracking_scope::TrackingScope};

/// Trait that defines a view, which is a template that constructs a hierarchy of
/// entities and components.
///
/// Lifecycle: To create a view, use [`ViewHandle::spawn`]. This creates an entity to hold the view,
/// and which drives the reaction system. When the view is no longer needed, call [`View::raze`].
/// This will destroy the view entity, and all of its children and display nodes.
pub trait View {
    /// Returns the display nodes produced by this `View`.
    fn nodes(&self) -> NodeSpan;

    /// Initialize the view, creating any entities needed.
    ///
    /// Arguments:
    /// * `view_entity`: The entity that owns this view.
    /// * `world`: The Bevy world.
    fn build(&mut self, view_entity: Entity, world: &mut World);

    /// Update the view, reacting to changes in dependencies. This is optional, and need only
    /// be implemented for views that are reactive.
    fn react(&mut self, _view_entity: Entity, _world: &mut World, _tracking: &mut TrackingScope) {}

    /// Destroy the view, including the display nodes, and all descendant views.
    fn raze(&mut self, view_entity: Entity, world: &mut World);

    /// Notification from child views that the child display nodes have changed and need
    /// to be re-attached to the parent. This is optional, and need only be implemented for
    /// views which have display nodes that have child display nodes (like [`Element`]).
    fn children_changed(&mut self, _view_entity: Entity, _world: &mut World) -> bool {
        false
    }
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
    /// Construct a new [`ViewHandle`] from a [`View`].
    pub fn new(view: impl View + Sync + Send + 'static) -> Self {
        Self {
            view: Arc::new(Mutex::new(view)),
        }
    }

    /// Construct a new [`ViewHandle`] from a [`ViewRef`].
    pub fn from_ref(view: ViewRef) -> Self {
        Self { view }
    }

    /// Given a view template, construct a new view. This creates an entity to hold the view
    /// and the view handle, and then calls [`View::build`] on the view. The resuling entity
    /// is part of the template invocation hierarchy, it is not a display node.
    pub fn spawn(view: &ViewRef, parent: Entity, world: &mut World) -> Entity {
        let mut child_ent = world.spawn(ViewHandle::from_ref(view.clone()));
        child_ent.set_parent(parent);
        let id = child_ent.id();
        view.lock().unwrap().build(child_ent.id(), world);
        id
    }
}

#[derive(Component)]
/// Marker component used to signal that a view's output nodes have changed.
pub struct DisplayNodeChanged;

/// A `[View]` which displays nothing - can be used as a placeholder.
pub struct EmptyView;

impl View for EmptyView {
    fn nodes(&self) -> NodeSpan {
        NodeSpan::Empty
    }

    fn build(&mut self, _view_entity: Entity, _world: &mut World) {}
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
        inner.lock().unwrap().build(*root_entity, world);
    }
}

/// System that looks for changed child views and replaces the parent's child nodes.
pub(crate) fn attach_child_views(world: &mut World) {
    let mut query = world.query_filtered::<Entity, With<DisplayNodeChanged>>();
    let query_copy = query.iter(world).collect::<Vec<Entity>>();
    for entity in query_copy {
        world.entity_mut(entity).remove::<DisplayNodeChanged>();
        let mut e = entity;
        loop {
            if let Some(handle) = world.entity(e).get::<ViewHandle>() {
                let inner = handle.view.clone();
                if inner.lock().unwrap().children_changed(e, world) {
                    break;
                }
            }

            if let Some(handle) = world.entity(e).get::<ViewRoot>() {
                let inner = handle.view.clone();
                if inner.lock().unwrap().children_changed(e, world) {
                    break;
                }
            }

            e = match world.entity(e).get::<Parent>() {
                Some(parent) => parent.get(),
                None => {
                    warn!("DisplayNodeChanged not handled.");
                    break;
                }
            };
        }
    }
}
