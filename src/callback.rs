use bevy::prelude::*;

use crate::Cx;

pub(crate) trait CallbackFnRef<P> {
    fn call(&self, cx: &mut Cx<P>);
}

impl<P, F: Fn(&mut Cx<P>)> CallbackFnRef<P> for F {
    fn call(&self, cx: &mut Cx<P>) {
        self(cx);
    }
}

pub(crate) trait CallbackFnMutRef<P> {
    fn call(&mut self, cx: &mut Cx<P>);
}

impl<P, F: FnMut(&mut Cx<P>)> CallbackFnMutRef<P> for F {
    fn call(&mut self, cx: &mut Cx<P>) {
        self(cx);
    }
}

/// Contains a boxed, type-erased callback.
#[derive(Component)]
pub(crate) struct CallbackFnValue<P> {
    pub(crate) inner: Option<Box<dyn CallbackFnRef<P> + Send + Sync>>,
}

#[derive(Component)]
pub(crate) struct CallbackFnMutValue<P> {
    pub(crate) inner: Option<Box<dyn CallbackFnMutRef<P> + Send + Sync>>,
}

/// Contains a reference to a callback. `P` is the type of the props.
#[derive(Copy, Clone, PartialEq)]
pub struct CallbackFn<P = ()> {
    pub(crate) id: Entity,
    pub(crate) marker: std::marker::PhantomData<P>,
}
