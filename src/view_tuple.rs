use crate::{IntoView, View};
use bevy::ecs::{entity::Entity, world::World};
use impl_trait_for_tuples::*;

// ViewTuple

#[doc(hidden)]
pub trait ViewTuple {
    fn gather(self, out: &mut Vec<Box<dyn View + Send + Sync>>);
    fn get_handles(self, world: &mut World, out: &mut Vec<Entity>);
}

impl<I: IntoView> ViewTuple for I {
    fn gather(self, out: &mut Vec<Box<dyn View + Send + Sync>>) {
        out.push(self.into_view());
    }

    fn get_handles(self, world: &mut World, out: &mut Vec<Entity>) {
        out.push(self.into_handle(world));
    }
}

impl<I: IntoView> ViewTuple for Option<I> {
    fn gather(self, out: &mut Vec<Box<dyn View + Send + Sync>>) {
        if let Some(view) = self {
            out.push(view.into_view());
        }
    }

    fn get_handles(self, world: &mut World, out: &mut Vec<Entity>) {
        if let Some(view) = self {
            out.push(view.into_handle(world));
        }
    }
}

#[impl_for_tuples(1, 15)]
#[tuple_types_custom_trait_bound(IntoView)]
impl ViewTuple for Tuple {
    #[rustfmt::skip]
    fn gather(self, out: &mut Vec<Box<dyn View + Send + Sync>>) {
        for_tuples!(#( out.push(self.Tuple.into_view()); )*)
    }

    fn get_handles(self, world: &mut World, out: &mut Vec<Entity>) {
        for_tuples!(#( out.push(self.Tuple.into_handle(world)); )*)
    }
}
