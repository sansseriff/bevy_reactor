use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy::{ecs::world::World, ui::GhostNode};
use bevy_reactor_signals::{Rcx, Reaction, ReactionCell, Signal, TrackingScope};

use crate::{IntoView, View};

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

/// A conditional view which renders one of two children depending on the condition expression.
#[allow(clippy::type_complexity)]
pub struct Cond<
    Test: TestCondition + 'static,
    Pos: IntoView,
    PosFn: Fn() -> Pos + Send + Sync,
    Neg: IntoView,
    NegFn: Fn() -> Neg + Send + Sync,
> {
    reaction: Arc<Mutex<CondReaction<Test, Pos, PosFn, Neg, NegFn>>>,
}

impl<
        Test: TestCondition,
        Pos: IntoView,
        PosFn: Fn() -> Pos + Send + Sync,
        Neg: IntoView,
        NegFn: Fn() -> Neg + Send + Sync,
    > Cond<Test, Pos, PosFn, Neg, NegFn>
{
    /// Construct a new conditional View.
    pub fn new(test: Test, pos: PosFn, neg: NegFn) -> Self {
        Self {
            reaction: Arc::new(Mutex::new(CondReaction {
                test,
                pos,
                neg,
                state: CondState::Unset,
            })),
        }
    }
}

impl<
        Test: TestCondition,
        Pos: IntoView + 'static,
        PosFn: Fn() -> Pos + Send + Sync + 'static,
        Neg: IntoView + 'static,
        NegFn: Fn() -> Neg + Send + Sync + 'static,
    > View for Cond<Test, Pos, PosFn, Neg, NegFn>
{
    fn build(
        &self,
        owner: Entity,
        world: &mut World,
        _scope: &mut TrackingScope,
        out: &mut Vec<Entity>,
    ) {
        // Create an entity to represent the condition.
        let cond_owner = world.spawn(Name::new("Cond")).set_parent(owner).id();
        out.push(cond_owner);

        // Create a tracking scope and reaction.
        let mut tracking = TrackingScope::new(world.change_tick());
        let mut lock = self.reaction.lock().unwrap();

        // Trigger the initial reaction.
        lock.react(cond_owner, world, &mut tracking);
        world.entity_mut(cond_owner).insert((
            GhostNode,
            tracking,
            ReactionCell(self.reaction.clone()),
        ));
    }
}

impl<
        Test: TestCondition,
        Pos: IntoView + 'static,
        PosFn: Fn() -> Pos + Send + Sync + 'static,
        Neg: IntoView + 'static,
        NegFn: Fn() -> Neg + Send + Sync + 'static,
    > IntoView for Cond<Test, Pos, PosFn, Neg, NegFn>
{
    fn into_view(self) -> Arc<dyn View + 'static> {
        Arc::new(self)
    }
}

/// A reaction that handles the conditional rendering logic.
struct CondReaction<
    Test: TestCondition,
    Pos: IntoView,
    PosFn: Fn() -> Pos,
    Neg: IntoView,
    NegFn: Fn() -> Neg,
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
        Pos: IntoView + 'static,
        PosFn: Fn() -> Pos + Send + Sync + 'static,
        Neg: IntoView + 'static,
        NegFn: Fn() -> Neg + Send + Sync + 'static,
    > CondReaction<Test, Pos, PosFn, Neg, NegFn>
{
    /// Helper function to build either the true or false branch content.
    fn build_branch_state<V: IntoView, Factory: Fn() -> V>(
        &self,
        branch: &Factory,
        owner: Entity,
        scope: &mut TrackingScope,
        world: &mut World,
    ) {
        world.entity_mut(owner).despawn_descendants();
        let state_view = (branch)().into_view();
        let mut children = Vec::new();
        state_view.build(owner, world, scope, &mut children);
        world.entity_mut(owner).replace_children(&children);
    }
}

impl<
        Test: TestCondition,
        Pos: IntoView + 'static,
        PosFn: Fn() -> Pos + Send + Sync + 'static,
        Neg: IntoView + 'static,
        NegFn: Fn() -> Neg + Send + Sync + 'static,
    > Reaction for CondReaction<Test, Pos, PosFn, Neg, NegFn>
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
                CondState::True => self.build_branch_state(&self.pos, owner, tracking, world),
                CondState::False => self.build_branch_state(&self.neg, owner, tracking, world),
            };
            self.state = cond;
        }
    }
}
