use std::sync::{Arc, Mutex};

use bevy::prelude::{BuildChildren, DespawnRecursiveExt, Entity};
use bevy::ui::GhostNode;
use bevy::{core::Name, ecs::world::World};
use bevy_reactor_signals::{Rcx, Reaction, ReactionCell, Signal, TrackingScope};

use crate::{IntoView, View};

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

trait AnyCaseValue: Send + Sync {
    fn build(&self) -> Arc<dyn View>;
}

impl<V: IntoView, F: Send + Sync + Fn() -> V> AnyCaseValue for F {
    fn build(&self) -> Arc<dyn View> {
        self().into_view()
    }
}

/// A conditional view which selects one case and renders it.
pub struct Switch<Value: Send + Sync, F: SwitchTestValue<Value>> {
    reaction: Arc<Mutex<SwitchReaction<Value, F>>>,
}

impl<Value: Send + Sync, F: SwitchTestValue<Value>> Switch<Value, F> {
    /// Construct a new Switch View.
    pub fn new(value: F) -> Self {
        Self {
            reaction: Arc::new(Mutex::new(SwitchReaction {
                test_value: value,
                switch_index: usize::MAX - 1,
                cases: Vec::new(),
                fallback: None,
            })),
        }
    }
}

impl<Value: Send + Sync + PartialEq, F: SwitchTestValue<Value>> Switch<Value, F> {
    pub fn case<V: IntoView, CF: Fn() -> V + Send + Sync + 'static>(
        self,
        value: Value,
        factory: CF,
    ) -> Self {
        self.reaction
            .lock()
            .unwrap()
            .cases
            .push((value, Box::new(factory)));
        self
    }

    pub fn fallback<V: IntoView, CF: Fn() -> V + Send + Sync + 'static>(
        self,
        fallback: CF,
    ) -> Self {
        self.reaction.lock().unwrap().fallback = Some(Box::new(fallback));
        self
    }
}

impl<Value: Send + Sync + PartialEq + Clone + 'static, F: SwitchTestValue<Value> + 'static> View
    for Switch<Value, F>
{
    fn build(
        &self,
        owner: Entity,
        world: &mut World,
        _scope: &mut bevy_reactor_signals::TrackingScope,
        out: &mut Vec<Entity>,
    ) {
        // Create an entity to represent the condition.
        let reaction_owner = world.spawn(Name::new("Switch")).set_parent(owner).id();
        out.push(reaction_owner);

        // Create a tracking scope and reaction.
        let mut tracking = TrackingScope::new(world.change_tick());
        let mut lock = self.reaction.lock().unwrap();

        // Trigger the initial reaction.
        lock.react(reaction_owner, world, &mut tracking);
        world.entity_mut(reaction_owner).insert((
            GhostNode,
            tracking,
            ReactionCell(self.reaction.clone()),
        ));
    }
}

impl<Value: Send + Sync + PartialEq + Clone + 'static, F: SwitchTestValue<Value> + 'static> IntoView
    for Switch<Value, F>
{
    fn into_view(self) -> Arc<dyn View + 'static> {
        Arc::new(self)
    }
}

/// A reaction that handles the conditional rendering logic.
struct SwitchReaction<Value, F: SwitchTestValue<Value>>
where
    Self: Send + Sync,
{
    test_value: F,
    switch_index: usize,
    cases: Vec<(Value, Box<dyn AnyCaseValue>)>,
    fallback: Option<Box<dyn AnyCaseValue>>,
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
            let case_view = if index < self.cases.len() {
                self.cases[index].1.build()
            } else if let Some(ref fallback) = self.fallback {
                fallback.build()
            } else {
                return;
            };

            let mut children = Vec::new();
            case_view.build(owner, world, tracking, &mut children);
            world.entity_mut(owner).replace_children(&children);
        }
    }
}
