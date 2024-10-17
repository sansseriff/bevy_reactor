use bevy::{prelude::*, ui::experimental::GhostNode};
use bevy_reactor_signals::{Rcx, Reaction, ReactionCell, TrackingScope};

use crate::test_condition::TestCondition;

pub trait InsertComponentBuilder {
    /// Add a static bundle to the element, when the condition is true.
    /// Removes the component when the condition is false.
    fn insert_if<C: Component, T: TestCondition + 'static, F: Fn() -> C + Send + Sync + 'static>(
        &mut self,
        condition: T,
        factory: F,
    ) -> &mut Self;
}

impl<'w> InsertComponentBuilder for EntityWorldMut<'w> {
    fn insert_if<C: Component, T: TestCondition + 'static, F: Fn() -> C + Send + Sync + 'static>(
        &mut self,
        condition: T,
        factory: F,
    ) -> &mut Self {
        let mut scope = TrackingScope::new(self.world().last_change_tick());
        let mut reaction = ConditionalInsertComponentReaction {
            target: self.id(),
            condition,
            factory,
            prev_state: false,
        };
        let owner = self.id();
        self.world_scope(|world| {
            // Spawn a new reaction entity to contain the effect.
            let effect_owner = world.spawn_empty().set_parent(owner).id();
            reaction.react(effect_owner, world, &mut scope);
            world.entity_mut(effect_owner).insert((
                scope,
                ReactionCell::new(reaction),
                GhostNode::default(),
            ));
        });
        self
    }
}

pub struct ConditionalInsertComponentReaction<
    C: Component,
    T: TestCondition,
    F: Fn() -> C + Send + Sync,
> {
    target: Entity,
    factory: F,
    condition: T,
    prev_state: bool,
}

impl<C: Component, T: TestCondition, F: Fn() -> C + Send + Sync> Reaction
    for ConditionalInsertComponentReaction<C, T, F>
{
    fn react(&mut self, _owner: Entity, world: &mut World, tracking: &mut TrackingScope) {
        let rcx = Rcx::new(world, self.target, tracking);
        let condition = self.condition.test(&rcx);
        if condition != self.prev_state {
            self.prev_state = condition;
            let mut target = world.entity_mut(self.target);
            if condition {
                target.insert((self.factory)());
            } else {
                target.remove::<C>();
            }
        }
    }
}
