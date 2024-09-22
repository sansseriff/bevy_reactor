use bevy::prelude::*;
use bevy_mod_stylebuilder::{get_inherited_text_styles, UseInheritedTextStyles};
use bevy_reactor_signals::{Rcx, Reaction, ReactionCell, TrackingScope};

use crate::{view::View, IntoView};

/// A UI element that displays text
pub struct TextStatic {
    /// The text to display
    text: String,
}

impl TextStatic {
    /// Construct a new static text view.
    pub fn new(text: String) -> Self {
        Self { text }
    }
}

impl View for TextStatic {
    fn build(
        &mut self,
        parent: Entity,
        world: &mut World,
        _scope: &mut TrackingScope,
        out: &mut Vec<Entity>,
    ) {
        let style = get_inherited_text_styles(world, parent).unwrap_or_default();
        let node = world
            .spawn((
                Name::new("TextStatic"),
                TextBundle {
                    text: Text::from_section(self.text.clone(), style),
                    ..default()
                },
                UseInheritedTextStyles,
            ))
            .id();
        out.push(node);
    }
}

impl IntoView for TextStatic {
    fn into_view(self) -> Box<dyn View + 'static> {
        Box::new(self)
    }
}

/// A UI element that displays text that is dynamically computed.
pub struct TextComputed<F: FnMut(&Rcx) -> String + Send + Sync + 'static> {
    /// The function that produces the text to display
    text: Option<F>,
}

impl<F: FnMut(&Rcx) -> String + Send + Sync + 'static> TextComputed<F> {
    /// Construct a new computed text view.
    pub fn new(text: F) -> Self {
        Self { text: Some(text) }
    }
}

impl<F: FnMut(&Rcx) -> String + Send + Sync + 'static> View for TextComputed<F> {
    fn build(
        &mut self,
        parent: Entity,
        world: &mut World,
        _scope: &mut TrackingScope,
        out: &mut Vec<Entity>,
    ) {
        let style = get_inherited_text_styles(world, parent).unwrap_or_default();
        let node = world.spawn_empty().id();
        let mut tracking = TrackingScope::new(world.change_tick());
        let re = Rcx::new(world, node, &mut tracking);
        let mut text_fn = self
            .text
            .take()
            .expect("TextComputed text function missing");
        let text = (text_fn)(&re);
        world.entity_mut(node).insert((
            tracking,
            Name::new("TextComputed"),
            TextBundle {
                text: Text::from_section(text, style),
                ..default()
            },
            UseInheritedTextStyles,
            ReactionCell::new(TextComputedReaction {
                node,
                text: text_fn,
            }),
        ));
        out.push(node);
    }
}

/// A UI element that displays text that is dynamically computed.
struct TextComputedReaction<F: FnMut(&Rcx) -> String + Send + Sync + 'static> {
    /// The visible UI node for this element.
    node: Entity,

    /// The function that produces the text to display
    text: F,
}

impl<F: FnMut(&Rcx) -> String + Send + Sync + 'static> Reaction for TextComputedReaction<F> {
    fn react(&mut self, owner: Entity, world: &mut World, tracking: &mut TrackingScope) {
        let re = Rcx::new(world, owner, tracking);
        let text = (self.text)(&re);
        world
            .entity_mut(self.node)
            .get_mut::<Text>()
            .unwrap()
            .sections[0]
            .value = text;
    }
}

impl<F: Send + Sync + 'static + FnMut(&Rcx) -> String> IntoView for TextComputed<F> {
    fn into_view(self) -> Box<dyn View + 'static> {
        Box::new(self)
    }
}
