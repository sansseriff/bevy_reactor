use bevy::{
    prelude::{BuildChildren, Entity, EntityWorldMut, World},
    ui::GhostNode,
};
use bevy_reactor_signals::{Rcx, Reaction, ReactionCell, TrackingScope};

pub trait EntityEffectBuilder {
    fn effect<
        D: 'static,
        VF: Fn(&Rcx) -> D + Send + Sync + 'static,
        SF: Fn(D, &mut EntityWorldMut) + Send + Sync + 'static,
    >(
        &mut self,
        deps_fn: VF,
        style: SF,
    ) -> &mut Self;
}

impl<'w> EntityEffectBuilder for EntityWorldMut<'w> {
    fn effect<
        D: 'static,
        VF: Fn(&Rcx) -> D + Send + Sync + 'static,
        SF: Fn(D, &mut EntityWorldMut) + Send + Sync + 'static,
    >(
        &mut self,
        deps_fn: VF,
        effect_fn: SF,
    ) -> &mut Self {
        let mut scope = TrackingScope::new(self.world().last_change_tick());
        let reaction = TargetedEffectReaction {
            target: self.id(),
            deps_fn,
            effect_fn,
        };
        let owner = self.id();
        self.world_scope(|world| {
            // Spawn a new reaction entity to contain the effect.
            let effect_owner = world.spawn_empty().set_parent(owner).id();
            reaction.apply(effect_owner, world, &mut scope);
            world
                .entity_mut(effect_owner)
                .insert((scope, ReactionCell::new(reaction), GhostNode));
        });
        self
    }
}

struct TargetedEffectReaction<D, VF: Fn(&Rcx) -> D, SF: Fn(D, &mut EntityWorldMut)> {
    target: Entity,
    deps_fn: VF,
    effect_fn: SF,
}

impl<
        D,
        VF: Fn(&Rcx) -> D + Send + Sync + 'static,
        SF: Fn(D, &mut EntityWorldMut) + Send + Sync + 'static,
    > TargetedEffectReaction<D, VF, SF>
{
    fn apply(&self, _owner: Entity, world: &mut World, tracking: &mut TrackingScope) {
        let rcx = Rcx::new(world, self.target, tracking);
        let val = (self.deps_fn)(&rcx);

        let mut target = world.entity_mut(self.target);
        (self.effect_fn)(val, &mut target);
    }
}

impl<
        D,
        VF: Fn(&Rcx) -> D + Send + Sync + 'static,
        SF: Fn(D, &mut EntityWorldMut) + Send + Sync + 'static,
    > Reaction for TargetedEffectReaction<D, VF, SF>
{
    fn react(&mut self, owner: Entity, world: &mut World, tracking: &mut TrackingScope) {
        self.apply(owner, world, tracking);
    }
}
