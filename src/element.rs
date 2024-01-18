use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use bevy::{core::Name, prelude::*};

use crate::{
    bundle::{BundleComputed, BundleProducer, BundleStatic},
    node_span::NodeSpan,
    view::{View, ViewContext},
    view_tuple::ViewTuple,
    IntoView, Re, ViewHandle, ViewRef,
};

/// A basic UI element
#[derive(Default)]
pub struct Element<B: Bundle + Default> {
    /// Debug name for this element.
    debug_name: String,

    /// The visible UI node for this element.
    display: Option<Entity>,

    /// Children of this element.
    children: Vec<ViewRef>,

    /// Children after they have been spawned as entities.
    child_entities: Vec<Entity>,

    /// List of producers for components to be added to the element.
    producers: Vec<Box<dyn BundleProducer>>,

    marker: PhantomData<B>,
}

impl<B: Bundle + Default> Element<B> {
    /// Construct a new `Element`.
    pub fn new() -> Self {
        Self {
            debug_name: String::new(),
            display: None,
            children: Vec::new(),
            child_entities: Vec::new(),
            producers: Vec::new(),
            marker: PhantomData,
        }
    }

    /// Construct a new `Element` with a given entity id.
    pub fn for_entity(node: Entity) -> Self {
        Self {
            debug_name: String::new(),
            display: Some(node),
            children: Vec::new(),
            child_entities: Vec::new(),
            producers: Vec::new(),
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
        views.get_handles(&mut self.children);
        self
    }

    /// Add a static bundle to the element.
    pub fn insert<T: Bundle>(mut self, bundle: T) -> Self {
        self.producers.push(Box::new(BundleStatic {
            bundle: Some(bundle),
        }));
        self
    }

    /// Add a computed bundle to the element.
    pub fn insert_computed<T: Bundle, F: Send + Sync + 'static + FnMut(&mut Re) -> T>(
        mut self,
        factory: F,
    ) -> Self {
        self.producers.push(Box::new(BundleComputed::new(factory)));
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

    /// Attach the children to the node. Note that each child view may produce multiple nodes,
    /// or none.
    fn attach_children(&mut self, world: &mut World) {
        let mut count: usize = 0;
        for child_ent in self.child_entities.iter_mut() {
            let child = world.entity_mut(*child_ent);
            let handle = child.get::<ViewHandle>().unwrap();
            count += handle.nodes().count();
        }

        let mut flat: Vec<Entity> = Vec::with_capacity(count);
        for child_ent in self.child_entities.iter() {
            let child = world.entity_mut(*child_ent);
            let handle = child.get::<ViewHandle>().unwrap();
            handle.nodes().flatten(&mut flat);
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

    fn build(&mut self, view_entity: Entity, vc: &mut ViewContext) {
        // Build element node
        assert!(self.display.is_none());
        let display = vc
            .world
            .spawn((B::default(), Name::new(self.debug_name.clone())))
            .id();

        // Insert components
        for producer in self.producers.iter_mut() {
            producer.start(view_entity, display, vc.world);
        }

        self.display = Some(display);

        // Build child nodes.
        for child in self.children.iter() {
            let child_ent = vc.world.spawn(ViewHandle {
                view: child.clone(),
            });
            self.child_entities.push(child_ent.id());
            let child_view = child_ent.get::<ViewHandle>().unwrap();
            let child_inner = child_view.view.clone();
            child_inner.lock().unwrap().build(child_ent.id(), vc);
        }

        self.attach_children(vc.world);
    }

    fn raze(&mut self, view_entity: Entity, world: &mut World) {
        assert!(self.display.is_some());
        // Raze all child views
        for child_ent in self.child_entities.drain(..) {
            let child = world.entity_mut(child_ent);
            let handle = child.get::<ViewHandle>().unwrap();
            let inner = handle.view.clone();
            inner.lock().unwrap().raze(child_ent, world);
            world.entity_mut(child_ent).despawn();
        }

        // Delete all reactions.
        world.entity_mut(view_entity).despawn_recursive();

        // Delete the display node.
        world.entity_mut(self.display.unwrap()).despawn();
        self.display = None;
    }
}

impl<B: Bundle + Default> IntoView for Element<B> {
    fn into_view(self) -> ViewRef {
        Arc::new(Mutex::new(self))
    }
}
