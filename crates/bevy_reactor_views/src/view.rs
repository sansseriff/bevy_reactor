use std::sync::{Arc, Mutex};

use bevy::prelude::{Component, Entity, World};

use crate::TextStatic;

/// Trait that defines a view, which is a template that constructs a hierarchy of
/// entities and components.
///
/// Views are also reactions, and must implement the `react` method.
#[allow(unused_variables)]
pub trait View {
    /// Returns the display nodes produced by this `View`.
    fn nodes(&self, out: &mut Vec<Entity>);

    /// Initialize the view, creating any entities needed.
    ///
    /// Arguments:
    /// * `owner`: The entity that owns this view.
    /// * `world`: The Bevy world.
    fn build(&mut self, owner: Entity, world: &mut World);

    /// Destroy the view, including the display nodes, and all descendant views.
    fn raze(&mut self, owner: Entity, world: &mut World);

    /// Notification from child views that the child display nodes have changed and need
    /// to be re-attached to the parent. This is optional, and need only be implemented for
    /// views which have display nodes that have child display nodes (like [`Element`]).
    ///
    /// Returns `true` if the view was able to update its display nodes. If it returns `false`,
    /// then it means that this view is only a thin wrapper for other views, and doesn't actually
    /// have any display nodes of its own, in which case the parent view will need to handle the
    /// change.
    fn children_changed(&mut self, owner: Entity, world: &mut World) -> bool {
        false
    }

    /// Convert this View into a view root which can be spawned.
    fn to_root(self) -> (ViewCell, ViewRoot)
    where
        Self: Sized + Send + Sync + 'static,
    {
        (ViewCell(Arc::new(Mutex::new(self))), ViewRoot)
    }
}

/// Marker on a [`View`] entity to indicate that it's output [`Vec<Entity>`] has changed, and that
/// the parent needs to re-attach it's children.
#[derive(Component)]
pub struct ViewOutputChanged;

#[derive(Component)]
pub struct ViewRoot;

#[derive(Component)]
pub struct ViewCell(pub(crate) Arc<Mutex<dyn View + Send + Sync + 'static>>);

impl ViewCell {
    pub fn new<V: View + Send + Sync + 'static>(view: V) -> Self {
        Self(Arc::new(Mutex::new(view)))
    }
}

pub trait IntoView {
    fn into_view(self) -> Box<dyn View + Send + Sync + 'static>;
}

impl IntoView for &str {
    fn into_view(self) -> Box<dyn View + Send + Sync + 'static> {
        Box::new(TextStatic::new(self.to_string()))
    }
}

impl IntoView for String {
    fn into_view(self) -> Box<dyn View + Send + Sync + 'static> {
        Box::new(TextStatic::new(self))
    }
}

pub trait IntoViewVec {
    fn into_view_vec(self, out: &mut Vec<Box<dyn View + Send + Sync + 'static>>);
}

impl<V: IntoView> IntoViewVec for V {
    fn into_view_vec(self, out: &mut Vec<Box<dyn View + Send + Sync + 'static>>) {
        out.push(self.into_view());
    }
}

impl<V: View> View for Option<V> {
    fn nodes(&self, out: &mut Vec<Entity>) {
        if let Some(view) = self {
            view.nodes(out);
        }
    }

    fn build(&mut self, owner: Entity, world: &mut World) {
        if let Some(view) = self {
            view.build(owner, world);
        }
    }

    fn raze(&mut self, owner: Entity, world: &mut World) {
        if let Some(view) = self {
            view.raze(owner, world);
        }
    }
}

macro_rules! impl_view_tuple {
    ( $($view: ident, $idx: tt);+ ) => {
        impl<$(
            $view: IntoViewVec + Send + Sync + 'static,
        )+> IntoViewVec for ( $( $view, )* ) {
            fn into_view_vec(self, out: &mut Vec<Box<dyn View + Send + Sync + 'static>>) {
                $( self.$idx.into_view_vec(out); )*
            }
        }
    };
}

impl_view_tuple!(V0, 0);
impl_view_tuple!(V0, 0; V1, 1);
impl_view_tuple!(V0, 0; V1, 1; V2, 2);
impl_view_tuple!(V0, 0; V1, 1; V2, 2; V3, 3);
impl_view_tuple!(V0, 0; V1, 1; V2, 2; V3, 3; V4, 4);
impl_view_tuple!(V0, 0; V1, 1; V2, 2; V3, 3; V4, 4; V5, 5);
impl_view_tuple!(V0, 0; V1, 1; V2, 2; V3, 3; V4, 4; V5, 5; V6, 6);
impl_view_tuple!(V0, 0; V1, 1; V2, 2; V3, 3; V4, 4; V5, 5; V6, 6; V7, 7);
impl_view_tuple!(V0, 0; V1, 1; V2, 2; V3, 3; V4, 4; V5, 5; V6, 6; V7, 7; V8, 8);
impl_view_tuple!(V0, 0; V1, 1; V2, 2; V3, 3; V4, 4; V5, 5; V6, 6; V7, 7; V8, 8; V9, 9);
impl_view_tuple!(V0, 0; V1, 1; V2, 2; V3, 3; V4, 4; V5, 5; V6, 6; V7, 7; V8, 8; V9, 9; V10, 10);
impl_view_tuple!(V0, 0; V1, 1; V2, 2; V3, 3; V4, 4; V5, 5; V6, 6; V7, 7; V8, 8; V9, 9; V10, 10; V11, 11);
impl_view_tuple!(V0, 0; V1, 1; V2, 2; V3, 3; V4, 4; V5, 5; V6, 6; V7, 7; V8, 8; V9, 9; V10, 10; V11, 11; V12, 12);
impl_view_tuple!(V0, 0; V1, 1; V2, 2; V3, 3; V4, 4; V5, 5; V6, 6; V7, 7; V8, 8; V9, 9; V10, 10; V11, 11; V12, 12; V13, 13);
impl_view_tuple!(V0, 0; V1, 1; V2, 2; V3, 3; V4, 4; V5, 5; V6, 6; V7, 7; V8, 8; V9, 9; V10, 10; V11, 11; V12, 12; V13, 13; V14, 14);
impl_view_tuple!(V0, 0; V1, 1; V2, 2; V3, 3; V4, 4; V5, 5; V6, 6; V7, 7; V8, 8; V9, 9; V10, 10; V11, 11; V12, 12; V13, 13; V14, 14; V15, 15);

#[allow(unused)]
impl View for () {
    fn nodes(&self, out: &mut Vec<Entity>) {}
    fn build(&mut self, owner: Entity, world: &mut World) {}
    fn raze(&mut self, owner: Entity, world: &mut World) {}
}
