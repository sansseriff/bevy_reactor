use bevy::{
    prelude::{Entity, World},
    ui,
};
use bevy_mod_stylebuilder::{StyleBuilder, StyleTuple};
use bevy_reactor_signals::TrackingScope;

use crate::effect::Effect;

/// Inserts a static, pre-constructed bundle into the target entity. No reactivity.
pub struct ApplyStylesEffect<S: StyleTuple> {
    pub(crate) styles: S,
}

impl<S: StyleTuple> Effect for ApplyStylesEffect<S> {
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
