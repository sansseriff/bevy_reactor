use std::any::Any;

use bevy::prelude::*;

// use crate::Cx;

// pub trait CallbackFnRef<P>: Fn(&Cx<P>) + Send + Sync + 'static {}
// pub trait CallbackFnMutRef<P>: FnMut(&mut Cx<P>) + Send + Sync + 'static {}

/// Contains a boxed, type-erased callback.
#[derive(Component)]
pub(crate) struct CallbackFnValue {
    pub(crate) inner: Option<Box<dyn Any + Send + Sync + 'static>>,
}

#[derive(Component)]
pub(crate) struct CallbackFnMutValue {
    pub(crate) inner: Option<Box<dyn Any + Send + Sync + 'static>>,
}

/// Contains a reference to a callback. `P` is the type of the props.
pub struct CallbackFn<P> {
    pub(crate) id: Entity,
    pub(crate) marker: std::marker::PhantomData<P>,
}
