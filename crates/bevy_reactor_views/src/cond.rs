use std::sync::{Arc, Mutex};

use bevy::ecs::world::World;
use bevy::prelude::*;
use bevy_reactor_signals::{Rcx, Reaction, ReactionCell, Signal, TrackingScope};

use crate::{IntoView, View};

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
        &mut self,
        owner: Entity,
        world: &mut World,
        _scope: &mut TrackingScope,
        out: &mut Vec<Entity>,
    ) {
        let cond_owner = world.spawn(Name::new("Cond")).set_parent(owner).id();
        let mut tracking = TrackingScope::new(world.change_tick());
        let mut lock = self.reaction.lock().unwrap();
        lock.react(cond_owner, world, &mut tracking);
        world
            .entity_mut(cond_owner)
            .insert((tracking, ReactionCell(self.reaction.clone())));
        out.push(cond_owner);
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
    fn into_view(self) -> Box<dyn View + Send + Sync + 'static> {
        Box::new(self)
    }
}

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
    fn build_branch_state<V: IntoView, Factory: Fn() -> V>(
        &self,
        branch: &Factory,
        parent: Entity,
        world: &mut World,
    ) -> (Box<dyn View + Send + Sync + 'static>, Entity) {
        let state_entity = world.spawn_empty().set_parent(parent).id();
        let mut state_view = (branch)().into_view();
        let mut scope = TrackingScope::new(world.change_tick());
        let mut children = Vec::new();
        state_view.build(state_entity, world, &mut scope, &mut children);
        world
            .entity_mut(state_entity)
            .insert(scope)
            .replace_children(&children);
        (state_view, state_entity)
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
        let re = Rcx::new(world, owner, tracking);
        let cond: CondState = self.test.test(&re).into();
        if cond == self.state {
            return;
        }
        world.entity_mut(owner).despawn_descendants();
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
