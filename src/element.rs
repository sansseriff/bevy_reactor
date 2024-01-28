use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use bevy::prelude::*;

use crate::{
    element_effect::{
        ComputedBundleReaction, ElementEffect, InsertBundleEffect, RunReactionEffect,
        UpdateReaction,
    },
    node_span::NodeSpan,
    view::View,
    view_tuple::ViewTuple,
    Cx, DespawnScopes, IntoView, Rcx, TrackingScope, ViewHandle, ViewRef,
};

struct ElementChild {
    view: ViewRef,
    entity: Option<Entity>,
}

/// A basic UI element
#[derive(Default)]
pub struct Element<B: Bundle + Default> {
    /// Debug name for this element.
    debug_name: String,

    /// The visible UI node for this element.
    display: Option<Entity>,

    /// Children of this element.
    children: Vec<ElementChild>,

    /// List of effects for components to be added to the element.
    effects: Vec<Box<dyn ElementEffect>>,

    marker: PhantomData<B>,
}

impl<B: Bundle + Default> Element<B> {
    /// Construct a new `Element`.
    pub fn new() -> Self {
        Self {
            debug_name: String::new(),
            display: None,
            children: Vec::new(),
            effects: Vec::new(),
            marker: PhantomData,
        }
    }

    /// Construct a new `Element` with a given entity id.
    pub fn for_entity(node: Entity) -> Self {
        Self {
            debug_name: String::new(),
            display: Some(node),
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

    /// Set the child views for this element.
    pub fn children<V: ViewTuple>(mut self, views: V) -> Self {
        if !self.children.is_empty() {
            panic!("Children already set");
        }
        let mut child_views: Vec<ViewRef> = Vec::new();
        views.get_handles(&mut child_views);
        self.children = child_views
            .iter()
            .map(|v| ElementChild {
                view: v.clone(),
                entity: None,
            })
            .collect();
        self
    }

    /// Add a static bundle to the element.
    pub fn insert<T: Bundle>(mut self, bundle: T) -> Self {
        self.effects.push(Box::new(InsertBundleEffect {
            bundle: Some(bundle),
        }));
        self
    }

    /// Add a computed bundle to the element.
    pub fn insert_computed<T: Bundle, F: Send + Sync + 'static + FnMut(&mut Rcx) -> T>(
        mut self,
        factory: F,
    ) -> Self {
        self.effects.push(Box::new(RunReactionEffect::new(
            ComputedBundleReaction::new(factory),
        )));
        self
    }

    // pub fn insert_update<
    //     T: Component,
    //     F1: Send + Sync + 'static + FnMut() -> T,
    //     F2: Send + Sync + 'static + FnMut(&mut Re, &mut T),
    // >(
    //     mut self,
    //     init: F1,
    //     update: F2,
    // ) -> Self {
    //     self.producers.push(Arc::new(Mutex::new(BundleComputedRef {
    //         target: None,
    //         init,
    //         update,
    //         tracker: None,
    //         marker: PhantomData,
    //     })));
    //     self
    // }

    /// Create a reactive effect which is attached to the element.
    pub fn create_effect<F: Send + Sync + 'static + FnMut(&mut Cx, Entity)>(
        mut self,
        effect: F,
    ) -> Self {
        self.effects
            .push(Box::new(RunReactionEffect::new(UpdateReaction::new(
                effect,
            ))));
        self
    }

    /// Attach the children to the node. Note that each child view may produce multiple nodes,
    /// or none.
    fn attach_children(&self, world: &mut World) {
        let mut count: usize = 0;
        for child in self.children.iter() {
            count += child.view.lock().unwrap().nodes().count();
        }

        let mut flat: Vec<Entity> = Vec::with_capacity(count);
        for child in self.children.iter() {
            child.view.lock().unwrap().nodes().flatten(&mut flat);
        }

        world
            .entity_mut(self.display.unwrap())
            .replace_children(&flat);
    }
}

impl<B: Bundle + Default> View for Element<B> {
    fn nodes(&self) -> NodeSpan {
        match self.display {
            None => NodeSpan::Empty,
            Some(node) => NodeSpan::Node(node),
        }
    }

    fn build(&mut self, view_entity: Entity, world: &mut World) {
        world.entity_mut(view_entity).insert(Name::new("Element"));
        // Build element node
        assert!(self.display.is_none());
        let display = world
            .spawn((B::default(), Name::new(self.debug_name.clone())))
            .id();

        // Insert components
        if !self.effects.is_empty() || !self.effects.is_empty() {
            let mut tracking = TrackingScope::new(world.change_tick());
            for producer in self.effects.iter_mut() {
                producer.start(&mut tracking, display, world);
            }
            world.entity_mut(view_entity).insert(tracking);
        }

        self.display = Some(display);

        // Build child nodes.
        for child in self.children.iter_mut() {
            child.entity = Some(ViewHandle::spawn(&child.view, view_entity, world));
        }

        self.attach_children(world);
    }

    fn raze(&mut self, view_entity: Entity, world: &mut World) {
        assert!(self.display.is_some());
        // Raze all child views
        for child in self.children.drain(..) {
            let inner = child.view.clone();
            inner.lock().unwrap().raze(child.entity.unwrap(), world);
            // Child raze() will despawn itself.
        }

        // Delete the display node.
        world.entity_mut(self.display.unwrap()).remove_parent();
        world.entity_mut(self.display.unwrap()).despawn();
        self.display = None;

        // Delete all reactions.
        world.despawn_owned_recursive(view_entity);
    }

    fn children_changed(&mut self, _view_entity: Entity, world: &mut World) -> bool {
        // info!("children_changed handled");
        self.attach_children(world);
        true
    }
}

impl<B: Bundle + Default> IntoView for Element<B> {
    fn into_view(self) -> ViewRef {
        Arc::new(Mutex::new(self))
    }
}
