use std::sync::Arc;

use bevy::ecs::world::World;
use bevy::prelude::*;

use crate::node_span::NodeSpan;
use crate::{DespawnScopes, DisplayNodeChanged, Rcx, TrackingScope, View, ViewRef};

#[doc(hidden)]
pub trait CasePredicate: Send + Sync {
    fn test(&self, re: &Rcx) -> bool;
}

impl<F: Send + Sync + Fn(&Rcx) -> bool> CasePredicate for F {
    fn test(&self, re: &Rcx) -> bool {
        (self)(re)
    }
}

impl CasePredicate for bool {
    fn test(&self, _re: &Rcx) -> bool {
        *self
    }
}

#[doc(hidden)]
pub trait CaseBody: Send + Sync {
    fn build(&self, parent: Entity, world: &mut World) -> (ViewRef, Entity);
}

impl<V: Into<ViewRef>, FV: Send + Sync + Fn() -> V> CaseBody for FV {
    fn build(&self, parent: Entity, world: &mut World) -> (ViewRef, Entity) {
        let view = (self)().into();
        let entity = ViewRef::spawn(&view, parent, world);
        world.entity_mut(parent).insert(DisplayNodeChanged);
        (view, entity)
    }
}

/// A conditional case entry within a [`Switch`] element.
#[derive(Clone)]
pub struct Case {
    test: Arc<dyn CasePredicate>,
    view: Arc<dyn CaseBody>,
}

impl Case {
    /// Construct a new conditional case.
    pub fn new<
        F: Send + Sync + 'static + Fn(&Rcx) -> bool,
        V: Into<ViewRef>,
        FV: Send + Sync + 'static + Fn() -> V,
    >(
        test: F,
        view: FV,
    ) -> Self {
        Self {
            test: Arc::new(test),
            view: Arc::new(view),
        }
    }

    /// Construct a default case, one which is always true.
    pub fn default<V: Into<ViewRef>, FV: Send + Sync + 'static + Fn() -> V>(view: FV) -> Self {
        Self {
            test: Arc::new(true),
            view: Arc::new(view),
        }
    }

    fn test(&self, re: &Rcx) -> bool {
        self.test.test(re)
    }

    fn build(&self, parent: Entity, world: &mut World) -> (ViewRef, Entity) {
        self.view.build(parent, world)
    }
}

/// A series of conditional expressions and corresponding views, only one of which can render.
pub struct Switch {
    cases: Vec<Case>,
    state_index: usize,
    state: Option<(ViewRef, Entity)>,
}

impl Switch {
    /// Construct a new conditional View.
    pub fn new(cases: &[Case]) -> Self {
        Self {
            cases: cases.to_vec(),
            state_index: usize::MAX,
            state: None,
        }
    }
}

impl View for Switch {
    fn nodes(&self) -> NodeSpan {
        match self.state {
            Some((ref state, _entity)) => state.nodes(),
            None => NodeSpan::Empty,
        }
    }

    fn build(&mut self, view_entity: Entity, world: &mut World) {
        let mut tracking = TrackingScope::new(world.change_tick());
        self.react(view_entity, world, &mut tracking);
        world.entity_mut(view_entity).insert(tracking);
        assert!(
            world.entity_mut(view_entity).get::<Parent>().is_some(),
            "Switch should have a parent view"
        );
    }

    fn react(&mut self, view_entity: Entity, world: &mut World, tracking: &mut TrackingScope) {
        let re = Rcx::new(world, tracking);
        if let Some((index, case)) = self.cases.iter().enumerate().find(|(_, c)| c.test(&re)) {
            if index != self.state_index {
                if let Some((ref mut state, entity)) = self.state {
                    state.raze(entity, world);
                }
                self.state_index = index;
                self.state = Some(case.build(view_entity, world));
            }
        } else {
            if let Some((ref mut state, entity)) = self.state {
                state.raze(entity, world);
            }
            self.state_index = usize::MAX;
            self.state = None;
        }
    }

    fn raze(&mut self, view_entity: Entity, world: &mut World) {
        if let Some((ref mut state, entity)) = self.state {
            state.raze(entity, world);
        }
        world.despawn_owned_recursive(view_entity);
    }
}

impl From<Switch> for ViewRef {
    fn from(value: Switch) -> Self {
        ViewRef::new(value)
    }
}
