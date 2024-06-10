use crate::{effect_target::EffectTarget, Element, EntityEffect};
use bevy::{prelude::*, ui};
pub use bevy_mod_stylebuilder::*;
use bevy_reactor_signals::TrackingScope;

/// Inserts a static, pre-constructed bundle into the target entity. No reactivity.
pub struct ApplyStylesEffect<S: StyleTuple> {
    pub(crate) styles: S,
}

impl<S: StyleTuple> EntityEffect for ApplyStylesEffect<S> {
    // For a style builder, run the builder over the target entity.
    fn start(
        &mut self,
        _owner: Entity,
        target: Entity,
        world: &mut World,
        _tracking: &mut TrackingScope,
    ) {
        let mut target = world.entity_mut(target);
        let mut style = ui::Style::default();
        if let Some(s) = target.get::<ui::Style>() {
            style.clone_from(s);
        }
        let mut sb = StyleBuilder::new(&mut target, style);
        self.styles.apply(&mut sb);
        sb.finish();
    }
}

/// Trait to add a collection of styles to the receiver.
pub trait WithStyles {
    /// Apply a set of style builders to a target.
    fn style<S: StyleTuple + 'static>(self, styles: S) -> Self;
}

impl<B: Bundle + Default> WithStyles for Element<B> {
    fn style<S: StyleTuple + 'static>(mut self, styles: S) -> Self {
        self.add_effect(Box::new(ApplyStylesEffect { styles }));
        self
    }
}
