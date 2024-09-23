use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use bevy::{
    core::Name,
    ecs::system::IntoObserverSystem,
    prelude::{BuildChildren, Bundle, Component, Entity, Event, Observer, World},
};
use bevy_mod_stylebuilder::{StyleBuilder, StyleTuple};
use bevy_reactor_signals::{IntoSignal, Rcx, Reaction, ReactionCell, Signal, TrackingScope};

use crate::{
    effect::Effect,
    style::{DynamicStyleEffect, StaticStyleEffect},
    view::IntoViewVec,
    IntoView, View,
};

#[derive(Default)]
pub struct Element<B: Bundle> {
    /// Debug name for this element.
    debug_name: String,

    /// The visible UI node for this element.
    display: Option<Entity>,

    /// Children of this element.
    children: Vec<Arc<dyn View + 'static>>,

    /// List of effects to be added to the element.
    effects: Mutex<Vec<Box<dyn Effect>>>,

    /// List of observers to be added to the element.
    observers: Mutex<Vec<Observer>>,

    /// Marker for bundle type.
    marker: PhantomData<B>,
}

impl<B: Bundle + Default> Element<B> {
    /// Construct a new `Element`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct a new `Element` with a given entity id.
    pub fn for_entity(entity: Entity) -> Self {
        Self {
            debug_name: String::new(),
            display: Some(entity),
            children: Vec::new(),
            effects: Mutex::new(Vec::new()),
            observers: Mutex::new(Vec::new()),
            marker: PhantomData,
        }
    }

    /// Set the debug name for this element.
    pub fn named(mut self, name: &str) -> Self {
        self.debug_name = name.to_string();
        self
    }

    /// Add a static bundle to the element.
    pub fn insert<T: Bundle>(self, bundle: T) -> Self {
        self.effects
            .lock()
            .unwrap()
            .push(Box::new(InsertBundleEffect { bundle }));
        self
    }

    /// Add a static bundle to the element, if a condition is true.
    pub fn insert_if<C: Component, F: Fn() -> C + Send + Sync + 'static>(
        self,
        condition: impl IntoSignal<bool>,
        factory: F,
    ) -> Self {
        self.effects
            .lock()
            .unwrap()
            .push(Box::new(ConditionalInsertComponentEffect {
                factory,
                condition: condition.into_signal(),
            }));
        self
    }

    /// Set the static styles for this element.
    pub fn style<S: StyleTuple + 'static>(self, styles: S) -> Self {
        self.effects
            .lock()
            .unwrap()
            .push(Box::new(StaticStyleEffect { styles }));
        self
    }

    /// Set a dynamic style for this element.
    ///
    /// Arguments:
    /// - `deps_fn`: A reactive function which accesses the reactive data sources and returns
    ///     the values used as inputs for the dynamic style computation.
    /// - `style_fn`: A non-reactive function which takes the computed style data and applies it to
    ///     the element.
    pub fn style_dyn<
        D: 'static,
        VF: Fn(&Rcx) -> D + Send + Sync + 'static,
        SF: Fn(D, &mut StyleBuilder) + Send + Sync + 'static,
    >(
        self,
        deps_fn: VF,
        style_fn: SF,
    ) -> Self {
        self.effects
            .lock()
            .unwrap()
            .push(Box::new(DynamicStyleEffect {
                style_fn: Arc::new((deps_fn, style_fn)),
            }));
        self
    }

    /// Set the child views for this element.
    pub fn children<V: IntoViewVec + 'static>(mut self, child_views: V) -> Self {
        child_views.into_view_vec(&mut self.children);
        self
    }

    /// Creates an [`Observer`] listening for events of type `E` targeting this entity.
    /// In order to trigger the callback the entity must also match the query when the event is fired.
    pub fn observe<E: Event, B2: Bundle, M: Send + Sync + 'static>(
        self,
        observer: impl IntoObserverSystem<E, B2, M> + Sync,
    ) -> Self {
        self.observers.lock().unwrap().push(Observer::new(observer));
        self
    }
}

