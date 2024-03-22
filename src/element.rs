use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{
    element_effect::{ElementEffect, ElementEffectTarget},
    node_span::NodeSpan,
    view::View,
    view_children::ViewChildren,
    DespawnScopes, TrackingScope, ViewHandle,
};

struct ElementChild {
    view: ViewHandle,
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

    /// List of effects to be added to the element.
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
    pub fn children<V: ViewChildren>(mut self, views: V) -> Self {
        if !self.children.is_empty() {
            panic!("Children already set");
        }
        let child_views = views.to_vec();
        self.children = child_views
            .iter()
            .map(|v| ElementChild {
                view: v.clone(),
                entity: None,
            })
            .collect();
        self
    }

    /// Set the child views for this element.
    pub fn with_child(mut self, view: &ViewHandle) -> Self {
        if !self.children.is_empty() {
            panic!("Children already set");
        }
        self.children = vec![ElementChild {
            view: view.clone(),
            entity: None,
        }];
        self
    }

    /// Add a child views to this element.
    pub fn append_child(mut self, view: &ViewHandle) -> Self {
        self.children.push(ElementChild {
            view: view.clone(),
            entity: None,
        });
        self
    }

    // pub fn insert_computed_ref<
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

    /// Attach the children to the node. Note that each child view may produce multiple nodes,
    /// or none.
    fn attach_children(&self, world: &mut World) {
        let mut count: usize = 0;
        for child in self.children.iter() {
            count += child.view.nodes().count();
        }

        let mut flat: Vec<Entity> = Vec::with_capacity(count);
        for child in self.children.iter() {
            child.view.nodes().flatten(&mut flat);
        }

        world
            .entity_mut(self.display.unwrap())
            .replace_children(&flat);
    }
}

impl<B: Bundle + Default> ElementEffectTarget for Element<B> {
    fn add_effect(&mut self, effect: Box<dyn ElementEffect>) {
        self.effects.push(effect);
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
                self.display = Some(entity);
                entity
            }
        };

        // Insert components from effects.
        if !self.effects.is_empty() {
            let mut tracking = TrackingScope::new(world.read_change_tick());
            for producer in self.effects.iter_mut() {
                producer.start(&mut tracking, display, world);
            }
            world.entity_mut(view_entity).insert(tracking);
        }

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
            let inner = child.view;
            inner.raze(child.entity.unwrap(), world);
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

impl<B: Bundle + Default> From<Element<B>> for ViewHandle {
    fn from(value: Element<B>) -> Self {
        ViewHandle::new(value)
    }
}
