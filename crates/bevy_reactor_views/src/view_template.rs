use std::sync::{Arc, Mutex};

use bevy::prelude::{Entity, World};
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
            ViewCell(Arc::new(Mutex::new(ViewTemplateView { template: self }))),
            ViewRoot,
        )
    }
}

impl<VT: ViewTemplate + Send + Sync + 'static> IntoView for VT {
    fn into_view(self) -> Box<dyn View + Send + Sync + 'static> {
        Box::new(ViewTemplateView { template: self })
    }
}

impl<VT: ViewTemplate + 'static> From<VT> for Box<dyn View> {
    fn from(value: VT) -> Self {
        Box::new(ViewTemplateView { template: value })
    }
}

pub struct ViewTemplateView<VT: ViewTemplate> {
    template: VT,
}

impl<VT: ViewTemplate> View for ViewTemplateView<VT> {
    fn build(
        &mut self,
        owner: Entity,
        world: &mut World,
        scope: &mut TrackingScope,
        out: &mut Vec<Entity>,
    ) {
        let mut cx = Cx::new(world, owner, scope);
        let mut view = self.template.create(&mut cx).into_view();
        view.build(owner, world, scope, out);
    }
}