impl<B: Bundle + Default> View for Element<B> {
    fn build(
        &self,
        _owner: Entity,
        world: &mut bevy::prelude::World,
        scope: &mut TrackingScope,
        out: &mut Vec<Entity>,
    ) {
        // Build display entity if it doesn't already exist.
        let display = match self.display {
            Some(display) => {
                world
                    .entity_mut(display)
                    .insert((B::default(), Name::new(self.debug_name.clone())));
                display
            }
            None => {
                let entity = world
                    .spawn((B::default(), Name::new(self.debug_name.clone())))
                    .id();
                scope.add_owned(entity);
                // self.display = Some(entity);
                entity
            }
        };

        // Insert components from effects.
        let mut effects = self.effects.lock().unwrap();
        for effect in effects.drain(..) {
            effect.start(display, world);
        }

        // Build child nodes.
        let mut children: Vec<Entity> = Vec::new();
        for child in self.children.iter() {
            child.build(display, world, scope, &mut children);
        }
        let mut entt = world.entity_mut(display);
        entt.replace_children(&children);

        // Add observers
        for observer in self.observers.lock().unwrap().drain(..) {
            world.spawn(observer.with_entity(display));
        }

        out.push(display);
    }
}

impl<B: Bundle + Default> IntoView for Element<B> {
    fn into_view(self) -> Arc<dyn View + 'static> {
        Arc::new(self)
    }
}

/// Inserts a static, pre-constructed bundle into the target entity. No reactivity.
pub struct InsertBundleEffect<B: Bundle> {
    pub(crate) bundle: B,
}

impl<B: Bundle> Effect for InsertBundleEffect<B> {
    // For a static bundle, we can just insert it once.
    fn start(self: Box<Self>, owner: Entity, world: &mut World) {
        world.entity_mut(owner).insert(self.bundle);
    }
}

/// Conditionally inserts a bundle into the target. If the condition is true, then the bundle
/// will be inserted. If the condition later becomes false, the component will be removed.
pub struct ConditionalInsertComponentEffect<C: Component, F: Fn() -> C> {
    pub factory: F,
    pub condition: Signal<bool>,
}

impl<C: Component, F: Fn() -> C + Send + Sync + 'static> Effect
    for ConditionalInsertComponentEffect<C, F>
{
    fn start(self: Box<Self>, owner: Entity, world: &mut World) {
        // Spawn a new entity with the effect.
        let effect_owner = world.spawn_empty().set_parent(owner).id();
        let mut scope = TrackingScope::new(world.change_tick());

        let mut reaction = ConditionalInsertComponentReaction {
            target: owner,
            factory: self.factory,
            condition: self.condition,
            prev_state: false,
        };
        reaction.react(effect_owner, world, &mut scope);
        world
            .entity_mut(effect_owner)
            .insert((scope, ReactionCell::new(reaction)));
    }

    // type State = bool;
    // fn apply(&self, cx: &mut Cx, target: Entity) -> Self::State {
    //     if self.condition {
    //         let mut target = cx.world_mut().entity_mut(target);
    //         target.insert((self.factory)());
    //     }
    //     self.condition
    // }

    // fn reapply(&self, cx: &mut Cx, target: Entity, state: &mut Self::State) {
    //     if self.condition != *state {
    //         *state = self.condition;
    //         if self.condition {
    //             self.apply(cx, target);
    //         } else {
    //             let mut target = cx.world_mut().entity_mut(target);
    //             target.remove::<C>();
    //         }
    //     }
    // }
}

pub struct ConditionalInsertComponentReaction<C: Component, F: Fn() -> C + Send + Sync> {
    target: Entity,
    factory: F,
    condition: Signal<bool>,
    prev_state: bool,
}

impl<C: Component, F: Fn() -> C + Send + Sync> Reaction
    for ConditionalInsertComponentReaction<C, F>
{
    fn react(&mut self, owner: Entity, world: &mut World, tracking: &mut TrackingScope) {
        let rcx = Rcx::new(world, self.target, tracking);
        let condition = self.condition.get(&rcx);
        if condition != self.prev_state {
            self.prev_state = condition;
            let mut target = world.entity_mut(owner);
            if condition {
                target.insert((self.factory)());
            } else {
                target.remove::<C>();
            }
        }
    }
}
