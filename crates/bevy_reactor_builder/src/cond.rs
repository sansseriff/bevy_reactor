use bevy::prelude::*;
use bevy::{ecs::world::World, ui::GhostNode};
use bevy_reactor_signals::{Rcx, Reaction, ReactionCell, Signal, TrackingScope};

/// Trait that abstracts over the boolean condition that controls the If. We use this trait
/// to allow boolean signals to be passed directly as conditions.
pub trait TestCondition: Send + Sync {
    fn test(&self, rcx: &Rcx) -> bool;
}

impl<F: Send + Sync + Fn(&Rcx) -> bool> TestCondition for F {
    fn test(&self, rcx: &Rcx) -> bool {
        self(rcx)
    }
}

impl TestCondition for bool {
    fn test(&self, _rcx: &Rcx) -> bool {
        *self
    }
}

impl TestCondition for Signal<bool> {
    fn test(&self, rcx: &Rcx) -> bool {
        self.get(rcx)
    }
}

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
        PosFn: Send + Sync + Fn(&mut WorldChildBuilder) + 'static,
        NegFn: Send + Sync + Fn(&mut WorldChildBuilder) + 'static,
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
        PosFn: Send + Sync + Fn(&mut WorldChildBuilder) + 'static,
        NegFn: Send + Sync + Fn(&mut WorldChildBuilder) + 'static,
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

        // Safety: this should be save because we don't use cond_owner any more after this
        // point.
        let world = unsafe { cond_owner.world_mut() };
        // Trigger the initial reaction.
        reaction.react(cond_owner_id, world, &mut tracking);
        world
            .entity_mut(cond_owner_id)
            .insert((GhostNode, tracking, ReactionCell::new(reaction)));
        self
    }
}

/// A reaction that handles the conditional rendering logic.
struct CondReaction<
    Test: TestCondition,
    PosFn: Fn(&mut WorldChildBuilder),
    NegFn: Fn(&mut WorldChildBuilder),
> where
    Self: Send + Sync,
{
    test: Test,
    pos: PosFn,
    neg: NegFn,
    state: CondState,
}

impl<
        Test: TestCondition,
        PosFn: Send + Sync + Fn(&mut WorldChildBuilder),
        NegFn: Send + Sync + Fn(&mut WorldChildBuilder),
    > CondReaction<Test, PosFn, NegFn>
{
    /// Helper function to build either the true or false branch content.
    fn build_branch_state<Factory: Fn(&mut WorldChildBuilder)>(
        &self,
        branch: &Factory,
        owner: Entity,
        world: &mut World,
    ) {
        world.entity_mut(owner).despawn_descendants();
        world.entity_mut(owner).with_children(branch);
    }
}

impl<
        Test: TestCondition,
        PosFn: Send + Sync + Fn(&mut WorldChildBuilder),
        NegFn: Send + Sync + Fn(&mut WorldChildBuilder),
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
