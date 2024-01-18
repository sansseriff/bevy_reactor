use bevy::prelude::*;

use crate::{
    node_span::NodeSpan,
    scope::TrackingScope,
    view::{View, ViewContext},
    Cx,
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

    fn build(&mut self, _view_entity: Entity, vc: &mut ViewContext) {
        assert!(self.node.is_none());
        self.node = Some(
            vc.world
                .spawn((TextBundle {
                    text: Text::from_section(self.text.clone(), TextStyle { ..default() }),
                    ..default()
                },))
                .id(),
        );
    }

    fn raze(&mut self, view_entity: Entity, world: &mut World) {
        // Delete the tracking scope.
        world.entity_mut(view_entity).remove::<TrackingScope>();

        // Delete the display node.
        world
            .entity_mut(self.node.expect("Razing unbuilt TextNode"))
            .despawn();
    }
}

/// A UI element that displays text that is dynamically computed.
pub struct DynTextView<F: FnMut(&mut Cx) -> String> {
    /// The visible UI node for this element.
    node: Option<Entity>,

    /// The text to display
    text: F,
}

impl<F: FnMut(&mut Cx) -> String> DynTextView<F> {
    pub fn new(text: F) -> Self {
        Self { node: None, text }
    }
}

impl<F: FnMut(&mut Cx) -> String> View for DynTextView<F> {
    fn nodes(&self) -> NodeSpan {
        NodeSpan::Node(self.node.unwrap())
    }

    fn build(&mut self, view_entity: Entity, vc: &mut ViewContext) {
        assert!(self.node.is_none());
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
        self.node = node;
        vc.world.entity_mut(view_entity).insert(tracking);
    }

    fn react(&mut self, _view_entity: Entity, vc: &mut ViewContext, tracking: &mut TrackingScope) {
        let mut cx = Cx::new(&(), vc.world, tracking);
        let text = (self.text)(&mut cx);
        vc.world
            .entity_mut(self.node.unwrap())
            .get_mut::<Text>()
            .unwrap()
            .sections[0]
            .value = text;
    }

    fn raze(&mut self, _view_entity: Entity, world: &mut World) {
        world
            .entity_mut(self.node.expect("Razing unbuilt DynTextNode"))
            .despawn();
    }
}
