#![allow(clippy::type_complexity)]

use bevy::prelude::{BuildChildren, DespawnRecursiveExt, Entity};
use bevy::ui::experimental::GhostNode;
use bevy::{core::Name, ecs::world::World};
use bevy_reactor_signals::{Rcx, Reaction, ReactionCell, Signal, TrackingScope};

use crate::{CreateChilden, UiBuilder};

/// Trait that abstracts over the switch test value that controls the If. We use this trait
/// to allow boolean signals to be passed directly as conditions.
pub trait SwitchTestValue<Value>: Send + Sync {
    fn compute_test_value(&self, rcx: &Rcx) -> Value;
}

impl<Value, F: Send + Sync + Fn(&Rcx) -> Value> SwitchTestValue<Value> for F {
    fn compute_test_value(&self, rcx: &Rcx) -> Value {
        self(rcx)
    }
}

// impl<Value> SwitchCondition<Value> for Value {
//     fn test(&self, _rcx: &Rcx) -> Value {
//         *self
//     }
// }

impl<Value: Send + Sync + Copy + 'static> SwitchTestValue<Value> for Signal<Value> {
    fn compute_test_value(&self, rcx: &Rcx) -> Value {
        self.get(rcx)
    }
}

pub trait SwitchBuilder {
    fn switch<
        Value: Send + Sync + PartialEq + 'static,
        VF: SwitchTestValue<Value> + 'static,
        CF: Fn(&mut CaseBuilder<Value>),
    >(
        &mut self,
        value_fn: VF,
        cases_fn: CF,
    ) -> &mut Self;
}

impl<'w> SwitchBuilder for UiBuilder<'w> {
    fn switch<
        Value: Send + Sync + PartialEq + 'static,
        VF: SwitchTestValue<Value> + 'static,
        CF: Fn(&mut CaseBuilder<Value>),
    >(
        &mut self,
        value_fn: VF,
        cases_fn: CF,
    ) -> &mut Self {
        let mut cases: Vec<(Value, Box<dyn Fn(&mut UiBuilder) + Send + Sync>)> = Vec::new();
        let mut fallback: Option<Box<dyn Fn(&mut UiBuilder) + Send + Sync>> = None;

        let mut case_builder = CaseBuilder {
            cases: &mut cases,
            fallback: &mut fallback,
        };
        cases_fn(&mut case_builder);
        // TODO: Populate cases
        let mut reaction = SwitchReaction {
            cases,
            fallback,
            test_value: value_fn,
            switch_index: usize::MAX - 1, // Means no case selected yet.
        };

        // Create an entity to represent the condition.
        let parent = self.parent();
        let reaction_owner = self
            .world_mut()
            .spawn((Name::new("Switch"), GhostNode::default()))
            .set_parent(parent)
            .id();

        // Create a tracking scope and reaction.
        let mut tracking = TrackingScope::new(self.world().last_change_tick());

        // Trigger the initial reaction.
        reaction.react(reaction_owner, self.world_mut(), &mut tracking);
        self.world_mut()
            .entity_mut(reaction_owner)
            .insert((tracking, ReactionCell::new(reaction)));
        self
    }
}

pub struct CaseBuilder<'a, Value: Send + Sync> {
    cases: &'a mut Vec<(Value, Box<dyn Fn(&mut UiBuilder) + Send + Sync>)>,
    fallback: &'a mut Option<Box<dyn Fn(&mut UiBuilder) + Send + Sync>>,
}

impl<'a, Value: Send + Sync> CaseBuilder<'a, Value> {
    pub fn case<CF: Send + Sync + 'static + Fn(&mut UiBuilder)>(
        &mut self,
        value: Value,
        case_fn: CF,
    ) -> &mut Self {
        self.cases.push((value, Box::new(case_fn)));
        self
    }

    pub fn fallback<FF: Send + Sync + 'static + Fn(&mut UiBuilder)>(
        &mut self,
        fallback_fn: FF,
    ) -> &mut Self {
        *self.fallback = Some(Box::new(fallback_fn));
        self
    }
}

/// A reaction that handles the conditional rendering logic.
struct SwitchReaction<Value, F: SwitchTestValue<Value>>
where
    Self: Send + Sync,
{
    test_value: F,
    switch_index: usize,
    cases: Vec<(Value, Box<dyn Fn(&mut UiBuilder) + Send + Sync>)>,
    fallback: Option<Box<dyn Fn(&mut UiBuilder) + Send + Sync>>,
}

impl<Value: Send + Sync + PartialEq, F: SwitchTestValue<Value>> Reaction
    for SwitchReaction<Value, F>
{
    fn react(&mut self, owner: Entity, world: &mut World, tracking: &mut TrackingScope) {
        // Create a reactive context and call the test condition.
        let re = Rcx::new(world, owner, tracking);
        let value: Value = self.test_value.compute_test_value(&re);
        let index = self
            .cases
            .iter()
            .enumerate()
            .find_map(|(i, f)| if f.0 == value { Some(i) } else { None })
            .unwrap_or(usize::MAX);

        if index != self.switch_index {
            self.switch_index = index;
            world.entity_mut(owner).despawn_descendants();
            if index < self.cases.len() {
                world
                    .entity_mut(owner)
                    .create_children_mut(self.cases[index].1.as_mut());
            } else if let Some(ref fallback) = self.fallback {
                world.entity_mut(owner).create_children(fallback);
            };
        }
    }
}
