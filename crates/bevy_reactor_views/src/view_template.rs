use std::sync::{Arc, Mutex};

use bevy::prelude::{BuildChildren, Entity, World};
use bevy_reactor_signals::{Cx, TrackingScope};

use crate::{view::ViewCell, IntoView, View, ViewRoot};

/// Trait that defines a template object that can construct a [`View`] from a reactive context.
/// View templates both contain [`Views`], and can be invoked as Views themselves. The `create()`
/// method returns a [`View`] that is subsequently built.
pub trait ViewTemplate {
    fn create(&self, cx: &mut Cx) -> impl IntoView;

    /// Convert this View into a view root which can be spawned.
    fn to_root(self) -> (ViewCell, ViewRoot)
    where
        Self: Sized + Send + Sync + 'static,
    {
        (
            ViewCell(Arc::new(Mutex::new(ViewTemplateView {
                template: self,
                root: None,
            }))),
            ViewRoot,
        )
    }
}

impl<VT: ViewTemplate + Send + Sync + 'static> IntoView for VT {
    fn into_view(self) -> Box<dyn View + Send + Sync + 'static> {
        Box::new(ViewTemplateView {
            template: self,
            root: None,
        })
    }
}

impl<VT: ViewTemplate + 'static> From<VT> for Box<dyn View> {
    fn from(value: VT) -> Self {
        Box::new(ViewTemplateView {
            template: value,
            root: None,
        })
    }
}

pub struct ViewTemplateView<VT: ViewTemplate> {
    template: VT,
    root: Option<(Entity, Box<dyn View + Send + Sync + 'static>)>,
}

impl<VT: ViewTemplate> View for ViewTemplateView<VT> {
    fn build(
        &mut self,
        owner: Entity,
        world: &mut World,
        scope: &mut TrackingScope,
        out: &mut Vec<Entity>,
    ) {
        assert!(self.root.is_none());
        let mut tracking = TrackingScope::new(world.change_tick());
        let root = world.spawn_empty().set_parent(owner).id();
        let mut cx = Cx::new(world, root, &mut tracking);
        let mut view = self.template.create(&mut cx).into_view();
        world.entity_mut(root).insert(tracking);
        view.build(root, world, scope, out);
        self.root = Some((root, view));
        // world.entity_mut(inner).insert(ViewHandle(view.0));
    }
}
