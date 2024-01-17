use bevy::prelude::*;

use crate::{
    node_span::NodeSpan,
    scope::TrackingScope,
    view::{View, ViewContext},
    Cx, IntoView, ViewHandle,
};

/// A UI element that displays text
pub struct TextView {
    /// The visible UI node for this element.
    node: Option<Entity>,

    /// The text to display
    text: String,
}

impl TextView {
    pub fn new(text: String) -> Self {
        Self { node: None, text }
    }
}

impl View for TextView {
    fn nodes(&self) -> NodeSpan {
        NodeSpan::Node(self.node.unwrap())
    }

    fn build(&mut self, vc: &mut ViewContext) {
        if self.node.is_none() {
            self.node = Some(
                vc.world
                    .spawn((TextBundle {
                        text: Text::from_section(self.text.clone(), TextStyle { ..default() }),
                        ..default()
                    },))
                    .id(),
            );
        }
    }
}

impl IntoView for TextView {
    fn into_view(self) -> Box<dyn View + Send + Sync> {
        Box::new(self)
    }

    fn into_handle(self, world: &mut World) -> Entity {
        world.spawn(ViewHandle::new(self)).id()
    }
}

/// A UI element that displays text that is dynamically computed.
pub struct DynTextView<F: Fn(&mut Cx) -> String> {
    /// The visible UI node for this element.
    node: Option<Entity>,

    /// The text to display
    text: F,
}

impl<F: Fn(&mut Cx) -> String> DynTextView<F> {
    pub fn new(text: F) -> Self {
        Self { node: None, text }
    }
}

impl<F: Fn(&mut Cx) -> String> View for DynTextView<F> {
    fn nodes(&self) -> NodeSpan {
        NodeSpan::Node(self.node.unwrap())
    }

    fn build(&mut self, vc: &mut ViewContext) {
        if self.node.is_none() {
            let mut tracking = TrackingScope::new(vc.world.change_tick());
            let mut cx = Cx::new(&(), vc.world, &mut tracking);
            let text = (self.text)(&mut cx);
            let node = Some(
                vc.world
                    .spawn((TextBundle {
                        text: Text::from_section(text, TextStyle { ..default() }),
                        ..default()
                    },))
                    .id(),
            );
            // let reaction: Arc::new(|_| self.nodes());
            // vc.world
            //     .entity_mut(node.unwrap())
            //     .insert(Reaction { inner: |_| {} });
            self.node = node;
        }
    }
}
