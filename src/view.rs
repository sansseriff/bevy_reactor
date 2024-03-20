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

use crate::{
    node_span::NodeSpan, text::TextStatic, tracking_scope::TrackingScope, Cx, DespawnScopes,
};

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
    ///
    /// Returns `true` if the view was able to update its display nodes. If it returns `false`,
    /// then it means that this view is only a thin wrapper for other views, and doesn't actually
    /// have any display nodes of its own, in which case the parent view will need to handle the
    /// change.
    fn children_changed(&mut self, view_entity: Entity, world: &mut World) -> bool {
        false
    }
}

/// A [`View`] with all the trimmings.
pub trait SyncView: View + Send + Sync + 'static {}

// This From impl is commented out because it causes many conflicts with other From impls.
// impl<V: View + Send + Sync + 'static> From<V> for ViewHandle {
//     fn from(view: V) -> Self {
//         ViewHandle::new(view)
//     }
// }

impl From<()> for ViewHandle {
    fn from(_value: ()) -> Self {
        ViewHandle::new(EmptyView)
    }
}

impl From<&str> for ViewHandle {
    fn from(value: &str) -> Self {
        ViewHandle::new(TextStatic::new(value.to_string()))
    }
}

impl From<String> for ViewHandle {
    fn from(value: String) -> Self {
        ViewHandle::new(TextStatic::new(value))
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
        Self::new(EmptyView)
    }
}

#[derive(Component)]
/// Marker component used to signal that a view's output nodes have changed.
pub struct DisplayNodeChanged;

/// A `[View]` which displays nothing - can be used as a placeholder.
pub struct EmptyView;

#[allow(unused_variables)]
impl View for EmptyView {
    fn nodes(&self) -> NodeSpan {
        NodeSpan::Empty
    }

    fn build(&mut self, view_entity: Entity, world: &mut World) {}
    fn raze(&mut self, view_entity: Entity, world: &mut World) {}
}

/// Trait that defines a factory object that can construct a [`View`] from a reactive context.
pub trait ViewFactory {
    /// Create the view for the control.
    fn create(&self, cx: &mut Cx) -> impl View + Send + Sync + 'static;
}

/// Holds a [`ViewFactory`], and the entity and output nodes created by the [`View`] produced
/// by the factory.
pub struct ViewFactoryState<VF: ViewFactory> {
    /// Reference to presenter function.
    factory: VF,

    /// The view handle for the presenter output.
    inner: Option<Entity>,

    /// Display nodes.
    nodes: NodeSpan,
}

impl<W: ViewFactory> ViewFactoryState<W> {
    /// Construct a new `WidgetInstance`.
    pub fn new(widget: W) -> Self {
        Self {
            factory: widget,
            inner: None,
            nodes: NodeSpan::Empty,
        }
    }
}

impl<W: ViewFactory> View for ViewFactoryState<W> {
    fn nodes(&self) -> NodeSpan {
        self.nodes.clone()
    }

    fn build(&mut self, view_entity: Entity, world: &mut World) {
        assert!(self.inner.is_none());
        let mut tracking = TrackingScope::new(world.read_change_tick());
        let mut cx = Cx::new((), world, &mut tracking);
        let mut view = self.factory.create(&mut cx);
        let inner = world.spawn(tracking).set_parent(view_entity).id();
        view.build(inner, world);
        self.nodes = view.nodes();
        world.entity_mut(inner).insert(ViewHandle::new(view));
        self.inner = Some(inner);
    }

    fn raze(&mut self, view_entity: Entity, world: &mut World) {
        assert!(self.inner.is_some());
        let mut entt = world.entity_mut(self.inner.unwrap());
        if let Some(handle) = entt.get_mut::<ViewHandle>() {
            // Despawn the inner view.
            handle.clone().raze(entt.id(), world);
        };
        self.inner = None;
        world.despawn_owned_recursive(view_entity);
    }
}

impl<W: ViewFactory> From<W> for ViewHandle
where
    W: Send + Sync + 'static,
{
    fn from(value: W) -> Self {
        ViewHandle::new(ViewFactoryState::new(value))
    }
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
