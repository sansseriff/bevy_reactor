use std::sync::{Arc, Mutex};

use bevy::ecs::world::World;
use bevy::prelude::*;

use crate::node_span::NodeSpan;
use crate::{
    DespawnScopes, DisplayNodeChanged, IntoView, Rcx, TrackingScope, View, ViewHandle, ViewRef,
};

pub enum CondState {
    Unset,
    True((ViewRef, Entity)),
    False((ViewRef, Entity)),
}

/// A conditional view which renders one of two children depending on the condition expression.
pub struct Cond<Test: 'static, Pos: IntoView, PosFn: Fn() -> Pos, Neg: IntoView, NegFn: Fn() -> Neg>
{
    test: Test,
    pos: PosFn,
    neg: NegFn,
    state: CondState,
}

impl<
        Test: Fn(&Rcx) -> bool,
        Pos: IntoView,
        PosFn: Fn() -> Pos,
        Neg: IntoView,
        NegFn: Fn() -> Neg,
    > Cond<Test, Pos, PosFn, Neg, NegFn>
{
    /// Construct a new conditional View.
    pub fn new(test: Test, pos: PosFn, neg: NegFn) -> Self {
        Self {
            test,
            pos,
            neg,
            state: CondState::Unset,
        }
    }

    fn build_branch_state<V: IntoView, Factory: Fn() -> V>(
        &self,
        branch: &Factory,
        parent: Entity,
        world: &mut World,
    ) -> (ViewRef, Entity) {
        let state_view = (branch)().into_view();
        let state_entity = ViewHandle::spawn(&state_view, parent, world);
        world.entity_mut(parent).insert(DisplayNodeChanged);
        (state_view, state_entity)
    }
}

impl<
        Test: Fn(&Rcx) -> bool,
        Pos: IntoView,
        PosFn: Fn() -> Pos,
        Neg: IntoView,
        NegFn: Fn() -> Neg,
    > View for Cond<Test, Pos, PosFn, Neg, NegFn>
{
    fn nodes(&self) -> NodeSpan {
        match self.state {
            CondState::Unset => NodeSpan::Empty,
            CondState::True(ref true_state) => true_state.0.lock().unwrap().nodes(),
            CondState::False(ref false_state) => false_state.0.lock().unwrap().nodes(),
        }
    }

    fn build(&mut self, view_entity: Entity, world: &mut World) {
        let mut tracking = TrackingScope::new(world.read_change_tick());
        self.react(view_entity, world, &mut tracking);
        world.entity_mut(view_entity).insert(tracking);
    }

    fn react(&mut self, view_entity: Entity, world: &mut World, tracking: &mut TrackingScope) {
        let re = Rcx::new(world, tracking);
        let cond = (self.test)(&re);
        if cond {
            match self.state {
                CondState::True(_) => {
                    // Already true, do nothing.
                }
                CondState::False((ref mut false_state, entity)) => {
                    false_state.lock().unwrap().raze(entity, world);
                    self.state = CondState::True(self.build_branch_state::<Pos, PosFn>(
                        &self.pos,
                        view_entity,
                        world,
                    ));
                }
                CondState::Unset => {
                    self.state = CondState::True(self.build_branch_state::<Pos, PosFn>(
                        &self.pos,
                        view_entity,
                        world,
                    ));
                }
            }
        } else {
            match self.state {
                CondState::False(_) => {
                    // Already false, do nothing.
                }
                CondState::True((ref mut true_state, entity)) => {
                    true_state.lock().unwrap().raze(entity, world);
                    self.state = CondState::False(self.build_branch_state::<Neg, NegFn>(
                        &self.neg,
                        view_entity,
                        world,
                    ));
                }
                CondState::Unset => {
                    self.state = CondState::False(self.build_branch_state::<Neg, NegFn>(
                        &self.neg,
                        view_entity,
                        world,
                    ));
                }
            }
        }
    }

    fn raze(&mut self, view_entity: Entity, world: &mut World) {
        match self.state {
            CondState::True((ref mut true_state, entity)) => {
                true_state.lock().unwrap().raze(entity, world)
            }
            CondState::False((ref mut false_state, entity)) => {
                false_state.lock().unwrap().raze(entity, world)
            }
            CondState::Unset => {}
        }
        world.despawn_owned_recursive(view_entity);
    }
}

/// Creates a conditional branch view.
pub fn cond<
    Test: Send + Sync + Fn(&Rcx) -> bool,
    Pos: 'static + IntoView,
    PosFn: Send + Sync + 'static + Fn() -> Pos,
    Neg: 'static + IntoView,
    NegFn: Send + Sync + 'static + Fn() -> Neg,
>(
    test: Test,
    pos: PosFn,
    neg: NegFn,
) -> Cond<Test, Pos, PosFn, Neg, NegFn> {
    Cond::new(test, pos, neg)
}

impl<
        Test: Send + Sync + Fn(&Rcx) -> bool,
        Pos: 'static + IntoView,
        PosFn: Send + Sync + 'static + Fn() -> Pos,
        Neg: 'static + IntoView,
        NegFn: Send + Sync + 'static + Fn() -> Neg,
    > IntoView for Cond<Test, Pos, PosFn, Neg, NegFn>
{
    fn into_view(self) -> ViewRef {
        Arc::new(Mutex::new(self))
    }
}
