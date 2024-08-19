use bevy::prelude::*;
use bevy_mod_stylebuilder::UseInheritedTextStyles;
use bevy_reactor_signals::{
    DespawnScopes, Rcx, Reaction, ReactionCell, ReactionThunk, TrackingScope,
};

use crate::{view::View, IntoView};

/// A UI element that displays text
pub struct TextStatic {
    /// The visible UI node for this element.
    node: Option<Entity>,

    /// The text to display
    text: String,
}

impl TextStatic {
    /// Construct a new static text view.
    pub fn new(text: String) -> Self {
        Self { node: None, text }
    }
}

impl View for TextStatic {
    fn nodes(&self, out: &mut Vec<Entity>) {
        if let Some(node) = self.node {
            out.push(node);
        }
    }

    fn build(&mut self, _view_entity: Entity, world: &mut World) {
        assert!(self.node.is_none());
        self.node = Some(
            world
                .spawn((
                    TextBundle {
                        text: Text::from_section(self.text.clone(), TextStyle { ..default() }),
                        ..default()
                    },
                    UseInheritedTextStyles,
                ))
                .id(),
        );
    }

    fn raze(&mut self, _view_entity: Entity, world: &mut World) {
        // Delete the display node.
        let display = self.node.expect("Razing unbuilt TextStatic");
        world.entity_mut(display).remove_parent();
        world.entity_mut(display).despawn();
        self.node = None;
    }
}

impl Reaction for TextStatic {
    fn react(&mut self, _view_entity: Entity, _world: &mut World, _tracking: &mut TrackingScope) {}
}

impl IntoView for TextStatic {
    fn into_view(self) -> Box<dyn View + Send + Sync + 'static> {
        Box::new(self)
    }
}

/// A UI element that displays text that is dynamically computed.
pub struct TextComputed<F: FnMut(&Rcx) -> String + Send + Sync + 'static> {
    /// The visible UI node for this element.
    node: Option<Entity>,

    /// The function that produces the text to display
    text: Option<F>,
}

impl<F: FnMut(&Rcx) -> String + Send + Sync + 'static> TextComputed<F> {
    /// Construct a new computed text view.
    pub fn new(text: F) -> Self {
        Self {
            node: None,
            text: Some(text),
        }
    }
}

impl<F: FnMut(&Rcx) -> String + Send + Sync + 'static> View for TextComputed<F> {
    fn nodes(&self, out: &mut Vec<Entity>) {
        if let Some(node) = self.node {
            out.push(node);
        }
    }

    fn build(&mut self, view_entity: Entity, world: &mut World) {
        assert!(self.node.is_none());
        let mut tracking = TrackingScope::new(world.change_tick());
        let re = Rcx::new(world, view_entity, &mut tracking);
        let mut text_fn = self
            .text
            .take()
            .expect("TextComputed text function missing");
        let text = (text_fn)(&re);
        let node = Some(
            world
                .spawn((
                    TextBundle {
                        text: Text::from_section(text, TextStyle { ..default() }),
                        ..default()
                    },
                    UseInheritedTextStyles,
                ))
                .id(),
        );
        self.node = node;
        world.entity_mut(view_entity).insert((
            tracking,
            ReactionThunk::for_reaction::<TextComputedReaction<F>>(),
            ReactionCell::new(TextComputedReaction {
                node,
                text: text_fn,
            }),
        ));
    }

    fn raze(&mut self, view_entity: Entity, world: &mut World) {
        let display = self.node.expect("Razing unbuilt TextComputed");
        world.entity_mut(display).remove_parent();
        world.entity_mut(display).despawn();
        world.despawn_owned_recursive(view_entity);
        self.node = None;
    }
}

/// A UI element that displays text that is dynamically computed.
struct TextComputedReaction<F: FnMut(&Rcx) -> String + Send + Sync + 'static> {
    /// The visible UI node for this element.
    node: Option<Entity>,

    /// The function that produces the text to display
    text: F,
}

impl<F: FnMut(&Rcx) -> String + Send + Sync + 'static> Reaction for TextComputedReaction<F> {
    fn react(&mut self, owner: Entity, world: &mut World, tracking: &mut TrackingScope) {
        let re = Rcx::new(world, owner, tracking);
        let text = (self.text)(&re);
        world
            .entity_mut(self.node.unwrap())
            .get_mut::<Text>()
            .unwrap()
            .sections[0]
            .value = text;
    }
}

impl<F: Send + Sync + 'static + FnMut(&Rcx) -> String> IntoView for TextComputed<F> {
    fn into_view(self) -> Box<dyn View + Send + Sync + 'static> {
        Box::new(self)
    }
}
