use std::sync::{Arc, Mutex};

use bevy::prelude::*;

use crate::{
    node_span::NodeSpan, view::View, view_tuple::ViewTuple, DespawnScopes, IntoView, ViewHandle,
    ViewRef,
};

struct FragmentChild {
    view: ViewRef,
    entity: Option<Entity>,
}

/// A basic UI fragment
#[derive(Default)]
pub struct Fragment {
    /// Children of this fragment.
    children: Vec<FragmentChild>,
}

impl Fragment {
    /// Construct a new `Fragment`.
    pub fn new<V: ViewTuple>(views: V) -> Self {
        let mut child_views: Vec<ViewRef> = Vec::new();
        views.get_handles(&mut child_views);
        Self {
            children: child_views
                .iter()
                .map(|v| FragmentChild {
                    view: v.clone(),
                    entity: None,
                })
                .collect(),
        }
    }
}

impl View for Fragment {
    fn nodes(&self) -> NodeSpan {
        let child_spans: Vec<NodeSpan> = self
            .children
            .iter()
            .map(|item| item.view.lock().unwrap().nodes())
            .collect();
        NodeSpan::Fragment(child_spans.into_boxed_slice())
    }

    fn build(&mut self, view_entity: Entity, world: &mut World) {
        // Build child nodes.
        for child in self.children.iter_mut() {
            child.entity = Some(ViewHandle::spawn(&child.view, view_entity, world));
        }
    }

    fn raze(&mut self, view_entity: Entity, world: &mut World) {
        // Raze all child views
        for child in self.children.drain(..) {
            let inner = child.view.clone();
            inner.lock().unwrap().raze(child.entity.unwrap(), world);
            // Child raze() will despawn itself.
        }

        world.despawn_owned_recursive(view_entity);
    }
}

impl IntoView for Fragment {
    fn into_view(self) -> ViewRef {
        Arc::new(Mutex::new(self))
    }
}
