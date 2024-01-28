use std::any::Any;

use bevy::prelude::*;

use crate::Cx;

pub trait CallbackFnRef<P>: Fn(&Cx<P>) + Send + Sync + 'static {}
pub trait CallbackFnMutRef<P>: FnMut(&mut Cx<P>) + Send + Sync + 'static {}

/// Contains a mutable reactive value.
#[derive(Component)]
pub(crate) struct CallbackFnValue {
    pub(crate) inner: Option<Box<dyn Any + Send + Sync + 'static>>,
}

/// Contains a reference to a callback.
pub struct CallbackFn<T> {
    pub(crate) id: Entity,
    pub(crate) marker: std::marker::PhantomData<T>,
}

// Contains a reference to a mutable callback.
// pub struct CallbackFnMut<T> {
//     pub(crate) id: Entity,
//     pub(crate) marker: std::marker::PhantomData<T>,
// }
