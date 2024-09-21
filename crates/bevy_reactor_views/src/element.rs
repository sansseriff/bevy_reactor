use std::{marker::PhantomData, sync::Arc};

use bevy::{
    core::Name,
    prelude::{BuildChildren, Bundle, Entity},
};
use bevy_mod_stylebuilder::{StyleBuilder, StyleTuple};
use bevy_reactor_signals::{Rcx, TrackingScope};

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
    children: Vec<Box<dyn View + Send + Sync + 'static>>,

    /// List of effects to be added to the element.
    effects: Vec<Box<dyn Effect>>,

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
            effects: Vec::new(),
            marker: PhantomData,
        }
    }

    /// Set the debug name for this element.
    pub fn named(mut self, name: &str) -> Self {
        self.debug_name = name.to_string();
        self
    }

    /// Set the static styles for this element.
    pub fn style<S: StyleTuple + 'static>(mut self, styles: S) -> Self {
        self.effects.push(Box::new(StaticStyleEffect { styles }));
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
        mut self,
        deps_fn: VF,
        style_fn: SF,
    ) -> Self {
        self.effects.push(Box::new(DynamicStyleEffect {
            style_fn: Arc::new((deps_fn, style_fn)),
        }));
        self
    }

    /// Set the child views for this element.
    pub fn children<V: IntoViewVec + 'static>(mut self, child_views: V) -> Self {
        child_views.into_view_vec(&mut self.children);
        self
    }
}

impl<B: Bundle + Default> View for Element<B> {
    fn build(
        &mut self,
        owner: Entity,
        world: &mut bevy::prelude::World,
        scope: &mut TrackingScope,
        out: &mut Vec<Entity>,
    ) {
        // assert!(self.display.is_none());
        if self.debug_name.is_empty() {
            world.entity_mut(owner).insert(Name::new("Element"));
        } else {
            world
                .entity_mut(owner)
                .insert(Name::new(format!("Element::{}", self.debug_name)));
        }

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
                self.display = Some(entity);
                entity
            }
        };

        // Insert components from effects.
        if !self.effects.is_empty() {
            for effect in self.effects.iter_mut() {
                effect.start(owner, display, world);
            }
        }

        // Build child nodes.
        let mut children: Vec<Entity> = Vec::new();
        for child in self.children.iter_mut() {
            child.build(owner, world, scope, &mut children);
        }

        world
            .entity_mut(self.display.unwrap())
            .replace_children(&children);
        out.push(display);
    }
}

impl<B: Bundle + Default> IntoView for Element<B> {
    fn into_view(self) -> Box<dyn View + Send + Sync + 'static> {
        Box::new(self)
    }
}
