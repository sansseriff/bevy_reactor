use std::sync::{Arc, Mutex};

use bevy::prelude::{Component, Entity, World};
use bevy_reactor_signals::TrackingScope;

use crate::TextStatic;

/// Trait that defines a view, which is a template that constructs a hierarchy of
/// entities and components.
///
/// Views are also reactions, and must implement the `react` method.
pub trait View {
    /// Initialize the view, creating any entities needed.
    ///
    /// Arguments:
    /// * `owner`: The entity that owns this view.
    /// * `world`: The Bevy world.
    /// * `scope`: The parent tracking scope which owns any reactions created by this view.
    /// * `out`: A mutable reference to a vector where the output entities will be stored.
    fn build(
        &mut self,
        owner: Entity,
        world: &mut World,
        scope: &mut TrackingScope,
        out: &mut Vec<Entity>,
    );

    /// Convert this View into a view root which can be spawned.
    fn to_root(self) -> (ViewCell, ViewRoot)
    where
        Self: Sized + Send + Sync + 'static,
    {
        (ViewCell(Arc::new(Mutex::new(self))), ViewRoot)
    }
}

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
    fn build(
        &mut self,
        owner: Entity,
        world: &mut World,
        scope: &mut TrackingScope,
        out: &mut Vec<Entity>,
    ) {
        if let Some(view) = self {
            view.build(owner, world, scope, out);
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
    fn build(
        &mut self,
        owner: Entity,
        world: &mut World,
        scope: &mut TrackingScope,
        out: &mut Vec<Entity>,
    ) {
    }
}
