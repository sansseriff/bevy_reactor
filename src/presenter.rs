use std::sync::{Arc, Mutex};

use bevy::ecs::{entity::Entity, world::World};

use crate::{
    node_span::NodeSpan, Cx, DespawnScopes, IntoView, TrackingScope, View, ViewHandle, ViewRef,
};

/// A trait that allows methods to be added to presenter function references.
pub trait PresenterFn<F: 'static>: Sized + Send + Sync + Copy + 'static {
    /// The type of properties expected by this presenter.
    type Props: Send + Sync;

    /// The type of view produced by this presenter.
    type View: View + Sync + Send;

    /// Used to invoke a presenter. This binds a set of properties to the presenter function, and
    /// constructs a new [`ViewHandle`].
    fn bind(self, props: Self::Props) -> Bind<F, Self>;

    /// Method which calls the presenter, creating the [`View`].
    fn call(&mut self, cx: &mut Cx<Self::Props>) -> Self::View;
}

impl<
        V: View + Sync + Send + 'static,
        P: Send + Sync + 'static,
        F: FnMut(&mut Cx<P>) -> V + Copy + Send + Sync + 'static,
    > PresenterFn<fn(&mut Cx<P>) -> V> for F
{
    type Props = P;
    type View = V;

    fn bind(self, props: Self::Props) -> Bind<fn(&mut Cx<P>) -> V, Self> {
        Bind::new(self, props)
    }

    fn call(&mut self, cx: &mut Cx<Self::Props>) -> Self::View {
        self(cx)
    }
}

/// A binding between a presenter function the parameters passed to it.
pub struct Bind<F: 'static, P: PresenterFn<F>> {
    /// Reference to presenter function.
    presenter: P,

    /// Props to pass to presenter function.
    props: Option<P::Props>,

    /// The view handle for the presenter output.
    inner: Option<Entity>,

    /// Display nodes.
    nodes: NodeSpan,
}

impl<F: 'static, P: PresenterFn<F>> Bind<F, P> {
    fn new(presenter: P, props: P::Props) -> Self {
        Self {
            presenter,
            props: Some(props),
            inner: None,
            nodes: NodeSpan::Empty,
        }
    }
}

impl<F: 'static, P: PresenterFn<F>> View for Bind<F, P> {
    fn nodes(&self) -> NodeSpan {
        self.nodes.clone()
    }

    fn build(&mut self, _view_entity: Entity, world: &mut World) {
        assert!(self.inner.is_none());
        assert!(self.props.is_some());
        let mut tracking = TrackingScope::new(world.read_change_tick());
        let mut cx = Cx::new(self.props.take().unwrap(), world, &mut tracking);
        let mut view = self.presenter.call(&mut cx);
        let inner = world.spawn(tracking).id();
        view.build(inner, world);
        self.nodes = view.nodes();
        world.entity_mut(inner).insert(ViewHandle::new(view));
        self.inner = Some(inner);
    }

    fn raze(&mut self, view_entity: Entity, world: &mut World) {
        assert!(self.inner.is_some());
        let mut entt = world.entity_mut(self.inner.unwrap());
        if let Some(handle) = entt.get_mut::<ViewHandle>() {
            let inner = handle.view.clone();
            // Despawn the inner view.
            inner.lock().unwrap().raze(entt.id(), world);
        };
        self.inner = None;
        world.despawn_owned_recursive(view_entity);
    }
}

impl<F: 'static, P: PresenterFn<F>> IntoView for Bind<F, P> {
    fn into_view(self) -> ViewRef {
        Arc::new(Mutex::new(self))
    }
}

// impl<V: View + Send + Sync, F: Send + Sync + 'static + Fn(&mut Cx<()>) -> V> IntoView for F {
//     fn into_view(self) -> ViewRef {
//         Arc::new(Mutex::new(Bind::new(self, ())))
//     }
// }
