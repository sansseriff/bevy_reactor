use std::sync::Arc;

use bevy::{
    prelude::{BuildChildren, Entity, World},
    ui,
};
use bevy_mod_stylebuilder::{StyleBuilder, StyleTuple};
use bevy_reactor_signals::{Rcx, Reaction, ReactionCell, TrackingScope};

use crate::effect::Effect;

/// Inserts a static, pre-constructed bundle into the target entity. No reactivity.
pub struct StaticStyleEffect<S: StyleTuple> {
    pub(crate) styles: S,
}

impl<S: StyleTuple> Effect for StaticStyleEffect<S> {
    // For a style builder, run the builder over the target entity.
    fn start(self: Box<Self>, owner: Entity, world: &mut World) {
        let mut target = world.entity_mut(owner);
        let mut style = ui::Style::default();
        if let Some(s) = target.get::<ui::Style>() {
            style.clone_from(s);
        }
        let mut sb = StyleBuilder::new(&mut target, style);
        self.styles.apply(&mut sb);
        sb.finish();
    }
}

/// Inserts a static, pre-constructed bundle into the target entity. No reactivity.
pub struct DynamicStyleEffect<D, VF: Fn(&Rcx) -> D, SF: Fn(D, &mut StyleBuilder)> {
    pub(crate) style_fn: Arc<(VF, SF)>,
}

impl<
        D: 'static,
        VF: Fn(&Rcx) -> D + Send + Sync + 'static,
        SF: Fn(D, &mut StyleBuilder) + Send + Sync + 'static,
    > Effect for DynamicStyleEffect<D, VF, SF>
{
    // For a style builder, run the builder over the target entity.
    fn start(self: Box<Self>, target: Entity, world: &mut World) {
        // Spawn a new entity with the effect.
        let effect_owner = world.spawn_empty().set_parent(target).id();
        let mut scope = TrackingScope::new(world.change_tick());

        let reaction = DynamicStyleReaction {
            target,
            style_fn: self.style_fn.clone(),
        };
        reaction.apply(effect_owner, world, &mut scope);
        world.entity_mut(effect_owner).insert((
            scope,
            ReactionCell::new(DynamicStyleReaction {
                target,
                style_fn: self.style_fn.clone(),
            }),
        ));
    }
}

pub struct DynamicStyleReaction<D, VF: Fn(&Rcx) -> D, SF: Fn(D, &mut StyleBuilder)> {
    pub(crate) target: Entity,
    pub(crate) style_fn: Arc<(VF, SF)>,
}

impl<
        D,
        VF: Fn(&Rcx) -> D + Send + Sync + 'static,
        SF: Fn(D, &mut StyleBuilder) + Send + Sync + 'static,
    > DynamicStyleReaction<D, VF, SF>
{
    fn apply(&self, _owner: Entity, world: &mut World, tracking: &mut TrackingScope) {
        let rcx = Rcx::new(world, self.target, tracking);
        let val = (self.style_fn.0)(&rcx);

        let mut target = world.entity_mut(self.target);
        let mut style = ui::Style::default();
        if let Some(s) = target.get::<ui::Style>() {
            style.clone_from(s);
        }
        let mut sb = StyleBuilder::new(&mut target, style);
        (self.style_fn.1)(val, &mut sb);
        sb.finish();
    }
}

impl<
        D,
        VF: Fn(&Rcx) -> D + Send + Sync + 'static,
        SF: Fn(D, &mut StyleBuilder) + Send + Sync + 'static,
    > Reaction for DynamicStyleReaction<D, VF, SF>
{
    fn react(&mut self, owner: Entity, world: &mut World, tracking: &mut TrackingScope) {
        self.apply(owner, world, tracking);
    }
}
