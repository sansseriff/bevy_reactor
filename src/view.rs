use std::sync::{Arc, Mutex};

use bevy::{
    core::Name,
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
#[allow(unused_variables)]
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
    fn react(&mut self, view_entity: Entity, world: &mut World, tracking: &mut TrackingScope) {}

    /// Destroy the view, including the display nodes, and all descendant views.
    fn raze(&mut self, view_entity: Entity, world: &mut World);

    /// Notification from child views that the child display nodes have changed and need
    /// to be re-attached to the parent. This is optional, and need only be implemented for
    /// views which have display nodes that have child display nodes (like [`Element`]).
    fn children_changed(&mut self, view_entity: Entity, world: &mut World) -> bool {
        false
    }
}

impl<V: View + Send + Sync + 'static> From<V> for ViewHandle {
    fn from(view: V) -> Self {
        ViewHandle(Arc::new(Mutex::new(view)))
    }
}

impl From<()> for ViewHandle {
    fn from(_value: ()) -> Self {
        ViewHandle(Arc::new(Mutex::new(EmptyView)))
    }
}

impl From<&str> for ViewHandle {
    fn from(value: &str) -> Self {
        ViewHandle(Arc::new(Mutex::new(TextStatic::new(value.to_string()))))
    }
}

impl From<String> for ViewHandle {
    fn from(value: String) -> Self {
        ViewHandle(Arc::new(Mutex::new(TextStatic::new(value))))
    }
}

#[derive(Component)]
/// Component which holds the top level of the view hierarchy.
pub struct ViewRoot {
    pub(crate) view: Arc<Mutex<dyn View + Sync + Send + 'static>>,
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
pub struct ViewHandle(pub(crate) Arc<Mutex<dyn View + Sync + Send + 'static>>);

impl ViewHandle {
    /// Construct a new [`ViewHandle`] from a [`View`].
    pub fn new(view: impl View + Sync + Send + 'static) -> Self {
        Self(Arc::new(Mutex::new(view)))
    }

    /// Given a view template, construct a new view. This creates an entity to hold the view
    /// and the view handle, and then calls [`View::build`] on the view. The resuling entity
    /// is part of the template invocation hierarchy, it is not a display node.
    pub fn spawn(view: &ViewHandle, parent: Entity, world: &mut World) -> Entity {
        let mut child_ent = world.spawn(ViewHandle(view.0.clone()));
        child_ent.set_parent(parent);
        let id = child_ent.id();
        view.build(child_ent.id(), world);
        id
    }

    /// Returns the display nodes produced by this `View`.
    pub fn nodes(&self) -> NodeSpan {
        self.0.lock().unwrap().nodes()
    }

    /// Initialize the view, creating any entities needed.
    ///
    /// Arguments:
    /// * `view_entity`: The entity that owns this view.
    /// * `world`: The Bevy world.
    pub fn build(&self, view_entity: Entity, world: &mut World) {
        self.0.lock().unwrap().build(view_entity, world);
    }

    /// Destroy the view, including the display nodes, and all descendant views.
    pub fn raze(&self, view_entity: Entity, world: &mut World) {
        self.0.lock().unwrap().raze(view_entity, world);
    }
}

impl Clone for ViewHandle {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Default for ViewHandle {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(EmptyView)))
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
        let mut finished = false;
        loop {
            if let Some(handle) = world.entity(e).get::<ViewHandle>() {
                let inner = handle.0.clone();
                if inner.lock().unwrap().children_changed(e, world) {
                    finished = true;
                    break;
                }
            }

            if let Some(handle) = world.entity(e).get::<ViewRoot>() {
                let inner = handle.view.clone();
                if inner.lock().unwrap().children_changed(e, world) {
                    finished = true;
                    break;
                }
            }

            e = match world.entity(e).get::<Parent>() {
                Some(parent) => parent.get(),
                None => {
                    break;
                }
            };
        }

        if !finished {
            warn!("DisplayNodeChanged not handled.");
            e = entity;
            loop {
                if let Some(name) = world.entity(e).get::<Name>() {
                    println!("* Entity: {:?}", name);
                } else {
                    println!("* Entity: {:?}", e);
                }
                e = match world.entity(e).get::<Parent>() {
                    Some(parent) => parent.get(),
                    None => {
                        break;
                    }
                };
            }
        }
    }
}
