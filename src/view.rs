use std::sync::{Arc, Mutex};

use bevy::ecs::{
    component::Component,
    entity::Entity,
    world::{Command, World},
};
use bevy_reactor_signals::{Cx, Reaction, Signal, TrackingScope};

use crate::{node_span::NodeSpan, text::TextStatic, TextComputed};

/// Trait that defines a view, which is a template that constructs a hierarchy of
/// entities and components.
///
/// Lifecycle: To create a view, use [`ViewHandle::spawn`]. This creates an entity to hold the view,
/// and which drives the reaction system. When the view is no longer needed, call [`View::raze`].
/// This will destroy the view entity, and all of its children and display nodes.
///
/// Views are also reactions, and must implement the `react` method.
#[allow(unused_variables)]
pub trait View: Reaction {
    /// Returns the display nodes produced by this `View`.
    fn nodes(&self) -> NodeSpan;

    /// Initialize the view, creating any entities needed.
    ///
    /// Arguments:
    /// * `view_entity`: The entity that owns this view.
    /// * `world`: The Bevy world.
    fn build(&mut self, view_entity: Entity, world: &mut World);

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

    /// Coerce this view into a [`Reaction`].
    fn to_reaction_ref(&self) -> &dyn Reaction
    where
        Self: Sized + Send + Sync + 'static,
    {
        self
    }
}

#[derive(Component)]
/// Component which holds the top level of the view hierarchy.
pub struct ViewRoot(pub(crate) Arc<Mutex<dyn View + Sync + Send + 'static>>);

impl ViewRoot {
    /// Construct a new [`ViewRoot`].
    pub fn new(view: impl View + Sync + Send + 'static) -> Self {
        Self(Arc::new(Mutex::new(view)))
    }

    /// Despawn the view, including the display nodes, and all descendant views.
    pub fn despawn(&mut self, root: Entity, world: &mut World) {
        self.0.lock().unwrap().raze(root, world);
        world.entity_mut(root).despawn();
    }
}

/// Command which can be used to despawn a view root and all of it's contents.
pub struct DespawnViewRoot(Entity);

impl DespawnViewRoot {
    /// Construct a new [`DespawnViewRoot`] command.
    pub fn new(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Command for DespawnViewRoot {
    fn apply(self, world: &mut World) {
        let entt = world.entity_mut(self.0);
        let Some(root) = entt.get::<ViewRoot>() else {
            return;
        };
        let handle = root.0.clone();
        let mut view = handle.lock().unwrap();
        view.raze(self.0, world);
        // let entt = world.entity_mut(self.0);
        // entt.despawn();
    }
}

/// A reference to a [`View`] which can be passed around as a parameter.
pub struct ViewRef(pub(crate) Arc<Mutex<dyn View + Sync + Send + 'static>>);

impl ViewRef {
    /// Construct a new [`ViewRef`] from a [`View`].
    pub fn new(view: impl View + Sync + Send + 'static) -> Self {
        Self(Arc::new(Mutex::new(view)))
    }

    /// Given a view template, construct a new view. This creates an entity to hold the view
    /// and the view handle, and then calls [`View::build`] on the view. The resuling entity
    /// is part of the template invocation hierarchy, it is not a display node.
    pub fn spawn(_view: &ViewRef, _parent: Entity, _world: &mut World) -> Entity {
        todo!();
    }

    /// Returns the display nodes produced by this `View`.
    pub fn nodes(&self) -> NodeSpan {
        self.0.lock().unwrap().nodes()
    }

    /// Destroy the view, including the display nodes, and all descendant views.
    pub fn raze(&self, view_entity: Entity, world: &mut World) {
        self.0.lock().unwrap().raze(view_entity, world);
    }
}

impl Clone for ViewRef {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Default for ViewRef {
    fn default() -> Self {
        Self::new(EmptyView)
    }
}

/// Trait that allows a type to be converted into a [`ViewRef`].
pub trait IntoView {
    /// Convert the type into a [`ViewRef`].
    fn into_view(self) -> ViewRef;
}

impl IntoView for ViewRef {
    fn into_view(self) -> ViewRef {
        self
    }
}

impl IntoView for () {
    fn into_view(self) -> ViewRef {
        ViewRef::new(EmptyView)
    }
}

impl IntoView for &str {
    fn into_view(self) -> ViewRef {
        ViewRef::new(TextStatic::new(self.to_string()))
    }
}

impl IntoView for Signal<String> {
    fn into_view(self) -> ViewRef {
        ViewRef::new(TextComputed::new(move |rcx| self.get_clone(rcx)))
    }
}

impl IntoView for String {
    fn into_view(self) -> ViewRef {
        ViewRef::new(TextStatic::new(self))
    }
}

impl<V: IntoView> IntoView for Option<V> {
    fn into_view(self) -> ViewRef {
        match self {
            Some(v) => v.into_view(),
            None => ViewRef::new(EmptyView),
        }
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

impl Reaction for EmptyView {
    fn react(&mut self, _owner: Entity, _world: &mut World, _tracking: &mut TrackingScope) {}
}

/// Trait that defines a factory object that can construct a [`View`] from a reactive context.
/// Similar to a `PresenterFn`, but allows the template to be defined as a type, rather than as
/// a function.
pub trait ViewTemplate {
    /// Create the view for the control.
    fn create(&self, cx: &mut Cx) -> impl IntoView;

    /// Associate this view template with a state object that tracks the nodes created by
    /// the view. Consumes the template.
    fn to_root(self) -> ViewRoot
    where
        Self: Sized + Send + Sync + 'static,
    {
        todo!();
    }
}

impl<VT: ViewTemplate + Send + Sync + 'static> IntoView for VT {
    fn into_view(self) -> ViewRef {
        todo!();
    }
}
