use bevy::ecs::world::World;
use bevy::prelude::*;
use bevy::ui::experimental::GhostNode;
use bevy_reactor_signals::{Rcx, Reaction, ReactionCell, TrackingScope};

use crate::test_condition::TestCondition;
use crate::{CreateChilden, UiBuilder};

/// The state of the conditional branch, which is initially "unset".
#[derive(PartialEq)]
pub enum CondState {
    Unset,
    True,
    False,
}

impl From<bool> for CondState {
    fn from(value: bool) -> Self {
        if value {
            Self::True
        } else {
            Self::False
        }
    }
}

pub trait CondBuilder {
    /// Build a reactive conditional node, which renders one of it's two children depending
    /// on a test condition.
    fn cond<
        Test: TestCondition + 'static,
        PosFn: Send + Sync + Fn(&mut UiBuilder) + 'static,
        NegFn: Send + Sync + Fn(&mut UiBuilder) + 'static,
    >(
        &mut self,
        test: Test,
        pos: PosFn,
        neg: NegFn,
    ) -> &mut Self;
}

impl<'w> CondBuilder for WorldChildBuilder<'w> {
    fn cond<
        Test: TestCondition + 'static,
        PosFn: Send + Sync + Fn(&mut UiBuilder) + 'static,
        NegFn: Send + Sync + Fn(&mut UiBuilder) + 'static,
    >(
        &mut self,
        test: Test,
        pos: PosFn,
        neg: NegFn,
    ) -> &mut Self {
        // Create an entity to represent the condition.
        let mut cond_owner = self.spawn(Name::new("Cond"));
        let cond_owner_id = cond_owner.id();

        // Create a tracking scope and reaction.
        let mut tracking = TrackingScope::new(cond_owner.world().last_change_tick());
        let mut reaction = CondReaction {
            test,
            pos,
            neg,
            state: CondState::Unset,
        };

        // Safety: this should be safe because we don't use cond_owner any more after this
        // point.
        let world = unsafe { cond_owner.world_mut() };
        // Trigger the initial reaction.
        reaction.react(cond_owner_id, world, &mut tracking);
        world.entity_mut(cond_owner_id).insert((
            GhostNode::default(),
            tracking,
            ReactionCell::new(reaction),
        ));
        self
    }
}

impl<'w> CondBuilder for UiBuilder<'w> {
    fn cond<
        Test: TestCondition + 'static,
        PosFn: Send + Sync + Fn(&mut UiBuilder) + 'static,
        NegFn: Send + Sync + Fn(&mut UiBuilder) + 'static,
    >(
        &mut self,
        test: Test,
        pos: PosFn,
        neg: NegFn,
    ) -> &mut Self {
        // Create an entity to represent the condition.
        let mut cond_owner = self.spawn(Name::new("Cond"));
        let cond_owner_id = cond_owner.id();

        // Create a tracking scope and reaction.
        let mut tracking = TrackingScope::new(cond_owner.world().last_change_tick());
        let mut reaction = CondReaction {
            test,
            pos,
            neg,
            state: CondState::Unset,
        };

        // Safety: this should be safe because we don't use cond_owner any more after this
        // point.
        let world = unsafe { cond_owner.world_mut() };
        // Trigger the initial reaction.
        reaction.react(cond_owner_id, world, &mut tracking);
        world.entity_mut(cond_owner_id).insert((
            GhostNode::default(),
            tracking,
            ReactionCell::new(reaction),
        ));
        self
    }
}

/// A reaction that handles the conditional rendering logic.
struct CondReaction<Test: TestCondition, PosFn: Fn(&mut UiBuilder), NegFn: Fn(&mut UiBuilder)>
where
    Self: Send + Sync,
{
    test: Test,
    pos: PosFn,
    neg: NegFn,
    state: CondState,
}

impl<
        Test: TestCondition,
        PosFn: Send + Sync + Fn(&mut UiBuilder),
        NegFn: Send + Sync + Fn(&mut UiBuilder),
    > CondReaction<Test, PosFn, NegFn>
{
    /// Helper function to build either the true or false branch content.
    fn build_branch_state<Factory: Fn(&mut UiBuilder)>(
        &self,
        branch: &Factory,
        owner: Entity,
        world: &mut World,
    ) {
        world.entity_mut(owner).despawn_descendants();
        world.entity_mut(owner).create_children_mut(branch);
    }
}

impl<
        Test: TestCondition,
        PosFn: Send + Sync + Fn(&mut UiBuilder),
        NegFn: Send + Sync + Fn(&mut UiBuilder),
    > Reaction for CondReaction<Test, PosFn, NegFn>
{
    fn react(&mut self, owner: Entity, world: &mut World, tracking: &mut TrackingScope) {
        // Create a reactive context and call the test condition.
        let re = Rcx::new(world, owner, tracking);
        let cond: CondState = self.test.test(&re).into();

        if cond != self.state {
            // Destroy the old branch state and build the new one.
            match cond {
                CondState::Unset => {
                    unreachable!("Condition should not be unset");
                }
                CondState::True => self.build_branch_state(&self.pos, owner, world),
                CondState::False => self.build_branch_state(&self.neg, owner, world),
            };
            self.state = cond;
        }
    }
}
